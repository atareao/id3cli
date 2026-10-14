# id3cli — AGENTS.md

Single-crate Rust CLI (`edition 2024`, `lofty` + `clap` derive) that reads/writes metadata on **MP3** (tags ID3v2.4) and **MP4 / M4A** (átomos `ilst` estilo iTunes).

## Commands

```sh
cargo build                      # debug
cargo test                       # 219 tests (124 unit + 95 integración)
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
  - **Común**: `detect_format()`, `detect_format_from_bytes()`, `detect_mime_type()`, `detect_cover_mime_type()`, `MP4_ATOM_MAP`
- **`src/main.rs`** — thin CLI wrapper with clap subcommands: `show`, `edit`, `remove`
- **`src/output.rs`** — capa de escritura de la salida estándar: contiene el macro `outln!` (equivalente a `println!`) que trata el cierre del pipe como un final normal de la salida, y es el único sitio por el que se escribe en stdout
- **`src/tests.rs`** — unit tests (module of lib.rs via `#[cfg(test)]`)
- **`tests/integration_test.rs`** — integration tests spawning the binary via `Command`
- **`tests/fixtures/`** — `sample.mp3` y `sample.m4a` reales, más `generate_sample_m4a.sh`

## Key conventions

- Multiple artists joined with `"; "` separator
- Apple metadata via `.set_text()` frames: TCMP (`"1"`), TSOA, TSOP, TSOT
- Lyrics stored in USLT frame, language code `"spa"`
- Cover art MIME auto-detected from file extension: `.jpg`/`.jpeg` → `image/jpeg`, `.png` → `image/png`, `.webp` → `image/webp`
- `remove_tags()` accepts both English and Spanish names (title/título, cover/carátula, etc.)
- Error messages in Spanish
- Integration tests use `create_temp_mp3()`, `create_temp_m4a()` y `create_temp_fixture()` helpers + `cleanup_file()`
- El formato se decide por **contenido** (magic bytes), nunca por la extensión: `detect_format()` / `detect_format_from_bytes()`, y las lecturas de `lofty` vía `read_from()`, nunca `read_from_path()`
- Formatos no soportados (FLAC, OGG/Opus, WAV, AIFF, APE, WavPack): salida distinta de cero, mensaje en español y archivo intacto
- MP3 tags are always written as ID3v2.4 through `lofty`; MP4 metadata is written as iTunes-style `ilst` atoms (tabla de mapeo en README.md)
- En MP4, `--year` y `--date` comparten el átomo `©day`: si se indican ambos gana `--date` y se avisa por stderr; `--subtitle` va a un átomo *freeform* (`----:com.apple.iTunes:SUBTITLE`)
- Degradaciones documentadas en MP4: carátula WebP en `covr` con tipo de dato reservado (0) más aviso por stderr; `©lyr` sin código de idioma; `--url` y `--original-artist` se omiten con aviso y nunca se escriben como átomo *freeform*
- Las escrituras pasan por `tag.save_to_path(file, WriteOptions::default())`
- `id3` es **solo dev-dependency**: los tests de integración releen con esa librería lo que escribe `lofty`, de modo que la verificación es entre librerías distintas. Producción no la usa
- Al eliminar el último campo restante, el bloque ID3v2 no se conserva vacío: desaparece del archivo, igual que con `remove --all`
- La salida normal se escribe con `outln!` y **nunca** con `println!`, porque `println!` entra en pánico si el consumidor cierra el pipe; los avisos y errores van con `eprintln!` y no se tocan
- La integración continua ejecuta `cargo` con `--locked`, de modo que si `Cargo.toml` y `Cargo.lock` no coinciden la CI falla en lugar de actualizar el lock en silencio; por eso, al añadir o cambiar dependencias, el lock MUST commitearse en el mismo cambio

## Git workflow

Git Flow — see [GIT_FLOW.md](GIT_FLOW.md) for full branching model, auto-release pipeline, and conventional commit conventions.

- `main` — production (merge aquí = release automático)
- `development` — feature integration
- `feature/*` — work branches
- `hotfix/*` — urgent fixes from `main`

Commits: conventional-commit style (`feat:`, `fix:`, `refactor:`, `chore:`, etc.). CI determines version bump automatically — never bump manually.

## Existing instructions

`.github/copilot-instructions.md` has additional detail (ID3 frame mappings, átomos MP4, testing patterns) — consult if deeper context is needed.
