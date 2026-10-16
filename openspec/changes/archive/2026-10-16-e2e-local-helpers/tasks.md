# Tasks

El cambio añade tooling de verificación e2e **local** —un runner común fuenteable (`scripts/e2e/e2e_lib.sh`) y un script de flujo por extensión (`e2e_mp3.sh`, `e2e_m4a.sh`, `e2e_ogg.sh`, `e2e_opus.sh`)— que genera su propio material con `ffmpeg` bajo `/tmp/opencode/e2e/` sin versionar nada, verifica los 20 tags de `VALID_TAG_NAMES` con `id3cli show` + `ffprobe` (tabla de clave por formato) y las carátulas JPG/PNG por roundtrip de bytes, y termina en 0 solo si todas las aserciones pasan. Los grupos 2 y 3 son estrictamente RED/GREEN: el grupo 2 demuestra primero que las aserciones declaradas **pueden fallar** (casos negativos con exit 1) y el grupo 3 las pone en verde; los grupos 1, 4, 5 y 6 fijan los artefactos, el punto de entrada, la verificación de cierre y el archive. Ningún script toca `src/` ni `tests/`: la suite Rust MUST quedar intacta (281 tests: 163 unitarios + 118 de integración).

## 1. Artefactos de especificación (OpenSpec)

El contrato se cierra antes de tocar tooling: ubicación y CLI de los scripts, funciones del runner, recipes del `justfile`, restricciones de temporal y la matriz de verificación por formato con las degradaciones ya fijadas (MP4/OGG/Opus omiten `url` y `original_artist` con aviso; MP4 colisiona `©day`; MP3 sin degradaciones).

- [x] 1.1 Crear el change con `openspec new change e2e-local-helpers` (andamiaje creado el 2026-10-16; el directorio ya existe con `.openspec.yaml` de esquema `spec-driven`).
      Verificación: existe `openspec/changes/e2e-local-helpers/.openspec.yaml` con `schema: spec-driven` y `created: 2026-10-16`.
- [x] 1.2 Redactar `proposal.md` con la intención (verificación e2e del binario real), el alcance (20 tags + carátulas JPG/PNG en los 4 formatos; datos NO versionados) y el impacto (tooling en `scripts/e2e/` + `.justfile` + `README.md`; sin tocar `src/`/`tests/`; commit `test(e2e):`, PR a `development`).
      Verificación: `openspec/changes/e2e-local-helpers/proposal.md` MUST contener las secciones Why, What Changes, Capabilities (nuevas `e2e-tooling` y `e2e-verification`, sin modificadas) e Impact, con la definición de no-rotura (281 tests, 0 fallos).
- [x] 1.3 Redactar la capability `e2e-tooling` con los contratos del tooling.
      Verificación: `openspec/changes/e2e-local-helpers/specs/e2e-tooling/spec.md` MUST fijar con MUST los nombres y ubicación de los 5 scripts, el CLI (`--bin`/`--keep`, exit 0/1 con mensaje en español y aserción fallida), las funciones del runner común, las recipes del `justfile` (`e2e` en serie + una por extensión) y las restricciones (temporal `mktemp -d` bajo `/tmp/opencode/e2e/<ext>/` con `trap`, una sola construcción por ejecución, `set -euo pipefail`), cada requirement con escenarios BDD WHEN/THEN.
- [x] 1.4 Redactar la capability `e2e-verification` con la matriz de verificación.
      Verificación: `openspec/changes/e2e-local-helpers/specs/e2e-verification/spec.md` MUST fijar los valores de prueba, la cobertura de los 20 tags por formato (MP3 20/20; MP4/OGG/Opus 18 + 2 omitidos con aviso), la verificación en dos capas con la tabla de clave ffprobe por formato (celdas medidas el 2026-10-16: `format_tags` para MP3/MP4 y `stream_tags` para OGG/Opus, plegados `TYER`/`TDRC` y `©day`, `TXXX` no expuesto, `-c copy` para la carátula), roundtrip JPG/PNG, multi-artista `"; "`, track/season con y sin total, degradaciones, `remove`/`remove --all` con vendor preservado y audio reproducible, y falsabilidad.
- [x] 1.5 Redactar `design.md` y `tasks.md`.
      Verificación: `design.md` MUST documentar las decisiones con sus trade-offs (bash+ffmpeg/ffprobe sin dependencias, runner común, JPG/PNG y no WebP, temporal no versionado, `test:` y no `feat:` verificado en `release-prepare.yml:38`, tabla de claves medida) y las alternativas descartadas; `tasks.md` MUST tener los grupos 1-6 de este fichero con verificación y evidencia por tarea.
- [x] 1.6 STOP — presentar la especificación al usuario y esperar aprobación explícita antes de escribir los scripts.
      Verificación: ninguna casilla de los grupos 2-6 marcada hasta la aprobación.

> [!note] Los grupos 2 y 3 son tooling del proyecto y se desarrollan también test-first: en bash el «rojo» es la demostración de que las aserciones declaradas fallan (falsabilidad, exit 1), y el «verde» es la ejecución real terminando en 0.

## 2. RED: esqueletos de los scripts con aserciones declaradas y casos negativos

Antes de implementar nada, los esqueletos declaran el flujo completo (show inicial → edit → show → ffprobe → carátula → remove → remove --all → reproducibilidad) con las aserciones aún sin implementar del todo y con un **caso negativo deliberado** que debe terminar en 1: así se demuestra que el esqueleto detecta discrepancias antes de que exista el flujo verde.

- [x] 2.1 Crear `scripts/e2e/e2e_lib.sh` como esqueleto del runner común.
      Verificación: el esqueleto MUST declarar `set -euo pipefail` y las firmas de `e2e_setup <ext>`, la generación de audio y carátulas, `e2e_run <args...>`, `check_show_contains`, `check_ffprobe_tag`, `check_ffprobe_absent`, `check_cover_roundtrip`, `check_stderr_aviso` y el resumen PASS/FAIL con recuento; `bash -n` MUST pasar.
      Evidencia (2026-10-16): `scripts/e2e/e2e_lib.sh` creado (12082 bytes, ejecutable). Declara `set -euo pipefail`, cabecera de uso, la tabla de códec por extensión (`declare -A E2E_CODEC`), los 21 valores de prueba del spec y las funciones: `e2e_die`, `e2e_setup <ext>` (mktemp -d bajo `/tmp/opencode/e2e/<ext>/` + `trap e2e_cleanup EXIT` respetando `E2E_KEEP` + decisión de binario: `cargo build --locked` si `E2E_BIN` vacío), `e2e_cleanup` (conserva e informa con `--keep`), `e2e_gen_audio` (anullsrc 1.5 s, `-map_metadata -1`, códec por extensión), `e2e_gen_covers` (testsrc → `cover.jpg`/`cover.png`), `e2e_run <args...>` (captura stdout→`E2E_OUT`, stderr→`E2E_ERR`, código→`E2E_RC` sin abortar por errores del CLI), `check_show_contains`/`check_show_absent` (grep -aF del valor), `check_stderr_aviso`, `check_fail` (caso negativo deliberado), `ff_key_for` y `e2e_summary <formato>` (recuento PASS/FAIL, exit 0 solo sin fallos). `check_ffprobe_tag`, `check_ffprobe_absent` y `check_cover_roundtrip` declaradas con cuerpo TODO(GREEN) que registra FAIL honesto (no pasan en silencio) sin romper `bash -n`:
      ```
      $ bash -n scripts/e2e/e2e_lib.sh && echo OK
      OK
      ```
- [x] 2.2 Crear los cuatro esqueletos por extensión (`e2e_mp3.sh`, `e2e_m4a.sh`, `e2e_ogg.sh`, `e2e_opus.sh`) con el flujo y la tabla de su formato.
      Verificación: cada esqueleto MUST declarar la tabla de clave ffprobe de su formato (celdas medidas del spec 1.4) y el flujo de aserciones; `bash -n` MUST pasar en los cuatro.
      Evidencia (2026-10-16): los cuatro esqueletos creados (ejecutables) con `HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"` + `source "$HERE/e2e_lib.sh"`, parseo `--bin`/`--keep`, `declare -A FF_TAGS` con las celdas medidas por formato (MP3: title, artist, album, year→date plegado, genre, track, season→disc, date, copyright, composer, subtitle→TIT3, original_artist→TOPE, album_artist, lyrics→lyrics-spa, compilation, sorts→*-sort, url→«—» TXXX no expuesto; MP4: title, artist, album, year→date ©day plegado, genre, track, season→disc, date, url/original_artist→«—» omitidos; OGG/Opus: TITLE, ARTIST, ALBUM, YEAR, DATE, GENRE, track/disc normalizados + TRACKTOTAL/DISCTOTAL, LYRICS, COMPILATION), el flujo completo declarado en comentarios (show inicial → edit 20/18 tags → show campo a campo → ffprobe → carátula JPG/PNG → remove → remove --all → reproducibilidad) con las degradaciones de cada formato, y la parte ejecutable (setup + generación + show inicial con `check_show_absent`). `bash -n` pasa en los cuatro:
      ```
      $ for f in scripts/e2e/e2e_mp3.sh scripts/e2e/e2e_m4a.sh scripts/e2e/e2e_ogg.sh scripts/e2e/e2e_opus.sh; do bash -n "$f" && echo "OK  $f"; done
      OK  scripts/e2e/e2e_mp3.sh
      OK  scripts/e2e/e2e_m4a.sh
      OK  scripts/e2e/e2e_ogg.sh
      OK  scripts/e2e/e2e_opus.sh
      ```
      Ejecución de andamiaje del esqueleto MP3 (sin `--bin`: `cargo build --locked` una vez y `target/debug/id3cli`; el `trap` limpia el temporal y el directorio base queda vacío tras la ejecución):
      ```
      $ ./scripts/e2e/e2e_mp3.sh; echo "EXIT=$?"
      → Construyendo id3cli (cargo build --locked)…
          Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
      → e2e de mp3 con el binario /data/rust/id3cli/target/debug/id3cli (temporal: /tmp/opencode/e2e/mp3/e2e.JaS3qx)
      → Audio generado: /tmp/opencode/e2e/mp3/e2e.JaS3qx/audio.mp3
      → Carátulas generadas: /tmp/opencode/e2e/mp3/e2e.JaS3qx/cover.jpg y /tmp/opencode/e2e/mp3/e2e.JaS3qx/cover.png
      PASS: check_show_absent: el valor «TítuloE2E» NO aparece en show
      PASS: check_show_absent: el valor «Uno E2E; Dos E2E» NO aparece en show
      PASS: check_show_absent: el valor «https://e2e.example.org/id3cli» NO aparece en show
      PASS/FAIL: mp3: 3 aserciones, 0 fallidas
      EXIT=0
      ```
- [x] 2.3 Demostrar el rojo en bash con los casos negativos (falsabilidad).
      Verificación: un fallo deliberado (corromper el valor esperado de una aserción o forzar un fallo) MUST terminar con exit 1, mensaje en español y nombre de la aserción fallida; sin el fallo inyectado, el mismo flujo MUST terminar con exit 0.
      Evidencia (2026-10-16): script temporal `scripts/e2e/red_demo.sh` (no queda en el repo; se elimina al cerrar la fase) que con `RED_DEMO_INJECT=1` espera deliberadamente el valor «TítuloQueNoExiste» en `show` (corrupción del valor esperado, escenario «Corrupción de un valor esperado») y con `RED_DEMO_INJECT=0` verifica la ausencia real. Mismo esqueleto, mismas funciones del runner:
      ```
      $ ./scripts/e2e/red_demo.sh --bin target/debug/id3cli; echo "EXIT_NEGATIVO=$?"
      → red_demo: modo inyectado (esperado: exit 1) (RED_DEMO_INJECT=1)
      → e2e de mp3 con el binario target/debug/id3cli (temporal: /tmp/opencode/e2e/mp3/e2e.xwTDnj)
      → Audio generado: /tmp/opencode/e2e/mp3/e2e.xwTDnj/audio.mp3
      FAIL: aserción 'check_show_contains': el valor esperado «TítuloQueNoExiste» NO aparece en show (grep -a sobre /tmp/opencode/e2e/mp3/e2e.xwTDnj/stdout.txt)
      PASS/FAIL: red-demo (mp3): 1 aserciones, 1 fallidas
      EXIT_NEGATIVO=1

      $ RED_DEMO_INJECT=0 ./scripts/e2e/red_demo.sh --bin target/debug/id3cli; echo "EXIT_CORREGIDO=$?"
      → red_demo: modo corregido (esperado: exit 0) (RED_DEMO_INJECT=0)
      → e2e de mp3 con el binario target/debug/id3cli (temporal: /tmp/opencode/e2e/mp3/e2e.ovLmLg)
      → Audio generado: /tmp/opencode/e2e/mp3/e2e.ovLmLg/audio.mp3
      PASS: check_show_absent: el valor «TítuloE2E» NO aparece en show
      PASS/FAIL: red-demo (mp3): 1 aserciones, 0 fallidas
      EXIT_CORREGIDO=0
      ```
      El caso inyectado termina en **1** con mensaje en español y nombre de la aserción fallida (`check_show_contains`); el mismo flujo corregido termina en **0**. El 0 real del flujo completo (edit → show → ffprobe → carátula → remove → remove --all) llega en GREEN (grupo 3): en RED el esqueleto solo ejecuta el andamiaje (setup, generación y estado inicial sin campos), que ya termina 0 como se ve en 2.2.

## 3. GREEN: implementación completa

Implementación del runner común y de los cuatro flujos completos hasta que cada script termine 0 en verde sobre su formato, con el binario construido una vez por ejecución del runner. Aquí se miden y fijan en este fichero las celdas restantes de la tabla de claves ffprobe (las que el spec 1.4 dejó «por medir»), con el comando y la salida de la medición.

- [x] 3.1 Implementar `e2e_lib.sh` completo.
      Verificación: `e2e_setup` crea el temporal con `mktemp -d` bajo `/tmp/opencode/e2e/<ext>/`, registra `trap` de limpieza (respetando `--keep`) y decide el binario (por defecto `cargo build` una vez y `target/debug/id3cli`; `--bin` lo sustituye); la generación produce audio silencioso (`anullsrc` ~1-2 s, `-map_metadata -1`; mp3 → `libmp3lame`, m4a → `aac`, ogg → `libvorbis`, opus → `libopus`) y carátulas `cover.jpg`/`cover.png` (`-f lavfi -i testsrc`); las aserciones grep el valor (con `grep -a`), consultan `ffprobe` (¡en OGG/Opus por `stream_tags`, que es donde vive la metadata!) y comparan la carátula extraída con `-map 0:v:0 -c copy -frames:v 1` byte a byte o por hash; el resumen imprime aserciones ejecutadas y fallidas; `bash -n` pasa.
      Evidencia (2026-10-16): funciones verdes implementadas en `scripts/e2e/e2e_lib.sh`: `ffprobe_entries_for` (selector `format_tags` para mp3/m4a y `stream_tags` para ogg/opus), `ffprobe_tag <ext> <clave>` (sintaxis `-show_entries "<formato>=<clave>"`; la sintaxis con `:` da «No match for section» en ffmpeg 9.0.2 y se descartó al medir), `check_ffprobe_tag`/`check_ffprobe_absent`, `check_ffprobe_raw_contains` (par clave+valor crudo para discriminaciones tipo `YEAR=2024`), `check_ffprobe_no_user_tags` (tras `remove --all` en OGG/Opus), `check_cover_roundtrip` (`-c copy` + `cmp -s`; sin `-c copy` ffmpeg re-codifica y los bytes difieren —medido: 9039 frente a 9056—), `grep_raw_absent`/`grep_raw_contains` (bytes crudos del comentario OGG/Opus, incluido el vendor), `check_duration_approx` (reproducibilidad), `check_rc`, `check_no_id3v2_header` y `e2e_gen_audio [nombre]` (nombre parametrizable para «audio recién generado» por carátula y por la variante sin total). Las funciones de aserción registran PASS/FAIL para el recuento de `e2e_summary` y nunca abortan la ejecución; el comportamiento queda comprobado por los cuatro scripts del grupo (3.2-3.4), todos verdes. `bash -n` pasa en los cinco:
      ```
      $ for f in scripts/e2e/*.sh; do bash -n "$f" && echo "OK  $f"; done
      OK  scripts/e2e/e2e_lib.sh
      OK  scripts/e2e/e2e_m4a.sh
      OK  scripts/e2e/e2e_mp3.sh
      OK  scripts/e2e/e2e_ogg.sh
      OK  scripts/e2e/e2e_opus.sh
      ```
- [x] 3.2 Implementar `e2e_mp3.sh` y ponerlo en verde.
      Verificación: flujo completo sobre MP3 —show inicial sin campos de usuario → `edit` con los 20 tags en 1-2 invocaciones → `show` campo a campo (los 20, incluidos `url` y `original_artist`, que aquí NO se omiten) → `ffprobe` con la tabla de su formato → `remove` de un campo → `remove --all` (el bloque ID3v2 MUST desaparecer del archivo) → `ffprobe` final de reproducibilidad con duración conservada; además la variante `track`/`season` sin total sobre audio recién generado y la carátula JPG y PNG por roundtrip de bytes; el script MUST terminar 0. `bash -n` pasa.
      Evidencia (2026-10-16): ejecución con el binario ya construido:
      ```
      $ ./scripts/e2e/e2e_mp3.sh --bin target/debug/id3cli; echo "EXIT=$?"
      → e2e de mp3 con el binario target/debug/id3cli (temporal: /tmp/opencode/e2e/mp3/e2e.mK8f65)
      (… 63 aserciones PASS …)
      PASS: check_ffprobe_tag: «disc»=1 en format_tags
      → Audio generado: …/cover_jpg.mp3
      PASS: check_cover_roundtrip: …/cover.jpg idéntica a la extracción (9056 bytes, byte a byte)
      PASS: check_cover_roundtrip: …/cover.png idéntica a la extracción (1765 bytes, byte a byte)
      PASS/FAIL: mp3: 65 aserciones, 0 fallidas
      EXIT=0
      ```
      Evolución del diseño: la primera versión probó la variante sin total **sobre el mismo archivo** con `--track 7` tras `7/12`, y el discriminador `check_show_absent 7/12` falló porque el CLI **conserva el total previo del frame `TRCK` si solo se indica el número** (comportamiento documentado en `apply_metadata`: «el total solo se sobrescribe si `--track` lo indica explícitamente»; reproducido manualmente y confirmado por ffprobe `track=7/12`). Como el spec no exige sustituir un total sino «escribir el número sin dejar total», la variante se ejerce sobre un **audio recién generado** (escenario «Número sin total» del spec), y la comprobación queda discriminada por la ausencia de «7/12»/«1/2» —también se corrigió un falso positivo propio del runner: `check_ffprobe_tag track 7` es subcadena de `7/12` y pasaba en falso sobre el archivo con total, por eso el discriminador es la ausencia.
- [x] 3.3 Implementar `e2e_m4a.sh` y ponerlo en verde.
      Verificación: flujo completo sobre M4A con los 18 tags escritos y releídos + `url`/`original_artist` **omitidos con aviso** (stderr contiene el aviso), más el par `--year`/`--date` con victoria de `--date` y aviso de absorción; ffprobe MUST no mostrar claves de URL ni de artista original; carátula JPG y PNG; `remove`/`remove --all` con audio reproducible; el script MUST terminar 0.
      Evidencia (2026-10-16): ejecución con el binario ya construido:
      ```
      $ ./scripts/e2e/e2e_m4a.sh --bin target/debug/id3cli; echo "EXIT=$?"
      PASS: check_stderr_aviso: aviso «El campo '--url' no tiene átomo equivalente en MP4; se omite.» presente en stderr
      PASS: check_stderr_aviso: aviso «El campo '--original-artist' no tiene átomo equivalente en MP4; se omite.» presente en stderr
      PASS: check_stderr_aviso: aviso «--year 2023 queda absorbido por --date» presente en stderr
      PASS: check_ffprobe_absent: la clave «url» NO está expuesta en format_tags
      PASS: check_ffprobe_absent: la clave «original_artist» NO está expuesta en format_tags
      (…)
      PASS/FAIL: m4a: 68 aserciones, 0 fallidas
      EXIT=0
      ```
- [x] 3.4 Implementar `e2e_ogg.sh` y `e2e_opus.sh` y ponerlos en verde.
      Verificación: flujo completo sobre OGG y Opus con los 18 tags escritos y releídos + `url`/`original_artist` **omitidos con aviso**; ausencia comprobada con ffprobe (`stream_tags`) y en los **bytes crudos** del comentario (MUST NOT aparecer `WEBSITE=` ni `ORIGINALARTIST=`); carátula JPG y PNG; `remove` de un campo; `remove --all` MUST vaciar el comentario **preservando el vendor** y el audio MUST seguir siendo reproducible con duración conservada; cada script MUST terminar 0.
      Evidencia (2026-10-16): ejecución con el binario ya construido:
      ```
      $ ./scripts/e2e/e2e_ogg.sh --bin target/debug/id3cli; echo "EXIT=$?"
      PASS: check_stderr_aviso: aviso «El campo '--url' no tiene clave equivalente en OGG; se omite.» presente en stderr
      PASS: check_stderr_aviso: aviso «El campo '--original-artist' no tiene clave equivalente en OGG; se omite.» presente en stderr
      PASS: grep_raw_absent: el patrón «WEBSITE=» NO aparece en crudo en …/audio.ogg
      PASS: grep_raw_absent: el patrón «ORIGINALARTIST=» NO aparece en crudo en …/audio.ogg
      PASS: check_ffprobe_no_user_tags: stream_tags sin claves tras remove --all
      PASS: grep_raw_contains: el patrón «Lavf63.1.102» aparece en crudo en …/audio.ogg
      PASS/FAIL: ogg: 76 aserciones, 0 fallidas
      EXIT=0

      $ ./scripts/e2e/e2e_opus.sh --bin target/debug/id3cli; echo "EXIT=$?"
      PASS: grep_raw_contains: el patrón «Lavf63.1.102» aparece en crudo en …/audio.opus
      PASS/FAIL: opus: 76 aserciones, 0 fallidas
      EXIT=0
      ```
- [x] 3.5 Registrar en este fichero la medición de las celdas restantes de la tabla de claves.
      Verificación: para cada celda aún no fijada en el spec, el comando `ffprobe` (o `grep` en crudo) ejecutado sobre un archivo escrito por `id3cli` con ese campo y su salida literal, con la celda resultante anotada; si un campo no tiene clave distinguible, se anota «—» y su cobertura queda por `show`.
      Evidencia (2026-10-16): bloque «Medición de claves ffprobe (2026-10-16)» al final de esta tarea, con comando y salida por formato. Las celdas que contradecían al spec (la afirmación general «OGG/Opus preservan la caja» frente a `ALBUMARTIST`→`album_artist`; la caja de `SUBTITLE` y las claves `sort_name`/`sort_artist`/`sort_album` en MP4; el ganador del plegado `date` en MP3) se corrigieron en `specs/e2e-verification/spec.md` (nota «Medición GREEN 2026-10-16»). Los hallazgos no bloqueantes para el e2e (MP4 `edit` tras `remove --all` invisible a ffmpeg; `edit --cover` que añade una segunda carátula en vez de sustituir) se anotan al final para que el director los revise; no se tocó `src/`.

> [!warning] La tabla de claves ffprobe del runner NO puede contener claves supuestas: cada celda se mide sobre un archivo real escrito por `id3cli` y la medición queda registrada en la tarea 3.5. Un `ffprobe` que no expone la clave esperada es un hallazgo que se documenta, no un fallo que se silencia.

### Medición de claves ffprobe (2026-10-16)

Material: cuatro archivos recién generados con `ffmpeg` 9.0.2 (`anullsrc` 1.5 s, `-map_metadata -1`; `libmp3lame`/`aac`/`libvorbis`/`libopus`) y los 20 valores fijados del spec, escritos con `id3cli edit` (una invocación por formato; en MP4 con `--year 2023` para medir la colisión `©day`). Comando por formato: `ffprobe -hide_banner -v error -show_entries format_tags|stream_tags -of default=noprint_wrappers=1 <archivo>`.

**MP3** (celdas en `format_tags`; claves minúsculas; valor literal de la salida):
```
TAG:title=TítuloE2E            → title
TAG:artist=Uno E2E; Dos E2E    → artist (multi-artista unido con "; ")
TAG:album=ÁlbumE2E             → album
TAG:title-sort=TitleSortE2E    → title-sort
TAG:genre=RockE2E              → genre
TAG:track=7/12                 → track (par n/total)
TAG:disc=1/2                   → disc
TAG:artist-sort=ArtistSortE2E  → artist-sort
TAG:copyright=©2024E2E         → copyright
TAG:composer=CompositorE2E     → composer
TAG:TIT3=SubtítuloE2E          → TIT3 (subtitle)
TAG:TOPE=OriginalArtistE2E     → TOPE (original_artist)
TAG:album_artist=AlbumArtistE2E→ album_artist
TAG:lyrics-spa=LetraE2E        → lyrics-spa (USLT, valor multilínea: la segunda
Segunda línea E2E                línea cuelga de la misma entrada)
TAG:compilation=1              → compilation («1»)
TAG:album-sort=AlbumSortE2E    → album-sort
TAG:date=2024                  → date
```
- `url` (frame `TXXX`/`WOAR`) NO aparece en `format_tags` → celda «—», cobertura por `show` (ya fijado en RED).
- **Plegado `date`** (celda nueva, hallazgo): con `--year 2024` Y `--date 2024-05-01` escritos (frames `TYER` y `TDRC`), ffprobe expone solo `date=2024` — el **año (TYER) gana** al último `TDRC`. Aislados: `--date` solo → `date=2024-05-01`; `--year` solo → `date=2024`. Consecuencia: en el flujo verde el par `--year`/`--date` se verifica por `show` (Año + Fecha) y `check_ffprobe_tag date 2024` fija el valor ganador del plegado.

**MP4/M4A** (celdas en `format_tags`; claves minúsculas salvo las que ffmpeg preserva):
```
TAG:title=TítuloE2E                          → title
TAG:artist=Uno E2E; Dos E2E                  → artist
TAG:album=ÁlbumE2E                           → album
TAG:album_artist=AlbumArtistE2E              → album_artist
TAG:genre=RockE2E                            → genre
TAG:copyright=©2024E2E                       → copyright
TAG:composer=CompositorE2E                   → composer
TAG:SUBTITLE=SubtítuloE2E                    → SUBTITLE (freeform, caja que preserva ffmpeg)
TAG:lyrics=LetraE2E                          → lyrics (©lyr, multilínea)
Segunda línea E2E
TAG:compilation=1                            → compilation («1»)
TAG:sort_name=TitleSortE2E                   → sort_name (sonm; NO «title-sort»)
TAG:sort_artist=ArtistSortE2E                → sort_artist (soar)
TAG:sort_album=AlbumSortE2E                  → sort_album (soal)
TAG:date=2024-05-01                          → date (©day; gana la fecha, aviso de absorción)
TAG:track=7/12                               → track
TAG:disc=1/2                                 → disc
```
- `url` y `original_artist` NO aparecen (omitidos con aviso; nunca átomo freeform) → «—».
- **Colisión `©day`** (medida): con `--year 2023 --date 2024-05-01`, stderr = `⚠️  Advertencia: --year 2023 queda absorbido por --date "2024-05-01" en el átomo ©day (el año es el prefijo de la fecha); se escribe la fecha.` y `date=2024-05-01` en ffprobe; `show` no imprime «Año» (solo se confirma cuando `--date` no está).

**OGG y Opus** (celdas en `stream_tags`; `format_tags` vacío; caja Vorbis preservada **salvo `ALBUMARTIST`**) — salida idéntica para ambos códecs (mismo backend de comentarios):
```
TAG:TITLE=TítuloE2E                          → TITLE
TAG:ARTIST=Uno E2E; Dos E2E                  → ARTIST
TAG:ALBUM=ÁlbumE2E                           → ALBUM
TAG:album_artist=AlbumArtistE2E              → album_artist (¡minúsculas! ffmpeg normaliza ALBUMARTIST → hallazgo)
TAG:YEAR=2024                                → YEAR
TAG:DATE=2024-05-01                          → DATE (clave separada de YEAR)
TAG:GENRE=RockE2E                            → GENRE
TAG:track=7                                  → track (TRACKNUMBER normalizado; el total va aparte)
TAG:TRACKTOTAL=12                            → TRACKTOTAL
TAG:disc=1                                   → disc (DISCNUMBER normalizado)
TAG:DISCTOTAL=2                              → DISCTOTAL
TAG:COPYRIGHT=©2024E2E                       → COPYRIGHT
TAG:COMPOSER=CompositorE2E                   → COMPOSER
TAG:SUBTITLE=SubtítuloE2E                    → SUBTITLE
TAG:LYRICS=LetraE2E                          → LYRICS (multilínea)
Segunda línea E2E
TAG:COMPILATION=1                            → COMPILATION («1»)
TAG:TITLESORT=TitleSortE2E                   → TITLESORT
TAG:ARTISTSORT=ArtistSortE2E                 → ARTISTSORT
TAG:ALBUMSORT=AlbumSortE2E                   → ALBUMSORT
```
- `url`/`original_artist` NO aparecen (omitidos con aviso) → «—»; bytes crudos del comentario sin `WEBSITE=` ni `ORIGINALARTIST=` (`grep -ac` → 0 y 0 en ambos archivos).
- **Vendor** (medido): `ffmpeg` escribe el vendor `Lavf63.1.102`; tras `remove --all` el comentario queda sin claves (`ffprobe stream_tags` vacío) pero `Lavf63.1.102` sigue en los bytes crudos y la duración se conserva (`1.500000` s).

**Correcciones al spec** (registradas en `specs/e2e-verification/spec.md`, nota «Medición GREEN 2026-10-16»): la afirmación general «OGG/Opus preservan la caja de las claves Vorbis» se matiza (ALBUMARTIST → `album_artist`); se fijan `SUBTITLE` y `sort_name`/`sort_artist`/`sort_album` de MP4 y el ganador del plegado `date` de MP3. Celdas sin clave distinguible («—»): `url` (TXXX/WOAR no expuesto; MP3 lo cubre `show`), `cover` (roundtrip de bytes) y, en MP4/OGG/Opus, `url`/`original_artist` (omitidos, cubiertos por `show` + ausencia en ffprobe + bytes crudos en OGG/Opus).

**Hallazgos para el director** (no bloqueantes; se reportan sin tocar `src/`):
1. **MP4: `edit` tras `remove --all` deja la metadata invisible a ffmpeg.** Flujo: `id3cli edit` completo → `remove --all` → `edit` de nuevo sobre el mismo `.m4a`. `id3cli show` relee los 18 campos, pero `ffprobe -show_entries format_tags` solo muestra `major_brand`/`minor_version`/`compatible_brands`. El trace de ffmpeg muestra que el átomo `meta` (dentro de `udta`) queda **sin el hijo `hdlr`** que el demuxer exige para parsear `ilst`; en un archivo editado sin `remove --all` de por medio el `meta` sí contiene `hdlr` y ffmpeg lee todo. El e2e no lo ejercita (por contrato, cada carátula y cada variante se escriben sobre audio recién generado y tras `remove --all` no se vuelve a editar), pero es un comportamiento del binario real que un reproductor externo (VLC, QuickTime…) percibiría como «etiquetas perdidas».
2. **MP3/MP4/OGG: `edit --cover` sobre un archivo que ya tiene carátula AÑADE una segunda imagen en vez de sustituirla.** Medido con `insert_picture` de `lofty` (MP3): tras `edit --cover cover.jpg` y luego `edit --cover cover.png` sobre el mismo archivo, `show` reporta `Carátulas: 2 imagen(es)` (CoverFront JPG + CoverFront PNG) y `ffmpeg -map 0:v:0` extrae la primera. Sin relevancia para el e2e (por contrato cada carátula se escribe sobre audio recién generado), pero probablemente el director quiera una issue.
3. **`--track N` no sustituye un total previo**: al indicar solo el número (sin total) sobre un archivo con `7/12`, el total se conserva (comportamiento intencional y documentado en `apply_metadata`, no un bug); el e2e verifica la forma sin total sobre audio recién generado.

## 4. Punto de entrada: `.justfile` y `README.md`

- [x] 4.1 Añadir las recipes `e2e`, `e2e-mp3`, `e2e-m4a`, `e2e-ogg` y `e2e-opus` al `.justfile`.
      Verificación: `just e2e` MUST ejecutar los cuatro scripts en serie y detenerse propagando el exit code del primero que falle; las recipes por extensión MUST aceptar el paso de `--bin` y `--keep`; `just --list` MUST mostrar las cinco recipes.
      Evidencia (2026-10-16): recipes añadidas al final del `.justfile` con la sintaxis del fichero. **Hallazgo de versión, verificado empíricamente**: en `just` 1.58.0 el shell de las recipes **ya no recibe los argumentos vía `"$@"`** (queda vacío; probado con un justfile temporal de juguete); los argumentos solo llegan con un parámetro variádico (`*args`) y su sustitución `{{args}}`. Por eso las recipes se declaran `e2e *args:` / `e2e-mp3 *args:` … y reenvían `{{args}}`, lo que permite `just e2e --bin …` **sin** separador `--` (exactamente el escenario «Just por extensión con argumentos» del spec). Además se midió que `set -e` en la **primera línea** y el bucle en la **segunda** NO funciona (just ejecuta cada línea en un shell distinto y el loop continuaba tras un fallo, ejecutando `opus`): la recipe `e2e` lleva `set -e; for … done` en **una sola línea**, y la propagación del exit code del primer fallo se verificó con scripts falsos (ogg falla con 3 → `EXIT=3` y `opus` no se ejecuta). `just --list` muestra las cinco recipes junto a las existentes:
      ```
      $ just --list
      Available recipes:
          e2e *args      # del primero que falle. Acepta --bin RUTA y --keep.
          e2e-m4a *args
          e2e-mp3 *args  # e2e de un formato concreto (acepta --bin RUTA y --keep)
          e2e-ogg *args
          e2e-opus *args
          fmt
          fmt-fix
          install
          lint
          list
          upgrade
      ```
      `just e2e` ejecuta los cuatro en serie y termina 0 (cada script imprime su resumen PASS/FAIL). **Fix (2026-10-16, MUST «una sola construcción» del spec Req 5)**: la recipe `e2e` pre-construye explícitamente UNA vez — `cargo build --locked` como línea propia de la recipe (just ejecuta cada línea en un shell distinto y aborta con el exit code de la que falle por defecto; verificado en esta ejecución) — y pasa `--bin target/debug/id3cli` a los cuatro scripts, de modo que `e2e_setup` recibe `E2E_BIN` no vacío y no ejecuta `cargo build`. La salida completa muestra **una sola línea de build** (la invocación `cargo build --locked` de la recipe + su `Finished`, y **0 líneas «Construyendo»** dentro de los scripts; `grep -c 'cargo build --locked'` → 1 y `grep -c 'Construyendo id3cli'` → 0 sobre la salida completa). `{{args}}` (p. ej. `--keep`) se reenvía después del `--bin` fijo sin duplicarlo; si el usuario pasara `--bin OTRO` por args quedaría duplicado, pero es el caso documentado: `just e2e` construye el binario por defecto y los flags por extensión son para las recipes individuales:
      ```
      $ just e2e 2>&1 | grep -E "cargo build --locked|Finished|→ e2e de|PASS/FAIL"; echo "EXIT=${PIPESTATUS[0]}"
      cargo build --locked
          Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
      → e2e de mp3 con el binario target/debug/id3cli (temporal: /tmp/opencode/e2e/mp3/e2e.oM9rea)
      PASS/FAIL: mp3: 65 aserciones, 0 fallidas
      → e2e de m4a con el binario target/debug/id3cli (temporal: /tmp/opencode/e2e/m4a/e2e.SiNaGC)
      PASS/FAIL: m4a: 68 aserciones, 0 fallidas
      → e2e de ogg con el binario target/debug/id3cli (temporal: /tmp/opencode/e2e/ogg/e2e.95SFxm)
      PASS/FAIL: ogg: 76 aserciones, 0 fallidas
      → e2e de opus con el binario target/debug/id3cli (temporal: /tmp/opencode/e2e/opus/e2e.pHtBLP)
      PASS/FAIL: opus: 76 aserciones, 0 fallidas
      EXIT=0
      ```
      Y el escenario del spec con argumentos sin `--`, verificado con el binario ya construido y `--keep`:
      ```
      $ just e2e-opus --bin target/debug/id3cli --keep 2>&1 | grep -E "→ e2e de|Temporal conservado|PASS/FAIL"; echo "EXIT=${PIPESTATUS[0]}"
      → e2e de opus con el binario target/debug/id3cli (temporal: /tmp/opencode/e2e/opus/e2e.JY3TMA)
      PASS/FAIL: opus: 76 aserciones, 0 fallidas
      → Temporal conservado (--keep): /tmp/opencode/e2e/opus/e2e.JY3TMA
      EXIT=0
      ```
- [x] 4.2 Documentar la sección «Verificación e2e local» en `README.md`.
      Verificación: el README MUST explicar qué verifica el e2e (20 tags + carátulas JPG/PNG en los 4 formatos, dos capas), cómo ejecutarlo (`just e2e`, por extensión), los flags `--bin`/`--keep`, el requisito de tener `ffmpeg`/`ffprobe` en el PATH y que todo el material se genera bajo `/tmp/opencode/e2e/` sin versionarse.
      Evidencia (2026-10-16): sección `## Verificación e2e local` insertada en el README entre `## Tests` y `## Licencia` (tras el separador `---`), y entrada `scripts/` añadida al árbol de «Estructura del proyecto». Documenta las dos capas (`id3cli show` + `ffprobe` como lector independiente con `format_tags`/`stream_tags` y bytes crudos en OGG/Opus, más el roundtrip de bytes de la carátula con `ffmpeg -map 0:v:0 -c copy`), la cobertura por formato (MP3 20/20; MP4/OGG/Opus 18 + `url`/`original_artist` omitidos con aviso), la ejecución (`just e2e`, `just e2e-<ext>`, scripts directos), la tabla de flags (`--bin RUTA` con `cargo build --locked` una vez como default; `--keep`), la localidad (`/tmp/opencode/e2e/<ext>/`, `mktemp -d` + `trap`, nada versionado), los requisitos (`ffmpeg`/`ffprobe` en el PATH y `cargo`), los códigos de salida (0/1 con el nombre de la aserción fallida) y el propósito (regresión de lectura/escritura sin sobrecargar el repo con binarios). Incluye también una nota sobre `*args`/`{{args}}` en `just` >= 1.58 (por qué no se usa `"$@"`). **Fix 2026-10-16 (MUST «una sola construcción» del spec Req 5)**: la tabla de flags y los ejemplos se ajustaron a la nueva recipe `e2e` — la fila `--bin RUTA` matiza «`just e2e` pre-construye una vez y pasa `--bin target/debug/id3cli` a los scripts; los scripts invocados directamente construyen si no reciben `--bin`», y el ejemplo `just e2e --bin target/debug/id3cli   # sin reconstruir` se sustituyó por `just e2e` / `just e2e --keep` (con la pre-construcción de la recipe, el `--bin` del usuario quedaría duplicado y el comentario «sin reconstruir» era falso: la recipe siempre invoca `cargo build --locked`). Nombres verificados contra el código real: `VALID_TAG_NAMES` (src/lib.rs:529, 20 tags) y las funciones del runner (`e2e_setup`, `--bin`/`--keep`, `trap`, `mktemp -d` bajo `/tmp/opencode/e2e/<ext>/`, `e2e_summary`). Texto inicial de la sección:
      ```
      ## Verificación e2e local

      Además de la suite de Rust, el repositorio incluye un tooling de **verificación e2e local** (`scripts/e2e/`) que ejercita el binario real de extremo a extremo: escribe y relee los **20 tags** de `VALID_TAG_NAMES` y las carátulas **JPG y PNG** sobre los **cuatro formatos soportados** (MP3, MP4/M4A, OGG y Opus). Es una verificación de regresión de lectura/escritura del flujo completo `edit → show → remove`, contrastada con un **lector independiente**; está pensada para no sobrecargar el repositorio, así que **todo el material se genera al vuelo y nada se versiona**.
      ```

## 5. Verificación final

Cierre con la ejecución real de los cuatro scripts, la falsabilidad demostrada y la suite Rust intacta, con la evidencia fechada dentro de este change.

- [x] 5.1 Los cuatro scripts terminan 0 en verde.
      Verificación: `just e2e` (o `e2e_mp3.sh`/`e2e_m4a.sh`/`e2e_ogg.sh`/`e2e_opus.sh` individualmente) MUST terminar 0 para los cuatro formatos, cada uno verificando TODOS los tags aplicables (20 en MP3; 18 + 2 omitidos con aviso en MP4/OGG/Opus) y la carátula JPG y PNG por roundtrip de bytes.
      Evidencia (2026-10-16, verificada por el director y re-ejecutada en la auditoría del grupo 6): `just e2e` termina **EXIT=0** con 4 scripts verdes — `PASS/FAIL: mp3: 65 aserciones, 0 fallidas`, `m4a: 68 aserciones, 0 fallidas`, `ogg: 76 aserciones, 0 fallidas`, `opus: 76 aserciones, 0 fallidas` (salida literal de la re-ejecución de la auditoría: `→ e2e de …` por formato, un único `PASS/FAIL` por script con esos recuentos y `EXIT=0` final).
- [x] 5.2 Falsabilidad demostrada.
      Verificación: un caso negativo deliberado MUST terminar 1 con mensaje en español y nombre de la aserción fallida; el flujo corregido MUST volver a terminar 0.
      Evidencia (2026-10-16, verificada por el director y registrada en 2.3): `red_demo.sh` temporal con `RED_DEMO_INJECT=1` (valor esperado corrompido a «TítuloQueNoExiste») termina `EXIT_NEGATIVO=1` con `FAIL: aserción 'check_show_contains': el valor esperado «TítuloQueNoExiste» NO aparece en show …` (mensaje en español y nombre de la aserción); el mismo flujo con `RED_DEMO_INJECT=0` termina `EXIT_CORREGIDO=0` con `1 aserciones, 0 fallidas`.
- [x] 5.3 Suite Rust intacta.
      Verificación: `cargo test --no-fail-fast` MUST reportar **281 tests (163 unitarios + 118 de integración), 0 fallos**, sin cambios en `src/` ni `tests/`; `cargo fmt --check` y `cargo clippy --all-targets --all-features -- -D warnings` MUST quedar limpios (los scripts no afectan a cargo; solo se confirma que nada se rompió).
      Evidencia (2026-10-16, verificada por el director y re-ejecutada en la auditoría del grupo 6): `cargo test --no-fail-fast` → `test result: ok. 163 passed` (unit, lib) + `test result: ok. 118 passed` (integración) = **281 tests, 0 failed**; `cargo fmt --check` → sin diferencias; `cargo clippy --all-targets --all-features -- -D warnings` → `Finished` sin warnings.
- [x] 5.4 `bash -n` en los cinco scripts y, si está disponible, `shellcheck` (si no, documentar la omisión).
      Verificación: `bash -n scripts/e2e/*.sh` MUST pasar en los cinco; si `shellcheck` está instalado, MUST ejecutarse sin errores bloqueantes, y si no, la omisión queda anotada aquí.
      Evidencia (2026-10-16, verificada por el director y re-ejecutada en la auditoría del grupo 6): `bash -n` OK en los 5 scripts (`OK scripts/e2e/e2e_lib.sh`, `OK …e2e_m4a.sh`, `OK …e2e_mp3.sh`, `OK …e2e_ogg.sh`, `OK …e2e_opus.sh`). **`shellcheck` NO está instalado** en el entorno (`command -v shellcheck` vacío) — omisión documentada; los scripts llevan directivas `# shellcheck source=./e2e_lib.sh` para cuando se ejecute en CI.

## 6. Cierre y archive

- [x] 6.1 Revisión final del spec contra la implementación y cierre fechado.
      Verificación: cada escenario de `e2e-tooling` y `e2e-verification` MUST quedar cubierto por una aserción de un script (tabla escenario → aserción en este fichero); la evidencia fechada (2026-10-16) de los grupos 2-5 queda registrada; todas las casillas de este fichero marcadas.
      Cierre (2026-10-16, auditoría del grupo 6): re-verificado `bash -n` en los 5 scripts (OK), `just e2e` termina 0 (mp3: 65, m4a: 68, ogg: 76, opus: 76 aserciones, 0 fallidas), sin `TODO(GREEN)`/`red_demo`/`FIXME` residuales en `scripts/e2e/`, temporales bajo `/tmp/opencode/e2e/<ext>/` vacíos tras la ejecución (el `trap` limpia), `git status` solo con `.justfile`, `README.md`, `scripts/e2e/*.sh` (5), `PLAN.md` y `openspec/changes/e2e-local-helpers/`, y `openspec status --change e2e-local-helpers` con 4/4 artefactos completos. Tabla escenario → aserción:

      | Spec | Requirement / Escenario | Cobertura (script:aserciones, nombres reales) |
      |---|---|---|
      | e2e-tooling | Req 1 · scripts versionados en `scripts/e2e/`, datos NO versionados | Los 5 ficheros existen y están bajo control de versiones (`git status`: `AM scripts/e2e/*.sh`); todo el material se genera en `mktemp -d` bajo `/tmp/opencode/e2e/<ext>/` (e2e_lib.sh:111-114) y `git status` no muestra audio ni imágenes (auditoría 2026-10-16) |
      | e2e-tooling | Req 1 · esc. «La estructura de scripts existe y está versionada» | `ls scripts/e2e/` → `e2e_lib.sh` + `e2e_mp3.sh`/`e2e_m4a.sh`/`e2e_ogg.sh`/`e2e_opus.sh` (auditoría 2026-10-16) |
      | e2e-tooling | Req 1 · esc. «Los archivos generados no entran en el repositorio» | `git status --short` sin binarios/imágenes (auditoría 2026-10-16); `find /tmp/opencode/e2e/<ext> -mindepth 1` vacío tras `just e2e` |
      | e2e-tooling | Req 2 · `--bin RUTA` / `--keep` / exit 0/1 | Parseo del CLI en cada script (`--bin`/`--keep` en `main`), decisión del binario en `e2e_setup` (e2e_lib.sh:100-134), salida 0/1 en `e2e_summary` (e2e_lib.sh:453-458) |
      | e2e-tooling | Req 2 · esc. «Ejecución con el binario por defecto» | `e2e_setup` (e2e_lib.sh:124-129): `E2E_BIN` vacío → `cargo build --locked` y `target/debug/id3cli`; `check_rc 0` tras cada `e2e_run`. (Bajo `just e2e` la recipe pre-construye y pasa `--bin target/debug/id3cli`, ver Req 5) |
      | e2e-tooling | Req 2 · esc. «Ejecución con un binario indicado» | Parseo `--bin` en cada `main` + `e2e_setup` (e2e_lib.sh:124): `E2E_BIN` no vacío → no se construye |
      | e2e-tooling | Req 2 · esc. «Conservación del temporal para depuración» | Parseo `--keep` + `e2e_cleanup` (e2e_lib.sh:137-145): conserva e imprime «Temporal conservado (--keep)» |
      | e2e-tooling | Req 2 · esc. «Fallo con identificación de la aserción» | Resumen `e2e_summary` exit 1 si `E2E_FAIL>0` + mensajes FAIL con nombre de la aserción y valor esperado (p. ej. `check_show_contains` e2e_lib.sh:200; `check_ffprobe_tag` :296) |
      | e2e-tooling | Req 3 · funciones del runner común | Todas definidas en `e2e_lib.sh`: `e2e_setup`:100, `e2e_gen_audio`:152, `e2e_gen_covers`:174, `e2e_run`:186, `check_show_contains`:193, `check_ffprobe_tag`:284, `check_ffprobe_absent`:301, `check_cover_roundtrip`:361, `check_stderr_aviso`:219, `e2e_summary`:453 (más `e2e_die`, `check_rc`, etc.) |
      | e2e-tooling | Req 3 · esc. «El resumen cuenta las aserciones» | `e2e_summary` (e2e_lib.sh:453-458): las funciones de aserción incrementan `E2E_PASS`/`E2E_FAIL`; exit 0 solo sin fallos |
      | e2e-tooling | Req 3 · esc. «La aserción de show grep el valor, no el estilo» | `check_show_contains` con `grep -aqF` (e2e_lib.sh:193-203): grep binario del valor, nunca ANSI |
      | e2e-tooling | Req 4 · recipes del `.justfile` | `.justfile:23-43`: `e2e *args` (28-30), `e2e-mp3/m4a/ogg/opus *args` (32-43) con `{{args}}`; `just --list` muestra las 5 (tarea 4.1) |
      | e2e-tooling | Req 4 · esc. «Just e2e ejecuta los cuatro formatos en serie» | `.justfile:29-30`: `cargo build --locked` (línea 29, pre-construcción única) + `set -e; for ext in mp3 m4a ogg opus; do …; done` (línea 30) en una sola línea pasando `--bin target/debug/id3cli` (propaga el exit code del primer fallo, medido en 4.1); `just e2e` exit 0 (2026-10-16) |
      | e2e-tooling | Req 4 · esc. «Just por extensión con argumentos» | `.justfile:33-43` (`*args` + `{{args}}`); `just e2e-opus --bin target/debug/id3cli --keep` verificado en 4.1 |
      | e2e-tooling | Req 5 · temporal no versionado con limpieza garantizada | `e2e_setup` (e2e_lib.sh:111-115): `mkdir -p` + `mktemp -d "$base/e2e.XXXXXX"` bajo `/tmp/opencode/e2e/<ext>/` + `trap e2e_cleanup EXIT`; `set -euo pipefail` (e2e_lib.sh:39) |
      | e2e-tooling | Req 5 · esc. «Limpieza automática del temporal» | `trap e2e_cleanup EXIT` (e2e_lib.sh:115) + `rm -rf` en `e2e_cleanup` (:142-144); verificado: `/tmp/opencode/e2e/<ext>` vacíos tras `just e2e` (auditoría 2026-10-16) |
      | e2e-tooling | Req 5 · esc. «El binario se construye una única vez» | **CUMPLIDO (fix 2026-10-16)** — `.justfile:29`: la recipe `e2e` pre-construye UNA vez (`cargo build --locked` como línea propia) y `.justfile:30` pasa `--bin target/debug/id3cli` a los 4 scripts, de modo que `e2e_setup` recibe `E2E_BIN` no vacío y no ejecuta `cargo build` (e2e_lib.sh:124-129). Medido: `just e2e` → 1 línea «cargo build --locked» + 1 «Finished», 0 «Construyendo» en los scripts (evidencia en 4.1). Las recipes por extensión, invocadas solas sin `--bin`, construyen una vez cada una (escenario Req 2 «binario por defecto») |
      | e2e-tooling | Req 5 · esc. «Ausencia de herramientas obligatorias» | `e2e_setup` (e2e_lib.sh:104-109): `command -v ffmpeg/ffprobe` → `e2e_die` «la herramienta obligatoria «X» no está en el PATH» (exit 1, español) antes de crear el temporal |
      | e2e-verification | Req 1 · esc. «Cobertura completa en MP3» | e2e_mp3.sh: `e2e_run edit` con los 20 tags (82-101) + `check_rc 0` (102) + `check_show_contains` ×20 campo a campo (108-127, incl. `url`/`original_artist` sin omitir) + bucle `check_ffprobe_tag` sobre `FF_TAGS` (133-153) |
      | e2e-verification | Req 1 · esc. «Cobertura con omisiones avisadas en MP4, OGG y Opus» | e2e_m4a.sh: 18 tags + `check_stderr_aviso` url/original_artist (111-112) + `check_show_absent` URL/ORIGINAL (135-136) + `check_ffprobe_absent` url/original_artist (158-159); e2e_ogg.sh: :110-111, :134-135, :159-162; e2e_opus.sh: :106-107, :130-131, :154-157 |
      | e2e-verification | Req 1 · esc. «Estado inicial sin campos de usuario» | Paso 1 de cada script: `e2e_run show` + `check_show_absent` ×3 (`$E2E_V_TITLE`, `$E2E_V_ARTIST_JOINED`, `$E2E_V_URL`) — mp3:76-79, m4a:82-85, ogg:82-85, opus:78-81 |
      | e2e-verification | Req 2 · esc. «Relectura por el propio CLI» | `check_show_contains` (grep -aqF) paso 3 de los 4 scripts; valores de prueba en `E2E_V_*` (e2e_lib.sh:64-89, «ningún valor subcadena de otro» salvo los pares del spec) |
      | e2e-verification | Req 2 · esc. «Relectura independiente por ffprobe» | `check_ffprobe_tag`/`check_ffprobe_absent` con `ffprobe_entries_for` (e2e_lib.sh:264-270: `format_tags` mp3/m4a, `stream_tags` ogg/opus) y tabla `FF_TAGS` por script (celdas medidas, tarea 3.5) |
      | e2e-verification | Req 2 · esc. «Los campos plegados por ffmpeg» | MP3: `check_ffprobe_tag date "$E2E_V_YEAR"` (e2e_mp3.sh:151, TYER gana) + `check_show_contains "Año:"` (:111); MP4: `check_ffprobe_tag date "$E2E_V_DATE"` (e2e_m4a.sh:156, ©day gana fecha) + `check_show_contains "$E2E_V_DATE"` (:121); OGG/Opus: claves SEPARADAS → `check_ffprobe_raw_contains "YEAR=2024"` (ogg:155, opus:150) + `check_ffprobe_tag DATE` |
      | e2e-verification | Req 3 · esc. «Roundtrip JPG en los cuatro formatos» | Paso 8: `e2e_gen_audio cover_jpg` + `e2e_run edit --cover "$E2E_COVER_JPG"` + `check_cover_roundtrip "$E2E_COVER_JPG"` (mp3:193-196, m4a:198-201, ogg:213-216, opus:207-210) |
      | e2e-verification | Req 3 · esc. «Roundtrip PNG en los cuatro formatos» | Igual con `$E2E_COVER_PNG` (mp3:198-201, m4a:203-206, ogg:218-221, opus:211-214) |
      | e2e-verification | Req 3 · esc. «La extracción no re-codifica la imagen» | `check_cover_roundtrip` (e2e_lib.sh:361-383): extrae con `ffmpeg -map 0:v:0 -c copy -frames:v 1` y compara `cmp -s` byte a byte |
      | e2e-verification | Req 4 · esc. «Dos artistas unidos» | `e2e_run edit --artist ×2` (cada script) + `check_show_contains "$E2E_V_ARTIST_JOINED"` + `check_ffprobe_tag artist/ARTIST "$E2E_V_ARTIST_JOINED"` (separador `"; "` con `E2E_V_ARTIST_JOINED="Uno E2E; Dos E2E"`) |
      | e2e-verification | Req 5 · esc. «Pista con total» | MP3/MP4: `check_show_contains "7/12"` + `check_ffprobe_tag track "7/12"`; OGG/Opus: `check_show_contains "7/12"` (show imprime el par) + `check_ffprobe_tag track "7"` + `check_ffprobe_tag TRACKTOTAL "12"` (ogg:147,156; opus:142,151) |
      | e2e-verification | Req 5 · esc. «Temporada con total» | MP3/MP4: `check_show_contains "1/2"` + `check_ffprobe_tag disc "1/2"`; OGG/Opus: `check_ffprobe_tag disc "1"` + `check_ffprobe_tag DISCTOTAL "2"` (ogg:148,157; opus:143,152) |
      | e2e-verification | Req 5 · esc. «Número sin total» | Paso 7 (audio recién generado, mp3:181-189, m4a:186-194, ogg:198-208, opus:193-203): `check_show_contains "7"`/`"1"` + `check_show_absent "7/12"`/`"1/2"` + `check_ffprobe_tag track "7"`/`disc "1"`; OGG/Opus además `check_ffprobe_absent TRACKTOTAL`/`DISCTOTAL` |
      | e2e-verification | Req 6 · esc. «Omisión con aviso en MP4» | e2e_m4a.sh: `check_stderr_aviso` «no tiene átomo equivalente en MP4» ×2 (111-112) + `check_rc 0` (109, no aborta la edición) + `check_show_absent` URL/ORIGINAL (135-136) + `check_ffprobe_absent` url/original_artist (158-159, sin átomo freeform) |
      | e2e-verification | Req 6 · esc. «Omisión con aviso en OGG y Opus» | e2e_ogg.sh:110-111 y e2e_opus.sh:106-107 (`check_stderr_aviso` «no tiene clave equivalente en OGG») + `check_ffprobe_absent` url/original_artist (ogg:159-160, opus:154-155) + `grep_raw_absent "WEBSITE="`/`"ORIGINALARTIST="` en bytes crudos (ogg:161-162, opus:156-157) |
      | e2e-verification | Req 6 · esc. «Colisión de año y fecha en MP4» | e2e_m4a.sh: `check_stderr_aviso "--year 2023 queda absorbido por --date"` (113) + `check_ffprobe_tag date "2024-05-01"` (156) + `check_show_contains "$E2E_V_DATE"` (:121, valor ganador) |
      | e2e-verification | Req 7 · esc. «Borrado de un campo concreto» | Paso 5: `e2e_run remove <genre>` + `check_rc 0` + `check_show_absent genre` + `check_show_contains title`/`artist_joined` + `check_ffprobe_absent genre` + `check_ffprobe_tag title` (mp3:156-163, m4a:162-169, ogg:165-172, opus:160-167) |
      | e2e-verification | Req 7 · esc. «Borrado total con audio intacto y vendor preservado» | Paso 6: `e2e_run remove --all` + `check_rc 0` + `check_show_absent` ×3 + OGG/Opus `check_ffprobe_no_user_tags` (e2e_lib.sh:341) + `grep_raw_contains "$vendor"` (ogg:189, opus:184, vendor capturado `Lavf…` antes del borrado) + `check_duration_approx 1.5` (e2e_lib.sh:418); MP3 `check_no_id3v2_header` (e2e_lib.sh:441) + duration; MP4 `check_ffprobe_absent title/date` (:178-179) + duration |
      | e2e-verification | Req 7 · esc. «El bloque ID3v2 desaparece al borrar el último campo en MP3» | `check_no_id3v2_header` (e2e_mp3.sh:169) tras `remove --all` (la ruta de `remove --all` es la misma que deja el archivo sin campos; la suite Rust cubre `remove <campo>` sobre el último campo restante). Nota: el e2e no ejercita `remove <último campo>` en concreto — cobertura equivalente documentada |
      | e2e-verification | Req 8 · esc. «Corrupción de un valor esperado» | RED demostrado (tarea 2.3) con `red_demo.sh` temporal: `check_show_contains` con valor corrompido («TítuloQueNoExiste») → exit 1, mensaje en español y nombre de la aserción; `check_fail` (e2e_lib.sh:233-238) disponible como mecanismo de fallo inyectado |
      | e2e-verification | Req 8 · esc. «El flujo sin el fallo inyectado termina en verde» | `RED_DEMO_INJECT=0` → exit 0 y resumen con 0 fallidas (evidencia tarea 2.3) |

      > [!note] RESUELTO (2026-10-16, fix del `.justfile`): el hallazgo «cargo build ×4» ya no aplica — la recipe `e2e` pre-construye explícitamente UNA vez (`cargo build --locked` como línea propia de la recipe) y pasa `--bin target/debug/id3cli` a los cuatro scripts, por lo que el escenario «El binario se construye una única vez» (e2e-tooling Req 5) se cumple literalmente: una sola invocación de `cargo build` por `just e2e` (medido: 1 línea «cargo build --locked» + 1 «Finished», 0 «Construyendo» en los scripts; ver 4.1). Sin `just`, cada script invocado directamente sin `--bin` sigue construyendo una vez por su cuenta, como exige el escenario Req 2 «Ejecución con el binario por defecto». Sin impacto en aserciones ni en exit codes: `just e2e` sigue en EXIT=0 (mp3: 65, m4a: 68, ogg: 76, opus: 76, 0 fallidas).
- [x] 6.2 Commit convencional y PR a `development` — commit `test(e2e):` (d7a85ef), PR #25 fusionado a development el 2026-10-16, CI pass 36s.
      Verificación: commit con prefijo **`test(e2e): …`** (nunca `feat:`, verificado que `release-prepare.yml:38` hace `grep -q "^feat"` → `--minor`) desde la rama `feature/e2e-local-helpers` creada sobre `development`; PR a `development`; **sin PR a `main`**.
- [x] 6.3 Archivar el change al fusionar el PR — `openspec archive e2e-local-helpers --yes` ejecutado el 2026-10-16; capabilities consolidades en openspec/specs/ (e2e-tooling, e2e-verification).
      Verificación: `openspec archive e2e-local-helpers --yes` tras la aprobación y el merge; las capabilities `e2e-tooling` y `e2e-verification` quedan en `openspec/specs/`.
      Evidencia: (pendiente) salida de `openspec archive e2e-local-helpers --yes` y `openspec status` mostrando el change archivado.