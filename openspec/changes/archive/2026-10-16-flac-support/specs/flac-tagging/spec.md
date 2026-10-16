# Spec Delta

## Purpose

Los archivos FLAC utilizan el mismo mecanismo de metadatos que OGG y Opus: los comentarios de Vorbis (`VorbisComments`, pares clave=valor). Esta capability nueva define el contrato de lectura y escritura de metadatos en FLAC reutilizando el backend de comentarios de Vorbis: el tipo concreto con el que se abre el archivo (`FlacFile`), la materialización del comentario cuando el archivo no lo lleva (en FLAC el bloque VORBIS_COMMENT es opcional, a diferencia de Ogg), el mapeo de claves (idéntico al de OGG/Opus), las degradaciones heredadas, la carátula (bloque PICTURE separado del archivo — comportamiento del escritor de lofty 0.25.4 verificado, no campo `METADATA_BLOCK_PICTURE` del comentario) y el borrado (el bloque VORBIS_COMMENT se elimina completo con `remove --all`, sin vendor conservado, a diferencia de Ogg/Opus). Quedan fuera de alcance el tag ID3v2 que un FLAC pueda llevar y el códec FLAC dentro del contenedor Ogg.

## ADDED Requirements

### Requirement: Contrato de tipos: `FlacFile` con su `VorbisComments` materializado

El CLI SHALL abrir un archivo FLAC con el tipo concreto `FlacFile` de `lofty` y operar sobre su comentario interno. A diferencia de los contenedores Ogg (donde el comentario de Vorbis es obligatorio), en FLAC el bloque VORBIS_COMMENT es opcional y `FlacFile::vorbis_comments()` devuelve un `Option`: `read_vorbis_tag` SHALL generalizarse para que, ante `AudioFormat::Flac`, abra con `FlacFile::read_from` y devuelva `vorbis_comments().cloned()` o, si el archivo no tiene comentario, un `VorbisComments` vacío recién creado, conservando el mismo nombre de función, la misma firma y el mismo tipo de retorno `VorbisComments` que hoy. `write_vorbis_tag` MUST conservar su implementación (`tag.save_to_path(path, WriteOptions::default())`): `lofty` inspecciona el contenedor real (probe.rs: `b"fLaC"` → `FileType::Flac`), escribe el bloque VORBIS_COMMENT en su posición y lo crea si no existía.

#### Scenario: Lectura del tag de un FLAC existente
- **WHEN** se ejecuta `show` sobre un `.flac` que ya tiene campos en su comentario de Vorbis
- **THEN** el CLI MUST leer esos campos con `read_vorbis_tag` y MOSTRARlos con los mismos nombres de campo que en OGG, Opus, MP3 y MP4

#### Scenario: FLAC sin bloque VORBIS_COMMENT
- **WHEN** se ejecuta `edit` o `show` sobre un FLAC válido que no contiene bloque VORBIS_COMMENT
- **THEN** `read_vorbis_tag` MUST devolver un comentario vacío (sin error); `edit` MUST crear el bloque al escribir y `show` MUST informar de que no hay metadatos, ambos con código de salida correcto

#### Scenario: La misma función lee OGG, Opus y FLAC
- **WHEN** se invoca `read_vorbis_tag` sobre un `.ogg`, un `.opus` y un `.flac`
- **THEN** las tres llamadas MUST usar el mismo nombre de función y devolver el mismo tipo `VorbisComments`, sin que la llamada indique el contenedor

### Requirement: Mapeo de los flags del CLI a las claves de Vorbis en FLAC

El CLI SHALL escribir cada flag de `edit` sobre un `.flac` en la misma clave Vorbis que en OGG/Opus: `--title` → `TITLE`, `--artist` → `ARTIST`, `--album` → `ALBUM`, `--album-artist` → `ALBUMARTIST`, `--year` → `YEAR`, `--date` → `DATE` (claves separadas e independientes), `--genre` → `GENRE`, `--track` → `TRACKNUMBER` con el total en `TRACKTOTAL`, `--season` → `DISCNUMBER` con el total en `DISCTOTAL`, `--composer` → `COMPOSER`, `--copyright` → `COPYRIGHT`, `--lyrics` → `LYRICS`, `--cover` → imagen del archivo (bloque PICTURE), `--compilation` → `COMPILATION` con valor `"1"`, `--title-sort` → `TITLESORT`, `--artist-sort` → `ARTISTSORT`, `--album-sort` → `ALBUMSORT` y `--subtitle` → `SUBTITLE`. El CLI MUST NOT escribir ninguna otra clave ni en otra caja para estos campos. `--url` y `--original-artist` no tienen clave equivalente en Vorbis: MUST omitirse con aviso por stderr y MUST NOT escribirse una clave inventada.

#### Scenario: Escritura de las claves de texto básicas en FLAC
- **WHEN** se ejecuta `edit --title`, `--artist` y `--album` sobre un `.flac`
- **THEN** las claves `TITLE`, `ARTIST` y `ALBUM` MUST contener los valores indicados y MUST poder releerse con un lector independiente

#### Scenario: Año y fecha en claves separadas
- **WHEN** se ejecuta `edit --year 2026 --date 2026-10-13` sobre un `.flac`
- **THEN** `YEAR` MUST contener `2026`, `DATE` MUST contener `2026-10-13` y ninguno debe sobrescribir al otro

#### Scenario: Número de pista y temporada con totales
- **WHEN** se ejecuta `edit --track 7/12 --season 1/2` sobre un `.flac`
- **THEN** `TRACKNUMBER` MUST contener `7`, `TRACKTOTAL` `12`, `DISCNUMBER` `1` y `DISCTOTAL` `2`

#### Scenario: url y original_artist omitidos con aviso
- **WHEN** se ejecuta `edit --url ... --original-artist ...` sobre un `.flac`
- **THEN** el CLI MUST avisar por stderr de que se omiten (sin clave equivalente) y MUST NOT escribir ninguna clave inventada; la ausencia MUST poder comprobarse con un lector independiente

### Requirement: Carátula en FLAC como bloque PICTURE del archivo

El CLI MUST escribir la carátula de FLAC reutilizando `push_ogg_cover` (la picture viaja en el tag al guardar) y, por el comportamiento del escritor de lofty 0.25.4 verificado empíricamente (bloque PICTURE separado del archivo, no dentro del comentario Vorbis), la carátula MUST leerse y mostrarse desde los bloques PICTURE del propio `FlacFile` (`flac.pictures()`), no desde el tag. El CLI MUST sustituir la `CoverFront` previa en una segunda edición (una única imagen, la de la última indicada — verificado: `remove_picture_type(CoverFront)` + `insert_picture` deja exactamente un bloque PICTURE). El MIME de la imagen MUST derivarse de la extensión (`.jpg`/`.jpeg` → `image/jpeg`, `.png` → `image/png`, `.webp` → `image/webp` sin degradación, igual que OGG).

#### Scenario: Carátula JPG y PNG con roundtrip
- **WHEN** se ejecuta `edit --cover` con una imagen JPG y luego con una PNG sobre dos FLAC recién generados
- **THEN** cada archivo MUST contener exactamente una imagen como bloque PICTURE, el MIME MUST ser el derivado de la extensión y la extracción (lector independiente) MUST coincidir byte a byte con la imagen original

#### Scenario: Segunda edición de carátula sustituye
- **WHEN** se ejecuta `edit --cover imagen_a` y después `edit --cover imagen_b` (distinta) sobre el mismo `.flac`
- **THEN** el archivo MUST quedar con una única carátula (la `b`), sin acumular la `a` (comprobable por bloques y por roundtrip de bytes)

#### Scenario: show muestra la carátula de FLAC
- **WHEN** se ejecuta `show` sobre un `.flac` con carátula escrita por el CLI
- **THEN** el CLI MUST mostrar la carátula (contador «Carátulas: N imagen(es)») leyéndola de los bloques PICTURE del archivo, con el mismo formato que en OGG/Opus

### Requirement: Borrado de metadatos en FLAC (bloque VORBIS_COMMENT eliminado, sin vendor)

`remove` de un campo concreto y `remove --all` MUST funcionar sobre `.flac`: el campo indicado desaparece y los no mencionados sobreviven. `remove --all` MUST eliminar los campos de usuario y las carátulas; por el comportamiento del escritor de lofty 0.25.4 verificado, el bloque VORBIS_COMMENT queda ELIMINADO del archivo al guardar un tag vacío, de modo que el vendor del codificador NO se conserva (degradación documentada de FLAC, a diferencia de Ogg/Opus donde el comentario es obligatorio y el vendor se preserva). El archivo MUST seguir siendo un FLAC válido por contenido y reproducible por `ffprobe`. Los tags sin clave equivalente en Vorbis (`url`, `original_artist`) MUST omitirse con aviso en `remove`, igual que en OGG/Opus.

#### Scenario: remove de un campo conserva el resto
- **WHEN** se ejecuta `remove title` sobre un `.flac` con título, artista y álbum
- **THEN** `TITLE` MUST desaparecer y `ARTIST`/`ALBUM` MUST permanecer

#### Scenario: remove --all elimina el bloque de comentario y las carátulas
- **WHEN** se ejecuta `remove --all` sobre un `.flac` con campos de usuario y carátula
- **THEN** el archivo MUST quedar sin campos de usuario, sin carátulas y sin bloque VORBIS_COMMENT (el vendor se pierde con el bloque — degradación documentada), y MUST seguir siendo FLAC por contenido (patrón `fLaC` y bloque STREAMINFO intactos) y reproducible