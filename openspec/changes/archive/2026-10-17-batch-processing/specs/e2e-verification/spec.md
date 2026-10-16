# Spec Delta

## Purpose

La matriz de verificación e2e incorpora la capa batch: un script dedicado (`e2e_batch.sh`) que genera audio de varias extensiones, aplica operaciones batch, verifica que TODOS los archivos se procesaron (show/ffprobe por archivo), valida el NDJSON con `jq -s`, comprueba que un archivo inexistente produce exit ≠ 0 mientras el resto se procesa, y re-demuestra la falsabilidad. El resto de la matriz (20 tags, roundtrips, degradaciones, remove, completions) no cambia.

## MODIFIED Requirements

### Requirement: Cada formato MUST verificar su salida JSON con jq

El e2e de cada formato MUST ejecutar `show --json` sobre el archivo editado y MUST verificar: (a) `check_json_valid` (JSON válido + sin ANSI); (b) `check_json_format` con el códec real del formato (`mp3`, `m4a`→`mp4`, `ogg`, `opus`, `flac`); (c) una muestra representativa de campos con `check_json_field`: título, primer artista del array (en la etapa multi-artista), álbum, `track.number`, `compilation` y `pictures` (count/type); (d) en MP3 y MP4, el caso `tags:null` sobre audio recién generado sin tags (stderr vacío, exit 0). Además, el e2e de batch MUST validar la salida NDJSON de `show --json f1 f2 ...` con `jq -s` (N archivos → N líneas), de modo que cada línea sea el objeto JSON de su archivo.

#### Scenario: La capa JSON de MP3
- **WHEN** `e2e_mp3.sh` ejecuta su etapa JSON sobre el MP3 con los 20 tags escrito
- **THEN** `check_json_valid` MUST pasar, `check_json_format 'mp3'` MUST pasar, `check_json_field '.tags.title'` con el título de prueba MUST pasar y `check_json_field '.tags.track.number' '7'` MUST pasar

#### Scenario: Los formatos Ogg/Opus/FLAC nombran su códec real
- **WHEN** `e2e_ogg.sh`, `e2e_opus.sh` y `e2e_flac.sh` ejecutan su etapa JSON
- **THEN** `check_json_format` MUST esperar `ogg`, `opus` y `flac` respectivamente, y `check_json_field '.tags.track.number' '7'` MUST pasar en los tres

#### Scenario: MP3 sin tags en el e2e
- **WHEN** `e2e_mp3.sh` ejecuta `show --json` sobre el audio recién generado (antes de editar)
- **THEN** la aserción MUST confirmar `"tags": null` en el documento (`check_json_null '.tags'`), stderr vacío y exit 0

#### Scenario: La capa NDJSON del batch
- **WHEN** `e2e_batch.sh` ejecuta `id3cli show --json f1 f2` (dos archivos con tags)
- **THEN** el stdout tiene exactamente 2 líneas y `jq -s 'length'` devuelve 2, con el título de cada archivo en su línea correspondiente

### Requirement: El e2e MUST verificar la capa de completions con `bash -n` como oráculo

El e2e MUST ejecutar `id3cli completions bash` y verificar: (a) exit 0 con stderr vacío; (b) el stdout contiene la subcadena estable `complete -F _id3cli` (sin fijar los flags de la plantilla); (c) el stdout, guardado a fichero, pasa `bash -n` (script bash sintácticamente válido — el intérprete real como oráculo). Para `completions zsh` MUST verificar `#compdef id3cli` y para `completions fish` líneas `complete -c id3cli`, ambas con exit 0 y stderr vacío. MUST verificar que el script bash menciona los subcomandos `show`, `edit`, `remove` y `completions`. Con el modo batch, el script generado MUST reflejar los argumentos múltiples (`<FILE>...`) en los tres subcomandos.

#### Scenario: La capa de completions de bash
- **WHEN** el e2e de completions verifica `id3cli completions bash`
- **THEN** exit 0, stderr vacío, la subcadena estable `complete -F _id3cli` presente en stdout y `bash -n` sobre el stdout guardado devuelve 0

#### Scenario: La capa de completions de zsh y fish
- **WHEN** el e2e de completions verifica `id3cli completions zsh` y `id3cli completions fish`
- **THEN** ambos terminan con exit 0 y stderr vacío, el primero contiene `#compdef id3cli` y el segundo `complete -c id3cli`

#### Scenario: El script de completions refleja los argumentos múltiples
- **WHEN** el e2e de batch inspecciona el script de completions generado tras el change
- **THEN** el script MUST reflejar que `show`, `edit` y `remove` aceptan `<FILE>...` (múltiples archivos)

## ADDED Requirements

### Requirement: El e2e MUST verificar las operaciones batch y el exit agregado

El e2e de batch MUST verificar que una operación batch (`edit --title X f1 f2`, `remove --all f1 f2`) procesa todos los archivos: cada uno verificado por `show` y, cuando proceda, por `check_ffprobe_tag`. MUST verificar que un archivo inexistente entre archivos válidos produce exit ≠ 0 mientras el resto se procesa. MUST verificar el exit 0 solo cuando todos los archivos se procesan, y que `show` sin archivos termina con exit 2 (clap).

#### Scenario: Edit batch edita todos los archivos
- **WHEN** el e2e de batch ejecuta `id3cli edit --title X f1 f2` y verifica ambos por `show`/ffprobe
- **THEN** ambos archivos exponen el título X y el exit code es 0

#### Scenario: Archivo inexistente en el batch
- **WHEN** el e2e de batch ejecuta `id3cli edit --title X f1 INEXISTENTE f2`
- **THEN** el exit code es distinto de 0, f2 queda editado y el stderr menciona la ruta inexistente

#### Scenario: Show sin archivos
- **WHEN** el e2e de batch ejecuta `id3cli show` sin argumentos de archivo
- **THEN** el exit code es 2 (clap: argumento requerido) y el stderr lo indica

### Requirement: La falsabilidad MUST re-demostrarse sobre una aserción de batch

El e2e de batch MUST demostrar que el resumen detecta un fallo real: corrompiendo deliberadamente una expectativa (p. ej. esperar un título que no se escribió) el script termina con exit 1 y nombra la aserción fallida con lo esperado vs lo obtenido; restaurada, termina con exit 0.

#### Scenario: Falsabilidad de la capa de batch
- **GIVEN** una copia temporal del script con una expectativa corrompida
- **WHEN** se ejecuta la copia
- **THEN** termina con exit 1 y el FAIL nombra la aserción y el valor esperado
- **AND WHEN** se ejecuta la versión original
- **THEN** termina con exit 0 y 0 fallidas