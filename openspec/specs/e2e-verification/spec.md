# e2e-verification Specification

## Purpose
Esta capability define la **matriz de verificación de extremo a extremo** que los scripts `scripts/e2e/e2e_*.sh` MUST cumplir: los 20 tags de `VALID_TAG_NAMES` con su forma por formato (escrito y releído, u omitido con aviso), el roundtrip de carátula JPG y PNG en los cuatro formatos, el multi-artista con separador `"; "`, track/season con y sin total, las degradaciones de MP4 y OGG, `remove`/`remove --all` con vendor preservado y audio reproducible, y la falsabilidad. La verificación es siempre en dos capas: `id3cli show` relee lo escrito por el propio CLI (grep del **valor** con `grep -a`) y `ffprobe` actúa como lector independiente con una tabla de clave por formato fijada en el runner. Esta capability fija el **qué** se verifica; los contratos de los scripts y del runner común son la capability `e2e-tooling`.

## Requirements

### Requirement: El e2e MUST ejercitar los 20 tags de `VALID_TAG_NAMES` con su forma por formato

El e2e MUST recorrer los 20 nombres de `VALID_TAG_NAMES` (`title, artist, album, year, genre, track, season, date, copyright, composer, subtitle, original_artist, album_artist, cover, lyrics, url, compilation, album_sort, artist_sort, title_sort`) sobre archivos recién generados. Por formato, MUST verificarse exactamente: en **MP3**, los 20 tags escritos y releídos (sin degradaciones); en **MP4/M4A**, en **OGG** y en **Opus**, 18 tags escritos y releídos más `url` y `original_artist` verificados como **omitidos con aviso por stderr** y ausentes del lector independiente. El flujo MUST empezar con un `show` inicial sobre el archivo recién generado que no muestre ningún campo de usuario: el runner MUST confirmar que no aparece ninguna etiqueta de campo de los 20 valores de prueba antes de escribir nada.

#### Scenario: Cobertura completa en MP3

- **WHEN** se ejecuta `e2e_mp3.sh` sobre un MP3 recién generado
- **THEN** los 20 tags MUST escribirse, MOSTRARSE en `show` con sus valores de prueba y releerse por el lector independiente según la tabla de claves de la capability, sin ninguna omisión ni aviso de degradación.

#### Scenario: Cobertura con omisiones avisadas en MP4, OGG y Opus

- **WHEN** se ejecuta `e2e_m4a.sh`, `e2e_ogg.sh` o `e2e_opus.sh` indicando también `--url` y `--original-artist`
- **THEN** los 18 tags restantes MUST escribirse y releerse, y `url` y `original_artist` MUST NO escribirse, debiendo quedar constancia del aviso por stderr (escenario «Campos sin equivalente se omiten con aviso») y su ausencia en el lector independiente.

#### Scenario: Estado inicial sin campos de usuario

- **WHEN** un script ejecuta `show` sobre el audio recién generado con `ffmpeg` (antes de cualquier `edit`)
- **THEN** la salida MUST NOT contener ninguno de los valores de prueba de la lista fijada (el archivo parte sin tags de usuario).

### Requirement: La verificación MUST hacerse en dos capas con una tabla de clave ffprobe por formato

Cada tag escrito MUST verificarse por `id3cli show` (capa 1, siempre) y, cuando el formato exponga una clave distinguible, por `ffprobe` como lector independiente (capa 2) mediante la tabla de mapeo del runner. La tabla SHALL fijar, por formato, el conjunto de entradas que lee `ffprobe` y la clave de cada campo: medida con `ffmpeg` 9.0.2 que **MP3 y MP4/M4A exponen la metadata en `format_tags`** y **OGG y Opus en `stream_tags`** (`format_tags` vacío en el contenedor Ogg); que **MP3 y MP4/M4A normalizan las claves a minúsculas** (`title`, `artist`, `album`, `track` como `7/12`, `disc` como `1/2`) mientras **OGG y Opus preservan la caja** de las claves Vorbis (`TITLE`, `ARTIST`, `ALBUM`, `YEAR`, `DATE`, `GENRE`, `LYRICS`, `COMPILATION`), con `TRACKNUMBER` y `DISCNUMBER` normalizados por `ffmpeg` a `track` y `disc` y los totales en `TRACKTOTAL`/`DISCTOTAL`. Las celdas ya medidas que la tabla MUST fijar son: MP3 → `title`, `artist`, `album`, `genre`, `track`, `disc`, `date` (TDRC), `copyright`, `composer`, `album_artist`, `compilation`, `lyrics-spa` (USLT con idioma), `title-sort`, `artist-sort`, `album-sort`, `TIT3` (subtitle) y `TOPE` (original-artist); MP4/M4A → `title`, `artist`, `album`, `genre`, `date` (átomo `©day`; `--year` se pliega en la misma clave), `track`, `disc`; OGG/Opus → `TITLE`, `ARTIST`, `ALBUM`, `YEAR`, `DATE`, `GENRE`, `LYRICS`, `COMPILATION`, `track`/`TRACKTOTAL`, `disc`/`DISCTOTAL`. Para el resto de celdas la tabla SHALL fijarse durante GREEN con una clave **real medida** sobre un archivo escrito por `id3cli` con ese campo —nunca una clave supuesta— y la medición SHALL registrarse en `tasks.md`; si para un campo `ffmpeg` no expone una clave distinguible en un formato (medido ya en MP3 para `url`: el frame TXXX no se expone en `format_tags`), la celda se marca «—» y ese campo se verifica por `show` y, si el runner lo define, por los bytes crudos del contenedor.

> [!note] Medición GREEN 2026-10-16 (tarea 3.5 de `tasks.md`): las celdas restantes se fijaron midiendo la salida literal de `ffprobe` sobre archivos escritos por `id3cli`. Correcciones al texto anterior (la medición no se maquilla): **MP4** expone el subtítulo freeform como `SUBTITLE` (caja que preserva ffmpeg) y los átomos `sonm`/`soar`/`soal` como `sort_name`/`sort_artist`/`sort_album` (NO como `*-sort`), además de `album_artist`, `lyrics` (multilínea), `copyright`, `composer` y `compilation`; **OGG/Opus** preservan la caja Vorbis EXCEPTO `ALBUMARTIST`, que ffmpeg normaliza a `album_artist` (minúsculas), y exponen `COPYRIGHT`, `COMPOSER`, `SUBTITLE`, `TITLESORT`, `ARTISTSORT`, `ALBUMSORT`; en **MP3**, cuando coexisten `TYER` (año) y `TDRC` (fecha) la clave compartida `date` expone el valor del AÑO (`TYER` gana al último `TDRC`), por lo que el par `--year`/`--date` se verifica por `show` y la clave `date` de ffprobe fija el valor ganador del plegado.

#### Scenario: Relectura por el propio CLI

- **WHEN** se ha escrito un tag y se ejecuta `id3cli show`
- **THEN** la salida MUST contener el valor de prueba del campo (grep del valor con `grep -a`, sin depender de etiquetas ANSI) y el runner MUST dar la aserción por buena.

#### Scenario: Relectura independiente por ffprobe

- **WHEN** se ha escrito un tag cuyo campo tiene clave en la tabla de su formato
- **THEN** `ffprobe` MUST exponer esa clave con el valor de prueba en el conjunto de entradas correcto (`format_tags` para MP3/MP4, `stream_tags` para OGG/Opus) y el runner MUST dar la aserción por buena.

#### Scenario: Los campos plegados por ffmpeg se verifican por el valor de unión

- **WHEN** un formato expone dos campos lógicos bajo la misma clave (p. ej. `year` y `date` pliegan en `date` en MP3 por `TYER`/`TDRC` y en MP4 por el átomo compartido `©day`)
- **THEN** la verificación independiente de esos campos MUST comprobar que la clave compartida contiene el valor ganador esperado, y la distinción campo a campo MUST quedar cubierta por la capa 1 (`show`), tal y como fije la tabla.

### Requirement: La carátula MUST verificarse como roundtrip de bytes JPG y PNG en los cuatro formatos

El e2e MUST escribir la carátula `cover.jpg` y la `cover.png` generadas con `ffmpeg` (`-f lavfi -i testsrc`), una por archivo recién generado, y verificar el roundtrip extrayendo el stream de imagen con `ffmpeg -map 0:v:0 -c copy -frames:v 1 <salida>` y comparando **byte a byte o por hash** la extracción contra la imagen original. La extracción MUST usar `-c copy` (medido que sin él `ffmpeg` re-codifica y los bytes difieren, p. ej. 9039 frente a 9056) y MUST escribir a un fichero con la extensión de la imagen. El WebP queda **fuera de alcance** de esta fase; el runner MUST dejar documentado el hook para ampliarlo.

#### Scenario: Roundtrip JPG en los cuatro formatos

- **WHEN** cada script escribe `cover.jpg` sobre un audio recién generado de su formato y extrae el stream de imagen
- **THEN** la extracción MUST ser idéntica (mismo hash) a `cover.jpg`, en MP3, MP4/M4A, OGG y Opus.

#### Scenario: Roundtrip PNG en los cuatro formatos

- **WHEN** cada script escribe `cover.png` sobre un audio recién generado de su formato y extrae el stream de imagen
- **THEN** la extracción MUST ser idéntica (mismo hash) a `cover.png`, en MP3, MP4/M4A, OGG y Opus.

#### Scenario: La extracción no re-codifica la imagen

- **WHEN** el runner extrae la carátula
- **THEN** MUST usar `-c copy` para que los bytes extraídos sean los almacenados y no una re-codificación del decodificador.

### Requirement: El multi-artista MUST escribirse y releerse con el separador `"; "`

El e2e MUST escribir dos artistas y verificar que el CLI los une con el separador `"; "` (un valor `Uno E2E; Dos E2E`) tanto en `show` como en el lector independiente (en MP3 y MP4 como `artist`, en OGG y Opus como `ARTIST`).

#### Scenario: Dos artistas unidos

- **WHEN** se ejecuta `edit --artist "Uno E2E" --artist "Dos E2E"`
- **THEN** `show` y `ffprobe` MUST exponer `Uno E2E; Dos E2E` con el separador `"; "` en los cuatro formatos.

### Requirement: `track` y `season` MUST verificarse con y sin total en su forma nativa

El e2e MUST ejercitar `--track` y `--season` en las dos formas: número solo (`7`, `1`) y número con total (`7/12`, `1/2`). La forma con total MUST reclararse en la representación nativa de cada contenedor y verificarse por las dos capas: en MP3 y MP4 el par se relee como `7/12` (frames `TRCK`/`TPOS` y átomos `trkn`/`disk`); en OGG y Opus el lector independiente MUST exponer `track=7` con `TRACKTOTAL=12` y `disc=1` con `DISCTOTAL=2`. La forma sin total MUST NOT dejar total en ningún formato.

#### Scenario: Pista con total

- **WHEN** se ejecuta `edit --track 7/12` en cada formato
- **THEN** MP3 y MP4 MUST releer el par `7/12`; OGG y Opus MUST exponer `track=7` y `TRACKTOTAL=12` en `stream_tags`.

#### Scenario: Temporada con total

- **WHEN** se ejecuta `edit --season 1/2` en cada formato
- **THEN** MP3 y MP4 MUST releer el par `1/2` (clave `disc` en ffprobe); OGG y Opus MUST exponer `disc=1` y `DISCTOTAL=2`.

#### Scenario: Número sin total

- **WHEN** se ejecuta `edit --track 7 --season 1` en cada formato
- **THEN** MUST escribirse el número y MUST NOT quedar ningún total en el lector independiente ni en `show`.

### Requirement: Los campos sin equivalente MUST omitirse con aviso por stderr y sin escribir claves o átomos inventados

Los campos que cada formato no puede representar MUST omitirse con un aviso por stderr y MUST NOT escribirse como clave o átomo *freeform* inventado: en **MP4/M4A**, `--url` y `--original-artist` (medido el aviso `El campo '--url' no tiene átomo equivalente en MP4; se omite.`); en **OGG y Opus**, `--url` y `--original-artist` (medido el aviso `El campo '--url' no tiene clave equivalente en OGG; se omite.`), y la ausencia MUST comprobarse además en los bytes crudos del comentario (MUST NOT aparecer `WEBSITE=` ni `ORIGINALARTIST=`). En **MP4**, cuando se indican `--year` y `--date` a la vez, MUST ganar `--date` (comparten el átomo `©day`) con el aviso medido (`--year 2023 queda absorbido por --date ...`). La omisión de un campo MUST NO abortar la escritura del resto de la edición.

#### Scenario: Omisión con aviso en MP4

- **WHEN** se ejecuta `edit --url ... --original-artist ...` (junto al resto de campos) sobre un `.m4a`
- **THEN** el stderr MUST contener los avisos de ambos campos, el CLI MUST terminar con exit 0, el resto de campos MUST escribirse, `ffprobe` MUST no mostrar ninguna clave de URL ni de artista original, y MUST NOT existir ningún átomo *freeform* para ellos.

#### Scenario: Omisión con aviso en OGG y Opus

- **WHEN** se ejecuta `edit --url ... --original-artist ...` (junto al resto de campos) sobre un `.ogg` o un `.opus`
- **THEN** el stderr MUST contener los avisos de ambos campos, el resto de campos MUST escribirse, `ffprobe` MUST no mostrar ninguna clave de URL ni de artista original, y los bytes crudos del comentario MUST NOT contener `WEBSITE=` ni `ORIGINALARTIST=`.

#### Scenario: Colisión de año y fecha en MP4

- **WHEN** se ejecuta `edit --year 2023 --date 2024-05-01` sobre un `.m4a`
- **THEN** el stderr MUST contener el aviso de absorción, el átomo `©day` MUST quedar con `2024-05-01` y `show` MUST mostrar el valor ganador.

### Requirement: `remove` y `remove --all` MUST verificarse con vendor preservado y audio reproducible

El e2e MUST ejercitar `remove <campo>` (eliminar un campo concreto y confirmar que el resto permanece) y `remove --all` (eliminar todos los metadatos) en los cuatro formatos. Tras `remove --all`, en **OGG y Opus** el comentario MUST quedar vacío **preservando el vendor** (que es obligatorio en el contenedor), y en los cuatro formatos el audio MUST seguir siendo reproducible: `ffprobe` MUST seguir reconociendo el archivo y su duración MUST conservarse. En MP3, al eliminar el último campo, el bloque ID3v2 MUST desaparecer del archivo según el comportamiento actual del CLI.

#### Scenario: Borrado de un campo concreto

- **WHEN** se ejecuta `remove <campo>` tras una edición completa
- **THEN** el campo indicado MUST desaparecer y el resto de campos MUST permanecer verificables por las dos capas.

#### Scenario: Borrado total con audio intacto y vendor preservado

- **WHEN** se ejecuta `remove --all` en cada formato
- **THEN** `show` MUST no mostrar campos de usuario; en OGG y Opus `ffprobe` MUST no mostrar claves de usuario pero el archivo MUST seguir reconociéndose con su duración (vendor preservado); en los cuatro formatos el audio MUST permanecer reproducible.

#### Scenario: El bloque ID3v2 desaparece al borrar el último campo en MP3

- **WHEN** se elimina el último campo restante de un MP3
- **THEN** el bloque ID3v2 MUST desaparecer del archivo (no se conserva vacío), igual que con `remove --all`.

### Requirement: El e2e MUST ser falsable: un fallo deliberado MUST terminar en 1

La verificación de calidad de los scripts MUST incluir un caso negativo deliberado —corromper un valor esperado o forzar un fallo de una aserción— que demuestre que el script detecta la discrepancia: al inyectar el fallo, el script MUST terminar con exit 1, mensaje claro en español y nombre de la aserción fallida; sin el fallo inyectado, el mismo flujo MUST terminar con exit 0. Este patrón es el mismo que usan los casos negativos de `generate_sample_*.sh`.

#### Scenario: Corrupción de un valor esperado

- **WHEN** se corrompe deliberadamente el valor esperado de una aserción (p. ej. se espera `TítuloE2E` y el archivo contiene otro valor)
- **THEN** el script MUST terminar con exit 1 e informar del nombre de la aserción fallida y del valor esperado frente al obtenido.

#### Scenario: El flujo sin el fallo inyectado termina en verde

- **WHEN** se restaura el valor correcto y se vuelve a ejecutar el mismo flujo
- **THEN** el script MUST terminar con exit 0 y el resumen PASS/FAIL MUST reportar cero aserciones fallidas.

### Requirement: El e2e MUST verificar la visibilidad externa de las etiquetas MP4 tras `remove --all` + `edit`

El flujo de `e2e_m4a.sh` MUST incluir una secuencia que edite el archivo con `--title`, elimine todos los metadatos con `remove --all` y vuelva a editarlo con los mismos campos, y MUST verificar después de esa secuencia que `id3cli show` muestra los campos y que `ffprobe -show_entries format_tags` los lista de nuevo (no solo los brands), de modo que un fallo de visibilidad externa como el del bug `hdlr` haga terminar el script con exit 1.

#### Scenario: La secuencia de reescritura mantiene visibles las etiquetas

- **WHEN** `e2e_m4a.sh` ejecuta `edit` (campos de prueba) → `remove --all` → `edit` (los mismos campos) sobre el `.m4a` generado
- **THEN** `check_show_contains` MUST verificar los campos por `id3cli show`, `check_ffprobe_tag` MUST verificar al menos el título y el artista en `format_tags` del archivo final, y el flujo MUST terminar en verde sin fallos

#### Scenario: La regresión del `hdlr` es detectada como fallo

- **WHEN** el archivo final de la secuencia anterior no expone las etiquetas en `ffprobe format_tags` (regresión del `hdlr`)
- **THEN** la aserción `check_ffprobe_tag` MUST fallar y el script MUST terminar con exit 1 e informar del nombre de la aserción

### Requirement: El e2e MUST verificar que una segunda edición de `--cover` sustituye en lugar de acumular

El flujo de carátula MUST incluir una segunda edición de `--cover` con una imagen distinta y verificar que el archivo resultante contiene exactamente una carátula del tipo por defecto del formato (`CoverFront` en MP3; la que fije el backend en los demás) con los bytes de la **última** imagen, sin conservar los de la primera. La aserción se aplica en MP3 y en los demás formatos donde la evidencia demuestre la acumulación (ver change `e2e-findings-fixes`); donde el formato ya sustituya nativamente, la aserción MUST fijar igualmente el comportamiento correcto.

#### Scenario: Segunda carátula sustituye a la primera

- **WHEN** el flujo de un formato ejecuta `edit --cover caratula_a.jpg` y después `edit --cover caratula_b.jpg` sobre el mismo archivo
- **THEN** la verificación de `show` MUST reportar una única carátula con los bytes de `caratula_b.jpg` (roundtrip `-c copy` contra `caratula_b.jpg`), MUST NOT aparecer la imagen `caratula_a.jpg`, y la aserción MUST fallar con exit 1 si el formato acumula las dos

#### Scenario: Sustitución con un solo cover no regresa

- **WHEN** se ejecuta el flujo de carátula con una única edición de `--cover` (el roundtrip JPG/PNG ya existente)
- **THEN** la verificación MUST seguir pasando exactamente igual que antes de este cambio, sin regresión en las aserciones existentes

### Requirement: El e2e MUST ejercitar los 20 tags de VALID_TAG_NAMES con su forma por formato (FLAC)

Sobre un FLAC recién generado, el e2e MUST verificar exactamente la misma forma que OGG y Opus: **18 tags** escritos y releídos más `url` y `original_artist` verificados como **omitidos con aviso por stderr** y ausentes del lector independiente. El flujo MUST empezar con un `show` inicial que no muestre ningún campo de usuario.

#### Scenario: Los 18 tags de FLAC escritos y releídos
- **WHEN** se ejecuta `e2e_flac.sh` y este edita el `.flac` con los 18 tags (sin `url` ni `original_artist`)
- **THEN** `show` MUST releerlos con los mismos valores y el lector independiente MUST exponerlos según la tabla de clave ffprobe de FLAC

#### Scenario: url y original_artist omitidos con aviso en FLAC
- **WHEN** el mismo script indica también `--url` y `--original-artist`
- **THEN** MUST emitirse un aviso por stderr para cada uno, MUST no escribirse ninguna clave inventada y el lector independiente MUST no exponerlos

### Requirement: La verificación MUST hacerse en dos capas con una tabla de clave ffprobe por formato (FLAC medido)

La tabla de entradas de ffprobe de FLAC SHALL fijarse durante GREEN con una clave **real medida** sobre un archivo escrito por `id3cli` con ese campo —registrada en tasks.md— y reflejar: el conjunto de entradas (`format_tags` o `stream_tags`), la caja de las claves (Vorbis en mayúsculas o normalizadas), el par `track`/`disc` con sus totales (`TRACKTOTAL`/`DISCTOTAL`) y la ausencia de `url`/`original_artist`. Si para algún campo `ffmpeg` no expone una clave distinguible, la celda se marca «—» y ese campo se verifica por `show`, igual que se hace en los demás formatos.

#### Scenario: Las celdas de FLAC son las medidas
- **WHEN** se consulta la tabla del spec con las claves de FLAC
- **THEN** cada celda MUST proceder de la medición registrada en tasks.md (con fecha y versión de `ffmpeg`), no de una suposición

> [!note] Medición GREEN 2026-10-16 (tarea 6.3 de `tasks.md`, ffmpeg/ffprobe n9.0.2): FLAC expone la metadata en **`format_tags`** (el `stream_tags` queda vacío, a diferencia de OGG/Opus) con la caja Vorbis **preservada** (`TITLE`, `ARTIST`, `ALBUM`, `YEAR`, `DATE`, `GENRE`, `COPYRIGHT`, `COMPOSER`, `SUBTITLE`, `LYRICS`, `COMPILATION`, `TITLESORT`, `ARTISTSORT`, `ALBUMSORT`), y con las mismas normalizaciones a minúsculas que OGG/Opus: `TRACKNUMBER`→`track`, `DISCNUMBER`→`disc` y `ALBUMARTIST`→`album_artist`; los totales viven en `TRACKTOTAL`/`DISCTOTAL`, que sí se exponen. `url`/`original_artist` → «—» (omitidos con aviso por stderr, literal generalizado «en los comentarios de Vorbis»; MUST NO aparecer). El par `--year`/`--date` son claves separadas (`YEAR`/`DATE`), por lo que el año se fija por el par clave+valor crudo (`YEAR=…`), como en OGG/Opus. Tras `remove --all` el bloque VORBIS_COMMENT se elimina completo (vendor NO conservado) y `format_tags` queda vacío.

### Requirement: La carátula MUST verificarse como roundtrip de bytes JPG y PNG en FLAC y mostrarse por show

El e2e MUST escribir la carátula `cover.jpg` y la `cover.png` (generadas con `ffmpeg testsrc`) sobre FLAC recién generado, una por archivo, y verificar el roundtrip extrayendo el stream de imagen con `ffmpeg -map 0:v:0 -c copy -frames:v 1` y comparando byte a byte contra la imagen original. Además, sobre el MISMO archivo, una segunda edición de `--cover` con la otra imagen MUST dejar exactamente una carátula (la última), verificada por `show` («Carátulas: 1 imagen(es)», mostrada desde los bloques PICTURE), por roundtrip contra la última imagen y por la ausencia de la primera (`check_cover_not_roundtrip`).

#### Scenario: Roundtrip JPG y PNG en FLAC
- **WHEN** `e2e_flac.sh` escribe `cover.jpg` y `cover.png` sobre FLAC recién generado y extrae el stream de imagen
- **THEN** la extracción MUST coincidir byte a byte con la imagen original en los dos casos

#### Scenario: Segunda edición de carátula sustituye en FLAC
- **WHEN** el mismo archivo recibe `edit --cover imagen_a` y después `edit --cover imagen_b`
- **THEN** `show` MUST reportar «Carátulas: 1 imagen(es)», el roundtrip contra `imagen_b` MUST pasar y el anti-roundtrip contra `imagen_a` MUST fallar si el formato acumulara

### Requirement: remove --all en FLAC elimina el bloque de comentario y deja el audio reproducible

`e2e_flac.sh` MUST verificar `remove --all`: el comentario queda sin campos de usuario ni carátulas y el bloque VORBIS_COMMENT desaparece del archivo (el vendor NO se conserva en FLAC — degradación verificada, a diferencia de Ogg/Opus donde el vendor se preserva). El archivo MUST seguir siendo un FLAC reproducible por `ffprobe` (codec `flac`, duración aproximada esperada). El script MUST verificar también `remove` de un campo concreto con el resto intacto.

#### Scenario: remove --all deja el FLAC reproducible sin bloque de comentario
- **WHEN** se ejecuta `remove --all` sobre el `.flac` editado
- **THEN** el archivo MUST quedar sin metadatos de usuario, sin carátulas y sin bloque VORBIS_COMMENT (vendor perdido, degradación documentada), y `ffprobe` MUST reconocerlo como audio FLAC reproducible (duración ≈1,5 s)

### Requirement: Cada formato MUST verificar su salida JSON con jq

El e2e de cada formato MUST ejecutar `show --json` sobre el archivo editado y MUST verificar: (a) `check_json_valid` (JSON válido + sin ANSI); (b) `check_json_format` con el códec real del formato (`mp3`, `m4a`→`mp4`, `ogg`, `opus`, `flac`); (c) una muestra representativa de campos con `check_json_field`: título, primer artista del array (en la etapa multi-artista), álbum, `track.number`, `compilation` y `pictures` (count/type); (d) en MP3 y MP4, el caso `tags:null` sobre audio recién generado sin tags (stderr vacío, exit 0).

#### Scenario: La capa JSON de MP3
- **WHEN** `e2e_mp3.sh` ejecuta su etapa JSON sobre el MP3 con los 20 tags escrito
- **THEN** `check_json_valid` MUST pasar, `check_json_format 'mp3'` MUST pasar, `check_json_field '.tags.title'` con el título de prueba MUST pasar y `check_json_field '.tags.track.number' '7'` MUST pasar

#### Scenario: Los formatos Ogg/Opus/FLAC nombran su códec real
- **WHEN** `e2e_ogg.sh`, `e2e_opus.sh` y `e2e_flac.sh` ejecutan su etapa JSON
- **THEN** `check_json_format` MUST esperar `ogg`, `opus` y `flac` respectivamente, y `check_json_field '.tags.track.number' '7'` MUST pasar en los tres

#### Scenario: MP3 sin tags en el e2e
- **WHEN** `e2e_mp3.sh` ejecuta `show --json` sobre el audio recién generado (antes de editar)
- **THEN** la aserción MUST confirmar `"tags": null` en el documento (`check_json_null '.tags'`), stderr vacío y exit 0

### Requirement: La falsabilidad MUST re-demostrarse sobre una aserción JSON

El flujo de falsabilidad del e2e MUST incluir la corrupción deliberada de una aserción JSON (p. ej. esperar `'mp3'` donde el formato es `'flac'` o un valor de campo erróneo): con el fallo inyectado el script MUST terminar con exit 1 nombrando la aserción; restaurado, MUST terminar con exit 0.

#### Scenario: Corrupción de una aserción JSON
- **WHEN** se corrompe deliberadamente el valor esperado de un `check_json_format` o `check_json_field`
- **THEN** el script MUST terminar con exit 1 e informar del nombre de la aserción fallida y del valor esperado frente al obtenido

#### Scenario: El flujo restaurado queda en verde
- **WHEN** se restaura el valor correcto y se vuelve a ejecutar el mismo flujo
- **THEN** el script MUST terminar con exit 0 y el resumen PASS/FAIL MUST reportar cero aserciones fallidas

### Requirement: El e2e MUST verificar la capa de completions con `bash -n` como oráculo

El e2e MUST ejecutar `id3cli completions bash` y verificar: (a) exit 0 con stderr vacío; (b) el stdout contiene la subcadena estable `complete -F _id3cli` (sin fijar los flags de la plantilla); (c) el stdout, guardado a fichero, pasa `bash -n` (script bash sintácticamente válido — el intérprete real como oráculo). Para `completions zsh` MUST verificar `#compdef id3cli` y para `completions fish` líneas `complete -c id3cli`, ambas con exit 0 y stderr vacío. MUST verificar que el script bash menciona los subcomandos `show`, `edit`, `remove` y `completions`.

#### Scenario: La capa de completions de bash
- **WHEN** el e2e de completions verifica `id3cli completions bash`
- **THEN** exit 0, stderr vacío, la subcadena estable `complete -F _id3cli` presente en stdout y `bash -n` sobre el stdout guardado devuelve 0

#### Scenario: La capa de completions de zsh y fish
- **WHEN** el e2e de completions verifica `id3cli completions zsh` y `id3cli completions fish`
- **THEN** ambos terminan con exit 0 y stderr vacío, el primero contiene `#compdef id3cli` y el segundo `complete -c id3cli`

### Requirement: El e2e MUST verificar el fallo de un shell no soportado y el cierre del pipe

El e2e MUST ejecutar `id3cli completions powershell` y verificar exit ≠ 0 con stderr conteniendo `bash`, `zsh` y `fish` y stdout vacío. MUST verificar que el cierre del pipe durante `completions bash` es un final normal (exit 0).

#### Scenario: Shell no soportado y pipe cerrado
- **WHEN** el e2e ejecuta `id3cli completions powershell`
- **THEN** el exit code es distinto de 0 y el stderr nombra los tres shells soportados
- **AND WHEN** un consumidor cierra el pipe durante `id3cli completions bash`
- **THEN** el proceso termina con exit 0

### Requirement: La falsabilidad MUST re-demostrarse sobre una aserción de completions

El e2e de completions MUST demostrar que el resumen detecta un fallo real: corrompiendo deliberadamente una expectativa (p. ej. esperar `#compdef id3cli` a la salida de bash) el script termina con exit 1 y nombra la aserción fallida con lo esperado vs lo obtenido; restaurada, termina con exit 0.

#### Scenario: Falsabilidad de la capa de completions
- **GIVEN** una copia temporal del script con una expectativa corrompida
- **WHEN** se ejecuta la copia
- **THEN** termina con exit 1 y el FAIL nombra la aserción y el valor esperado
- **AND WHEN** se ejecuta la versión original
- **THEN** termina con exit 0 y 0 fallidas
