# Tasks

El change incorpora el modo batch: `<FILE>...` (1..n) en `show`, `edit` y `remove`, aplicando la operación a todos los archivos, con no-rotura estricta N=1 (byte a byte contra el binario de referencia), cabecera legible y NDJSON para `show` con N>1, exit agregado (0 solo si todos OK), e2e `e2e_batch.sh`. Sigue el patrón de los changes archivados (`flac-support`, `json-output`, `shell-completions`): aditivo, TDD estricto, test documental con recuentos medidos y release minor única. Nada de `src/` ni `tests/` se toca antes del STOP del grupo 1. SIN dependencias nuevas: Cargo.toml/Cargo.lock NO cambian.

## 1. Artefactos de especificación (OpenSpec)

- [ ] 1.1 Redactar `proposal.md` con Why/What Changes/Capabilities/Impact (`<FILE>...` en show/edit/remove, NDJSON y cabeceras con N>1, no-rotura N=1, exit agregado; no-goals: subcomando batch, paralelismo, wildcards).
- [ ] 1.2 Redactar el delta `specs/batch-processing/spec.md` (capability NUEVA) con el contrato completo (véase el archivo).
- [ ] 1.3 Redactar el delta `specs/cli-output/spec.md` (MODIFIED) con los requisitos literales del spec base (pipe cerrado, avisos/errores, superficie con completions) reemplazados y ampliados a batch + ADDED para NDJSON/pipe batch.
- [ ] 1.4 Redactar el delta `specs/e2e-tooling/spec.md` (MODIFIED) con los requisitos literales del spec base (runner común, justfile) reemplazados y ampliados a batch + ADDED para el script no ligado a extensión.
- [ ] 1.5 Redactar el delta `specs/e2e-verification/spec.md` (MODIFIED + ADDED) con la capa batch y su falsabilidad.
- [ ] 1.6 STOP: presentar la especificación al usuario y esperar aprobación explícita antes de tocar `src/` ni `tests/`.
- [ ] 1.7 Validar el change: `openspec validate batch-processing` sin advertencias.

## 2. Fase RED — unitarios (src/tests.rs)

- [ ] 2.1 Tests de la cabecera de bloque legible: con N>1 la función produce `=== <ruta> ===` y con N=1 no se antecede cabecera.
- [ ] 2.2 Test del juntado NDJSON: N archivos → N líneas en el orden dado; el objeto de cada línea es el del archivo correspondiente.
- [ ] 2.3 Test del despliegue batch por archivo: sobre una lista con un archivo que falla, los demás se procesan, el fallo se acumula y el resultado indica «hubo fallos».
      Verificación: los tests 2.x fallan por compilación (E0425: funciones de batch aún inexistentes); el resto de la suite unitaria pasa.

## 3. Fase RED — integración CLI (tests/integration_test.rs)

- [ ] 3.1 `show f1 f2` (dos MP3 con tags) → exit 0, stdout contiene la cabecera `=== <ruta f1> ===` y `=== <ruta f2> ===`, y los valores de ambos archivos; stderr sin errores.
- [ ] 3.2 `show --json f1 f2` → exit 0, stdout tiene EXACTAMENTE 2 líneas y `jq -s` parsea ambas (length == 2), cada objeto con su `.formato` y sus tags; stderr vacío.
- [ ] 3.3 `edit --title X f1 f2` → exit 0, ambos archivos reeditados (releídos con la librería `id3` en los tests).
- [ ] 3.4 `edit --title X f1 INEXISTENTE f2` → exit ≠ 0, f1 y f2 SÍ editados, stderr menciona la ruta inexistente en español.
- [ ] 3.5 `remove --all f1 f2` → exit 0, ambos sin tags (releídos con `id3`/lofty).
- [ ] 3.6 Formato mixto: `edit --title X a.mp3 b.m4a` → exit 0, ambos editados por su backend.
- [ ] 3.7 `show` sin archivos → exit 2 (clap: argumento requerido).
      Verificación: los tests 3.x fallan (clap rechaza los argumentos múltiples con el campo `file` único) mientras el resto de integración pasa.

## 4. Fase GREEN — implementación

- [ ] 4.1 `src/main.rs`: `files: Vec<PathBuf>` con `#[arg(required = true, num_args = 1..)]` en Show/Edit/Remove; bucles de despacho por archivo (cada iteración usa la rama actual de `detect_supported_format`); sin cambios en el comportamiento por archivo.
- [ ] 4.2 `src/lib.rs`: funciones de batch testables — p. ej. `batch_show_heading(ruta) -> String` (devuelve `=== <ruta> ===`), `batch_show_json_line(ruta) -> String` (una línea NDJSON) y el despliegue que itera, despacha y acumula fallos (resultado con «hubo fallos»). Reutiliza la lógica por archivo existente.
- [ ] 4.3 Exit agregado: 0 solo si todos OK; fallos → stderr en español (ruta + motivo) y exit ≠ 0; avisos NO cuentan como fallo.
- [ ] 4.4 No-rotura N=1: verificar con el binario de referencia que `show f` y `show --json f` son byte a byte idénticos antes/después.
      Verificación: toda la suite (unit + integración, incluidos RED 2.x/3.x y los 318 existentes) en verde; `cargo fmt --check` y `cargo clippy --all-targets --all-features -- -D warnings` limpios; evidencias manuales: `show f1 f2` con cabeceras, `show --json f1 f2 | jq -s 'length'` == 2, `edit --title X f1 INEXISTENTE f2; echo $?` ≠ 0 con f2 editado, `show` sin archivos → exit 2.

## 5. Fase GREEN — e2e

- [ ] 5.1 `scripts/e2e/e2e_batch.sh`: genera 2-3 audios (extensión representativa, p. ej. mp3 + m4a) con `e2e_gen_audio`; `edit` batch con título/artista; verifica con `show` y ffprobe que TODOS se editaron; NDJSON con `jq -s` (n líneas = n archivos); archivo inexistente → rc ≠ 0 con el resto editado; contabilizado en E2E_PASS/E2E_FAIL con mensajes en español y `e2e_summary "batch"`.
- [ ] 5.2 Recipe `just e2e`: añade `scripts/e2e/e2e_batch.sh --bin target/debug/id3cli` tras completions (una sola construcción). Añadir recipe `e2e-batch`.
- [ ] 5.3 Medir el recuento de aserciones del script con `scripts/e2e/e2e_batch.sh --bin target/debug/id3cli` — número MEDIDO, nunca supuesto.
- [ ] 5.4 Falsabilidad: corromper deliberadamente una aserción (p. ej. esperar un título que no se escribió) en una copia temporal → exit 1; restaurar → exit 0. Verifica además `bash -n scripts/e2e/*.sh`.
      Verificación: `just e2e` completo en verde (5 formatos + completions + batch) y el reporte de la falsabilidad literal.

## 6. Fase REFACTOR — coherencia documental

- [ ] 6.1 Medir la suite (unit + integración) con `cargo test --no-fail-fast` tras GREEN y actualizar README.md: escudo `tests-<N>%20passing`, línea de desglose `<N> tests (<U> unitarios + <I> de integración)` y comentarios de los comandos de la sección Tests con los recuentos MEDIDOS.
- [ ] 6.2 README.md: sección nueva «Modo batch» (o subsección de Uso) documentando `<FILE>...`, la cabecera `=== <ruta> ===`, el NDJSON (`jq -s`), el exit agregado y ejemplos `edit --title X f1 f2 ...`; NO romper las secciones verificadas por el test documental.
- [ ] 6.3 Roadmap del README: línea nueva `- [x] Modo batch (\`<FILE>...\` en \\`show\\`, \\`edit\\` y \\`remove\\`)`.
- [ ] 6.4 AGENTS.md y .github/copilot-instructions.md: recuentos 318 → medidos.
      Verificación: `cargo test --lib test_documentacion` en verde; no queda ningún recuento huérfano en los docs (grep).

## 7. Verificación final y cierre

- [ ] 7.1 Barrido completo del director: `cargo test --no-fail-fast` (recuento final), `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `bash -n scripts/e2e/*.sh`, `just e2e` en verde, `openspec validate --specs` sin fallos, no-rotura N=1 byte a byte contra el binario de referencia.
- [ ] 7.2 Cierre git: commits convencionales en `feature/batch-processing` (chore(openspec) → feat: código+tests → test(e2e) → docs), PR a `development`, CI verde, fusión, `openspec archive batch-processing` con las tareas marcadas, commit del archive.
- [ ] 7.3 Release única **v0.10.0**: PR `release: v0.10.0` development → main, `release-prepare.yml` (bump minor por el `feat:`), tag, `release.yml` (GitHub Release + crates.io), verificación de `max_version: 0.10.0` y `development` sincronizado con `main`.