# Spec Delta

## Purpose

Fijar qué debe quedar en el archivo MP3 cuando `remove` elimina el **último campo** del tag, caso sobre el que la spec anterior de `mp3-tagging` guardaba silencio: el único requisito que hablaba de borrado físico era el de `--all`, y no decía nada sobre el resultado de `remove <campo>` cuando el tag se queda sin ningún campo.

## ADDED Requirements

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
