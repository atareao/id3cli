# Proposal

## Why

El tooling e2e local recién fusionado (285 aserciones en verde sobre los cuatro formatos) fue diseñado con dos capas de verificación —`id3cli show` como releído del propio CLI y `ffprobe` como lector independiente de terceros— precisamente para destapar discrepancias que la suite Rust no ve. Y ha destapado dos, ambas reproducidas con el binario de `development` (2026-10-16):

1. **MP4: la metadata es invisible a los demuxers externos tras `remove --all` + `edit`.** Reproducción: `edit --title "Antes"` → `remove --all` → `edit --title "Despues"` sobre un `.m4a`; `id3cli show` lee `Despues`, pero `ffprobe -show_entries format_tags` solo lista los brands (`major_brand`, `minor_version`, `compatible_brands`) — ni `title` ni `artist`. Es decir, las etiquetas existen y son legibles por `lofty`, pero los reproductores externos (VLC, QuickTime, navegadores) las percibirían como perdidas. La causa raíz identificada: `lofty` 0.25.4 (última versión publicada, sin fix upstream) reconstruye el átomo `meta` SIN su hijo `hdlr` cuando reescribe un archivo cuyo `ilst` fue eliminado con `remove --all`; el demuxer de ffmpeg exige ese `hdlr` para parsear `ilst`.
2. **MP3: `edit --cover` acumula carátulas en vez de sustituirlas.** Reproducción: `edit --cover cover1.jpg` → `edit --cover cover2.jpg` sobre un MP3 da `Carátulas: 2 imagen(es)`, ambas `CoverFront` (2161 + 2146 bytes). `lofty` `insert_picture` no deduplica; el CLI debe sustituir la carátula del tipo indicado.

## What Changes

- **Fix 1.1 (MP4, `fix:`):** garantizar que toda escritura MP4 del CLI produce un árbol `moov.udta.meta` con su hijo `hdlr` válido, de modo que un lector de terceros (`ffprobe format_tags`) vea las etiquetas escritas tras cualquier secuencia de operaciones, incluida `remove --all` + `edit`. Implementación a decidir en el design con evidencia (post-proceso del archivo tras `save_to_path`, o manipulación del árbol en el camino de escritura). Verificación: test de integración RED que reproduce el flujo y comprueba con ffprobe (o parseo ligero de bytes: el patrón `hdlr` presente en el árbol `meta`) → fix → verde; el e2e `e2e_m4a.sh` amplía su flujo con esta aserción.
- **Fix 1.2 (MP3, `fix:`):** `edit --cover` sobre un MP3 que ya tiene carátula MUST sustituir la carátula del mismo tipo (`CoverFront`) por la nueva en lugar de acumular: eliminar los frames `APIC` de tipo `CoverFront` existentes antes de insertar la nueva imagen. Las carátulas de otros tipos (p. ej. `BackCover`) se conservan. Verificación: test de integración RED (doblete de `--cover` → exactamente 1 `APIC` CoverFront con los bytes de la última imagen) → fix → verde; `e2e_mp3.sh` amplía su flujo con esta aserción.
- **Generalización del fix 1.2 (a confirmar por evidencia):** el mismo patrón de acumulación puede afectar a MP4 (`push_mp4_cover`) y OGG/Opus (`push_ogg_cover`), que también usan inserción de pictures de `lofty`. Si el test RED por formato demuestra la acumulación, la semántica «sustituir la carátula del tipo indicado» MUST generalizarse a todos los backends en este mismo cambio; si algún formato ya sustituye, se documenta y el e2e lo fija con su aserción de «segunda edición sustituye».
- **e2e ampliado:** `e2e_m4a.sh` incluye el flujo `edit → remove --all → edit` y verifica que `ffprobe format_tags` vuelve a listar las etiquetas; `e2e_mp3.sh` (y los formatos que la evidencia indique) verifica que una segunda edición de `--cover` sustituye en vez de acumular.
- **Sin cambios de formato:** no se añade ni elimina ningún tag soportado; el CLI sigue escribiendo exactamente los mismos frames/átomos/claves, solo con la semántica corregida de sustitución y la estructura `meta` completa.

## Capabilities

### New Capabilities

Ninguna: el cambio corrige el comportamiento de contenedores ya soportados y no añade formato, tag ni superficie CLI nueva.

### Modified Capabilities

- `m4a-tagging`: se modifica el requirement de creación del árbol `moov.udta.meta.ilst` para exigir el hijo `hdlr` en toda escritura y la visibilidad de las etiquetas por un lector de terceros (ffprobe) tras cualquier secuencia de operaciones, incluida `remove --all` + `edit`.
- `mp3-tagging`: se modifica el requirement de carátulas para fijar la semántica de sustitución: `edit --cover` sobre un MP3 con carátula `CoverFront` MUST sustituirla en lugar de acumular, conservando las carátulas de otros tipos.
- `e2e-verification`: se amplía la matriz con la aserción de visibilidad externa MP4 tras `remove --all` + `edit` y la aserción de sustitución de carátula en los formatos afectados.

## Impact

- **Código:** `src/lib.rs` (camino de escritura MP4 y `add_cover_art`/inserción de carátula MP3, y según evidencia los caminos `push_mp4_cover`/`push_ogg_cover`). No cambia la tabla de alias, el mapeo de frames/átomos/claves ni ninguna firma pública.
- **Tests:** nuevos tests RED (integración, reproduciendo cada bug) y ampliación de los existentes; los 281 tests actuales (163 unit + 118 integración) MUST seguir pasando sin modificación, salvo los que fijen el bug corregido (que se actualizan a la nueva semántica con su propio RED→GREEN).
- **e2e:** `scripts/e2e/e2e_m4a.sh` y `scripts/e2e/e2e_mp3.sh` (y según evidencia `e2e_ogg.sh`/`e2e_opus.sh`) ganan aserciones; el recuento PASS pasa de 285 a 285 + N.
- **Documentación:** `README.md` (semántica de `--cover`: sustituye) y, si la generalización alcanza otros formatos, las tablas correspondientes.
- **Commits:** `fix:` (nunca `feat:` → no libera minor; verificado `release-prepare.yml:38`). PR a `development`. La release de estos fixes viaja con la del plan director (Fase 4), no se libera por separado.
- **No-rotura:** la suite Rust MUST seguir en verde completa; `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings` y `bash -n scripts/e2e/*.sh` limpios; `just e2e` en verde.