# Spec Delta

## Purpose

La detección de formato pasa de dos backends a tres: se incorpora OGG a la lista de formatos soportados, de modo que el reconocimiento por magic bytes debe cubrir la cabecera de página `OggS` y distinguir Ogg Vorbis de otros códecs alojados en el mismo contenedor. El spec anterior no lo cubría porque trataba `OggS` como un magic byte rechazado y porque el mensaje de error enumeraba únicamente MP3 y MP4 como formatos aceptados.

## MODIFIED Requirements

### Requirement: Detección del formato por magic bytes

El CLI MUST determinar el formato del archivo leyendo su contenido y MUST NOT basarse en la extensión del nombre. El CLI SHALL reconocer MP3 cuando los primeros bytes del archivo sean la cabecera `ID3` o una cabecera de frame MPEG. Para aceptar la sincronía de frame MPEG no basta con que el primer byte sea `0xFF` con los tres bits altos del segundo byte a `1`: el CLI SHALL validar además que la cabecera MPEG sea plausible, esto es, que los campos de versión y de layer NO tengan el valor reservado, que el índice de bitrate no sea el valor inválido y que el índice de frecuencia de muestreo no sea el reservado. El CLI SHALL reconocer MP4 cuando exista un átomo `ftyp` en el offset 4 del archivo, con independencia del brand declarado, que MUST NOT usarse como criterio de aceptación. El CLI SHALL reconocer Ogg Vorbis cuando el archivo empiece por el patrón de captura `OggS`, la cabecera de página esté completa, su versión sea `0` y el primer paquete empiece por el identificador de códec `\x01vorbis`. El primer paquete empieza en el offset `27 + n`, donde `n` es el número de segmentos declarado en el byte 26 de la cabecera. El reconocimiento de OGG MUST NOT depender de la extensión del nombre del archivo.

#### Scenario: Detección de MP3 por cabecera ID3
- **WHEN** se invoca cualquier subcomando sobre un archivo cuyos tres primeros bytes son `ID3`
- **THEN** el CLI MUST seleccionar el backend MP3, con independencia del nombre y de la extensión del archivo

#### Scenario: Detección de MP3 por sincronía de frame MPEG
- **WHEN** se procesa un archivo sin cabecera `ID3` cuyos primeros bytes forman una cabecera de frame MPEG válida (`0xFF` seguido de un byte con los bits 5, 6 y 7 a `1`, con campos de versión, layer, bitrate y frecuencia de muestreo no reservados ni inválidos)
- **THEN** el CLI MUST seleccionar el backend MP3 y MUST NOT informar de un formato no soportado

#### Scenario: Falso positivo de sincronía con cabecera MPEG implausible
- **WHEN** el archivo empieza por `0xFF` seguido de un byte con los bits altos a `1` pero cuyo campo de versión o de layer tiene el valor reservado, o cuyo índice de bitrate es el valor inválido, o cuyo índice de frecuencia de muestreo es el reservado
- **THEN** el CLI MUST NOT reconocer el archivo como MP3 y MUST tratarlo como formato no reconocido, terminando con un código de salida distinto de cero

#### Scenario: Detección de MP4 por átomo ftyp
- **WHEN** los bytes 4 a 7 del archivo son `ftyp`
- **THEN** el CLI MUST seleccionar el backend MP4, sea cual sea el brand declarado

#### Scenario: Detección de MP4 con brand minoritario
- **WHEN** el archivo tiene `ftyp` en el offset 4 y su brand es `M4B `, `avc1`, `iso2`, `dash` o `qt  `
- **THEN** el CLI MUST seleccionar igualmente el backend MP4 y MUST NOT rechazarlo por el brand

#### Scenario: La extensión no determina el formato
- **WHEN** un archivo MP3 se renombra con extensión `.m4a`, o un archivo MP4 se renombra con extensión `.mp3`
- **THEN** el CLI MUST procesarlo según el formato detectado en su contenido, ignorando la extensión del nombre

#### Scenario: Detección de OGG Vorbis
- **WHEN** se invoca cualquier subcomando sobre un archivo cuya primera página es `OggS` de versión 0 y cuyo primer paquete empieza por `\x01vorbis`, con cualquier extensión, incluida `.oga` o ninguna extensión
- **THEN** el CLI MUST seleccionar el backend OGG

#### Scenario: Contenedor Ogg con un códec no soportado
- **WHEN** la primera página es `OggS` válida pero el primer paquete empieza por `OpusHead`, por `Speex   `, por `\x7fFLAC` o por un identificador de vídeo como el de Theora
- **THEN** el CLI MUST rechazar el archivo como formato no soportado, con código de salida distinto de cero y sin modificar el archivo

#### Scenario: Página Ogg incompleta
- **WHEN** el archivo empieza por `OggS` pero no llega a contener la cabecera de página completa o el primer paquete
- **THEN** el CLI MUST informar de que el formato no se pudo determinar, con código de salida distinto de cero

### Requirement: Rechazo explícito de formatos no soportados

Cuando el contenido del archivo no corresponda a ninguno de los formatos soportados (MP3, MP4 u OGG), el CLI MUST terminar con un código de salida distinto de cero, MUST escribir en stderr un mensaje en español que identifique el problema y MUST NOT modificar el archivo. El CLI MUST NOT interpretar un archivo de formato desconocido como MP3 por defecto.

#### Scenario: Formato de audio fuera de alcance
- **WHEN** se invoca `show`, `edit` o `remove` sobre un archivo FLAC (`fLaC`), Opus (`OpusHead`), Speex, WAV (`RIFF`), AIFF (`FORM`) o APE
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr un mensaje en español indicando que ese formato no está soportado

#### Scenario: Contenido no reconocible
- **WHEN** el archivo no empieza por ninguno de los magic bytes conocidos (por ejemplo, un archivo de texto plano)
- **THEN** el CLI MUST terminar con un código de salida distinto de cero, MUST informar de que el formato no se pudo determinar y MUST dejar el archivo sin cambios

#### Scenario: Archivo demasiado corto para decidir el formato
- **WHEN** el archivo tiene menos bytes de los necesarios para contener un átomo `ftyp` en el offset 4 o una cabecera de frame MPEG
- **THEN** el CLI MUST informar de que el formato no se pudo determinar y MUST terminar con un código de salida distinto de cero

#### Scenario: Archivo inexistente
- **WHEN** la ruta indicada no existe en el sistema de archivos
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr el mensaje en español de archivo no encontrado

### Requirement: Mensaje de error de formato en español y accionable

El mensaje que se emite ante un formato no soportado MUST estar en español y SHALL enumerar los formatos soportados por la herramienta, de modo que el usuario pueda saber qué entrada sí se acepta.

#### Scenario: El mensaje enumera los formatos soportados
- **WHEN** se rechaza un archivo por formato no soportado
- **THEN** el mensaje de stderr MUST mencionar explícitamente MP3, MP4 y OGG como los formatos aceptados

#### Scenario: Salida por stdout no se usa para el error
- **WHEN** se rechaza un archivo por formato no soportado
- **THEN** el mensaje de error MUST escribirse en stderr y MUST NOT mezclarse con la salida normal de stdout
