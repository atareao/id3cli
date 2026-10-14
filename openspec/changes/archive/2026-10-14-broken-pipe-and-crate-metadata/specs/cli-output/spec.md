# Spec Delta

## Purpose

El spec de `cli-output` no decía nada sobre qué debe pasar cuando la salida estándar deja de tener consumidor: la escritura seguía dando por hecho que siempre habría un lector al otro lado del pipe. Hoy, si el consumidor cierra el extremo de lectura antes de que el CLI termine de escribir, `println!` convierte el `ErrorKind::BrokenPipe` en un pánico y el proceso termina con código **101**, que es un fallo del propio CLI y no un error del archivo. Este delta fija el contrato que faltaba: el cierre del pipe es un final normal, mientras que un fallo real de escritura sigue siendo un error.

## ADDED Requirements

### Requirement: El cierre del pipe por el consumidor MUST NOT ser un error

Si el consumidor de la salida estándar cierra el extremo de lectura antes de que el CLI termine de escribir, el CLI MUST terminar con código de salida **cero**, MUST NOT escribir ningún mensaje por stderr —ni de pánico, ni de aviso, ni la nota de `RUST_BACKTRACE`— y MUST NOT abortar el proceso con un pánico. Este contrato aplica por igual a las dos rutas de visualización (`show` sobre MP3 y `show` sobre MP4).

#### Scenario: Pipe cerrado antes de leer nada sobre un MP3
- **WHEN** el consumidor cierra el extremo de lectura antes de leer nada y se invoca `show` sobre un archivo MP3
- **THEN** el CLI MUST terminar con código de salida cero, MUST NOT emitir ningún mensaje por stderr y MUST NOT generar un pánico. La prueba cierra el extremo de lectura antes de leer nada a propósito: con pocos metadatos el proceso podría terminar antes de que se cierre el pipe y la prueba no distinguiría nada; cerrando antes de leer, el fallo es determinista

#### Scenario: Pipe cerrado antes de leer nada sobre un M4A
- **WHEN** el consumidor cierra el extremo de lectura antes de leer nada y se invoca `show` sobre un archivo M4A
- **THEN** el CLI MUST terminar con código de salida cero, MUST NOT emitir ningún mensaje por stderr y MUST NOT generar un pánico. La prueba cierra el extremo de lectura antes de leer nada a propósito: con pocos metadatos el proceso podría terminar antes de que se cierre el pipe y la prueba no distinguiría nada; cerrando antes de leer, el fallo es determinista

#### Scenario: Consumidor que lee solo la primera línea
- **WHEN** se canaliza la salida de `show` a un consumidor que lee únicamente la primera línea y cierra el pipe a continuación (por ejemplo `head -1`)
- **THEN** el CLI MUST terminar con código de salida cero, MUST NOT emitir ningún mensaje por stderr y MUST NOT generar un pánico al escribir las líneas que el consumidor ya no lee

#### Scenario: El contrato es idéntico en las dos rutas de visualización
- **WHEN** se cierra el pipe durante la visualización de un MP3 y durante la de un M4A
- **THEN** el comportamiento MUST ser el mismo en ambos casos: código de salida cero, ningún mensaje por stderr y ningún pánico

#### Scenario: Salida consumida por completo
- **WHEN** no hay ningún consumidor que cierre el pipe antes de tiempo (por ejemplo, al redirigir la salida a un archivo)
- **THEN** la salida MUST imprimirse igual que antes y el código de salida MUST seguir siendo cero

### Requirement: Un fallo de escritura distinto del pipe cerrado MUST seguir siendo un error

Si la escritura en la salida estándar falla por una causa distinta del pipe cerrado —por ejemplo `ENOSPC` o cualquier otro error del dispositivo de salida—, el CLI MUST avisar por stderr con un mensaje en español y MUST terminar con código de salida distinto de cero. Tragarse ese error MUST NOT ser aceptable.

#### Scenario: El dispositivo de salida rechaza la escritura
- **WHEN** se invoca `show` redirigiendo la salida estándar a `/dev/full`, que rechaza toda escritura con `ENOSPC`
- **THEN** el CLI MUST escribir por stderr un mensaje en español indicando que la salida no se pudo escribir y MUST terminar con código de salida distinto de cero

#### Scenario: El fallo de escritura MUST NOT tocar el archivo de audio
- **WHEN** la escritura en la salida estándar falla por una causa distinta del pipe cerrado
- **THEN** el archivo de audio MUST quedar byte a byte intacto y MUST NOT modificarse por causa del fallo de salida

#### Scenario: El mensaje de error MUST distinguirse del pánico
- **WHEN** se produce un fallo de escritura distinto del pipe cerrado
- **THEN** el mensaje MUST ir por stderr precedido de `Error:` y MUST NOT contener el texto `panicked at` ni la nota de `RUST_BACKTRACE`, y el código de salida MUST ser distinto de cero y MUST NOT ser 101

### Requirement: Los avisos y los errores MUST seguir saliendo por stderr

Este cambio MUST NOT alterar el flujo de los mensajes de aviso y de error del CLI: siguen saliendo por stderr y los códigos de salida que ya tenían MUST NOT cambiar —cero para los avisos, distinto de cero para los errores—. La salida estándar MUST NOT recibir ningún aviso.

#### Scenario: Un campo omitido con aviso sobre un M4A
- **WHEN** se invoca `edit --url` sobre un archivo M4A, campo que ese formato omite
- **THEN** el aviso MUST escribirse por stderr y el CLI MUST terminar con código de salida cero

#### Scenario: Un formato no soportado
- **WHEN** se invoca un subcomando sobre un archivo de un formato no soportado
- **THEN** el mensaje en español MUST escribirse por stderr y el CLI MUST terminar con código de salida distinto de cero, igual que antes del cambio

#### Scenario: Los avisos MUST NOT contaminar la salida estándar
- **WHEN** el CLI emite un aviso o un error
- **THEN** ningún aviso MUST aparecer en la salida estándar y el mensaje MUST quedar confinado a stderr
