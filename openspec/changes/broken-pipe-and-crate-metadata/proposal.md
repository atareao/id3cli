# Proposal

## Why

Este cambio agrupa **tres problemas independientes** que el usuario ha pedido tratar juntos: un fallo de comportamiento del CLI, una laguna en la ficha del paquete en crates.io y una desincronización del `Cargo.lock` tras la release. No comparten causa ni código, pero sí una misma rama y un mismo cierre.

**1. El cierre del pipe de salida provoca un pánico.** Si el consumidor cierra el extremo de lectura de la salida estándar antes de que el CLI termine de escribir, el proceso entra en pánico en lugar de terminar con normalidad. La evidencia medida hoy con el binario publicado de la 0.5.0 es concluyente:

- `id3cli show fichero.m4a | head -1` escribe `failed printing to stdout: Broken pipe (os error 32)` por salida estándar, termina con código de salida **101** y emite por stderr la nota «run with RUST_BACKTRACE=1…».
- `id3cli show fichero.mp3 > /dev/full` (dispositivo que rechaza toda escritura con `ENOSPC`) reproduce el mismo pánico y el mismo código **101**.

El mismo comportamiento se reprodujo con el binario de la **v0.4.3**, así que **no es una regresión de la 0.5.0**: viene de `println!`, que hace `unwrap` del resultado de escribir en la salida estándar. La causa de fondo es que Rust pone `SIGPIPE` a `SIG_IGN` por defecto, de modo que la escritura devuelve `ErrorKind::BrokenPipe` en vez de matar el proceso, y `println!` convierte ese error en un `panic!`. La consecuencia observable es que cualquier guion que canalice la salida del CLI a `head`, `grep` o similar —o que vuelque a un destino que se llena— rompe con un código de error espurio.

**2. La ficha del paquete en crates.io está vacía.** `Cargo.toml` no declara `keywords` ni `categories`, y la API de crates.io devuelve `"keywords":[]` y `"categories":[]` para la crate. La crate es publicable, pero su ficha no aporta ninguna señal de descubrimiento ni clasificación.

**3. El `Cargo.lock` quedó desincronizado tras la release.** El commit de release de la 0.5.0 (`9681456`, mensaje `chore: release v0.5.0`) tocó solo `.vampus.yml`, `CHANGELOG.md` y `Cargo.toml`: **no tocó `Cargo.lock`**. Como consecuencia, hoy tanto `main` como `development` declaran `version = "0.4.3"` en la entrada `id3cli` del lock mientras `Cargo.toml` dice `0.5.0`, así que cualquier orden de `cargo` sobre un árbol limpio reescribe el lock y deja el árbol de trabajo sucio sin que el desarrollador haya tocado nada. La solución adoptada es commitear el lock refrescado en este change y añadir al pipeline de release un paso que lo refresque antes del commit del release.

## What Changes

El contrato de este cambio tiene cuatro puntos:

- **Pipe cerrado es un final normal de la salida.** Cuando la escritura en la salida estándar falla con `ErrorKind::BrokenPipe`, el CLI MUST terminar con código de salida **cero** y MUST NOT escribir nada por stderr: ni pánico, ni aviso, ni nota de backtrace. Cerrar el pipe deja de ser un error.
- **Un fallo real de escritura sigue siendo un error.** Cuando la escritura falla por una causa distinta del pipe cerrado (por ejemplo `ENOSPC` al volcar a `/dev/full`), el CLI MUST informar en español por stderr y MUST terminar con código de salida distinto de cero.
- **Ficha válida en el manifiesto.** `Cargo.toml` MUST declarar `keywords` y `categories` con valores que crates.io acepte, de modo que `cargo publish` no falle y la ficha deje de estar vacía.
- **El lock queda con la versión publicada.** El commit de release MUST dejar la entrada del propio paquete en `Cargo.lock` con la misma versión que `Cargo.toml`, y el pipeline de release MUST refrescar el lock **antes** de crear ese commit, de modo que un `cargo` en limpio MUST NOT ensuciar el árbol de trabajo.

**Consecuencia observable:** el **único** cambio de comportamiento del CLI es que cerrar el pipe deja de producir un pánico y un código 101, y pasa a producir un cierre limpio con código 0. **Los flags, los mensajes, los avisos por stderr y el resto de códigos de salida MUST NOT cambiar.** Los `eprintln!` (24 usos: avisos y errores) MUST NOT cambiar de flujo ni de códigos de salida. La adición de metadatos al manifiesto no altera en absoluto el comportamiento del binario.

## Capabilities

### New Capabilities

- `cli-output`: contrato de escritura de la salida estándar del CLI. Distingue el cierre del pipe (final normal, código cero, sin mensajes) del fallo real de escritura (error en español por stderr, código distinto de cero), y fija que la salida de avisos y errores por stderr permanece en su flujo y códigos actuales. El delta se materializará en `specs/cli-output/spec.md`.
- `crate-metadata`: contrato de los metadatos del manifiesto destinados a crates.io. Fija que `Cargo.toml` declara un conjunto válido y no vacío de `keywords` (máximo cinco, minúsculas ASCII, hasta 20 caracteres) y de `categories` (slugs existentes en la taxonomía de crates.io). Incluye además la consistencia entre `Cargo.lock` y el manifiesto tras una release: el lock MUST declarar la misma versión que el paquete y el pipeline MUST refrescarlo antes del commit de release, de modo que un `cargo` en limpio MUST NOT ensuciar el árbol. El delta se materializará en `specs/crate-metadata/spec.md`.

### Modified Capabilities

Ninguna. Este cambio no toca el comportamiento de ningún formato de audio: no modifica la detección de formato (`format-detection`), ni la edición de MP3 (`mp3-tagging`), ni la de MP4 (`m4a-tagging`). El pánico del pipe no es un comportamiento contratado por ninguna de esas capacidades —no está documentado en ninguna spec—, sino un defecto de la capa de salida que este cambio caracteriza por primera vez en una capacidad nueva. Del mismo modo, la ficha de crates.io no pertenece a ninguna capacidad existente. Por eso ambos deltas se crean como capacidades nuevas en lugar de modificar las tres ya existentes.

## Impact

- `src/output.rs`: **archivo nuevo**. Contiene el macro `outln!` y la función que escribe por `io::stdout()` y clasifica el error de escritura (pipe cerrado → cierre limpio; resto → error en español).
- `src/lib.rs`: se declara el módulo `output` y **los 54 `println!`** de este archivo se renombran a `outln!`. Los 4 `eprintln!` quedan intactos.
- `src/main.rs`: se importa el macro `outln!` y **los 54 `println!`** de este archivo se renombran a `outln!`. Los 20 `eprintln!` quedan intactos.
- `src/tests.rs`: se añaden tests unitarios del manifiesto y del lock (lectura de `Cargo.toml` y de `Cargo.lock` mediante `include_str!`): uno comprueba la forma y los valores de `keywords`, otro los slugs de `categories` y otro que la versión del lock coincide con la del manifiesto.
- `tests/integration_test.rs`: se añaden tres tests de integración: el del pipe cerrado sobre MP3, el del pipe cerrado sobre M4A y el del fallo real de escritura (`/dev/full`).
- `Cargo.toml`: se añaden `keywords` y `categories` al bloque `[package]`.
- `Cargo.lock`: se commitea la versión refrescada, de modo que la entrada `id3cli` pase de `0.4.3` a `0.5.0` y coincida con el manifiesto.
- `.github/workflows/release-prepare.yml`: paso nuevo que refresca el lock antes del commit de release, para que el lock entre en ese mismo commit.
- `README.md` y `AGENTS.md`: se documentan el nuevo módulo de salida y los metadatos del paquete.
- **Sin dependencias nuevas.** Ni de producción ni de desarrollo: ni `libc` ni ninguna otra entra en el árbol. La solución se apoya solo en `std`.
