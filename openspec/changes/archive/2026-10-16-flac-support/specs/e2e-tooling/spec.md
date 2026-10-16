# Spec Delta

## Purpose

El tooling e2e pasa de cuatro a cinco formatos: FLAC se incorpora como quinto script por extensión y quinta recipe, con su códec y su conjunto de entradas de ffprobe en el runner común. El resto de contratos del runner (temporal no versionado, `--bin`/`--keep`, aserciones que devuelven 0, `e2e_summary`, `set -euo pipefail`, mensajes en español) no cambia.

## MODIFIED Requirements

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

### Requirement: El `.justfile` MUST exponer recipes `e2e` y una por extensión

El `.justfile` SHALL exponer la recipe `e2e` (ejecuta los cinco scripts por extensión **en serie**, deteniéndose en el primer fallo propagando su exit code) y las recipes `e2e-mp3`, `e2e-m4a`, `e2e-ogg`, `e2e-opus` y `e2e-flac` (un script cada una), con paso opcional de argumentos (`--bin`, `--keep`) al script correspondiente.

#### Scenario: Just e2e ejecuta los cuatro formatos en serie

- **WHEN** se ejecuta `just e2e`
- **THEN** MUST ejecutarse `e2e_mp3.sh`, `e2e_m4a.sh`, `e2e_ogg.sh`, `e2e_opus.sh` y `e2e_flac.sh` en ese orden y, si alguno termina con exit distinto de cero, la recipe MUST detenerse y propagar ese código.

#### Scenario: Just por extensión con argumentos

- **WHEN** se ejecuta `just e2e-opus --bin ./target/release/id3cli --keep`
- **THEN** MUST ejecutarse `scripts/e2e/e2e_opus.sh` recibiendo `--bin ./target/release/id3cli --keep`.

#### Scenario: Just por extensión con FLAC

- **WHEN** se ejecuta `just e2e-flac --bin ./target/release/id3cli --keep`
- **THEN** MUST ejecutarse `scripts/e2e/e2e_flac.sh` recibiendo `--bin ./target/release/id3cli --keep`.

## ADDED Requirements

### Requirement: El runner común MUST conocer la generación de audio y el conjunto de entradas de ffprobe de FLAC

El runner común SHALL declarar el códec `flac` en su tabla `E2E_CODEC` (valor `flac`, generado con `ffmpeg` sin flags adicionales, como en Opus) y SHALL declarar en `ffprobe_entries_for` el conjunto de entradas de ffprobe que expone la metadata de un FLAC nativo —medido durante GREEN con `ffmpeg` 9.0.2 y registrado en tasks.md (candidatas: `format_tags` o `stream_tags`, la caja de las claves y el par track/season)— y el `e2e_die` de esa función MUST aceptar `flac` como extensión conocida.

#### Scenario: La tabla de códec conoce FLAC

- **WHEN** `e2e_gen_audio` se ejecuta con `E2E_EXT=flac`
- **THEN** MUST generarse un audio FLAC decodificable con `ffmpeg` y el runner no debe abortar por extensión desconocida

#### Scenario: Las entradas de ffprobe de FLAC son las medidas

- **WHEN** `ffprobe_entries_for flac` se invoca
- **THEN** MUST devolver el conjunto de entradas medido y documentado en tasks.md (no un valor supuesto), y el runner debe usarlo en `check_ffprobe_tag` y `check_ffprobe_absent` de `e2e_flac.sh`