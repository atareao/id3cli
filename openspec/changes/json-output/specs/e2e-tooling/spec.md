# Spec Delta

## Purpose

El tooling e2e gana la verificación de la salida JSON: el runner común `e2e_lib.sh` incorpora aserciones `check_json_*` que ejecutan `show --json` y validan el documento con `jq` —parser independiente, disponible en el entorno (v1.8.2)— como oráculo, igual que `ffprobe` lo es para los tags; `jq` se suma a las herramientas obligatorias que `e2e_setup` comprueba. Cada script por extensión incorpora una etapa JSON tras el show normal. El resto de contratos (temporal no versionado, `--bin`/`--keep`, resumen PASS/FAIL, `set -euo pipefail`, español) no cambia.

## MODIFIED Requirements

### Requirement: El runner común `e2e_lib.sh` MUST exponer las funciones de verificación y el resumen PASS/FAIL

El runner común SHALL exponer, además de las funciones existentes, las aserciones JSON: `check_json_valid` (ejecuta `show --json`, valida `jq -e . >/dev/null` y comprueba que la salida NO contiene bytes ANSI), `check_json_format <formato>` (el `.formato` coincide), `check_json_field <ruta-jq> <valor>` (`jq -r` sobre la ruta y `grep -aF` del valor contra la salida) y `check_json_null <ruta-jq>` (la ruta extrae `null`). Todas MUST registrar el resultado en el recuento del resumen y MUST devolver 0 (el fallo se mide por el contador), igual que las aserciones existentes.

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

### Requirement: La ejecución MUST transcurrir bajo temporal no versionado con limpieza garantizada

Todo el material generado (audio y carátulas) SHALL crearse bajo un directorio temporal creado con `mktemp -d` dentro de `/tmp/opencode/e2e/<ext>/` y SHALL eliminarse al terminar mediante `trap`, salvo que se indique `--keep`. El binario MUST construirse una sola vez por ejecución del runner (no una vez por script) para no sobrecargar. Los scripts SHALL declarar `set -euo pipefail` y SHALL emitir los avisos y errores en español. La verificación JSON añade `jq` a las herramientas externas obligatorias: `e2e_setup` SHALL comprobar la presencia de `ffmpeg`, `ffprobe` y `jq` en el PATH y MUST fallar con exit 1 y un mensaje en español nombrando la herramienta ausente si falta alguna, sin dejar archivos huérfanos en el temporal.

#### Scenario: Limpieza automática del temporal
- **WHEN** un script termina (con éxito o con fallo) sin `--keep`
- **THEN** el directorio temporal de su ejecución MUST haber sido eliminado por el `trap`, con independencia del código de salida.

#### Scenario: El binario se construye una única vez
- **WHEN** se ejecuta `just e2e` sin `--bin`
- **THEN** `cargo build` MUST ejecutarse una sola vez en toda la ejecución conjunta (el runner reutiliza el binario resultante para los cuatro formatos).

#### Scenario: Ausencia de herramientas obligatorias
- **WHEN** `ffmpeg`, `ffprobe` o `jq` no están disponibles en el PATH
- **THEN** el script MUST fallar con exit 1 y un mensaje en español indicando la herramienta ausente, sin dejar archivos huérfanos en el temporal.

