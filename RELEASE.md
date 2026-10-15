# Release

El proceso de release es **automático** — consulta [GIT_FLOW.md](GIT_FLOW.md) para el flujo completo.

Resumen: al mergear un PR a `main`, CI detecta el bump, ejecuta vampus, genera changelog, crea tag, y publica en GitHub Releases + crates.io.

## Qué se publica

El paquete que se sube a crates.io se limita a la lista blanca `include` de `Cargo.toml`:

- **Código fuente:** `src/**`.
- **Tests con sus ficheros de ejemplo:** `tests/**`, que pesan 9.031 bytes entre los tres.
- **Documentación y metadata:** `README.md`, `LICENSE`, `CHANGELOG.md`, `Cargo.toml` y `Cargo.lock`.

El andamiaje interno del repositorio —`.github/`, `openspec/`, `AGENTS.md`, `GIT_FLOW.md`, `RELEASE.md`, `cliff.toml`, `.justfile`, `.vampus.yml` y `.gitignore`— **no viaja** en el paquete.

Los patrones de primer nivel van anclados a la raíz con una barra inicial (`/README.md`), porque dentro de `include` la semántica es la de gitignore: sin la barra, un patrón casa a cualquier profundidad y llegó a arrastrar al paquete un fichero ajeno del andamiaje (`openspec/changes/archive/2026-10-14-remove-empty-id3-header/README.md`).

`Cargo.lock` viaja **a propósito**: se publica un binario y la compilación debe ser reproducible.

## Recuperación de un release a medias

1. **Síntoma:** el run `Prepare Release` falla en el paso `Commit, tag and push`.
2. **Averigua el estado real antes de tocar nada:**
   - `git fetch origin --tags && git log --oneline -3 origin/main` — ¿aparece un commit `chore: release vX.Y.Z`?
   - `git ls-remote --tags origin | grep vX.Y.Z` — ¿existe el tag?
3. **Si el commit de release no está en `main`:** no hay nada que reparar. El push atómico no dejó estado a medias, así que el siguiente push a `main` reintenta la release desde el mismo estado, con la misma versión y sin doble bump.
4. **Si el commit de release sí está en `main` y el tag no existe** (el caso que se dio el 2026-10-14 con la 0.5.1): crea el tag apuntando a ese commit y empújalo, y después sincroniza `development` con `main`:
   ```
   git fetch origin --tags
   git tag vX.Y.Z <commit-del-release>
   git push origin vX.Y.Z
   git push origin origin/main:development
   ```
5. **Antes de lanzar la publicación, mira si ya está en marcha:** un tag empujado por una persona SÍ dispara `release.yml` (evento `push`), mientras que los tags que empuja el propio pipeline con su `GH_PAT` no lo disparan (los cuatro últimos releases figuran como `workflow_dispatch`). Comprueba `gh run list --workflow release.yml --limit 3` y, si ya hay un run para ese tag, **no** lances `gh workflow run release.yml`: publicarías dos veces.

> [!warning] **Nunca reejecutes el job fallido.** El bump se recalcula desde el estado actual, así que un `vampus upgrade --patch` sobre un árbol ya en `0.5.1` daría `0.5.2`: un rerun produce una release fantasma. La recuperación siempre es volver a empujar a `main` desde el estado limpio.
>
> **El arreglo del pipeline solo actúa en el release siguiente.** El workflow que se ejecuta es el de la rama que dispara el evento, no el de la rama donde se desarrolla el cambio.