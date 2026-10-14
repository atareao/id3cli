# Proposal

## Why

`id3cli` solo entiende MP3. El crate `id3` 1.16.4 da acceso directo a frames ID3v2.4, pero no conoce el contenedor MP4 ni los metadatos estilo iTunes (átomos `ilst`), así que editar un `.m4a` hoy falla o produce un archivo corrupto. Además, hoy no existe detección de formato: el CLI entrega la ruta directamente al crate `id3`, de modo que sobre un archivo no-MP3 no encuentra tags y, al escribir, genera un tag ID3v2 en un contenedor que no lo entiende, invalidando el archivo para los reproductores MP4.

A esto se suma que `openspec/specs/` está vacío: no existe ninguna spec que fije el comportamiento actual del CLI. Antes de cambiar el backend hay que caracterizar lo que hoy funciona (MP3 con ID3v2.4, con 63 tests unitarios + 43 de integración en verde) para no perder paridad durante la migración.

Este cambio amplía el alcance a contenedores MP4 y, de paso, sustituye el backend de tags por `lofty` 0.25.4, un crate mantenido que cubre MP3 y MP4 con una única API y que ya resuelve el mapeo entre frames ID3 y átomos MP4 en su propia tabla `ILST_MAP`.

## What Changes

- Se sustituye el crate `id3` por `lofty` 0.25.4 como backend único de tags, también para MP3. `src/lib.rs` se reescribe contra la API de `lofty` y la API pública deja de exponer `id3::Tag`.
- Se añade detección de formato por magic bytes, independiente de la extensión del archivo: MP3 por cabecera `ID3` o por sincronía de frame MPEG; MP4 por la sola existencia del átomo `ftyp` en el offset 4, **sin lista blanca de brands**. La detección de MP3 sin cabecera `ID3` por sincronía de frame MPEG valida los campos de la cabecera (versión, layer, bitrate y frecuencia de muestreo) para evitar falsos positivos.
- Se añade lectura y escritura de metadatos estilo iTunes (`ilst`) en contenedores MP4: `.m4a`, `.mp4` y `.m4b`.
- Se mapean los flags del CLI a átomos MP4: `--title`→`©nam`, `--artist`→`©ART`, `--album`→`©alb`, `--album-artist`→`aART`, `--year`/`--date`→`©day`, `--genre`→`©gen`, `--track`→`trkn`, `--season`→`disk`, `--composer`→`©wrt`, `--copyright`→`cprt`, `--lyrics`→`©lyr`, `--cover`→`covr`, `--compilation`→`cpil`, `--title-sort`→`sonm`, `--artist-sort`→`soar`, `--album-sort`→`soal` y `--subtitle`→`----:com.apple.iTunes:SUBTITLE` (átomo freeform).
- Los campos sin equivalente en MP4 (`--url`, `--original-artist`) emiten un warning por stderr y se omiten: no se aborta, no se escriben átomos freeform y el resto de campos se escriben con normalidad.
- Cuando se pasan `--year` y `--date` sobre un M4A, ambos colisionan en el átomo `©day`: gana `--date` por ser el valor más específico y se emite un warning que indica que el año ha quedado absorbido por la fecha.
- Se documenta el cambio de comportamiento observable de la carátula: el átomo `covr` solo admite los tipos well-known JPEG (13), PNG (14), GIF (12) y BMP (27); WebP no existe en esa tabla y degrada a `Reserved(0)`, que iTunes/macOS no muestran, mientras que en MP3 el `.webp` sigue funcionando vía `APIC`.
- Se documenta que el átomo `©lyr` no tiene campo de idioma: la letra sí se conserva, pero el código `spa` del frame `USLT` en MP3 no tiene equivalente en M4A y por tanto no se puede preservar.
- Los formatos fuera de alcance (FLAC, OGG, Opus, WAV, AIFF, APE) hacen que el CLI salga con código distinto de cero y un mensaje claro en español, sin modificar el archivo.
- Se mantienen sin cambios la interfaz de flags del CLI, el separador `"; "` para múltiples artistas y el comportamiento de `remove` (nombres en inglés y español, `--all` borra físicamente el bloque ID3v2 y el tag ID3v1, `edit` sin cambios no reescribe el archivo).
- El mapeo de MP3 se conserva sin cambios, incluido `--year` → `TYER`, que está obsoleto en ID3v2.4 pero se conserva a propósito para que la migración de backend sea mecánica y la paridad sea verificable.

## Capabilities

### New Capabilities

- `format-detection`: detección del formato de audio del archivo por magic bytes (MP3 y MP4) con independencia de la extensión, y rechazo explícito de los formatos fuera de alcance con código de salida distinto de cero y mensaje en español.
- `mp3-tagging`: contrato de paridad del comportamiento de edición ID3v2.4 en MP3 durante la migración de backend (`id3` → `lofty`), incluyendo la escritura siempre en ID3v2.4, los frames de cada flag, el idioma `spa` del frame `USLT`, `remove --all` con borrado físico de ID3v1 e ID3v2 y la preservación de los tags existentes.
- `m4a-tagging`: lectura y escritura de metadatos estilo iTunes (átomos `ilst`) en contenedores MP4 (`.m4a`, `.mp4`, `.m4b`), con el mapeo completo de flags a átomos, la prioridad de `--date` sobre `--year`, la omisión avisada de los campos sin equivalente, la restricción de formatos de imagen de `covr`, el separador `"; "` para múltiples artistas y la creación del árbol `moov.udta.meta.ilst` cuando no existe.

### Modified Capabilities

Ninguna. `openspec/specs/` está vacío, así que no existe ninguna spec previa que modificar: este cambio es la caracterización inicial del proyecto y crea tres capacidades nuevas en lugar de modificar ninguna.

## Impact

- **Código afectado:** `src/lib.rs` (reescritura del backend y de toda la API de manipulación de tags), `src/main.rs` (selección de backend por formato, superficie de `clap`, mensajes en español), `src/tests.rs` (63 tests unitarios a adaptar) y `tests/integration_test.rs` (43 tests de integración a adaptar).
- **Dependencias:** entra `lofty` 0.25.4; sale `id3` 1.16.4.
- **API pública:** cambia la firma de `apply_metadata`, `add_lyrics`, `add_url`, `add_apple_metadata`, `add_cover_art`, `remove_tags`, `remove_all_id3` y `display_tags`, porque hoy reciben `&mut id3::Tag`.
- **`remove --all`:** hoy delega en `id3::v1v2::remove_from_path`; hay que reimplementar el borrado físico de ID3v1 e ID3v2 sobre `lofty` (verificar `TagExt::remove_from_path`) conservando los mensajes observables por estado previo.
- **Tests:** red de seguridad de referencia = 106 tests en verde (63 unitarios + 43 de integración). `AGENTS.md` y `README.md` citan "99 tests (59 unit + 40 integration)", cifra desactualizada que hay que corregir.
- **Fixtures:** el truco actual de fixture MP3 de 10 bytes (`create_temp_mp3()`) no sirve para MP4, que requiere una estructura `ftyp`+`moov`+`udta`+`meta`+`ilst` real. Se generan fixtures `.m4a` deterministas con `ffmpeg` (disponible en la máquina) y se commitean en `tests/fixtures/`.
- **Documentación:** `Cargo.toml` (description), `README.md`, `AGENTS.md` y los textos `about`/`help` de `clap`, que hoy dicen "MP3".
- **Comportamientos que se degradan:** `.webp` en carátulas M4A y el idioma de la letra en M4A.
- **`--year` en MP3:** la unificación de `--year` en `TDRC` con la regla de colisión ya aprobada para M4A se abordará en el cambio posterior e independiente `id3v24-date-unification`, fuera del alcance de este cambio.
- **Fuera de alcance:** FLAC, OGG, Opus, WAV, AIFF, APE, capítulos de `.m4b` y escritura de tags ID3 dentro de un contenedor MP4.
