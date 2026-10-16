# Design

## Problema

Los usuarios de bash/zsh/fish no pueden autocompletar subcomandos, flags ni valores de id3cli; escribir completions a mano los desincroniza de la superficie real del CLI en cada nueva feature (v0.7.0, v0.8.0...). La solución debe ser aditiva, sin tocar `show`/`edit`/`remove`, testeable en TDD y coherente con la arquitectura del repo (lógica en lib.rs, salida solo por la capa `output.rs`).

## Decisión 1 — Subcomando `completions <SHELL>`, no flag global

`id3cli completions <SHELL>` es el patrón de cargo/gh. Un flag global `--generate-completions` contaminaría todos los subcomandos. Como subcomando además es directamente testeable en integración (`id3cli completions bash` es un proceso completo).

## Decisión 2 — Exactamente tres shells: bash, zsh, fish

Enum propio `GenerationShell { Bash, Zsh, Fish }` en lib.rs con `FromStr` (error en español: «el shell 'X' no está soportado; shells válidos: bash, zsh y fish») y `Display` (nombres minúsculos). Un value_parser en main.rs convierte los errores en fallos de clap (exit 2). Se descarta exponer `clap_complete::Shell` directamente (incluiría elvish/powershell, fuera del pedido del usuario y del alcance).

## Decisión 3 — Generación con `clap_complete` 4.5 sobre el Command real

`clap_complete = "4.5"` (misma minor que clap 4.5.54, sin conflicto de versiones). En lib.rs: `pub fn generate_completions(shell: GenerationShell, cmd: &mut clap::Command) -> String` que escribe el script en un `String` (generación a un `Write` interno). En main.rs: `let mut cmd = Args::command();` (traits `Parser`/`CommandFactory`) y `outln!`/`print_block` del resultado. El script refleja SIEMPRE la superficie real: subcomandos, options y value_names actuales.

## Decisión 4 — Salida solo por la capa `output.rs`: nueva `print_block`

El repo prohíbe `println!` (pánico con pipe cerrado). Los scripts de completions son multilínea, así que se añade `print_block(texto: &str)` a `output.rs` que escribe el bloque sin salto extra y aplica el contrato de `print_line`: pipe cerrado → exit 0 sin escribir, otros errores de escritura → mensaje en español por stderr y exit 1. Está en la misma línea que la cláusula de salida JSON que ya pasa por `outln!`.

## Decisión 5 — Marcadores por shell verificables (oráculos)

- bash: el script contiene la función `_id3cli` y la línea `complete -F _id3cli id3cli`.
- zsh: contiene `#compdef id3cli` y `_arguments`/`_id3cli`.
- fish: contiene líneas `complete -c id3cli ...`.

Estos marcadores son estables entre minors de clap_complete 4.5; los tests RED los fijan y el e2e los valida además con un oráculo real de sintaxis (`bash -n` sobre el script bash generado). zsh/fish se cubren por marcadores (no se exige tener esos intérpretes instalados).

## Decisión 6 — e2e dedicado sin audio

`scripts/e2e/e2e_completions.sh` reutiliza las helpers del runner (e2e_run, check_rc, check_show_contains) pero NO genera audio: verifica quirúrgicamente `completions bash|zsh|fish` (bash -n, marcadores, subcomandos presentes), el shell inválido (exit ≠ 0), el pipe cerrado (exit 0) y la falsabilidad (corromper el marcador esperado → exit 1). La recipe `just e2e` pasa a ejecutar los 5 formatos + completions en serie con una sola construcción (`cargo build --locked`), extendiendo la regla ya existente.

## Decisión 7 — Versionado y lock

`clap_complete` entra en `[dependencies]`; `Cargo.lock` se regenera y se commitea en el mismo commit (CI con `--locked`). La feature es `feat:` → release minor **v0.9.0** por CI (no tocar la versión en Cargo.toml).

## Decisión 8 — Test documental intacto

El test `test_documentacion_describe_los_formatos_soportados` exige recuentos medidos y keywords intactas. Tras GREEN se mide la suite (unit + integración) y en la fase de REFACTOR docs se actualizan README (escudo, desglose, nueva sección completions, roadmap), AGENTS.md y copilot-instructions con esos recuentos. Las keywords de Cargo.toml NO se tocan.

## Alternativas descartadas

- Flag global `--generate-completions`: contamina todos los subcomandos; menos estándar.
- `clap_complete::Shell` directo en la CLI: expone elvish/powershell fuera del alcance del usuario.
- Escribir el script a un fichero: obliga a decidir ruta; stdout permite `id3cli completions bash > ~/.bash_completion.d/id3cli` y dotfiles.
- Generación AOT (build.rs): complica el empaquetado y no aporta valor para un CLI instalado desde crates.io.

## Riesgos y mitigaciones

- **Formato exacto de los scripts entre minors de clap_complete 4.5**: fijado por medición en GREEN; los tests RED usan marcadores estables (`complete -F _id3cli`, `#compdef`, `complete -c`).
- **`bash -n` solo cubre bash**: zsh/fish se cubren por marcadores de subcadena (documentado, no enmascarado).
- **El subcomando debe listarse a sí mismo**: el script generado de `completions` debe incluir `completions` entre los subcomandos (los tests lo verifican).
- **Recuentos de tests suben**: se miden y se reflejan en README/AGENTS/copilot en REFACTOR (test documental como red de seguridad).