# Design

## Context

`id3cli` escribe metadatos en MP3 (frames ID3v2.4 vía `lofty` 0.25.4), en MP4/M4A (átomos `ilst`) y en OGG (comentarios de Vorbis, pares `CLAVE=valor`). Este cambio añade **Opus como segundo códec del contenedor Ogg**: comparte el tag `VorbisComments` con OGG Vorbis, así que el backend de metadatos se reutiliza íntegramente y el coste se concentra en la detección del códec y en el despacho de lectura.

La detección del formato por magic bytes vive en `src/lib.rs`: `detect_format_from_bytes` lee un prólogo de 512 bytes (ya ampliado en el change de OGG) y decide el códec del contenedor Ogg por el **identificador del primer paquete de la primera página**, que empieza en el offset `27 + header[26]`: la cabecera de página ocupa 27 bytes y el byte 26 es el número de entradas de la tabla de segmentos. El enum `AudioFormat` clasifica el contenido —`Mp3`, `Mp4`, `OggVorbis`, `OggOpus`, `Unsupported`, `Unknown`— y este cambio añade la variante `OggOpus`.

`src/main.rs` contiene el enum `Backend` (`Mp3`, `Mp4`, `Ogg`) y `detect_supported_format`, que traduce el resultado de la detección a un backend; los mensajes en español que enumeran los formatos soportados pasan a decir «Formatos soportados: MP3, MP4, OGG y Opus.»

Los contratos vivos afectados por este cambio son `openspec/specs/format-detection/spec.md` (capability modificada: el contenedor Ogg reconoce dos códecs, `\x01vorbis` → Ogg Vorbis y `OpusHead` → Opus) y la capability nueva `opus-tagging` (el `OpusFile` de `lofty` con su `VorbisComments` interno, su mapeo de claves, sus degradaciones, la carátula, el comentario obligatorio y el borrado). La suite medida es de **281 tests (163 unitarios + 118 de integración)**, con baseline 273: el change aporta 1 test unitario de detección (`test_detect_format_ogg_opus`) y 7 tests de integración CLI sobre `sample.opus`, e invierte los tests que hoy esperan el rechazo de Opus.

## Goals / Non-Goals

**Goals**

- Aceptar **Opus dentro de Ogg** como segundo códec soportado del contenedor, con paridad de flags respecto a OGG Vorbis.
- Reutilizar el backend de comentarios de Vorbis **sin duplicar** una familia de funciones «espejo» para Opus.
- Mantener el contrato público: `read_vorbis_tag` conserva nombre, firma y tipo de retorno `VorbisComments`.
- Dejar la suite y la documentación actualizadas como contrato verificable (281 tests medidos).

**Non-Goals**

- Speex, FLAC (nativo o dentro de Ogg) y el vídeo Theora dentro del contenedor: siguen rechazados.
- Capítulos de `.m4b`.
- Metadatos de audio Opus que no viajen en el comentario de Vorbis.
- ID3 dentro de un contenedor Ogg.
- Renombrar la familia `*_ogg` / `*_vorbis`: la API pública no cambia.
- Nuevas dependencias: se mantiene `lofty` 0.25.4 y `clap` 4.5.

## Decisions

### 1. Detección por contenido: `OpusHead` como segundo códec del contenedor Ogg

`detect_format_from_bytes` decide el códec del contenedor Ogg por el identificador del primer paquete y ahora hace match sobre dos firmas: `\x01vorbis` → `AudioFormat::OggVorbis` y `OpusHead` —los 8 bytes `\x4f\x70\x75\x73\x48\x65\x61\x64`— → `AudioFormat::OggOpus`. El resto de identificadores del contenedor (Speex `Speex   `, FLAC `\x7fFLAC`, Theora `\x80theora`) → `AudioFormat::Unsupported`, y una página Ogg incompleta o un primer paquete sin cerrar → `Unknown`.

La decisión se toma **antes de tocar el archivo** y siempre por contenido, no por extensión: un `.opus` renombrado a `.ogg`, a `.mp3` o sin extensión se sigue reconociendo por el identificador de su primer paquete, igual que ya ocurre con el resto de formatos.

Alternativa descartada:

- **Decidir por extensión**: deja el comportamiento roto ante renombrados, y el proyecto ya decidió por contenido en `format-detection`.

### 2. `read_vorbis_tag` generalizado: despacho por códec con el mismo contrato

`read_vorbis_tag(path)` conserva su nombre, su firma y su retorno `VorbisComments`. Internamente detecta el códec vía `detect_format(path)` y despacha: `OggVorbis` → `VorbisFile::read_from` + `vorbis_comments()` y `OggOpus` → `OpusFile::read_from` + `vorbis_comments()`, clonando el comentario en ambos casos. Así las ~15 llamadas existentes no cambian: el tipo devuelto es el mismo y el pipeline show/edit/remove es idéntico para los dos códecs.

`write_vorbis_tag` no cambia: su guardado pasa por `save_to_path(path, WriteOptions::default())`, que inspecciona el contenedor real del archivo —Vorbis u Opus— y escribe el comentario en la posición que le corresponde a cada códec.

Alternativas descartadas:

- **Renombrar la familia `*_ogg` / `*_vorbis` a `*_ogg_comments`**: rompería la API pública y ~15 llamadas de tests para un beneficio puramente nominal; se prefirió actualizar los doc comments.
- **Usar `lofty::read_from` genérico para detectar Opus sin `detect_format`**: pierde el control explícito del códec y mezcla la responsabilidad de la detección con la de la lectura; la detección sigue siendo la única fuente de verdad del formato.

### 3. Un único `Backend::Ogg` sirve Vorbis y Opus

`detect_supported_format` enruta `Ok(AudioFormat::OggVorbis) => Backend::Ogg` y `Ok(AudioFormat::OggOpus) => Backend::Ogg`: no existe `Backend::Opus` ni ramas nuevas en show/edit/remove, y el CLI no distingue los dos códecs en ningún punto. Los mensajes de error pasan a «Formatos soportados: MP3, MP4, OGG y Opus.»

Alternativa descartada:

- **`Backend::Opus` separado en el CLI**: duplicaría las ramas show/edit/remove sin ningún beneficio (el pipeline de los dos códecs es el mismo) y obligaría a duplicar o parametrizar todas las funciones `*_ogg`.

### 4. Fixture sintético mínimo con modo `--check`

`tests/fixtures/sample.opus` versionado, de 272 bytes: `ffmpeg` con `anullsrc` (48 kHz, mono, 0,6 s), `libopus` y `-map_metadata -1 -fflags +bitexact -flags:a +bitexact`. Invariante de estructura: **sin campos de usuario** —solo el vendor `ffmpeg` y la clave `ENCODER` de fábrica—, de modo que los tests pre-escriben los tags que necesitan.

El script `tests/fixtures/generate_sample_opus.sh` añade un modo `--check ARCHIVO` además del modo de regeneración: permite probar casos negativos (por ejemplo, corromper el identificador del primer paquete) sin tocar el fixture versionado.

Alternativa descartada:

- **Generar `sample.opus` con etiquetas precargadas**: complica el script de regeneración, que exige «sin campos de usuario» como invariante de estructura; los tests pre-escriben lo que necesitan.

### 5. Los asserts de formato de integración comprueban bytes, no el enum

Los tests de integración CLI sobre `sample.opus` comprueban el formato por **bytes**, no por `AudioFormat::OggOpus`: nombran `OggS` + `OpusHead` en el offset `27 + header[26]` del archivo. Así la compilación de la suite no depende del enum mientras las tests unitarias de detección (`test_detect_format_ogg_opus`) cubren la variante `OggOpus`.

## Risks / Trade-offs

- **(a) Un solo backend frente a backends espejo**: se generalizó `read_vorbis_tag` (despacho por códec) en lugar de duplicar una familia `*_opus`, porque el tag es el mismo tipo y el pipeline show/edit/remove es idéntico. Coste asumido: `read_vorbis_tag` / `write_vorbis_tag` conservan nombres con raíz «vorbis» aunque sirvan a Opus, mitigado con doc comments actualizados.
- **(b) `read_vorbis_tag` lee el archivo dos veces**: primero con `detect_format(path)` para decidir el códec (abre y lee el prólogo) y luego con el lector concreto de `lofty`. Alternativas consideradas y descartadas: un `TaggedFile::read_from` genérico cambiaría el tipo devuelto y el contrato de 15+ llamadas, y exponer el lector Ogg interno acoplaría la API. La doble apertura es el coste aceptado por simplicidad y contrato estable.
- **(c) Fixture sintético mínimo**: `sample.opus` de 272 bytes sin campos de usuario, con vendor `ffmpeg` y `ENCODER` de fábrica; los tests pre-escriben los tags. El modo `--check ARCHIVO` del script de regeneración permite probar casos negativos sin alterar el fixture versionado.
- **(d) Asserts de formato por bytes en integración**: los tests de integración Opus comprueban `OggS` + `OpusHead` en `27 + header[26]` en lugar de `AudioFormat::OggOpus`, para no romper la compilación de la suite; el enum queda cubierto por las tests unitarias.
- **(e) Degradaciones idénticas a OGG Vorbis, heredadas del backend**: `--url` y `--original-artist` se omiten con aviso por stderr; la letra se guarda sin código de idioma; la carátula WebP **no** se degrada (MIME real en `METADATA_BLOCK_PICTURE`, a diferencia de `covr` en MP4); el comentario es obligatorio y `show` ignora el vendor y `ENCODER`; `remove --all` vacía el comentario preservando el vendor y dejando el audio intacto.
- **(f) La suite crece de 273 a 281 tests** (1 unitario + 7 de integración) y se invierten los tests que hoy esperan el rechazo de Opus; el recuento declarado en la documentación se actualiza al medido.
- **(g) La release publicará una versión menor automáticamente** por el prefijo `feat:` del commit del pipeline de release, sin tocarla a mano.

## Migration Plan

El orden de los grupos de `tasks.md`, y por qué:

1. **Baseline y red de seguridad**, en un commit propio antes de tocar producción — fija el recuento de 273 tests e invierte la expectativa de los tests que hoy rechazan Opus (`test_detect_format_ogg_other_codecs` en unitarios y el caso `"opus"` de `test_cli_rejects_ogg_other_codecs_and_leaves_file_untouched` en integración).
2. **Detección de Opus**, test-first — `test_detect_format_ogg_opus` falla (RED) antes de ampliar `detect_format_from_bytes` con la firma `OpusHead` del primer paquete.
3. **Fixture `sample.opus`**, en su propio commit — el insumo verificable de los grupos siguientes, con `generate_sample_opus.sh` y su modo `--check ARCHIVO`.
4. **Backend generalizado**, test-first — el despacho por códec de `read_vorbis_tag` (Vorbis u Opus) y la confirmación de que `write_vorbis_tag` no cambia.
5. **CLI**, test-first — el enrutado de `AudioFormat::OggOpus` a `Backend::Ogg` y el mensaje «Formatos soportados: MP3, MP4, OGG y Opus.»
6. **Tests de integración sobre `.opus`**, test-first — `show`, `edit` y `remove` sobre `sample.opus` (metadatos de texto, carátula, total `N/T` y borrado), con asserts de formato por bytes.
7. **Documentación y cierre**, con sus tests de coherencia — `README.md`, `AGENTS.md` y `.github/copilot-instructions.md` sacan Opus de las listas de formatos no soportados y lo incorporan a las de soportados; verificación final del recuento medido (281 = 163 + 118).

Cada grupo funcional MUST empezar con un test que falla (RED) antes de la implementación (GREEN), y ninguna casilla se marca sin la verificación indicada en ella.