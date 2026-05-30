# Changelog

## [0.2.2] - 2026-05-30

### Features
- Añade integración de prueba de álbum en tests de integración
- Implementa comando `remove` para eliminar tags específicos

### Documentation
- Agrega AGENTS.md con instrucciones para OpenCode
- Agrega GIT_FLOW.md con modelo de branching y release automático
- Agrega cliff.toml para generación automática de changelog
- Agrega CI workflow (fmt, clippy, build, test en PRs/pushes)

### Miscellaneous Tasks
- Migra release pipeline a Git Flow: release-prepare.yml + release.yml automáticos
- Actualiza .justfile con recetas lint/fmt/fmt-fix
- Publicación automática en GitHub Releases y crates.io

## [0.2.1] - 2026-02-01

### Features
- Actualiza y amplia la CLI con soporte para comandos show y edit, mejor estructura, nuevos tags y mejoras en manejo de metadatos

## [0.1.7] - 2026-01-22

(No changes between v0.1.6 and v0.1.7 — tag alignment)

## [0.1.6] - 2026-01-22

### Features
- Agrega soporte para metadatos de Apple: flags de compilación y órdenes de clasificación

## [0.1.5] - 2026-01-22

### Features
- Actualiza soporte para letras (lyrics) y URL en la gestión de tags ID3 en CLI
- Añade tarea de instalación con cargo en Justfile

## [0.1.4] - 2026-01-22

### Features
- Actualiza funciones de detección automática de MIME y soporte para formatos de carátula soportados (JPG, PNG, WEBP)

## [0.1.3] - 2026-01-22

(Internal changes — no user-facing features)

## [0.1.2] - 2026-01-22

### Features
- Actualiza flujo de trabajo para cargar binarios y sumas de verificación en lanzamientos

## [0.1.1] - 2026-01-22

### Features
- Añade instrucciones detalladas para contributors y configuración CI/CD

## [0.1.0] - 2026-01-22

### Features
- Initial release: CLI para añadir tags ID3 y carátulas a archivos MP3
- Soporte para metadatos básicos: título, artista, álbum, año, género, pista
- Soporte para carátulas en formato JPG
- Integración con crate id3 para lectura/escritura de tags ID3v2