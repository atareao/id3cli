# agentes-id3cli-convenciones.md — convenciones del proyecto (extracto literal del AGENTS.md previo)

> Extracto literal del `AGENTS.md` previo (poda 2026-10-31). Texto tal cual, sin resumen.

## Key conventions

- Multiple artists joined with `"; "` separator
- Apple metadata via `.set_text()` frames: TCMP (`"1"`), TSOA, TSOP, TSOT
- Lyrics stored in USLT frame, language code `"spa"`
- Cover art MIME auto-detected from file extension: `.jpg`/`.jpeg` → `image/jpeg`, `.png` → `image/png`, `.webp` → `image/webp`
- `remove_tags()` accepts both English and Spanish names (title/título, cover/carátula, etc.)
- Error messages in Spanish
- Integration tests use `create_temp_mp3()`, `create_temp_m4a()`, `create_temp_ogg()`, `create_temp_opus()`, `create_temp_flac()` y `create_temp_fixture()` helpers + `cleanup_file()`
- El formato se decide por **contenido** (magic bytes), nunca por la extensión: `detect_format()` / `detect_format_from_bytes()`, y las lecturas de `lofty` vía `read_from()`, nunca `read_from_path()`. El contenedor Ogg se decide por el **códec de su primer paquete**, no por la cabecera de página: Vorbis y Opus están soportados
- Formatos no soportados (Speex, WAV, AIFF, APE y WavPack): salida distinta de cero, mensaje en español y archivo intacto. El contenedor Ogg **no** se rechaza por ser Ogg: se rechaza cuando el primer paquete de su códec no es de un códec soportado del contenedor (Speex, FLAC, Theora…)
- MP3 tags are always written as ID3v2.4 through `lofty`; MP4 metadata is written as iTunes-style `ilst` atoms y OGG como comentarios de Vorbis (tabla de mapeo de cada formato en README.md)
- OGG se escribe con el tipo concreto `VorbisComments`, no con el `Tag` genérico —que descarta las claves sin mapeo y normaliza la caja de las que sí mapean—; `VORBIS_COMMENT_MAP` es la única fuente del mapeo. El comentario es obligatorio en el contenedor: el *vendor* y `ENCODER` no se muestran como metadatos, `edit` conserva las claves ajenas (incluida `ENCODER`) y el vendor, y `remove --all` vacía el comentario y solo preserva el vendor (obligatorio, no se puede eliminar del contenedor)
- En MP4, `--year` y `--date` comparten el átomo `©day`: si se indican ambos gana `--date` y se avisa por stderr; `--subtitle` va a un átomo *freeform* (`----:com.apple.iTunes:SUBTITLE`)
- Degradaciones documentadas en MP4: carátula WebP en `covr` con tipo de dato reservado (0) más aviso por stderr; `©lyr` sin código de idioma; `--url` y `--original-artist` se omiten con aviso y nunca se escriben como átomo *freeform*
- Degradaciones documentadas en OGG: la carátula WebP **no** se degrada (se escribe en `METADATA_BLOCK_PICTURE` con su MIME real y sin aviso, a diferencia de `covr` en MP4); `LYRICS` pierde el código de idioma, igual que `©lyr` en MP4; `--url` y `--original-artist` se omiten con aviso por stderr y nunca se escribe una clave inventada. En MP3 no ocurre ninguna de las tres
- FLAC nativo se soporta por `fLaC` (offset 0); el códec `\x7fFLAC` dentro del contenedor Ogg NO: se rechaza como los demás códecs Ogg no soportados. La carátula de FLAC viaja como bloque PICTURE del archivo (`read_flac_pictures`), `remove --all` elimina el bloque VORBIS_COMMENT completo (vendor no conservado — degradación documentada), y `--url`/`--original-artist` se omiten con aviso igual que en OGG
- `--track` y `--season` aceptan `N` o `N/T`. En MP3 y OGG el total se guarda como **texto** (`TRCK`/`TPOS` como `n/total`; `TRACKTOTAL`/`DISCTOTAL` como claves) y admite el cero, mientras que en MP4 el átomo guarda el par en 16 bits y el CLI avisa por stderr cuando el número o el total no son representables (`1..=65535`)
- Las escrituras pasan por `tag.save_to_path(file, WriteOptions::default())`
- `id3` es **solo dev-dependency**: los tests de integración releen con esa librería lo que escribe `lofty`, de modo que la verificación es entre librerías distintas. Producción no la usa
- Al eliminar el último campo restante, el bloque ID3v2 no se conserva vacío: desaparece del archivo, igual que con `remove --all`
- La salida normal se escribe con `outln!` y **nunca** con `println!`, porque `println!` entra en pánico si el consumidor cierra el pipe; los avisos y errores van con `eprintln!` y no se tocan
- La integración continua ejecuta `cargo` con `--locked`, de modo que si `Cargo.toml` y `Cargo.lock` no coinciden la CI falla en lugar de actualizar el lock en silencio; por eso, al añadir o cambiar dependencias, el lock MUST commitearse en el mismo cambio
- El binario precompilado se construye para `x86_64-unknown-linux-musl` y sale **estático** (sin `NEEDED`, sin símbolos `GLIBC_*`), de modo que no depende de la libc del sistema; antes se construía para `x86_64-unknown-linux-gnu` y exigía glibc ≥ 2.34 (medido el 2026-10-25: 4 entradas `NEEDED` y `GLIBC_2.34` como versión más alta), así que no arrancaba en Ubuntu 20.04, Debian 11, RHEL 8 ni CentOS 7. El release publica además el binario desnudo `id3cli-linux-x86_64`, que es la ruta que documenta el README; `release.yml` y `ci.yml` verifican el enlazado estático antes de publicar, y `just build-musl` lo construye en local
- El pipeline de release empuja la rama y el tag en un **único** `git push --atomic` con reintentos: si no lo consigue, falla de forma explícita antes de publicar, de modo que un fallo deja `main` sin commit de release y sin tag en lugar de a medias (con dos push separados, el servidor llegó a rechazar el tag con `missing necessary objects` mientras la rama ya se había actualizado)
- El paquete publicado se limita a la lista blanca `include` de `Cargo.toml`, y los patrones de primer nivel van anclados con `/` porque `include` usa la semántica de gitignore: sin la barra inicial, `"README.md"` habría arrastrado al paquete un README del andamiaje de `openspec/`
