# Spec Delta

## Purpose

El requirement de creación del árbol `moov.udta.meta.ilst` no exigía el hijo `hdlr`, y `lofty` 0.25.4 reconstruye `meta` sin ese hijo cuando reescribe un archivo cuyo `ilst` fue eliminado con `remove --all`: las etiquetas quedan legibles por `lofty` y por `id3cli show`, pero invisibles para los demuxers externos (ffprobe/ffmpeg, VLC, QuickTime). Este delta añade a ese requirement la obligación de que toda escritura MP4 del CLI produzca un árbol `meta` con su `hdlr` válido y verifica la visibilidad por un lector de terceros tras cualquier secuencia de operaciones.

## MODIFIED Requirements

### Requirement: Creación del árbol de metadatos `moov.udta.meta.ilst`

Cuando el archivo MP4 no tenga átomo de metadatos, el CLI MUST crear la estructura `moov.udta.meta.ilst` necesaria para alojar los átomos escritos, MUST preservar el resto de la estructura del archivo (en especial el átomo `moov` y sus pistas de audio) y MUST NOT corromper ni reordenar los átomos preexistentes. El árbol `meta` resultante de CUALQUIER escritura del CLI MUST incluir su hijo `hdlr` (handler de tipo de medio) válido: sin ese átomo, los demuxers externos (ffmpeg/ffprobe, VLC, QuickTime) no parsean el `ilst` y las etiquetas resultan invisibles aunque existan en el archivo. Además de crear el árbol en archivos sin metadatos, el CLI MUST garantizar el `hdlr` cuando reescribe un archivo cuyo `ilst` fue eliminado previamente con `remove --all`, de modo que tras `edit` posterior las etiquetas MUST ser visibles por un lector de terceros (`ffprobe format_tags`).

#### Scenario: MP4 sin metadatos previos

- **WHEN** se ejecuta `edit --title "Prueba"` sobre un `.m4a` recién generado que no tiene átomo `ilst`
- **THEN** el archivo resultante MUST contener la ruta `moov.udta.meta.ilst` con el átomo `©nam`, el árbol `moov.udta.meta` MUST contener su hijo `hdlr`, y el audio MUST seguir siendo reproducible

#### Scenario: MP4 con otros átomos preexistentes

- **WHEN** se edita un `.m4a` que ya contiene átomos de metadatos ajenos al CLI
- **THEN** los átomos preexistentes MUST conservarse sin cambios y solo MUST añadirse o modificarse los átomos correspondientes a los flags indicados

#### Scenario: Estructura de átomos inválida o truncada

- **WHEN** el archivo empieza por `ftyp` pero su estructura de átomos está truncada o es inconsistente
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr un mensaje en español indicando que el archivo MP4 no se pudo leer

#### Scenario: Escritura fallida no deja el archivo a medias

- **WHEN** la escritura de los átomos falla a mitad del proceso
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y el archivo original MUST permanecer legible con su estructura `moov` intacta

#### Scenario: Las etiquetas son visibles para un lector de terceros tras `remove --all` + `edit`

- **WHEN** se ejecuta `edit --title "Antes" --album "AlbumAntes"`, después `remove --all` y finalmente `edit --title "Despues" --album "AlbumDespues"` sobre el mismo `.m4a`
- **THEN** `id3cli show` MUST mostrar los campos `Despues`/`AlbumDespues`, `ffprobe -show_entries format_tags` MUST listar `title=Despues` y `album=AlbumDespues` además de los brands, el árbol `moov.udta.meta` del archivo resultante MUST contener su hijo `hdlr`, y el audio MUST seguir siendo reproducible

#### Scenario: El árbol `meta` reescrito siempre lleva `hdlr`

- **WHEN** se inspecciona el árbol de átomos de un `.m4a` reescrito por el CLI en cualquier secuencia de operaciones (`edit` sobre archivo limpio, `edit` sobre archivo con metadatos, `remove --all` + `edit`)
- **THEN** todo átomo `meta` del archivo resultante MUST contener su hijo `hdlr`, y MUST NOT existir ningún `meta` sin `hdlr` hijo