# track-numbering Specification

## Purpose
Esta capability define cómo `id3cli` interpreta, escribe, muestra y elimina el número de pista y el número de temporada, incluido su total. Cubre la sintaxis aceptada por `--track` y `--season`, la representación nativa del total en cada contenedor (frames ID3v2.4 en MP3, átomos `ilst` en MP4/M4A y claves de comentario Vorbis en OGG Vorbis), su visualización en `show` y su borrado en `remove`. El objetivo es que el par número/total se comporte de forma equivalente en los tres formatos soportados, sin inventar representaciones nuevas y sin degradar la forma sin total ya existente.

## Requirements

### Requirement: El número de pista y de temporada MUST aceptar un número o un par número/total

Los flags `--track` y `--season` MUST aceptar un número (`7`) o un par de número y total separados por una barra (`7/12`), y los dos valores MUST ser enteros sin signo. Cualquier otra forma —entre ellas una barra sin total (`7/`), una barra sin número (`/12`), más de una barra (`7/12/3`), un valor no numérico (`7/x`), un número negativo o una cadena vacía— MUST rechazarse **antes de tocar el archivo**, con un mensaje en español que indique la forma admitida y con código de salida distinto de cero; el archivo MUST quedar byte a byte intacto. La forma sin total MUST seguir comportándose exactamente igual que antes de este cambio.

#### Scenario: Número sin total

- **WHEN** se ejecuta `edit --track 7`
- **THEN** el número 7 MUST escribirse y MUST NOT escribirse ningún total.

#### Scenario: Número con total

- **WHEN** se ejecuta `edit --track 7/12`
- **THEN** el número 7 y el total 12 MUST escribirse.

#### Scenario: Temporada con total

- **WHEN** se ejecuta `edit --season 1/2`
- **THEN** el número de temporada 1 y su total 2 MUST escribirse.

#### Scenario: Forma inválida rechazada sin tocar el archivo

- **WHEN** se ejecuta `edit --track` con `7/`, `/12`, `7/12/3`, `7/x` o una cadena vacía
- **THEN** el CLI MUST terminar con código de salida distinto de cero, MUST avisar en español de la forma admitida y el archivo MUST quedar byte a byte idéntico.

#### Scenario: La forma válida sin total no cambia de comportamiento

- **WHEN** se ejecuta `edit --track 7 --season 2` sobre cualquiera de los tres formatos
- **THEN** el resultado MUST ser el mismo que daba el CLI antes de este cambio.

### Requirement: El total MUST escribirse en la representación nativa de cada contenedor

El total indicado SHALL escribirse en la representación propia de cada formato: en MP3 el frame `TRCK` (y `TPOS` para la temporada) como `n/total`; en MP4 los átomos `trkn` (y `disk`) con su campo de total; y en OGG Vorbis las claves `TRACKTOTAL` (y `DISCTOTAL`) separadas del número. El CLI MUST NOT escribir un total en un formato que no lo admita ni MUST inventar una clave o un campo nuevos para representarlo. Cuando se indique número sin total y el archivo ya tuviera un total, el CLI MUST conservar ese total preexistente en los tres formatos; cuando el archivo no tuviera total, el CLI MUST NOT crear uno.

#### Scenario: Total en MP3

- **WHEN** se ejecuta `edit --track 7/12` sobre un MP3
- **THEN** `TRCK` MUST quedar como `7/12` y MUST releerse con total 12.

#### Scenario: Total en MP4

- **WHEN** se ejecuta `edit --track 7/12` sobre un `.m4a`
- **THEN** el átomo `trkn` MUST quedar con número 7 y total 12.

#### Scenario: Total en OGG

- **WHEN** se ejecuta `edit --track 7/12` sobre un `.ogg`
- **THEN** `TRACKNUMBER` MUST quedar en `7` y `TRACKTOTAL` en `12`.

#### Scenario: Total de temporada en los tres formatos

- **WHEN** se ejecuta `edit --season 1/2`
- **THEN** el total MUST quedar respectivamente en `TPOS` como `1/2`, en el átomo `disk` y en la clave `DISCTOTAL`.

#### Scenario: Total preexistente conservado

- **WHEN** un archivo de cualquiera de los tres formatos ya tiene un total y se ejecuta `edit --track 5`
- **THEN** el número MUST quedar en 5 y el total MUST seguir siendo el que ya estaba.

#### Scenario: Sin total preexistente no se inventa uno

- **WHEN** se ejecuta `edit --track 5` sobre un archivo sin total
- **THEN** MUST escribirse el número y MUST NOT quedar ningún total.

### Requirement: `show` MUST mostrar el total cuando exista

`show` SHALL mostrar el total de pista y el de temporada cuando el archivo los tenga, con el mismo nombre de campo en los tres formatos, y MUST NOT mostrar el campo del total cuando no exista.

#### Scenario: Total tras una edición con total

- **WHEN** se ejecuta `show` tras `edit --track 7/12`
- **THEN** la salida MUST mostrar el número 7 y el total 12.

#### Scenario: Sin total no aparece el campo

- **WHEN** se ejecuta `show` sobre un archivo cuyo número de pista no tiene total
- **THEN** la salida MUST mostrar el número y MUST NOT mostrar ningún campo de total.

### Requirement: `remove` MUST eliminar el número y el total como una unidad

En los tres formatos, `remove` sobre el campo del número de pista o de temporada MUST eliminar el número **y** el total, de modo que MUST NOT quedar un total huérfano sin número.

#### Scenario: Eliminación del par completo de pista

- **WHEN** se ejecuta `remove track` sobre un archivo con número y total
- **THEN** ninguno de los dos MUST permanecer.

#### Scenario: Eliminación del par completo de temporada

- **WHEN** se ejecuta `remove season` sobre un archivo con número de temporada y total
- **THEN** ninguno de los dos MUST permanecer.

#### Scenario: Los tres formatos se comportan igual

- **WHEN** se ejecuta `remove track` sobre un MP3, un `.m4a` y un `.ogg` que tienen número y total
- **THEN** el resultado MUST ser equivalente en los tres.
