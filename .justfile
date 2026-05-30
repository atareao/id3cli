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