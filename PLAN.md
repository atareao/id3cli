# PLAN — Helpers e2e por extensión (verificación local de tags y carátulas) — Implementation Plan

## Objetivo

Añadir **un script e2e por extensión soportada** (mp3, m4a, ogg, opus) + un **runner común** en bash, que verifican de extremo a extremo que `id3cli` **lee y escribe todos y cada uno de los tags y las carátulas** sobre un archivo de audio **generado localmente con ffmpeg**, que **no se versiona** (se queda en local, bajo `/tmp/opencode/e2e/`), de modo que el repo no se sobrecarga con archivos de audio ni imágenes nuevas. Los scripts en sí SÍ se versionan (son tooling); los datos sobre los que operan, NO.

## Arquitectura

Estructura de 4 scripts (uno por extensión) + lógica de verificación compartida en un runner común (`e2e_lib.sh`, fuenteable, `set -euo pipefail`), evitando duplicar ~200 líneas x4. Verificación en dos capas: **id3cli show** (el propio CLI relee lo escrito) + **ffprobe** como lector independiente (patrón cross-herramienta del repo: los tests de integración ya releen con `id3` lo que escribe `lofty`). Carátula: extracción del stream de imagen con ffmpeg y comparación por hash/bytes contra la original. Los fixtures (audio de silencio y carátulas JPG/PNG) se generan al vuelo con ffmpeg bajo `/tmp/opencode/e2e/<ext>/` y se limpian con `trap`; los scripts se versionan, los datos NO.

## Decisiones cerradas (con el usuario)

- Estructura: 4 scripts (uno por extensión) + lógica de verificación compartida en un runner común (evitar duplicar ~200 líneas x4).
- Verificación en dos capas: **id3cli show** (el propio CLI relee lo escrito) + **ffprobe** como lector independiente (patrón cross-herramienta del repo: los tests de integración ya releen con `id3` lo que escribe `lofty`). Carátula: extracción del stream de imagen con ffmpeg y comparación por hash/bytes contra la original.
- Carátulas: **JPG y PNG** en los cuatro formatos (los dos MIME universales; no se ejercita WebP en esta fase).
- Integración: **change de OpenSpec pequeño** (`e2e-local-helpers`) + PR a `development` siguiendo el gitflow del repo.

## Alcance técnico verificado (hecho por el director, no repetir)

### Tags que debe ejercitar el e2e (20, según VALID_TAG_NAMES del CLI)

- `title`
- `artist` (repetible, soporta múltiples)
- `album`
- `year`
- `genre`
- `track` (acepta `N` o `N/TOTAL`)
- `season` (ídem)
- `date` (YYYY-MM-DD o YYYY)
- `copyright`
- `composer`
- `subtitle`
- `original_artist`
- `album_artist`
- `cover` (ruta de imagen)
- `lyrics`
- `url`
- `compilation` (flag booleano)
- `album_sort`
- `artist_sort`
- `title_sort`

### Degradaciones por formato que el e2e DEBE modelar (no verificar como escritas donde el CLI no las escribe)

- **MP3**: sin degradaciones; los 20 tags se escriben y releen.
- **MP4/M4A**: `--url` y `--original-artist` se **omiten con aviso por stderr** y nunca se escriben como átomo freeform; `--year` y `--date` comparten el átomo `©day` (si se indican ambos gana `--date` con aviso); `©lyr` sin código de idioma (no bloqueante).
- **OGG (Vorbis y Opus)**: `--url` y `--original-artist` se **omiten con aviso por stderr** y nunca se escribe una clave inventada; `LYRICS` sin código de idioma (no bloqueante).
- WebP: fuera de alcance (usamos JPG/PNG), pero el runner debe dejar el hook documentado para ampliarlo.

Por tanto la verificación "todos los tags" es **por formato**: en MP3 se verifican 20/20; en MP4 y OGG/Opus, 18 tags se verifican escritos y releídos + 2 (`url`, `original_artist`) se verifican como **omitidos con aviso** (stderr contiene el aviso y ffprobe no muestra el campo).

### Fixtures locales (generados al vuelo, NUNCA versionados)

Todo bajo `/tmp/opencode/e2e/<ext>/` (la env del proyecto prefiere /tmp/opencode; usar `mktemp -d` dentro):

- Audio por formato con ffmpeg: `anullsrc` (silencio), ~1-2 s; mp3 → `libmp3lame`, m4a → `aac`, ogg → `libvorbis`, opus → `libopus`; con `-map_metadata -1` para partir sin tags.
- Carátulas: generar `cover.jpg` y `cover.png` con ffmpeg (p. ej. `-f lavfi -i testsrc=duration=0.1:size=320x240:rate=1 -frames:v 1`), sin depender de ImageMagick.
- Ambos tipos de archivo se crean en el temporal de la ejecución y se borran con `trap` (o `--keep` para depurar).

### Estructura de archivos propuesta

- `scripts/e2e/e2e_lib.sh` — runner común fuenteable (bash, `set -euo pipefail`):
  - `e2e_setup <ext>` (mktemp -d + trap de limpieza + build del binario si no se pasa `--bin`),
  - generación de audio + carátulas,
  - `e2e_run <args...>` (ejecuta el binario capturando stdout/stderr/código),
  - aserciones: `check_show_contains` (grep en stdout de show), `check_ffprobe_tag <clave> <valor>` (ffprobe -show_entries format_tags), `check_ffprobe_absent <clave>`, `check_cover_roundtrip <img>` (ffmpeg extrae el stream de imagen → cmp/hash contra original), `check_stderr_aviso <substring>`,
  - resumen final PASS/FAIL con recuento de aserciones.
- `scripts/e2e/e2e_mp3.sh`, `scripts/e2e/e2e_m4a.sh`, `scripts/e2e/e2e_ogg.sh`, `scripts/e2e/e2e_opus.sh` — flujo común por formato: `show` inicial (vacío/sin tags) → `edit` con TODOS los tags aplicables en 1-2 invocaciones → `show` verificación de cada tag → ffprobe verificación independiente → carátula JPG y PNG roundtrip → `remove` de campos concretos → `remove --all` (comentario vaciado preservando vendor y audio) → verificación final ffprobe de que el audio sigue siendo reproducible.
- Cada script acepta `--bin RUTA` (binario; por defecto `cargo build` y usa target/debug/id3cli) y `--keep` (no limpiar el temporal). Exit 0 = todo OK; exit 1 = fallo con mensaje claro en español y nombre de la aserción fallida.
- `.justfile`: recipes `e2e` (los 4 en serie), `e2e-mp3`, `e2e-m4a`, `e2e-ogg`, `e2e-opus`.

### Verificación de calidad de los scripts

- `bash -n` (sintaxis) en los 5 scripts; shellcheck si está disponible (si no, omitir documentándolo).
- Ejecución real de los 4 scripts: deben terminar 0 en verde.
- **Prueba de falsabilidad**: un caso negativo deliberado (p. ej. corromper el valor esperado de una clave o forzar un fallo) debe hacer que el script termine 1 — igual que los `generate_sample_*.sh` tienen modo `--check` con casos negativos.

## Tareas

### Tarea 0: Artifacts de especificación (OpenSpec)

**Archivos:**
- Crear: `openspec/changes/e2e-local-helpers/proposal.md`
- Crear: `openspec/changes/e2e-local-helpers/specs/e2e-tooling/spec.md`
- Crear: `openspec/changes/e2e-local-helpers/specs/e2e-verification/spec.md`
- Crear: `openspec/changes/e2e-local-helpers/design.md`
- Crear: `openspec/changes/e2e-local-helpers/tasks.md`

- [ ] **Paso 1:** Ejecutar `openspec new change e2e-local-helpers`.
- [ ] **Paso 2:** Redactar `proposal.md` con intención, alcance e impacto (tooling local de verificación, sin datos versionados).
- [ ] **Paso 3:** Redactar 2 capabilities: `e2e-tooling` (contratos de los scripts: nombres de funciones del runner, flags `--bin`/`--keep`, exit codes) y `e2e-verification` (escenarios BDD: `Given/When/Then` para todos los tags por formato, degradaciones MP4/OGG, roundtrip JPG/PNG, `remove --all` preservando vendor y audio).
- [ ] **Paso 4:** Redactar `tasks.md` con el checklist TDD (fases RED/GREEN/REFACTOR abajo).
- [ ] **Paso 5:** STOP — presentar la especificación al usuario y esperar aprobación explícita antes de tocar código.

### Tarea 1: RED (equivalente bash)

**Archivos:**
- Crear: `scripts/e2e/e2e_lib.sh` (esqueleto)
- Crear: `scripts/e2e/e2e_mp3.sh` (esqueleto)
- Crear: `scripts/e2e/e2e_m4a.sh` (esqueleto)
- Crear: `scripts/e2e/e2e_ogg.sh` (esqueleto)
- Crear: `scripts/e2e/e2e_opus.sh` (esqueleto)

- [ ] **Paso 1:** Escribir la matrix de escenarios en el spec (20 tags por formato, degradaciones, carátulas).
- [ ] **Paso 2:** Escribir los esqueletos de los scripts con las aserciones declaradas (aún fallando: binario no construido o valores que no cuadran).
- [ ] **Paso 3:** Demostrar el "rojo" en bash con los casos negativos de la verificación de calidad (falsabilidad): un fallo deliberado debe terminar en 1.

### Tarea 2: GREEN

**Archivos:**
- Crear: `scripts/e2e/e2e_lib.sh` (implementación completa)
- Crear: `scripts/e2e/e2e_mp3.sh` (implementación completa)
- Crear: `scripts/e2e/e2e_m4a.sh` (implementación completa)
- Crear: `scripts/e2e/e2e_ogg.sh` (implementación completa)
- Crear: `scripts/e2e/e2e_opus.sh` (implementación completa)
- Crear: `.justfile` (recipes `e2e`, `e2e-mp3`, `e2e-m4a`, `e2e-ogg`, `e2e-opus`)

- [ ] **Paso 1:** Implementar `e2e_lib.sh` con `e2e_setup <ext>`, `e2e_run <args...>`, aserciones (`check_show_contains`, `check_ffprobe_tag`, `check_ffprobe_absent`, `check_cover_roundtrip`, `check_stderr_aviso`) y resumen PASS/FAIL con recuento.
- [ ] **Paso 2:** Implementar los 4 scripts por formato con el flujo común (show inicial vacío → edit con todos los tags aplicables en 1-2 invocaciones → show por tag → ffprobe independiente → carátula JPG y PNG → remove de campos → remove --all → ffprobe final de reproducibilidad).
- [ ] **Paso 3:** Añadir las recipes del `.justfile`.
- [ ] **Paso 4:** Ejecutar los 4 scripts hasta que terminen 0 en verde (binario construido UNA vez por ejecución del runner, no por script).

### Tarea 3: REFACTOR

**Archivos:**
- Modificar: `scripts/e2e/*.sh` (revisión de estilo)
- Modificar: `README.md:«Verificación e2e local»` (nueva sección)

- [ ] **Paso 1:** Revisión de estilo bash, mensajes en español, coherencia de nombres entre runner y scripts.
- [ ] **Paso 2:** `bash -n` en los 5 scripts; shellcheck si está disponible (si no, documentar la omisión).
- [ ] **Paso 3:** Documentar la sección «Verificación e2e local» en el README (uso de `just e2e`, `--bin`, `--keep`).

### Tarea 4: Verificación final

**Archivos:**
- Modificar: `openspec/changes/e2e-local-helpers/*` (marcar tareas completadas)

- [ ] **Paso 1:** Los 4 scripts en verde (mp3, m4a, ogg, opus), cada uno verificando TODOS los tags aplicables + carátula JPG y PNG.
- [ ] **Paso 2:** Falsabilidad demostrada: un fallo deliberado → exit 1 con mensaje claro en español y nombre de la aserción fallida.
- [ ] **Paso 3:** Suite Rust intacta: `cargo test --no-fail-fast` sigue en 281 tests (0 fallos).
- [ ] **Paso 4:** `cargo fmt --check` y `cargo clippy --all-targets --all-features -- -D warnings` limpios (los scripts no afectan a cargo; solo se confirma que nada se rompió).

## Gitflow / release

- Commit convencional **`test(e2e): …`** o **`feat(e2e): …`** — IMPORTANTE: decidir `test:` o `feat:` según lo que el pipeline de release considere bump: si `feat:` dispara minor, usar `test:` para tooling interno sin release. Verificar en GIT_FLOW.md/Cargo.toml (vampus) qué tipos disparan bump y elegir el que NO libere (es tooling de verificación, no funcionalidad de usuario).
- PR a `development` (rama `feature/e2e-local-helpers` desde development). NO hay PR a main: tooling interno no libera release.
- El change de OpenSpec se archiva (`openspec archive e2e-local-helpers --yes`) al fusionar el PR (patrón del repo: el archive ocurre en el cierre, tras aprobación).

## Riesgos / notas

- ffprobe expone las claves por formato con nombres distintos (p. ej. en MP4 el formato puede mostrar `©nam` en vez de `title`): el runner debe mapear la clave ffprobe por formato (tabla en el script, similar a las tablas de mapeo del README). Verificar el mapeo real ejecutando ffprobe sobre un archivo con tags antes de fijarlo.
- `id3cli show` usa colores/estilo: las aserciones `check_show_contains` deben grepear subcadenas estables (el valor del tag), no etiquetas con estilos ANSI. Si show emite ANSI, usar `grep -a` o forzar la salida sin color si el CLI lo permite.
- artist múltiple: el CLI une con `"; "` — el e2e debe escribir 2 artistas y verificar el separador.
- track/season con total: verificar `N/TOTAL` en ambas formas (`7/12`); en MP4 el total se guarda como par de 16 bits y el CLI avisa si no es representable (no aplica a 7/12).
- El binario se construye UNA vez por ejecución del runner (`cargo build` si no hay `--bin`), no por script, para no sobrecargar.

## Criterios de terminado (Definition of Done)

- [ ] `scripts/e2e/e2e_lib.sh` + `e2e_mp3.sh` + `e2e_m4a.sh` + `e2e_ogg.sh` + `e2e_opus.sh` versionados, `bash -n` limpio.
- [ ] Los 4 scripts terminan 0 ejecutados de verdad (mp3, m4a, ogg, opus), cada uno verificando TODOS los tags aplicables + carátula JPG y PNG.
- [ ] Falsabilidad demostrada (un fallo deliberado → exit 1 con mensaje).
- [ ] `.justfile` con `just e2e` y por extensión.
- [ ] README con sección de uso.
- [ ] Change OpenSpec `e2e-local-helpers` completo y archivado; PR a development fusionado.
- [ ] `cargo test --no-fail-fast` sigue en 281 tests (0 fallos) y fmt/clippy limpios (nada del tooling debe romper la suite).