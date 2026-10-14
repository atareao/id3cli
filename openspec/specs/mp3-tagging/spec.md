# mp3-tagging Specification

## Purpose
Fijar el contrato de paridad del comportamiento de edición de tags en MP3 mientras se sustituye el backend `id3` por `lofty`, de modo que todo lo que hoy funciona en MP3 siga funcionando exactamente igual después del cambio. El frame `TYER` se conserva tal cual, aunque esté obsoleto en ID3v2.4, para que la migración de backend sea mecánica y la paridad sea verificable; su unificación en `TDRC` se abordará en un cambio posterior e independiente.

## Requirements

### Requirement: Escritura siempre en ID3v2.4

Toda escritura de tags sobre un archivo MP3 MUST usar la versión ID3v2.4, del mismo modo que hace hoy `tag.write_to_path(file, id3::Version::Id3v24)`. El CLI MUST NOT degradar la versión del tag existente ni MUST escribir ID3v1 al guardar.

#### Scenario: Edición de un MP3 sin tags previos
- **WHEN** se ejecuta `edit` sobre un MP3 que no tiene ningún tag
- **THEN** el archivo resultante MUST contener un tag ID3v2.4 con los campos indicados

#### Scenario: Edición de un MP3 con tag ID3v1 existente
- **WHEN** se ejecuta `edit` sobre un MP3 que solo tiene un tag ID3v1
- **THEN** el CLI MUST escribir el tag ID3v2.4 con los campos indicados y MUST NOT escribir ni modificar un tag ID3v1

#### Scenario: Reescritura preserva la versión v2.4
- **WHEN** se ejecuta `edit` dos veces seguidas cambiando campos distintos sobre el mismo MP3
- **THEN** el tag resultante MUST seguir siendo ID3v2.4 en la segunda escritura

#### Scenario: Archivo no escribible
- **WHEN** el archivo MP3 no se puede abrir para escritura (permisos insuficientes)
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr un mensaje de error en español

### Requirement: Mapeo de los flags del CLI a los frames ID3v2.4

El CLI SHALL escribir cada flag de `edit` en el frame ID3v2.4 que le corresponde hoy, sin cambiar ningún mapeo: `--title` en `TIT2`, `--artist` en `TPE1`, `--album` en `TALB`, `--album-artist` en `TPE2`, `--genre` en `TCON`, `--track` en `TRCK`, `--season` en `TPOS`, `--composer` en `TCOM`, `--copyright` en `TCOP`, `--subtitle` en `TIT3`, `--original-artist` en `TOPE`, `--url` en `WOAR`, `--compilation` en `TCMP` con el valor `"1"`, `--title-sort` en `TSOT`, `--artist-sort` en `TSOP`, `--album-sort` en `TSOA`, `--lyrics` en `USLT` y `--cover` en `APIC`. En cuanto a las fechas, `--year` SHALL escribirse en el frame `TYER` y `--date` en el frame `TDRC`; son dos frames distintos y NO se sobrescriben entre sí. El CLI MUST NOT omitir ni redirigir a otro frame ninguno de estos campos en MP3.

#### Scenario: Mapeo de los campos básicos
- **WHEN** se ejecuta `edit` con `--title`, `--artist`, `--album`, `--album-artist`, `--genre`, `--track` y `--season` sobre un MP3
- **THEN** el archivo MUST contener sendos frames `TIT2`, `TPE1`, `TALB`, `TPE2`, `TCON`, `TRCK` y `TPOS` con los valores indicados

#### Scenario: Mapeo de los campos de fecha
- **WHEN** se ejecuta `edit --year 2026` sobre un MP3
- **THEN** el archivo MUST contener el frame `TYER` con el valor `2026`

#### Scenario: Mapeo de `--date`
- **WHEN** se ejecuta `edit --date 2026-10-13` sobre un MP3
- **THEN** el archivo MUST contener el frame `TDRC` con el valor `2026-10-13`

#### Scenario: `--year` y `--date` coexisten sin pisarse
- **WHEN** se ejecutan `edit --year 2026` y `edit --date 2026-10-13` sobre el mismo MP3
- **THEN** el archivo MUST contener simultáneamente el frame `TYER` con `2026` y el frame `TDRC` con `2026-10-13`, sin que ninguno sobrescriba al otro

#### Scenario: `--year` escribe `TYER`
- **WHEN** se ejecuta `edit --year 2026` sobre un MP3 sin tags previos
- **THEN** el archivo MUST contener el frame `TYER` con `2026` y MUST NOT contener un frame `TDRC`

#### Scenario: Mapeo de los créditos y las ordenaciones
- **WHEN** se ejecuta `edit` con `--composer`, `--copyright`, `--subtitle`, `--compilation`, `--title-sort`, `--artist-sort` y `--album-sort` sobre un MP3
- **THEN** el archivo MUST contener `TCOM`, `TCOP`, `TIT3`, `TCMP` con el valor `"1"`, `TSOT`, `TSOP` y `TSOA` con los valores indicados

#### Scenario: Mapeo de `--url` y `--original-artist` en MP3
- **WHEN** se ejecuta `edit` con `--url` y `--original-artist` sobre un MP3
- **THEN** el archivo MUST contener el frame `WOAR` con la URL y el frame `TOPE` con el artista original, sin ningún warning ni omisión

#### Scenario: Campo no indicado no se toca
- **WHEN** se ejecuta `edit` indicando solo `--title` sobre un MP3 que ya tenía artista y álbum
- **THEN** los frames de artista y álbum MUST permanecer intactos

### Requirement: Múltiples artistas unidos con el separador `"; "`

Cuando se indica `--artist` más de una vez, los valores MUST unirse en un único frame `TPE1` usando el separador `"; "`. Este comportamiento MUST NOT cambiar con respecto al actual.

#### Scenario: Tres artistas con separador
- **WHEN** se ejecuta `edit` con `--artist "Luis Fonsi" --artist "Daddy Yankee" --artist "Justin Bieber"`
- **THEN** el frame `TPE1` MUST contener exactamente el valor `Luis Fonsi; Daddy Yankee; Justin Bieber` en un solo frame

#### Scenario: Un único artista sin separador
- **WHEN** se ejecuta `edit` con un solo `--artist "Queen"`
- **THEN** el frame `TPE1` MUST contener exactamente `Queen`, sin separador añadido

#### Scenario: El separador no se aplica al artista del álbum
- **WHEN** se ejecuta `edit` con varios `--artist` y un `--album-artist`
- **THEN** el separador `"; "` MUST aplicarse solo al frame `TPE1` y el frame `TPE2` MUST conservar el valor literal del `--album-artist`

### Requirement: Letra en el frame USLT con código de idioma `spa`

La letra indicada con `--lyrics` MUST almacenarse en un frame `USLT` cuyo código de idioma sea `spa`. El CLI MUST NOT usar ningún otro código de idioma para las letras de MP3.

#### Scenario: Letra con idioma español
- **WHEN** se ejecuta `edit --lyrics "Primera línea\nSegunda línea"` sobre un MP3
- **THEN** el archivo MUST contener un frame `USLT` con el campo de idioma a `spa` y el texto completo de la letra

#### Scenario: Letra vacía
- **WHEN** se ejecuta `edit --lyrics ""` sobre un MP3
- **THEN** el CLI MUST escribir el frame `USLT` con idioma `spa` y texto vacío, y MUST NOT fallar

### Requirement: Carátulas en MP3 con detección de MIME por extensión

En MP3, la carátula indicada con `--cover` MUST escribirse en un frame `APIC`, y el tipo MIME MUST detectarse a partir de la extensión del archivo de imagen. El CLI SHALL aceptar `jpg` y `jpeg` como `image/jpeg`, `png` como `image/png` y `webp` como `image/webp`.

#### Scenario: Carátula JPEG y PNG
- **WHEN** se ejecuta `edit --cover portada.jpg` y, en otra invocación, `edit --cover portada.png` sobre un MP3
- **THEN** el archivo MUST contener un frame `APIC` con MIME `image/jpeg` en el primer caso y `image/png` en el segundo, con los bytes de la imagen intactos

#### Scenario: Carátula WebP en MP3
- **WHEN** se ejecuta `edit --cover portada.webp` sobre un MP3
- **THEN** el archivo MUST contener un frame `APIC` con MIME `image/webp`, sin ningún warning

#### Scenario: Extensión de imagen no soportada
- **WHEN** se ejecuta `edit --cover imagen.tiff` sobre un MP3
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr un mensaje en español que indique que el formato de imagen no está soportado

#### Scenario: Archivo de carátula inexistente
- **WHEN** se ejecuta `edit --cover ruta/que/no/existe.jpg` sobre un MP3
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr el mensaje en español de archivo de carátula no encontrado

### Requirement: `remove --all` borra físicamente ID3v1 e ID3v2

La opción `--all` de `remove` MUST eliminar físicamente el bloque ID3v2 y el tag ID3v1 del archivo, dejando el audio intacto, y MUST informar en español del estado previo encontrado: sin tags, solo ID3v1, solo ID3v2 o ambos. El CLI MUST NOT limitarse a vaciar los campos del tag.

#### Scenario: Archivo con ID3v1 e ID3v2
- **WHEN** se ejecuta `remove --all` sobre un MP3 que tiene los dos tags
- **THEN** el archivo resultante MUST NOT contener cabecera ID3v2 ni tag ID3v1 al final, y MUST emitir el mensaje de que ambos se eliminaron

#### Scenario: Archivo sin ningún tag
- **WHEN** se ejecuta `remove --all` sobre un MP3 que no tiene tags
- **THEN** el CLI MUST terminar con código de salida cero y MUST emitir el aviso de que el archivo no tenía tags ID3, sin modificar los bytes

#### Scenario: Archivo con un solo tipo de tag
- **WHEN** se ejecuta `remove --all` sobre un MP3 que solo tiene ID3v2, y en otra invocación sobre uno que solo tiene ID3v1
- **THEN** el mensaje emitido MUST distinguir cada caso y el tag presente MUST quedar eliminado del archivo

#### Scenario: El audio sobrevive al borrado
- **WHEN** se compara el tamaño y el contenido de la parte de audio antes y después de `remove --all`
- **THEN** los frames MPEG MUST ser idénticos y solo MUST haber desaparecido los bloques de tags

### Requirement: `remove` acepta nombres de tag en inglés y en español

El subcomando `remove` MUST aceptar cada tag tanto por su nombre en inglés como por su nombre en español (`title`/`título`, `artist`/`artista`, `album`/`álbum`, `year`/`año`, `genre`/`género`, `track`/`pista`, `season`/`temporada`, `date`/`fecha`, `composer`/`compositor`, `cover`/`carátula`, `lyrics`/`letra`, `album_artist`/`artista_album`, `album_sort`/`orden_album`, `artist_sort`/`orden_artista` y `title_sort`/`orden_titulo`, entre otros).

Regla: la comparación MUST ser insensible a mayúsculas y minúsculas (la entrada se normaliza a minúsculas antes de comparar). NO existe plegado de acentos: cada variante con y sin tilde que se acepte MUST figurar explícitamente en la tabla, y cualquier variante no listada MUST NOT reconocerse.

| Tag | Alias aceptados |
|---|---|
| title | `title`, `título` |
| artist | `artist`, `artista` |
| album | `album`, `álbum` |
| year | `year`, `año` |
| genre | `genre`, `género`, `genero` |
| track | `track`, `pista` |
| season | `season`, `temporada` |
| date | `date`, `fecha` |
| copyright | `copyright` |
| composer | `composer`, `compositor` |
| subtitle | `subtitle`, `subtítulo`, `subtitulo`, `description`, `descripción`, `descripcion` |
| original_artist | `original_artist`, `original-artist`, `artista_original`, `artista-original` |
| album_artist | `album_artist`, `album-artist`, `artista_album`, `artista-album` |
| cover | `cover`, `carátula`, `caratula` |
| lyrics | `lyrics`, `letra` |
| url | `url` |
| compilation | `compilation`, `compilación`, `compilacion` |
| album_sort | `album_sort`, `album-sort`, `orden_album`, `orden-album` |
| artist_sort | `artist_sort`, `artist-sort`, `orden_artista`, `orden-artista` |
| title_sort | `title_sort`, `title-sort`, `orden_titulo`, `orden-titulo` |

#### Scenario: Eliminación con nombres en español
- **WHEN** se ejecuta `remove cancion.mp3 título artista carátula` sobre un MP3 con esos tags
- **THEN** el título, el artista y la carátula MUST quedar eliminados del archivo

#### Scenario: Eliminación con nombres en inglés
- **WHEN** se ejecuta `remove cancion.mp3 title artist cover` sobre un MP3 con esos tags
- **THEN** el mismo conjunto de frames MUST quedar eliminado que con los nombres en español

#### Scenario: Combinación de nombres en ambos idiomas
- **WHEN** se ejecuta `remove cancion.mp3 title artista letra`
- **THEN** el CLI MUST interpretar cada nombre en su idioma y MUST eliminar los tres tags correspondientes

#### Scenario: Nombre de tag desconocido
- **WHEN** se ejecuta `remove cancion.mp3 inventado` sobre un MP3
- **THEN** el CLI MUST escribir en stderr un aviso en español con la lista de tags válidos, MUST NOT abortar y MUST NOT modificar el archivo si no queda ningún tag válido por eliminar

#### Scenario: `remove` sin tags y sin `--all`
- **WHEN** se ejecuta `remove cancion.mp3` sin argumentos de tags y sin `--all`
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST explicar en español que hay que indicar al menos un tag

#### Scenario: Límite del plegado de acentos
- **WHEN** se ejecuta `remove cancion.mp3 titulo`, y en otra invocación `remove cancion.mp3 genero caratula`, sobre un MP3 con esos tags
- **THEN** `titulo` (sin tilde) MUST NOT aceptarse para `title`, de modo que el CLI MUST avisar de que el nombre no es válido, mientras que `genero` y `caratula` (sin tilde) MUST aceptarse por figurar en la tabla y MUST eliminar los tags `genre` y `cover` respectivamente

### Requirement: `edit` sin cambios no reescribe el archivo

Cuando `edit` se invoca sin ningún flag de modificación, el CLI MUST NOT abrir el archivo para escritura y MUST limitarse a informar de que no se especificaron cambios.

#### Scenario: Invocación sin flags
- **WHEN** se ejecuta `edit cancion.mp3` sin ningún flag
- **THEN** el CLI MUST terminar con código de salida cero, MUST avisar de que no se especificaron cambios y el contenido del archivo MUST ser byte a byte idéntico al original

#### Scenario: Marca de tiempo intacta
- **WHEN** se ejecuta `edit cancion.mp3` sin flags sobre un archivo cuya fecha de modificación se registra antes de la ejecución
- **THEN** la fecha de modificación del archivo MUST NOT cambiar

#### Scenario: Un flag sin valor efectivo sí escribe
- **WHEN** se ejecuta `edit cancion.mp3 --title ""`
- **THEN** el CLI MUST considerar que hubo un cambio, MUST escribir el tag ID3v2.4 y MUST confirmar el guardado

### Requirement: Preservación de los tags existentes al añadir otros

Al editar campos concretos, el CLI MUST preservar los frames que no se han indicado en la invocación, incluidos los añadidos en ejecuciones anteriores.

#### Scenario: Añadir carátula conserva los metadatos
- **WHEN** se ejecuta `edit --cover nueva.jpg` sobre un MP3 que ya tenía título, artista y letra
- **THEN** los frames `TIT2`, `TPE1` y `USLT` MUST permanecer intactos y el archivo MUST incorporar el nuevo frame `APIC`

#### Scenario: Cambio de un solo campo
- **WHEN** se ejecuta `edit --year 2026` sobre un MP3 con título, artista, álbum y género
- **THEN** solo MUST cambiar el frame del año y el resto de frames MUST conservar sus valores

### Requirement: Mensajes de error del CLI en español

Los mensajes que el CLI redacta por sí mismo (sus propios avisos y confirmaciones) MUST estar en español, tanto en la salida de error como en la salida estándar, y el CLI MUST usar un código de salida distinto de cero para los errores de entrada del usuario. Los errores propagados desde la capa de entrada/salida o desde la librería de metadatos MAY aparecer con el texto propio de esa librería, que puede estar en inglés. Esa divergencia es una característica conocida del comportamiento actual y su traducción queda fuera del alcance de este cambio.

#### Scenario: Archivo inexistente
- **WHEN** se ejecuta cualquiera de los subcomandos sobre una ruta que no existe
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr el mensaje en español `Error: El archivo '<ruta>' no existe`

#### Scenario: Error de escritura del tag
- **WHEN** la escritura del tag falla por un error de entrada/salida
- **THEN** el CLI MUST terminar con un código de salida distinto de cero, y el mensaje propagado MAY aparecer en el idioma de la librería, que es el inglés. El test de caracterización fija el texto real observado

#### Scenario: Éxito confirmado en la salida estándar
- **WHEN** un `edit` o un `remove` se completa correctamente
- **THEN** el CLI MUST confirmar el resultado por stdout con un mensaje en español y MUST terminar con código de salida cero

### Requirement: El bloque ID3v2 MUST NOT quedar vacío tras eliminar el último campo

Cuando `remove` elimina el último campo que quedaba en el tag de un MP3, el CLI MUST eliminar ese campo, MUST terminar con código de salida cero, MUST NOT dejar en el archivo ningún bloque ID3v2 (ni siquiera uno vacío) y MUST preservar el audio intacto. Esto aplica al caso en que el tag queda sin ningún campo: el archivo resultante es un MP3 sin metadatos ID3v2. Cuando en el tag aún queden otros campos, el bloque ID3v2 MUST permanecer en el archivo con el resto de los campos intactos.

#### Scenario: Eliminación del último campo
- **WHEN** se ejecuta `remove` sobre el único campo de un MP3 cuyo tag tiene exactamente un campo (por ejemplo la letra)
- **THEN** el CLI MUST eliminar ese campo, MUST terminar con código de salida cero, el archivo MUST NOT contener ningún bloque ID3v2 y el audio MUST quedar byte a byte intacto

#### Scenario: Eliminación de un campo entre varios
- **WHEN** se ejecuta `remove` sobre uno de los campos de un MP3 cuyo tag tiene varios campos
- **THEN** el bloque ID3v2 MUST permanecer en el archivo y el resto de los campos MUST quedar intactos

#### Scenario: Coincidencia deliberada con `remove --all`
- **WHEN** se elimina el último campo del tag
- **THEN** el resultado sobre el archivo (ausencia de metadatos ID3v2) MUST ser el mismo que produce `remove --all`. Se trata de un comportamiento **aceptado deliberadamente** en este cambio, no de un efecto colateral no previsto

#### Scenario: `remove --all` no cambia
- **WHEN** se ejecuta `remove --all` sobre un MP3
- **THEN** el CLI MUST seguir eliminando físicamente los bloques ID3v1 e ID3v2 y MUST seguir informando en español del estado previo encontrado. Este escenario existe únicamente para dejar constancia de que el requisito ya existente sobre `--all` NO se modifica con este cambio
