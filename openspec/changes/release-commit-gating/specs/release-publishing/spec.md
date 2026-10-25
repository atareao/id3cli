# Spec Delta

## Purpose

El pipeline de release publica hoy una versión por cada push a `main`: el paso `Determine bump type from commits` de `.github/workflows/release-prepare.yml` clasifica en su rama `else` cualquier commit que no sea `feat` ni breaking como `--patch`, de modo que `docs`, `chore`, `style`, `test`, `ci`, `build` y los merge commits producen un patch. La medición del 2026-10-25 lo dejó escrito: un merge de SOLO documentación (`chore: sincroniza development en main`) publicó v0.11.2, una versión sin cambios de producto. Este delta fija que el pipeline MUST publicar solo cuando los commits del rango justifiquen un bump, y MUST terminar sin release cuando solo haya commits no publicables. NO toca el push atómico, la lista blanca `include`, el binario estático ni el job de integración continua, que siguen contratados en los requisitos ya presentes.

## ADDED Requirements

### Requirement: El pipeline MUST publicar una release solo cuando los commits del rango justifiquen un bump

El paso de decisión de `.github/workflows/release-prepare.yml` MUST clasificar TODOS los commits del rango `$LAST_TAG..HEAD` por su tipo conventional y MUST decidir el bump por el MAYOR presente: `BREAKING CHANGE`/`!:` → major, `feat` → minor, `fix`/`perf`/`refactor` → patch. Si NINGÚN commit del rango justifica un bump —solo hay `docs`, `chore`, `style`, `test`, `ci` o `build`, o merge commits sin tipo publicable—, el pipeline MUST terminar con éxito SIN crear tag, SIN commit de release y SIN disparar la publicación; y los pasos de bump, changelog, commit/tag/push y sincronización de `development` MUST quedar condicionados a que exista un bump.

#### Scenario: Solo documentación o andamiaje MUST NOT publicar

- **WHEN** el rango desde el último tag solo contiene commits `docs`/`chore`/`style`/`test`/`ci`/`build` o merge commits
- **THEN** el pipeline MUST terminar con código 0, MUST NOT crear ningún tag nuevo, MUST NOT commitear un `chore: release` y MUST NOT publicar; la medición del 2026-10-25 dejó constancia del estado que se corrige: el merge `chore: sincroniza development en main` (solo documentación y `openspec/`) publicó v0.11.2 y disparó `release.yml`

#### Scenario: `feat` MUST publicar un minor

- **WHEN** el rango contiene al menos un commit `feat` y ningún breaking
- **THEN** el bump MUST ser minor; la medición del 2026-10-25 lo dejó escrito: el `feat:` del binario estático musl produjo v0.11.0

#### Scenario: `fix`/`perf`/`refactor` MUST publicar un patch

- **WHEN** el rango contiene al menos un commit `fix`, `perf` o `refactor` y ningún `feat` ni breaking
- **THEN** el bump MUST ser patch; la medición del 2026-10-25 lo dejó escrito: el `refactor(test):` del oráculo JSON produjo v0.11.1

#### Scenario: Breaking MUST publicar un major

- **WHEN** el rango contiene un commit con el marcador de ruptura, ya sea `!:` en el ASUNTO o `BREAKING CHANGE` en el CUERPO del mensaje (el footer, donde lo pone la convención)
- **THEN** el bump MUST ser major, con independencia de los demás tipos presentes

#### Scenario: Con tipos mezclados gana el mayor

- **WHEN** el rango contiene a la vez un `docs`, un `fix` y un `feat`
- **THEN** el bump MUST ser el mayor de ellos (minor), y el `docs` MUST NOT rebajarlo

#### Scenario: El commit de release del propio pipeline MUST NOT re-dispararlo

- **WHEN** el job empuja su commit `chore: release v${V}` a `main`
- **THEN** el pipeline MUST NOT volver a dispararse; el commit de release es un `chore` y MUST quedar fuera de la clasificación, además del guard explícito a nivel de job

### Requirement: El pipeline MUST NOT ejecutar dos releases en paralelo

El job de `.github/workflows/release-prepare.yml` MUST declarar un bloque `concurrency` que serialice sus ejecuciones (`group` fijo y `cancel-in-progress: false`), de modo que dos pushes consecutivos a `main` MUST NOT calcular el bump en paralelo ni crear dos tags; la segunda ejecución MUST esperar a la primera en lugar de solaparse.

#### Scenario: Dos pushes rápidos se serializan

- **WHEN** dos pushes a `main` llegan casi a la vez
- **THEN** las dos ejecuciones del job MUST serializarse en el mismo grupo de `concurrency` (la segunda espera a la primera), MUST NOT calcular el bump en paralelo y MUST NOT crear dos tags; medido el 2026-10-25: el flujo era read-modify-write sobre `.vampus.yml` y el tag, sin `concurrency`, de modo que dos runs con el mismo `LAST_TAG` podían intentar la misma versión y chocar en el push
