# Design

## Context

Este cambio tiene tres partes independientes: el tratamiento del cierre del pipe de salida, la declaración de metadatos del paquete para crates.io y la consistencia de `Cargo.lock` con el manifiesto tras una release. La motivación de las tres está en `proposal.md`; aquí no se repite. Este documento fija cómo se resuelve cada una, qué alternativas se descartaron y por qué, y cómo se verificará.

Hechos de referencia sobre los que se construye el diseño, medidos y comprobados en vez de supuestos:

- **El pánico es inherente a `println!`, no una regresión.** Rust pone `SIGPIPE` a `SIG_IGN` al arrancar, así que una escritura sobre un pipe cerrado no mata el proceso: devuelve `ErrorKind::BrokenPipe`. `println!` hace `unwrap` del resultado de esa escritura y, al recibir el error, entra en pánico. El código de salida resultante es **101** y por stderr sale la nota de backtrace. Se reprodujo igual con los binarios publicados de la **0.5.0** y de la **0.4.3**, de modo que el defecto precede a esta release.
- **Un fallo real de escritura comparte el mismo síntoma.** Volcar a `/dev/full` (que rechaza toda escritura con `ENOSPC`) produce el mismo pánico y el mismo 101, pero su causa **no** es un pipe cerrado. Cualquier diseño MUST distinguir ambos casos: tratar todo error de escritura como final normal convertiría un disco lleno en un éxito silencioso.
- **Convención de los CLI modernos.** Medido en esta máquina con `| head -1`, `cat`, `rg`, `fd` y `cargo` (los tres últimos escritos en Rust) terminan con código **0**; `ls` y `git` terminan con **141** (muerte por `SIGPIPE`). Es decir, los CLI modernos —y en particular los escritos en Rust— tratan el pipe cerrado como un final normal de la salida.
- **`libc` no está en el árbol.** `cargo tree -e normal` no muestra `libc` y no aparece en `Cargo.lock`, así que cualquier vía que dependa de `libc` obligaría a introducir una dependencia nueva que el repositorio exige justificar.
- **Superficie de impresión acotada.** El proyecto tiene **108** puntos de impresión por salida estándar: 54 `println!` en `src/lib.rs` y 54 en `src/main.rs`. No existe ningún `print!` sin salto de línea y la salida estándar no se escribe por ningún otro sitio. Los avisos y errores van por `eprintln!` (**24** usos: 4 en `src/lib.rs` y 20 en `src/main.rs`) y no se tocan.
- **La ficha de crates.io está vacía.** `Cargo.toml` no declara `keywords` ni `categories`, y la API de crates.io devuelve `"keywords":[]` y `"categories":[]` para la crate.
- **`keywords` y `categories` se validan en el servidor.** Un slug de categoría inexistente hace fallar `cargo publish` con un error del servidor, así que la lista exacta importa y MUST verificarse contra la taxonomía real, no inventarse.
- **El commit de release no refresca `Cargo.lock`.** El commit `9681456` (mensaje `chore: release v0.5.0`) tocó **solo** `.vampus.yml`, `CHANGELOG.md` y `Cargo.toml`: no tocó `Cargo.lock`. Como consecuencia, hoy `main` y `development` (que el pipeline sincroniza) declaran `version = "0.4.3"` en la entrada `id3cli` del lock frente a `0.5.0` en el manifiesto, así que cualquier orden de `cargo` sobre un árbol recién clonado reescribe el lock y deja el árbol de trabajo sucio sin que el desarrollador haya tocado nada. Esta es la **tercera** parte del cambio y también es independiente de las otras dos.

## Goals / Non-Goals

**Goals:**

- Que cerrar el pipe de salida sea un final normal: código 0 y silencio absoluto por stderr.
- Que un fallo real de escritura siga siendo un error: mensaje en español por stderr y código de salida distinto de cero.
- Resolverlo **sin añadir ninguna dependencia** y sin cambiar ningún flag, mensaje ni código de salida que no sea el del pipe cerrado.
- Declarar en `Cargo.toml` un conjunto válido de `keywords` y `categories` para la ficha de crates.io.
- Cubrir las tres cosas con tests: tres de integración (pipe cerrado sobre MP3, pipe cerrado sobre M4A y fallo real de escritura), y tres unitarios para el manifiesto y el lock (keywords, categories y coincidencia de versiones).
- Que `Cargo.lock` quede en el mismo commit que la subida de versión, de modo que un `cargo` en limpio no ensucie el árbol de trabajo.

**Non-Goals:**

- Cambiar el flujo o los códigos de los avisos y errores que hoy van por `eprintln!`.
- Unificar todas las funciones de impresión de `src/lib.rs` en una API que devuelva `String` (ver la decisión 1).
- Muerte por `SIGPIPE` con código 141, que es justo lo que se quiere evitar.
- Tocar la detección de formato o la edición de MP3 o MP4: este cambio no altera el comportamiento de ningún formato.
- Publicar la crate: este cambio solo deja el manifiesto listo para ello.

## Decisions

### 1. Un macro `outln!` en `src/output.rs` que trata el error de escritura, en lugar de restaurar `SIGPIPE`, de un hook de pánico o de devolver `String`

La decisión es interceptar **el error de escritura en el punto donde se produce**. Se añade un módulo nuevo `src/output.rs` con un macro `outln!`, equivalente a `println!`, que delega en una función que escribe con `io::stdout()` y clasifica el error:

- `ErrorKind::BrokenPipe` → `std::process::exit(0)` sin escribir nada, porque el consumidor ya no está escuchando y cerrar el pipe es un final normal de la salida.
- Cualquier otro error de escritura → mensaje en español por stderr y `process::exit(1)`, porque un destino que falla (por ejemplo un disco lleno) es un error real que no puede pasar por un éxito.

Se elige esta vía por tres razones concretas: no añade dependencias (solo usa `std`), no cambia ningún flujo de `stderr` y no obliga a tocar la API pública de `lib.rs`.

**Alternativa descartada: restaurar `SIGPIPE` a su disposición por defecto** con `libc::signal(SIGPIPE, SIG_DFL)`, que es lo que hacen `ls` y `git`. Serían unas tres líneas y resuelve el caso del pipe, pero se descarta por dos motivos. Primero, obliga a añadir la dependencia `libc`, que **hoy no está en el árbol** (`cargo tree -e normal` no la muestra y no aparece en `Cargo.lock`), y el repositorio exige justificar cada dependencia nueva. Segundo, el proceso moriría con código **141**, lo que rompe cualquier guion con `set -o pipefail`; es precisamente el comportamiento de `ls` y `git` que se quiere evitar, y lo contrario de la convención que siguen `rg`, `fd` y `cargo`.

**Alternativa descartada: un hook de pánico que reconozca el mensaje.** `std::panic::set_hook` podría inspeccionar el texto del pánico y salir con 0 cuando mencione el pipe. Se descarta porque depende de un texto de la biblioteca estándar que no es API pública estable: es frágil y es un apaño, no un tratamiento del error.

**Alternativa descartada: convertir `display_tags()` y `display_tags_mp4()` en funciones que devuelvan `String`.** Resolvería el pánico de esas dos funciones devolviendo el texto y dejando que quien lo imprima trate el error. Se descarta por parcial y por invasiva: cambiaría la API pública y obligaría a adaptar los 6 tests unitarios que las llaman, y dejaría sin cubrir el resto de mensajes del CLI, que seguirían usando `println!` y seguirían paniqueando. La solución elegida cubre los 108 puntos de impresión sin tocar ninguna firma.

### 2. El renombrado de los 108 puntos de impresión es mecánico y completo; `eprintln!` queda intacto

La implementación concreta es un renombrado de `println!` a `outln!` en los **108** puntos del proyecto (54 en `src/lib.rs` y 54 en `src/main.rs`) más la declaración del módulo y su importación. En `src/lib.rs` se declara el módulo `output`; en `src/main.rs` se importa el macro. La operación es puramente mecánica porque el proyecto **no tiene ningún `print!` sin salto de línea** y la salida estándar no se escribe por ningún otro sitio: el renombrado es completo por construcción, no por muestreo.

Los **24 `eprintln!`** (avisos y errores) **MUST NOT** tocarse: su flujo es stderr y su contrato de códigos de salida no forma parte de este cambio. Igualmente, `outln!` MUST NOT redirigir nada a stderr.

Detalle de comportamiento a conservar: `process::exit` no ejecuta destructores ni vacía búferes pendientes, pero la garantía relevante aquí es por punto de impresión —cada `outln!` escribe y comprueba el resultado de **esa** escritura—, de modo que el cierre limpio en caso de `BrokenPipe` es correcto y el mensaje de error real se emite por stderr antes de salir con código 1. El macro MUST preservar el formato de `println!` (argumentos de formato y salto de línea final) para que ningún mensaje existente cambie.

### 3. `keywords` con cinco valores, el máximo permitido, todos válidos

Se declara `keywords = ["id3", "mp3", "mp4", "metadata", "cli"]`.

Son cinco, que es el **máximo** que admite crates.io, de modo que la ficha aprovecha toda su capacidad. Todos están en minúsculas ASCII y tienen 20 caracteres o menos (`metadata` es el más largo, con 8), que son las dos restricciones que crates.io aplica a cada palabra clave. Describen la crate por sus dos ejes: el dominio (`id3`, `mp3`, `mp4`, `metadata`) y su forma de entrega (`cli`).

### 4. `categories` con tres slugs verificados, en lugar de slugs inventados

Se declara `categories = ["command-line-utilities", "multimedia::audio", "filesystem"]`.

Los tres slugs se verificaron hoy contra la taxonomía real, porque un slug inexistente haría **fallar `cargo publish`** con un error del servidor y la lista exacta es, por tanto, parte del contrato:

- `command-line-utilities` y `filesystem` aparecen en `https://crates.io/api/v1/categories`.
- `multimedia::audio` es la categoría que declara la propia crate `lofty` (0.25.4), que es el backend de tags del proyecto. Se verifica contra una crate publicada y no contra una suposición.

Se descarta inventar categorías adicionales o slugs plausibles pero no comprobados: el coste de un slug inválido es un fallo de publicación en el peor momento, y el beneficio de añadir uno de más es nulo.

### 5. Verificación: tres tests de integración para el pipe y el fallo real, y tests unitarios para el manifiesto y el lock

Cada garantía del contrato se verifica de forma independiente:

- **Pipe cerrado.** Un test de integración lanza el binario con la salida estándar canalizada (`Stdio::piped()`) y **cierra el extremo de lectura antes de leer nada**, lo que fuerza `BrokenPipe` en la primera escritura. El test MUST comprobar que el proceso termina con código **0** y que stderr queda **vacío** (ni pánico, ni nota de backtrace).
- **Fallo real de escritura.** Un test de integración abre `/dev/full` y lo usa como salida estándar del proceso. El test MUST comprobar que el proceso termina con código **distinto de cero** y que por stderr sale un mensaje en español. Como `/dev/full` es específico de Linux, el test se protege para ejecutarse solo donde exista y no reventar la suite en otras plataformas.
- **Manifiesto.** Tests unitarios en `src/tests.rs` leen `Cargo.toml` mediante `include_str!` (sin depender del directorio de trabajo) y comprueban que `keywords` tiene cinco entradas válidas (minúsculas ASCII, longitud ≤ 20) y que `categories` contiene exactamente los tres slugs acordados. Verificar el contrato del manifiesto en un test evita que una edición futura deje la ficha vacía o inválida en silencio.

El resto de la suite existente hace de red de seguridad de la parte no tocada: ningún test de comportamiento de formato MUST cambiar como consecuencia de este cambio.

### 6. El lock se refresca en el propio commit de release, y se verifica con un test que compara las dos versiones

**Causa.** El pipeline (`release-prepare.yml`) sube la versión con vampus, regenera el changelog y hace `git add -A`, pero nada refresca `Cargo.lock` antes de ese `add`, así que el lock no entra en el commit de release y queda desincronizado del manifiesto.

**Decisión.** Se ataca por dos vías complementarias:

- Añadir al pipeline, en `release-prepare.yml` y **antes** del commit de release, un paso que refresque el lock ejecutando `cargo metadata --format-version 1 > /dev/null`: el comando realiza la resolución y reescribe `Cargo.lock`, de modo que el `git add -A` posterior ya lo incluye. Así el lock no vuelve a quedarse atrás.
- Commitear en este change el `Cargo.lock` ya refrescado, que arregla el estado actual de `main` y `development`.

**Verificación.** Un test unitario lee los dos archivos con `include_str!` (sin depender del directorio de trabajo) y compara la versión declarada para el paquete en el lock con el campo `version` del manifiesto, para que la desincronización se detecte en CI en lugar de en el árbol de alguien. A ello se añade la comprobación local del comando: revertir el lock a `0.4.3`, ejecutar `cargo metadata --format-version 1 > /dev/null` y ver que vuelve a `0.5.0`.

**Alternativa descartada: descartar el lock del control de versiones** añadiéndolo a `.gitignore`, que es lo que hacen algunas bibliotecas. Se descarta porque este proyecto publica un **binario** y quiere compilaciones reproducibles con las versiones de dependencias fijadas; sin el lock versionado, cada build resolvería versiones distintas.

## Risks / Trade-offs

- **`/dev/full` no existe fuera de Linux** → el test de fallo real se condiciona a la existencia del dispositivo (`cfg(target_os = "linux")` y comprobación en tiempo de ejecución), de modo que en otras plataformas la suite sigue verde sin fingir cobertura que no puede dar. El caso `BrokenPipe`, que es el importante, es portable.
- **Un `process::exit` evita destructores y buffers** → se asume como parte del diseño: en el caso de pipe cerrado no hay nada que conservar y en el de error real el mensaje ya se emitió por stderr. `outln!` comprueba el resultado por escritura, así que ninguna salida se pierde en silencio.
- **Riesgo de renombrado a medias** → la superficie está acotada y es verificable: 108 `println!` y cero `print!`. La revisión puede comprobar por búsqueda que no queda ningún `println!` en `src/lib.rs` ni en `src/main.rs`, y que los 24 `eprintln!` siguen intactos.
- **El código 0 ante `BrokenPipe` oculta errores de escritura reales si la clasificación de `ErrorKind` fuese demasiado amplia** → la clasificación es **exacta**: solo `ErrorKind::BrokenPipe` cierra con 0; cualquier otra variante de `ErrorKind` es un error. Esto es precisamente lo que separa el pipe cerrado (`/dev/full` da `ENOSPC`, no `BrokenPipe`) y lo que cubren por separado los tests de integración del pipe cerrado y el del fallo real.
- **La ficha de crates.io depende de la taxonomía del servidor** → se mitiga verificando los tres slugs contra la API y contra una crate publicada; si crates.io retirara un slug en el futuro, `cargo publish` lo señalaría y la corrección es de una línea.
- **Sin dependencias nuevas** → no hay descarga adicional ni superficie de mantenimiento: toda la solución usa `std`, y el renombrado no cambia la firma de ninguna función.

## Migration Plan

El orden es estricto y cada paso se cierra con la suite verde.

1. **Añadir el módulo de salida.** Crear `src/output.rs` con el macro `outln!` y la función que clasifica el error, declarar el módulo en `src/lib.rs` e importar el macro en `src/main.rs`.
2. **Renombrar.** Sustituir los 108 `println!` por `outln!` (54 en `src/lib.rs` y 54 en `src/main.rs`) sin tocar los 24 `eprintln!` ni ningún mensaje. Al final de este paso, el comportamiento observable es idéntico salvo por el pipe cerrado.
3. **Añadir la verificación del pipe.** Los tres tests de integración (pipe cerrado sobre MP3, pipe cerrado sobre M4A y `/dev/full`) en rojo antes de la implementación del paso 1 y en verde después; aquí se encierra el contrato.
4. **Añadir los metadatos del manifiesto y arreglar el lock.** Declarar `keywords` y `categories` en `Cargo.toml`, refrescar `Cargo.lock` con `cargo metadata --format-version 1 > /dev/null` para que su versión coincida con la del manifiesto, añadir el paso equivalente al pipeline de release y añadir los tests unitarios que leen el manifiesto y el lock con `include_str!`.
5. **Actualizar la documentación.** `README.md` y `AGENTS.md`: el módulo de salida, el contrato del pipe cerrado y los metadatos del paquete.

**Rollback:** el proyecto sigue Git Flow, así que este cambio se desarrolla en la rama `fix/broken-pipe-and-crate-metadata` sobre `development` y no se mezcla hasta que la suite completa está verde. Las tres partes son independientes entre sí, de modo que revertir la del pipe no afecta a la de metadatos ni a la del lock, ni al contrario.
