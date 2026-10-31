# id3cli — AGENTS.md

CLI Rust (`edition 2024`, `lofty` + `clap`) para MP3 (ID3v2.4), MP4/M4A (`ilst`), OGG (Vorbis/Opus) y FLAC (`fLaC`). Detalle literal en `docs/`.

## Commands

```sh
cargo build                      # debug
cargo test                       # 331 tests (178 unit + 153 integración)
cargo test --lib                 # unit tests only (src/tests.rs)
cargo test --test '*'            # integration tests only (tests/integration_test.rs)
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- edit <file> --title "X"    # run CLI with args
RUST_BACKTRACE=1 cargo run -- ...       # debug with backtrace
```

Nota: `cargo test` para en el primer binario que falla; usa `--no-fail-fast` si fallan varias suites.

## Reglas clave

- Formato por **contenido** (magic bytes), nunca extensión; `read_from()`, no `read_from_path()`; Ogg por códec del 1er paquete.
- Salida normal **solo** con `outln!` (`src/output.rs`); **nunca** `println!` (pánico al cerrar el pipe). Errores con `eprintln!`.
- MP3 siempre ID3v2.4; escrituras vía `tag.save_to_path(file, WriteOptions::default())`.
- OGG con `VorbisComments` (no `Tag` genérico); `VORBIS_COMMENT_MAP` única fuente; `remove --all` deja el vendor (Ogg), borra el bloque (FLAC).
- `id3` **solo dev-dependency**; errores en español; `remove_tags()` acepta inglés y español.
- Varios artistas con `"; "`; lyrics USLT `"spa"`; carátula MIME por extensión.
- Apple metadata con `.set_text()` (TCMP/TSOA/TSOP/TSOT).
- `--track`/`--season` aceptan `N` o `N/T` (MP4 avisa fuera de `1..=65535`).
- FLAC por `fLaC`; `\x7fFLAC` en Ogg **no**; no soportados (Speex, WAV, AIFF, APE, WavPack): exit≠0, archivo intacto.
- MP4: `--year`/`--date` comparten `©day` (gana `--date`); `--subtitle` a freeform; degradaciones en docs.
- Al eliminar el último campo, el bloque ID3v2 desaparece del archivo.
- CI `--locked`: commitear `Cargo.lock` al cambiar dependencias.
- Binario `x86_64-unknown-linux-musl` **estático**; release con un **único** `git push --atomic`.

## Git workflow

- `main` publica release al mergear **solo si** los commits lo justifican; `development` integra; `feature/*`; `hotfix/*` desde `main`; conventional commits (el bump lo calcula CI).

## Existing instructions

`.github/copilot-instructions.md`: ID3 frames, átomos MP4, testing patterns.

## Dónde está el resto

| Tema | Fichero |
|---|---|
| Arquitectura y módulos (`src/*`, completions, batch, fixtures) | `docs/agentes-id3cli-arquitectura.md` |
| Convenciones (formatos, degradaciones, tests, CI, musl, release) | `docs/agentes-id3cli-convenciones.md` |
| Git Flow: ramas, comandos y releases | `docs/agentes-id3cli-gitflow.md` |
| Ramas y auto-release | `GIT_FLOW.md` |
| Recuperar release a medias | `RELEASE.md` |
| Mapeo de metadatos | `README.md` |
| ID3 frames, átomos MP4, tests | `.github/copilot-instructions.md` |
