# Spec Delta

## Purpose

El requirement de carátulas MP3 no definía la semántica de una segunda edición de `--cover`: `lofty` `insert_picture` no deduplica, de modo que `edit --cover` repetido acumulaba dos frames `APIC` `CoverFront` en lugar de sustituir la carátula. Este delta fija la semántica de sustitución: la carátula `CoverFront` preexistente se elimina antes de insertar la nueva, y las carátulas de otros tipos se conservan.

## MODIFIED Requirements

### Requirement: Carátulas en MP3 con detección de MIME por extensión

En MP3, la carátula indicada con `--cover` MUST escribirse en un frame `APIC`, y el tipo MIME MUST detectarse a partir de la extensión del archivo de imagen. El CLI SHALL aceptar `jpg` y `jpeg` como `image/jpeg`, `png` como `image/png` y `webp` como `image/webp`. Cuando el MP3 ya contiene una carátula del tipo `CoverFront`, `edit --cover` MUST **sustituirla** por la nueva imagen —eliminando los frames `APIC` de tipo `CoverFront` existentes antes de insertar la indicada— y MUST NOT acumular varias carátulas `CoverFront` por ediciones sucesivas: el archivo resultante MUST contener exactamente un frame `APIC` `CoverFront` con los bytes de la **última** imagen indicada. Las carátulas de otros tipos de picture (p. ej. `BackCover`) MUST conservarse intactas al sustituir la `CoverFront`.

#### Scenario: Carátula JPEG y PNG

- **WHEN** se ejecuta `edit --cover portada.jpg` y, en otra invocación, `edit --cover portada.png` sobre un MP3
- **THEN** el archivo MUST contener un frame `APIC` con MIME `image/jpeg` en el primer caso y `image/png` en el segundo, con los bytes de la imagen intactos

#### Scenario: Carátula WebP en MP3

- **WHEN** se ejecuta `edit --cover portada.webp` sobre un MP3
- **THEN** el archivo MUST contener un frame `APIC` con MIME `image/webp`, sin ningún warning

#### Scenario: Extensión de imagen no soportada

- **WHEN** se ejecuta `edit --cover imagen.tiff` sobre un MP3
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr un mensaje en español que indique que el formato de imagen no está soportado

#### Scenario: Archivo de carátula inexistente

- **WHEN** se ejecuta `edit --cover ruta/que/no/existe.jpg` sobre un MP3
- **THEN** el CLI MUST terminar con un código de salida distinto de cero y MUST escribir en stderr el mensaje en español de archivo de carátula no encontrado

#### Scenario: Segunda edición sustituye a la primera

- **WHEN** se ejecuta `edit --cover caratula_a.jpg` y después `edit --cover caratula_b.jpg` sobre el mismo MP3
- **THEN** el archivo resultante MUST contener exactamente un frame `APIC` de tipo `CoverFront`, con el MIME y los bytes de `caratula_b.jpg` y MUST NOT conservar los bytes de `caratula_a.jpg`

#### Scenario: Sustitución conserva carátulas de otros tipos

- **WHEN** se ejecuta `edit --cover caratula_b.jpg` sobre un MP3 que tiene un `APIC` `CoverFront` y además un `APIC` de otro tipo (p. ej. `BackCover`)
- **THEN** el `CoverFront` MUST sustituirse por los bytes de `caratula_b.jpg`, el `APIC` de `BackCover` MUST permanecer intacto y el archivo MUST contener los dos frames `APIC`

#### Scenario: Primera carátula sin acumulación previa

- **WHEN** se ejecuta `edit --cover caratula.jpg` sobre un MP3 sin carátulas previas
- **THEN** el archivo MUST contener exactamente un frame `APIC` `CoverFront` con los bytes de la imagen indicada