# Proposal

## Why

El 2026-10-25 el pipeline publicó TRES releases en 48 minutos: v0.11.0 (un `feat:`), v0.11.1 (un `refactor(test):`) y v0.11.2, esta última disparada por un commit de SOLO documentación —un merge `chore: sincroniza development en main`—. La causa está medida en `.github/workflows/release-prepare.yml`: el paso `Determine bump type from commits` clasifica en su rama `else` CUALQUIER commit que no sea `feat` ni breaking como `--patch`, de modo que `docs`, `chore`, `style`, `test`, `ci`, `build` y los merge commits producen un patch. El único filtro existente (`if: "!startsWith(github.event.head_commit.message, 'chore: release')"`) evita que el pipeline se dispare sobre su propio commit de release, pero NO sobre un merge de documentación: cualquier push a `main` sin un `feat` publica una versión. Resultado: GitHub y crates.io tienen v0.11.2, una versión sin ningún cambio de producto, y `main` == `development` == `c7c07c0`.

## What Changes

El paso de decisión pasa a clasificar TODOS los commits del rango `$LAST_TAG..HEAD` por su tipo conventional y a decidir el bump por el MAYOR: `BREAKING CHANGE`/`!:` → major, `feat` → minor, `fix`/`perf`/`refactor` → patch. Si NINGÚN commit del rango justifica un bump —solo hay `docs`/`chore`/`style`/`test`/`ci`/`build` o merge commits—, el pipeline MUST terminar con éxito SIN crear tag, SIN commit de release y SIN publicar; los pasos de bump, changelog, commit/tag/push y sincronización de `development` quedan condicionados a que exista un bump. Sin cambios en `src/`, `tests/` ni dependencias (`Cargo.toml`/`Cargo.lock` no cambian).

## Capabilities

### Modified Capabilities

- **release-publishing**: el pipeline MUST publicar solo cuando los commits del rango justifiquen un bump, y MUST terminar sin release cuando solo haya commits no publicables (documentación, andamiaje o merge commits).

## Impact

- `.github/workflows/release-prepare.yml`: el paso `Determine bump type from commits` (clasificación por tipo + decisión por el mayor + salida `release`) y un `if` condicionando los pasos posteriores.
- `openspec/specs/release-publishing/spec.md`: requisito nuevo (vía delta ADDED).
- No-goals: cambiar el empaquetado, el binario estático musl, el push atómico con reintentos ni la lista blanca `include`; despublicar v0.11.2 (se mantiene, ya está en crates.io); tocar `ci.yml` ni `release.yml`.
- Versionado de este change: `fix(ci):` → patch (v0.11.3). Nota: este mismo commit es un `fix`, así que con el pipeline actual disparará el release destino; a partir de él, los commits de solo documentación dejarán de hacerlo.
