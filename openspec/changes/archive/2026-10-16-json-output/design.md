# Design

## Problema

`show` solo sabe imprimir para humanos: emojis, caja, previews. El plan director (Fase 3) exige una salida máquina estable para los cinco formatos. La lectura ya está resuelta por completo —`display_tags` (MP3), `display_tags_mp4`, `display_vorbis_content`/`display_tags_ogg`/`display_tags_flac` y `display_pictures_block`, todas en src/lib.rs—, así que el cambio es añadir serialización serde sobre los mismos datos y un flag `--json` en `show`. La evidencia clave del repo: la capa de salida `outln!`/`print_line` (src/output.rs) ya clasifica el BrokenPipe como final normal (exit 0) y es la única vía de escritura a stdout permitida por el proyecto; el backend enum distingue `Backend::Ogg` (que sirve a Vorbis Y Opus) de `Backend::Flac`, y `detect_format`/`AudioFormat` distinguen `OggVorbis` de `OggOpus`, información que el JSON necesita para nombrar el códec real.

## Decisión 1 — `show --json`: flag booleano, no subcomando

`Command::Show` gana `#[arg(short, long)] json: bool` (flag `--json`). Sin el flag, la rama actual se ejecuta byte a byte igual (no-rotura garantizada y verificable). Con el flag, cada rama de backend construye el struct serializable y vuelca `serde_json::to_string(...)` con `outln!`. Motivación: la salida JSON ES la misma información que `show` ya muestra, en formato máquina; un subcomando dedicado duplicaría el enrutado y la detección. El flag es aditivo: `id3cli show --json flac.flac` y `id3cli show --json -- flac.flac` son equivalentes para clap.

## Decisión 2 — Esquema: objeto único `{formato, tags}` con claves siempre presentes

Un único objeto JSON compacto en una línea. `formato` es string con el códec REAL detectado por contenido: `"mp3"`, `"mp4"`, `"ogg"`, `"opus"`, `"flac"`. `tags` es siempre un objeto (salvo `tags: null` cuando el contenedor no tiene tag) con TODAS las claves canónicas presentes: `title`, `artists` (array; MP3 divide el TPE1 por `"; "` con trim — misma convención de unión del CLI), `album`, `year` (número|null), `date` (string|null), `genre`, `track`/`season` (objeto `{"number": u32, "total": u32|null}` o null; en OGG/FLAC se componen de las claves TRACKNUMBER/TRACKTOTAL y DISCNUMBER/DISCTOTAL, en MP3/MP4 de los accessors de lofty), `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `url`, `lyrics` (texto COMPLETO, sin preview), `compilation` (bool), `album_sort`, `artist_sort`, `title_sort`, `pictures` (array, siempre; cada elemento `{"type": string del Debug del PictureType, "mime": string|null, "size_bytes": usize}`). Reglas de estabilidad: las claves se emiten SIEMPRE (null si ausente) para que un consumidor no tenga que defender `null` contra ausencia; los arrays nunca son null; `compilation` nunca es null. `year` es número porque el CLI lo escribe como entero (`--year i32`, TYER i32, YEAR numérico en Vorbis, ©day de 4 dígitos); `date` es string porque admite `YYYY-MM-DD`. La serialización con serde respeta el orden de campos del struct (orden de declaración), que el spec fija como estable.

## Decisión 3 — Sin tag: `tags: null` con exit 0 y stderr limpio, solo en modo JSON

Hoy, `show` sobre MP3 sin ID3 o MP4 sin ilst imprime un aviso `⚠️` por stderr y termina con exit 0; OGG/Opus/FLAC materializan siempre comentario (posiblemente vacío). En modo JSON: MP3 sin ID3 y MP4 sin ilst emiten `{"formato": "...", "tags": null}` por stdout, termina con exit 0 y NO escriben nada en stderr (el aviso ⚠️ se suprime porque el consumidor del JSON es una máquina; el contrato `cli-output` de «avisos por stderr» se matiza en el delta: aplica al modo legible y a los avisos de degradación de edit, no a este caso en modo JSON). OGG/Opus/FLAC sin campos emiten `tags` con todas las claves null/vacías (el comentario existe) — asimetría intencional y documentada: refleja la verdad del contenedor.

## Decisión 4 — El nombre del formato viene del `AudioFormat` real, no del `Backend`

`Backend::Ogg` sirve a dos códecs; el JSON debe nombrar `"ogg"` o `"opus"`. En la rama `Command::Show`, el modo JSON necesita el `AudioFormat` exacto: el mecanismo lo elige el implementador entre (a) volver a `detect_format(file)` en el modo JSON (barato: lee los primeros bytes) o (b) reestructurar el enrutado para conservar el `AudioFormat` junto al `Backend`; la decisión de diseño es que el nombre NUNCA se infiera del `Backend` (mentiría: `Backend::Ogg` no distingue códec). `Backend::Flac` → `"flac"` explícito (decisión 2.2 del change `flac-support`). El mapeo `AudioFormat → &str` es total y se testea por los cinco valores.

## Decisión 5 — Serialización con serde+serde_json en producción y Cargo.lock commiteado

`serde = { version = "1", features = ["derive"] }` y `serde_json = "1"` entran en `[dependencies]`. Alternativa evaluada y descartada: serialización manual con `format!`/`write!` — frágil (escapado, orden, null) e incoherente con la mantenibilidad del proyecto; serde da el esquema declarativo versionable que el spec exige. Por la regla del repo (CI `--locked`), Cargo.lock MUST cambiar y commitearse en el MISMO commit que Cargo.toml. Las estructuras viven en src/lib.rs (`ShowJson`, `TagsJson`, `TrackJson`, `PictureJson`, todas `#[derive(Serialize)]`) y las funciones constructoras por backend reciben los datos ya leídos (Option<String>, arrays, numbers) sin re-abrir el archivo.

## Decisión 6 — Letra completa, carátulas sin bytes

`lyrics` se emite COMPLETO (el preview de 3 líneas del modo legible es una decisión de presentación, no de datos: el JSON es para máquinas y truncar perdería información). `pictures` emite solo metadatos (type, mime, size_bytes) — NUNCA los bytes en base64 (un JPG de 9 KB aquí, pero en general MBs; el JSON debe ser ligero y el cliente extrae la imagen por los cauces existentes; no-goal explícito). La lista de pictures se computa con la misma lógica que el modo legible (APIC/ilst en tag, METADATA_BLOCK_PICTURE/`read_flac_pictures` en FLAC).

## Decisión 7 — e2e: `jq` como parser independiente y aserción por formato

El runner común gana `check_json_valid` (ejecuta `show --json`, valida `jq -e .` y comprueba ausencia de bytes ANSI con `grep -a $'\x1b'`), `check_json_format <esperado>` (`.formato`), `check_json_field <ruta-jq> <valor>` (`jq -r` + `grep -aF`) y `check_json_null <ruta-jq>` (extrae y compara contra `null`). Cada `e2e_*.sh` añade una etapa JSON tras el show normal: `formato` correcto, una muestra representativa (title, `artists[0]` en el archivo multi-artista, album, track.number, compilation, pictures count/type), y en MP3/MP4 el caso `tags:null` sobre audio recién generado sin tags. `jq` (v1.8.2 verificado en el sistema) se suma a las herramientas que `e2e_setup` comprueba. Recuentos: 392 actuales + ≈10-14 por formato, MEDIDOS en GREEN (nunca supuestos), con falsabilidad re-demostrada sobre una aserción JSON (6.x).

## Decisión 8 — Test documental intacto

`test_documentacion_describe_los_formatos_soportados` mide recuentos de keywords en el código fuente (`ESPERADAS`: id3/mp3/mp4/metadata/cli). La feature JSON NO altera esos recuentos (los structs y funciones se nombran `ShowJson`/`TagsJson`/`json_output_*` sin introducir las palabras medidas); el test no se toca salvo que el GREEN demuestre lo contrario (en cuyo caso el ajuste se registra como evidencia, nunca a ciegas).

## Alternativas descartadas

- **Subcomando `id3cli json FILE`**: duplica detección/enrutado; descartada (Decisión 1).
- **Serialización manual**: frágil e incoherente (Decisión 5).
- **`pretty`/indentado**: una línea compacta consumible con `jq .`; el indentado solo añade bytes (no-goal).
- **Incluir bytes de carátula (base64)**: peso desproporcionado; solo metadatos (Decisión 6).
- **Reutilizar `Backend` para nombrar el formato**: mentiría en Ogg; el nombre sale del `AudioFormat` (Decisión 4).
- **Un script `e2e_json.sh` transversal separado**: duplicaría la generación de audio por formato; la etapa JSON se integra en los `e2e_*.sh` existentes que ya tienen el archivo y los tags escritos (Decisión 7).

## Riesgos y mitigaciones

- **Recuentos e2e desconocidos a priori** → se miden en GREEN y se fijan con evidencia (Decisión 7), como se hizo con FLAC.
- **Multi-artista en MP3 con `"; "` dentro de un nombre real** → degradación documentada: la división por `"; "` es la convención del CLI (el propio CLI une con ese separador); el roundtrip JSON es coherente con lo que `edit` escribe.
- **`year` no numérico en Vorbis (archivo ajeno)** → `year: null` (no se inventa un número); `date` conserva el string.
- **Orden de claves**: lo fija el orden de campos del struct (serde respeta la declaración); el spec lo fija como estable y el e2e no depende de él (consulta por ruta jq).
- **Pipe cerrado con `--json`** → `outln!` ya lo clasifica: exit 0 sin pánico (contrato `cli-output` confirmado en el delta).
- **Cargo.lock desincronizado** → se commitea en el mismo cambio (regla `--locked`).