# Spec Delta

## Purpose

Esta capability fija que la documentación de `id3cli` describa los tres formatos que la herramienta escribe —MP3, MP4/M4A y OGG Vorbis— y que no contenga afirmaciones obsoletas sobre cuáles están soportados. Cubre el `README.md`, la ayuda del CLI, los documentos de trabajo internos del repositorio y la `description` del manifiesto. El objetivo es que ningún lector, ni humano ni agente, deduzca del texto un conjunto de formatos, un mapeo de flags o un recuento de tests distinto del que el código implementa realmente.

## ADDED Requirements

### Requirement: El README MUST documentar los tres formatos soportados

`README.md` MUST presentar MP3, MP4/M4A y OGG Vorbis como formatos soportados y MUST incluir, para cada uno de los tres, el mapeo de los flags del CLI a la representación de ese formato (frames ID3v2.4, átomos `ilst`, claves de comentario de Vorbis) junto con sus degradaciones propias. MUST NOT presentar OGG Vorbis como formato no soportado en ninguna sección, y la lista de formatos no soportados MUST seguir incluyendo Opus, Speex, FLAC, WAV, AIFF y APE. La hoja de ruta del README MUST marcar OGG Vorbis como soportado y MUST seguir listando como pendientes Opus, Speex y FLAC. El recuento de tests que declare el README MUST coincidir con el de la suite real.

#### Scenario: Los tres formatos figuran como soportados

- **WHEN** se lee la sección de formatos admitidos
- **THEN** MP3, MP4/M4A y OGG Vorbis MUST figurar como soportados

#### Scenario: Mapeo documentado por formato

- **WHEN** se lee el README
- **THEN** MUST existir una sección o una tabla de mapeo para MP3, otra para MP4 y otra para OGG

#### Scenario: Ninguna afirmación obsoleta sobre OGG

- **WHEN** se busca OGG en la lista de formatos no soportados
- **THEN** MUST NOT aparecer, y Opus, Speex y FLAC MUST seguir apareciendo

#### Scenario: La hoja de ruta refleja el estado real

- **WHEN** se lee la hoja de ruta
- **THEN** OGG MUST figurar como soportado y Opus, Speex y FLAC MUST seguir como pendientes

#### Scenario: Degradaciones documentadas por formato

- **WHEN** se lee el README
- **THEN** MUST estar documentado que el WebP se degrada en MP4 y no en OGG, y que la letra pierde el código de idioma en MP4 y en OGG frente al `spa` de MP3

#### Scenario: Recuento de tests correcto

- **WHEN** se compara el recuento que declara el README con el que reporta la suite real
- **THEN** MUST coincidir

### Requirement: La ayuda del CLI MUST nombrar los tres formatos

La descripción del binario y la ayuda de los subcomandos `show`, `edit` y `remove` MUST nombrar MP3, MP4 y OGG; MUST NOT quedar ningún texto de ayuda que presente solo dos formatos. La ayuda de los flags `--track` y `--season` MUST documentar la forma `N` o `N/T`.

#### Scenario: La ayuda general nombra los tres formatos

- **WHEN** se ejecuta el binario con `--help`
- **THEN** el texto MUST mencionar MP3, MP4 y OGG

#### Scenario: La ayuda de los subcomandos nombra los tres formatos

- **WHEN** se ejecutan `show --help`, `edit --help` y `remove --help`
- **THEN** ninguno MUST presentar solo dos formatos

#### Scenario: La ayuda documenta la forma número/total

- **WHEN** se ejecuta `edit --help`
- **THEN** la ayuda de `--track` y de `--season` MUST documentar que admiten `N` o `N/T`

#### Scenario: La ayuda del borrado nombra los tres formatos

- **WHEN** se ejecuta `remove --help`
- **THEN** la lista de nombres de campo aceptados MUST seguir siendo la misma que antes del cambio y la ayuda MUST nombrar los tres formatos

### Requirement: Los documentos de trabajo del repositorio MUST ser coherentes con lo implementado

`AGENTS.md` y `.github/copilot-instructions.md` MUST describir los tres backends y sus convenciones, MUST NOT listar OGG como formato no soportado y MUST declarar el recuento de tests correcto. Como estos dos ficheros no viajan dentro del paquete publicado (la lista blanca `include` de `Cargo.toml` no los incluye), su verificación MUST hacerse con evidencia registrada en el cambio —la búsqueda y su salida— y MUST NOT depender de un test de la suite que los lea, porque un `include_str!` sobre un fichero fuera del paquete rompería la verificación del paquete publicado.

#### Scenario: AGENTS.md describe los tres formatos

- **WHEN** se lee `AGENTS.md`
- **THEN** los tres formatos MUST estar descritos como soportados y OGG MUST NOT aparecer en la lista de formatos rechazados

#### Scenario: Las instrucciones de Copilot describen los tres formatos

- **WHEN** se lee `.github/copilot-instructions.md`
- **THEN** MUST estar descritos los tres formatos y sus degradaciones

#### Scenario: Recuento de tests coherente

- **WHEN** se compara el recuento declarado en esos documentos con el de la suite
- **THEN** MUST coincidir

#### Scenario: Verificación con evidencia registrada

- **WHEN** se cierra el cambio
- **THEN** la evidencia de las comprobaciones anteriores MUST estar registrada dentro del propio cambio

### Requirement: El manifiesto MUST describir los tres formatos

La `description` de `Cargo.toml` MUST nombrar los tres formatos. Las cinco `keywords` MUST NOT modificarse, porque están fijadas por la capability `crate-metadata`.

#### Scenario: La descripción nombra los tres formatos

- **WHEN** se lee la `description` del manifiesto
- **THEN** MUST mencionar MP3, MP4 y OGG

#### Scenario: Las keywords no cambian

- **WHEN** se leen las `keywords`
- **THEN** MUST seguir siendo exactamente las cinco fijadas por `crate-metadata`, sin añadir ni quitar ninguna
