# Design

## Problema

Hoy `b"fLaC"` vivía en `UNSUPPORTED_MAGIC` (src/lib.rs:926) y cualquier `.flac` se rechazaba con «formato no soportado». FLAC usa comentarios de Vorbis (`VorbisComments`, pares clave=valor) del mismo modo que OGG y Opus, así que todo el backend Vorbis (mapeo, aplicación, carátula, borrado, visualización) es reutilizable; cambian la detección, el despacho de lectura y el enrutado del CLI. La implementación del GREEN (verificada empíricamente con el binario, `metaflac` y `ffprobe`) matizó dos premisas del plan: en FLAC nativo lofty 0.25.4 materializa las carátulas como bloque PICTURE separado del archivo (no dentro del comentario) y elimina el bloque VORBIS_COMMENT al guardar un tag vacío (el vendor no se conserva). Las Decisiones 4 y 5 (renumeradas) reflejan esa evidencia.

## Decisión 1 — Detección por contenido (sin ambigüedad con Ogg)

`detect_format_from_bytes` gana la comprobación `b"fLaC"` en el offset 0 ANTES de la tabla de magics no soportados, y `b"fLaC"` sale de `UNSUPPORTED_MAGIC` junto con su comentario. El contenido manda, no la extensión: un `.flac` renombrado a `.mp3` se reconoce por sus bytes y un archivo que empiece por `fLaC` con cualquier extensión se enruta al `Backend::Flac`. `\x7fFLAC` dentro de un contenedor Ogg (códec FLAC en Ogg) es OTRO formato y sigue rechazado como `Unsupported`, documentado en el comentario de `UNSUPPORTED_MAGIC` y en los tests de códecs Ogg no soportados (donde ya vive junto a Speex). Evidencia: `UNSUPPORTED_MAGIC` (src/lib.rs:925-931), rama de detección Ogg (src/lib.rs:1063-1067), probe.rs de lofty:375 (`b"fLaC" => Ok(Some(FileType::Flac))`).

## Decisión 2 — `Backend::Flac` explícito en main.rs

Siguiendo la filosofía de `Backend::Ogg` (un backend que sirve a varios códecs del mismo mecanismo), FLAC recibe su propio `Backend::Flac` que enruta a las funciones Vorbis existentes (show/edit/remove comparten el cuerpo con `Backend::Ogg` — verificado en GREEN que el refactor trivial de fusionar ramas `Backend::Ogg | Backend::Flac` funciona y mantiene los mensajes correctos). Alternativa evaluada y descartada: reutilizar `Backend::Ogg` para FLAC — mentiría en los mensajes, acoplaría el nombre del contenedor al del backend y dificultaría el filtrado futuro (p. ej. la salida JSON de la Fase 3 debe nombrar `flac`).

## Decisión 3 — `read_vorbis_tag` materializa un comentario vacío para FLAC

`VorbisFile` y `OpusFile` garantizan comentario presente (`vorbis_comments()` devuelve `&VorbisComments`); `FlacFile::vorbis_comments()` devuelve `Option<&VorbisComments>` porque el bloque VORBIS_COMMENT es opcional en FLAC (src/flac/read.rs: campo `vorbis_comments_tag: Option<VorbisComments>` leído solo si existe el bloque). `read_vorbis_tag` (firma: `Result<VorbisComments, Box<dyn Error>>`) gana la rama `AudioFormat::Flac =>` que abre con `FlacFile::read_from(&mut File::open(path)?, ParseOptions::default())` y devuelve `vorbis_comments().cloned().unwrap_or_else(VorbisComments::new)`: un FLAC sin comentario se materializa como comentario vacío, con la MISMA semántica que `remove_all_ogg`. Así ninguna llamada existente cambia de firma y `edit` sobre un FLAC sin comentario «crea» el bloque al guardar (lofty inserta el bloque VORBIS_COMMENT si no existe).

## Decisión 4 — Carátula: bloque PICTURE separado del archivo (evidencia del GREEN)

Premisa inicial del plan («la carátula vive dentro del comentario como `METADATA_BLOCK_PICTURE`, igual que OGG/Opus») CORREGIDA por la verificación empírica del GREEN: el escritor de lofty 0.25.4 para FLAC materializa las pictures como bloque PICTURE separado del archivo (confirmado con `metaflac`: `type: 6 (PICTURE)`), y al releer pasan a `FlacFile::pictures()`, NO a `vorbis_comments().pictures()`. Consecuencias implementadas: (a) `edit --cover` reutiliza `push_ogg_cover` (sustitución por `remove_picture_type(CoverFront)` + `insert_picture` — verificado: segunda edición deja UN solo bloque PICTURE con la última imagen y roundtrip de bytes contra ella); (b) el CLI MUST mostrar la carátula en `show` leyéndola de los bloques PICTURE del archivo (una variante de display específica de la rama FLAC), no del tag; (c) los bloques PICTURE de archivos escritos por otras herramientas se leen y muestran igual (misma vía), lo que simplifica el alcance: no hay «camino del comentario» separado que conservar. El test de integración de carátula verifica contra `flac.pictures()` (MIME y bytes).

## Decisión 5 — `remove --all` elimina el bloque VORBIS_COMMENT completo (sin vendor)

Premisa inicial del plan («el vendor se conserva al vaciar, como en OGG/Opus») CORREGIDA por la verificación empírica: el escritor FLAC de lofty 0.25.4 NO emite un bloque VORBIS_COMMENT vacío (con vendor y sin campos) — lo descarta por completo. Por tanto, en FLAC nativo `remove --all` elimina el bloque VORBIS_COMMENT entero: los campos de usuario, las carátulas Y el vendor desaparecen. Es una degradación documentada de FLAC (en Ogg/Opus el comentario es obligatorio en el contenedor y el vendor se preserva). El resultado es un FLAC válido y reproducible (patrón `fLaC` y STREAMINFO intactos, comprobado con `ffprobe`). El spec `flac-tagging` y el e2e reflejan esta realidad; el README documentará la degradación junto a las de MP4/OGG.

## Decisión 6 — Fixture real con verificación estructural

`tests/fixtures/sample.flac` se genera con `ffmpeg` (`anullsrc` mono 48 kHz, 0,6 s, `-c:a flac`, `-map_metadata -1`, `-fflags +bitexact -flags:a +bitexact`) y se versiona; `generate_sample_flac.sh` lo regenera y verifica estructura (patrón `fLaC`, primer bloque STREAMINFO obligatorio, bloque VORBIS_COMMENT con vendor y SIN campos de usuario — ffmpeg NO escribe `ENCODER` en FLAC nativo, verificado; `ffprobe` codec=flac, `ffmpeg` decodifica por completo) — igual que `generate_sample_opus.sh` (nunca igualdad de bytes). `create_temp_flac()` en integration_test.rs copia el fixture a un temporal único.

## Decisión 7 — e2e: FLAC como quinto formato, claves de ffprobe medidas en GREEN

El runner común gana `flac` en `E2E_CODEC` (=`flac`), en el `case` de `e2e_gen_audio` (sin flags extra) y en `ffprobe_entries_for`. Para las claves de ffprobe de FLAC nativo NO se supone nada: durante GREEN se mide con `ffmpeg` 9.0.2 sobre un FLAC escrito por `id3cli` qué conjunto de entradas expone la metadata (`format_tags` o `stream_tags`) y la caja de las claves, y el resultado se registra en tasks.md y se fija en la tabla del spec `e2e-verification` (patrón ya usado para OGG/Opus: «medido»). `e2e_flac.sh` replica el flujo de `e2e_ogg.sh` (18 tags + url/original omitidos con aviso + carátula JPG/PNG con roundtrip y sustitución verificada por `show` + remove --all SIN vendor + audio reproducible) y el `.justfile` gana la recipe `e2e-flac` y añade `flac` al bucle de `e2e`.

## Decisión 8 — Mensajes del backend Vorbis generalizados

Los avisos/errores que decían «OGG» de forma excluyente («El tag 'x' no tiene clave equivalente en OGG; se omite.» y el error de lectura) se generalizan a «comentarios de Vorbis» o al formato real, porque el backend sirve a OGG, Opus y FLAC. Implementado en GREEN: `ogg_field_omitted_warning` → «en los comentarios de Vorbis», error de `read_vorbis_tag` → «no se pudo leer el comentario de Vorbis del archivo», y `report_ogg_error` renombrado a `report_vorbis_error` (ningún test fijaba los literales antiguos, verificado).

## Alternativas descartadas

- **Bump de lofty**: no hay versión posterior que cambie el comportamiento de bloques PICTURE/vendor en FLAC (0.25.4 es la última; la limitación es del diseño del escritor, no un bug). Post-procesar el archivo para reinsertar un bloque VORBIS_COMMENT con vendor sería pelear contra el escritor con riesgo alto; se descarta y se documenta la degradación.
- **Gestionar pictures solo desde el tag**: imposible, lofty las extrae al bloque PICTURE al leer; la lectura desde `FlacFile::pictures()` es el camino único (y más simple, porque unifica el tratamiento de carátulas propias y ajenas).
- **Renombrar las funciones `*_ogg` a `*_vorbis`**: más limpio a largo plazo pero toca TODAS las llamadas y tests (≈40 sitios) sin ganancia funcional; se difiere a un refactor futuro.
- **`Backend::Flac` reutilizando `Backend::Ogg`**: descartada en Decisión 2.

## Riesgos y mitigaciones

- **Claves de ffprobe de FLAC desconocidas a priori** → se miden en GREEN y se fijan con la medición (Decisión 7); el e2e no asume.
- **FLAC con ID3v2 además de Vorbis** (assets de lofty `flac_with_id3v2.flac`): lofty advierte que ese tag no se reescribe («This tag cannot be rewritten to the FLAC file»); el CLI lee/limpia solo el camino VorbisComments. Fuera de alcance, documentado.
- **`--year`/`--date` en FLAC**: mismo comportamiento Vorbis (claves separadas `YEAR`/`DATE`); el e2e lo mide igual que en OGG.
- **Carátula WebP**: en OGG/Opus no se degrada (MIME real); se hereda el comportamiento, sin aviso, igual que OGG.