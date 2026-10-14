# Changelog
## [0.5.1] - 2026-10-14

### Bug Fixes

- Deja de tratar el cierre del pipe como un error

### Documentation

- *(openspec)* Propone el cambio broken-pipe-and-crate-metadata
- Documenta la salida, el modulo outln! y los recuentos
- *(openspec)* Corrige el guardia del lock tras medir un falso verde
- *(openspec)* Archiva el cambio broken-pipe-and-crate-metadata

### Miscellaneous Tasks

- Declara keywords y categories para crates.io
- Valida el lock con --locked y lo refresca antes del commit de release

### Testing

- Cubre el cierre del pipe y los metadatos del paquete
## [0.5.0] - 2026-10-14

### Bug Fixes

- Deja de conservar una cabecera ID3v2 vacia al borrar el ultimo campo

### Documentation

- Registra el baseline y corrige el recuento de tests
- *(openspec)* Añade la propuesta del cambio m4a-support
- *(openspec)* Documenta el spike de fixtures y la API real de lofty
- *(openspec)* Registra las decisiones de la migración y las regresiones detectadas
- *(openspec)* Registra la detección por contenido y el recuento de tests
- *(openspec)* Documenta el mapeo MP4 y corrige el mecanismo de WebP
- *(openspec)* Registra las limitaciones de MP4 y el escollo de la API por extensión
- *(openspec)* Registra la omisión de los campos sin equivalente en MP4
- Actualiza README, AGENTS y el registro del cambio
- *(openspec)* Archiva el cambio m4a-support
- *(openspec)* Propone el cambio remove-empty-id3-header
- *(openspec)* Archiva el cambio remove-empty-id3-header
- Alinea README y AGENTS con el estado real del proyecto
- Reescribe la guía de Copilot, que describía un CLI solo-MP3
- Alinea AGENTS.md con lofty y el soporte de MP4
- Recoge el MP4 en la lista de funciones y en remove --all

### Features

- Detecta el formato por contenido y rechaza los no soportados
- Añade soporte de metadatos MP4 con átomos ilst
- Escribe la carátula en el átomo covr de MP4
- Avisa y omite --url y --original-artist en MP4
- Adapta los textos y la ayuda del CLI a MP3 y MP4

### Miscellaneous Tasks

- Sincroniza Cargo.lock con Cargo.toml
- Release v0.5.0

### Refactor

- Migra el backend de tags de id3 a lofty manteniendo la paridad

### Testing

- Añade la caracterización de MP3 como red de seguridad de la migración
- Sustituye los fixtures MP3 falsos por un fixture de audio real
- Cubre el cambio de un solo campo y la integridad de la caratula
- Fija el contrato de borrado del ultimo campo
## [0.4.3] - 2026-10-12

### Miscellaneous Tasks

- Añade actions:write para disparar release.yml vía API
- Release v0.4.3
## [0.4.2] - 2026-10-12

### Miscellaneous Tasks

- Añade GH_TOKEN al paso de disparo de release.yml
- Release v0.4.2
## [0.4.1] - 2026-10-12

### Bug Fixes

- Release-prepare checkout no usa GH_PAT (repo público) y push con PAT explícito

### Features

- Remove --all borra físicamente ID3v1 e ID3v2

### Miscellaneous Tasks

- Release v0.4.0
- Sync development vía git push y disparo explícito de release.yml
- Release v0.4.1
## [0.3.1] - 2026-05-30

### Miscellaneous Tasks

- Add package metadata for crates.io publishing
- Release v0.3.1
## [0.3.0] - 2026-05-30

### Features

- Add remove --all flag and fix CI/clippy

### Miscellaneous Tasks

- Release v0.3.0
## [0.2.3] - 2026-05-30

### Miscellaneous Tasks

- Release v0.2.3
## [0.2.2] - 2026-05-30

### Features

- Add gitflow release pipeline, AGENTS.md, CI workflow, and auto-changelog
## [0.1.3] - 2026-01-22

### Other

- 25:29.096662Z[0m [31mERROR[0m [2mdime[0m[2m:[0m Internal server error
## [0.1.1] - 2026-01-22
