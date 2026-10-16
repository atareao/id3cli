# Proposal

## Why

Opus es el códec Ogg moderno: buena parte de la música actual en contenedor Ogg viene codificada con él (audio de baja latencia, streaming y podcasts incluidos), y hoy cualquier colección `.opus` queda fuera de la herramienta porque el primer paquete `OpusHead` se reconoce únicamente para rechazarlo. El salto es barato: Opus comparte el contenedor Ogg y usa el mismo mecanismo de metadatos —los comentarios de Vorbis («VorbisComments», pares clave=valor)—, así que el coste es mínimo sobre el backend OGG que ya existe: la detección gana una variante, la lectura despacha a `OpusFile` y el resto del backend (mapeo, aplicación, carátula, borrado y visualización) se reutiliza sin cambios.

## What Changes

- **Detección.** El contenedor Ogg identifica su códec por el primer paquete de la primera página: cuando ese paquete empieza por `OpusHead` (8 bytes), el archivo pasa a reconocerse con la variante nueva `AudioFormat::OggOpus`, con cualquier extensión (`.opus`, `.ogg` o ninguna, incluido un archivo renombrado). El resto del reconocimiento por magic bytes no cambia. Speex, FLAC dentro de Ogg y el vídeo Theora siguen rechazados con el mensaje en español actual.
- **Backend.** Reutilización total del backend de comentarios de Vorbis: `VORBIS_COMMENT_MAP`, `apply_metadata_ogg`, la carátula en `METADATA_BLOCK_PICTURE`, `remove_tags_ogg`/`remove_all_ogg` y `display_tags_ogg` sirven igual para un `OpusFile`. `read_vorbis_tag` se generaliza: detecta el códec del contenedor (Vorbis u Opus) y despacha a `VorbisFile` u `OpusFile`, conservando el mismo nombre y el mismo tipo de retorno `VorbisComments`, de modo que ninguna llamada existente cambia. `write_vorbis_tag` no cambia: `tag.save_to_path(path, WriteOptions::default())` inspecciona el contenedor real del archivo y escribe el comentario en la posición que le corresponde a cada códec.
- **CLI.** Un único `Backend::Ogg` sirve Vorbis y Opus: `AudioFormat::OggOpus` se enruta al mismo backend y ninguna rama nueva distingue los dos códecs. Los mensajes de error pasan a «Formatos soportados: MP3, MP4, OGG y Opus.»
- **Tests.** Se invierten los tests que hoy esperan el rechazo de Opus —`test_detect_format_ogg_other_codecs` (unitario) y el caso `"opus"` de `test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched` (integración)— y se añade cobertura nueva de `show`, `edit` y `remove` sobre un `.opus` real: metadatos de texto, carátula, total `N/T` y borrado.
- **Fixture.** `tests/fixtures/sample.opus` versionado, generado con `ffmpeg` (`anullsrc` + `libopus`), más `tests/fixtures/generate_sample_opus.sh` que lo regenera y verifica su estructura, siguiendo el patrón del `.ogg`.
- **Documentación.** `README.md`, `AGENTS.md` y `.github/copilot-instructions.md`: Opus sale de las listas de formatos no soportados y entra en las de soportados, con su sección de mapeo y su hoja de ruta actualizadas.
- **Fuera de alcance (no-goals).** Speex, FLAC (nativo o dentro de Ogg) y el vídeo Theora dentro del contenedor Ogg, los capítulos de `.m4b`, los metadatos de audio Opus que no viajen en el comentario de Vorbis y la escritura de tags ID3 dentro de un contenedor Ogg.

## Capabilities

### New Capabilities

- `opus-tagging`: lectura y escritura de metadatos en Opus dentro de Ogg mediante los mismos comentarios de Vorbis que OGG: contrato de tipos (`OpusFile` con su `VorbisComments` interno), mapeo de claves, degradaciones, carátula, comentario obligatorio y borrado.

### Modified Capabilities

- `openspec/specs/format-detection/spec.md`: el contenedor Ogg reconoce dos códecs —`\x01vorbis` → Ogg Vorbis y `OpusHead` → Opus— y el mensaje de formato no soportado pasa a enumerar MP3, MP4, OGG y Opus.

## Impact

- **Código:** `src/lib.rs`, `src/main.rs`, `src/tests.rs` y `tests/integration_test.rs`.
- **Fixtures nuevos:** `tests/fixtures/sample.opus` y su script de regeneración `tests/fixtures/generate_sample_opus.sh`.
- **Documentación:** `README.md`, `AGENTS.md`, `.github/copilot-instructions.md`.
- **Specs del repositorio** (capability nueva y modificada listadas arriba).
- **Dependencias:** ninguna nueva; se mantiene `lofty` 0.25.4 y `clap` 4.5.
- **Versión:** sube automáticamente por el prefijo `feat:` del pipeline de release, sin tocarla a mano.
- **Tests existentes:** los tests de detección y de rechazo que hoy afirman que Opus no está soportado cambian de expectativa (se invierten a aceptación), y la suite gana los tests nuevos de Opus; el recuento declarado en la documentación se actualiza al medido.