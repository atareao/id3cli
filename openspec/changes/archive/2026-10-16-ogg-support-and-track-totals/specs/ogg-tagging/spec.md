# Spec Delta

## Purpose

Definir la lectura y la escritura de metadatos en archivos Ogg Vorbis (`.ogg`) mediante comentarios de Vorbis (`VorbisComments`, pares clave=valor en mayúsculas), con el mapeo de cada flag del CLI a su clave, las reglas de omisión de campos sin equivalente y las degradaciones conocidas del contenedor. El alcance son los archivos Ogg Vorbis que contengan (siempre) un comentario de Vorbis y que el CLI edite a través de las claves de la tabla de mapeo de `lofty` 0.25.4 (`VORBIS_MAP`), con independencia de la herramienta que los haya creado. La numeración de pista y de temporada (`N` o `N/T`) queda contratada en la capability `track-numbering`; aquí solo se define en qué clave se escribe cada parte.

## ADDED Requirements

### Requirement: Mapeo de los flags del CLI a las claves de Vorbis

El CLI SHALL escribir cada flag de `edit` en la clave de Vorbis que le corresponde: `--title` en `TITLE`, `--artist` en `ARTIST`, `--album` en `ALBUM`, `--album-artist` en `ALBUMARTIST`, `--year` en `YEAR`, `--date` en `DATE`, `--genre` en `GENRE`, `--track` en `TRACKNUMBER`, `--season` en `DISCNUMBER`, `--composer` en `COMPOSER`, `--copyright` en `COPYRIGHT`, `--lyrics` en `LYRICS`, `--cover` en `METADATA_BLOCK_PICTURE`, `--compilation` en `COMPILATION` con el valor `"1"`, `--title-sort` en `TITLESORT`, `--artist-sort` en `ARTISTSORT`, `--album-sort` en `ALBUMSORT` y `--subtitle` en `SUBTITLE`. El total de pista y de temporada SHALL escribirse en `TRACKTOTAL` y `DISCTOTAL` respectivamente. El CLI MUST NOT escribir ninguna otra clave para representar estos campos, y MUST NOT escribirlos en ninguna otra forma de la clave (por ejemplo en minúsculas). `--year` y `--date` son claves distintas e independientes: cada una MUST escribirse en su propia clave y el CLI MUST NOT sobrescribirlas entre sí ni absorber una en la otra.

#### Scenario: Escritura de las claves de texto básicas
- **WHEN** se ejecuta `edit --title`, `--artist` y `--album` sobre un `.ogg`
- **THEN** las claves `TITLE`, `ARTIST` y `ALBUM` MUST contener los valores indicados y MUST poder releerse con un lector independiente

#### Scenario: Escritura del año y de la fecha en claves separadas
- **WHEN** se ejecuta `edit --year 2026 --date 2026-10-13`
- **THEN** `YEAR` MUST contener `2026` y `DATE` MUST contener `2026-10-13`, y ninguno de los dos MUST haber sobrescrito al otro

#### Scenario: Escritura de los créditos y las ordenaciones
- **WHEN** se ejecuta `edit --composer`, `--copyright`, `--title-sort`, `--artist-sort`, `--album-sort` y `--subtitle`
- **THEN** MUST escribirse respectivamente en `COMPOSER`, `COPYRIGHT`, `TITLESORT`, `ARTISTSORT`, `ALBUMSORT` y `SUBTITLE`

#### Scenario: Escritura de la compilación
- **WHEN** se ejecuta `edit --compilation` sobre un `.ogg`
- **THEN** la clave `COMPILATION` MUST quedar escrita con el valor `1`

#### Scenario: Número de pista y de temporada
- **WHEN** se ejecuta `edit --track 7/12 --season 1/2`
- **THEN** `TRACKNUMBER` MUST contener `7`, `TRACKTOTAL` MUST contener `12`, `DISCNUMBER` MUST contener `1` y `DISCTOTAL` MUST contener `2`

#### Scenario: Múltiples artistas unidos con el separador `"; "`
- **WHEN** se indica `--artist` más de una vez
- **THEN** la clave `ARTIST` MUST contener todos los valores unidos por `"; "`, igual que en MP3 y en MP4

### Requirement: Campos sin equivalente en Vorbis

`--url` y `--original-artist` MUST NOT escribirse en ninguna clave de un archivo Ogg: la tabla de mapeo de `lofty` 0.25.4 (`VORBIS_MAP`) no tiene clave para ninguno de los dos. El CLI MUST omitirlos, MUST avisar de la omisión por stderr en español, MUST NOT abortar la edición y MUST NOT escribir una clave equivalente inventada (ni `WEBSITE`, ni `ORIGINALARTIST`, ni ninguna otra) para representarlos.

#### Scenario: Omisión avisada de la URL y del artista original
- **WHEN** se ejecuta `edit --url` (o `--original-artist`) sobre un `.ogg`
- **THEN** el CLI MUST terminar con código de salida cero, MUST avisar por stderr de que ese campo se omite en OGG y el archivo MUST NOT recibir ninguna clave nueva por ello

#### Scenario: La omisión no impide el resto de la edición
- **WHEN** se ejecuta `edit --title "X" --url "https://ejemplo.test"` sobre un `.ogg`
- **THEN** `TITLE` MUST quedar escrito, el aviso MUST salir por stderr y el resto de la edición MUST aplicarse

#### Scenario: El campo omitido MUST NOT reaparecer en la visualización
- **WHEN** se ejecuta `show` sobre ese mismo archivo
- **THEN** el campo omitido MUST NOT aparecer en la salida

### Requirement: Letra en la clave `LYRICS` sin código de idioma

La letra indicada con `--lyrics` MUST escribirse en la clave `LYRICS`. El contenedor no guarda un código de idioma para la letra, así que a diferencia del frame `USLT` de MP3 la letra MUST NOT llevar idioma asociado, y esa pérdida conocida MUST quedar documentada como degradación. Al visualizar, el CLI SHALL mostrar la letra escrita en `LYRICS` y SHALL mostrar además la de `UNSYNCEDLYRICS` si esa clave estuviera presente, para no ocultar la letra escrita por otras herramientas.

#### Scenario: Escritura de la letra
- **WHEN** se ejecuta `edit --lyrics` sobre un `.ogg`
- **THEN** la letra MUST quedar en `LYRICS`

#### Scenario: Visualización de la letra
- **WHEN** se ejecuta `show` sobre un `.ogg` con `LYRICS`
- **THEN** la letra MUST mostrarse en la salida

#### Scenario: Letra ajena en `UNSYNCEDLYRICS`
- **WHEN** el archivo lleva la letra en `UNSYNCEDLYRICS` (escrita por otra herramienta)
- **THEN** `show` MUST mostrarla también y MUST NOT informar de que no hay letra

### Requirement: Carátula en `METADATA_BLOCK_PICTURE` con el tipo MIME de la extensión

La carátula indicada con `--cover` MUST escribirse como imagen en la clave `METADATA_BLOCK_PICTURE`, con el tipo MIME derivado de la extensión del archivo de imagen (`image/jpeg` para `.jpg` y `.jpeg`, `image/png` para `.png` y `image/webp` para `.webp`) y con tipo de imagen carátula frontal. Como `METADATA_BLOCK_PICTURE` no tiene una lista de tipos de imagen admitidos, el CLI MUST NOT degradar el tipo MIME de una carátula WebP, MUST NOT avisar de que no será visible y MUST NOT omitir la carátula por su formato: a diferencia de `covr` en MP4, una carátula WebP MUST quedar escrita con su tipo MIME real. Cuando los bytes de la imagen no permitan determinar sus dimensiones, el CLI MUST escribir igualmente la carátula (las dimensiones pueden quedar a cero) y MUST NOT abortar la edición. Una extensión de imagen no soportada MUST seguir rechazándose con un mensaje en español **antes** de escribir el archivo, sin dejar el archivo a medias.

#### Scenario: Carátula JPEG y PNG
- **WHEN** se ejecuta `edit --cover` con un `.jpg` o un `.png`
- **THEN** la imagen MUST quedar en `METADATA_BLOCK_PICTURE` con el MIME `image/jpeg` o `image/png` respectivamente

#### Scenario: Carátula WebP sin degradación
- **WHEN** se ejecuta `edit --cover` con un `.webp`
- **THEN** la carátula MUST escribirse con MIME `image/webp`, MUST NOT emitirse ningún aviso de degradación y la carátula MUST releerse como WebP

#### Scenario: Imagen cuyas dimensiones no se pueden determinar
- **WHEN** la imagen indicada es válida pero demasiado corta para inferir sus dimensiones
- **THEN** la carátula MUST escribirse igualmente y la edición MUST NOT fallar

#### Scenario: Extensión de imagen no soportada
- **WHEN** se ejecuta `edit --cover` con una extensión no soportada
- **THEN** el CLI MUST terminar con código de salida distinto de cero, MUST avisar en español y MUST NOT haber modificado el archivo

### Requirement: El comentario obligatorio MUST NOT contarse como metadato

El comentario de Vorbis es obligatorio en el contenedor, así que un archivo Ogg sin ningún campo de usuario sigue teniendo comentario (con su vendor y, si la herramienta que lo creó lo escribió, una clave `ENCODER`). El CLI MUST NOT mostrar el vendor ni `ENCODER` como metadatos del archivo, y cuando no haya ningún campo de la tabla de mapeo el CLI MUST informar de que no se encontraron metadatos y MUST terminar con código de salida cero.

#### Scenario: Visualización de un Ogg sin campos de usuario
- **WHEN** se ejecuta `show` sobre un `.ogg` recién creado que solo tiene el comentario con su vendor y su `ENCODER`
- **THEN** el CLI MUST avisar de que no se encontraron metadatos, MUST terminar con código de salida cero y MUST NOT imprimir el vendor ni el contenido de `ENCODER`

#### Scenario: Visualización con campos presentes
- **WHEN** se ejecuta `show` sobre un `.ogg` con campos de la tabla de mapeo
- **THEN** MUST mostrar esos campos con los mismos nombres de campo que usa hoy en MP3 y en MP4, y MUST NOT mostrar ninguna clave ajena al mapeo

### Requirement: `remove` sobre Ogg Vorbis

El subcomando `remove <campo>` MUST aceptar los mismos nombres, en inglés y en español, que ya acepta en MP3 y en MP4 (misma tabla de alias), y MUST eliminar la clave correspondiente. `remove --track` y `remove --season` MUST eliminar el número y el total **como una unidad**. `remove --all` MUST vaciar el comentario: MUST eliminar todas las claves de usuario y todas las carátulas y MUST preservar el vendor, porque el comentario no se puede eliminar del contenedor; el audio MUST quedar intacto y el archivo resultante MUST seguir siendo reproducible. El CLI MUST NOT dejar el archivo inutilizable ni MUST borrar el contenedor.

#### Scenario: Eliminación de un campo
- **WHEN** se ejecuta `remove title`
- **THEN** la clave `TITLE` MUST desaparecer y el resto de las claves MUST conservarse

#### Scenario: Eliminación del número y del total como unidad
- **WHEN** se ejecuta `remove track` sobre un archivo con `TRACKNUMBER` y `TRACKTOTAL`
- **THEN** ambas claves MUST desaparecer

#### Scenario: Eliminación de la carátula
- **WHEN** se ejecuta `remove cover`
- **THEN** `METADATA_BLOCK_PICTURE` MUST desaparecer y MUST quedar registrado qué carátula se eliminó, igual que en MP3 y en MP4

#### Scenario: `remove --all` vacía el comentario y preserva el vendor
- **WHEN** se ejecuta `remove --all` sobre un `.ogg` con campos
- **THEN** todas las claves de usuario y las carátulas MUST desaparecer, el vendor MUST conservarse y el archivo MUST seguir siendo reproducible

#### Scenario: Nombre de campo desconocido
- **WHEN** se ejecuta `remove` con un nombre que no está en la tabla de alias
- **THEN** el CLI MUST avisar en español y MUST NOT modificar el archivo

#### Scenario: Nombres en español
- **WHEN** se ejecuta `remove` con `título`, `artista` o `carátula`
- **THEN** MUST comportarse igual que con sus equivalentes en inglés

### Requirement: Preservación de las claves ajenas y del vendor al editar

Al editar, el CLI MUST conservar intactas las claves que no gestiona (`ENCODER`, `MUSICBRAINZ_*` o cualquier otra escrita por otra herramienta) y MUST conservar el vendor del comentario. El CLI MUST NOT reescribir el archivo cuando `edit` se invoca sin ningún flag: en ese caso el archivo MUST quedar byte a byte idéntico.

#### Scenario: Claves de terceros conservadas
- **WHEN** un `.ogg` lleva `ENCODER` y `MUSICBRAINZ_TRACKID` escritos por otra herramienta y se ejecuta `edit --title`
- **THEN** las dos claves MUST seguir presentes con su valor original y `TITLE` MUST quedar escrito

#### Scenario: El vendor se conserva
- **WHEN** se edita un `.ogg`
- **THEN** el vendor del comentario MUST ser el mismo que tenía el archivo antes de la edición

#### Scenario: `edit` sin ningún flag no reescribe el archivo
- **WHEN** se ejecuta `edit` sin ningún flag sobre un `.ogg`
- **THEN** el contenido del archivo MUST quedar byte a byte idéntico
