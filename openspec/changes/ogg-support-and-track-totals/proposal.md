# Proposal

## Why

OGG Vorbis se reconoce hoy únicamente para rechazarlo: `OggS` figura en la lista de magic bytes no soportados y el CLI termina con error, así que cualquier colección con ficheros `.ogg` queda fuera de la herramienta. Al mismo tiempo, el total de pista y de disco —que los tres contenedores guardan de forma nativa— no se puede fijar desde el CLI, porque `--track` y `--season` son enteros sin total. Además, la documentación describe dos formatos y ya está desincronizada con la suite: declara 219 tests (124 unitarios + 95 de integración) cuando los reales son 220 (125 + 95).

## What Changes

- **Detección.** `OggS` deja de ser un magic byte rechazado; se acepta **solo Ogg Vorbis**, reconocido por el identificador de códec `\x01vorbis` dentro del primer paquete de la primera página. Opus, Speex, FLAC dentro de Ogg y el vídeo Theora siguen rechazados con el mensaje en español actual.
- **Backend OGG nuevo** sobre comentarios de Vorbis, con una tabla única de mapeo flag → clave que cubre los 18 flags, con el mismo alcance que los otros dos formatos.
- **`--url` y `--original-artist`** se omiten con aviso por stderr en OGG, igual que ya ocurre en MP4, porque la tabla de mapeo de `lofty` no tiene clave para ellos.
- **`--year` → `YEAR` y `--date` → `DATE`** como claves independientes, sin colisión ni absorción entre ellas (a diferencia de MP4, donde comparten el átomo `©day`).
- **`--track` y `--season` pasan a aceptar `N` o `N/T`** (número y total) **en los tres formatos**, escribiendo el total en la representación nativa de cada contenedor: `TRCK`/`TPOS` como `n/total` en MP3, los átomos `trkn`/`disk` con su campo de total en MP4, y `TRACKNUMBER`+`TRACKTOTAL`/`DISCNUMBER`+`DISCTOTAL` en OGG. La forma sin barra sigue comportándose igual que antes, así que el cambio es aditivo para quien usa el CLI, aunque cambia la firma interna de las funciones de la librería.
- **`show` muestra el total** de pista y de temporada en los tres formatos; **`remove` elimina número y total como una unidad**.
- **Carátula en OGG** en la clave `METADATA_BLOCK_PICTURE` con el tipo MIME derivado de la extensión: el WebP **no se degrada** en OGG (en MP4 sí, al tipo de dato reservado, con aviso).
- **Letra en OGG** en la clave `LYRICS`, sin código de idioma (degradación frente al `spa` del frame `USLT` de MP3), leyendo además `UNSYNCEDLYRICS` al visualizar.
- **Comentario obligatorio.** El comentario de Vorbis es obligatorio en el contenedor: `show` no cuenta el vendor ni `ENCODER` como metadatos, y `remove --all` vacía el comentario preservando el vendor, el audio y la reproducibilidad.
- **Documentación.** Capability nueva que fija los tres formatos documentados y sin afirmaciones obsoletas; se tocan `README.md`, `AGENTS.md`, `.github/copilot-instructions.md`, la `description` de `Cargo.toml` y la ayuda de `clap`, y se corrige el recuento de tests desincronizado.
- **Fuera de alcance (no-goals).** Opus, Speex, FLAC (nativo o dentro de Ogg), el vídeo Theora, los capítulos de `.m4b`, la escritura de tags ID3 dentro de un contenedor Ogg y el cambio de las cinco `keywords` del manifiesto (fijadas por el spec `crate-metadata`).

## Capabilities

### New Capabilities

- `ogg-tagging`: lectura y escritura de metadatos en Ogg Vorbis: mapeo flag → clave, campos sin equivalente, carátula, letra, comentario obligatorio y borrado.
- `track-numbering`: sintaxis `N`/`N/T` de `--track` y `--season`, destino nativo del total en cada formato, visualización y borrado del par como unidad.
- `format-documentation`: coherencia de la documentación de los tres formatos, verificable.

### Modified Capabilities

- `openspec/specs/format-detection/spec.md`: se acepta Ogg Vorbis y el mensaje de formato no soportado pasa a enumerar MP3, MP4 y OGG.
- `openspec/specs/cli-output/spec.md`: el contrato del cierre del pipe y de los avisos por stderr cubre las tres rutas de visualización.
- `openspec/specs/mp3-tagging/spec.md`: el total de pista y temporada se escribe en `TRCK` y `TPOS` como `n/total`.
- `openspec/specs/m4a-tagging/spec.md`: el total se escribe en los átomos `trkn` y `disk`.

## Impact

- **Código:** `src/lib.rs`, `src/main.rs`, `src/tests.rs` y `tests/integration_test.rs`.
- **Fixtures nuevos:** `tests/fixtures/sample.ogg` y su script de regeneración.
- **Documentación:** `README.md`, `AGENTS.md`, `.github/copilot-instructions.md`, `Cargo.toml`.
- **Specs del repositorio** (capabilities nuevas y modificadas listadas arriba).
- **Dependencias:** ninguna nueva; se mantiene `lofty` 0.25.4 y `clap` 4.5.
- **Versión:** sube a 0.6.0 **automáticamente** por el prefijo `feat:` del pipeline de release, sin tocarla a mano.
- **Tests existentes:** los tests de MP3 y MP4 que hoy escriben pista y temporada sin total tendrán que adaptarse.
