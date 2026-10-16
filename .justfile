user     := "atareao"
name     := `basename ${PWD}`
version  := `vampus show`

list:
    @just --list

install:
    cargo install --path .

lint:
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo fmt -- --check

fmt-fix:
    cargo fmt

upgrade:
    @vampus upgrade --patch

# Verificación e2e local: los 5 formatos en serie, se detiene con el exit code
# del primero que falle. Pre-construye el binario UNA vez (cargo build --locked)
# y lo pasa con --bin target/debug/id3cli a cada script (spec e2e-tooling Req 5:
# «una sola construcción por ejecución»). Acepta --keep y flags extra vía
# {{args}}; los flags --bin/--keep por extensión son para las recipes individuales.
e2e *args:
    cargo build --locked
    set -e; for ext in mp3 m4a ogg opus flac; do scripts/e2e/e2e_${ext}.sh --bin target/debug/id3cli {{args}}; done

# e2e de un formato concreto (acepta --bin RUTA y --keep)
e2e-mp3 *args:
    scripts/e2e/e2e_mp3.sh {{args}}

e2e-m4a *args:
    scripts/e2e/e2e_m4a.sh {{args}}

e2e-ogg *args:
    scripts/e2e/e2e_ogg.sh {{args}}

e2e-opus *args:
    scripts/e2e/e2e_opus.sh {{args}}

e2e-flac *args:
    scripts/e2e/e2e_flac.sh {{args}}
