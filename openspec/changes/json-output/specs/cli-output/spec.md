# Spec Delta

## Purpose

El contrato de salida existente se extiende al modo JSON de `show` sin cambiar su esencia: toda salida de datos a stdout MUST pasar por `outln!` (pipe cerrado → exit 0 sin pánico), los avisos y errores MUST seguir yendo por stderr, y el JSON —que es la única salida de datos del modo `--json`— MUST ser un único documento plano sin ANSI. El matiz nuevo: en modo `--json`, el caso «archivo sin tags» (MP3 sin ID3, MP4 sin ilst) MUST NO emitir el aviso por stderr (el aviso ⚠️ es para humanos; el consumidor del JSON es una máquina y el stderr queda limpio salvo errores reales).

## MODIFIED Requirements

### Requirement: El cierre del pipe por el consumidor MUST NOT ser un error

El contrato de BrokenPipe se confirma para el modo `--json`: el JSON se escribe con `outln!`, de modo que un consumidor que cierra el extremo de lectura MUST terminar con exit 0, sin pánico y sin mensajes por stderr, igual que el modo legible.

#### Scenario: Pipe cerrado durante la salida JSON
- **WHEN** el consumidor cierra el extremo de lectura mientras `show --json` escribe su línea
- **THEN** el CLI MUST terminar con exit 0, MUST NOT emitir nada por stderr y MUST NOT generar un pánico

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

En modo legible, los avisos (p. ej. «⚠️ No se encontraron tags ID3 en …») MUST seguir saliendo por stderr con exit 0, sin cambios. En modo JSON, ese aviso concreto de «sin tags» MUST NO emitirse: el caso se representa con `"tags": null` en el propio JSON y el stderr MUST quedar vacío (salvo errores reales, que siguen yendo por stderr con exit ≠ 0). Los avisos de degradación de `edit` (`--url`, `--original-artist`, etc.) no cambian: siguen por stderr en cualquier modo.

#### Scenario: Aviso de sin tags en modo legible (sin cambios)
- **WHEN** se ejecuta `id3cli show` (sin `--json`) sobre un MP3 sin tags
- **THEN** el aviso en español MUST escribirse por stderr y el exit code MUST ser 0, exactamente como antes del change

#### Scenario: Sin aviso de sin tags en modo JSON
- **WHEN** se ejecuta `id3cli show --json` sobre un MP3 sin tags
- **THEN** el stdout MUST contener el JSON con `"tags": null`, el stderr MUST estar vacío y el exit code MUST ser 0

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