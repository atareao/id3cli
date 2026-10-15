# Tasks

## 1. RED: fijar el contrato nuevo en los tests

El cambio tiene tres contratos independientes. El primero invierte el comportamiento de la salida estándar: cerrar el pipe deja de ser un pánico con código 101 y pasa a ser un final normal con código 0 y stderr vacío, mientras que un fallo real de escritura sigue siendo un error en español con código distinto de cero. El segundo fija los metadatos del manifiesto que crates.io debe aceptar. El tercero exige que `Cargo.lock` quede con la misma versión que el manifiesto tras la release. Este grupo fija los cinco tests que describen los tres contratos antes de tocar el código de producción.

- [x] 1.1 Añadir a `tests/integration_test.rs` el test de integración `test_cli_pipe_cerrado_no_es_error` (MP3): lanzar `show` con la salida estándar conectada a un pipe, cerrar el extremo de lectura antes de leer nada y esperar a que el proceso termine. El cierre antes de leer es deliberado: con pocos metadatos el proceso podría terminar antes de que el pipe se cierre y la prueba no distinguiría nada; cerrándolo antes de leer, el fallo es determinista.
      Verificación: `cargo test test_cli_pipe_cerrado_no_es_error` MUST fallar (hoy el proceso termina con 101 y escribe el pánico por stderr), fijando el contrato nuevo: código de salida 0 y stderr vacío (sin `panicked`, sin `failed printing to stdout` y sin la nota de `RUST_BACKTRACE`).
- [x] 1.2 Añadir a `tests/integration_test.rs` el test de integración `test_cli_pipe_cerrado_no_es_error_m4a`: mismo montaje que el anterior sobre un archivo M4A, con el extremo de lectura cerrado antes de leer nada, para cubrir la segunda ruta de visualización.
      Verificación: `cargo test test_cli_pipe_cerrado_no_es_error_m4a` MUST fallar hoy, y el contrato comprobado MUST ser idéntico al del MP3: código de salida 0 y stderr vacío.
- [x] 1.3 Añadir a `tests/integration_test.rs` el test de integración `test_cli_error_de_escritura_falla`, anotado como específico de Linux con `#[cfg(target_os = "linux")]` porque usa `/dev/full`: lanzar `show` con la salida estándar redirigida a `/dev/full`, dispositivo que rechaza toda escritura con `ENOSPC`.
      Verificación: `cargo test test_cli_error_de_escritura_falla` MUST fallar hoy, y el contrato comprobado MUST ser: código de salida distinto de cero y un mensaje en español por stderr.
- [x] 1.4 Añadir a `src/tests.rs` el test unitario `test_manifiesto_keywords_validos`, que lee `Cargo.toml` con `include_str!("../Cargo.toml")` y comprueba las reglas de crates.io: entre 1 y 5 keywords, cada una de 20 caracteres o menos, ASCII en minúsculas y solo caracteres del conjunto `[a-z0-9_-]`.
      Verificación: `cargo test test_manifiesto_keywords_validos` MUST fallar hoy (el manifiesto no declara `keywords`) y al pasar MUST verificar la forma y la cantidad de cada keyword.
- [x] 1.5 Añadir a `src/tests.rs` el test unitario `test_manifiesto_categories_validas`, que lee `Cargo.toml` con `include_str!("../Cargo.toml")` y comprueba que las categorías declaradas están entre los tres slugs verificados: `command-line-utilities`, `multimedia::audio` y `filesystem`.
      Verificación: `cargo test test_manifiesto_categories_validas` MUST fallar hoy (el manifiesto no declara `categories`) y al pasar MUST verificar que cada slug pertenece a la lista de slugs verificados.
- [x] 1.6 Ejecutar la suite completa para fijar el contrato rojo.
      Verificación: `cargo test --no-fail-fast` MUST fallar exactamente en esos 5 tests nuevos y MUST NOT fallar ninguno de los 214 tests existentes; el recuento total pasa a ser 219 (214 + 5).
- [x] 1.7 (RED) Tras restaurar `Cargo.lock` a su estado commiteado con `git checkout -- Cargo.lock`, añadir `--locked` a las órdenes de cargo de `.github/workflows/ci.yml`: a `cargo clippy --all-targets --all-features` (la primera que resuelve dependencias) y también a `cargo build` y `cargo test`. Deja escrito que MUST NOT añadirse un test de Rust que lea `Cargo.lock` del árbol de trabajo: es un falso verde por construcción, porque cargo normaliza el lock antes de compilar (medido), y por eso se retiró el que había.
      Verificación: con el lock en su estado commiteado, `cargo build --locked` MUST fallar con «cannot update the lock file … because --locked was passed» y MUST NOT reescribir el lock.

## 2. GREEN: implementar

Se implementa lo mínimo para satisfacer los tests: una capa de salida propia que clasifique el error de escritura, el renombrado de los 108 puntos de impresión a esa capa, los metadatos del manifiesto y el refresco de `Cargo.lock`. La vía del `libc` está descartada en el diseño, así que no entra ninguna dependencia nueva.

- [x] 2.1 Crear `src/output.rs` con un único macro `outln!`, equivalente a `println!`, y la función que escribe en la salida estándar tratando el error: `ErrorKind::BrokenPipe` MUST salir con código 0 y sin ningún mensaje; cualquier otro error de escritura MUST emitir un mensaje en español por stderr y salir con código distinto de cero. La declaración del módulo con `#[macro_use]` en `src/lib.rs` y su importación en `src/main.rs` MUST permitir usar el macro desde los dos crates.
      Verificación: `cargo test test_cli_pipe_cerrado_no_es_error test_cli_pipe_cerrado_no_es_error_m4a test_cli_error_de_escritura_falla` MUST pasar, y `cargo build` MUST NOT emitir ningún aviso.
- [x] 2.2 Renombrar los 108 puntos de impresión de `println!` a `outln!`: 54 en `src/lib.rs` y 54 en `src/main.rs`, dejando intactos los 24 `eprintln!` (4 en `src/lib.rs` y 20 en `src/main.rs`).
      Verificación: `grep -c 'println!' src/lib.rs src/main.rs` MUST dar 0 apariciones sin renombrar en ambos archivos (excluyendo `eprintln!`), y el diff MUST ser solo del nombre del macro, línea a línea: ningún par de líneas `-`/`+` difiere en algo más que `println!` → `outln!`.
- [x] 2.3 Añadir `keywords` y `categories` a `Cargo.toml` dentro de `[package]`, cada uno en una sola línea, con los valores fijados: `["id3", "mp3", "mp4", "metadata", "cli"]` y `["command-line-utilities", "multimedia::audio", "filesystem"]`.
      Verificación: `cargo test test_manifiesto_keywords_validos test_manifiesto_categories_validas` MUST pasar, y `cargo metadata --no-deps` MUST exponer las keywords y las categories declaradas.
- [x] 2.4 Confirmar que el árbol de dependencias no crece: la solución se apoya solo en `std` y MUST NOT añadirse ninguna dependencia nueva, ni de producción ni de desarrollo.
      Verificación: el diff de `Cargo.toml` y `Cargo.lock` MUST NOT añadir ninguna línea de dependencia; el único cambio de `Cargo.toml` MUST ser `keywords` y `categories`.
- [x] 2.5 Ejecutar la suite completa tras la implementación.
      Verificación: `cargo test --no-fail-fast` en verde con 219 tests y 0 fallos.
- [x] 2.6 Refrescar `Cargo.lock` con `cargo metadata --format-version 1 > /dev/null` (que realiza la resolución y reescribe el lock) y dejarlo listo para commitear en este mismo change.
      Verificación: `git diff Cargo.lock` MUST mostrar solo la línea de la versión del paquete, `cargo build --locked` MUST pasar, y tras ejecutar `cargo test` MUST NOT quedar ninguna modificación pendiente en el lock.
- [x] 2.7 Añadir a `.github/workflows/release-prepare.yml` un paso que refresque `Cargo.lock` **antes** del paso que hace `git add -A` y crea el commit de release, con el mismo comando (`cargo metadata --format-version 1 > /dev/null`): el pipeline MUST incluir el lock en el propio commit de release, de modo que no vuelva a quedarse atrás.
      Verificación: el YAML MUST seguir siendo válido tras el cambio; y la comprobación local del comando MUST ser reproducible: dejar el lock en `0.4.3`, ejecutar el comando y comprobar que el lock vuelve a `0.5.0`.

## 3. REFACTOR y documentación

- [x] 3.1 Documentar en `AGENTS.md` la convención de que la salida normal se escribe con `outln!` y nunca con `println!`, y que `println!` queda reservado a nada (los avisos y errores van con `eprintln!`), explicando en una línea por qué: un pipe cerrado no debe provocar un pánico.
      Verificación: `AGENTS.md` contiene la convención con la razón, y ningún `println!` queda documentado como uso permitido.
- [x] 3.2 Documentar en `README.md`, en el sitio que corresponda al comportamiento del CLI, que si el consumidor cierra la salida el CLI termina en silencio con código 0, y que un fallo real de escritura sigue siendo un error.
      Verificación: `README.md` describe ambos comportamientos (cierre del pipe → código 0 sin mensajes; fallo real → error en español con código distinto de cero).
- [x] 3.3 Comprobar el formato.
      Verificación: `cargo fmt --check` sin diferencias.
- [x] 3.4 Comprobar el linter en todos los targets y features.
      Verificación: `cargo clippy --all-targets --all-features -- -D warnings` sin warnings.

## 4. Cierre

- [x] 4.1 Ejecutar la suite completa.
      Verificación: 219 tests, 0 fallos y sin warnings del compilador.
- [x] 4.2 Validar el cambio contra OpenSpec.
      Verificación: `openspec validate broken-pipe-and-crate-metadata --strict` sin errores.
- [x] 4.3 Verificación manual de extremo a extremo del pipe cerrado sobre un MP3 real con el binario: canalizar la salida a un consumidor que lee solo la primera línea.
      Verificación: `cargo run -- show <mp3> | head -1` MUST terminar con código 0 y stderr vacío (hoy da 101 con un pánico).
- [x] 4.4 Verificación manual de extremo a extremo del pipe cerrado sobre un M4A real con el binario.
      Verificación: `cargo run -- show <m4a> | head -1` MUST terminar con código 0 y stderr vacío, igual que en el MP3.
- [x] 4.5 Verificación manual de extremo a extremo del fallo real de escritura.
      Verificación: `cargo run -- show <mp3> > /dev/full` MUST escribir un mensaje en español por stderr y dar código de salida distinto de cero.
- [x] 4.6 Comprobar que la interfaz del CLI no ha cambiado: el cambio es puramente de comportamiento de la salida y de metadatos, así que no toca flags ni mensajes.
      Verificación: la superficie de flags de `cargo run -- --help`, `cargo run -- show --help`, `cargo run -- edit --help` y `cargo run -- remove --help` MUST ser idéntica a la de antes del cambio.
- [x] 4.7 Archivar el cambio una vez verificado todo lo anterior.
      Verificación: `openspec archive broken-pipe-and-crate-metadata` listo para ejecutar; requiere la aprobación previa del usuario.
- [x] 4.8 Comprobar que la CI del PR pasa con la guardia `--locked` activa.
      Verificación: la ejecución del workflow del PR MUST mostrar las órdenes de cargo con `--locked` terminando en verde.

## Seguimiento (fuera del cierre de este cambio)

El contrato ya queda fijado en el spec; estas comprobaciones no formaban parte de las tareas de este cambio, sino de su seguimiento posterior a la publicación. Ya se han confirmado después de publicar, así que quedan cerradas y marcadas aquí, sin dejar de estar fuera del cierre del cambio.

- [x] Comprobar la ficha de crates.io tras la próxima publicación: la API de crates.io MUST devolver las keywords y las categories declaradas en lugar de las listas vacías actuales. Verificado el 2026-10-14 tras publicar la 0.5.1 y reconfirmado el 2026-10-15 con la 0.5.2: la API de crates.io devuelve ya `keywords` (`cli`, `id3`, `metadata`, `mp3`, `mp4`) y las tres `categories` (`command-line-utilities`, `multimedia::audio`, `filesystem`) en lugar de las listas vacías.
- [x] Comprobar, tras la próxima publicación, que el lock de `main` declara la misma versión que `Cargo.toml` y que un `cargo` en limpio no ensucia el árbol: en `main`, la entrada `id3cli` de `Cargo.lock` MUST declarar la misma versión que `Cargo.toml`, y ejecutar cualquier orden de `cargo` sobre un árbol recién clonado MUST NOT dejar `Cargo.lock` modificado. Verificado el 2026-10-15: en `main` (commit `1b10f4c`, `chore: release v0.5.2`) tanto `Cargo.toml` como `Cargo.lock` declaran `0.5.2`; además, sobre un árbol **recién clonado** desde GitHub se ejecutaron `cargo test` y `cargo build` **sin** `--locked` y `git status` quedó sin cambios, de modo que `Cargo.lock` no se reescribe.
