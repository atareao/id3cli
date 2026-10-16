# Spec Delta

## Purpose

La matriz e2e verificaba `remove --all` y la carátula por roundtrip, pero no ejercitaba la secuencia MP4 `edit → remove --all → edit` con lectura del lector independiente —la secuencia que destapó la invisibilidad de las etiquetas al demuxer— ni la semántica de una segunda edición de `--cover` —que destapó la acumulación de carátulas en MP3. Este delta añade ambas aserciones a la matriz de verificación.

## ADDED Requirements

### Requirement: El e2e MUST verificar la visibilidad externa de las etiquetas MP4 tras `remove --all` + `edit`

El flujo de `e2e_m4a.sh` MUST incluir una secuencia que edite el archivo con `--title`, elimine todos los metadatos con `remove --all` y vuelva a editarlo con los mismos campos, y MUST verificar después de esa secuencia que `id3cli show` muestra los campos y que `ffprobe -show_entries format_tags` los lista de nuevo (no solo los brands), de modo que un fallo de visibilidad externa como el del bug `hdlr` haga terminar el script con exit 1.

#### Scenario: La secuencia de reescritura mantiene visibles las etiquetas

- **WHEN** `e2e_m4a.sh` ejecuta `edit` (campos de prueba) → `remove --all` → `edit` (los mismos campos) sobre el `.m4a` generado
- **THEN** `check_show_contains` MUST verificar los campos por `id3cli show`, `check_ffprobe_tag` MUST verificar al menos el título y el artista en `format_tags` del archivo final, y el flujo MUST terminar en verde sin fallos

#### Scenario: La regresión del `hdlr` es detectada como fallo

- **WHEN** el archivo final de la secuencia anterior no expone las etiquetas en `ffprobe format_tags` (regresión del `hdlr`)
- **THEN** la aserción `check_ffprobe_tag` MUST fallar y el script MUST terminar con exit 1 e informar del nombre de la aserción

### Requirement: El e2e MUST verificar que una segunda edición de `--cover` sustituye en lugar de acumular

El flujo de carátula MUST incluir una segunda edición de `--cover` con una imagen distinta y verificar que el archivo resultante contiene exactamente una carátula del tipo por defecto del formato (`CoverFront` en MP3; la que fije el backend en los demás) con los bytes de la **última** imagen, sin conservar los de la primera. La aserción se aplica en MP3 y en los demás formatos donde la evidencia demuestre la acumulación (ver change `e2e-findings-fixes`); donde el formato ya sustituya nativamente, la aserción MUST fijar igualmente el comportamiento correcto.

#### Scenario: Segunda carátula sustituye a la primera

- **WHEN** el flujo de un formato ejecuta `edit --cover caratula_a.jpg` y después `edit --cover caratula_b.jpg` sobre el mismo archivo
- **THEN** la verificación de `show` MUST reportar una única carátula con los bytes de `caratula_b.jpg` (roundtrip `-c copy` contra `caratula_b.jpg`), MUST NOT aparecer la imagen `caratula_a.jpg`, y la aserción MUST fallar con exit 1 si el formato acumula las dos

#### Scenario: Sustitución con un solo cover no regresa

- **WHEN** se ejecuta el flujo de carátula con una única edición de `--cover` (el roundtrip JPG/PNG ya existente)
- **THEN** la verificación MUST seguir pasando exactamente igual que antes de este cambio, sin regresión en las aserciones existentes