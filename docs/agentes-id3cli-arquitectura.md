# agentes-id3cli-arquitectura.md — stack y arquitectura (extracto literal del AGENTS.md previo)

> Extracto literal del `AGENTS.md` previo (poda 2026-10-31). Texto tal cual, sin resumen.

Single-crate Rust CLI (`edition 2024`, `lofty` + `clap` derive) that reads/writes metadata on **MP3** (tags ID3v2.4), **MP4 / M4A** (átomos `ilst` estilo iTunes), **OGG** (comentarios de Vorbis, pares `CLAVE=valor`; el contenedor Ogg cubre Vorbis y Opus) y **FLAC** (comentarios de Vorbis, `fLaC` nativo).

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

## Notas literales del AGENTS.md previo

Nota: `cargo test` se detiene en el primer binario que falla — añade `--no-fail-fast` cuando falle más de una suite.

`.github/copilot-instructions.md` has additional detail (ID3 frame mappings, átomos MP4, testing patterns) — consult if deeper context is needed.

