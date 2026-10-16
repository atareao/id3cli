# Spec Delta

## Purpose

Capability nueva: la salida JSON de `show`. `id3cli show --json <archivo>` debe emitir en stdout UN ÚNICO objeto JSON compacto en una línea (con `\n` final) que represente los mismos metadatos que el modo legible muestra, en los cinco formatos (MP3, MP4/M4A, OGG, Opus y FLAC), con un esquema estable: claves siempre presentes (`null` si ausentes), arrays que nunca son `null`, booleana de compilación, pista/temporada como objetos numéricos, letra completa (sin preview) y carátulas SOLO como metadatos (type/mime/tamaño, sin bytes). El modo por defecto (sin `--json`) MUST quedar byte a byte idéntico.

## ADDED Requirements

### Requirement: `show --json` MUST emitir un único objeto JSON válido en stdout, plano y en una línea

`Command::Show` MUST aceptar un flag `--json`. Con el flag, el CLI MUST escribir en stdout exactamente un objeto JSON —`{ "formato": ..., "tags": ... }`— válido, compacto, en una sola línea y con salto de línea final, a través de la capa de salida existente (`outln!`), de modo que el cierre de pipe por el consumidor MUST terminar con exit 0 sin pánico (contrato `cli-output`). La salida MUST NOT contener bytes ANSI ni ningún otro texto fuera del objeto JSON. `formato` MUST ser `"mp3"`, `"mp4"`, `"ogg"`, `"opus"` o `"flac"` según el códec real detectado por contenido —en el contenedor Ogg, MUST distinguirse `"ogg"` (Vorbis) de `"opus"` (OpusHead), y el nombre MUST NO inferirse del `Backend` (que fusiona Ogg para los dos códecs)—.

#### Scenario: JSON del formato MP3
- **WHEN** se ejecuta `id3cli show --json` sobre un MP3 con tag ID3v2 con título y artista
- **THEN** el stdout MUST ser un único objeto JSON válido con `"formato": "mp3"`, `"tags.title"` con el título y `"tags.artists"` con el artista como array, en una línea, sin ANSI y con `\n` final

#### Scenario: JSON del formato Ogg distingue el códec
- **WHEN** se ejecuta `id3cli show --json` sobre un contenedor Ogg cuyo primer paquete es `OpusHead`
- **THEN** el `formato` MUST ser `"opus"` (y `"ogg"` para Vorbis), nunca un nombre genérico del contenedor

#### Scenario: JSON del formato FLAC
- **WHEN** se ejecuta `id3cli show --json` sobre un FLAC nativo con comentario Vorbis
- **THEN** el `formato` MUST ser `"flac"` y las carátulas MUST leerse de los bloques PICTURE del archivo (`read_flac_pictures`), no del comentario

#### Scenario: La salida es plana y única
- **WHEN** se captura el stdout completo del modo `--json`
- **THEN** el stdout MUST contener exactamente un documento JSON válido, sin bytes ANSI, sin líneas de aviso ni decoración, y con salto de línea final

### Requirement: El objeto `tags` MUST tener todas las claves presentes con `null` cuando el campo no existe

`tags` MUST ser un objeto con TODAS las claves canónicas siempre presentes: `title`, `artists`, `album`, `year`, `date`, `genre`, `track`, `season`, `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `url`, `lyrics`, `compilation`, `album_sort`, `artist_sort`, `title_sort` y `pictures`. Los campos ausentes en el archivo MUST emitirse como `null` (campos escalares), como array vacío (`artists`, `pictures`) o como booleana `false` (`compilation`) —nunca omitidos—. `track` y `season` MUST ser objetos `{"number": <u32>, "total": <u32|null>}` o `null` (en OGG/Opus/FLAC se componen de `TRACKNUMBER`/`TRACKTOTAL` y `DISCNUMBER`/`DISCTOTAL`; en MP3/MP4 de los accessors de `lofty`). `year` MUST ser número o `null`. `lyrics` MUST contener el texto COMPLETO (sin el preview de 3 líneas del modo legible). Los elementos de `pictures` MUST ser `{"type": string, "mime": string|null, "size_bytes": number}` y la lista MUST NOT incluir los bytes de la imagen.

#### Scenario: Campos presentes con valor
- **WHEN** el archivo tiene título, artista (dos, unidos con `"; "`), pista `7/12` y compilación
- **THEN** `tags.title` MUST ser el título, `tags.artists` MUST ser un array de dos elementos (MP3 divide por `"; "`), `tags.track` MUST ser `{"number": 7, "total": 12}` y `tags.compilation` MUST ser `true`

#### Scenario: Campos ausentes
- **WHEN** el archivo solo tiene álbum y el resto de campos no existen
- **THEN** `tags.title`, `tags.year`, etc. MUST ser `null`, `tags.artists` y `tags.pictures` MUST ser arrays vacíos, `tags.compilation` MUST ser `false`, y todas las claves MUST estar presentes en el documento

#### Scenario: Campos que el formato no soporta
- **WHEN** se serializa un MP4/M4A o un OGG/Opus/FLAC (formatos que omiten `--url` y `--original-artist`) sobre un archivo sin esos campos
- **THEN** `tags.url` y `tags.original_artist` MUST ser `null` (la verdad del archivo; no se inventan valores)

#### Scenario: Letra completa
- **WHEN** el archivo tiene una letra con más de 3 líneas
- **THEN** `tags.lyrics` MUST contener la letra completa sin truncar, a diferencia del preview del modo legible

### Requirement: Sin tag, `tags` MUST ser `null` con exit 0 y stderr limpio (solo modo JSON)

Cuando el contenedor no tiene tag —MP3 sin ID3, MP4 sin `ilst`—, el modo `--json` MUST emitir `{ "formato": "...", "tags": null }` por stdout, MUST terminar con exit 0 y MUST NOT escribir NADA en stderr (el aviso `⚠️` del modo legible queda suprimido: el consumidor del JSON es una máquina). En los contenedores que materializan siempre comentario (OGG/Opus/FLAC), un comentario vacío MUST emitirse como objeto `tags` con todas las claves `null`/vacías (el comentario existe): asimetría intencional que refleja la verdad del contenedor.

#### Scenario: MP3 sin tags en modo JSON
- **WHEN** se ejecuta `id3cli show --json` sobre un MP3 sin tag ID3v2
- **THEN** el stdout MUST ser `{"formato": "mp3", "tags": null}` (válido), el exit code MUST ser 0 y el stderr MUST estar vacío

#### Scenario: MP4 sin metadatos en modo JSON
- **WHEN** se ejecuta `id3cli show --json` sobre un MP4 sin átomo `ilst`
- **THEN** el stdout MUST ser `{"formato": "mp4", "tags": null}`, el exit code MUST ser 0 y el stderr MUST estar vacío

#### Scenario: OGG o FLAC sin campos de usuario
- **WHEN** se ejecuta `id3cli show --json` sobre un OGG/Opus/FLAC cuyo comentario no tiene campos de usuario
- **THEN** `tags` MUST ser un objeto con todas las claves `null` (o arrays/booleana vacíos) —MUST NOT ser `null`— y el exit code MUST ser 0

### Requirement: Los errores MUST seguir por stderr con exit ≠ 0 y NADA de JSON en stdout

Un archivo en formato no soportado (Speex, WAV, AIFF, APE, WavPack), un contenido no reconocible o un archivo inexistente MUST terminar con exit ≠ 0 y su mensaje en español por stderr, EXACTAMENTE igual que en el modo legible, y el stdout MUST NO recibir ningún JSON ni texto.

#### Scenario: Formato no soportado en modo JSON
- **WHEN** se ejecuta `id3cli show --json` sobre un archivo de formato no soportado
- **THEN** el CLI MUST terminar con exit ≠ 0, el mensaje en español MUST ir por stderr y el stdout MUST estar vacío (sin JSON)

#### Scenario: Pipe cerrado con JSON
- **WHEN** un consumidor cierra el extremo de lectura mientras `show --json` escribe
- **THEN** el CLI MUST terminar con exit 0 sin pánico ni mensajes por stderr (mismo contrato que el modo legible, vía `outln!`)