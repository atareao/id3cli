# Proposal

## Why

La verificación de `id3cli` vive hoy en la suite Rust (281 tests: 163 unitarios + 118 de integración), que relee con la librería `id3` lo que escribe `lofty` — patrón cross-herramienta — pero siempre sobre fixtures versionados y por canales de librería o de `Command`. No existe ninguna comprobación que ejercite el **binario real de extremo a extremo** sobre un archivo generado al vuelo, leyendo lo escrito con un lector independiente de terceros (`ffprobe`) y verificando además la **carátula** como flujo de imagen (no solo como metadato). El repo tampoco tiene un punto de entrada local que cualquiera pueda ejecutar para confirmar que todos los tags y las carátulas se leen y se escriben en los cuatro formatos soportados (MP3, MP4/M4A, OGG Vorbis y Ogg Opus).

## What Changes

- **Tooling e2e versionado, datos NO versionados.** Se añaden `scripts/e2e/e2e_lib.sh` (runner común fuenteable, bash `set -euo pipefail`) y cuatro scripts de flujo por extensión — `e2e_mp3.sh`, `e2e_m4a.sh`, `e2e_ogg.sh`, `e2e_opus.sh` — que generan **localmente** (bajo `/tmp/opencode/e2e/<ext>/`, con `mktemp -d` y `trap`) un audio de silencio con `ffmpeg` (`anullsrc`, ~1-2 s, `-map_metadata -1`; mp3 → `libmp3lame`, m4a → `aac`, ogg → `libvorbis`, opus → `libopus`) y una carátula `cover.jpg` y `cover.png` (`testsrc`), sin versionar ningún archivo generado y sin dependencias nuevas.
- **CLI de los scripts.** Cada script acepta `--bin RUTA` (binario; por defecto `cargo build` una vez por ejecución y `target/debug/id3cli`) y `--keep` (no limpiar el temporal). Exit 0 = todo OK; exit 1 = fallo con mensaje claro en español y nombre de la aserción fallida.
- **Verificación en dos capas.** `id3cli show` relee lo escrito (se grep el **valor**, nunca etiquetas ANSI) y `ffprobe` actúa como lector independiente con una **tabla de mapeo de clave por formato** dentro del runner (medida sobre archivos reales: MP3/MP4 se leen por `format_tags` y OGG/Opus por `stream_tags`, con las colisiones documentadas — p. ej. `TYER`/`TDRC` en MP3 y `--year`/`--date` en MP4 se exponen bajo la misma clave). La carátula se verifica extrayendo el stream de imagen (`-map 0:v:0 -c copy -frames:v 1`) y comparando bytes contra la original.
- **Cobertura.** Los **20 tags** de `VALID_TAG_NAMES` (title, artist con unión `"; "`, album, year, genre, track `N`/`N/TOTAL`, season ídem, date, copyright, composer, subtitle, original_artist, album_artist, cover, lyrics, url, compilation, album_sort, artist_sort, title_sort) más el roundtrip de carátula JPG y PNG en los cuatro formatos. Por formato: en MP3 se verifican 20/20 escritos y releídos; en MP4, OGG y Opus se verifican 18 escritos y releídos + `url` y `original_artist` como **omitidos con aviso por stderr** y ausentes en ffprobe.
- **Degradaciones modeladas.** Colisión `--year`/`--date` en `©day` (gana `--date` con aviso) en MP4. WebP queda **fuera de alcance**: solo JPG/PNG, con el hook de ampliación documentado en el runner.
- **Falsabilidad.** Un caso negativo deliberado (corromper un valor esperado o forzar un fallo) MUST terminar con exit 1 y mensaje en español, igual que los casos negativos de `generate_sample_*.sh`.
- **Recipes en `.justfile`.** `e2e` (los 4 en serie, deteniéndose en el primer fallo) y `e2e-mp3`, `e2e-m4a`, `e2e-ogg`, `e2e-opus` con paso opcional de `--bin`/`--keep`. Documentación de uso en `README.md` («Verificación e2e local»).
- **Sin tocar el código Rust.** Los scripts no modifican `src/` ni `tests/`; la suite MUST seguir en 281 tests (163 + 118) con 0 fallos y fmt/clippy limpios.

## Capabilities

### New Capabilities

- `e2e-tooling`: contratos del tooling de verificación e2e — ubicación y nombres de los scripts, CLI (`--bin`, `--keep`, exit 0/1), funciones del runner común, recipes del `justfile` y restricciones (nada versionado, temporal bajo `/tmp/opencode/e2e/`, `set -euo pipefail`, mensajes en español).
- `e2e-verification`: la matriz de verificación de extremo a extremo — los 20 tags con su forma por formato (escrito y releído vs. omitido con aviso), roundtrip de carátula JPG/PNG en los cuatro formatos, degradaciones MP4 y OGG, multi-artista con `"; "`, track/season con y sin total, `remove`/`remove --all` con vendor preservado y audio reproducible, y falsabilidad.

### Modified Capabilities

Ninguna: el cambio no altera comportamiento del CLI ni de la librería, y no toca ningún spec base existente.

## Impact

- **Código:** ninguno en `src/` ni `tests/` (los scripts son tooling independiente). Los fixtures y scripts `generate_sample_*.sh` existentes no se tocan.
- **Tooling nuevo:** `scripts/e2e/e2e_lib.sh`, `scripts/e2e/e2e_mp3.sh`, `scripts/e2e/e2e_m4a.sh`, `scripts/e2e/e2e_ogg.sh`, `scripts/e2e/e2e_opus.sh` (versionados). Datos generados en `/tmp/opencode/e2e/` (NO versionados).
- **Documentación:** `.justfile` (recipes `e2e*`), `README.md` (sección «Verificación e2e local»).
- **Specs del repositorio:** dos capabilities nuevas, `e2e-tooling` y `e2e-verification`.
- **Dependencias:** ninguna; se requieren `ffmpeg` y `ffprobe` en el PATH del desarrollador que ejecute los scripts (no son dependencias de build ni de runtime del paquete).
- **Versión:** sin cambio. Commit convencional **`test(e2e): …`** — nunca `feat:`, porque `release-prepare.yml` hace `grep -q "^feat"` → `--minor` y el tooling interno de verificación no debe liberar release. PR a `development` (rama `feature/e2e-local-helpers`), sin PR a `main`, y `openspec archive e2e-local-helpers --yes` al fusionar.
- **Tests existentes:** no se modifican; definición de no-rotura: `cargo test --no-fail-fast` sigue en 281 tests (163 unitarios + 118 de integración) con 0 fallos.