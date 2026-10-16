# shell-completions Specification

## Purpose
Capability NUEVA: contrato del subcomando `id3cli completions <SHELL>` que emite por stdout el script de completions generado en runtime por `clap_complete` 4.5 sobre el Command real del CLI. Cubre los tres shells exactos (bash/zsh/fish), la salida a través de la capa `output.rs`, el error en español para shells no soportados, los marcadores verificables por shell y la no-rotura del resto de subcomandos.

## Requirements

### Requirement: El subcomando `completions` MUST emitir el script de completions para bash, zsh y fish

`id3cli completions <SHELL>` MUST aceptar exactamente `bash`, `zsh` y `fish` (minúsculas). El script MUST generarse en runtime con `clap_complete` 4.5 sobre el Command real del CLI (incluido el propio subcomando `completions`) y MUST escribirse COMPLETO en stdout vía la capa de salida (`print_block`), con exit 0 y stderr vacío. El script es la salida NATIVA del generador: para bash la línea de registro es `complete -F _id3cli -o nosort -o bashdefault -o default id3cli` (la subcadena estable `complete -F _id3cli` es el marcador de verificación; el resto de flags pertenecen a la plantilla de clap_complete y no se fijan).

#### Scenario: Generación para bash
- **GIVEN** un id3cli compilado
- **WHEN** se ejecuta `id3cli completions bash`
- **THEN** el exit code es 0, el stderr está vacío y el stdout contiene la función `_id3cli` y la subcadena estable `complete -F _id3cli` (sin fijar el resto de flags de la plantilla)

#### Scenario: Generación para zsh
- **GIVEN** un id3cli compilado
- **WHEN** se ejecuta `id3cli completions zsh`
- **THEN** el exit code es 0, el stderr está vacío y el stdout contiene `#compdef id3cli` y `_arguments`

#### Scenario: Generación para fish
- **GIVEN** un id3cli compilado
- **WHEN** se ejecuta `id3cli completions fish`
- **THEN** el exit code es 0, el stderr está vacío y el stdout contiene líneas `complete -c id3cli`

#### Scenario: El script refleja la superficie real
- **GIVEN** un id3cli compilado
- **WHEN** se ejecuta `id3cli completions bash`
- **THEN** el stdout menciona los subcomandos `show`, `edit`, `remove` y `completions`

### Requirement: Un shell no soportado MUST fallar en español sin tocar stdout

Cualquier valor de SHELL distinto de `bash`, `zsh` o `fish` (p. ej. `powershell`, `elvish`, vacío) MUST hacer que el proceso termine con exit ≠ 0, MUST emitir por stderr un mensaje de error en español que nombre los shells válidos, y MUST dejar el stdout vacío.

#### Scenario: Shell no soportado
- **GIVEN** un id3cli compilado
- **WHEN** se ejecuta `id3cli completions powershell`
- **THEN** el exit code es distinto de 0, el stderr menciona `bash`, `zsh` y `fish`, y el stdout está vacío

### Requirement: La salida MUST pasar por la capa de salida (pipe cerrado → exit 0)

El script MUST escribirse por la capa de salida del CLI (la misma que `print_line`): si el consumidor cierra el pipe, el proceso MUST terminar con exit 0 (final normal, sin pánico); si la escritura falla por otra causa, el proceso MUST terminar con exit ≠ 0 y mensaje en español por stderr.

#### Scenario: Cierre del pipe durante la generación
- **GIVEN** un consumidor que cierra el stdout durante `id3cli completions bash`
- **THEN** el proceso termina con exit 0, sin pánico y sin escribir nada en stderr

### Requirement: La salida MUST ser solo el script (sin avisos ni emojis)

En modo `completions` el stdout MUST contener ÚNICAMENTE el script de completions: no deben aparecer los avisos ni los emojis del modo legible de `show` (p. ej. «no se encontraron metadatos», ⚠️).

#### Scenario: stdout libre de contaminación del modo legible
- **GIVEN** un id3cli compilado
- **WHEN** se ejecuta `id3cli completions bash`
- **THEN** el stdout no contiene la subcadena «no se encontraron metadatos»
