<!-- Copilot instructions for contributors and AI coding agents -->
# id3cli — AI guidance

Purpose: help an AI coding agent become productive quickly in this Rust CLI that reads and writes audio metadata.

- **Big picture:** single-binary Rust CLI that reads/writes metadata on **MP3** (ID3v2.4 tags), **MP4 / M4A** (iTunes-style `ilst` atoms) and **OGG** (Vorbis comments, `KEY=value` pairs; the Ogg container covers both Vorbis and Opus). The project is organized as:
  - [src/lib.rs](src/lib.rs#L1) - Core library with all business logic (2702 lines)
  - [src/main.rs](src/main.rs#L1) - CLI application with clap subcommands `show`/`edit`/`remove` (871 lines)
  - [src/tests.rs](src/tests.rs#L1) - Unit tests for library functions (4888 lines)
  - [tests/integration_test.rs](tests/integration_test.rs#L1) - Integration tests spawning the binary (5438 lines)
  - [tests/fixtures](tests/fixtures) - real samples `sample.mp3`, `sample.m4a`, `sample.ogg` and `sample.opus`, plus their regeneration scripts `generate_sample_m4a.sh`, `generate_sample_ogg.sh` and `generate_sample_opus.sh`
  - [Cargo.toml](Cargo.toml#L1) - Project metadata, edition 2024, and dependencies

  Production uses `lofty` (v0.25) to read/write metadata and `clap` (v4.5, derive) for argument parsing. The `id3` crate (v1.16.4) is a **dev-dependency only**: integration tests re-read with `id3` what `lofty` wrote, so verification happens across two different libraries. `cargo tree -e normal` does not show `id3`.

- **Architecture & data flow:** the CLI parses paths/arguments with clap's derive macro, detects the audio format from the file's content, and dispatches to the MP3 (ID3v2.4), MP4 (`ilst`) or OGG (`VorbisComments`, Vorbis or Opus) code path in `src/lib.rs`. Output goes to stdout or to the file on disk. There are no web services, databases, or background workers — changes are local file I/O.

- **Key files to reference:**
  - **src/lib.rs** (2702 lines) - All business logic: `apply_metadata()`, `apply_metadata_mp4()`, `apply_metadata_ogg()`, `edit_mp4()`, `edit_ogg()`, `read_mp4_tag()`, `read_vorbis_tag()`, `write_mp4_tag()`, `write_vorbis_tag()`, `remove_tags()`, `remove_tags_mp4()`, `remove_tags_ogg()`, `remove_all_id3()`, `remove_all_mp4()`, `remove_all_ogg()`, `display_tags()`, `display_tags_mp4()`, `display_tags_ogg()`, `add_lyrics()`, `add_url()`, `add_cover_art()`, `add_apple_metadata()`, `detect_format()`, `detect_format_from_bytes()`, `detect_mime_type()`, `detect_cover_mime_type()`, `MP4_ATOM_MAP`, `VORBIS_COMMENT_MAP`
  - **src/main.rs** (871 lines) - CLI with clap derive and the `show`/`edit`/`remove` subcommands
  - **src/tests.rs** (4888 lines) - Unit tests (a module of lib.rs via `#[cfg(test)]`)
  - **tests/integration_test.rs** (5438 lines) - Integration tests that spawn the binary as a subprocess
  - **Cargo.toml** - Project metadata, `edition 2024`, and dependencies

- **Developer workflows (explicit commands):**
  - Build debug binary: `cargo build`
  - Run locally with subcommands: `cargo run -- edit song.mp3 --title "X" --artist "Y"`
  - Build optimized release: `cargo build --release`
  - Run all tests: `cargo test` (281 = 163 unit + 118 integration); use `--no-fail-fast` if more than one suite fails, since `cargo test` stops at the first failing binary
  - Run only unit tests: `cargo test --lib`
  - Run only integration tests: `cargo test --test '*'`
  - Format: `cargo fmt`
  - Lint: `cargo clippy --all-targets --all-features -- -D warnings`
  - Debug with backtraces: `RUST_BACKTRACE=1 cargo run -- show song.mp3`

- **CI/CD workflows:**
  - `ci.yml` — runs on every `pull_request` to `main`/`development` and every `push` to `development`. Steps: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build`, `cargo test`.
  - `release-prepare.yml` — runs on every `push` to `main` (skipped when the commit already starts with `chore: release`). It derives the bump from the commits (`BREAKING CHANGE`/`!:` → major; any `^feat` → minor; otherwise patch), runs `vampus upgrade`, regenerates `CHANGELOG.md` with `git-cliff`, commits `chore: release vX.Y.Z`, creates the tag, pushes it, triggers `release.yml`, and syncs `development` with `main`.
  - `release.yml` — runs on pushing a `v*` tag (or manually via `workflow_dispatch`). It builds the binary for `x86_64-unknown-linux-gnu`, packages `id3cli-x86_64-unknown-linux-gnu.tar.gz` with its `.sha256`, creates the GitHub Release with the attached binaries, and publishes the crate to crates.io with `cargo publish`.

- **Project conventions & patterns discovered here:**
  - Rust `edition = "2024"` in `Cargo.toml`
  - Keep the binary small and dependency-light — adding new deps must be justified
  - CLI uses clap's derive macros; the `edit` options are `-t/--title`, `-a/--artist` (repeatable), `--album`, `--year`, `--date`, `--genre`, `--track`, `--season`, `--composer`, `--copyright`, `--subtitle`, `--original-artist`, `--album-artist`, `-L/--lyrics`, `-c/--cover`, `-u/--url`, `--compilation`, `--title-sort`, `--artist-sort`, `--album-sort`; `--track` and `--season` accept the form `N` or `N/T`
  - `remove <FILE> [--all] [TAGS...]`: `--all` deletes everything; without it, the listed tags are removed by name (English or Spanish names accepted)
  - Multiple artists are joined with a `"; "` separator
  - Error messages are in Spanish
  - MP3 tags are always written as ID3v2.4; MP4 metadata is written as iTunes-style `ilst` atoms; OGG is written as Vorbis comments through the concrete `VorbisComments` type (not the generic `Tag`, which drops unmapped keys and normalizes the case of the ones that do map), with `VORBIS_COMMENT_MAP` as the single source of the mapping; the Vorbis comment is mandatory, so its vendor and `ENCODER` are not shown as metadata, `edit` preserves foreign keys (including `ENCODER`) and the vendor, and `remove --all` empties the comment and only preserves the vendor (mandatory, it cannot be removed from the container)
  - Track/season totals are stored per format: MP3 and OGG keep the total as **text** (`TRCK`/`TPOS` as `n/total`; `TRACKTOTAL`/`DISCTOTAL` keys) and accept zero, while the MP4 atom stores the pair in 16 bits and the CLI warns on stderr when the number or the total is not representable (`1..=65535`)
  - Writes go through `tag.save_to_path(file, WriteOptions::default())`
  - When the last remaining field is removed, the ID3v2 block is not kept empty: it disappears from the file, exactly like `remove --all`
  - All testable logic is extracted into functions; tests cover success and error cases

- **Integration notes:**
  - Format is decided by **content** (magic bytes), never by extension: `ftyp` at offset 4 → MP4; an `ID3` header or a valid MPEG frame sync → MP3; an `OggS` page whose first packet starts with `\x01vorbis` → OGG Vorbis and `OpusHead` → OGG Opus (the Ogg container is decided by the **codec** of its first packet, not by the page header)
  - Unsupported formats (Speex, FLAC, WAV, AIFF, APE, WavPack): exit with a non-zero code, a Spanish message, and the file is left **untouched**. Ogg is not rejected by itself: an Ogg container is rejected only when the codec of its first packet is not a supported Ogg codec (Speex, FLAC, Theora…)
  - Cover MIME types are auto-detected from file extension: `.jpg`/`.jpeg` → `image/jpeg`, `.png` → `image/png`, `.webp` → `image/webp`
  - MP3 uses ID3 frames; MP4 uses `ilst` atoms under `moov.udta.meta.ilst`; OGG uses Vorbis comments (`KEY=value`). The flag→frame/atom/key mapping and the known degradations are documented in detail in `README.md`:
    - `--year` and `--date` share the `©day` atom; if both are given `--date` wins, with a stderr warning
    - `--subtitle` goes to a *freeform* atom `----:com.apple.iTunes:SUBTITLE`
    - A WebP cover in `covr` is written with the reserved data type (0) plus a warning (not visible in iTunes/macOS)
    - `©lyr` has no language field
    - `--url` and `--original-artist` have no equivalent atom and are omitted with a warning (never *freeform*), exit code 0
  - OGG flag→key mapping, the single source being `VORBIS_COMMENT_MAP` (18 pairs) plus the pair totals `TRACKTOTAL`/`DISCTOTAL`. The exact same mapping and the same degradations apply verbatim to OGG Opus: the Ogg container covers Vorbis and Opus, and both write Vorbis comments:
    - `--title`→`TITLE`, `--artist`→`ARTIST`, `--album`→`ALBUM`, `--album-artist`→`ALBUMARTIST`, `--year`→`YEAR`, `--date`→`DATE`, `--genre`→`GENRE`, `--track`→`TRACKNUMBER` (+`TRACKTOTAL`), `--season`→`DISCNUMBER` (+`DISCTOTAL`), `--composer`→`COMPOSER`, `--copyright`→`COPYRIGHT`, `--lyrics`→`LYRICS`, `--cover`→`METADATA_BLOCK_PICTURE`, `--compilation`→`COMPILATION` (`"1"`), `--title-sort`→`TITLESORT`, `--artist-sort`→`ARTISTSORT`, `--album-sort`→`ALBUMSORT`, `--subtitle`→`SUBTITLE`
    - OGG degradations: a WebP cover is written in `METADATA_BLOCK_PICTURE` with its real MIME and **no** warning (unlike `covr` in MP4); `LYRICS` has no language code (like `©lyr`); `--url` and `--original-artist` have no key and are omitted with a warning, never written under an invented key, exit code 0
  - No network or external credentials discovered — changes are local filesystem operations

- **Supported features (as of current version):**
  - Basic metadata for MP3, MP4, OGG and Opus: title, artist(s), album, year, date, genre, track, season
  - Extended metadata: copyright, composer, subtitle, original artist, album artist, lyrics, url
  - Apple metadata: compilation flag, title/artist/album sort orders
  - Cover art: JPG, PNG, and WEBP as front cover with MIME auto-detection
  - Display: `show` prints all tags with a readable format
  - Tag removal: `remove` deletes specific tags (English/Spanish names) or everything with `--all` (the ID3 block in MP3, the `ilst` tree in MP4, the user keys in OGG — which preserves the mandatory vendor)
  - Multiple artists: `--artist` can be repeated, joined with `"; "`

- **What an AI helper should do first:**
  1. Run `cargo build` to ensure the toolchain and dependencies are available
  2. Run `cargo test` to verify all 281 tests pass (163 unit + 118 integration)
  3. Review **src/lib.rs** for all business logic functions
  4. Review **src/main.rs** for CLI structure and argument handling
  5. Test with: `cargo run -- show /tmp/test.mp3`

- **Examples of small, acceptable changes:**
  - Add a new metadata field: update the `edit` options in main.rs, add logic in `src/lib.rs`, update the tests
  - Add a new display format: modify `display_tags()` / `display_tags_mp4()` / `display_tags_ogg()` in lib.rs
  - Add validation: add checks in main.rs before calling library functions
  - Always update both unit tests (src/tests.rs) and integration tests (tests/integration_test.rs) when adding features

- **Constraints & cautions for AI PRs:**
  - Do not change `edition` or bump deps without mentioning compatibility reasons
  - All new features must include tests (both unit and integration)
  - Keep the CLI user-friendly with clear error messages in Spanish
  - Preserve existing coverage — currently 281 tests (163 unit + 118 integration)
  - When modifying shared functions (`apply_metadata()`, `add_cover_art()`, `add_lyrics()`, `add_url()`, `add_apple_metadata()`), update ALL tests that call them
  - Use the `"; "` separator for multiple artists
  - Supported image formats: JPG, PNG, WEBP — validate extensions and return helpful errors

- **Testing patterns:**
  - Unit tests live in `src/tests.rs` as a module of lib.rs under `#[cfg(test)]`
  - Integration tests in `tests/integration_test.rs` spawn the CLI as a subprocess
  - Helpers such as `create_temp_mp3()`, `create_temp_m4a()`, `create_temp_ogg()`, `create_temp_opus()` and `create_temp_fixture()` build fixtures; `cleanup_file()` removes them
  - Assertions use `assert_eq!`; MP3 tests re-read what `lofty` wrote using the `id3` crate
  - `tests/fixtures/` holds real `sample.mp3`, `sample.m4a`, `sample.ogg` and `sample.opus` files

If any section is unclear or you'd like the file to include more examples, ask for clarification on specific areas.
