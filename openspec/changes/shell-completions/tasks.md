# Tasks

El change incorpora el subcomando `id3cli completions <SHELL>` (bash/zsh/fish) generado en runtime con `clap_complete` 4.5 sobre el Command real del CLI, siguiendo el patrón de los changes archivados (`flac-support`, `json-output`): aditivo sin tocar la salida legible, dependencia en producción con Cargo.lock commiteado, tests unitarios + integración + e2e dedicado, coherencia documental y release minor única. Nada de `src/` ni `tests/` se toca antes del STOP del grupo 1.

## 1. Artefactos de especificación (OpenSpec)

- [ ] 1.1 Redactar `proposal.md` con Why/What Changes/Capabilities/Impact (subcomando `completions`, shells exactos bash/zsh/fish, `clap_complete` 4.5 en producción, salida por `output.rs`, tests, e2e con `bash -n`, docs; no-goals: elvish/powershell, AOT, escritura a fichero).
      Verificación: `openspec/changes/shell-completions/proposal.md` MUST contener las 4 secciones con las capabilities (NEW: shell-completions; MODIFIED: cli-output, e2e-tooling, e2e-verification).
- [ ] 1.2 Redactar el delta `specs/shell-completions/spec.md` (capability NUEVA) con el contrato completo: subcomando `completions <SHELL>`, tres shells exactos (bash/zsh/fish), generación sobre el Command real (CommandFactory), salida estándar solo por `print_block`, stderr vacío en éxito, shell inválido → error en español por stderr con exit ≠ 0, marcadores por shell.
- [ ] 1.3 Redactar el delta `specs/cli-output/spec.md` (MODIFIED): `print_block` en la capa de salida con el mismo contrato de pipe cerrado que `print_line` (pipe cerrado → exit 0; otros errores → exit 1 en español); el subcomando `completions` se suma a la superficie del CLI sin cambiar show/edit/remove.
- [ ] 1.4 Redactar el delta `specs/e2e-tooling/spec.md` (MODIFIED): el runner cubre un script no ligado a formato (`e2e_completions.sh`) que reutiliza e2e_run/check_rc/check_show_contains sin generar audio; la recipe `just e2e` ejecuta los 5 formatos + completions en serie con una sola construcción (`cargo build --locked`, regla existente).
- [ ] 1.5 Redactar el delta `specs/e2e-verification/spec.md` (MODIFIED): la matriz gana la capa de completions — sintaxis real del script bash con `bash -n`, marcadores zsh (`#compdef id3cli`) y fish (`complete -c id3cli`), subcomandos presentes (show/edit/remove/completions), pipe cerrado → exit 0, shell no soportado → exit ≠ 0, y falsabilidad re-demostrada (corromper el marcador esperado → exit 1).
- [ ] 1.6 STOP: presentar la especificación al usuario y esperar aprobación explícita antes de tocar `src/` ni `tests/`.
- [ ] 1.7 Validar el change: `openspec validate shell-completions` sin advertencias.

## 2. Fase RED — unitarios (src/tests.rs)

- [ ] 2.1 Tests de `GenerationShell::from_str`: acepta `"bash"`, `"zsh"`, `"fish"` (Display devuelve el mismo nombre en minúsculas) y rechaza `"powershell"`, `"elvish"`, `""` con el error en español que nombra los tres shells válidos.
- [ ] 2.2 Test de `generate_completions` para bash sobre un `clap::Command` mínimo (con subcomandos `show`, `edit`, `remove`, `completions`): la salida contiene `_id3cli` y `complete -F _id3cli id3cli`, y los nombres de los subcomandos.
- [ ] 2.3 Test de `generate_completions` para zsh: la salida contiene `#compdef id3cli` y `_arguments`.
- [ ] 2.4 Test de `generate_completions` para fish: la salida contiene `complete -c id3cli`.
      Verificación: los tests 2.2-2.4 fallan por compilación (E0432/E0425: `clap_complete` y/o símbolos aún inexistentes); el resto de la suite unitaria pasa.

## 3. Fase RED — integración CLI (tests/integration_test.rs)

- [ ] 3.1 `completions bash` → exit 0, stdout contiene `complete -F _id3cli id3cli`, stderr VACÍO.
- [ ] 3.2 `completions zsh` → exit 0, stdout contiene `#compdef id3cli`, stderr VACÍO.
- [ ] 3.3 `completions fish` → exit 0, stdout contiene `complete -c id3cli`, stderr VACÍO.
- [ ] 3.4 Los tres shells: stdout NO contiene la línea «no se encontraron metadatos» ni los emojis del modo legible (no-rotura de la separación de canales).
- [ ] 3.5 Algún subcomando de la superfice (`show`, `edit`, `remove`) aparece en el script bash.
- [ ] 3.6 Shell inválido (`completions powershell`) → exit ≠ 0, stderr menciona `bash`, `zsh` y `fish`, stdout vacío.
- [ ] 3.7 Pipe cerrado: `completions bash` con stdout consumido y cerrado por el test → el proceso termina con exit 0 (contrato de la capa de salida).
      Verificación: los tests 3.x fallan (el subcomando no existe → clap rc 2) mientras el resto de integración pasa.

## 4. Fase GREEN — implementación

- [ ] 4.1 `clap_complete = "4.5"` en `[dependencies]` de Cargo.toml + regenerar Cargo.lock y COMMITEARLO en el mismo cambio (regla `--locked`). `cargo tree -e normal` muestra `clap_complete`.
- [ ] 4.2 `src/output.rs`: nueva `pub fn print_block(texto: &str)` que escribe el bloque por la capa de salida con el contrato de `print_line` (BrokenPipe → exit 0; otros errores → stderr en español y exit 1). Documentar en el doc-comment del módulo.
- [ ] 4.3 `src/lib.rs`: enum `GenerationShell { Bash, Zsh, Fish }` con `FromStr` (error en español con los tres shells) y `Display`; `pub fn generate_completions(shell: GenerationShell, cmd: &mut clap::Command) -> String` que genera el script vía `clap_complete::generate` a un String interno. `clap_complete` debe resolverse (4.5.x).
- [ ] 4.4 `src/main.rs`: variante `Completions` en `Command` con `#[arg(value_parser = ...)]` para `GenerationShell` (sin el trait ValueEnum: el value_parser se encarga); rama que construye `Args::command()` (traits `Parser`/`CommandFactory`), llama a `generate_completions` y emite con `print_block`. No cambia ninguna rama de Show/Edit/Remove.
      Verificación: toda la suite (unit + integración, incluidos RED 2.x/3.x y los 306 existentes) en verde; `cargo fmt --check` y `cargo clippy --all-targets --all-features -- -D warnings` limpios; evidencia manual: `id3cli completions bash | bash -n` → exit 0; `id3cli completions zsh | head -3` muestra `#compdef id3cli`; `id3cli completions fish | head -2` muestra `complete -c id3cli`; `id3cli completions powershell` → exit 2 con el error en español.

## 5. Fase GREEN — e2e

- [ ] 5.1 `scripts/e2e/e2e_completions.sh`: usa e2e_setup + helper de binario (--bin opcional), SIN e2e_gen_audio; verifica con e2e_run/check_rc/check_show_contains: `completions bash` → rc 0 + `complete -F _id3cli id3cli` presente + STDOUT pasa `bash -n` (oráculo de sintaxis real; la verificación se hace capturando la salida a un fichero y ejecutando `bash -n <fichero>`); `completions zsh` → rc 0 + `#compdef id3cli`; `completions fish` → rc 0 + `complete -c id3cli`; los tres con stderr vacío; `completions powershell` → rc ≠ 0 + stderr contiene los tres shells; contabilizado en E2E_PASS/E2E_FAIL con mensajes en español, resumen `e2e_summary "completions"`.
- [ ] 5.2 Recipe `just e2e`: añade `scripts/e2e/e2e_completions.sh --bin target/debug/id3cli` tras el bucle de los 5 formatos (una sola construcción). Añadir recipe `e2e-completions` para ejecutarlo solo.
- [ ] 5.3 Medir el recuento de aserciones del script (p. ej. «completions: N aserciones») con `scripts/e2e/e2e_completions.sh --bin target/debug/id3cli` — numero MEDIDO, nunca supuesto.
- [ ] 5.4 Falsabilidad: corromper deliberadamente una aserción (p. ej. esperar `#compdef id3cli` en bash) en una copia temporal → exit 1 con la aserción nombrada; restaurar → exit 0. Verifica además `bash -n scripts/e2e/*.sh`.
      Verificación: `just e2e` completo en verde (5 formatos 479 aserciones + completions) y el reporte de la falsabilidad literal.

## 6. Fase REFACTOR — coherencia documental

- [ ] 6.1 Medir la suite (unit + integración) con `cargo test --no-fail-fast` tras GREEN y actualizar README.md: escudo `tests-<N>%20passing`, línea de desglose `<N> tests (<U> unitarios + <I> de integración)` y comentarios de los comandos de la sección Tests con los recuentos MEDIDOS.
- [ ] 6.2 README.md: sección nueva «Completions» (o subsección de Uso) documentando `id3cli completions bash|zsh|fish` con ejemplos de instalación en `.bashrc` (o `.bash_completion.d`), `.zshrc` (compinit) y `config.fish`; mención de `bash -n` como validación; NO romper las secciones verificadas por el test documental (Formatos admitidos, mapeos, degradaciones MP4 `©lyr`/`letra` y OGG `webp`+`OGG`+`no`, roadmap con OGG/Opus/FLAC `- [x]` y Speex `- [ ]`).
- [ ] 6.3 Roadmap del README: línea nueva `- [x] Completions para bash, zsh y fish (subcomando \`completions\`)`.
- [ ] 6.4 AGENTS.md y .github/copilot-instructions.md: recuentos 306 → medidos.
      Verificación: `cargo test --lib test_documentacion` en verde; no queda ningún `306` huérfano en los docs (grep) salvo lo que corresponda.

## 7. Verificación final y cierre

- [ ] 7.1 Barrido completo del director: `cargo test --no-fail-fast` (recuento final), `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `bash -n scripts/e2e/*.sh`, `just e2e` en verde, `openspec validate --specs` sin fallos, no-rotura de `show --json` y del modo legible.
- [ ] 7.2 Cierre git: commits convencionales en `feature/shell-completions` (chore(openspec) → feat: código+tests → test(e2e) → docs), Cargo.lock en el commit feat, PR a `development`, CI verde, fusión, `openspec archive shell-completions` con las 25 tareas marcadas, commit del archive.
- [ ] 7.3 Release única **v0.9.0**: PR `release: v0.9.0` development → main, `release-prepare.yml` (bump minor por el `feat:`), tag, `release.yml` (GitHub Release + crates.io), verificación de `max_version: 0.9.0` y `development` sincronizado con `main`.