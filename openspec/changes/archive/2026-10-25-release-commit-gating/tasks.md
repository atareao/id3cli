# Tasks

El change corrige el pipeline de release para que solo publique cuando los commits del rango justifiquen un bump, y termine sin publicar cuando solo haya commits no publicables (documentación, andamiaje, openspec, merge commits). Solo toca `.github/workflows/release-prepare.yml`. SIN dependencias nuevas. Sigue el patrón de los changes archivados: TDD estricto (con la falsabilidad del caso «sin release»), medición del incidente y coherencia documental. Nada de `.github/` se toca antes del STOP del grupo 1.

## 1. Artefactos de especificación (OpenSpec)

- [x] 1.1 Redactar `proposal.md` con Why/What Changes/Capabilities/Impact (el incidente v0.11.2, la clasificación por tipo, el caso sin bump; no-goals). — `proposal.md` redactado con el incidente medido (v0.11.2 por un merge de solo documentación), la clasificación por tipo y el caso sin bump
- [x] 1.2 Redactar `design.md` con las cinco decisiones y los riesgos. — `design.md` redactado con las cinco decisiones iniciales
- [x] 1.3 Redactar el delta `specs/release-publishing/spec.md` (ADDED) con el requisito de gating y sus seis escenarios, incluida la medición del incidente. — delta `specs/release-publishing/spec.md` redactado con 1 requisito ADDED y 6 escenarios
- [x] 1.4 Redactar `tasks.md` con este checklist. — `tasks.md` redactado con 6 grupos
- [x] 1.5 STOP: presentar la especificación al usuario y esperar aprobación explícita antes de tocar `.github/workflows/release-prepare.yml`. — presentado al usuario y aprobado explícitamente («Aprobado — arranca TDD RED»), con la política `refactor` → patch confirmada, antes de tocar `.github/`
- [x] 1.6 Validar el change: `openspec validate release-commit-gating --strict` sin advertencias. — `openspec validate release-commit-gating --strict` → «Change 'release-commit-gating' is valid», exit 0 (re-validado tras las correcciones de auditoría)

## 2. Fase RED — medir el comportamiento actual y la falsabilidad del gate

- [x] 2.1 Medir el estado actual sobre un rango de commits reales: `git log "$LAST_TAG"..HEAD --format="%s"` con el último tag y aplicar la lógica actual para dejar constancia de que un rango sin `feat`/breaking cae en `--patch`. — medido 2026-10-25: la lógica ACTUAL aplicada a un rango sin `feat`/breaking decide `--patch`, que es el defecto (incluso un rango vacío cae en `--patch`); `LAST_TAG=v0.11.2`, rango `v0.11.2..HEAD` vacío
      Verificación: salida literal del rango y del tipo decidido por la lógica actual (debe dar `--patch` para un rango de solo documentación).
- [x] 2.2 Escribir y probar (en local, con `bash`) la nueva lógica de clasificación sobre rangos sintéticos: (a) solo `docs`/`chore` → sin bump; (b) `feat` → minor; (c) `fix`/`perf`/`refactor` → patch; (d) breaking → major; (e) mezcla → mayor. — lógica nueva probada sobre 9 rangos sintéticos: solo docs/chore/merge → sin bump; `feat` → minor; `fix`/`perf`/`refactor` → patch; breaking → major; mezcla docs+fix+feat → minor; `feat(scope)` → minor; `chore: release` → sin bump; solo style/test/ci/build → sin bump
      Verificación: salida literal de los cinco casos con el tipo decidido por la función nueva.

## 3. Fase GREEN — pipeline

- [x] 3.1 `release-prepare.yml`: reescribir el paso `Determine bump type from commits` para clasificar el rango por tipo conventional, decidir por el mayor y exponer las salidas `release` (`true`/`false`) y `type`. — paso `Determine bump type from commits` reescrito: clasifica el rango por tipo, decide por el mayor y expone las salidas `release` y `type`. CORRECCIÓN POST-AUDITORÍA (hallazgo ALTO): el breaking también se detecta en el CUERPO (`--format="%B"`, `BREAKING[ -]CHANGE`), no solo en el asunto (`!:` anclado al tipo), porque la convención lo pone en el footer; antes un `feat:` con `BREAKING CHANGE:` en el cuerpo daba minor en vez de major. Patrones anclados al `:` para no confundir `feature:` con `feat:`; `git describe --match 'v*'`
- [x] 3.2 `release-prepare.yml`: condicionar `Bump version with vampus`, `Get new version`, `Generate changelog`, `Refresh Cargo.lock`, `Commit, tag and push` y `Sync development with main` con `if: steps.bump.outputs.release == 'true'`; conservar el guard `chore: release` a nivel de job. — pasos gateados con `if: steps.bump.outputs.release == 'true'` (Install git-cliff, Install vampus, Bump, Get new version, Generate changelog, Refresh Cargo.lock, Commit/tag/push, Sync development); `checkout`, `Determine bump` y `Cache cargo` siguen sin gatear; guard `chore: release` conservado. CORRECCIÓN POST-AUDITORÍA (hallazgo MEDIO): bloque `concurrency` (`group: release-prepare`, `cancel-in-progress: false`) para serializar ejecuciones y no crear dos tags
      Verificación: YAML válido; el camino «sin release» no ejecuta ningún paso que dependa de una versión; el camino «con release» conserva el push atómico y la sincronización.

## 4. Fase GREEN — verificación

- [x] 4.1 Validar el YAML (`python3 -c "import yaml..."`) y revisar el diff del paso de decisión y de los `if`. — `python3 -c "import yaml"` valida los tres workflows; el paso de decisión corre siempre y los 8 pasos dependientes de la versión llevan su `if`
- [x] 4.2 Simular el caso «sin release» con la lógica nueva sobre un rango de solo `docs`/`chore` → salida `release=false` y ningún paso de release. — caso «sin release» sobre un rango de solo docs/chore → `release=false` y ningún paso de release se ejecuta
- [x] 4.3 Simular el caso «con release» sobre un rango con `fix` → `release=true type=--patch`. — caso «con release» sobre un rango con `fix` → `release=true type=--patch`
      Verificación: salida literal de los dos casos.

## 5. Fase REFACTOR — coherencia documental

- [x] 5.1 Si `README.md`, `AGENTS.md`, `.github/copilot-instructions.md` o `GIT_FLOW.md` describen la regla de versionado («cualquier merge a main dispara release»), actualizarlos a la regla nueva (solo `feat`/`fix`/`perf`/`refactor`/breaking publican). — actualizadas las descripciones obsoletas de la regla de versionado: `AGENTS.md` (línea de `main`), `GIT_FLOW.md` (tabla de ramas, tabla de Conventional Commits con `perf` y la fila «ninguno» para docs/chore/style/test/ci/build, nota del rango sin publicables, descripción de `release-prepare.yml`) y `.github/copilot-instructions.md`; el `grep` de menciones obsoletas queda vacío
      Verificación: `grep` sin descripciones obsoletas.
- [x] 5.2 Comprobar que no se toca `ci.yml` ni `release.yml` (fuera de alcance). — `ci.yml` y `release.yml` NO se tocaron (fuera de alcance); el diff versionado es `release-prepare.yml` + `AGENTS.md` + `GIT_FLOW.md` + `.github/copilot-instructions.md`

## 6. Verificación final y cierre

- [x] 6.1 Barrido: YAML válido, `cargo test` sin cambios (331), `cargo fmt --check`, `cargo clippy -- -D warnings`, `openspec validate --specs` y `openspec validate --archived`. — barrido 2026-10-25: 3 workflows YAML válidos; `cargo test` 331 (178 + 153) sin cambios; `cargo fmt --check` limpio; `cargo clippy -- -D warnings` 0 warnings; `openspec validate --specs` 16 passed y 0 failed; `openspec validate --archived` 14 passed y 0 failed
- [x] 6.2 Cierre git: commits convencionales en `feature/release-commit-gating` (chore(openspec) → fix(ci)), PR a `development`, CI verde, fusión y `openspec archive release-commit-gating` con las tareas marcadas. — PR #38 fusionado en `development` vía squash (`ae97972`) con CI verde (jobs `musl` y `test`); commits convencionales en `feature/release-commit-gating` (`chore(openspec)` → `fix(ci)`); change archivado
- [ ] 6.3 Release del propio change (`fix(ci):` → patch → v0.11.3) por el flujo estándar; verificar que tras este change un push de solo documentación a `main` NO publica.
