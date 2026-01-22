<!-- Copilot instructions for contributors and AI coding agents -->
# id3cli — AI guidance

Purpose: help an AI coding agent become productive quickly in this Rust CLI that manipulates ID3 tags.

- **Big picture:** single-binary Rust CLI for adding ID3 tags and cover art to MP3 files. The program entrypoint is [src/main.rs](src/main.rs#L1). Project metadata and deps are in [Cargo.toml](Cargo.toml#L1-L8). The code uses the `id3` crate (v1.16.4) to read/write MP3 tags and `clap` (v4.5) for CLI argument parsing.

- **Architecture & data flow:** CLI reads paths/arguments via clap's derive macro and delegates to the `id3` crate to parse and mutate ID3 frames. Output is typically to stdout or the mp3 file on disk. There are no web services, databases, or background workers — changes are local file I/O.

- **Key files to reference:**
  - [src/main.rs](src/main.rs#L1): complete CLI implementation with argument parsing, tag manipulation, and display functions
  - [tests/integration_test.rs](tests/integration_test.rs#L1): integration tests that verify CLI behavior end-to-end
  - [Cargo.toml](Cargo.toml#L1-L8): crate name, edition (`2024`) and dependency pins
  - [README.md](README.md#L1): user-facing documentation with examples
  - [RELEASE.md](RELEASE.md#L1): release process documentation
  - [.github/workflows/release.yml](.github/workflows/release.yml#L1): GitHub Actions workflow for automated releases

- **Developer workflows (explicit commands):**
  - Build debug binary: `cargo build`
  - Run locally with args: `cargo run -- --your-args`
  - Example: `cargo run -- -f song.mp3 -t "Title" -a "Artist"`
  - Build optimized release: `cargo build --release`
  - Run all tests: `cargo test` (38 unit + 25 integration = 63 total)
  - Run only unit tests: `cargo test --test ''`
  - Run only integration tests: `cargo test --test '*'`
  - Format: `cargo fmt`
  - Lint: `cargo clippy -- -D warnings`
  - Debug with backtraces: `RUST_BACKTRACE=1 cargo run -- --your-args`
  - Create release: See [RELEASE.md](RELEASE.md#L1) for detailed release process

- **CI/CD workflows:**
  - GitHub Actions workflow at [.github/workflows/release.yml](.github/workflows/release.yml#L1) builds Linux binaries automatically on release creation
  - Triggers on: git tag push + GitHub release creation
  - Generates: `id3cli-linux-x86_64` binary and SHA256 checksum
  - Binary uploaded as release asset automatically

- **Project conventions & patterns discovered here:**
  - Rust `edition = "2024"` in `Cargo.toml`; follow modern Rust idioms (async only if added explicitly)
  - Keep the binary small and dependency-light — adding new deps must be justified
  - CLI uses clap's derive macros for argument parsing
  - Multiple artists are joined with `"; "` separator (not `" / "`)
  - All testable logic extracted into pure functions (e.g., `apply_metadata()`, `add_cover_art()`)
  - Tests verify both success and error cases
  - Functions accept references/slices when possible to avoid unnecessary cloning

- **Integration notes:**
  - Primary external dependency: the `id3` crate. Inspect its API (e.g., `id3::Tag`, `id3::frame::Picture`) when adding features
  - Date fields use `Timestamp` type - parse strings with `.parse()`
  - Copyright stored in TCOP frame via `tag.set_text("TCOP", value)`
  - Lyrics stored in USLT frame (Unsynchronised lyrics) with language code "spa" and Content::Lyrics
  - URL stored in WOAR frame (Official artist webpage) with Content::Link
  - Apple metadata: TCMP (compilation flag, "1" = compilation), TSOA (album sort), TSOP (artist sort), TSOT (title sort)
  - Cover art MIME types auto-detected from file extension: .jpg/.jpeg → image/jpeg, .png → image/png, .webp → image/webp
  - `detect_mime_type()` function validates image formats and returns appropriate MIME type
  - `add_cover_art()` now accepts `&Path` to detect format before embedding
  - `add_lyrics()` creates Frame with Content::Lyrics and adds to tag
  - `add_url()` creates Frame with Content::Link for WOAR frame
  - `add_apple_metadata()` handles all Apple-specific frames (TCMP, TSOA, TSOP, TSOT)
  - No network or external credentials discovered — changes are local filesystem operations

- **Supported features (as of current version):**
  - Basic metadata: title, artist(s), album, year, genre, track
  - Extended metadata: date (recorded), copyright, lyrics (USLT frame), url (WOAR frame)
  - Standard ID3v2 tags: composer (TCOM), subtitle (TIT3), original artist (TOPE), album artist (TPE2)
  - Apple metadata: compilation flag (TCMP), sort orders (TSOA, TSOP, TSOT)
  - Cover art: JPG, PNG, and WEBP files as front cover with MIME type auto-detection
  - Display: `--show` flag to view all tags (lyrics preview shows first 3 lines)
  - Tag removal: `--remove` flag to delete specific tags (supports English/Spanish names)
  - Multiple artists: specify `--artist` multiple times, joined with "; "
  - **Perfect for podcasts:** All recommended tags for podcast episodes supported

- **What an AI helper should do first:**
  1. Run `cargo build` to ensure the toolchain and dependencies are available
  2. Run `cargo test` to verify all 92 tests pass (55 unit + 37 integration)
  3. Review [src/main.rs](src/main.rs#L1) to understand the complete implementation
  4. Test with: `cargo run -- -f /tmp/test.mp3 --show`

- **Examples of small, acceptable changes:**
  - Add new metadata field: update `Args` struct, add parameter to `apply_metadata()`, update all call sites and tests
  - Add new display format: modify `display_tags()` function
  - Add validation: add checks in `main()` before calling `apply_metadata()`
  - Always update both unit tests and integration tests when adding features

- **Constraints & cautions for AI PRs:**
  - Do not change `edition` or bump deps without mentioning compatibility reasons
  - All new features must include tests (both unit and integration)
  - Maintain the pattern of extracting testable functions from `main()`
  - Keep the CLI user-friendly with clear error messages in Spanish
  - Preserve existing test coverage - currently at 92 tests (55 unit + 37 integration)
  - When modifying `apply_metadata()`, `add_cover_art()`, `add_lyrics()`, `add_url()`, or `add_apple_metadata()`, update ALL tests that call them
  - apply_metadata() now takes 13 parameters: title, artists, album, year, genre, track, date, copyright, composer, subtitle, original_artist, album_artist
  - Use "; " separator for multiple artists (not " / ")
  - Supported image formats: JPG, PNG, WEBP - validate extensions and return helpful errors
  - Tag names accept both English and Spanish for user-friendly CLI
  - Lyrics use ISO-639-2 language code "spa" for Spanish
  - Apple metadata: TCMP uses "1" for compilation, TSOA/TSOP/TSOT use set_text() for sort orders
  - Standard ID3v2: TCOM (composer), TIT3 (subtitle), TOPE (original artist), TPE2 (album artist via set_album_artist())

- **Testing patterns:**
  - Unit tests in `src/main.rs` under `#[cfg(test)] mod tests`
  - Integration tests in `tests/integration_test.rs` that spawn the CLI as a subprocess
  - Helper function `create_temp_mp3()` for creating test fixtures
  - Use `assert_eq!` for expected values, verify via `Tag::read_from_path()`
  - Clean up temp files with `cleanup_file()` helper

If any section is unclear or you'd like the file to include more examples, ask for clarification on specific areas.
