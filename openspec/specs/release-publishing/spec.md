# release-publishing Specification

## Purpose
El pipeline de release empuja `main` y el tag en dos `git push` consecutivos dentro del mismo paso, lo que abre una ventana de carrera entre el commit de release y su referencia: el 2026-10-14 el run `Prepare Release` 38527464056 aceptó `main` (`8fb0583..5a070b6`) y rechazó el tag con `! [remote rejected] v0.5.1 -> v0.5.1 (missing necessary objects)`, dejando la versión ya subida sin tag y `release.yml` sin disparar. En paralelo, `cargo package --list` publica 54 ficheros, incluido todo el andamiaje interno del repositorio (`.github/**`, `openspec/**`, `AGENTS.md`). Este delta fija dos contratos nuevos: el envío atómico con reintentos acotados y el contenido publicable del paquete mediante una lista blanca `include` con los patrones de primer nivel anclados a la raíz, porque medir el paquete mostró que un patrón sin anclar casa a cualquier profundidad y dejaba viajar un fichero del andamiaje.

Nota razonada: el `--locked` de la integración continua y el refresco de `Cargo.lock` antes del commit de release ya están contratados en la capability `crate-metadata` y NO se mueven aquí; esta capability cubre el push y el contenido del paquete, que son contratos nuevos e independientes del lock. El solapamiento es deliberado: `release-publishing` consume el estado que `crate-metadata` garantiza, sin reabrirlo.

## Requirements

### Requirement: El pipeline de release MUST empujar la rama y el tag en una única operación atómica

El paso `Commit, tag and push` de `.github/workflows/release-prepare.yml` MUST actualizar `main` y el tag `v${V}` con una sola invocación de `git push --atomic` que incluya las dos referencias, de modo que el objeto del commit de release y la referencia del tag viajen en el mismo pack. MUST NOT existir ventana entre el envío de la rama y el del tag, y el contrato MUST NOT admitir el estado parcial en que `main` queda actualizado y el tag no existe. Si el envío no lo consigue tras reintentos acotados, el pipeline MUST fallar con error explícito y MUST NOT disparar la publicación.

#### Scenario: Un único push que lleva las dos referencias

- **WHEN** el pipeline de release empuja la rama `main` y el tag `v${V}` recién creados
- **THEN** MUST hacerlo con una sola invocación de `git push --atomic` que incluya las dos referencias, de modo que el commit de release y la referencia del tag viajen en el mismo pack y MUST NOT existir ventana entre ambos; la medición del 2026-10-14 dejó constancia del fallo del montaje anterior: dos `git push` separados por 0,49 s en el run 38527464056, frente a 1,1 s y 1,3 s en los dos runs que funcionaron (release 0.5.0 y v0.4.3), con comando idéntico

#### Scenario: Fallo total en vez de parcial

- **WHEN** una de las dos referencias del push atómico es rechazada
- **THEN** ninguna MUST actualizarse y el servidor MUST rechazar las dos; el experimento del 2026-10-14 contra este remoto con refs desechables MUST reproducir ese contrato: el push atómico de una rama non-FF junto con un tag nuevo devolvió `! [rejected] 5ba4e85 -> prueba/atomic-ff (non-fast-forward)` y `! [rejected] prueba-atomic-ff -> prueba-atomic-ff (atomic push failed)`, el tag NO se creó y la rama quedó intacta, mientras que el mismo push sin `--atomic` sí creó el tag y solo rechazó la rama. El contrato MUST NOT admitir el estado en que `main` queda actualizado y el tag no existe

#### Scenario: El camino normal MUST seguir funcionando

- **WHEN** el push atómico lleva dos referencias válidas y ninguna es rechazable
- **THEN** el servidor MUST aceptar las dos y el comando MUST terminar con código 0; el experimento del 2026-10-14 contra este remoto con refs desechables lo midió: un push atómico de una rama nueva junto con un tag nuevo aceptó ambos (`* [new branch]` y `* [new tag]`) con `exit 0`, de modo que la atomicidad no rompe el camino que ya funcionaba

#### Scenario: Reintentos acotados

- **WHEN** el push atómico falla
- **THEN** el pipeline MUST reintentarlo hasta tres intentos separados cinco segundos antes de darse por vencido

#### Scenario: Estado limpio ante el fallo

- **WHEN** el pipeline agota los tres intentos sin empujar
- **THEN** MUST terminar con un `::error::` y salida distinta de cero, MUST NOT ejecutar `gh workflow run`, y la rama `main` remota MUST quedar sin el commit de release y sin el tag

#### Scenario: Sin doble bump

- **WHEN** un envío falla y hay que recuperar el release
- **THEN** la recuperación MUST ser volver a empujar a `main` desde el mismo estado, y el pipeline MUST NOT necesitar ni admitir como recuperación la reejecución del job fallido, que recalcularía la versión desde el estado actual (`vampus upgrade --patch` desde 0.5.1 daría 0.5.2)

### Requirement: El paquete publicado MUST limitarse a los ficheros publicables del proyecto

`Cargo.toml` MUST declarar en `[package]` una clave `include` con la lista blanca de los ficheros publicables: `/src/**`, `/tests/**`, `/Cargo.toml`, `/Cargo.lock`, `/README.md`, `/LICENSE` y `/CHANGELOG.md`, con los patrones de primer nivel anclados a la raíz. El andamiaje interno del repositorio MUST NOT viajar en el paquete, y el paquete MUST seguir siendo verificable con `cargo publish --dry-run` conservando la metadata ya declarada.

#### Scenario: Lista blanca declarada

- **WHEN** se lee `Cargo.toml`
- **THEN** `[package]` MUST declarar una clave `include` con la lista blanca de patrones anclados `/src/**`, `/tests/**`, `/Cargo.toml`, `/Cargo.lock`, `/README.md`, `/LICENSE` y `/CHANGELOG.md`

#### Scenario: Los patrones de primer nivel MUST ir anclados a la raíz

- **WHEN** se declara un patrón de primer nivel en `include` sin barra inicial
- **THEN** MUST NOT acotarse a la raíz del proyecto, porque `include` usa la semántica de gitignore y el patrón casa a cualquier profundidad; la medición del 2026-10-14 lo dejó escrito: con la lista sin anclar `cargo package --list` daba **16** entradas pero incluía `openspec/changes/archive/2026-10-14-remove-empty-id3-header/README.md`, arrastrado por el patrón `README.md`, mientras que con la lista anclada bajaba a **15** entradas y **cero** del andamiaje

#### Scenario: El andamiaje MUST NOT viajar

- **WHEN** se ejecuta `cargo package --list` con la lista blanca anclada declarada
- **THEN** `openspec/`, `.github/`, `AGENTS.md`, `GIT_FLOW.md`, `RELEASE.md`, `cliff.toml`, `.justfile` y `.vampus.yml` MUST NOT aparecer en la salida; la medición del 2026-10-14 dejó la salida en **15** entradas con **cero** ficheros del andamiaje, frente a las 54 del estado sin `include` (59 con los ficheros de este change en el árbol)

#### Scenario: Lo necesario MUST seguir

- **WHEN** se ejecuta `cargo package --list` con la lista blanca anclada declarada
- **THEN** MUST seguir presente `/src/**`, `/tests/**` con sus fixtures (`sample.mp3`, `sample.m4a` y `generate_sample_m4a.sh`, 9.031 bytes en total: 2.481, 3.295 y 3.255), `README.md`, `LICENSE`, `CHANGELOG.md` y `Cargo.lock`, de modo que el paquete publicado conserve su suite ejecutable; la medición del 2026-10-14 dio **15** entradas —13 del repositorio, con `Cargo.lock`, más 2 que genera cargo—, y `Cargo.lock` MUST viajar porque el paquete publica un binario y la compilación debe ser reproducible con las versiones del lock

#### Scenario: El paquete MUST seguir verificándose

- **WHEN** se ejecuta `cargo publish --dry-run` con la lista blanca anclada declarada
- **THEN** MUST empaquetar y MUST verificar la compilación del paquete, y MUST NOT perder las `keywords` ni las `categories` ya declaradas en el manifiesto; la medición del 2026-10-14 dejó constancia de que el empaquetado produce **15** ficheros (331.0KiB, 74.0KiB comprimidos), verifica la compilación y termina con `warning: aborting upload due to dry run`, y de que el manifiesto normalizado dentro del `.crate` conserva `include` anclado, `keywords` y `categories`

### Requirement: El binario precompilado MUST ser un ejecutable estático y MUST NOT requerir la libc del sistema

El job `build` de `.github/workflows/release.yml` MUST construir el target `x86_64-unknown-linux-musl` con `cargo build --release --locked` y MUST publicar el binario resultante, de modo que el artefacto sea un ejecutable estático: MUST NOT declarar entradas `NEEDED` en su cabecera ELF y MUST NOT referenciar símbolos versión `GLIBC_*`. El pipeline MUST verificar esa condición antes de publicar y MUST fallar, sin publicar, si el binario no es estático.

#### Scenario: Sin dependencias dinámicas

- **WHEN** se inspecciona el binario publicado con `readelf -d` y `ldd`
- **THEN** `readelf -d` MUST NOT listar ninguna entrada `NEEDED`, y `ldd` MUST responder `not a dynamic executable` o `statically linked`; la medición del 2026-10-25 sobre el artefacto gnu de `target/release/id3cli` dejó constancia del estado que se corrige: cuatro entradas `NEEDED` —`libgcc_s.so.1`, `libm.so.6`, `libc.so.6` y `ld-linux-x86-64.so.2`

#### Scenario: Sin símbolos glibc

- **WHEN** se listan los símbolos versionados del binario con `objdump -T`
- **THEN** MUST NOT aparecer ningún símbolo con versión `GLIBC_*`; la medición del 2026-10-25 sobre el artefacto gnu dio como versión más alta `GLIBC_2.34`, exigida por `__libc_start_main`, `pthread_attr_getguardsize`, `pthread_attr_getstack`, `pthread_key_create`, `pthread_key_delete` y `pthread_setspecific`

#### Scenario: Ejecutable sin la glibc del build

- **WHEN** el binario se ejecuta en un sistema sin la glibc de la máquina que lo construyó (un contenedor musl/alpine)
- **THEN** MUST arrancar y MUST producir la misma salida que en un sistema con glibc para `--help` y `show` sobre un fixture; medido el 2026-10-25 en un contenedor `alpine` (musl, sin glibc): el binario estático ejecutó `show` con código 0, mientras el artefacto gnu no llegó a arrancar (`sh: ...: not found`, le falta el cargador de glibc)

### Requirement: El release MUST publicar el binario desnudo además del archivo comprimido

Los assets de la release MUST incluir el binario desnudo con el nombre `id3cli-linux-x86_64` —ejecutable, con su bit de ejecución— además de `id3cli-x86_64-unknown-linux-musl.tar.gz` y `id3cli-x86_64-unknown-linux-musl.tar.gz.sha256`, de modo que la ruta documentada en el README MUST existir y MUST devolver el binario.

#### Scenario: Asset desnudo presente

- **WHEN** se listan los assets de la release publicada
- **THEN** MUST aparecer `id3cli-linux-x86_64` junto al `.tar.gz` y a su `.sha256`; la medición del 2026-10-25 sobre v0.10.0 y v0.9.0 dejó constancia del estado que se corrige: solo `id3cli-x86_64-unknown-linux-gnu.tar.gz` y `id3cli-x86_64-unknown-linux-gnu.tar.gz.sha256`

#### Scenario: La ruta documentada responde

- **WHEN** se descarga `https://github.com/atareao/id3cli/releases/latest/download/id3cli-linux-x86_64`
- **THEN** MUST devolver el binario con HTTP 200, de modo que el comando de instalación del README —`wget`, `chmod +x`, `mv`— MUST funcionar tal cual

#### Scenario: El archivo y su suma siguen

- **WHEN** se listan los assets
- **THEN** MUST seguir presente el `.tar.gz` con su `.sha256` correspondiente, de modo que la descarga auditable no se pierda

### Requirement: La integración continua MUST construir y verificar el binario estático antes del tag

`.github/workflows/ci.yml` MUST incorporar un job que construya `x86_64-unknown-linux-musl` con `--locked`, MUST verificar el enlazado estático del binario y MUST ejecutar la suite completa contra ese target, de modo que una pérdida del estático o una regresión del binario musl MUST fallar en la pull request y MUST NOT descubrirse al publicar el tag. El job preexistente sobre gnu MUST seguir en verde.

#### Scenario: Job musl en la puerta de entrada

- **WHEN** se abre o se actualiza una pull request
- **THEN** MUST ejecutarse un job que construya musl, verifique el estático y corra `cargo test --locked --target x86_64-unknown-linux-musl`, además del job gnu existente

#### Scenario: Falsabilidad del gate

- **WHEN** el binario deja de ser estático —medido el 2026-10-25: el gate falla sobre el artefacto gnu, con 4 entradas `NEEDED` y `GLIBC_2.34` como versión más alta, y pasa sobre el musl, con 0 y 0; forzar la pérdida del estático con `-C target-feature=-crt-static` ni siquiera enlaza, porque el target de rustup solo trae `libc.a` estática
- **THEN** la verificación MUST fallar con salida distinta de cero y MUST impedir la publicación

#### Scenario: Sin regresión en el job gnu

- **WHEN** se ejecuta la integración continua con el cambio aplicado
- **THEN** el job sobre `x86_64-unknown-linux-gnu` (fmt, clippy, build, test) MUST seguir en verde

### Requirement: La salida del binario estático MUST ser idéntica a la del binario dinámico de referencia

El binario musl y el binario gnu de la misma versión MUST producir salidas byte a byte idénticas para `show` (legible) y `show --json` sobre los mismos fixtures, de modo que el cambio de enlace MUST NOT alterar ningún contrato observable del CLI.

#### Scenario: No-rotura medida sobre los cinco formatos

- **WHEN** se ejecutan `show` y `show --json` con ambos binarios sobre los fixtures mp3, m4a, ogg, opus y flac
- **THEN** la salida de los dos MUST ser byte a byte idéntica en los cinco formatos

#### Scenario: La matriz e2e MUST pasar con el binario estático

- **WHEN** se ejecuta la matriz e2e —`scripts/e2e/*.sh --bin <binario musl>`— contra el binario estático
- **THEN** MUST pasar con el mismo recuento de aserciones que contra el binario gnu
