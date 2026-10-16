# Evidencia de verificación — documentación interna (tareas 6.4 y 6.5)

> [!note] Por qué estos dos ficheros se verifican con `grep` y no con un test
> `AGENTS.md` y `.github/copilot-instructions.md` **no viajan** dentro del paquete publicado: la lista blanca `include` de `Cargo.toml` no los incluye. Un `include_str!` de la suite sobre ellos rompería la verificación del paquete publicado (el fichero no existiría en el `.crate`), así que su coherencia MUST comprobarse con **evidencia registrada** dentro del propio cambio y **no** con un test de la suite. Los ficheros que sí viajan en el paquete (`README.md`, `Cargo.toml`) se cubren con test en las tareas 6.2 y 6.3.

- **Fecha de la comprobación:** 2026-10-15
- **Rama:** `feature/ogg-support-and-track-totals`
- **Ficheros verificados:** `AGENTS.md`, `.github/copilot-instructions.md`
- Comprobaciones: (a) tres formatos soportados y OGG fuera de la lista de rechazados; (b) convenciones por formato; (c) degradaciones por formato; (d) recuento de tests correcto sin rastro de `219` ni `124`.

## 1. `AGENTS.md`

### (a) Los tres formatos figuran como soportados y OGG no está entre los rechazados

```console
$ grep -n 'MP4 / M4A' AGENTS.md
3:Single-crate Rust CLI (`edition 2024`, `lofty` + `clap` derive) that reads/writes metadata on **MP3** (tags ID3v2.4), **MP4 / M4A** (átomos `ilst` estilo iTunes) y **OGG Vorbis** (comentarios de Vorbis, pares `CLAVE=valor`).

$ grep -n 'OGG Vorbis' AGENTS.md
3:Single-crate Rust CLI (`edition 2024`, `lofty` + `clap` derive) that reads/writes metadata on **MP3** (tags ID3v2.4), **MP4 / M4A** (átomos `ilst` estilo iTunes) y **OGG Vorbis** (comentarios de Vorbis, pares `CLAVE=valor`).
25:  - **OGG Vorbis** (`lofty::ogg::tag::VorbisComments` + `VorbisFile`): `edit_ogg()`, `apply_metadata_ogg()`, `read_vorbis_tag()`, `write_vorbis_tag()`, `remove_tags_ogg()`, `remove_all_ogg()`, `display_tags_ogg()`, y la tabla `VORBIS_COMMENT_MAP` con `VorbisCommentMapping::{for_flag, for_key}`

$ grep -cE 'no soportados \([^)]*OGG' AGENTS.md
0

$ grep -n 'Formatos no soportados' AGENTS.md
43:- Formatos no soportados (Opus, Speex, FLAC, WAV, AIFF, APE y WavPack): salida distinta de cero, mensaje en español y archivo intacto. El contenedor Ogg **no** se rechaza por ser Ogg: se rechaza cuando el primer paquete de su códec no es Vorbis (Opus, Speex, FLAC, Theora…)
```

El `0` de `grep -cE` demuestra que OGG no aparece dentro del paréntesis de la lista de formatos no soportados; la línea 43 muestra que la lista deja Opus, Speex, FLAC, WAV, AIFF, APE y WavPack, y explica que el contenedor Ogg solo se rechaza cuando su primer paquete no es Vorbis.

### (b) Las convenciones por formato están descritas

```console
$ grep -n 'ID3v2.4 through\|VorbisComments\|VORBIS_COMMENT_MAP' AGENTS.md
25:  - **OGG Vorbis** (`lofty::ogg::tag::VorbisComments` + `VorbisFile`): `edit_ogg()`, `apply_metadata_ogg()`, `read_vorbis_tag()`, `write_vorbis_tag()`, `remove_tags_ogg()`, `remove_all_ogg()`, `display_tags_ogg()`, y la tabla `VORBIS_COMMENT_MAP` con `VorbisCommentMapping::{for_flag, for_key}`
26:  - **Común**: `detect_format()`, `detect_format_from_bytes()`, `detect_mime_type()`, `detect_cover_mime_type()`, `MP4_ATOM_MAP`, `VORBIS_COMMENT_MAP`
44:- MP3 tags are always written as ID3v2.4 through `lofty`; MP4 metadata is written as iTunes-style `ilst` atoms y OGG como comentarios de Vorbis (tabla de mapeo de cada formato en README.md)
45:- OGG se escribe con el tipo concreto `VorbisComments`, no con el `Tag` genérico —que descarta las claves sin mapeo y normaliza la caja de las que sí mapean—; `VORBIS_COMMENT_MAP` es la única fuente del mapeo. El comentario es obligatorio en el contenedor: el *vendor* y `ENCODER` no se muestran como metadatos y `remove --all` los preserva
```

### (c) Las degradaciones están descritas

```console
$ grep -n 'Degradaciones documentadas' AGENTS.md
47:- Degradaciones documentadas en MP4: carátula WebP en `covr` con tipo de dato reservado (0) más aviso por stderr; `©lyr` sin código de idioma; `--url` y `--original-artist` se omiten con aviso y nunca se escriben como átomo *freeform*
48:- Degradaciones documentadas en OGG: la carátula WebP **no** se degrada (se escribe en `METADATA_BLOCK_PICTURE` con su MIME real y sin aviso, a diferencia de `covr` en MP4); `LYRICS` pierde el código de idioma, igual que `©lyr` en MP4; `--url` y `--original-artist` se omiten con aviso por stderr y nunca se escribe una clave inventada. En MP3 no ocurre ninguna de las tres

$ grep -n -- '--track` y `--season`' AGENTS.md
49:- `--track` y `--season` aceptan `N` o `N/T`. En MP3 y OGG el total se guarda como **texto** (`TRCK`/`TPOS` como `n/total`; `TRACKTOTAL`/`DISCTOTAL` como claves) y admite el cero, mientras que en MP4 el átomo guarda el par en 16 bits y el CLI avisa por stderr cuando el número o el total no son representables (`1..=65535`)
```

### (d) El recuento declarado es el correcto y no queda rastro de `219` ni de `124`

```console
$ grep -c '219' AGENTS.md
0

$ grep -c '124' AGENTS.md
0

$ grep -n '271' AGENTS.md
9:cargo test                       # 271 tests (160 unit + 111 integración)
```

## 2. `.github/copilot-instructions.md`

### (a) Los tres formatos figuran como soportados y OGG no está entre los rechazados

```console
$ grep -n 'Big picture' .github/copilot-instructions.md
6:- **Big picture:** single-binary Rust CLI that reads/writes metadata on **MP3** (ID3v2.4 tags), **MP4 / M4A** (iTunes-style `ilst` atoms) and **OGG Vorbis** (Vorbis comments, `KEY=value` pairs). The project is organized as:

$ grep -cE 'Unsupported formats \([^)]*OGG' .github/copilot-instructions.md
0

$ grep -n 'Unsupported formats' .github/copilot-instructions.md
56:  - Unsupported formats (Opus, Speex, FLAC, WAV, AIFF, APE, WavPack): exit with a non-zero code, a Spanish message, and the file is left **untouched**. Ogg is not rejected by itself: an Ogg container is rejected only when the codec of its first packet is not Vorbis (Opus, Speex, FLAC, Theora…)
```

### (b) Las convenciones por formato están descritas

```console
$ grep -n 'always written as ID3v2.4\|VorbisComments\|VORBIS_COMMENT_MAP' .github/copilot-instructions.md
16:- **Architecture & data flow:** the CLI parses paths/arguments with clap's derive macro, detects the audio format from the file's content, and dispatches to the MP3 (ID3v2.4), MP4 (`ilst`) or OGG (`VorbisComments`) code path in `src/lib.rs`. Output goes to stdout or to the file on disk. There are no web services, databases, or background workers — changes are local file I/O.
19:  - **src/lib.rs** (1607 lines) - All business logic: `apply_metadata()`, `apply_metadata_mp4()`, `apply_metadata_ogg()`, `edit_mp4()`, `edit_ogg()`, `read_mp4_tag()`, `read_vorbis_tag()`, `write_mp4_tag()`, `write_vorbis_tag()`, `remove_tags()`, `remove_tags_mp4()`, `remove_tags_ogg()`, `remove_all_id3()`, `remove_all_mp4()`, `remove_all_ogg()`, `display_tags()`, `display_tags_mp4()`, `display_tags_ogg()`, `add_lyrics()`, `add_url()`, `add_cover_art()`, `add_apple_metadata()`, `detect_format()`, `detect_format_from_bytes()`, `detect_mime_type()`, `detect_cover_mime_type()`, `MP4_ATOM_MAP`, `VORBIS_COMMENT_MAP`
48:  - MP3 tags are always written as ID3v2.4; MP4 metadata is written as iTunes-style `ilst` atoms; OGG is written as Vorbis comments through the concrete `VorbisComments` type (not the generic `Tag`, which drops unmapped keys and normalizes the case of the ones that do map), with `VORBIS_COMMENT_MAP` as the single source of the mapping; the Vorbis comment is mandatory, so its vendor and `ENCODER` are not shown as metadata and `remove --all` preserves them
64:  - OGG Vorbis flag→key mapping, the single source being `VORBIS_COMMENT_MAP` (18 pairs) plus the pair totals `TRACKTOTAL`/`DISCTOTAL`:
```

La línea 64 encabeza el mapeo OGG; las 18 claves y su detección por códec quedan en las líneas 65–66 (véase la comprobación (c) y el listado completo del mapeo más abajo).

### (c) Las degradaciones están descritas

```console
$ grep -ni 'degradation' .github/copilot-instructions.md
58:  - MP3 uses ID3 frames; MP4 uses `ilst` atoms under `moov.udta.meta.ilst`; OGG uses Vorbis comments (`KEY=value`). The flag→frame/atom/key mapping and the known degradations are documented in detail in `README.md`:
66:    - OGG degradations: a WebP cover is written in `METADATA_BLOCK_PICTURE` with its real MIME and **no** warning (unlike `covr` in MP4); `LYRICS` has no language code (like `©lyr`); `--url` and `--original-artist` have no key and are omitted with a warning, never written under an invented key, exit code 0

$ grep -n 'Track/season totals' .github/copilot-instructions.md
49:  - Track/season totals are stored per format: MP3 and OGG keep the total as **text** (`TRCK`/`TPOS` as `n/total`; `TRACKTOTAL`/`DISCTOTAL` keys) and accept zero, while the MP4 atom stores the pair in 16 bits and the CLI warns on stderr when the number or the total is not representable (`1..=65535`)
```

### (d) El recuento declarado es el correcto y no queda rastro de `219` ni de `124`

```console
$ grep -c '219' .github/copilot-instructions.md
0

$ grep -c '124' .github/copilot-instructions.md
0

$ grep -c '214' .github/copilot-instructions.md
0

$ grep -c '122' .github/copilot-instructions.md
0

$ grep -n '271' .github/copilot-instructions.md
29:  - Run all tests: `cargo test` (271 = 160 unit + 111 integration); use `--no-fail-fast` if more than one suite fails, since `cargo test` stops at the first failing binary
80:  2. Run `cargo test` to verify all 271 tests pass (160 unit + 111 integration)
95:  - Preserve existing coverage — currently 271 tests (160 unit + 111 integration)
```

Además del `219` y el `124` pedidos, se comprueban las cifras obsoletas que arrastraba este fichero (`214`, `122`): también dan `0`.

## 3. Mapeo OGG añadido a `.github/copilot-instructions.md` (18 claves)

```console
$ grep -n -e '--title`→`TITLE' .github/copilot-instructions.md
65:    - `--title`→`TITLE`, `--artist`→`ARTIST`, `--album`→`ALBUM`, `--album-artist`→`ALBUMARTIST`, `--year`→`YEAR`, `--date`→`DATE`, `--genre`→`GENRE`, `--track`→`TRACKNUMBER` (+`TRACKTOTAL`), `--season`→`DISCNUMBER` (+`DISCTOTAL`), `--composer`→`COMPOSER`, `--copyright`→`COPYRIGHT`, `--lyrics`→`LYRICS`, `--cover`→`METADATA_BLOCK_PICTURE`, `--compilation`→`COMPILATION` (`"1"`), `--title-sort`→`TITLESORT`, `--artist-sort`→`ARTISTSORT`, `--album-sort`→`ALBUMSORT`, `--subtitle`→`SUBTITLE`

$ grep -n 'codec' .github/copilot-instructions.md
55:  - Format is decided by **content** (magic bytes), never by extension: `ftyp` at offset 4 → MP4; an `ID3` header or a valid MPEG frame sync → MP3; an `OggS` page whose first packet starts with `\x01vorbis` → OGG Vorbis (the Ogg container is decided by the **codec** of its first packet, not by the page header)
56:  - Unsupported formats (Opus, Speex, FLAC, WAV, AIFF, APE, WavPack): exit with a non-zero code, a Spanish message, and the file is left **untouched**. Ogg is not rejected by itself: an Ogg container is rejected only when the codec of its first packet is not Vorbis (Opus, Speex, FLAC, Theora…)
```

## 4. Recuento real de tests medido

Se cuenta la línea cuyo `trim()` es exactamente `#[test]` (atributo de test), que es el mismo criterio que usa el helper `contar_tests` de `src/tests.rs`:

```console
$ grep -c '^[[:space:]]*#\[test\][[:space:]]*$' src/tests.rs
160

$ grep -c '^[[:space:]]*#\[test\][[:space:]]*$' tests/integration_test.rs
111
```

`160 (unitarios) + 111 (integración) = 271`.

> [!note] Por qué no se usa `grep -c '#\[test\]'` a secas
> `src/tests.rs` contiene el helper `contar_tests`, cuyo comentario y cadena literal mencionan el token `#[test]` (`/// Cuenta las líneas cuyo \`trim()\` es exactamente \`#[test]\`` y `.filter(|linea| linea.trim() == "#[test]")`). La búsqueda cruda cuenta esas dos líneas de más y da `162`; la coincidencia de línea exacta da el número real de atributos `#[test]`:
> ```console
> $ grep -c '#\[test\]' src/tests.rs
> 162
> ```
> El desglose `160 + 111 = 271` es el que declaran ambos documentos.

## 5. Conclusión

- Los tres backends/caminos de formato (MP3 ID3v2.4, MP4 `ilst`, OGG `VorbisComments`) figuran en ambos documentos con sus funciones asociadas.
- Las convenciones por formato, las degradaciones y la asimetría del total de pista/temporada están descritas.
- OGG no aparece en ninguna lista de formatos rechazados; el contenedor Ogg se explica como decidido por el códec de su primer paquete.
- El recuento declarado (`271 = 160 + 111`) coincide con el medido y no queda rastro de `219`, `124`, `214` ni `122`.
