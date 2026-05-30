# id3cli — AGENTS.md

Single-crate Rust CLI (`edition 2024`, `id3` + `clap` derive) that reads/writes ID3v2.4 tags on MP3 files.

## Commands

```sh
cargo build                      # debug
cargo test                       # 99 tests (59 unit + 40 integration)
cargo test --lib                 # unit tests only (src/tests.rs)
cargo test --test '*'            # integration tests only (tests/integration_test.rs)
cargo fmt
cargo clippy -- -D warnings
cargo run -- edit <file> --title "X"    # run CLI with args
RUST_BACKTRACE=1 cargo run -- ...       # debug with backtrace
```

## Architecture

- **`src/lib.rs`** — all business logic: `apply_metadata()`, `add_lyrics()`, `add_url()`, `add_cover_art()`, `add_apple_metadata()`, `remove_tags()`, `display_tags()`, `detect_mime_type()`
- **`src/main.rs`** — thin CLI wrapper with clap subcommands: `show`, `edit`, `remove`
- **`src/tests.rs`** — unit tests (module of lib.rs via `#[cfg(test)]`)
- **`tests/integration_test.rs`** — integration tests spawning `cargo run` via `Command`

## Key conventions

- Multiple artists joined with `"; "` separator
- Apple metadata via `tag.set_text()` frames: TCMP (`"1"`), TSOA, TSOP, TSOT
- Lyrics stored in USLT frame, language code `"spa"`
- Cover art MIME auto-detected from file extension: `.jpg`/`.jpeg` → `image/jpeg`, `.png` → `image/png`, `.webp` → `image/webp`
- Tags always written with `id3::Version::Id3v24`
- `remove_tags()` accepts both English and Spanish names (title/título, cover/carátula, etc.)
- Error messages in Spanish
- Integration tests use `create_temp_mp3()` helper + `cleanup_file()`

## Git workflow

Git Flow — see [GIT_FLOW.md](GIT_FLOW.md) for full branching model, auto-release pipeline, and conventional commit conventions.

- `main` — production (merge aquí = release automático)
- `development` — feature integration
- `feature/*` — work branches
- `hotfix/*` — urgent fixes from `main`

Commits: conventional-commit style (`feat:`, `fix:`, `refactor:`, `chore:`, etc.). CI determines version bump automatically — never bump manually.

## Existing instructions

`.github/copilot-instructions.md` has additional detail (ID3 frame mappings, testing patterns) — consult if deeper context is needed.