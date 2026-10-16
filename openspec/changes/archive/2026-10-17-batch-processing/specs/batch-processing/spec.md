# Spec Delta

## Purpose

Capability NUEVA: contrato del modo batch — los subcomandos `show`, `edit` y `remove` aceptan `<FILE>...` (1..n) y aplican la operación a todos los archivos, con no-rotura estricta N=1 (salidas byte a byte idénticas a las actuales), cabecera legible y NDJSON con N>1, exit agregado (0 solo si todos OK; los fallos se informan por stderr y no abortan el resto).

## ADDED Requirements

### Requirement: Los subcomandos `show`, `edit` y `remove` MUST aceptar uno o más archivos (`<FILE>...`)

`show`, `edit` y `remove` SHALL aceptar `Vec<PathBuf>` (1..n, `num_args = 1..` y `required = true`): la operación de cada subcomando MUST aplicarse a todos los archivos en el orden dado. Cada archivo MUST despacharse a su backend por `detect_supported_format` (formato mixto permitido: p. ej. `edit --title X a.mp3 b.m4a` edita cada uno por su backend y emite por stderr las degradaciones del formato que las sufre). Sin archivos, clap MUST responder con el error estándar de argumento requerido y exit 2.

#### Scenario: Un único archivo conserva la salida actual
- **GIVEN** un id3cli compilado con el change y un MP3 con tags
- **WHEN** se ejecuta `id3cli show <MP3>` sin `--json`
- **THEN** la salida es byte a byte la misma que antes del change (sin cabecera añadida, sin regresión)

#### Scenario: Dos archivos en show legible
- **GIVEN** dos MP3 con tags y un id3cli compilado con el change
- **WHEN** se ejecuta `id3cli show f1.mp3 f2.mp3`
- **THEN** el exit code es 0, el stderr no tiene errores y el stdout contiene la cabecera `=== <ruta f1> ===` y la `=== <ruta f2> ===` con los bloques de ambos archivos

#### Scenario: Edit en batch
- **GIVEN** dos archivos de audio con tags y un id3cli compilado con el change
- **WHEN** se ejecuta `id3cli edit --title X f1 f2`
- **THEN** el exit code es 0 y AMBOS archivos quedan reeditados (verificado releyéndolos)

#### Scenario: Remove en batch
- **GIVEN** dos archivos de audio con tags y un id3cli compilado con el change
- **WHEN** se ejecuta `id3cli remove --all f1 f2`
- **THEN** el exit code es 0 y ambos archivos quedan sin tags

#### Scenario: Sin archivos
- **GIVEN** un id3cli compilado con el change
- **WHEN** se ejecuta `id3cli show` sin argumentos de archivo
- **THEN** clap responde con el error de argumento requerido, el stderr lo indica y el exit code es 2

### Requirement: `show` con varios archivos MUST emitir cabeceras legibles y NDJSON

En modo legible, cada bloque MUST precederse de una cabecera `=== <ruta> ===` (una línea) cuando hay más de un archivo; con un único archivo la cabecera MUST NO emitirse (no-rotura). En modo `--json`, el stdout MUST contener una línea JSON por archivo en el orden dado (NDJSON), donde cada línea es el objeto `ShowJson` actual de ese archivo (mismo esquema `{formato, tags}`); el documento completo MUST poder consumirse con `jq -s`.

#### Scenario: NDJSON con dos archivos
- **GIVEN** dos archivos con tags y un id3cli compilado con el change
- **WHEN** se ejecuta `id3cli show --json f1 f2`
- **THEN** el stdout tiene exactamente 2 líneas JSON, `jq -s 'length'` devuelve 2 y cada objeto conserva su `.formato` y sus tags

#### Scenario: NDJSON con un único archivo
- **GIVEN** un archivo con tags y un id3cli compilado con el change
- **WHEN** se ejecuta `id3cli show --json <archivo>`
- **THEN** el stdout es byte a byte la línea única actual (sin cambio)

### Requirement: El exit code del batch MUST ser 0 solo si todos los archivos se procesan

El batch MUST procesar TODOS los archivos: si alguno falla (ruta inexistente, formato no soportado, error de escritura), los demás MUST procesarse igualmente y, al final, el CLI MUST informar por stderr (mensaje en español con la ruta y el motivo, uno por archivo fallido) y terminar con exit ≠ 0. Solo con todos los archivos procesados con éxito el exit code MUST ser 0. Los avisos (sin tags, degradaciones por formato) MUST conservar su contrato actual por archivo y MUST NOT contar como fallo.

#### Scenario: Un archivo inexistente entre dos buenos
- **GIVEN** dos archivos válidos y una ruta inexistente, y un id3cli compilado con el change
- **WHEN** se ejecuta `id3cli edit --title X f1 INEXISTENTE f2`
- **THEN** el exit code es distinto de 0, el stderr menciona la ruta inexistente en español, y f1 y f2 SÍ quedan editados

#### Scenario: Todos los archivos válidos
- **GIVEN** N archivos válidos
- **WHEN** se ejecuta una operación batch sobre todos
- **THEN** el exit code es 0 y todos quedan procesados