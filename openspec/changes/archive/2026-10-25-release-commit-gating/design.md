# Design

## Problema

El pipeline de release decide el bump con un `else` que captura todo lo que no sea `feat` o breaking. Eso convierte cualquier push a `main` en una publicación: un merge de documentación, un cambio de `openspec/`, o un commit `chore:` producen un patch. La medición del 2026-10-25 lo dejó escrito: v0.11.2 se publicó por un merge `chore: sincroniza development en main`, sin ningún cambio de producto. Publicar versiones vacías contamina crates.io, el changelog y el historial de tags, y degrada la señal de «ha salido una versión» para el usuario.

## Decisión 1 — Clasificar el rango por tipo conventional y decidir por el MAYOR

Hoy la decisión es un `if/elif/else` sobre el conjunto de commits, con un `else` que siempre publica. Se sustituye por una clasificación explícita: se recorre el rango `$LAST_TAG..HEAD`, se identifica el mayor tipo presente —`BREAKING CHANGE`/`!:` → major, `feat` → minor, `fix`/`perf`/`refactor` → patch— y el resto (`docs`, `chore`, `style`, `test`, `ci`, `build`, `revert`, merges sin tipo) NO justifica release. Se sigue el modelo de conventional commits que el repo ya usa en los mensajes y que `git-cliff` ya conoce (`conventional_commits = true` en `cliff.toml`).

## Decisión 2 — El caso «sin bump» MUST terminar con éxito sin publicar

Cuando no hay ningún commit publicable, el job MUST terminar con código 0 y MUST NOT crear tag, commit de release ni disparar `release.yml`. No se trata de un fallo: es un push a `main` que no cambia el producto (documentación, andamiaje, openspec), y el pipeline simplemente no tiene nada que publicar. La alternativa —fallar el job— sería ruido; la alternativa actual —publicar un patch vacío— es el defecto que se corrige.

## Decisión 3 — Gatear los pasos con una salida, no duplicar lógica

El paso de decisión expone dos salidas: `release` (`true`/`false`) y `type` (`--major`/`--minor`/`--patch`). Los pasos `Bump version with vampus`, `Get new version`, `Generate changelog`, `Refresh Cargo.lock`, `Commit, tag and push` y `Sync development with main` llevan `if: steps.bump.outputs.release == 'true'`. Así el camino «sin release» no ejecuta nada que dependa de una versión nueva, y no hay lógica duplicada.

## Decisión 4 — El guard de `chore: release` sigue, y la sincronización de development se condiciona al release

El `if` a nivel de job que ignora los commits `chore: release` se conserva: evita que el propio commit de release re-dispare el pipeline (el commit que empuja el job es `chore: release vX.Y.Z`, y el nuevo gate ya lo ignoraría por tipo, pero mantenerlo es explícito y barato). La sincronización `main → development` (que usa `push --force`) se condiciona al release: si no hay bump, `development` puede quedar detrás de `main` por el merge, que es el estado normal de GitFlow entre releases, y la sincronización ocurrirá en el siguiente release real.

## Decisión 5 — No despublicar v0.11.2

crates.io no permite despublicar sin romper a quien ya la haya descargado, y borrar la GitHub Release dejaría crates.io con una versión huérfana. v0.11.2 se mantiene como está; la corrección evita las futuras, no reescribe el pasado. Decisión del usuario registrada.

## Decisión 6 — El breaking se detecta en el asunto Y en el cuerpo

La primera versión clasificaba con `git log --format="%s"` (solo el asunto). La auditoría detectó que `BREAKING CHANGE:` vive por convención en el FOOTER del mensaje, de modo que un `feat:` con `BREAKING CHANGE:` en el cuerpo se clasificaba minor en vez de major —sub-versionar un cambio incompatible—. Se lee el cuerpo completo (`--format="%B"`) para buscar `BREAKING[ -]CHANGE` y el asunto (`--format="%s"`) para el marcador `!:` anclado al tipo. Los patrones de tipo exigen el `:` (o `!:`) para no confundir `feature:` con `feat:`.

## Decisión 7 — Serializar las ejecuciones con `concurrency`

El flujo es read-modify-write sobre `.vampus.yml` y el tag. Sin `concurrency`, dos pushes rápidos a `main` calculan el mismo `LAST_TAG`, ejecutan `vampus upgrade` y crean el mismo tag, y el segundo push choca. Se añade un bloque `concurrency` con `group` fijo y `cancel-in-progress: false`: serializa sin cancelar (cancelar entre el tag y `gh workflow run` dejaría la release a medias). Hallazgo MEDIO de la auditoría; preexistente, pero el gate lo hace relevante al permitir runs que no publican encadenados con runs que sí.

## Alternativas descartadas

- **Filtrar por rutas de fichero** (p. ej. ignorar si solo cambia `openspec/**` o `*.md`): más frágil que el tipo de commit (un `feat` que solo toca `README` sí es producto; un `docs` que toca `src/` no existe por convención), y obliga a mantener una lista de rutas.
- **Fallar el job cuando no hay bump**: convierte un push legítimo de documentación en un rojo de CI, ruido.
- **Publicar siempre pero con `--skip-release`**: añade un modo que nadie usa y no aclara la política.
- **Un flag de commit para forzar release**: sobre-ingeniería para este problema; el tipo de commit ya lo expresa.

## Riesgos y mitigaciones

- **Un commit de producto mal tipado** (no conventional) no dispara release: mitigado por la convención del repo (AGENTS.md exige conventional commits) y porque `git-cliff` ya descarta los no-conventional del changelog (`filter_unconventional = true`). La política es type-based y se documenta.
- **`refactor` cuenta como patch**: es la convención del repo (el checkpoint de `json-test-oracle` esperaba `refactor(test): → patch`); se mantiene. Quien no quiera release por un refactor debe usar `chore`/`test` si no afecta al producto.
- **`revert`**: un `revert:` de un cambio de producto debería publicar; si se quiere cubrir, se añade a la lista de patch. Se documenta como caso a decidir en la implementación (por defecto NO justifica release, salvo que en la fase RED se mida lo contrario).
- **La sincronización condicionada deja `development` detrás de `main`**: estado normal de GitFlow; el siguiente release real vuelve a sincronizar.
