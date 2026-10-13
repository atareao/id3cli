# Spec Delta

## Purpose

Definir la lectura y la escritura de metadatos estilo iTunes (átomos `ilst`) en contenedores MP4 (`.m4a`, `.mp4` y `.m4b`), con el mapeo de cada flag del CLI a su átomo, las reglas de colisión y de omisión de campos, y las limitaciones conocidas del contenedor. El alcance son los contenedores MP4 (`.m4a`, `.mp4` y `.m4b`) que contengan o admitan un bloque `ilst`, con independencia del brand declarado en el átomo `ftyp`.

## ADDED Requirements

### Requirement: Mapeo de los flags del CLI a los átomos `ilst`

El CLI SHALL escribir cada flag de `edit` en el átomo MP4 que le corresponde: `--title` en `©nam`, `--artist` en `©ART`, `--album` en `©alb`, `--album-artist` en `aART`, `--date` en `©day`, `--year` en `©day` (el mismo átomo que `--date`, con la colisión resuelta por el requisito «Colisión de `--year` y `--date` en el átomo `©day`»), `--genre` en `©gen`, `--track` en `trkn`, `--season` en `disk`, `--composer` en `©wrt`, `--copyright` en `cprt`, `--lyrics` en `©lyr`, `--cover` en `covr`, `--compilation` en `cpil`, `--title-sort` en `sonm`, `--artist-sort` en `soar`, `--album-sort` en `soal` y `--subtitle` en el átomo freeform `----:com.apple.iTunes:SUBTITLE`. El CLI MUST NOT escribir ningún otro átomo para representar estos campos.

#### Scenario: Escritura de los átomos de texto básicos
- **WHEN** se ejecuta `edit` con `--title`, `--artist`, `--album`, `--album-artist` y `--genre` sobre un `.m4a`
- **THEN** el archivo MUST contener los átomos `©nam`, `©ART`, `©alb`, `aART` y `©gen` con los valores indicados

#### Scenario: Escritura de la numeración
- **WHEN** se ejecuta `edit --track 7 --season 2` sobre un `.m4a`
- **THEN** el archivo MUST contener el átomo `trkn` con la pista 7 y el átomo `disk` con el disco/temporada 2

#### Scenario: Escritura de la compilación
- **WHEN** se ejecuta `edit --compilation` sobre un `.m4a`
- **THEN** el archivo MUST contener el átomo `cpil` marcado como compilación

#### Scenario: Escritura de los créditos y las ordenaciones
- **WHEN** se ejecuta `edit` con `--composer`, `--copyright`, `--title-sort`, `--artist-sort` y `--album-sort` sobre un `.m4a`
- **THEN** el archivo MUST contener los átomos `©wrt`, `cprt`, `sonm`, `soar` y `soal` con los valores indicados

#### Scenario: El subtítulo se escribe como átomo freeform
- **WHEN** se ejecuta `edit --subtitle "Descripción corta"` sobre un `.m4a`
- **THEN** el archivo MUST contener el átomo `----:com.apple.iTunes:SUBTITLE` con ese valor

#### Scenario: Átomos no indicados no se tocan
- **WHEN** se ejecuta `edit --title "Nuevo"` sobre un `.m4a` que ya tenía artista, álbum y carátula
- **THEN** los átomos `©ART`, `©alb` y `covr` MUST permanecer intactos

### Requirement: Creación del árbol de metadatos `moov.udta.meta.ilst`

Cuando el archivo MP4 no tenga átomo de metadatos, el CLI MUST crear la estructura `moov.udta.meta.ilst` necesaria para alojar los átomos escritos, MUST preservar el resto de la estructura del archivo (en especial el átomo `moov` y sus pistas de audio) y MUST NOT corromper ni reordenar los átomos preexistentes.

#### Scenario: MP4 sin metadatos previos
- **WHEN** se ejecuta `edit --title "Prueba"` sobre un `.m4a` recién generado que no tiene átomo `ilst`
- **THEN** el archivo resultante MUST contener la ruta `moov.udta.meta.ilst` con el átomo `©nam` y el audio MUST seguir siendo reproducible

#### Scenario: MP4 con otros átomos preexistentes
- **WHEN** se edita un `.m4a` que ya contiene átomos de metadatos ajenos al CLI
- **THEN** los átomos preexistentes MUST conservarse sin cambios y solo MUST añadirse o modificarse los átomos correspondientes a los flags indicados

#### Scenario: Estructura de átomos inválida o truncada
- **WHEN** el archivo empieza por `ftyp` pero su estructura de átomos está truncada o es inconsistente
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr un mensaje en español indicando que el archivo MP4 no se pudo leer

#### Scenario: Escritura fallida no deja el archivo a medias
- **WHEN** la escritura de los átomos falla a mitad del proceso
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y el archivo original MUST permanecer legible con su estructura `moov` intacta

### Requirement: Colisión de `--year` y `--date` en el átomo `©day`

Tanto `--year` como `--date` MUST escribirse en el átomo `©day`, que es único. Cuando se indiquen ambos en la misma invocación sobre un archivo MP4, `--date` MUST tener prioridad por ser el valor más específico y el CLI MUST emitir un warning por stderr indicando que el año ha quedado absorbido por la fecha. El CLI MUST NOT abortar por esta colisión.

#### Scenario: Solo `--date`
- **WHEN** se ejecuta `edit --date "2026-10-13"` sobre un `.m4a`
- **THEN** el átomo `©day` MUST contener `2026-10-13` y MUST NOT emitirse ningún warning

#### Scenario: Solo `--year`
- **WHEN** se ejecuta `edit --year 2026` sobre un `.m4a`
- **THEN** el átomo `©day` MUST contener `2026` y MUST NOT emitirse ningún warning

#### Scenario: Ambos flags a la vez
- **WHEN** se ejecuta `edit --year 2026 --date "2026-10-13"` sobre un `.m4a`
- **THEN** el átomo `©day` MUST contener `2026-10-13` y MUST emitirse un warning por stderr indicando que el año ha quedado absorbido por la fecha

#### Scenario: La colisión no afecta al resto de campos
- **WHEN** se ejecuta `edit --year 2026 --date "2026-10-13" --title "X"` sobre un `.m4a`
- **THEN** además del warning, el átomo `©nam` MUST contener `X` y el CLI MUST terminar con código de salida cero

### Requirement: Campos sin equivalente en MP4

Los flags `--url` y `--original-artist` no tienen átomo equivalente en MP4. Cuando se indiquen sobre un archivo MP4, el CLI MUST emitir un warning por stderr y MUST omitir ese campo, MUST NOT escribir ningún átomo freeform para representarlo, MUST NOT abortar la operación y MUST escribir el resto de campos indicados con normalidad.

#### Scenario: `--url` sobre un M4A
- **WHEN** se ejecuta `edit --url "https://artista.com" --title "Cancion"` sobre un `.m4a`
- **THEN** el CLI MUST emitir un warning por stderr indicando que la URL se omite por no tener equivalente en MP4, el átomo `©nam` MUST contener `Cancion` y el CLI MUST terminar con código de salida cero

#### Scenario: `--original-artist` sobre un M4A
- **WHEN** se ejecuta `edit --original-artist "Autor" --album "Disco"` sobre un `.m4a`
- **THEN** el CLI MUST emitir un warning por stderr indicando que el artista original se omite por no tener equivalente en MP4, el átomo `©alb` MUST contener `Disco` y el CLI MUST terminar con código de salida cero

#### Scenario: Ambos campos sin equivalente a la vez
- **WHEN** se ejecuta `edit --url "https://artista.com" --original-artist "Autor"` sobre un `.m4a`
- **THEN** el CLI MUST emitir sendos warnings por stderr, MUST terminar con código de salida cero y el archivo MUST NOT contener ningún átomo nuevo

#### Scenario: Ningún átomo freeform para los campos omitidos
- **WHEN** se inspeccionan los átomos de un `.m4a` después de un `edit` con `--url` y `--original-artist`
- **THEN** el CLI MUST NOT haber creado átomos `----:com.apple.iTunes:URL` ni equivalentes para suplir la omisión

### Requirement: Letra en el átomo `©lyr` sin preservación del idioma

La letra indicada con `--lyrics` MUST almacenarse en el átomo `©lyr`. Dado que `©lyr` no tiene campo de idioma, el código `spa` que en MP3 se guarda en el frame `USLT` no se puede preservar ni escribir, y esta pérdida conocida MUST quedar documentada en el README.

#### Scenario: Escritura de la letra en M4A
- **WHEN** se ejecuta `edit --lyrics "Primera línea\nSegunda línea"` sobre un `.m4a`
- **THEN** el átomo `©lyr` MUST contener el texto completo de la letra

#### Scenario: La letra sobrevive a la lectura
- **WHEN** se ejecuta `show` sobre el `.m4a` que acaba de recibir una letra
- **THEN** la salida MUST mostrar el contenido de la letra escrito

#### Scenario: El idioma no se propaga desde MP3
- **WHEN** se inspecciona el `©lyr` de un `.m4a` escrito por el CLI
- **THEN** el átomo MUST contener el texto de la letra y MUST NOT contener ningún campo ni prefijo que declare el idioma `spa`

#### Scenario: Letra vacía
- **WHEN** se ejecuta `edit --lyrics ""` sobre un `.m4a`
- **THEN** el CLI MUST escribir el átomo `©lyr` con texto vacío y MUST NOT fallar

### Requirement: Carátula en el átomo `covr` y restricciones de formato de imagen

La carátula indicada con `--cover` MUST escribirse en el átomo `covr`. Este átomo solo admite los tipos well-known JPEG (13), PNG (14), GIF (12) y BMP (27), y el CLI MUST escribir JPEG y PNG con sus tipos well-known correspondientes (13 y 14). Cuando la imagen sea WebP, el CLI MUST NOT entregar a `lofty` 0.25.4 ningún tipo MIME que la librería no reconozca: ante un tipo MIME desconocido la escritura de la carátula **devuelve un error** en lugar de degradar el tipo, así que el CLI MUST escribir la carátula dejando el tipo MIME deliberadamente sin establecer —que es el caso que la librería mapea al tipo reservado—, de modo que el átomo `covr` resultante MUST quedar con el tipo de dato reservado (0), que iTunes y macOS NO muestran. Ante una carátula WebP sobre un archivo MP4 el CLI MUST NOT fallar ni abortar, MUST escribir la carátula por esa vía y MUST avisar al usuario por stderr de que esa carátula no será visible en esos reproductores, y MUST NOT abortar la operación ni el resto de la edición. En MP3, el `.webp` sigue funcionando vía `APIC`, según la spec de `mp3-tagging`.

#### Scenario: Carátula JPEG en M4A
- **WHEN** se ejecuta `edit --cover portada.jpg` sobre un `.m4a`
- **THEN** el átomo `covr` MUST contener la imagen con el tipo well-known JPEG y MUST NOT emitirse ningún aviso

#### Scenario: Carátula PNG en M4A
- **WHEN** se ejecuta `edit --cover portada.png` sobre un `.m4a`
- **THEN** el átomo `covr` MUST contener la imagen con el tipo well-known PNG y MUST NOT emitirse ningún aviso

#### Scenario: Carátula WebP en M4A
- **WHEN** se ejecuta `edit --cover portada.webp` sobre un `.m4a`
- **THEN** el CLI MUST escribir la carátula dejando el tipo MIME sin establecer, de modo que el átomo `covr` MUST quedar con el tipo de dato reservado (0), MUST emitir un warning por stderr advirtiendo de que esa carátula no será visible en iTunes ni en macOS, MUST NOT terminar con error ni abortar, y el resto de campos indicados MUST escribirse

#### Scenario: Archivo de carátula inexistente
- **WHEN** se ejecuta `edit --cover ruta/que/no/existe.jpg` sobre un `.m4a`
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr un mensaje en español

#### Scenario: Extensión de imagen ajena a los tipos admitidos
- **WHEN** se ejecuta `edit --cover imagen.tiff` sobre un `.m4a`
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST explicar en español que el formato de imagen no está soportado

### Requirement: Múltiples artistas unidos con el separador `"; "`

Cuando se indica `--artist` más de una vez sobre un archivo MP4, los valores MUST unirse en un único átomo `©ART` usando el separador `"; "`, del mismo modo que hace hoy la edición de MP3. El CLI MUST NOT escribir un átomo por cada artista.

#### Scenario: Varios artistas en un solo átomo
- **WHEN** se ejecuta `edit --artist "Luis Fonsi" --artist "Daddy Yankee"` sobre un `.m4a`
- **THEN** el átomo `©ART` MUST contener exactamente `Luis Fonsi; Daddy Yankee` y MUST existir un único átomo `©ART`

#### Scenario: Un único artista sin separador
- **WHEN** se ejecuta `edit --artist "Queen"` sobre un `.m4a`
- **THEN** el átomo `©ART` MUST contener exactamente `Queen`, sin separador añadido

#### Scenario: Separador consistente entre formatos
- **WHEN** se aplica la misma invocación con varios `--artist` sobre un MP3 y sobre un `.m4a`
- **THEN** el valor leído MUST ser idéntico en ambos, con el mismo separador `"; "`

### Requirement: `remove` sobre contenedores MP4

El subcomando `remove` MUST funcionar sobre archivos MP4 eliminando los átomos correspondientes a los tags indicados, con los mismos nombres en inglés y en español que en MP3, y la opción `--all` MUST eliminar el bloque de metadatos `ilst` completo dejando el audio intacto.

#### Scenario: Eliminación de un tag concreto
- **WHEN** se ejecuta `remove cancion.m4a título` sobre un `.m4a` con átomo `©nam`
- **THEN** el átomo `©nam` MUST quedar eliminado y el resto de átomos MUST permanecer intactos

#### Scenario: Eliminación de la carátula
- **WHEN** se ejecuta `remove cancion.m4a cover` sobre un `.m4a` con átomo `covr`
- **THEN** el átomo `covr` MUST quedar eliminado del archivo

#### Scenario: `remove --all` sobre un M4A
- **WHEN** se ejecuta `remove cancion.m4a --all` sobre un `.m4a` con metadatos
- **THEN** el bloque de metadatos `ilst` MUST quedar eliminado, el audio MUST permanecer intacto y MUST emitirse un mensaje en español confirmándolo

#### Scenario: `remove --all` sobre un M4A sin metadatos
- **WHEN** se ejecuta `remove cancion.m4a --all` sobre un `.m4a` sin átomos de metadatos
- **THEN** el CLI MUST terminar con código de salida cero y MUST avisar en español de que el archivo no tenía metadatos

### Requirement: Visualización de metadatos de un MP4 con `show`

`show` MUST funcionar sobre archivos MP4 mostrando los campos de la tabla de mapeo que estén presentes, con el mismo formato legible y los mismos nombres de campo que emplea hoy sobre MP3, y SHALL avisar sin error cuando el archivo no tenga metadatos.

#### Scenario: MP4 con metadatos
- **WHEN** se ejecuta `show cancion.m4a` sobre un `.m4a` con título, artista, álbum, fecha y carátula
- **THEN** la salida MUST mostrar esos campos con las mismas etiquetas legibles que `show` usa sobre MP3

#### Scenario: MP4 sin metadatos
- **WHEN** se ejecuta `show cancion.m4a` sobre un `.m4a` sin átomo `ilst`
- **THEN** el CLI MUST terminar con código de salida cero y MUST emitir en español un aviso de que no se encontraron metadatos

#### Scenario: Coherencia entre formatos
- **WHEN** se editan los mismos campos en un MP3 y en un `.m4a` y se ejecuta `show` sobre ambos
- **THEN** el conjunto de campos mostrados MUST ser el mismo, con independencia del formato del archivo
