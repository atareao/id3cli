# Tasks

## 1. Red de seguridad previa a la migración

Sin este grupo cerrado en verde no se toca el backend. El objetivo es fijar el comportamiento actual de MP3 antes de sustituir `id3` por `lofty`.

- [x] 1.1 Registrar el baseline real de la suite y comprobar que no hay fallos.
      Verificación: `cargo test` MUST reportar 63 tests unitarios + 43 de integración, 106 en total, 0 fallos; anotar la salida en el change.
- [x] 1.2 Verificar que los escenarios de `specs/mp3-tagging/spec.md` describen el comportamiento que el código actual ya implementa (frames, `USLT` con `spa`, `remove --all`, nombres en inglés y español, `edit` sin cambios).
      Verificación: revisar cada escenario contra `src/lib.rs` y `src/main.rs`; cada escenario MUST estar cubierto por al menos un test existente o anotarse como hueco a cubrir en 1.3.
- [x] 1.3 Añadir los tests de caracterización que falten para los huecos detectados en 1.2, sin cambiar código de producción.
      Verificación: `cargo test` sigue en verde con el número de tests incrementado; los tests nuevos MUST pasar contra el backend `id3` antes de cualquier migración.
- [x] 1.4 Caracterizar explícitamente `remove --all` y los nombres de tag en español, que son el punto más sensible de la migración.
      Verificación: test de integración que comprueba el borrado físico de ID3v1 e ID3v2 con el audio intacto, y otro que elimina con `título`, `artista`, `carátula` y `orden-titulo`.
- [x] 1.5 Dejar constancia del baseline y de la red de seguridad en el change.
      Verificación: añadir al `README.md` (sección de tests) y a `AGENTS.md` el recuento correcto de tests, sustituyendo la cifra desactualizada de 99.
- [x] 1.6 Caracterizar los flags `--year` y `--date` sobre MP3, que hoy no están fijados de forma explícita por ningún test.
      Verificación: tests que comprueban que `--year 2026` produce el frame `TYER` con `2026`, que `--date 2026-10-13` produce el frame `TDRC` con esa fecha, y que ambos coexisten sin pisarse.
- [x] 1.7 Caracterizar qué hace hoy el CLI sobre un archivo `.m4a`.
      Verificación: ejecución manual documentada del binario actual sobre un `.m4a`, anotando si no encuentra tags y si la escritura invalida el contenedor; evidencia registrada en `baseline.md`.

## 2. Migración del backend a `lofty` con paridad de MP3

> [!warning] Las tareas 2.1 y 2.2 se ejecutan bajo el backend `id3` actual y antes de migrar nada, y MUST ir en un commit propio: si el cambio de fixtures y la migración del backend se mezclan, no se puede atribuir ninguna regresión a una causa u otra.

Una sola ruta de código y ningún cambio de comportamiento observable en MP3. Al cerrar el grupo, los tests del grupo 1 siguen pasando sin haber sido relajados.

- [x] 2.1 Crear un fixture MP3 real con `ffmpeg` y commitearlo en `tests/fixtures/`: audio MPEG mínimo, sin ningún tag ID3 (generado de forma que no se escriba ID3, por ejemplo sin metadatos) y de tamaño pequeño. Los fixtures falsos de 10 y 4 bytes dejan de servir porque `lofty` no puede parsear un MP3 sin frames MPEG reales.
      Verificación: el fichero está versionado en `tests/fixtures/`, su tipo se detecta como MPEG y `ffprobe` lo reconoce como audio reproducible.
- [x] 2.2 Adaptar `tests/integration_test.rs` (helper `create_temp_mp3()` y los pocos tests que fabrican bytes falsos de 4 bytes) para que copien el fixture real a un temporal en lugar de construir bytes.
      Verificación: la suite completa da 125 tests y 0 fallos con el backend `id3` sin migrar todavía, sin que ninguna aserción de ningún test haya cambiado y sin que ningún test haya quedado con menos cobertura.
- [x] 2.3 Sustituir la dependencia en `Cargo.toml`: entra `lofty` 0.25.4, sale `id3` 1.16.4.
      Verificación: `cargo build` compila; `cargo tree` MUST NOT mostrar el crate `id3`.
- [x] 2.4 Reescribir `src/lib.rs` contra la API de `lofty`, conservando el mapeo de frames y la semántica de cada función de la tabla del spec `mp3-tagging`.
      Verificación: `cargo check` sin errores y revisión de que cada flag conserva su frame: `TIT2`, `TPE1`, `TALB`, `TPE2`, `TYER`/`TDRC`, `TDRC`, `TCON`, `TRCK`, `TPOS`, `TCOM`, `TCOP`, `USLT`, `APIC`, `TCMP`, `TSOT`, `TSOP`, `TSOA`, `TIT3`, `TOPE` y `WOAR`.
- [x] 2.5 Adaptar `src/main.rs` a la nueva API, manteniendo los subcomandos `show`, `edit` y `remove`, sus flags y sus mensajes en español.
      Verificación: `cargo run -- --help`, `cargo run -- edit --help` y `cargo run -- remove --help` MUST mostrar la misma superficie de flags que antes del cambio.
- [x] 2.6 Reimplementar el borrado físico de `remove --all` sobre `lofty`, verificando `TagExt::remove_from_path`, conservando los mensajes por estado previo (sin tags / solo ID3v1 / solo ID3v2 / ambos) y el audio intacto.
      Verificación: los tests de integración de `remove --all` del grupo 1 MUST pasar sin modificaciones.
- [x] 2.7 Adaptar `src/tests.rs` (63 tests) a las nuevas firmas, sin cambiar ninguna expectativa de comportamiento.
      Verificación: `cargo test --lib` en verde con 63 tests unitarios; las expectativas de los tests MUST preservarse tal cual (mismos frames y mismos valores, incluido `TYER` para el año y el idioma `spa` del `USLT`), y ningún test puede quedar con menos cobertura que antes.
- [x] 2.8 Adaptar `tests/integration_test.rs` (62 tests) al nuevo backend, ajustando las firmas y las llamadas a la nueva API.
      Verificación: `cargo test --test '*'` en verde con 62 tests de integración.
- [x] 2.9 Confirmar que el MIME de la carátula sigue detectándose desde la extensión y que `.webp` sigue funcionando vía `APIC` en MP3.
      Verificación: los tests de carátula jpg, png y webp del grupo 1 MUST pasar sin cambios.
- [x] 2.10 Actualizar la documentación del propio código y del backend.
      Verificación: el doc-comment del módulo en `src/lib.rs` MUST dejar de citar `id3` y la sección de dependencias del `README.md` MUST listar `lofty` 0.25.4 y `clap`.
- [x] 2.11 Cierre del grupo: suite completa, formato y linter.
      Verificación: `cargo test` (63 tests unitarios + 62 de integración, 125 en total, 0 fallos), `cargo fmt --check` y `cargo clippy --all-targets --all-features -- -D warnings` sin salida de error.

## 3. Detección de formato por magic bytes

Backend correcto en función del contenido, y rechazo explícito de lo que queda fuera de alcance.

- [x] 3.1 Implementar la detección de formato a partir de los primeros bytes del archivo, sin consultar la extensión en ningún punto del camino.
      Verificación: revisión de código; la función de detección MUST NOT recibir la ruta con extensión como fuente de decisión.
- [x] 3.2 Enrutar `show`, `edit` y `remove` al backend MP3 cuando el formato detectado sea MP3.
      Verificación: tests de integración que ejecutan las tres operaciones sobre un MP3, comprobando que cada una usa el backend MP3, y que un `.m4a` se rechaza sin modificar el archivo.
      > [!note] Estado intermedio seguro: mientras el backend MP4 no exista, un archivo detectado como MP4 recae en el rechazo de formatos no soportados, de modo que el CLI sale con código de salida distinto de cero y MUST NOT modificar el archivo. El enrutado a MP4 se completa en la tarea 4.9.
- [x] 3.3 Añadir los tests unitarios de detección con bytes sintéticos.
      Verificación: `cargo test --lib` MUST cubrir MP3 con cabecera `ID3`; MP3 sin `ID3` por sincronía de frame MPEG válida; MP4 por `ftyp` aceptando cualquier brand (probar al menos `M4A `, `M4B `, `mp42`, `isom`, `avc1` y `qt  `); y la guarda anti-falso-positivo, con un archivo cuyo primer byte sea `0xFF` y el segundo tenga los bits altos a `1` pero con campos de versión/layer/bitrate/frecuencia reservados, que MUST NOT reconocerse como MP3.
- [x] 3.4 Cubrir el caso de archivo renombrado en ambas direcciones.
      Verificación: test de integración con un MP3 copiado a `.m4a` y un `.m4a` copiado a `.mp3`; ambos MUST procesarse por su contenido real.
- [x] 3.5 Rechazar los formatos fuera de alcance con código de salida distinto de cero y mensaje en español que enumere los formatos soportados. En este punto del cambio esa enumeración es solo MP3 (MP4 se añade en la tarea 4.9).
      Verificación: tests que invocan el binario sobre archivos FLAC (`fLaC`), OGG (`OggS`), WAV (`RIFF`), AIFF (`FORM`) y APE, comprobando código de salida distinto de cero, mensaje en stderr y archivo sin modificar.
- [x] 3.6 Cubrir los casos de borde: archivo demasiado corto, contenido no reconocible, archivo inexistente y contenido vacío.
      Verificación: `cargo test` con un test por caso, comprobando código de salida y Mensaje en stderr.
- [x] 3.7 Documentar la detección por contenido.
      Verificación: el `README.md` MUST explicar que el formato se decide por el contenido, MUST listar los formatos soportados (MP3 y MP4: `.m4a`, `.mp4`, `.m4b`) y MUST listar los no soportados.

## 4. Backend MP4 ilst: metadatos

Los campos de la tabla de mapeo, sobre el árbol `moov.udta.meta.ilst`.

> [!note] Las tareas 4.9, 4.10 y 4.11 son las que hacen que MP4 sea utilizable desde el CLI, y dependen de que el backend de escritura de metadatos (4.1 a 4.7) esté completo.

- [x] 4.1 Definir en `src/lib.rs` el mapeo de flags a átomos como una tabla única, en lugar de ramas dispersas por función.
      Verificación: revisión de código; el mapeo MUST estar en un solo sitio y cubrir las 18 asignaciones flag→átomo (17 átomos distintos, porque `--year` y `--date` comparten `©day`) de la tabla de `specs/m4a-tagging/spec.md`.
- [x] 4.2 Implementar la creación del árbol `moov.udta.meta.ilst` cuando no exista, preservando `moov` y los átomos preexistentes.
      Verificación: tests unitarios sobre la representación en memoria del tag y comprobación de que los átomos ajenos a los flags indicados sobreviven.
- [x] 4.3 Implementar la escritura de los átomos de texto básicos y de numeración: `©nam`, `©ART`, `©alb`, `aART`, `©gen`, `trkn`, `disk` y `cpil`.
      Verificación: un test por átomo, comprobando el valor escrito y la ausencia de átomos de más.
- [x] 4.4 Implementar la escritura de créditos y ordenaciones: `©wrt`, `cprt`, `sonm`, `soar`, `soal` y el átomo freeform `----:com.apple.iTunes:SUBTITLE`.
      Verificación: un test por átomo; el subtítulo MUST quedar en el átomo freeform y NO en un átomo estándar.
- [x] 4.5 Implementar la escritura de fecha en `©day` con la regla de prioridad de `--date` sobre `--year` y el warning asociado.
      Verificación: tres tests (`solo --date`, `solo --year`, `ambos`) comprobando el contenido de `©day`, la presencia o ausencia del warning por stderr y el código de salida cero.
- [x] 4.6 Añadir el fixture `.m4a` real generado con `ffmpeg` y commitearlo en `tests/fixtures/`, junto con el helper que lo copia a un temporal.
      Verificación: el fichero `tests/fixtures/*.m4a` MUST estar versionado y el helper MUST usarlo; un test unitario MUST comprobar que el fixture tiene átomo `ftyp` y que la copia temporal es escribible.
- [x] 4.7 Añadir los tests de integración de ida y vuelta sobre el fixture para los campos de metadatos.
      Verificación: `edit` sobre la copia del fixture y posterior `show` MUST devolver los mismos valores; comparar con el mismo flujo sobre MP3 para comprobar la coherencia entre formatos.
- [x] 4.8 Documentar el mapeo MP4.
      Verificación: el `README.md` MUST incluir una tabla `Flag CLI → átomo MP4` alineada con la del spec, y MUST mencionar que `--subtitle` usa un átomo freeform; el `README.md` MUST explicar además la prioridad de `--date` sobre `--year` en `©day`.
- [x] 4.9 Enrutar `show`, `edit` y `remove` al backend MP4 cuando el formato detectado sea MP4, y añadir MP4 a la lista de formatos soportados del mensaje de error.
      Verificación: tests de integración que ejecutan las tres operaciones sobre un `.m4a` y comprueban que cada una usa el backend MP4; y que el mensaje de formato no soportado enumera ya MP3 y MP4.
      > [!note] Esta tarea cierra la dependencia que dejó abierta la tarea 3.2.
- [x] 4.10 Implementar `show` sobre MP4 reutilizando el formato legible y las etiquetas de MP3, y avisar sin error cuando no haya metadatos.
      Verificación: tests de integración que comparan el conjunto de campos mostrado por `show` sobre un MP3 y sobre un `.m4a` editados con los mismos flags, y que comprueban el aviso por un `.m4a` sin metadatos.
- [x] 4.11 Implementar `remove` sobre MP4 con la tabla de alias compartida con MP3 (inglés y español) y `remove --all` eliminando el bloque `ilst` completo.
      Verificación: tests unitarios y de integración de borrado de átomos concretos (en inglés y en español) con supervivencia del resto, y de `remove --all` con metadatos (bloque eliminado, audio intacto) y sin metadatos (aviso, sin error).

## 5. Backend MP4 ilst: carátulas y letras

- [x] 5.1 Implementar la escritura de la carátula en `covr`, con el tipo well-known correspondiente a JPEG y a PNG.
      Verificación: tests de integración con un fixture `.m4a` y sendas imágenes `.jpg` y `.png`, comprobando que el átomo `covr` contiene los bytes de la imagen y el tipo esperado.
- [x] 5.2 Implementar el aviso por WebP en `.m4a`, sin abortar la operación.
      Verificación: test de integración con `--cover portada.webp` que comprueba el warning por stderr, el código de salida cero y que el resto de campos indicados sí se escriben.
- [x] 5.3 Implementar la eliminación de la carátula con `remove cover` sobre un `.m4a`.
      Verificación: test de integración que comprueba la desaparición del átomo `covr` y la supervivencia del resto de átomos.
- [x] 5.4 Implementar la escritura de la letra en el átomo `©lyr`.
      Verificación: test de integración con `--lyrics` y posterior `show`, comprobando que el texto completo se recupera.
- [x] 5.5 Verificar que el idioma de la letra no se propaga a M4A y que la lectura lo refleja.
      Verificación: test que comprueba que el `©lyr` contiene la letra y que no hay ningún campo ni prefijo de idioma; el mismo test sobre MP3 MUST seguir mostrando `spa`.
- [x] 5.6 Cubrir los errores de carátula sobre un `.m4a`.
      Verificación: tests con ruta de imagen inexistente y con extensión no soportada (`imagen.tiff`), comprobando código de salida distinto de cero y mensaje en español.
- [x] 5.7 Documentar el cambio de comportamiento observable.
      Verificación: el `README.md` MUST explicar que `.webp` no es mostrable en `covr` por iTunes/macOS (a diferencia de MP3) y MUST documentar la pérdida del código de idioma `spa` en las letras de MP4.

## 6. Campos sin equivalente en MP4

- [x] 6.1 Implementar el warning y la omisión de `--url` sobre un archivo MP4, sin escribir ningún átomo y sin abortar.
      Verificación: test de integración con `--url "https://artista.com" --title "Cancion"` que comprueba el warning, el código de salida cero, el `©nam` escrito y la ausencia de átomos freeform de reemplazo.
- [x] 6.2 Implementar el warning y la omisión de `--original-artist` sobre un archivo MP4.
      Verificación: test de integración con `--original-artist "Autor" --album "Disco"` que comprueba el warning, el código de salida cero y el `©alb` escrito.
- [x] 6.3 Cubrir el caso de los dos flags a la vez.
      Verificación: test que comprueba dos warnings en stderr, código de salida cero y ausencia de átomos nuevos en el archivo.
- [x] 6.4 Verificar que ninguno de los avisos anteriores escribe en stdout ni contamina la salida normal.
      Verificación: test que captura stdout y stderr por separado y comprueba que los warnings solo aparecen en stderr.
- [x] 6.5 Documentar los dos comportamientos.
      Verificación: el `README.md` MUST indicar que `--url` y `--original-artist` se omiten con aviso en MP4.

## 7. Superficie del CLI y documentación

- [x] 7.1 Actualizar la `description` de `Cargo.toml` para reflejar que la herramienta trabaja con MP3 y con MP4.
      Verificación: `cargo metadata --no-deps` MUST mostrar la descripción nueva.
- [x] 7.2 Actualizar los textos de `clap` (`about` del binario, descripciones de subcomandos y de flags) que hoy dicen "MP3" y deberían decir "MP3/M4A".
      Verificación: `cargo run -- --help`, `cargo run -- edit --help` y `cargo run -- remove --help` MUST mostrar los textos actualizados y MUST NOT quedar ninguna referencia obsoleta a "MP3" a secas.
- [x] 7.3 Actualizar los mensajes en español de `src/main.rs` que asumen MP3 (avisos de "no se encontraron tags ID3", confirmaciones de guardado, etc.) para que sean correctos en ambos formatos.
      Verificación: `grep` sobre los literales del binario MUST NOT encontrar mensajes que afirmen que el archivo es MP3 cuando se ha detectado MP4.
- [x] 7.4 Actualizar `README.md` completo: características, formatos soportados, tabla de mapeo MP3 y MP4, ejemplos con `.m4a`, limitaciones conocidas (WebP en `covr`, idioma de la letra), formatos no soportados y recuento real de tests.
      Verificación: revisión del documento; MUST incluir al menos un ejemplo `edit` sobre `.m4a` y MUST retirar el `.webp` de la lista de carátulas sin matices.
- [x] 7.5 Actualizar `AGENTS.md`: recuento de tests real (211: 122 unitarios + 89 de integración).
      Verificación: `grep` sobre `AGENTS.md` MUST NOT encontrar la cifra 99; el recuento documentado MUST coincidir con la salida real de `cargo test --lib` y `cargo test --test '*'`.
      > [!note] La referencia al comando de creación de OpenSpec (`openspec change` vs `openspec new change`) se comprobó con `grep` y NO vive en este repositorio: está en la configuración global del usuario, fuera del repo, así que queda fuera del alcance de este cambio.
- [x] 7.6 Revisar que la ayuda del subcomando `remove` sigue enumerando los nombres de tag en inglés y español, ahora que también aplican a MP4.
      Verificación: `cargo run -- remove --help` MUST listar los mismos nombres que la tabla del `README.md`.

## 8. Verificación de integración final

- [x] 8.1 Ejecutar la suite completa.
      Verificación: `cargo test` en verde, 0 fallos, con el recuento de tests anotado y comparado contra los 106 del baseline.
- [x] 8.2 Comprobar el formato.
      Verificación: `cargo fmt --check` sin diferencias.
- [x] 8.3 Comprobar el linter en todos los targets y features.
      Verificación: `cargo clippy --all-targets --all-features -- -D warnings` sin warnings.
- [x] 8.4 Validar el cambio contra OpenSpec.
      Verificación: `openspec validate m4a-support` sin errores; `openspec show m4a-support` MUST listar las tres capacidades nuevas.
- [x] 8.5 Comprobar la paridad de MP3 contra el baseline del grupo 1.
      Verificación: revisar que ningún test del grupo 1 ha sido relajado o eliminado; `--year` MUST seguir escribiendo el frame `TYER` después de la migración, comparándolo contra la caracterización del grupo 1 (tarea 1.6); el diff de `src/tests.rs` y `tests/integration_test.rs` MUST mostrar solo adaptaciones de firma, no cambios de expectativa.
- [x] 8.6 Comprobar el flujo de extremo a extremo sobre los dos formatos con un archivo real de cada tipo.
      Verificación: ejecutar `edit`, `show` y `remove` sobre un `.mp3` y un `.m4a` reales y comprobar que la salida es coherente entre formatos y que los archivos siguen siendo reproducibles.
- [x] 8.7 Cerrar el cambio y marcar las tareas completadas.
      Verificación: todas las casillas de este fichero marcadas y `openspec archive m4a-support` listo para ejecutar tras la aprobación del usuario.
