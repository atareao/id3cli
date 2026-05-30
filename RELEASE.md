# Release

El proceso de release es **automático** — consulta [GIT_FLOW.md](GIT_FLOW.md) para el flujo completo.

Resumen: al mergear un PR a `main`, CI detecta el bump, ejecuta vampus, genera changelog, crea tag, y publica en GitHub Releases + crates.io.