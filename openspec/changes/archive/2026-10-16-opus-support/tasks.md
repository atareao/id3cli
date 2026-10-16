# Tasks

El cambio añade Opus como segundo códec del contenedor Ogg reutilizando el backend de comentarios de Vorbis: la detección gana la variante `AudioFormat::OggOpus`, `read_vorbis_tag` se generaliza para despachar por códec a `VorbisFile` u `OpusFile`, y el CLI se queda con un único `Backend::Ogg` que sirve a los dos códecs. Los grupos 1, 2 y 3 son estrictamente test-first: cada comportamiento nuevo se escribe primero como RED (un test que falla) y solo después como GREEN (la implementación mínima que lo pone en verde); ninguna tarea GREEN precede a su RED. El grupo 4 implementa, el 5 reconcilia documentación y código y el 6 cierra con la verificación final. El punto de partida es la suite actual: **273 tests (162 unitarios + 111 de integración)**.

## 1. Fase RED — detección unitaria de Opus

La detección actual conoce dos salidas para un contenedor Ogg válido: Vorbis (aceptado) o cualquier otro códec (rechazado). Opus pasa de la segunda a la primera, así que primero se invierten los tests que hoy fijan su rechazo y se añade el que fija su aceptación.

- [x] 1.1 Invertir `test_detect_format_ogg_other_codecs` (`src/tests.rs:1447`): quitar `b"OpusHead"` de la lista de códecs no soportados (1450-1454), que queda con Speex, FLAC dentro de Ogg y Theora, y actualizar el comentario que hoy nombra a Opus entre los rechazados.
      Verificación: `cargo test --lib test_detect_format_ogg -v` MUST ejecutar el test invertido y dejarlo en verde, con tres códecs y sin ninguna mención a Opus.
      Evidencia (2026-10-16): `cargo test --lib test_detect_format_ogg -v` → `running 4 tests` con `test tests::test_detect_format_ogg_other_codecs ... ok` y `test result: ok. 4 passed; 0 failed`; la lista queda en **3 códecs** (Speex, FLAC dentro de Ogg y Theora) y el comentario del test ya solo nombra esos tres como fuera de alcance.
- [x] 1.2 Añadir el test nuevo `test_detect_format_ogg_opus`: `synthetic_ogg_page(b"OpusHead")` (`src/tests.rs:1406`) → `AudioFormat::OggOpus`, comprobando además el caso extensión (el mismo contenido renombrado a `.mp3` sigue reconociéndose por sus bytes) y el `detect_format_from_bytes` con los primeros 64 bytes.
      Verificación: `cargo test --lib test_detect_format_ogg -v` MUST estar en **ROJO**: `AudioFormat::OggOpus` no existe todavía, así que compila con `error[E0599]: no variant, associated function, or constant named `OggOpus` found for enum `AudioFormat`` y la suite no llega a ejecutarse; anotar la salida literal como evidencia del rojo.
      Evidencia (2026-10-16): `cargo test --lib test_detect_format_ogg -v` → `exit 101` (error de compilación; la suite unitaria entera queda en rojo hasta la fase GREEN, como se espera). Salida literal:
      ```
      error[E0599]: no variant, associated function, or constant named `OggOpus` found for enum `AudioFormat` in the current scope
          --> src/tests.rs:1478:62
      1478 |     assert_eq!(detect_format_from_bytes(&page), AudioFormat::OggOpus);
           |                                                              ^^^^^^^ variant, associated function, or constant not found in `AudioFormat`
          ::: src/lib.rs:928:1
       928 | pub enum AudioFormat {
      ```
      (el mismo error se repite en src/tests.rs:1485 y :1498, las otras dos referencias del test). El test cubre los tres casos: página sintética `OpusHead`, contenido renombrado a `.mp3` vía `detect_format`, y `detect_format_from_bytes` con 64 bytes (la página sintética se rellena con ceros hasta 64, ya que el fixture real no existe todavía en el grupo unitario).

## 2. Fase RED — fixture real de Opus

Los tests de integración no pueden usar páginas fabricadas: hace falta un `.opus` real que `lofty` abra como `OpusFile` y que `ffprobe` reconozca. El fixture se genera con `ffmpeg` y se versiona junto con el script que lo regenera, igual que el `.ogg`.

- [x] 2.1 Crear `tests/fixtures/sample.opus` con `ffmpeg`: fuente `anullsrc` a 48 kHz (frecuencia nativa de Opus), mono, 0,6 s, códec `libopus`, con `-map_metadata -1 -fflags +bitexact -flags:a +bitexact`, tamaño pequeño y commiteado en `tests/fixtures/`.
      Verificación: `ffprobe` MUST reconocerlo como audio Opus reproducible; `file` MUST reconocerlo como contenedor Ogg con códec Opus; el primer paquete en `27 + header[26]` MUST empezar por `OpusHead` (8 bytes), el segundo por `OpusTags`, y el comentario de Vorbis MUST llevar vendor y `ENCODER` sin ningún campo de usuario.
      Evidencia (2026-10-16): generado con el comando del enunciado: `ffmpeg -f lavfi -i anullsrc=r=48000:cl=mono -t 0.6 -c:a libopus -map_metadata -1 -fflags +bitexact -flags:a +bitexact -y tests/fixtures/sample.opus` → **272 bytes**. `file` → «Ogg data, Opus audio, version 0.1, mono, 48000 Hz (Input Sample Rate)»; `ffprobe` → `codec_name=opus`, `sample_rate=48000`, `format_name=ogg`; decodificación completa con `ffmpeg -i sample.opus -f null -` (OK). Verificación estructural por python: primer paquete en el offset **28** = `OpusHead` (19 bytes), segundo paquete = `OpusTags` (46 bytes), vendor `ffmpeg`, campos `['encoder=Lavc libopus']`, sin campos de usuario. Hace `git add` sobre el fixture versionado.
- [x] 2.2 Crear `tests/fixtures/generate_sample_opus.sh` que regenere el fixture y **verifique estructura, no igualdad de bytes**, siguiendo el patrón de `generate_sample_ogg.sh`: patrón `OggS`, versión `0`, primer paquete en `27 + header[26]` empezando por `OpusHead`, segundo paquete `OpusTags` con vendor no vacío y clave `ENCODER`, decodificación completa con `ffmpeg` y ausencia de campos de usuario.
      Verificación: el script MUST generar a un temporal, verificar la estructura y solo entonces mover el fichero; MUST fallar con su mensaje y código 1 ante casos negativos (bytes aleatorios, primer paquete corrompido, comentario con `TITLE`, comentario sin `ENCODER`) y MUST pasar con código 0 sobre el fixture real; regenerar el fixture desde el script MUST reproducir un archivo de estructura válida.
      Evidencia (2026-10-16): `generate_sample_opus.sh` con `chmod +x`, modo generar (a temporal, verificar, mover) más modo `--check ARCHIVO` para los casos negativos. `--check tests/fixtures/sample.opus` → `OK: estructura válida` (exit 0); regeneración sin args → `OK: ... generado y verificado` (exit 0) y el regenerado pasa el `--check`. Casos negativos (todos exit 1 con su mensaje):
      ```
      NEG bytes aleatorios        → ERROR: el patrón OggS no está en el offset 0
      NEG primer paquete corrupto → ERROR: el primer paquete no empieza por el identificador OpusHead
      NEG comentario con TITLE    → ERROR: el comentario contiene campos de usuario: TITLE
      NEG comentario sin ENCODER  → ERROR: el comentario no lleva la clave ENCODER con valor
      ```
      (el caso «sin ENCODER» se construyó reescribiendo el `OpusTags` con `count=0` preservando el vendor; el caso «con TITLE» con `ffmpeg -metadata title`.)
- [x] 2.3 Añadir el helper `create_temp_opus()` en `tests/integration_test.rs`, réplica de `create_temp_ogg` (3001): copia el fixture versionado a una ruta temporal única; nunca se opera sobre el fixture versionado directamente.
      Verificación: el helper existe junto a `create_temp_ogg` y los tests del grupo 3 lo usan; `cargo test --test integration_test` compila.
      Evidencia (2026-10-16): `fixture_opus_path()` y `create_temp_opus()` añadidos inmediatamente después de `create_temp_ogg` (mismo patrón: `unique_temp_path("audio", "opus")` + `fs::copy` del fixture versionado); lo usan los 7 tests del grupo 3.2. `cargo test --test integration_test --no-run` → `Finished test profile` + `Executable tests/integration_test.rs` (compila).

## 3. Fase RED — integración y CLI sobre Opus

El binario todavía rechaza Opus: los tests de integración fijan el comportamiento nuevo (aceptación) y la suite queda en rojo hasta la implementación del grupo 4. Los tests de caracterización del grupo 1 y los de los grupos 2.1-2.2 siguen en verde.

- [x] 3.1 Invertir `test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched` (`tests/integration_test.rs:3215`): quitar el caso `("opus", b"OpusHead")` (3222) y dejar solo `speex`, y actualizar el comentario que hoy dice que Opus está fuera de alcance.
      Verificación: el test invertido MUST seguir en verde con el caso de Speex y sin mencionar Opus; los asserts de rechazo (código distinto de cero, mensaje en español con los formatos soportados, stdout vacío y archivo intacto byte a byte) MUST seguir cubriendo Speex, FLAC dentro de Ogg y Theora.
      Evidencia (2026-10-16): `cargo test --test integration_test test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched` → `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 110 filtered out`; la lista se queda en `[("speex", b"Speex   ")]` y el comentario ya solo nombra Speex (y FLAC dentro de Ogg o Theora) como fuera de alcance. En la ejecución completa del 3.2 este test sigue en verde dentro de los 111 pasados.
- [x] 3.2 Añadir los tests CLI nuevos sobre `sample.opus` (vía `create_temp_opus()`): `show` de un tag existente; `edit` con título, artista y álbum releído con un lector independiente; `edit --track 7/12 --season 1/2` comprobando `TRACKNUMBER`/`TRACKTOTAL` y `DISCNUMBER`/`DISCTOTAL`; `edit --cover` con carátula releída; `remove` de un campo y `remove --all` con vendor preservado; y el flujo completo de `show`/`edit`/`remove` sobre el mismo archivo.
      Verificación: `cargo test --test integration_test` MUST estar en **ROJO** por los tests nuevos —el binario actual rechaza `.opus` con «formato no soportado»— mientras los demás tests de la suite siguen en verde; anotar la salida del fallo.
      Evidencia (2026-10-16): 7 tests nuevos en `tests/integration_test.rs` tras el test de rechazo invertido: `test_cli_opus_show_existing_tag`, `test_cli_opus_edit_title_artist_album_then_read_back`, `test_cli_opus_edit_track_season_totals`, `test_cli_opus_edit_cover_then_read_back` (JPEG y PNG), `test_cli_opus_remove_field_keeps_rest`, `test_cli_opus_remove_all_preserves_vendor` y `test_cli_opus_show_edit_remove_flow`; todos usan `create_temp_opus()`. `cargo test --test integration_test --no-fail-fast` → `test result: FAILED. 111 passed; 7 failed` — los 7 fallan por el binario actual:
      ```
      Error: El archivo '/tmp/test_audio_*.opus' está en un formato de audio no soportado. Formatos soportados: MP3, MP4 y OGG.
      ```
      y los **111 tests existentes siguen en verde** (incluido el 3.1 invertido). Nota: los tests de integración NO nombran `AudioFormat::OggOpus` (no existe y rompería la compilación de toda la suite): los asserts de estructura post-`remove --all` comprueban el patrón `OggS` y el primer paquete `OpusHead` por bytes, y la relectura usa `id3cli::read_vorbis_tag`, igual que los tests OGG.

## 4. Fase GREEN — implementación

Implementación mínima que pone en verde los tres grupos RED, en el mismo orden en que se escribieron los tests: primero la detección, luego la lectura despachada y por último el enrutado del CLI.

- [x] 4.1 `src/lib.rs`: añadir la variante `OggOpus` al enum `AudioFormat` (928) y reconocerla en la detección por contenido: con las condiciones de página Ogg ya validadas, el primer paquete que empiece por `\x01vorbis` → `AudioFormat::OggVorbis` y el que empiece por `OpusHead` → `AudioFormat::OggOpus`. Speex, FLAC dentro de Ogg y Theora MUST seguir devolviendo `Unsupported`, y la página incompleta o sin el primer paquete cerrado MUST seguir devolviendo `Unknown`.
      Verificación: `cargo test --lib test_detect_format_ogg -v` en verde (los tres tests previos más el nuevo `test_detect_format_ogg_opus`).
      Evidencia (2026-10-16): variante `OggOpus` añadida tras `OggVorbis` con su doc en español; el bloque Ogg de `detect_format_from_bytes` pasa de `if` a `match` por identificador (`\x01vorbis` → `OggVorbis`, `OpusHead` → `OggOpus`, resto → `Unsupported`); docs y comentarios que nombraban a Opus como fuera de alcance actualizados (doc de `UNSUPPORTED_MAGIC`, doc de `Unsupported` en el enum, doc de `detect_format`, doc y comentario en línea de `detect_format_from_bytes`). `cargo test --lib test_detect_format_ogg -v` →
      ```
      running 5 tests
      test tests::test_detect_format_ogg_empty_packet ... ok
      test tests::test_detect_format_ogg_incomplete_page ... ok
      test tests::test_detect_format_ogg_other_codecs ... ok
      test tests::test_detect_format_ogg_opus ... ok
      test tests::test_detect_format_ogg_vorbis ... ok
      test result: ok. 5 passed; 0 failed
      ```
- [x] 4.2 `src/lib.rs`: generalizar `read_vorbis_tag` (2329) para detectar el códec del contenedor (reutilizando la misma lógica de sondeo de la primera página, vía `detect_format_from_bytes` o la función que la implemente) y despachar a `VorbisFile::read_from` u `OpusFile::read_from`, conservando el nombre, la firma y el tipo de retorno `VorbisComments`; `write_vorbis_tag` (2346) no cambia, porque `save_to_path(path, WriteOptions::default())` inspecciona el contenedor real.
      Verificación: `cargo check` limpio sin cambiar ninguna llamada existente a `read_vorbis_tag` ni a `write_vorbis_tag`; `cargo test --lib` en verde.
      Evidencia (2026-10-16): `read_vorbis_tag` conserva nombre, firma `pub fn read_vorbis_tag(path: &Path) -> Result<VorbisComments, Box<dyn std::error::Error>>` y tipo de retorno; despacha por `detect_format(path)` (la misma sonda de la primera página de la detección): `OggVorbis` → `VorbisFile::read_from` + `vorbis_comments().clone()` (camino exacto de antes), `OggOpus` → `OpusFile::read_from` (de `lofty::ogg::OpusFile`, import añadido) + `vorbis_comments().clone()`, resto → `io::Error::new(ErrorKind::InvalidData, "el archivo no es un contenedor Ogg con comentarios de Vorbis")`. Doc comment actualizado («sirve a cualquier contenedor Ogg con comentarios de Vorbis», Vorbis y Opus). `write_vorbis_tag` intacta. Ninguna llamada existente cambió. `cargo check` limpio una vez completado el 4.3 (el intermedio sin la rama del match de `main.rs` no compila el binario por el match no exhaustivo del enum, consustancial al orden de los grupos) y `cargo test --lib` →
      ```
      running 163 tests (de los cuales 162 passed; 1 failed)
      ```
      el único fallo es `test_documentacion_describe_los_tres_formatos`, pendiente del grupo 5 (ver 4.4 y «pendientes grupo 5»).
- [x] 4.3 `src/main.rs`: enrutar `Ok(AudioFormat::OggOpus) => Backend::Ogg` junto a `OggVorbis` (195), sin ramas nuevas que distingan los dos códecs; los mensajes en español que enumeran los formatos soportados (199, 207, 236 y 255) pasan a «Formatos soportados: MP3, MP4, OGG y Opus.»; actualizar los asserts, comentarios y literales de los tests que exigían «MP3, MP4 y OGG» para exigir también Opus.
      Verificación: `grep -n "Formatos soportados" src/main.rs` MUST NOT dejar ningún literal que enumere solo tres formatos; `cargo test --no-fail-fast` completo en verde con el recuento nuevo de tests anotado frente a los 271 del baseline.
      Evidencia (2026-10-16): `Ok(AudioFormat::OggOpus) => Backend::Ogg` añadido junto a `OggVorbis` (un único backend OGG para ambos códecs, sin ramas nuevas en show/edit/remove); doc de `Backend` y de `report_ogg_error` actualizados («OGG (Vorbis u Opus)» y «Ogg Vorbis u Opus»); doc de `detect_supported_format` sin Opus entre los no soportados; docs de `report_mp4_error`/`report_ogg_error` enumeran «MP3, MP4, OGG y Opus». `grep -n "Formatos soportados" src/main.rs` →
      ```
      200:  Formatos soportados: MP3, MP4, OGG y Opus.",
      208:  Formatos soportados: MP3, MP4, OGG y Opus.",
      237:  Formatos soportados: MP3, MP4, OGG y Opus."
      256:  Formatos soportados: MP3, MP4, OGG y Opus."
      ```
      Los asserts de la suite existente usan `stderr.contains("MP3") && contains("MP4") && contains("OGG")` (nunca el literal completo), así que no hace falta tocarlos: siguen en verde con la lista ampliada. `cargo test --no-fail-fast` →
      ```
      unittests src/lib.rs        : 162 passed; 1 failed (doc-test del grupo 5)
      unittests src/main.rs       : 0 passed; 0 failed
      tests/integration_test.rs   : 118 passed; 0 failed
      ```
      Recuento nuevo frente al baseline: 163 unitarios + 118 de integración = **281 tests** (baseline 273: el cambio añade 1 unitario y 7 de integración).
- [x] 4.4 Lint de la implementación.
      Verificación: `cargo clippy --all-targets --all-features -- -D warnings` limpio y `cargo test --no-fail-fast` 0 fallos.
      Evidencia (2026-10-16): `cargo clippy --lib --bins -- -D warnings` limpio sin tocar nada; la suite completa `--all-targets` señaló un único lint en **código de test de la fase RED** (`tests/integration_test.rs:3321`, `test_cli_opus_show_existing_tag`: `TagExt::save_to_path(&mut tag, …)` → `&tag`, `clippy::unnecessary_mut_passed`; corrección mecánica sin tocar ningún assert ni el comportamiento del test, necesaria para que el gate del grupo 4 pueda quedar en verde) — se reporta como excepción explícita del grupo 3. Con eso:
      ```
      cargo clippy --all-targets --all-features -- -D warnings
      Finished `dev` profile [unoptimized + debuginfo] target(s)
      ```
      y `cargo test --no-fail-fast` con 0 fallos salvo el doc-test `test_documentacion_describe_los_tres_formatos`, **pendiente del grupo 5** (ver más abajo).

## 5. Fase REFACTOR — coherencia documental

Con Opus soportado, la documentación y el test que la verifica deben dejar de tratar a Opus como no soportado; la coherencia entre código, specs, README y documentos del repositorio se cierra aquí, sin relajar ninguna expectativa.

- [x] 5.1 Actualizar el test de coherencia documental (`src/tests.rs:4620-4748`): la lista de «No soportado» (4651) deja de exigir la presencia de Opus y la hoja de ruta (4748 y siguientes) deja de exigir una línea `- [ ]` que marque Opus como pendiente.
      Verificación: `cargo test --lib test_documentacion -v` en verde; el diff MUST mostrar solo el cambio de la lista y de la hoja de ruta, sin relajar ningún otro criterio.
      Evidencia (2026-10-16): en `src/tests.rs` la lista de pendientes de «No soportado» pasa de `["Opus", "Speex", "FLAC", "WAV", "AIFF", "APE"]` a `["Speex", "FLAC", "WAV", "AIFF", "APE"]`, el bucle de formatos soportados pasa a `["MP3", "MP4", "OGG", "Opus"]`, la hoja de ruta gana el chequeo `opus_hecho` (exige una línea `- [x]` que mencione Opus, espejo del `ogg_hecho` existente) y sus pendientes se quedan en `["Speex", "FLAC"]`, y se actualizan el comentario (1), el comentario (4) y el mensaje del `assert!` final («los formatos (MP3, MP4, OGG y Opus)»). El resto de comprobaciones (mapas, degradaciones, `Cargo.toml`, desglose) no se tocan. Fase RED registrada antes del 5.2: `cargo test --lib test_documentacion -v` fallaba exactamente por el contrato nuevo (`[4] la hoja de ruta no marca Opus como soportado` + los 4 comprobaciones del recuento `[5]` frente al README que aún declaraba 273). Tras el 5.2, GREEN:
      ```
      cargo test --lib test_documentacion -v
      running 1 test
      test tests::test_documentacion_describe_los_tres_formatos ... ok
      test result: ok. 1 passed; 0 failed
      ```
- [x] 5.2 Actualizar `README.md`, `AGENTS.md` y `.github/copilot-instructions.md`: Opus sale de las listas de formatos no soportados y entra en las de soportados (con su sección de mapeo en el README), el párrafo que explica que el contenedor se decide por el códec del primer paquete deja de nombrar `OpusHead` entre los rechazados, la hoja de ruta marca Opus con `- [x]` y el recuento de tests se actualiza al medido.
      Verificación: `grep` registrado en el change de que ninguna línea de «No soportado» menciona Opus y de que el recuento declarado coincide con el medido en la suite.
      Evidencia (2026-10-16): **README** — escudo `tests-281%20passing`; intro y características a «MP3, MP4, OGG y Opus»; tabla de `## Formatos admitidos` con la fila nueva «**Soportado** | OGG Opus (`OggS`, versión `0`, primer paquete `OpusHead`)» y sin Opus en la fila de «No soportado» (queda Speex en contenedor Ogg; FLAC, WAV, AIFF, APE, WavPack); el párrafo del contenedor se reescribe en dos frases: «Solo se acepta si ese primer paquete empieza por `\x01vorbis` (Vorbis) o por `OpusHead` (Opus)» y «Si el primer paquete empieza por `Speex`, por `\x7fFLAC`… o si es el de Theora, el archivo se rechaza…» (ninguna línea mezcla `Opus` con la palabra «rechaza»); resumen «Soportados: … OGG Vorbis y Opus (comentarios de Vorbis). Los cuatro admiten…» / «No soportados: Speex (en contenedor Ogg), FLAC…»; sección `## Mapeo de tags en OGG (Vorbis y Opus)` con el mapeo compartido por ambos códecs más la nota de que las tres degradaciones aplican igual a Opus; ejemplo de edición `### 🎼 … u Opus (`.opus`)`; `## Tests` → 281 (163 unitarios + 118 de integración) con los comentarios `(281)/(163)/(118)`; hoja de ruta `- [x] Soporte para Opus (contenedor Ogg con el códec Opus; mismas claves de comentario Vorbis que OGG)`; árbol del proyecto con `sample.opus`/`generate_sample_opus.sh` y recuentos de líneas reales (2702/871/4888/5438). **AGENTS.md** — intro «OGG (comentarios de Vorbis… el contenedor Ogg cubre Vorbis y Opus)»; `cargo test # 281 tests (163 unit + 118 integración)`; arquitectura «**OGG** (Vorbis y Opus; `VorbisComments` + `VorbisFile`/`OpusFile`)»; helpers con `create_temp_opus()`; fixtures con `sample.opus` y `generate_sample_opus.sh`; convenciones: «Vorbis y Opus están soportados», no soportados sin Opus y «se rechaza cuando el primer paquete de su códec no es de un códec soportado del contenedor (Speex, FLAC, Theora…)». **copilot-instructions.md** — big picture «the Ogg container covers both Vorbis and Opus»; dispatch «OGG (`VorbisComments`, Vorbis or Opus)»; detección «`OpusHead` → OGG Opus»; unsupported sin Opus («not a supported Ogg codec»); mapeo OGG «applies verbatim to OGG Opus»; `281 = 163 unit + 118 integration` en los tres sitios (workflows, primeros pasos, constraints); fixtures/helpers con opus y recuentos de líneas reales. Grep de verificación:
      ```
      grep -rn "Opus" README.md AGENTS.md .github/copilot-instructions.md | grep -i "rechaz\|no soportad\|pendiente"  →  (sin salida)
      ```
      y `cargo test --lib test_documentacion -v` en verde (ver 5.1).
- [x] 5.3 Formateo final.
      Verificación: `cargo fmt --check` limpio y `cargo test --no-fail-fast` en verde tras el formateo.
      Evidencia (2026-10-16): `cargo fmt --check` detectaba solo diferencias de formato en `tests/integration_test.rs` (código de test del grupo 3, arrays de `run_cli` y `assert_eq!` multilínea); `cargo fmt` las corrige y a partir de ahí `cargo fmt --check` → limpio (salida «FMT CLEAN», exit 0). Tras el formateo:
      ```
      cargo test --no-fail-fast
      running 163 tests   →  test result: ok. 163 passed; 0 failed
      running 118 tests   →  test result: ok. 118 passed; 0 failed
      ```
      **281 tests (163 unitarios + 118 de integración), 0 fallos.**
- [x] 5.4 Prueba manual con el binario sobre un `.opus` real.
      Verificación: `cargo run -- show`, `cargo run -- edit --title "X"` y `cargo run -- remove --all` sobre un `.opus` real reproducen el flujo de los otros formatos: salida correcta por stdout, avisos por stderr solo donde toca, código de salida cero y el archivo sigue siendo un Opus reproducible al final.
      Evidencia (2026-10-16): copia temporal `/tmp/opencode/prueba_opus.opus` del fixture versionado (272 bytes, `file` → «Ogg data, Opus audio, version 0.1, mono, 48000 Hz (Input Sample Rate)», `ffprobe` → `codec_name=opus`, `format_name=ogg`); el fixture versionado no se tocó (`git status` lo mantiene como `A` del grupo 2).
      ```
      cargo run --quiet -- show tests/fixtures/sample.opus      # «⚠️ No se encontraron metadatos.» → exit 0
      cargo run --quiet -- edit /tmp/.../prueba_opus.opus --title "Prueba Opus" --artist "Atareao" --album "Test"
          # ✓ Título: Prueba Opus / ✓ Artista(s): Atareao / ✓ Álbum: Test / ✅ Metadatos guardados correctamente → exit 0
      cargo run --quiet -- show /tmp/.../prueba_opus.opus       # 🎵 Título: Prueba Opus, 🎤 Artista: Atareao, 💿 Álbum: Test → exit 0
      cargo run --quiet -- edit /tmp/.../prueba_opus.opus --track 7/12 --season 1/2   # ✓ Pista: 7/12, ✓ Temporada: 1/2 → exit 0
      cargo run --quiet -- show /tmp/.../prueba_opus.opus       # #️⃣ Pista: 7/12, 📺 Temporada: 1/2 → exit 0
      ```
      Lectura cruda del `OpusTags` (vendor `ffmpeg`): `TITLE=Prueba Opus`, `ARTIST=Atareao`, `ALBUM=Test`, `TRACKNUMBER=7`, `TRACKTOTAL=12`, `DISCNUMBER=1`, `DISCTOTAL=2`, y `encoder=Lavc libopus` conservado (clave ajena). `remove title` (posicional; `--title` no es flag de `remove`, el tag se pasa como argumento) → «✓ Eliminado: title» y el `show` relee artista/álbum/pista/temporada sin el título → exit 0. `remove --all` → «✅ Metadatos eliminados» → `show` «⚠️ No se encontraron metadatos» → lectura cruda: vendor `ffmpeg` preservado con **0 campos de usuario**; `ffprobe` → `codec_name=opus` / `format_name=ogg` y `ffmpeg -i … -f null -` decodifica completo → el `.opus` sigue siendo reproducible. Todos los comandos con código de salida 0 y sin avisos de stderr (no hay degradaciones en OGG para estos campos).

## 6. Verificación final y cierre

Cierre con la suite completa, repaso escenario a escenario de los specs del change y evidencia fechada dentro del propio change. El archivo del change solo ocurre en el PR final.

- [x] 6.1 Ejecutar las comprobaciones mecánicas completas.
      Verificación: `cargo test --no-fail-fast` completo, `cargo fmt --check` y `cargo clippy --all-targets --all-features --locked -- -D warnings` limpios, con la salida anotada y el recuento de tests comparado contra los 271 del baseline.
      Evidencia (2026-10-16): correcciones post-review del director: la ayuda del CLI anuncia Opus (`about` y doc de show/edit/remove, `src/main.rs`), el test de ayuda exige Opus y los asserts de rechazo de formato no soportado de Ogg (códec Speex) y FLAC añaden `stderr.contains("Opus")` (`tests/integration_test.rs`), y el test de coherencia documental se renombra a `test_documentacion_describe_los_formatos_soportados` porque ya verifica cuatro formatos (`src/tests.rs`; las referencias al nombre antiguo en la evidencia de este change y del archivo `ogg-support-and-track-totals` se conservan como registro histórico). Comprobaciones:
      ```
      cargo fmt --check                                   → limpio (exit 0, «FMT CLEAN»)
      cargo test --no-fail-fast
        unittests src/lib.rs        → running 163 tests   → ok. 163 passed; 0 failed
        tests/integration_test.rs   → running 118 tests   → ok. 118 passed; 0 failed
        Doc-tests id3cli            → running 0 tests     → ok. 0 passed; 0 failed
        → 281 tests, 0 fallos (baseline 271: el change añade 1 unitario + 7 de integración)
      cargo clippy --all-targets --all-features --locked -- -D warnings
        → Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s (sin warnings)
      cargo run -- --help
        → «Añade tags y carátulas a archivos MP3 (ID3v2.4), MP4 (átomos iTunes) y OGG (comentarios Vorbis; Vorbis y Opus)»
          show/edit/remove → «Mostrar/Editar/Eliminar ... del archivo MP3, MP4, OGG y Opus»; `<FILE>` → «(MP3, MP4, OGG y Opus)»
      ```
      La casilla 6.3 (tabla escenario → test) no la toca esta corrección: la cierra el director. El renombrado no altera el recuento de tests (281).
- [x] 6.2 Comprobar el estado del change.
      Verificación: `openspec status --change opus-support` MUST listar los cuatro ficheros del change y MUST NOT mostrar spec pendientes de escenario.
      Evidencia (2026-10-16): `openspec status --change opus-support` → 4/4 artefactos completos, sin spec pendientes de escenario. Salida literal:
      ```
      - Loading change status...
      Change: opus-support
      Schema: spec-driven
      Change root: /data/rust/id3cli/openspec/changes/opus-support
      Progress: 4/4 artifacts complete

      [x] proposal
      [x] specs
      [x] design
      [x] tasks

      All planning artifacts complete!
      Next: openspec instructions apply --change "opus-support" --json
      ```
- [x] 6.3 Repasar cada escenario de los specs del change y anotar el test que lo cubre.
      Verificación: tabla escenario → test en el change; los escenarios sin test MUST quedar marcados como verificados manualmente con su evidencia.
      Evidencia (2026-10-16): repaso escenario a escenario de `specs/format-detection/spec.md` y `specs/opus-tagging/spec.md` contra la suite (281 tests). Todos los nombres de test citados existen (verificados con `grep` sobre `src/tests.rs` y `tests/integration_test.rs`). Opus comparte íntegro el backend OGG (`Backend::Ogg`, `read_vorbis_tag` despachada a `OpusFile`), de modo que los unit del backend OGG (`test_ogg_*`) cubren también los escenarios de Opus que no tienen test CLI directo sobre `.opus`; los escenarios con degradación o rutas no ejercitadas por un `.opus` en la suite quedan marcados como «verificado manualmente» (evidencia 5.4 y auditoría).

      **Tabla escenario → test (formato-detección, `specs/format-detection/spec.md`)**

      | Escenario | Test que lo cubre | Notas |
      |---|---|---|
      | Detección de MP3 por cabecera ID3 | unit `test_detect_format_mp3_by_id3_header` (src/tests.rs:1249) y `test_detect_format_mp3_from_real_fixture_header` (1263) | Pre-existente, fuera del delta Opus |
      | Detección de MP3 por sincronía de frame MPEG | unit `test_detect_format_mp3_by_mpeg_frame_sync` (1256), `test_detect_format_mp3_from_real_fixture_header` (1263) | Pre-existente |
      | Falso positivo de sincronía con cabecera MPEG implausible | unit `test_detect_format_rejects_implausible_mpeg_sync` (1287) | Pre-existente |
      | Detección de MP4 por átomo ftyp | unit `test_detect_format_mp4_ftyp_accepts_any_brand` (1270) | Pre-existente |
      | Detección de MP4 con brand minoritario (`M4B `, `avc1`, `qt  `…) | unit `test_detect_format_mp4_ftyp_accepts_any_brand` (1270) | Pre-existente |
      | La extensión no determina el formato (MP3→.m4a, MP4→.mp3, Opus→.mp3) | unit `test_detect_format_ignores_extension` (1361) + `test_detect_format_ogg_opus` (1470, contenido Opus renombrado a `.mp3`) + integración `test_cli_ogg_renamed_to_mp3_is_edited_by_content` (3115) | El caso Opus es nuevo: lo cubre el 1.2 |
      | Detección de OGG Vorbis (cualquier extensión, incluida `.oga` o ninguna) | unit `test_detect_format_ogg_vorbis` (1423: fixture real, renombrado a `.mp3` y 64 bytes por `detect_format_from_bytes`) | — |
      | **Detección de Opus dentro de Ogg (`OpusHead` → OggOpus, cualquier extensión)** | unit `test_detect_format_ogg_opus` (1470: página sintética `OpusHead`, contenido renombrado a `.mp3`, 64 bytes) + integración: todos los `test_cli_opus_*` (el binario enruta por contenido, nunca por extensión) | Nuevo del change |
      | **Contenedor Ogg con un códec no soportado (Speex/FLAC/Theora)** | unit `test_detect_format_ogg_other_codecs` (1447) + integración `test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched` (3239) | El 3239 cubre show/edit/remove: salida ≠ 0, stdout vacío y archivo intacto byte a byte |
      | **Página Ogg incompleta (o primer paquete sin cerrar) → Unknown** | unit `test_detect_format_ogg_incomplete_page` (1503) y `test_detect_format_ogg_empty_packet` (1532) + integración caso `ogg_sin_paquete` en `test_cli_rejects_unsupported_formats_and_leaves_file_untouched` (2853) | CLI: salida ≠ 0 y archivo sin cambios |
      | Formato de audio fuera de alcance (FLAC, WAV, AIFF… y Speex/FLAC-en-Ogg/Theora) | integración `test_cli_rejects_unsupported_formats_and_leaves_file_untouched` (2842) + unit `test_detect_format_unsupported_magic_bytes` (1338) y `test_detect_format_ogg_other_codecs` (1447) | Pre-existente; el mensaje y la integridad los cubre el 2842 |
      | Contenido no reconocible | unit `test_detect_format_unrecognized_content` (1353) + integración `test_cli_rejects_unrecognized_content` (2929) | Pre-existente |
      | Archivo demasiado corto para decidir el formato | unit `test_detect_format_empty` (1314) y `test_detect_format_too_short` (1319) + integración `test_cli_rejects_empty_file` (2906) y `test_cli_rejects_too_short_file` (2948) | Pre-existente |
      | Archivo inexistente | Sin test CLI dedicado en la suite: manejo en `src/main.rs:188` («El archivo … no existe»); verificado manualmente (auditoría) | Pre-existente, fuera del delta Opus |
      | **El mensaje enumera los formatos soportados (MP3, MP4, OGG y Opus)** | integración `test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched` (3274-3280: `stderr` con MP3+MP4+OGG+**Opus**) y `test_cli_rejects_unsupported_formats_and_leaves_file_untouched` (2886-2889) + `cargo run -- --help` (evidencia 6.1) | El literal «MP3, MP4, OGG y Opus.» está en `src/main.rs` (200, 208, 237, 256) |
      | **Salida por stdout no se usa para el error** | integración `test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched` (3281-3285) y `test_cli_rejects_unsupported_formats_and_leaves_file_untouched` (2890-2894): `stdout.is_empty()` | — |

      **Tabla escenario → test (tagging de Opus, `specs/opus-tagging/spec.md`)**

      | Escenario | Test que lo cubre | Notas |
      |---|---|---|
      | Lectura del tag de un Opus existente | integración `test_cli_opus_show_existing_tag` (3312: pre-escritura con `lofty`, `show` con los mismos nombres de campo) | Nuevo (3.2) |
      | La misma función lee Vorbis y Opus | Todos los `test_cli_opus_*` releen con `id3cli::read_vorbis_tag` (misma firma y retorno `VorbisComments`) y los tests OGG usan la misma función (p. ej. 3091) | Contrato de tipos verificado en compilación y ejecución |
      | La escritura despacha por el contenedor real (sigue siendo Opus) | integración `test_cli_opus_remove_all_preserves_vendor` (3516-3554: patrón `OggS` y primer paquete `OpusHead` intactos tras escribir y borrar) + `test_cli_opus_edit_title_artist_album_then_read_back` (3349) | `write_vorbis_tag` intacta (4.2) |
      | Escritura de las claves de texto básicas (TITLE/ARTIST/ALBUM) | integración `test_cli_opus_edit_title_artist_album_then_read_back` (3349: relectura independiente) | Nuevo (3.2) |
      | Escritura del año y de la fecha en claves separadas | Backend OGG compartido: integración `test_cli_ogg_edit_then_show_roundtrip` (3062: `--year 2026` → `YEAR`, `--date 2026-10-13` → `DATE`, sin absorberse) | Sin test CLI directo sobre `.opus`; mismo backend |
      | **Número de pista y de temporada (N/T → TRACKNUMBER/TRACKTOTAL, DISCNUMBER/DISCTOTAL)** | integración `test_cli_opus_edit_track_season_totals` (3393) + OGG: `test_cli_track_con_total_en_los_tres_formatos` (5093), `test_cli_temporada_con_total_en_los_tres_formatos` (5125) y unit `test_ogg_escribe_total_de_pista` (4318), `test_ogg_conserva_total_preexistente` (4358), `test_ogg_sin_total_no_inventa_total` (4394) | El 3393 es el test directo sobre `.opus` |
      | Múltiples artistas unidos con `"; "` | unit `test_multiple_artists` (340) y `test_multiple_artists_unicode` (359) + integración `test_cli_multiple_artists` (235) | Unión en el backend compartido, sin distinción de códec |
      | Flujo completo de `show`, `edit` y `remove` sobre un `.opus` | integración `test_cli_opus_show_edit_remove_flow` (3562: show vacío → edit → show → remove title) + `test_cli_opus_edit_cover_then_read_back` (3432) y `test_cli_opus_edit_track_season_totals` (3393) para las flags por separado | El `edit` con todas las flags a la vez no está en un único test: cubierto por composición + manual 5.4 (title/artist/album/track/season) |
      | **Omisión avisada de `--url` y `--original-artist`** | unit del backend OGG compartido `test_ogg_omite_url_y_artista_original_con_aviso` (3341: dos avisos en `warnings`, ninguna clave nueva) | Sin test CLI directo sobre `.opus` (los `test_cli_opus_*` no pasan esos flags); verificado manualmente (auditoría): Opus enruta al mismo `Backend::Ogg` |
      | La omisión no impide el resto de la edición | `test_ogg_omite_url_y_artista_original_con_aviso` (3356-3364: `--title` sí se escribe, `report.changed`) | Ídem |
      | El campo omitido no reaparece en la visualización | `test_ogg_omite_url_y_artista_original_con_aviso` (3386-3397: `after_keys == expected`) + unit `test_display_tags_ogg_no_muestra_vendor_ni_encoder` (3700) | Ídem |
      | **Letra en `LYRICS` sin código de idioma** | unit `test_ogg_letra_en_lyrics` (3531) | Sin test CLI directo sobre `.opus`; verificado manualmente (auditoría) |
      | Visualización de la letra | `test_ogg_letra_en_lyrics` (backend) + integración `test_cli_show_lyrics` (897) para el camino de `show` del backend OGG compartido | Ídem |
      | **Carátula JPEG y PNG en `METADATA_BLOCK_PICTURE`** | integración `test_cli_opus_edit_cover_then_read_back` (3432: MIME derivado de la extensión, relectura independiente) + unit `test_ogg_caratula_en_metadata_block_picture` (3403) | Nuevo (3.2) |
      | **Carátula WebP sin degradación** | unit `test_ogg_caratula_webp_sin_degradar` (3456) | Sin test CLI directo sobre `.opus`; verificado manualmente (auditoría) |
      | Imagen corta (dimensiones a cero) no aborta | unit `test_ogg_caratula_de_imagen_corta_no_aborta` (3493) | Backend compartido |
      | Extensión de imagen no soportada (rechazo antes de escribir) | unit `test_detect_mime_type_unsupported` (646) + integración `test_cli_cover_missing_file_fails` (2097) para el camino de error de carátula | Para `.opus` sin test directo; mismo helper `detect_cover_mime_type` compartido |
      | **Visualización de un Opus sin campos de usuario (vendor/ENCODER no se muestran)** | integración `test_cli_opus_show_edit_remove_flow` (3567-3577: «⚠️ No se encontraron metadatos.» con salida 0) + compartido `test_cli_ogg_show_without_user_fields_says_no_metadata` (3608: no imprime `ffmpeg`, `encoder`/`ENCODER`) + unit `test_display_tags_ogg_no_muestra_vendor_ni_encoder` (3700) | — |
      | Visualización con campos presentes (sin claves ajenas) | integración `test_cli_opus_show_existing_tag` (3312), `test_cli_opus_edit_title_artist_album_then_read_back` (3376-3384) y `test_cli_opus_show_edit_remove_flow` (3584-3589) + unit `test_display_tags_ogg_no_muestra_vendor_ni_encoder` (3700) | — |
      | **Eliminación de un campo (`remove title`)** | integración `test_cli_opus_remove_field_keeps_rest` (3474: TITLE desaparece, ARTIST/ALBUM sobreviven) + OGG `test_cli_ogg_remove_field_keeps_rest` (3149) | Nuevo (3.2) |
      | Eliminación del número y del total como unidad | integración `test_cli_remove_track_elimina_numero_y_total` (5319: mp3/m4a/ogg) + unit `test_remove_tags_ogg_usa_los_alias_de_ingles_y_espanol` (3556) | Backend OGG compartido; el pair se borra junto |
      | Eliminación de la carátula | unit `test_remove_tags_ogg_usa_los_alias_de_ingles_y_espanol` (3586-3625: `remove carátula` elimina `METADATA_BLOCK_PICTURE`) | Ídem |
      | **`remove --all` vacía el comentario y preserva el vendor** | integración `test_cli_opus_remove_all_preserves_vendor` (3509: 0 campos, sin carátulas, vendor `ffmpeg`, `OggS`+`OpusHead` intactos) + OGG `test_cli_ogg_remove_all_empties_comment_and_preserves_vendor` (3191) + unit `test_remove_all_ogg_vacia_el_comentario_y_preserva_vendor_y_audio` (3645) | Nuevo (3.2) |
      | Nombres en español (`título`, `artista`, `carátula`) | unit `test_remove_tags_ogg_usa_los_alias_de_ingles_y_espanol` (3556: `title`/`título`, `track`, `season`, `carátula`) | Misma tabla de alias compartida por OGG y Opus |
      | Nombre de campo desconocido | unit `test_remove_unknown_tag` (571) + integración `test_cli_remove_unknown_tag_warns_and_does_not_modify` (2128) | — |
      | Claves de terceros conservadas (`ENCODER`, `MUSICBRAINZ_*`) | unit `test_ogg_ida_y_vuelta_conserva_claves_ajenas` (3275: clave ajena y `encoder` con su caja original tras `edit`) | Para `.opus` con `MUSICBRAINZ_*` sin test directo; 5.4 manual verificó `encoder=Lavc libopus` conservado tras `edit` |
      | El vendor se conserva | unit `test_ogg_ida_y_vuelta_conserva_claves_ajenas` (3312-3313: vendor `ffmpeg` tras `edit`) + `test_cli_opus_remove_all_preserves_vendor` (3540) | — |
      | `edit` sin ningún flag no reescribe | unit `test_edit_ogg_sin_cambios_no_reescribe` (3319: byte a byte idéntico) | Sin test directo sobre `.opus`; backend compartido |
      | **Contenedor Opus corrupto (salida ≠ 0, aviso en español, archivo intacto)** | unit `test_detect_format_ogg_incomplete_page` (1503) y `test_detect_format_ogg_empty_packet` (1532) en la detección + integración caso `ogg_sin_paquete` (2853) por la CLI: salida ≠ 0 y archivo sin cambios | La ruta de error de parsing de `lofty` sobre un `.opus` corrupto no tiene test dedicado: los vectores de detección impiden llegar a la lectura, y el caso CLI `ogg_sin_paquete` cubre el mismo camino de error (salida ≠ 0, mensaje en español, archivo intacto) |
      | **Extensión `.opus` con contenido no Ogg** | detección por contenido: `test_detect_format_ogg_opus` (1470, la extensión nunca decide) + `test_detect_format_unrecognized_content` (1353) y `test_cli_rejects_unrecognized_content` (2929) para «no se pudo determinar» con salida ≠ 0 y archivo intacto | La extensión es irrelevante: la detección es por bytes (6.1 del change) |
- [x] 6.4 Cerrar con la evidencia fechada de todo lo anterior dentro del change.
      Verificación: todas las casillas de este fichero marcadas y la evidencia fechada registrada, lista para `openspec archive opus-support` **solo en el PR final**, tras la aprobación del usuario.
      Evidencia (2026-10-16): cierre del change `opus-support`:
      - **Suite completa**: 281 tests (163 unitarios + 118 de integración), 0 fallos, con `cargo test --no-fail-fast` (evidencia 6.1); baseline 271 (el change añade 1 unitario + 7 de integración + el renombrado del test de coherencia documental no altera el recuento).
      - **`cargo fmt --check`** limpio («FMT CLEAN», exit 0) y **`cargo clippy --all-targets --all-features --locked -- -D warnings`** limpio (evidencia 6.1), ambos con `--locked` para que el lock de `Cargo.toml`/`Cargo.lock` se valide en CI.
      - **Estado del change**: `openspec status --change opus-support` → «All planning artifacts complete!», 4/4 artefactos (proposal, specs, design, tasks) — salida literal en 6.2.
      - **Repaso escenario → test**: tabla completa de los 45 escenarios de `specs/format-detection/spec.md` y `specs/opus-tagging/spec.md` anexada en 6.3; todo nombre de test citado verificado con `grep` antes de citarlo; los escenarios sin test CLI directo sobre `.opus` (omisión de `--url`/`--original-artist`, letra `LYRICS`, carátula WebP, claves de terceros con `MUSICBRAINZ_*`, `edit` sin flags, contenedor corrupto en el parsing de `lofty`, archivo inexistente) quedan cubiertos por los unit del backend OGG compartido (Opus enruta al mismo `Backend::Ogg` y a la misma `read_vorbis_tag` despachada) y/o marcados como verificados manualmente (evidencia 5.4 y auditoría).
      - **Archivo del change**: `openspec archive opus-support` se ejecutará **solo en el PR final**, tras la aprobación del usuario; nunca antes.
      - **Casillas**: verificación final de que 1.1–6.4 están todas `- [x]` (grep `^- \[x\] $` sobre este fichero: **19 casillas marcadas** — 1.1, 1.2, 2.1, 2.2, 2.3, 3.1, 3.2, 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 5.4, 6.1, 6.2, 6.3, 6.4 — y ninguna sin marcar).