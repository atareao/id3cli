# Design

## Context

`id3cli` escribe metadatos en MP3 (frames ID3v2.4 vía `lofty` 0.25.4) y en MP4/M4A (átomos `ilst`). Este cambio añade OGG Vorbis (`VorbisComments`) como tercer formato y extiende el total de pista y de temporada a los tres contenedores.

La detección del formato por **magic bytes** vive en `src/lib.rs`: la constante `FORMAT_PROBE_LEN = 16` fija cuántos bytes se leen, el enum `AudioFormat` (`Mp3`, `Mp4`, `Unsupported`, `Unknown`) clasifica el contenido y la lista `UNSUPPORTED_MAGIC` enumera las firmas rechazadas, entre ellas `OggS` — que este cambio MUST sacar de esa lista para pasar a inspeccionar el identificador de códec.

`src/main.rs` contiene el enum `Backend` (`Mp3`, `Mp4`) que enruta cada subcomando hacia el backend correspondiente, además de tres mensajes en español que enumeran los formatos soportados y que MUST actualizarse al incorporar el tercero. El backend MP4 trabaja con el `Tag` genérico de `lofty` y una tabla `MP4_ATOM_MAP` de flag → átomo; el de MP3 trabaja con el tipo concreto `Id3v2Tag` y frames explícitos. El backend de OGG seguirá el patrón de frames explícitos del de MP3, no el genérico del de MP4 (ver Decisión 2).

Los contratos vivos afectados por este cambio son `openspec/specs/format-detection/spec.md`, `openspec/specs/mp3-tagging/spec.md`, `openspec/specs/m4a-tagging/spec.md` y `openspec/specs/cli-output/spec.md`. La capability `crate-metadata` fija las keywords del manifiesto y la `release-publishing` fija el contenido del paquete publicado; ambas permanecen vigentes sin cambios.

## Goals / Non-Goals

**Goals**

- Aceptar **Ogg Vorbis** como tercer formato soportado, con paridad de flags respecto a MP3 y MP4.
- Permitir el **total de pista** (`N/T`) y el **total de temporada** (`N/T`) en los tres formatos.
- Dejar la documentación de los tres formatos como un **contrato verificable** por tests.

**Non-Goals**

- Opus, Speex y FLAC (ni nativo ni dentro de Ogg).
- Theora o cualquier flujo de vídeo en Ogg.
- Capítulos de `.m4b`.
- ID3 dentro de Ogg.
- Cambiar las keywords del manifiesto (`crate-metadata`) ni la lista blanca `include` del paquete publicado (`release-publishing`).

## Decisions

### 1. Detección por el identificador de códec del primer paquete

El identificador de códec del primer paquete es lo que decide si un `OggS` es Vorbis. Hay que decidirlo **antes de tocar el archivo**, porque un Ogg puede transportar Vorbis, Opus, Speex, FLAC o vídeo, y cada uno exige un tratamiento distinto.

El primer paquete empieza en el offset `27 + header[26]`: la cabecera de página ocupa 27 bytes y el byte 26 es el número de entradas de la tabla de segmentos. Medido sobre un `.ogg` real generado con `ffmpeg`, el identificador `\x01vorbis` cae en el offset 28.

Como consecuencia, el sondeo MUST subir de 16 a **512 bytes** para cubrir cualquier primera página válida (27 + hasta 255 bytes de tabla de segmentos).

Alternativas descartadas:

- **Decidir por extensión**: deja el comportamiento roto ante renombrados, y el proyecto ya decidió por contenido en `format-detection`.
- **Aceptar cualquier `OggS`**: arrastraría códecs fuera del alcance declarado en Goals / Non-Goals.

### 2. Backend con el tipo concreto `VorbisComments`, no con el `Tag` genérico más `ItemKey`

El backend de OGG SHALL usar el tipo concreto `VorbisComments`. El tipo genérico **descarta** los items sin mapeo — lo documenta la propia librería —, de modo que al reescribir se perderían `ENCODER`, `MUSICBRAINZ_*` y cualquier clave puesta por otra herramienta.

El tipo concreto conserva **todas** las claves y el vendor, y además deja los nombres de clave explícitos en la tabla de mapeo, lo que permite documentarlos en el README como contrato.

Alternativa descartada: reutilizar el patrón del backend MP4 (genérico + `ItemKey`) por simetría, con el coste de pérdida de datos descrito.

### 3. `YEAR` y `DATE` como claves separadas

La tabla `VORBIS_MAP` de `lofty` 0.25.4 mapea `YEAR` a `Year` y `DATE` a `RecordingDate`, así que no hay motivo para absorber una clave en otra. Se consigue así paridad con MP3, que ya separa `TYER` y `TDRC`.

Alternativa descartada: repetir la colisión de MP4, donde `--year` y `--date` comparten el átomo `©day` y uno de los dos se pierde.

### 4. `--track` y `--season` con total como par `N/T` validado en el parseo de `clap`

`--track` y `--season` SHALL aceptar la forma `N/T` y SHALL validarla mediante una implementación de `FromStr` en el parseo de `clap`, en lugar de introducir dos flags nuevos.

Al validar en el parseo, una forma inválida falla **antes de abrir el archivo**, así que el archivo queda intacto por construcción.

Los tres contenedores admiten el total de forma nativa:

- Medido que `Id3v2Tag::set_track` conserva el total existente (`insert_number_pair`, en `src/id3/v2/tag.rs`).
- Medido que la tabla `ILST_MAP` de `lofty` mapea `trkn` a la vez a `TrackNumber` y a `TrackTotal`.
- OGG Vorbis expone `TRACKNUMBER` y `TRACKTOTAL` como claves independientes.

Alternativa descartada: `--track-total` y `--season-total`, que añaden dos flags y se separan de la convención `n/total` que el usuario ya conoce de `TRCK`.

### 5. Carátula de OGG en `METADATA_BLOCK_PICTURE`

La carátula SHALL escribirse en la clave `METADATA_BLOCK_PICTURE`, con el MIME derivado de la extensión del archivo y con la información de imagen explícita.

Medido que `PictureInformation::from_picture` de `lofty` 0.25.4 solo infiere PNG y JPEG y devuelve ceros para todo lo demás, y que devuelve error si la imagen tiene menos de 8 bytes. Por eso se le pasa `from_picture(...).unwrap_or_default()`, para que ni un WebP ni una imagen diminuta aborten la edición.

La clave no tiene lista de tipos de imagen admitidos, así que el WebP **no se degrada**: contraste con `covr` en MP4, donde el WebP se escribe con el tipo de dato reservado (0) y con aviso por stderr.

### 6. Letra en `LYRICS`, leyendo además `UNSYNCEDLYRICS` al visualizar

La letra SHALL escribirse en la clave `LYRICS` y `show` SHALL leer además `UNSYNCEDLYRICS` al visualizar.

El contenedor Vorbis no guarda idioma para la letra, lo que es una degradación frente al `USLT` con código `spa` de MP3 y frente a lo que documenta `m4a-tagging` para `©lyr`. La degradación MUST quedar documentada.

Alternativa descartada: escribir `UNSYNCEDLYRICS` por imitar el significado de MP4.

### 7. El comentario obligatorio cambia el contrato de `show` y de `remove --all`

Como el **vendor** no se puede eliminar del contenedor, el contrato de `show` y de `remove --all` cambia:

- `show` SHALL ignorar el vendor y `ENCODER` — un `.ogg` recién creado siempre los lleva y no son metadatos del usuario.
- `remove --all` SHALL vaciar el comentario **preservando** el vendor.

Es un contraste deliberado con MP3, donde eliminar el último campo hace desaparecer el bloque ID3v2 del archivo.

### 8. `--url` y `--original-artist` se omiten con aviso

`--url` y `--original-artist` SHALL omitirse en OGG con aviso por stderr, en paridad con MP4, en lugar de escribirlos como claves crudas `WEBSITE` y `ORIGINALARTIST`.

Son convenciones no normalizadas y `show` no podría releerlas con el mapeo estándar, con lo que quedaría un campo que se escribe y no se puede mostrar.

### 9. Fixture real generado con `ffmpeg`, verificado por estructura

El fixture SHALL ser un `.ogg` real generado con `ffmpeg` (`libvorbis`, seno de 0,6 s a 44,1 kHz, mono), acompañado de un script de regeneración que verifica **estructura, no igualdad de bytes**.

Medido que el comentario de un `.ogg` de `ffmpeg` lleva siempre el vendor y una clave `ENCODER` con la versión del codificador, y que `-fflags +bitexact` **no** los elimina; por tanto no se puede exigir un fixture reproducible byte a byte entre versiones de `ffmpeg`.

Se conservan `-map_metadata -1` y `+bitexact` para no arrastrar otros metadatos.

### 10. La documentación es una capability con verificación por test, pero solo sobre ficheros que viajan en el paquete

La documentación de los tres formatos SHALL ser una capability con verificación por test, pero la verificación por `include_str!` solo puede aplicarse a ficheros que viajan en el paquete publicado:

- `README.md` y `Cargo.toml` pueden leerse con `include_str!` desde los tests.
- La ayuda de `clap` puede comprobarse lanzando el binario desde un test de integración.
- `AGENTS.md` y `.github/copilot-instructions.md` **no** están en la lista blanca `include` del paquete (contrato de `release-publishing`), así que un `include_str!` sobre ellos rompería la verificación del paquete publicado. Su verificación queda como tarea con evidencia de búsqueda registrada en el cambio.

## Risks / Trade-offs

- **(a) Cambiar el tipo de `--track` y `--season`** obliga a adaptar los tests de MP3 y MP4 que hoy escriben numeración sin total. Mitigación: un grupo de caracterización previa y un commit propio de baseline.
- **(b) El sondeo de archivo crece de 16 a 512 bytes.** Asumido: es una lectura única de los primeros bytes del archivo.
- **(c) Asimetría descubierta y fuera del alcance de este cambio:** el backend MP4, al usar el `Tag` genérico, descarta los átomos sin mapeo al reescribir, mientras el de OGG conserva las claves ajenas. Se deja constancia como hallazgo para un cambio futuro.
- **(d) La release publicará una 0.6.0 automáticamente** por el prefijo `feat:` del commit.

## Migration Plan

El orden de los grupos de `tasks.md`, y por qué:

1. **Baseline y red de seguridad**, en un commit propio antes de tocar producción — fija el comportamiento actual de MP3 y MP4 antes de cambiar el tipo de `--track` y `--season`.
2. **Detección de Ogg Vorbis**, test-first — decide antes de abrir el archivo y desbloquea todo lo demás.
3. **Fixture real**, en su propio commit — el insumo verificable de los grupos siguientes.
4. **Backend de OGG**, test-first — la escritura efectiva sobre el contenedor.
5. **Totales en los tres formatos**, test-first — consume la nueva forma `N/T` ya validada en el parseo.
6. **Documentación de los tres formatos**, con sus tests de coherencia — cierra el contrato documental sobre ficheros que viajan en el paquete.
7. **Verificación final y cierre**, con evidencia — recorre todas las casillas y exige su prueba.

Cada grupo funcional MUST empezar con un test que falla (RED) antes de la implementación (GREEN), y ninguna casilla se marca sin la verificación indicada en ella.
