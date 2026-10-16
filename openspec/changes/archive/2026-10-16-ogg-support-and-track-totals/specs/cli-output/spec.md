# Spec Delta

## Purpose

La incorporación de OGG añade una tercera ruta de visualización a `show`, de modo que el contrato sobre el cierre del pipe y la separación de avisos por stderr deben cubrirla igual que cubrían MP3 y MP4/M4A. El spec anterior fijaba ese contrato para dos rutas y solo verificaba el aviso de campo omitido sobre M4A, por lo que no decía nada sobre el comportamiento con un OGG.

El escenario de contrato conserva su nombre canónico (`El contrato es idéntico en las dos rutas de visualización`) porque el formato de delta no admite renombrar escenarios: un nombre que desaparece del bloque se interpreta como escenario omitido y `openspec archive` lo rechaza. Es el cuerpo del escenario el que fija el contrato, y ese cuerpo ya cubre las tres rutas de visualización (MP3, MP4/M4A y OGG).

## MODIFIED Requirements

### Requirement: El cierre del pipe por el consumidor MUST NOT ser un error

Si el consumidor de la salida estándar cierra el extremo de lectura antes de que el CLI termine de escribir, el CLI MUST terminar con código de salida **cero**, MUST NOT escribir ningún mensaje por stderr —ni de pánico, ni de aviso, ni la nota de `RUST_BACKTRACE`— y MUST NOT abortar el proceso con un pánico. Este contrato aplica por igual a las tres rutas de visualización (`show` sobre MP3, `show` sobre MP4/M4A y `show` sobre OGG).

#### Scenario: Pipe cerrado antes de leer nada sobre un MP3
- **WHEN** el consumidor cierra el extremo de lectura antes de leer nada y se invoca `show` sobre un archivo MP3
- **THEN** el CLI MUST terminar con código de salida cero, MUST NOT emitir ningún mensaje por stderr y MUST NOT generar un pánico. La prueba cierra el extremo de lectura antes de leer nada a propósito: con pocos metadatos el proceso podría terminar antes de que se cierre el pipe y la prueba no distinguiría nada; cerrando antes de leer, el fallo es determinista

#### Scenario: Pipe cerrado antes de leer nada sobre un M4A
- **WHEN** el consumidor cierra el extremo de lectura antes de leer nada y se invoca `show` sobre un archivo M4A
- **THEN** el CLI MUST terminar con código de salida cero, MUST NOT emitir ningún mensaje por stderr y MUST NOT generar un pánico. La prueba cierra el extremo de lectura antes de leer nada a propósito: con pocos metadatos el proceso podría terminar antes de que se cierre el pipe y la prueba no distinguiría nada; cerrando antes de leer, el fallo es determinista

#### Scenario: Pipe cerrado antes de leer nada sobre un OGG
- **WHEN** el consumidor cierra el extremo de lectura antes de leer nada y se invoca `show` sobre un archivo OGG
- **THEN** el CLI MUST terminar con código de salida cero, MUST NOT emitir ningún mensaje por stderr y MUST NOT generar un pánico. La prueba cierra el extremo de lectura antes de leer nada a propósito: con pocos metadatos el proceso podría terminar antes de que se cierre el pipe y la prueba no distinguiría nada; cerrando antes de leer, el fallo es determinista

#### Scenario: Consumidor que lee solo la primera línea
- **WHEN** se canaliza la salida de `show` a un consumidor que lee únicamente la primera línea y cierra el pipe a continuación (por ejemplo `head -1`)
- **THEN** el CLI MUST terminar con código de salida cero, MUST NOT emitir ningún mensaje por stderr y MUST NOT generar un pánico al escribir las líneas que el consumidor ya no lee

#### Scenario: El contrato es idéntico en las dos rutas de visualización
- **WHEN** se cierra el pipe durante la visualización de un MP3, durante la de un M4A y durante la de un OGG
- **THEN** el comportamiento MUST ser el mismo en los tres casos: código de salida cero, ningún mensaje por stderr y ningún pánico

#### Scenario: Salida consumida por completo
- **WHEN** no hay ningún consumidor que cierre el pipe antes de tiempo (por ejemplo, al redirigir la salida a un archivo)
- **THEN** la salida MUST imprimirse igual que antes y el código de salida MUST seguir siendo cero

### Requirement: Los avisos y los errores MUST seguir saliendo por stderr

Este cambio MUST NOT alterar el flujo de los mensajes de aviso y de error del CLI: siguen saliendo por stderr y los códigos de salida que ya tenían MUST NOT cambiar —cero para los avisos, distinto de cero para los errores—. La salida estándar MUST NOT recibir ningún aviso.

#### Scenario: Un campo omitido con aviso sobre un M4A
- **WHEN** se invoca `edit --url` sobre un archivo M4A, campo que ese formato omite
- **THEN** el aviso MUST escribirse por stderr y el CLI MUST terminar con código de salida cero

#### Scenario: Un campo omitido con aviso sobre un OGG
- **WHEN** se ejecuta `edit --url` sobre un `.ogg`
- **THEN** el aviso en español MUST salir por stderr, la salida estándar MUST NOT contaminarse y la edición del resto de campos MUST aplicarse

#### Scenario: Un formato no soportado
- **WHEN** se invoca un subcomando sobre un archivo de un formato no soportado
- **THEN** el mensaje en español MUST escribirse por stderr y el CLI MUST terminar con código de salida distinto de cero, igual que antes del cambio

#### Scenario: Los avisos MUST NOT contaminar la salida estándar
- **WHEN** el CLI emite un aviso o un error
- **THEN** ningún aviso MUST aparecer en la salida estándar y el mensaje MUST quedar confinado a stderr
