# agentes-id3cli-gitflow.md — flujo Git (extracto literal del AGENTS.md previo)

> Extracto literal del `AGENTS.md` previo (poda 2026-10-31). Texto tal cual, sin resumen.

## Git workflow

Git Flow — see [GIT_FLOW.md](GIT_FLOW.md) for full branching model, auto-release pipeline, and conventional commit conventions. La recuperación de un release a medias está en [RELEASE.md](RELEASE.md).

- `main` — production (un merge aquí publica release **solo si** los commits lo justifican: `feat`→minor, `fix`/`perf`/`refactor`→patch, ruptura→major; solo `docs`/`chore`/`style`/`test`/`ci`/`build` o merges no publican)
- `development` — feature integration
- `feature/*` — work branches
- `hotfix/*` — urgent fixes from `main`

Commits: conventional-commit style (`feat:`, `fix:`, `refactor:`, `chore:`, etc.). CI determines version bump automatically — never bump manually.
