# Spec Delta

## Purpose

Esta capability define los contratos del **tooling de verificación e2e local** de `id3cli`: la ubicación y los nombres de los scripts versionados, el CLI de cada script (`--bin`, `--keep`, códigos de salida), las funciones del runner común fuenteable, las recipes del `justfile` y las restricciones de ejecución (todo bajo temporal no versionado, `set -euo pipefail`, mensajes en español). El objetivo es que cualquiera pueda ejecutar `just e2e` y obtener una verificación de extremo a extremo del binario real —escribir y releer todos los tags y las carátulas en los cuatro formatos soportados— sin sobrecargar el repositorio con archivos de audio ni imágenes generados. Esta capability fija el **cómo** se verifica; la matriz de verificación (qué tags, qué degradaciones, qué aserciones) es la capability `e2e-verification`.

## ADDED Requirements

### Requirement: Los scripts de verificación MUST versionarse en `scripts/e2e/` y los datos sobre los que operan MUST NOT versionarse

Debe existir un runner común fuenteable `scripts/e2e/e2e_lib.sh` y un script de flujo por extensión soportada —`scripts/e2e/e2e_mp3.sh`, `scripts/e2e/e2e_m4a.sh`, `scripts/e2e/e2e_ogg.sh`, `scripts/e2e/e2e_opus.sh`— todos ellos bash y versionados. Los archivos de audio y las imágenes que generan MUST crearse en un directorio temporal de la ejecución y MUST NOT versionarse ni añadirse al repositorio; el repositorio MUST quedar sin binarios ni imágenes generados por el e2e.

#### Scenario: La estructura de scripts existe y está versionada

- **WHEN** se inspecciona el árbol del repositorio tras el cambio
- **THEN** MUST existir `scripts/e2e/e2e_lib.sh` y los cuatro scripts por extensión, y MUST estar bajo control de versiones.

#### Scenario: Los archivos generados no entran en el repositorio

- **WHEN** se ejecuta el e2e completo y se inspecciona `git status`
- **THEN** MUST NOT aparecer ningún archivo de audio ni imagen generados como cambio nuevo, porque todo el material generado MUST vivir en el temporal de la ejecución.

### Requirement: Cada script MUST aceptar `--bin RUTA` y `--keep` y MUST terminar con exit 0 o exit 1

Cada script por extensión SHALL aceptar los flags `--bin RUTA` (usar el binario indicado en lugar de construir) y `--keep` (no limpiar el directorio temporal al terminar). Sin `--bin`, el runner SHALL construir el binario una única vez por ejecución (`cargo build`) y usar `target/debug/id3cli`. El script SHALL terminar con **exit 0** únicamente cuando todas sus aserciones hayan pasado, y con **exit 1** ante cualquier fallo, imprimiendo en ese caso un mensaje claro **en español** que identifique la aserción fallida. Cualquier otra salida (p. ej. un archivo no generable, `ffmpeg` ausente) MUST tratarse también como fallo con exit 1 y mensaje en español.

#### Scenario: Ejecución con el binario por defecto

- **WHEN** se ejecuta `scripts/e2e/e2e_mp3.sh` sin argumentos
- **THEN** el runner MUST construir el binario con `cargo build` (una sola vez), ejecutar la verificación contra `target/debug/id3cli` y terminar con exit 0, o con exit 1 nombrando la aserción fallida.

#### Scenario: Ejecución con un binario indicado

- **WHEN** se ejecuta `scripts/e2e/e2e_m4a.sh --bin ./target/release/id3cli`
- **THEN** MUST usarse ese binario para toda la verificación y MUST NO ejecutarse `cargo build`.

#### Scenario: Conservación del temporal para depuración

- **WHEN** se ejecuta `scripts/e2e/e2e_ogg.sh --keep`
- **THEN** el directorio temporal creado por la ejecución MUST conservarse al terminar, y el script MUST informar al usuario de dónde queda.

#### Scenario: Fallo con identificación de la aserción

- **WHEN** alguna aserción de un script no se cumple durante la ejecución
- **THEN** el script MUST imprimir un mensaje en español con el nombre de la aserción fallida y el valor esperado frente al obtenido, y MUST terminar con exit 1.

### Requirement: El runner común `e2e_lib.sh` MUST exponer las funciones de verificación y el resumen PASS/FAIL

El runner común SHALL ser un script bash fuenteable con `set -euo pipefail` que exponga, como mínimo: `e2e_setup <ext>` (crear temporal con `mktemp -d`, registrar `trap` de limpieza, decidir binario), la generación de audio y de carátulas (JPG y PNG) con `ffmpeg`, `e2e_run <args...>` (ejecutar el binario capturando stdout, stderr y código de salida sin abortar por los errores del CLI), las aserciones `check_show_contains <valor>` (grep en la salida de `show` del **valor**, nunca de etiquetas ANSI), `check_ffprobe_tag <clave> <valor>`, `check_ffprobe_absent <clave>`, `check_cover_roundtrip <imagen>` (extraer el stream de imagen con `ffmpeg -map 0:v:0 -c copy` y comparar por bytes o hash contra la original) y `check_stderr_aviso <subcadena>`; y un resumen final PASS/FAIL con el recuento de aserciones ejecutadas y fallidas. Las funciones de aserción MUST registrar cada resultado para el recuento del resumen.

#### Scenario: El resumen cuenta las aserciones

- **WHEN** un script por extensión termina su flujo de verificación
- **THEN** el runner MUST imprimir un resumen final con el número de aserciones ejecutadas y fallidas, y el script MUST terminar con exit 0 si ninguna falló y exit 1 en caso contrario.

#### Scenario: La aserción de `show` grep el valor, no el estilo

- **WHEN** se verifica un tag por `id3cli show` cuyo valor coincide con el esperado
- **THEN** `check_show_contains` MUST dar por buena la aserción grepeando la subcadena del valor (si la salida contuviera bytes ANSI, el grep MUST tratar la salida como binaria, p. ej. `grep -a`), y MUST NO depender de la presencia de códigos de color en la cadena.

### Requirement: El `.justfile` MUST exponer recipes `e2e` y una por extensión

El `.justfile` SHALL exponer la recipe `e2e` (ejecuta los cuatro scripts por extensión **en serie**, deteniéndose en el primer fallo propagando su exit code) y las recipes `e2e-mp3`, `e2e-m4a`, `e2e-ogg` y `e2e-opus` (un script cada una), con paso opcional de argumentos (`--bin`, `--keep`) al script correspondiente.

#### Scenario: Just e2e ejecuta los cuatro formatos en serie

- **WHEN** se ejecuta `just e2e`
- **THEN** MUST ejecutarse `e2e_mp3.sh`, `e2e_m4a.sh`, `e2e_ogg.sh` y `e2e_opus.sh` en ese orden y, si alguno termina con exit distinto de cero, la recipe MUST detenerse y propagar ese código.

#### Scenario: Just por extensión con argumentos

- **WHEN** se ejecuta `just e2e-opus --bin ./target/release/id3cli --keep`
- **THEN** MUST ejecutarse `scripts/e2e/e2e_opus.sh` recibiendo `--bin ./target/release/id3cli --keep`.

### Requirement: La ejecución MUST transcurrir bajo temporal no versionado con limpieza garantizada

Todo el material generado (audio y carátulas) SHALL crearse bajo un directorio temporal creado con `mktemp -d` dentro de `/tmp/opencode/e2e/<ext>/` y SHALL eliminarse al terminar mediante `trap`, salvo que se indique `--keep`. El binario MUST construirse una sola vez por ejecución del runner (no una vez por script) para no sobrecargar. Los scripts SHALL declarar `set -euo pipefail` y SHALL emitir los avisos y errores en español.

#### Scenario: Limpieza automática del temporal

- **WHEN** un script termina (con éxito o con fallo) sin `--keep`
- **THEN** el directorio temporal de su ejecución MUST haber sido eliminado por el `trap`, con independencia del código de salida.

#### Scenario: El binario se construye una única vez

- **WHEN** se ejecuta `just e2e` sin `--bin`
- **THEN** `cargo build` MUST ejecutarse una sola vez en toda la ejecución conjunta (el runner reutiliza el binario resultante para los cuatro formatos).

#### Scenario: Ausencia de herramientas obligatorias

- **WHEN** `ffmpeg` o `ffprobe` no están disponibles en el PATH
- **THEN** el script MUST fallar con exit 1 y un mensaje en español indicando la herramienta ausente, sin dejar archivos huérfanos en el temporal.