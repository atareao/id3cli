# Proposal

## Why

El CLI genera hoy salida legible y JSON para cinco formatos, pero un usuario de terminal (o un dotfiles manager) no tiene forma de descubrir subcomandos, opciones ni valores sin leer la ayuda. Los CLIs Rust con clap de referencia (cargo, gh, rg) ofrecen completions generados del propio parser. Sin esta feature, cada shell necesita completions escritos a mano que se desincronizan de la superficie real del CLI.

## What Changes

Nuevo subcomando `id3cli completions <SHELL>` con `SHELL ∈ {bash, zsh, fish}` exactos. La generación se hace en runtime con el crate `clap_complete` 4.5 (misma minor que clap 4.5.54) sobre el Command real del CLI (vía `CommandFactory`), de modo que el script SIEMPRE refleja la superficie actual (subcomandos show/edit/remove/completions, flags y value_names). La lógica vive en lib.rs (enum `GenerationShell` con FromStr en español y `generate_completions` testeable); main.rs solo añade el subcomando con un value_parser y enruta. La salida pasa por la capa `output.rs` con una nueva `print_block` (bloque multilínea, pipe cerrado → exit 0). `clap_complete` entra en `[dependencies]` y el Cargo.lock se commitea (regla `--locked`).

## Capabilities

### New Capabilities

- **shell-completions**: contrato del subcomando `completions` — shells exactos bash/zsh/fish, generación sobre el Command real, salida por la capa output.rs, errores en español para shells no soportados, y marcadores por shell verificables.

### Modified Capabilities

- **cli-output**: la capa de salida gana `print_block` para bloques multilínea con el mismo contrato de pipe cerrado; el subcomando `completions` se suma a la superficie del CLI.
- **e2e-tooling**: el runner común cubre un script nuevo no ligado a formato (`e2e_completions.sh`) y la recipe `just e2e` pasa a ejecutar los 5 formatos + completions con una sola construcción.
- **e2e-verification**: la matriz de verificación gana la capa de completions (sintaxis real con `bash -n` como oráculo, marcadores zsh/fish, subcomandos presentes, pipe cerrado, shell inválido, falsabilidad).

## Impact

- Dependencia de producción nueva: `clap_complete = "4.5"` (Cargo.lock commiteado en el mismo cambio; CI con `--locked`).
- `src/lib.rs` (+enum `GenerationShell` + `generate_completions`), `src/output.rs` (+`print_block`), `src/main.rs` (+subcomando `Completions`), `src/tests.rs` (+unitarios), `tests/integration_test.rs` (+tests CLI), `Cargo.toml`/`Cargo.lock`.
- `scripts/e2e/e2e_completions.sh` (nuevo), `.justfile` (recipe `e2e` ampliada y opcionalmente `e2e-completions`).
- README/AGENTS.md/copilot-instructions (sección completions + recuentos de tests medidos + roadmap).
- No-rotura: `show`/`edit`/`remove` y sus salidas no cambian (subcomando aditivo); el test documental exige recuentos medidos y keywords intactas (`["id3", "mp3", "mp4", "metadata", "cli"]`).
- No-goals: shells elvish/powershell, generación en build-time (AOT), escritura a fichero, `completions` en edit/remove.