# Spec Delta

## Purpose

El tooling e2e cubre un script nuevo no ligado a formato (`e2e_completions.sh`) que reutiliza las helpers del runner común sin generar audio, y la recipe `just e2e` pasa a ejecutar los 5 formatos + completions en serie con una sola construcción. El resto de contratos del runner (temporal no versionado, `--bin`/`--keep`, resumen PASS/FAIL, `set -euo pipefail`, español) no cambia.

## MODIFIED Requirements

### Requirement: El runner común `e2e_lib.sh` MUST exponer las funciones de verificación y el resumen PASS/FAIL

El runner común SHALL exponer, además de las funciones existentes, las aserciones JSON: `check_json_valid` (ejecuta `show --json`, valida `jq -e . >/dev/null` y comprueba que la salida NO contiene bytes ANSI), `check_json_format <formato>` (el `.formato` coincide), `check_json_field <ruta-jq> <valor>` (`jq -r` sobre la ruta y `grep -aF` del valor contra la salida) y `check_json_null <ruta-jq>` (la ruta extrae `null`). Todas MUST registrar el resultado en el recuento del resumen y MUST devolver 0 (el fallo se mide por el contador), igual que las aserciones existentes. El runner SHALL exponer además las funciones necesarias para un script no ligado a formato: el script SHALL poder usar `e2e_setup`, `e2e_run`, `check_rc`, `check_show_contains` y `e2e_summary` sin generar audio (no hay archivo de audio en la capa de completions), y el chequeo del stderr vacío SHALL hacerse sobre el `$E2E_ERR` capturado, contabilizándose en el resumen.

#### Scenario: El resumen cuenta las aserciones JSON
- **WHEN** un script por extensión ejecuta su etapa JSON
- **THEN** cada `check_json_*` MUST incrementar el contador PASS/FAIL del resumen y el script MUST terminar con exit 0 solo si todas pasan

#### Scenario: La validación usa jq como parser independiente
- **WHEN** `check_json_valid` se ejecuta
- **THEN** MUST validarse el documento con `jq -e .` (un JSON inválido MUST fallar la aserción) y MUST comprobarse la ausencia de bytes ANSI (`grep` del byte ESC)

#### Scenario: La aserción de campo consulta por ruta jq
- **WHEN** `check_json_field '.tags.title' 'TítuloX'` se ejecuta sobre un archivo con ese título
- **THEN** la aserción MUST extraer `jq -r '.tags.title'` y dar por buena la comparación con `grep -aF`

#### Scenario: El resumen cuenta las aserciones
- **WHEN** un script por extensión termina su flujo de verificación
- **THEN** el runner MUST imprimir un resumen final con el número de aserciones ejecutadas y fallidas, y el script MUST terminar con exit 0 si ninguna falló y exit 1 en caso contrario.

#### Scenario: La aserción de `show` grep el valor, no el estilo
- **WHEN** se verifica un tag por `id3cli show` cuyo valor coincide con el esperado
- **THEN** `check_show_contains` MUST dar por buena la aserción grepeando la subcadena del valor (si la salida contuviera bytes ANSI, el grep MUST tratar la salida como binaria, p. ej. `grep -a`), y MUST NO depender de la presencia de códigos de color en la cadena.

#### Scenario: El script de completions suma aserciones al resumen
- **GIVEN** el script `e2e_completions.sh`, que reutiliza las helpers sin `e2e_gen_audio`
- **WHEN** termina su ejecución
- **THEN** MUST imprimir `PASS/FAIL: completions: <N> aserciones, 0 fallidas` y terminar con exit 0

#### Scenario: El stderr vacío se verifica por el fichero capturado
- **WHEN** el script verifica que `id3cli completions bash` no escribe en stderr
- **THEN** la verificación SHALL usar `$E2E_ERR` capturado por `e2e_run` (fichero no vacío → FAIL contabilizado; vacío → PASS)

### Requirement: El `.justfile` MUST exponer recipes `e2e` y una por extensión

El `.justfile` SHALL exponer la recipe `e2e` (ejecuta los cinco scripts por extensión **en serie** Y después `e2e_completions.sh --bin target/debug/id3cli`, deteniéndose en el primer fallo propagando su exit code, con una sola construcción previa `cargo build --locked`) y las recipes `e2e-mp3`, `e2e-m4a`, `e2e-ogg`, `e2e-opus`, `e2e-flac` y `e2e-completions` (un script cada una), con paso opcional de argumentos (`--bin`, `--keep`) al script correspondiente.

#### Scenario: Just e2e ejecuta los cuatro formatos en serie
- **WHEN** se ejecuta `just e2e`
- **THEN** MUST ejecutarse `e2e_mp3.sh`, `e2e_m4a.sh`, `e2e_ogg.sh`, `e2e_opus.sh` y `e2e_flac.sh` en ese orden y, si alguno termina con exit distinto de cero, la recipe MUST detenerse y propagar ese código

#### Scenario: Just por extensión con argumentos
- **WHEN** se ejecuta `just e2e-opus --bin ./target/release/id3cli --keep`
- **THEN** MUST ejecutarse `scripts/e2e/e2e_opus.sh` recibiendo `--bin ./target/release/id3cli --keep`.

#### Scenario: Just por extensión con FLAC
- **WHEN** se ejecuta `just e2e-flac --bin ./target/release/id3cli --keep`
- **THEN** MUST ejecutarse `scripts/e2e/e2e_flac.sh` recibiendo `--bin ./target/release/id3cli --keep`.

#### Scenario: Just e2e cubre los seis scripts
- **WHEN** se ejecuta `just e2e`
- **THEN** MUST ejecutarse `e2e_mp3.sh`, `e2e_m4a.sh`, `e2e_ogg.sh`, `e2e_opus.sh`, `e2e_flac.sh` y, después, `e2e_completions.sh --bin target/debug/id3cli` en ese orden y, si alguno termina con exit distinto de cero, la recipe MUST detenerse y propagar ese código

#### Scenario: Just de completions con argumentos
- **WHEN** se ejecuta `just e2e-completions --bin ./target/release/id3cli --keep`
- **THEN** MUST ejecutarse `scripts/e2e/e2e_completions.sh` recibiendo `--bin ./target/release/id3cli --keep`