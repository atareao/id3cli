# e2e-tooling Specification

## Purpose
Esta capability define los contratos del **tooling de verificación e2e local** de `id3cli`: la ubicación y los nombres de los scripts versionados, el CLI de cada script (`--bin`, `--keep`, códigos de salida), las funciones del runner común fuenteable, las recipes del `justfile` y las restricciones de ejecución (todo bajo temporal no versionado, `set -euo pipefail`, mensajes en español). El objetivo es que cualquiera pueda ejecutar `just e2e` y obtener una verificación de extremo a extremo del binario real —escribir y releer todos los tags y las carátulas en los cuatro formatos soportados— sin sobrecargar el repositorio con archivos de audio ni imágenes generados. Esta capability fija el **cómo** se verifica; la matriz de verificación (qué tags, qué degradaciones, qué aserciones) es la capability `e2e-verification`.

## Requirements

### Requirement: Los scripts de verificación MUST versionarse en `scripts/e2e/` y los datos sobre los que operan MUST NOT versionarse

Debe existir un runner común fuenteable `scripts/e2e/e2e_lib.sh` y un script de flujo por extensión soportada —`scripts/e2e/e2e_mp3.sh`, `scripts/e2e/e2e_m4a.sh`, `scripts/e2e/e2e_ogg.sh`, `scripts/e2e/e2e_opus.sh` y `scripts/e2e/e2e_flac.sh`— todos ellos bash y versionados. Los archivos de audio y las imágenes que generan MUST crearse en un directorio temporal de la ejecución y MUST NOT versionarse ni añadirse al repositorio; el repositorio MUST quedar sin binarios ni imágenes generados por el e2e.

#### Scenario: La estructura de scripts existe y está versionada

- **WHEN** se inspecciona el árbol del repositorio tras el cambio
- **THEN** MUST existir `scripts/e2e/e2e_lib.sh` y los cinco scripts por extensión, y MUST estar bajo control de versiones.

#### Scenario: Los archivos generados no entran en el repositorio

- **WHEN** se ejecuta el e2e completo y se inspecciona `git status`
- **THEN** MUST NOT aparecer ningún archivo de audio ni imagen generados como cambio nuevo, porque todo el material generado MUST vivir en el temporal de la ejecución.

#### Scenario: Ejecución de e2e_flac

- **WHEN** se ejecuta `scripts/e2e/e2e_flac.sh --bin ./target/release/id3cli`
- **THEN** MUST usarse ese binario, ejecutarse la verificación de FLAC y terminar con exit 0 solo si todas sus aserciones pasan.

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

El runner común SHALL exponer, además de las funciones existentes, las aserciones JSON: `check_json_valid` (ejecuta `show --json`, valida `jq -e . >/dev/null` y comprueba que la salida NO contiene bytes ANSI), `check_json_format <formato>` (el `.formato` coincide), `check_json_field <ruta-jq> <valor>` (`jq -r` sobre la ruta y `grep -aF` del valor contra la salida) y `check_json_null <ruta-jq>` (la ruta extrae `null`). Todas MUST registrar el resultado en el recuento del resumen y MUST devolver 0 (el fallo se mide por el contador), igual que las aserciones existentes. El runner SHALL exponer además las funciones necesarias para scripts no ligados a un solo formato: `e2e_completions.sh` usa `e2e_setup`, `e2e_run`, `check_rc`, `check_show_contains` y `e2e_summary` sin audio, y `e2e_batch.sh` SHALL poder generar audio de varias extensiones (`e2e_gen_audio` con `E2E_EXT` por archivo), ejecutar operaciones batch y verificar por archivo con `check_show_contains`/`check_ffprobe_*`; el chequeo del stderr vacío SHALL hacerse sobre el `$E2E_ERR` capturado, contabilizándose en el resumen.

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

#### Scenario: El script de batch suma aserciones al resumen
- **GIVEN** el script `e2e_batch.sh`, que genera audio de varias extensiones y ejecuta operaciones batch
- **WHEN** termina su ejecución
- **THEN** MUST imprimir `PASS/FAIL: batch: <N> aserciones, 0 fallidas` y terminar con exit 0

### Requirement: El `.justfile` MUST exponer recipes `e2e` y una por extensión

El `.justfile` SHALL exponer la recipe `e2e` (ejecuta los cinco scripts por extensión **en serie** Y después `e2e_completions.sh --bin target/debug/id3cli` y `e2e_batch.sh --bin target/debug/id3cli`, deteniéndose en el primer fallo propagando su exit code, con una sola construcción previa `cargo build --locked`) y las recipes `e2e-mp3`, `e2e-m4a`, `e2e-ogg`, `e2e-opus`, `e2e-flac`, `e2e-completions` y `e2e-batch` (un script cada una), con paso opcional de argumentos (`--bin`, `--keep`) al script correspondiente.

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

#### Scenario: Just e2e cubre los siete scripts
- **WHEN** se ejecuta `just e2e`
- **THEN** MUST ejecutarse `e2e_mp3.sh`, `e2e_m4a.sh`, `e2e_ogg.sh`, `e2e_opus.sh`, `e2e_flac.sh`, `e2e_completions.sh --bin target/debug/id3cli` y, después, `e2e_batch.sh --bin target/debug/id3cli` en ese orden y, si alguno termina con exit distinto de cero, la recipe MUST detenerse y propagar ese código

#### Scenario: Just de batch con argumentos
- **WHEN** se ejecuta `just e2e-batch --bin ./target/release/id3cli --keep`
- **THEN** MUST ejecutarse `scripts/e2e/e2e_batch.sh` recibiendo `--bin ./target/release/id3cli --keep`

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

### Requirement: El runner común MUST conocer la generación de audio y el conjunto de entradas de ffprobe de FLAC

El runner común SHALL declarar el códec `flac` en su tabla `E2E_CODEC` (valor `flac`, generado con `ffmpeg` sin flags adicionales, como en Opus) y SHALL declarar en `ffprobe_entries_for` el conjunto de entradas de ffprobe que expone la metadata de un FLAC nativo —medido durante GREEN con `ffmpeg` 9.0.2 y registrado en tasks.md (candidatas: `format_tags` o `stream_tags`, la caja de las claves y el par track/season)— y el `e2e_die` de esa función MUST aceptar `flac` como extensión conocida.

#### Scenario: La tabla de códec conoce FLAC

- **WHEN** `e2e_gen_audio` se ejecuta con `E2E_EXT=flac`
- **THEN** MUST generarse un audio FLAC decodificable con `ffmpeg` y el runner no debe abortar por extensión desconocida

#### Scenario: Las entradas de ffprobe de FLAC son las medidas

- **WHEN** `ffprobe_entries_for flac` se invoca
- **THEN** MUST devolver el conjunto de entradas medido y documentado en tasks.md (no un valor supuesto), y el runner debe usarlo en `check_ffprobe_tag` y `check_ffprobe_absent` de `e2e_flac.sh`
