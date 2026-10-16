# Design

## Context

`id3cli` escribe y lee metadatos en cuatro formatos —MP3 (frames ID3v2.4 vía `lofty`), MP4/M4A (átomos `ilst`), OGG Vorbis y Ogg Opus (comentarios Vorbis)— y su verificación vive hoy en la suite Rust: 281 tests (163 unitarios + 118 de integración). Los tests de integración releen con la librería independiente `id3` lo que escribe `lofty` (`id3` es solo dev-dependency), patrón cross-herramienta que este cambio traslada al nivel de binario: **dos librerías que no comparten código verifican el mismo archivo**. Lo que no existe es una comprobación que ejercite el binario real de extremo a extremo sobre un archivo **generado al vuelo**, con un lector independiente externo (`ffprobe`) y con la carátula verificada como flujo de imagen (extracción y comparación de bytes), ejecutable localmente por cualquiera (`just e2e`) sin fixture ni binario versionado.

Este change no toca una sola línea de `src/` ni de `tests/`: es tooling independiente versionado en `scripts/e2e/`, y por eso no modifica ningún spec base — las capabilities `e2e-tooling` y `e2e-verification` son las dos únicas nuevas.

## Goals / Non-Goals

**Goals**

- Verificación e2e local del binario real en los cuatro formatos: los 20 tags de `VALID_TAG_NAMES` escritos y releídos —u omitidos con aviso donde el formato no puede representarlos— y las carátulas JPG y PNG con roundtrip de bytes.
- Lectura independiente con `ffprobe` (segunda capa), con una tabla de clave por formato medida sobre archivos reales, no supuesta.
- Cero dependencias nuevas, cero archivos generados versionados, cero cambios en el código Rust y en la suite (281 tests intactos).
- Recipes de `justfile` y documentación en `README.md` para que `just e2e` sea el punto de entrada.

**Non-Goals**

- Soportar WebP en esta fase (solo JPG/PNG; el runner deja el hook documentado para ampliarlo).
- Añadir aserciones del e2e a la suite `cargo test` (el e2e es un complemento, no un sustituto; los scripts son independientes de cargo).
- Cambiar el comportamiento del CLI ni de la librería: ninguna degradación descrita aquí es nueva, todas se modelan tal y como ya están implementadas y probadas.
- Liberar release: el commit es `test(e2e):`, no `feat:`, porque el pipeline de release hace `grep -q "^feat"` → `--minor` (verificado en `.github/workflows/release-prepare.yml:38`).

## Decisions

### 1. Bash + `ffmpeg`/`ffprobe`, sin dependencias nuevas

Los scripts SHALL ser bash (`set -euo pipefail`) y SHALL generar y verificar con `ffmpeg`/`ffprobe`, las dos herramientas que el proyecto ya usa para generar los fixtures (`generate_sample_*.sh`) y para verificar estructura. No se añade ninguna dependencia de crate: `ffmpeg`/`ffprobe` son requisitos del développeur que ejecuta los scripts, no del build ni del paquete publicado, y su ausencia MUST fallar con mensaje en español (spec `e2e-tooling`).

`ffprobe` es el lector independiente elegido porque lee los cuatro formatos y expone las claves que `lofty` escribe — con matices **medidos** que la tabla del runner fija (Decisión 6). El patrón cross-herramienta ya estávalidado en el repo: los tests de integración releen con `id3` lo que escribe `lofty`, de modo que la verificación es entre dos caminos distintos y no una tautología.

### 2. Un runner común fuenteable en lugar de duplicar el flujo por formato

Los cuatro scripts comparten ~200 líneas de lógica (setup, generación, ejecución con captura, aserciones, resumen) que difieren solo en: código de `ffmpeg` del audio, extensión, conjunto de tags aplicables y mapa de claves `ffprobe`. La lógica común SHALL vivir en `scripts/e2e/e2e_lib.sh` (fuenteable con `source`), que expone `e2e_setup <ext>`, la generación de audio y carátulas con `ffmpeg`, `e2e_run <args...>`, las aserciones `check_show_contains`, `check_ffprobe_tag`, `check_ffprobe_absent`, `check_cover_roundtrip` y `check_stderr_aviso`, y el resumen final PASS/FAIL con recuento. Los cuatro scripts por extensión SHALL declarar la tabla de su formato y ejecutar el flujo común, como los backends del CLI comparten la tabla de alias `classify_tag_alias`.

Como consecuencia, la **tabla de clave ffprobe está en un solo sitio** dentro del runner (Decisión 6), y añadir un quinto formato se reduce a un script nuevo con su código de `ffmpeg` y su columna en la tabla.

### 3. JPG y PNG, no WebP (con hook documentado)

La carátula se ejercita con los dos MIME universales, `image/jpeg` y `image/png`, que además son los dos únicos que `lofty` infiere con seguridad y los dos que se almacenan byte a byte idénticos al extraer con `-c copy` (medido: MP3, MP4 y Opus extraen **idénticos**). El WebP queda fuera de alcance deliberadamente: en MP4 se degrada (tipo reservado 0 con aviso) y el roundtrip de bytes no es representativo, mientras que en MP3 y OGG se escribe sin degradar; validar ese abanico de comportamientos por formato es un cambio aparte. El runner SHALL dejar el hook documentado (una función de generación de imagen parametrizable por extensión/MIME) para ampliarlo sin reescribir.

### 4. Temporal bajo `/tmp/opencode/e2e/<ext>/`, nunca versionado

El proyecto prefiere `/tmp/opencode` para material transitorio. Los scripts SHALL crear el temporal con `mktemp -d` bajo `/tmp/opencode/e2e/<ext>/`, generar ahí el audio silencioso (`anullsrc`, ~1-2 s, `-map_metadata -1`) y las carátulas (`testsrc` de 320x240), y limpiarlo con `trap` al terminar — siempre, con éxito o con fallo. `--keep` desactiva la limpieza para depurar e informa del directorio.

Los scripts se versionan porque son tooling del repo; los datos sobre los que operan NO, porque versionar binarios e imágenes generados sobrecargaría el repositorio sin aportar nada verificable. Esta es la misma división que ya aplica el proyecto con los fixtures versionados `tests/fixtures/sample.*` + sus scripts de regeneración, pero llevada al extremo: aquí ni siquiera hay fixture, todo se genera en la ejecución.

La generación con `ffmpeg` (en lugar de fixtures versionados) elimina además la variabilidad entre versiones de `ffmpeg` como problema del repo: cada ejecución genera su propio material y el e2e siempre compara contra lo generado en esa misma ejecución.

### 5. `test(e2e):` y PR a `development`, nunca release

El commit del cambio SHALL ser convencional **`test(e2e): …`** y nunca `feat:`. Verificado en `.github/workflows/release-prepare.yml` (línea 38): `echo "$COMMITS" | grep -q "^feat"` → `type=--minor`; cualquier commit `feat:` dispara una release de minor. El e2e es tooling interno de verificación —no funcionalidad de usuario—, así que el prefijo `test:` lo mantiene fuera del pipeline de release. El flujo es: rama `feature/e2e-local-helpers` desde `development`, PR a `development`, y `openspec archive e2e-local-helpers --yes` al fusionar. **No hay PR a `main`**.

### 6. La capa independiente necesita una tabla de clave por formato, medida, no supuesta

`ffprobe` no expone las mismas claves en los cuatro formatos, y la PLAN advirtió de que el mapeo real había que verificarlo antes de fijarlo. Se ejecutó la medición sobre archivos escritos por `id3cli` con `ffmpeg`/`ffprobe` 9.0.2 (2026-10-16), y el spec `e2e-verification` fija los resultados:

- **MP3 y MP4/M4A** exponen la metadata en `format_tags` (`-show_entries format_tags`) con claves normalizadas a minúsculas: `title`, `artist`, `album`, `genre`, `track=7/12`, `disc=1/2`, `date`, `composer`, `copyright`, `album_artist`, `compilation`, `title-sort`, `artist-sort`, `album-sort`; en MP3 además las crudas `TIT3` (subtitle) y `TOPE` (original-artist) y la de letra `lyrics-spa` (el sufijo es el código de idioma `spa` del frame `USLT`). En MP4, `date` es el átomo compartido `©day`.
- **OGG y Opus** NO exponen nada en `format_tags`: la metadata vive en los `stream_tags` (`-show_entries stream_tags`) con la **caja original** de las claves Vorbis (`TITLE`, `ARTIST`, `ALBUM`, `YEAR`, `DATE`, `GENRE`, `LYRICS`, `COMPILATION`) salvo dos que `ffmpeg` normaliza: `TRACKNUMBER` → `track` y `DISCNUMBER` → `disc` (los totales quedan como `TRACKTOTAL`/`DISCTOTAL`).
- **Plegados medidos**: en MP3 `TYER` y `TDRC` se exponen ambos bajo `date`; en MP4 `--year` y `--date` comparten `©day`; en MP3 el frame `TXXX` de `url` **no se expone** en `format_tags`. Para estos campos la capa 2 verifica el valor de unión (o la ausencia) y la distinción campo a campo la cubre la capa 1 (`show`); donde no hay clave distinguible, la celda de la tabla se marca «—» y el campo se verifica por `show`.
- **Carátula**: en los cuatro formatos la imagen aparece como stream de vídeo y se extrae con `ffmpeg -map 0:v:0 -c copy -frames:v 1 cover.jpg`. Medido que **sin `-c copy` `ffmpeg` re-codifica y los bytes cambian** (9039 frente a 9056 en OGG), así que la comparación byte a byte o por hash exige `-c copy`; también medido que la salida MUST terminar en la extensión de la imagen para que el muxer `image2` escriba un único fichero.

Por eso el spec fija las celdas medidas y exige que las restantes se midan en GREEN sobre archivo real escrito por `id3cli` —nunca supuestas—, con la medición registrada en `tasks.md`. Una tabla tolerante (grep de las claves y de sus variantes de caja, `grep -a` para la salida binaria por si `show` emitiera ANSI) protege además el e2e frente a versiones de `ffmpeg` que normalicen distinto.

### 7. Valores de prueba inequívocos y flujo por fase

Los valores de prueba se fijan en el spec (`TítuloE2E`, `ÁlbumE2E`, `Uno E2E; Dos E2E`, `7/12`, `1/2`, …) elegidos para que **ninguno sea subcadena de otro**: así `check_show_contains` no puede dar un falso positivo por coincidir con otro campo (p. ej. `album` no es subcadena de `album_sort`). El flujo por archivo es: `show` inicial (sin campos de usuario) → `edit` con todos los tags aplicables en 1-2 invocaciones → `show` campo a campo → `ffprobe` según la tabla → carátula JPG y PNG (cada una sobre un audio recién generado, porque `lofty` acumula carátulas en lugar de reemplazar: medido que tras escribir JPG y PNG sobre el mismo archivo quedan 2 imágenes) → `remove` de un campo → `remove --all` (comentario vaciado preservando el vendor en OGG/Opus) → `ffprobe` final de reproducibilidad.

### 8. Falsabilidad como parte de la verificación de calidad

Los scripts se consideran verificados solo si demuestran que **pueden fallar**: un caso negativo deliberado (corromper el valor esperado de una aserción o forzar un fallo) MUST terminar con exit 1, mensaje en español y nombre de la aserción; sin el fallo inyectado, el mismo flujo MUST terminar con exit 0. Es el mismo patrón de los casos negativos de `generate_sample_*.sh` (bytes aleatorios, primer paquete corrompido…), que abortan con su mensaje y código 1 frente al 0 del caso real.

## Risks / Trade-offs

- **(a) La normalización de claves de `ffprobe` cambia entre versiones de `ffmpeg`** (nueva normalización en MP4, y renombrados como `TRACKNUMBER` → `track`). Mitigado por tres vías: la tabla se mide sobre archivo real y se fija en el spec, el grep es tolerante a caja, y el fallo de una aserción de `ffprobe` termina en 1 con el valor obtenido visible, de modo que una regresión de la herramienta se diagnostica al instante.
- **(b) `show` puede emitir bytes multibyte (emoji) o ANSI**: las aserciones grep el **valor** y tratan la salida como binaria (`grep -a`); el spec prohíbe depender de etiquetas de estilo.
- **(c) Los scripts dependen de `ffmpeg`/`ffprobe` en el PATH del desarrollador**. Asumido: son herramientas habituales, el proyecto ya las usa en `generate_sample_*.sh`, y su ausencia falla con mensaje claro antes de tocar nada.
- **(d) El e2e no se ejecuta en CI en esta fase** (no forma parte de `cargo test`). Asumido y declarado en Non-Goals: es una verificación local complementaria; integrarla en CI sería un cambio aparte.
- **(e) La suite no crece con el e2e**: los 281 tests (163 + 118) deben seguir intactos, lo que a la vez es la definición de no-rotura del cambio.

## Migration Plan

El orden de los grupos de `tasks.md`, y por qué:

1. **Artefactos de especificación** (este change) — el contrato cerrado antes de tocar tooling.
2. **RED de los scripts**, con sus casos negativos — los esqueletos con aserciones declaradas demuestran la falsabilidad (un fallo deliberado termina en 1) antes de implementar el GREEN.
3. **GREEN de los scripts** — implementación completa de `e2e_lib.sh` y de los cuatro flujos hasta terminar 0 en verde, midiendo y fijando en `tasks.md` las celdas restantes de la tabla de claves.
4. **`.justfile` y `README.md`** — las recipes y la documentación de uso cierran el punto de entrada.
5. **Verificación final** — los 4 scripts en verde, falsabilidad demostrada, suite Rust intacta en 281 tests y fmt/clippy limpios.
6. **Cierre y archive** — commit convencional `test(e2e): …`, PR a `development`, `openspec archive e2e-local-helpers --yes`.

Cada tarea funcional del cambio MUST cumplir el ciclo RED antes que GREEN, y ninguna casilla de `tasks.md` se marca sin la verificación indicada en ella.