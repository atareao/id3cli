# id3cli — AGENTS.md

Single-crate Rust CLI (`edition 2024`, `lofty` + `clap` derive) that reads/writes metadata on **MP3** (tags ID3v2.4), **MP4 / M4A** (átomos `ilst` estilo iTunes), **OGG** (comentarios de Vorbis, pares `CLAVE=valor`; el contenedor Ogg cubre Vorbis y Opus) y **FLAC** (comentarios de Vorbis, `fLaC` nativo).

## Commands

```sh
cargo build                      # debug
cargo test                       # 329 tests (178 unit + 151 integración)
cargo test --lib                 # unit tests only (src/tests.rs)
cargo test --test '*'            # integration tests only (tests/integration_test.rs)
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- edit <file> --title "X"    # run CLI with args
RUST_BACKTRACE=1 cargo run -- ...       # debug with backtrace
```

Nota: `cargo test` se detiene en el primer binario que falla — añade `--no-fail-fast` cuando falle más de una suite.

## Architecture

- **`src/lib.rs`** — all business logic, separada por formato:
  - **MP3** (`lofty::tag::Id3v2Tag`): `apply_metadata()`, `add_lyrics()`, `add_url()`, `add_cover_art()`, `add_apple_metadata()`, `remove_tags()`, `remove_all_id3()`, `display_tags()`
  - **MP4** (`lofty::tag::Tag` + `ItemKey`): `edit_mp4()`, `apply_metadata_mp4()`, `read_mp4_tag()`, `write_mp4_tag()`, `remove_tags_mp4()`, `remove_all_mp4()`, `display_tags_mp4()`
  - **OGG** (Vorbis y Opus; `lofty::ogg::tag::VorbisComments` + `VorbisFile`/`OpusFile`): `edit_ogg()`, `apply_metadata_ogg()`, `read_vorbis_tag()`, `write_vorbis_tag()`, `remove_tags_ogg()`, `remove_all_ogg()`, `display_tags_ogg()`
  - **FLAC** (`lofty::flac::FlacFile` + `VorbisComments`): `read_vorbis_tag` (despacha `Flac`), `display_tags_flac()`, `remove_all_flac()`, `read_flac_pictures()`
  - **Común**: `detect_format()`, `detect_format_from_bytes()`, `detect_mime_type()`, `detect_cover_mime_type()`, `MP4_ATOM_MAP`, `VORBIS_COMMENT_MAP`
- **`src/main.rs`** — thin CLI wrapper with clap subcommands: `show`, `edit`, `remove`
- **`src/output.rs`** — capa de escritura de la salida estándar: contiene el macro `outln!` (equivalente a `println!`) que trata el cierre del pipe como un final normal de la salida, y es el único sitio por el que se escribe en stdout
- **`src/tests.rs`** — unit tests (module of lib.rs via `#[cfg(test)]`)
- **`tests/integration_test.rs`** — integration tests spawning the binary via `Command`
- **Completions** — subcomando `completions <bash|zsh|fish>` en `src/main.rs`, enum `GenerationShell` + `generate_completions()` en `src/lib.rs` (clap_complete 4.5, salida nativa del generador), `print_block()` en `src/output.rs` (bloques multilínea con el mismo contrato de pipe cerrado)
- **Modo batch** — `show`/`edit`/`remove` aceptan `<FILE>...` (`Vec<PathBuf>`, `num_args = 1..`); cabecera `=== <ruta> ===` en `show` legible con N>1, NDJSON en `show --json` con N>1 (`batch_ndjson`), despacho por archivo con acumulación de fallos (`batch_dispatch`/`BatchOutcome`) y exit agregado (0 solo si todos OK)
- **`tests/fixtures/`** — `sample.mp3`, `sample.m4a`, `sample.ogg`, `sample.opus` y `sample.flac` reales, más `generate_sample_m4a.sh`, `generate_sample_ogg.sh`, `generate_sample_opus.sh` y `generate_sample_flac.sh`

## Key conventions

- Multiple artists joined with `"; "` separator
- Apple metadata via `.set_text()` frames: TCMP (`"1"`), TSOA, TSOP, TSOT
- Lyrics stored in USLT frame, language code `"spa"`
- Cover art MIME auto-detected from file extension: `.jpg`/`.jpeg` → `image/jpeg`, `.png` → `image/png`, `.webp` → `image/webp`
- `remove_tags()` accepts both English and Spanish names (title/título, cover/carátula, etc.)
- Error messages in Spanish
- Integration tests use `create_temp_mp3()`, `create_temp_m4a()`, `create_temp_ogg()`, `create_temp_opus()`, `create_temp_flac()` y `create_temp_fixture()` helpers + `cleanup_file()`
- El formato se decide por **contenido** (magic bytes), nunca por la extensión: `detect_format()` / `detect_format_from_bytes()`, y las lecturas de `lofty` vía `read_from()`, nunca `read_from_path()`. El contenedor Ogg se decide por el **códec de su primer paquete**, no por la cabecera de página: Vorbis y Opus están soportados
- Formatos no soportados (Speex, WAV, AIFF, APE y WavPack): salida distinta de cero, mensaje en español y archivo intacto. El contenedor Ogg **no** se rechaza por ser Ogg: se rechaza cuando el primer paquete de su códec no es de un códec soportado del contenedor (Speex, FLAC, Theora…)
- MP3 tags are always written as ID3v2.4 through `lofty`; MP4 metadata is written as iTunes-style `ilst` atoms y OGG como comentarios de Vorbis (tabla de mapeo de cada formato en README.md)
- OGG se escribe con el tipo concreto `VorbisComments`, no con el `Tag` genérico —que descarta las claves sin mapeo y normaliza la caja de las que sí mapean—; `VORBIS_COMMENT_MAP` es la única fuente del mapeo. El comentario es obligatorio en el contenedor: el *vendor* y `ENCODER` no se muestran como metadatos, `edit` conserva las claves ajenas (incluida `ENCODER`) y el vendor, y `remove --all` vacía el comentario y solo preserva el vendor (obligatorio, no se puede eliminar del contenedor)
- En MP4, `--year` y `--date` comparten el átomo `©day`: si se indican ambos gana `--date` y se avisa por stderr; `--subtitle` va a un átomo *freeform* (`----:com.apple.iTunes:SUBTITLE`)
- Degradaciones documentadas en MP4: carátula WebP en `covr` con tipo de dato reservado (0) más aviso por stderr; `©lyr` sin código de idioma; `--url` y `--original-artist` se omiten con aviso y nunca se escriben como átomo *freeform*
- Degradaciones documentadas en OGG: la carátula WebP **no** se degrada (se escribe en `METADATA_BLOCK_PICTURE` con su MIME real y sin aviso, a diferencia de `covr` en MP4); `LYRICS` pierde el código de idioma, igual que `©lyr` en MP4; `--url` y `--original-artist` se omiten con aviso por stderr y nunca se escribe una clave inventada. En MP3 no ocurre ninguna de las tres
- FLAC nativo se soporta por `fLaC` (offset 0); el códec `\x7fFLAC` dentro del contenedor Ogg NO: se rechaza como los demás códecs Ogg no soportados. La carátula de FLAC viaja como bloque PICTURE del archivo (`read_flac_pictures`), `remove --all` elimina el bloque VORBIS_COMMENT completo (vendor no conservado — degradación documentada), y `--url`/`--original-artist` se omiten con aviso igual que en OGG
- `--track` y `--season` aceptan `N` o `N/T`. En MP3 y OGG el total se guarda como **texto** (`TRCK`/`TPOS` como `n/total`; `TRACKTOTAL`/`DISCTOTAL` como claves) y admite el cero, mientras que en MP4 el átomo guarda el par en 16 bits y el CLI avisa por stderr cuando el número o el total no son representables (`1..=65535`)
- Las escrituras pasan por `tag.save_to_path(file, WriteOptions::default())`
- `id3` es **solo dev-dependency**: los tests de integración releen con esa librería lo que escribe `lofty`, de modo que la verificación es entre librerías distintas. Producción no la usa
- Al eliminar el último campo restante, el bloque ID3v2 no se conserva vacío: desaparece del archivo, igual que con `remove --all`
- La salida normal se escribe con `outln!` y **nunca** con `println!`, porque `println!` entra en pánico si el consumidor cierra el pipe; los avisos y errores van con `eprintln!` y no se tocan
- La integración continua ejecuta `cargo` con `--locked`, de modo que si `Cargo.toml` y `Cargo.lock` no coinciden la CI falla en lugar de actualizar el lock en silencio; por eso, al añadir o cambiar dependencias, el lock MUST commitearse en el mismo cambio
- El pipeline de release empuja la rama y el tag en un **único** `git push --atomic` con reintentos: si no lo consigue, falla de forma explícita antes de publicar, de modo que un fallo deja `main` sin commit de release y sin tag en lugar de a medias (con dos push separados, el servidor llegó a rechazar el tag con `missing necessary objects` mientras la rama ya se había actualizado)
- El paquete publicado se limita a la lista blanca `include` de `Cargo.toml`, y los patrones de primer nivel van anclados con `/` porque `include` usa la semántica de gitignore: sin la barra inicial, `"README.md"` habría arrastrado al paquete un README del andamiaje de `openspec/`

## Git workflow

Git Flow — see [GIT_FLOW.md](GIT_FLOW.md) for full branching model, auto-release pipeline, and conventional commit conventions. La recuperación de un release a medias está en [RELEASE.md](RELEASE.md).

- `main` — production (merge aquí = release automático)
- `development` — feature integration
- `feature/*` — work branches
- `hotfix/*` — urgent fixes from `main`

Commits: conventional-commit style (`feat:`, `fix:`, `refactor:`, `chore:`, etc.). CI determines version bump automatically — never bump manually.

## Existing instructions

`.github/copilot-instructions.md` has additional detail (ID3 frame mappings, átomos MP4, testing patterns) — consult if deeper context is needed.
