# HANDOFF — sesión 2026-10-25

Nota de traspaso para retomar el trabajo. Se puede borrar cuando la deuda de abajo esté cerrada.

## Estado del repositorio

- `main` == `development` == `91dd47b` (0/0, sincronizadas).
- Última release: **v0.11.3** (GitHub + crates.io). Anterior: **v0.11.1**.
- `openspec validate --specs` 16/16 · `--archived` 15/15 · `--all` 16/16.
- Suite: 331/331 (178 unit + 153 integración). Cambios activos: ninguno.

## Qué se hizo (por orden)

1. **Cabo 1** — Marcadas con evidencia las 20 tareas que quedaron sin marcar en el change archivado `2026-10-16-flac-support` (se habían hecho, faltaba el tracking). `validate --archived` pasó de 12/13 a 15/15.
2. **Cabo 2** — `git worktree prune`: eliminado el registro huérfano de `/tmp/opencode/id3cli-verify`.
3. **Cabo 4** — README: el árbol de fixtures ya incluye `sample.flac` y `generate_sample_flac.sh`.
4. **Cabo 5** — Confirmado que las issues #5 y #8 las cerró `atareao` el 2026-10-17 (`COMPLETED`), no en esta sesión.
5. **Cabo 3** — Change `json-test-oracle`: los tests de la serialización JSON pasan de verificar subcadenas a parsear con `serde_json::Value` (tipos + conjunto exacto de las 20 claves canónicas de `tags`), con falsabilidad demostrada. PR #37 → **v0.11.1**. Sin dependencias nuevas.
6. **Hallazgo** — El pipeline publicó **v0.11.2** por un merge de SOLO documentación (`chore: sincroniza development en main`): el paso `Determine bump type from commits` tenía un `else` que convertía todo lo no-`feat`/no-breaking en `--patch`, de modo que cualquier push a `main` publicaba. Tres releases en 48 min (0.11.0, 0.11.1, 0.11.2).
7. **Change `release-commit-gating`** → **v0.11.3**. El pipeline ahora publica solo si el rango justifica un bump: ruptura (`BREAKING CHANGE` en el cuerpo o `!:` en el asunto) → major; `feat` → minor; `fix`/`perf`/`refactor` → patch; `docs`/`chore`/`style`/`test`/`ci`/`build` o merges → **sin release**. Verificado EN VIVO: un push de solo documentación a `main` (reproduciendo el disparador de v0.11.2) terminó en verde con el log «Sin commits que justifiquen release…; no se publica.» y NO creó tag ni release.

## Deuda pendiente (candidata a change propio)

La auditoría de `release-commit-gating` (`rust-reviewer`) señaló dos riesgos PREEXISTENTES en `.github/workflows/release-prepare.yml`, fuera del alcance del change y por tanto **NO corregidos**:

### 1. `Sync development with main` usa `push --force`

El paso hace `git push ... main:development --force`, que sobrescribe `development` con `main`. Si entre el cálculo del bump y este paso se hubiera fusionado una feature en `development`, esos commits se destruyen. Además no tiene reintento ni `--atomic` (a diferencia del push de `main`).
**Recomendación:** `--force-with-lease` y/o comprobar antes que `development` no va por delante.
**Consecuencia conocida del gating:** el paso está condicionado al release, así que tras un push de solo documentación `development` puede quedar detrás de `main` hasta el siguiente release real (estado aceptado, ver `design.md` del change).

### 2. Doble disparo de `release.yml`

El tag se empuja con `GH_PAT` (dispara workflows por el trigger `push: tags v*`) **y** además se lanza `gh workflow run release.yml --ref "v${V}"`. Pueden correr dos builds concurrentes compitiendo por crear la misma GitHub Release.
**Recomendación:** quedarse con uno de los dos mecanismos (o confiar en el trigger del tag, o usar solo `gh workflow run`).

## Cómo retomar

```bash
cd /data/rust/id3cli
git log --oneline -5     # main == development en 91dd47b
openspec validate --all  # 16/16
```

No hay changes activos. Archivados recientes: `2026-10-25-json-test-oracle`, `2026-10-25-release-commit-gating`.

Para atacar la deuda: nuevo change OpenSpec (SDD → STOP → TDD → auditoría → release). Nota: con el pipeline ya gateado, un commit `refactor`/`fix` publica patch, pero un `docs`/`chore` no.
