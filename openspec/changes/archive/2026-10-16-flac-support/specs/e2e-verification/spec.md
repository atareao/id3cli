# Spec Delta

## Purpose

La matriz de verificación e2e incorpora FLAC como quinto formato: los mismos 18 tags escritos y releídos (con `url` y `original_artist` omitidos con aviso por stderr), la carátula JPG y PNG con roundtrip de bytes y la sustitución en una segunda edición, y `remove --all` que elimina el bloque de comentario completo (vendor NO conservado en FLAC, a diferencia de Ogg/Opus — degradación documentada). La tabla de clave ffprobe de FLAC se fija durante GREEN a partir de la medición real (nunca supuesta), siguiendo el patrón ya usado para OGG/Opus.

## ADDED Requirements

### Requirement: El e2e MUST ejercitar los 20 tags de VALID_TAG_NAMES con su forma por formato (FLAC)

Sobre un FLAC recién generado, el e2e MUST verificar exactamente la misma forma que OGG y Opus: **18 tags** escritos y releídos más `url` y `original_artist` verificados como **omitidos con aviso por stderr** y ausentes del lector independiente. El flujo MUST empezar con un `show` inicial que no muestre ningún campo de usuario.

#### Scenario: Los 18 tags de FLAC escritos y releídos
- **WHEN** se ejecuta `e2e_flac.sh` y este edita el `.flac` con los 18 tags (sin `url` ni `original_artist`)
- **THEN** `show` MUST releerlos con los mismos valores y el lector independiente MUST exponerlos según la tabla de clave ffprobe de FLAC

#### Scenario: url y original_artist omitidos con aviso en FLAC
- **WHEN** el mismo script indica también `--url` y `--original-artist`
- **THEN** MUST emitirse un aviso por stderr para cada uno, MUST no escribirse ninguna clave inventada y el lector independiente MUST no exponerlos

### Requirement: La verificación MUST hacerse en dos capas con una tabla de clave ffprobe por formato (FLAC medido)

La tabla de entradas de ffprobe de FLAC SHALL fijarse durante GREEN con una clave **real medida** sobre un archivo escrito por `id3cli` con ese campo —registrada en tasks.md— y reflejar: el conjunto de entradas (`format_tags` o `stream_tags`), la caja de las claves (Vorbis en mayúsculas o normalizadas), el par `track`/`disc` con sus totales (`TRACKTOTAL`/`DISCTOTAL`) y la ausencia de `url`/`original_artist`. Si para algún campo `ffmpeg` no expone una clave distinguible, la celda se marca «—» y ese campo se verifica por `show`, igual que se hace en los demás formatos.

#### Scenario: Las celdas de FLAC son las medidas
- **WHEN** se consulta la tabla del spec con las claves de FLAC
- **THEN** cada celda MUST proceder de la medición registrada en tasks.md (con fecha y versión de `ffmpeg`), no de una suposición

> [!note] Medición GREEN 2026-10-16 (tarea 6.3 de `tasks.md`, ffmpeg/ffprobe n9.0.2): FLAC expone la metadata en **`format_tags`** (el `stream_tags` queda vacío, a diferencia de OGG/Opus) con la caja Vorbis **preservada** (`TITLE`, `ARTIST`, `ALBUM`, `YEAR`, `DATE`, `GENRE`, `COPYRIGHT`, `COMPOSER`, `SUBTITLE`, `LYRICS`, `COMPILATION`, `TITLESORT`, `ARTISTSORT`, `ALBUMSORT`), y con las mismas normalizaciones a minúsculas que OGG/Opus: `TRACKNUMBER`→`track`, `DISCNUMBER`→`disc` y `ALBUMARTIST`→`album_artist`; los totales viven en `TRACKTOTAL`/`DISCTOTAL`, que sí se exponen. `url`/`original_artist` → «—» (omitidos con aviso por stderr, literal generalizado «en los comentarios de Vorbis»; MUST NO aparecer). El par `--year`/`--date` son claves separadas (`YEAR`/`DATE`), por lo que el año se fija por el par clave+valor crudo (`YEAR=…`), como en OGG/Opus. Tras `remove --all` el bloque VORBIS_COMMENT se elimina completo (vendor NO conservado) y `format_tags` queda vacío.

### Requirement: La carátula MUST verificarse como roundtrip de bytes JPG y PNG en FLAC y mostrarse por show

El e2e MUST escribir la carátula `cover.jpg` y la `cover.png` (generadas con `ffmpeg testsrc`) sobre FLAC recién generado, una por archivo, y verificar el roundtrip extrayendo el stream de imagen con `ffmpeg -map 0:v:0 -c copy -frames:v 1` y comparando byte a byte contra la imagen original. Además, sobre el MISMO archivo, una segunda edición de `--cover` con la otra imagen MUST dejar exactamente una carátula (la última), verificada por `show` («Carátulas: 1 imagen(es)», mostrada desde los bloques PICTURE), por roundtrip contra la última imagen y por la ausencia de la primera (`check_cover_not_roundtrip`).

#### Scenario: Roundtrip JPG y PNG en FLAC
- **WHEN** `e2e_flac.sh` escribe `cover.jpg` y `cover.png` sobre FLAC recién generado y extrae el stream de imagen
- **THEN** la extracción MUST coincidir byte a byte con la imagen original en los dos casos

#### Scenario: Segunda edición de carátula sustituye en FLAC
- **WHEN** el mismo archivo recibe `edit --cover imagen_a` y después `edit --cover imagen_b`
- **THEN** `show` MUST reportar «Carátulas: 1 imagen(es)», el roundtrip contra `imagen_b` MUST pasar y el anti-roundtrip contra `imagen_a` MUST fallar si el formato acumulara

### Requirement: remove --all en FLAC elimina el bloque de comentario y deja el audio reproducible

`e2e_flac.sh` MUST verificar `remove --all`: el comentario queda sin campos de usuario ni carátulas y el bloque VORBIS_COMMENT desaparece del archivo (el vendor NO se conserva en FLAC — degradación verificada, a diferencia de Ogg/Opus donde el vendor se preserva). El archivo MUST seguir siendo un FLAC reproducible por `ffprobe` (codec `flac`, duración aproximada esperada). El script MUST verificar también `remove` de un campo concreto con el resto intacto.

#### Scenario: remove --all deja el FLAC reproducible sin bloque de comentario
- **WHEN** se ejecuta `remove --all` sobre el `.flac` editado
- **THEN** el archivo MUST quedar sin metadatos de usuario, sin carátulas y sin bloque VORBIS_COMMENT (vendor perdido, degradación documentada), y `ffprobe` MUST reconocerlo como audio FLAC reproducible (duración ≈1,5 s)