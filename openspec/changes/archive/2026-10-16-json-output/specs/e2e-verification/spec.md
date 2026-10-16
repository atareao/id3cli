# Spec Delta

## Purpose

La matriz de verificación e2e incorpora la capa JSON: cada script por extensión, tras el `show` legible ya verificado, ejecuta `show --json` y comprueba con `jq` (parser independiente, como `ffprobe`) el formato declarado, una muestra representativa de campos, el caso de archivo sin tags (MP3/MP4) y la salida plana sin ANSI. La falsabilidad se re-demuestra sobre una aserción JSON. El resto de la matriz (20 tags, roundtrips, degradaciones, remove) no cambia.

## ADDED Requirements

### Requirement: Cada formato MUST verificar su salida JSON con jq

El e2e de cada formato MUST ejecutar `show --json` sobre el archivo editado y MUST verificar: (a) `check_json_valid` (JSON válido + sin ANSI); (b) `check_json_format` con el códec real del formato (`mp3`, `m4a`→`mp4`, `ogg`, `opus`, `flac`); (c) una muestra representativa de campos con `check_json_field`: título, primer artista del array (en la etapa multi-artista), álbum, `track.number`, `compilation` y `pictures` (count/type); (d) en MP3 y MP4, el caso `tags:null` sobre audio recién generado sin tags (stderr vacío, exit 0).

#### Scenario: La capa JSON de MP3
- **WHEN** `e2e_mp3.sh` ejecuta su etapa JSON sobre el MP3 con los 20 tags escrito
- **THEN** `check_json_valid` MUST pasar, `check_json_format 'mp3'` MUST pasar, `check_json_field '.tags.title'` con el título de prueba MUST pasar y `check_json_field '.tags.track.number' '7'` MUST pasar

#### Scenario: Los formatos Ogg/Opus/FLAC nombran su códec real
- **WHEN** `e2e_ogg.sh`, `e2e_opus.sh` y `e2e_flac.sh` ejecutan su etapa JSON
- **THEN** `check_json_format` MUST esperar `ogg`, `opus` y `flac` respectivamente, y `check_json_field '.tags.track.number' '7'` MUST pasar en los tres

#### Scenario: MP3 sin tags en el e2e
- **WHEN** `e2e_mp3.sh` ejecuta `show --json` sobre el audio recién generado (antes de editar)
- **THEN** la aserción MUST confirmar `"tags": null` en el documento (`check_json_null '.tags'`), stderr vacío y exit 0

### Requirement: La falsabilidad MUST re-demostrarse sobre una aserción JSON

El flujo de falsabilidad del e2e MUST incluir la corrupción deliberada de una aserción JSON (p. ej. esperar `'mp3'` donde el formato es `'flac'` o un valor de campo erróneo): con el fallo inyectado el script MUST terminar con exit 1 nombrando la aserción; restaurado, MUST terminar con exit 0.

#### Scenario: Corrupción de una aserción JSON
- **WHEN** se corrompe deliberadamente el valor esperado de un `check_json_format` o `check_json_field`
- **THEN** el script MUST terminar con exit 1 e informar del nombre de la aserción fallida y del valor esperado frente al obtenido

#### Scenario: El flujo restaurado queda en verde
- **WHEN** se restaura el valor correcto y se vuelve a ejecutar el mismo flujo
- **THEN** el script MUST terminar con exit 0 y el resumen PASS/FAIL MUST reportar cero aserciones fallidas