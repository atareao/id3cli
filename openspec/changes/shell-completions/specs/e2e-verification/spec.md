# Spec Delta

## Purpose

La matriz de verificación e2e incorpora la capa de completions: un script dedicado que ejecuta `completions bash|zsh|fish`, valida la sintaxis real del script bash con `bash -n` (oráculo independiente, como `ffprobe`/`jq` lo son para los tags y el JSON), comprueba los marcadores de zsh y fish, los subcomandos presentes, el stderr vacío, el shell no soportado y la falsabilidad. El resto de la matriz (20 tags, roundtrips, degradaciones, capas legible/JSON, remove) no cambia.

## ADDED Requirements

### Requirement: El e2e MUST verificar la capa de completions con `bash -n` como oráculo

El e2e MUST ejecutar `id3cli completions bash` y verificar: (a) exit 0 con stderr vacío; (b) el stdout contiene la subcadena estable `complete -F _id3cli` (sin fijar los flags de la plantilla); (c) el stdout, guardado a fichero, pasa `bash -n` (script bash sintácticamente válido — el intérprete real como oráculo). Para `completions zsh` MUST verificar `#compdef id3cli` y para `completions fish` líneas `complete -c id3cli`, ambas con exit 0 y stderr vacío. MUST verificar que el script bash menciona los subcomandos `show`, `edit`, `remove` y `completions`.

#### Scenario: La capa de completions de bash
- **WHEN** el e2e de completions verifica `id3cli completions bash`
- **THEN** exit 0, stderr vacío, la subcadena estable `complete -F _id3cli` presente en stdout y `bash -n` sobre el stdout guardado devuelve 0

#### Scenario: La capa de completions de zsh y fish
- **WHEN** el e2e de completions verifica `id3cli completions zsh` y `id3cli completions fish`
- **THEN** ambos terminan con exit 0 y stderr vacío, el primero contiene `#compdef id3cli` y el segundo `complete -c id3cli`

### Requirement: El e2e MUST verificar el fallo de un shell no soportado y el cierre del pipe

El e2e MUST ejecutar `id3cli completions powershell` y verificar exit ≠ 0 con stderr conteniendo `bash`, `zsh` y `fish` y stdout vacío. MUST verificar que el cierre del pipe durante `completions bash` es un final normal (exit 0).

#### Scenario: Shell no soportado y pipe cerrado
- **WHEN** el e2e ejecuta `id3cli completions powershell`
- **THEN** el exit code es distinto de 0 y el stderr nombra los tres shells soportados
- **AND WHEN** un consumidor cierra el pipe durante `id3cli completions bash`
- **THEN** el proceso termina con exit 0

### Requirement: La falsabilidad MUST re-demostrarse sobre una aserción de completions

El e2e de completions MUST demostrar que el resumen detecta un fallo real: corrompiendo deliberadamente una expectativa (p. ej. esperar `#compdef id3cli` a la salida de bash) el script termina con exit 1 y nombra la aserción fallida con lo esperado vs lo obtenido; restaurada, termina con exit 0.

#### Scenario: Falsabilidad de la capa de completions
- **GIVEN** una copia temporal del script con una expectativa corrompida
- **WHEN** se ejecuta la copia
- **THEN** termina con exit 1 y el FAIL nombra la aserción y el valor esperado
- **AND WHEN** se ejecuta la versión original
- **THEN** termina con exit 0 y 0 fallidas