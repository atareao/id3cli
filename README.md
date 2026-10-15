# id3cli

CLI en Rust para añadir tags y carátulas a archivos **MP3** (tags ID3v2.4), **MP4** (átomos `ilst` estilo iTunes: `.m4a`, `.mp4`, `.m4b`) y **OGG Vorbis** (comentarios Vorbis).

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-271%20passing-brightgreen.svg)](https://github.com/atareao/id3cli)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Características

✨ **Completo y fácil de usar**

- 🎧 **MP3, MP4 y OGG:** escribe tags ID3v2.4 en MP3, átomos `ilst` en MP4 (`.m4a`, `.mp4`, `.m4b`) y comentarios Vorbis en OGG Vorbis
- 🔍 **Detección de formato por contenido:** el formato se decide por los *magic bytes* del archivo, nunca por su extensión; los formatos que no soporta se rechazan con un mensaje claro y sin tocar el archivo
- 📝 Añadir/modificar metadatos ID3: título, artista, álbum, año, género, pista, temporada, fecha, copyright
- 🎙️ **Soporte completo para podcasts:** compositor, subtítulo, artista original, artista del álbum, temporada (TPOS)
- 📃 Soporte para letras de canciones (lyrics) en formato USLT
- 🌐 Soporte para URL (sitio web oficial del artista) en formato WOAR
- 🍎 Soporte para metadatos de Apple: compilation, album sort, artist sort, title sort
- 🎨 Soporte para carátulas en **JPG, PNG y WEBP** con detección automática de tipo MIME
- 👥 Soporte para múltiples artistas (colaboraciones)
- 🗑️ **Comando remove dedicado** para eliminar tags específicos con nombres en inglés o español
- 👀 Visualizar todos los tags existentes con formato legible
- 🔄 Preserva metadatos existentes al actualizar campos específicos

## Formatos admitidos

El formato del archivo se determina **por su contenido** (sus *magic bytes* iniciales), nunca por la extensión del nombre. Por eso un MP3 renombrado a `.m4a` se sigue editando correctamente y un `.flac` no se trata por error como MP3.

| Estado | Formatos | Comportamiento |
| --- | --- | --- |
| **Soportado** | MP3 (con o sin cabecera `ID3`) | `show`, `edit` y `remove` operan sobre él; los tags se escriben como ID3v2.4 |
| **Soportado** | MP4 / M4A (`.m4a`, `.mp4`, `.m4b`): átomo `ftyp` en el offset 4, con cualquier brand | `show`, `edit` y `remove` operan sobre él; los metadatos se **leen y escriben** como átomos `ilst` estilo iTunes |
| **Soportado** | OGG Vorbis (`OggS`, versión `0`, primer paquete `\x01vorbis`) | `show`, `edit` y `remove` operan sobre él; los metadatos se escriben como comentarios Vorbis (`CLAVE=valor`) |
| **No soportado** | Opus (`OpusHead`) y Speex, en contenedor Ogg; FLAC (`fLaC`), WAV (`RIFF`), AIFF (`FORM`), APE (`MAC `), WavPack (`wvpk`) | El CLI sale con código distinto de cero, avisa en español y **no modifica el archivo** |
| **Indeterminado** | Contenido no reconocible o archivo demasiado corto | El CLI sale con código distinto de cero, avisa en español y **no modifica el archivo** |

Un contenedor **Ogg** es solo un sobre, así que el formato se decide por el **códec de su primer paquete**, no por la cabecera de página. Solo se acepta si ese primer paquete empieza por `\x01vorbis`; si empieza por `OpusHead` (Opus), por `Speex` o por `\x7fFLAC` (FLAC dentro de un Ogg), o si es el de Theora, el archivo se rechaza como formato fuera de alcance. Una página Ogg **truncada**, **sin primer paquete** o con un paquete **vacío** queda como *formato sin determinar*, y en ese caso tampoco se toca el archivo.

En todos los casos de error el archivo queda **intacto**: la detección ocurre antes de abrir el archivo para escritura, de modo que un formato no soportado nunca se toca.

**En resumen:**

- **Soportados:** MP3 (ID3v2.4), MP4 / M4A (`.m4a`, `.mp4`, `.m4b`, átomos `ilst` estilo iTunes) y OGG Vorbis (comentarios Vorbis). Los tres admiten `show`, `edit` y `remove` completos.
- **No soportados:** Opus y Speex (contenedores Ogg cuyo primer paquete no es Vorbis), FLAC (`fLaC`), WAV (`RIFF`), AIFF (`FORM`), APE (`MAC `) y WavPack (`wvpk`). El CLI sale con código de salida distinto de cero, avisa en español y **no modifica el archivo**.

## Instalación

### Desde binario (Linux)

Descarga la última release:

```bash
wget https://github.com/atareao/id3cli/releases/latest/download/id3cli-linux-x86_64
chmod +x id3cli-linux-x86_64
sudo mv id3cli-linux-x86_64 /usr/local/bin/id3cli
```

### Compilar desde código fuente

```bash
git clone https://github.com/atareao/id3cli.git
cd id3cli
cargo build --release
sudo cp target/release/id3cli /usr/local/bin/
```

### Desde crates.io

```bash
cargo install id3cli
```

> [!note] Si instalas desde crates.io, el paquete contiene el código fuente, los tests con sus ficheros de ejemplo, el README, la licencia y el changelog. La infraestructura interna del repositorio no se distribuye en el paquete.

## Uso

### Comandos disponibles

```bash
# Mostrar todos los tags del archivo
id3cli show <FILE>

# Editar tags del archivo
id3cli edit <FILE> [OPTIONS]

# Eliminar todos los tags del archivo (ID3v1 e ID3v2 en MP3, átomos ilst en MP4 y campos de usuario en OGG: el comentario de Vorbis se vacía pero no se elimina)
id3cli remove <FILE> --all

# Eliminar tags específicos del archivo
id3cli remove <FILE> [TAGS...]
```

### Opciones para el comando remove

| Argumento    | Descripción                                         |
| ------------ | --------------------------------------------------- |
| `<FILE>`     | Ruta del archivo de audio MP3, MP4 u OGG (requerido) |
| `-a, --all`  | Eliminar todos los metadatos del archivo: en MP3 desaparece el bloque ID3 (ID3v1 e ID3v2) y en MP4 el árbol de átomos `ilst`. En OGG el comentario de Vorbis es obligatorio en el contenedor, así que **no se elimina**: se **vacían** todos sus campos de usuario preservando el *vendor* (ni el *vendor* ni `ENCODER` figuran como metadatos). Si el archivo no tenía tags, muestra un aviso |
| `[TAGS...]`  | Tags a eliminar (uno o más argumentos posicionales) |

**Tags disponibles:** `title`, `artist`, `album`, `year`, `genre`, `track`, `season`, `date`, `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `cover`, `lyrics`, `url`, `compilation`, `album_sort`, `artist_sort`, `title_sort`

También acepta nombres en **español**: `título`, `artista`, `álbum`, `año`, `género`, `pista`, `temporada`, `fecha`, `compositor`, `subtítulo`, `carátula`, `letra`, `compilación`, etc.

### Opciones para el comando edit

| Opción                                | Descripción                                                  |
| ------------------------------------- | ------------------------------------------------------------ |
| `<FILE>`                              | Ruta del archivo de audio MP3, MP4 u OGG (argumento posicional, requerido) |
| `-t, --title <TITLE>`                 | Título de la canción                                         |
| `-a, --artist <ARTIST>`               | Artista (se puede repetir para múltiples artistas)           |
| `-A, --album <ALBUM>`                 | Álbum                                                        |
| `-y, --year <YEAR>`                   | Año                                                          |
| `-g, --genre <GENRE>`                 | Género                                                       |
| `-T, --track <TRACK>`                 | Número de pista, en forma `N` o `N/T` (número y total)       |
| `-S, --season <SEASON>`               | Temporada, en forma `N` o `N/T` (TPOS - útil para podcasts)  |
| `-d, --date <DATE>`                   | Fecha de grabación (YYYY-MM-DD o YYYY)                       |
| `-C, --copyright <COPYRIGHT>`         | Copyright                                                    |
| `--composer <COMPOSER>`               | Compositor (TCOM)                                            |
| `--subtitle <SUBTITLE>`               | Subtítulo o descripción (TIT3)                               |
| `--original-artist <ORIGINAL_ARTIST>` | Artista original (TOPE)                                      |
| `--album-artist <ALBUM_ARTIST>`       | Artista del álbum / Publisher (TPE2)                         |
| `-c, --cover <COVER>`                 | Ruta del archivo de imagen para la carátula (JPG, PNG, WEBP) |
| `-L, --lyrics <LYRICS>`               | Letra de la canción (lyrics)                                 |
| `-u, --url <URL>`                     | URL asociada (sitio web del artista, página oficial, etc.)   |
| `--compilation`                       | Marcar como compilación (Apple TCMP)                         |
| `--album-sort <ALBUM_SORT>`           | Orden de clasificación del álbum (Apple TSOA)                |
| `--artist-sort <ARTIST_SORT>`         | Orden de clasificación del artista (Apple TSOP)              |
| `--title-sort <TITLE_SORT>`           | Orden de clasificación del título (Apple TSOT)               |
| `-h, --help`                          | Mostrar ayuda                                                |

### 🖥️ Salida estándar, avisos y errores

La salida normal del CLI se escribe por **stdout** y los avisos y los errores por **stderr**, y esto es idéntico en MP3, MP4 y OGG. El tratamiento del pipe cerrado también lo es:

- **Pipe cerrado = final normal.** Si el consumidor cierra la salida antes de que el CLI termine de escribir (por ejemplo `id3cli show cancion.m4a | head -1`), el proceso termina **en silencio y con código de salida 0**: cerrar el pipe no es un error y no se imprime ningún pánico ni ningún mensaje.
- **Fallo real de escritura = error.** Si la escritura falla por otra causa (por ejemplo un destino sin espacio), el CLI avisa **en español por stderr** y termina con un código de salida **distinto de cero**.

> [!note] Un pipe cerrado no es un error, es un final normal de la salida. Solo los fallos reales de escritura se tratan como error, y los avisos y los errores siguen saliendo por stderr en ambos casos.

## Ejemplos de uso

### 👀 Ver tags existentes

```bash
id3cli show cancion.mp3
```

**Salida:**

```text
📋 Tags ID3 encontrados:

═══════════════════════════════════════
🎵 Título:    Yesterday
🎤 Artista:   The Beatles
💿 Álbum:     Help!
📅 Año:       1965
📆 Fecha:     1965-08-06
🎸 Género:    Rock
#️⃣  Pista:     2
©️  Copyright: © 1965 Apple Records
🖼️  Carátulas: 1 imagen(es)
   [1] Tipo: CoverFront, MIME: image/jpeg, Tamaño: 24.5 KB

📦 Total de frames: 9
═══════════════════════════════════════
```

**Ejemplo de podcast con temporada:**

```text
📋 Tags ID3 encontrados:

═══════════════════════════════════════
🎵 Título:    La historia del TCP/IP
🎤 Artista:   Tech Podcast
💿 Álbum:     Historia de Internet
📆 Fecha:     2026-01-22
🎸 Género:    Podcast
#️⃣  Pista:     5
📺 Temporada: 2
©️  Copyright: © 2026 CC BY 4.0
🎼 Compositor: Tech Podcast
📄 Subtítulo: Cómo se creó el protocolo TCP/IP

📦 Total de frames: 10
═══════════════════════════════════════
```

### ✏️ Añadir metadatos básicos

```bash
id3cli edit cancion.mp3 \
  --title "Bohemian Rhapsody" \
  --artist "Queen" \
  --album "A Night at the Opera" \
  --year 1975 \
  --genre "Rock" \
  --track 11
```

### 🎧 Editar un archivo MP4 (`.m4a`) u OGG Vorbis (`.ogg`)

Los archivos `.m4a` (y en general `.mp4` / `.m4b`) y los OGG Vorbis (`.ogg`) se editan con **exactamente los mismos comandos y flags** que un MP3; lo único que cambia es dónde se guardan los metadatos: átomos `ilst` en MP4 y comentarios Vorbis en OGG, en lugar de frames ID3:

```bash
id3cli edit cancion.m4a \
  --title "Bohemian Rhapsody" \
  --artist "Queen" \
  --album "A Night at the Opera" \
  --year 1975 \
  --genre "Rock" \
  --track 11
```

Y se consultan igual con `show`:

```bash
id3cli show cancion.m4a
```

**Salida:**

```text
📋 Tags encontrados:

═══════════════════════════════════════
🎵 Título:    Bohemian Rhapsody
🎤 Artista:   Queen
💿 Álbum:     A Night at the Opera
📅 Año:       1975
🎸 Género:    Rock
#️⃣  Pista:     11
═══════════════════════════════════════
```

> [!tip] El formato se detecta por el **contenido** del archivo, no por su extensión: un MP3 renombrado a `.m4a` se sigue editando como MP3 y viceversa.

### 🎼 Editar un archivo OGG Vorbis (`.ogg`)

En un `.ogg` la edición es igual de directa: los metadatos van a **comentarios Vorbis** (pares `CLAVE=valor`) en lugar de frames ID3 o átomos `ilst`.

```bash
id3cli edit cancion.ogg \
  --title "Bohemian Rhapsody" \
  --artist "Queen" \
  --album "A Night at the Opera" \
  --year 1975 \
  --genre "Rock" \
  --track 11
```

Y se consulta igual con `show`:

```bash
id3cli show cancion.ogg
```

**Salida:**

```text
📋 Tags encontrados:

═══════════════════════════════════════
🎵 Título:    Bohemian Rhapsody
🎤 Artista:   Queen
💿 Álbum:     A Night at the Opera
📅 Año:       1975
🎸 Género:    Rock
#️⃣  Pista:     11
═══════════════════════════════════════
```

### 📊 Indicar el total de pista y de temporada

`--track` y `--season` admiten tanto un número suelto (`N`) como el par número/total (`N/T`), en los tres formatos:

```bash
# Pista 7 de 12, temporada 1 de 2
id3cli edit disco.mp3 --track 7/12 --season 1/2

# Lo mismo en un OGG: el total va a `TRACKTOTAL` y `DISCTOTAL`
id3cli edit disco.ogg --track 7/12 --season 1/2
```

> [!warning] El cero en el total se comporta distinto según el contenedor. MP3 y OGG guardan el par como **texto**, así que `--track 7/0` se escribe y se muestra tal cual. En MP4 el átomo `trkn`/`disk` guarda el par en **16 bits**: el CLI avisa por stderr cuando el número o el total no son representables y, para no dejar un total huérfano, omite el par completo si no cabe el número (ver [Limitaciones conocidas en MP4](#limitaciones-conocidas-en-mp4)).

### 👥 Múltiples artistas (colaboraciones)

```bash
id3cli edit remix.mp3 \
  --title "Despacito Remix" \
  --artist "Luis Fonsi" \
  --artist "Daddy Yankee" \
  --artist "Justin Bieber"
```

**Resultado:** Los artistas se unen con `"; "` → `Luis Fonsi; Daddy Yankee; Justin Bieber`

### 📺 Añadir temporada (ideal para podcasts)

```bash
# Episodio con temporada
id3cli edit episodio.mp3 \
  --title "El origen de Internet" \
  --artist "Tech Podcast" \
  --album "Historia de la Tecnología" \
  --track 5 \
  --season 2 \
  --genre "Podcast"
```

**Resultado:** Temporada 2, Episodio 5 (S02E05) perfectamente identificado

### 🎨 Añadir carátula

Soporta **JPG, PNG y WEBP** con detección automática de tipo MIME:

```bash
# JPG o JPEG
id3cli edit cancion.mp3 --cover portada.jpg

# PNG
id3cli edit cancion.mp3 --cover portada.png

# WEBP
id3cli edit cancion.mp3 --cover portada.webp
```

### 🎶 Añadir letra (lyrics)

```bash
# Letra simple
id3cli edit cancion.mp3 -L "Primera línea
Segunda línea
Coro completo"

# Desde archivo
id3cli edit cancion.mp3 -L "$(cat letra.txt)"

# Con otros metadatos
id3cli edit cancion.mp3 -t "Canción" -a "Artista" -L "Letra completa..."
```

### 🌐 Añadir URL

```bash
# URL del sitio oficial del artista
id3cli edit cancion.mp3 -u "https://artista.com"

# Con otros metadatos
id3cli edit cancion.mp3 -t "Canción" -a "Artista" -u "https://artista.com/official"
```

### 🍎 Metadatos de Apple

Los metadatos de Apple son útiles para organizar bibliotecas musicales en iTunes y otros reproductores compatibles:

```bash
# Marcar como compilación (álbum recopilatorio)
id3cli edit cancion.mp3 --compilation

# Especificar orden de clasificación personalizado
id3cli edit cancion.mp3 \
  --title "A Hard Day's Night" \
  --artist "The Beatles" \
  --album "A Hard Day's Night" \
  --album-sort "Hard Day's Night, A" \
  --artist-sort "Beatles, The" \
  --title-sort "Hard Day's Night, A"

# Compilación con orden de clasificación
id3cli edit cancion.mp3 \
  --compilation \
  --album-sort "Greatest Hits" \
  --artist-sort "Various Artists"
```

**Frames utilizados:**

- `TCMP` - Compilation flag (1 = compilación)
- `TSOA` - Album sort order
- `TSOP` - Performer/Artist sort order
- `TSOT` - Title sort order

### 🎙️ Metadatos para Podcasts

Configuración completa para episodios de podcast con todas las etiquetas recomendadas:

```bash
id3cli edit episodio42.mp3 \
  --title "Episodio 42: Introducción a Rust" \
  --subtitle "Aprendiendo sobre ownership y borrowing" \
  --artist "Lorenzo" \
  --album "atareao con Linux" \
  --album-artist "Lorenzo" \
  --composer "Lorenzo" \
  --original-artist "Lorenzo" \
  --genre "Podcast" \
  --track 42 \
  --season 3 \
  --date "2026-01-22" \
  --copyright "© 2026 CC BY 4.0"
```

**Correspondencia con frames ID3v2:**

- `--title` → TIT2 (título del episodio)
- `--subtitle` → TIT3 (descripción corta)
- `--artist` → TPE1 (host/presentador)
- `--album` → TALB (nombre del podcast)
- `--album-artist` → TPE2 (publisher/creador)
- `--composer` → TCOM (autor)
- `--original-artist` → TOPE (artista original)
- `--genre` → TCON ("Podcast")
- `--track` → TRCK (número de episodio)
- `--season` → TPOS (temporada)
- `--date` → TDRC (fecha de publicación)
- `--copyright` → TCOP (licencia)

### 📦 Metadata completa

```bash
id3cli edit cancion.mp3 \
  --title "Yesterday" \
  --artist "The Beatles" \
  --album "Help!" \
  --year 1965 \
  --genre "Rock" \
  --track 2 \
  --date "1965-08-06" \
  --copyright "© 1965 Apple Records" \
  --cover cover.jpg
```

### 🔄 Actualizar campos específicos

Los tags existentes se preservan automáticamente:

```bash
# Solo cambiar el año
id3cli edit cancion.mp3 --year 2026

# Solo añadir carátula (preserva título, artista, etc.)
id3cli edit cancion.mp3 --cover nueva_portada.jpg

# Cambiar artista sin afectar otros tags
id3cli edit cancion.mp3 -a "Nuevo Artista"
```

### 🗑️ Eliminar tags específicos

Acepta nombres en **inglés o español**:

```bash
# Eliminar un tag
id3cli remove cancion.mp3 title

# Eliminar varios tags a la vez
id3cli remove cancion.mp3 title artist album

# Usar nombres en español
id3cli remove cancion.mp3 título artista

# Eliminar carátula
id3cli remove cancion.mp3 cover

# Eliminar letra
id3cli remove cancion.mp3 lyrics

# Eliminar URL
id3cli remove cancion.mp3 url

# Eliminar metadatos de Apple
id3cli remove cancion.mp3 compilation
id3cli remove cancion.mp3 album_sort artist_sort title_sort

# Usar nombres en español para metadatos Apple
id3cli remove cancion.mp3 compilación
id3cli remove cancion.mp3 orden-album orden-artista orden-titulo
```

**Tags disponibles para eliminar:**
`title`, `artist`, `album`, `year`, `genre`, `track`, `season`, `date`, `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `cover`, `lyrics`, `url`, `compilation`, `album_sort`, `artist_sort`, `title_sort`

### 🗑️ Eliminar todos los tags

Borra **físicamente** el bloque ID3v2 completo y el tag ID3v1 del archivo, dejándolo
como si nunca hubiera tenido tags (el audio permanece intacto). Si el archivo no
tenía tags ID3, se muestra un aviso.

> [!note] Al eliminar el **último** campo que quedaba, el bloque ID3v2 no se conserva vacío: desaparece del archivo, exactamente igual que con `--all`. El audio nunca se toca.

```bash
# Eliminar todos los tags de una sola vez
id3cli remove cancion.mp3 --all
```

**Salida:**

```text
# Si el archivo tenía tags ID3v2 e ID3v1:
✅ Tags ID3v1 e ID3v2 eliminados de 'cancion.mp3'

# Si solo tenía ID3v2:
✅ Tag ID3v2 eliminado de 'cancion.mp3'

# Si el archivo no tenía tags:
⚠️  El archivo 'cancion.mp3' no tenía tags ID3
```

### 🎯 Ejemplos adicionales del comando remove

```bash
# Eliminar todos los metadatos básicos
id3cli remove cancion.mp3 title artist album year genre

# Limpiar metadatos de podcast
id3cli remove episodio.mp3 season subtitle composer original_artist

# Eliminar solo elementos multimedia
id3cli remove cancion.mp3 cover lyrics url

# Resetear ordenes de clasificación de Apple
id3cli remove cancion.mp3 compilation album_sort artist_sort title_sort
```

---

## Referencia Rápida de Tags ID3v2

| Frame ID3v2 | Opción CLI          | Descripción           | Uso en Podcasts      |
| ----------- | ------------------- | --------------------- | -------------------- |
| TIT2        | `--title`           | Título principal      | Nombre del episodio  |
| TIT3        | `--subtitle`        | Subtítulo/Descripción | Descripción corta    |
| TPE1        | `--artist`          | Artista/Intérprete    | Host/Presentador     |
| TPE2        | `--album-artist`    | Artista del álbum     | Publisher/Creador    |
| TALB        | `--album`           | Álbum                 | Nombre del podcast   |
| TCOM        | `--composer`        | Compositor            | Autor de la obra     |
| TOPE        | `--original-artist` | Artista original      | Creador original     |
| TCON        | `--genre`           | Género                | "Podcast"            |
| TRCK        | `--track`           | Número de pista       | Número de episodio   |
| TPOS        | `--season`          | Disco/Parte           | Temporada            |
| TDRC        | `--date`            | Fecha de grabación    | Fecha de publicación |
| TCOP        | `--copyright`       | Copyright             | Licencia (CC BY 4.0) |
| TYER        | `--year`            | Año                   | Año de publicación   |
| USLT        | `--lyrics`          | Letras                | Transcripción        |
| WOAR        | `--url`             | URL oficial           | Sitio web            |
| APIC        | `--cover`           | Carátula              | Logo del podcast     |
| TCMP        | `--compilation`     | Compilación (Apple)   | -                    |
| TSOA        | `--album-sort`      | Orden álbum (Apple)   | -                    |
| TSOP        | `--artist-sort`     | Orden artista (Apple) | -                    |
| TSOT        | `--title-sort`      | Orden título (Apple)  | -                    |

---

## Mapeo de tags en MP4

En un `.m4a` (y en general en un contenedor MP4 con `ilst`) los metadatos no son frames ID3, sino átomos dentro de la ruta `moov.udta.meta.ilst`. La siguiente tabla recoge el mapeo entre cada flag del CLI y el átomo que se escribe:

| Flag CLI | Átomo MP4 |
| --- | --- |
| `--title` | `©nam` |
| `--artist` | `©ART` |
| `--album` | `©alb` |
| `--album-artist` | `aART` |
| `--year` | `©day` |
| `--date` | `©day` |
| `--genre` | `©gen` |
| `--track` | `trkn` |
| `--season` | `disk` |
| `--composer` | `©wrt` |
| `--copyright` | `cprt` |
| `--lyrics` | `©lyr` |
| `--cover` | `covr` |
| `--compilation` | `cpil` |
| `--title-sort` | `sonm` |
| `--artist-sort` | `soar` |
| `--album-sort` | `soal` |
| `--subtitle` | `----:com.apple.iTunes:SUBTITLE` |

- **`--year` y `--date` comparten `©day`.** Es un único átomo, así que no pueden convivir: si se indican ambos flags, **gana `--date`** y se avisa por stderr de que el año ha quedado absorbido por la fecha.
- **`--subtitle` se guarda en un átomo *freeform*** (`----:com.apple.iTunes:SUBTITLE`), no en un átomo estándar. Los átomos freeform usan un nombre con espacio de nombres, de ahí el prefijo `----:com.apple.iTunes:`.

### Limitaciones conocidas en MP4

Tres comportamientos se degradan al escribir metadatos en un MP4 y quedan documentados aquí de forma explícita, junto con una limitación del átomo numérico:

- **La carátula WebP no se muestra.** El átomo `covr` solo admite los tipos *well-known* JPEG (13), PNG (14), GIF (12) y BMP (27). Ante un `.webp` el CLI **no falla**: escribe la carátula dejando el tipo de dato **reservado (0)** y avisa por stderr de que esa carátula **no será visible en iTunes ni en macOS**. En MP3 el `.webp` sigue funcionando con normalidad vía `APIC`.
- **La letra pierde el código de idioma.** El átomo `©lyr` no tiene campo de idioma, así que la letra de un MP4 se guarda sin el `spa` que sí lleva el frame `USLT` de MP3. El texto se conserva **íntegro**; lo único que se pierde es la marca de idioma.
- **`--url` y `--original-artist` se omiten con aviso.** Ninguno de los dos tiene átomo equivalente en `ilst`. Sobre un MP4 el CLI **no falla ni inventa** un átomo *freeform* de reemplazo: avisa por stderr, omite ese campo, escribe el resto con normalidad y termina con código de salida cero. Si son los únicos flags indicados, no hay cambio real que escribir y el archivo **no se reescribe**. En MP3 se comportan con normalidad (`--url` → `WOAR`, `--original-artist` → `TOPE`, sin aviso).
- **El total de pista y de temporada se limita a 16 bits.** Los átomos `trkn` y `disk` guardan el par número/total en **16 bits**, así que solo admiten valores `1..=65535` y el `0` no es representable. Cuando el **número** de `--track` / `--season` no cabe, el CLI avisa por stderr y **omite el par completo** (número y total) para no dejar un total huérfano; cuando solo es el **total** el que no cabe, escribe el número y omite el total. En MP3 (`TRCK`/`TPOS`) y en OGG (`TRACKNUMBER`/`DISCNUMBER` con `TRACKTOTAL`/`DISCTOTAL`) el par se guarda como **texto**, de modo que allí no existe ese límite y un total `0` se escribe y se muestra tal cual (`7/0`).

---

## Mapeo de tags en OGG Vorbis

En un `.ogg` los metadatos son **comentarios Vorbis** (`VORBIS_COMMENT`): pares `CLAVE=valor` en texto plano dentro del contenedor Ogg, no frames ID3 ni átomos `ilst`. La siguiente tabla recoge el mapeo entre cada flag del CLI y la clave del comentario que se escribe:

| Flag CLI | Clave de comentario Vorbis |
| --- | --- |
| `--title` | `TITLE` |
| `--artist` | `ARTIST` |
| `--album` | `ALBUM` |
| `--album-artist` | `ALBUMARTIST` |
| `--year` | `YEAR` |
| `--date` | `DATE` |
| `--genre` | `GENRE` |
| `--track` | `TRACKNUMBER` |
| `--season` | `DISCNUMBER` |
| `--composer` | `COMPOSER` |
| `--copyright` | `COPYRIGHT` |
| `--lyrics` | `LYRICS` |
| `--cover` | `METADATA_BLOCK_PICTURE` |
| `--compilation` | `COMPILATION` |
| `--title-sort` | `TITLESORT` |
| `--artist-sort` | `ARTISTSORT` |
| `--album-sort` | `ALBUMSORT` |
| `--subtitle` | `SUBTITLE` |

- **El total de pista y de temporada van en claves propias.** Como `TRACKNUMBER` y `DISCNUMBER` guardan solo el número, el total de la forma `N/T` se escribe aparte, en `TRACKTOTAL` y `DISCTOTAL`. Si se borra el número, el CLI elimina también su total para no dejar una clave huérfana.
- **`--year` y `--date` conviven.** Son claves distintas (`YEAR` y `DATE`), así que a diferencia del átomo `©day` de MP4 no se pisan: un mismo archivo puede llevar las dos.

### Limitaciones conocidas en OGG

- **La carátula WebP no se degrada en OGG.** El bloque `METADATA_BLOCK_PICTURE` admite el tipo MIME tal cual se declara, así que un `.webp` se guarda con su tipo real y se muestra con normalidad: a diferencia del átomo `covr` de MP4, **no** queda con el tipo reservado (0) ni pierde visibilidad. En MP3 también funciona vía `APIC`.
- **La letra pierde el código de idioma.** `LYRICS` es un comentario de texto sin campo de idioma, así que la letra de un OGG se guarda sin el `spa` que sí lleva el frame `USLT` de MP3 (igual que ocurre con `©lyr` en MP4). El texto se conserva **íntegro**; lo único que se pierde es la marca de idioma.
- **`--url` y `--original-artist` se omiten con aviso.** Ninguno de los dos tiene clave equivalente en los comentarios Vorbis. Sobre un OGG el CLI **no falla ni inventa** una clave de reemplazo: avisa por stderr, omite ese campo, escribe el resto con normalidad y termina con código de salida cero. Si son los únicos flags indicados, no hay cambio real que escribir y el archivo **no se reescribe**. En MP3 se comportan con normalidad (`--url` → `WOAR`, `--original-artist` → `TOPE`, sin aviso).
- **El total de pista y temporada no tiene límite de tamaño.** `TRACKNUMBER`/`DISCNUMBER` con `TRACKTOTAL`/`DISCTOTAL` guardan el par como **texto**, así que admiten cualquier valor, incluido un total `0`, que se escribe y se muestra tal cual (`7/0`). Solo el átomo numérico de MP4 lo restringe a 16 bits (ver [Limitaciones conocidas en MP4](#limitaciones-conocidas-en-mp4)).

---

## Para Desarrolladores

### Requisitos

- Rust 1.85+ (edition 2024)
- Cargo

### Compilar

```bash
cargo build
```

### Ejecutar tests

```bash
cargo test
```

### Ejecutar en modo desarrollo

```bash
cargo run -- edit test.mp3 --title "Test"
```

### Formatear código

```bash
cargo fmt
```

### Linter

```bash
cargo clippy -- -D warnings
```

## Estructura del proyecto

```tree
id3cli/
├── src/
│   ├── lib.rs                     # Librería (2664 líneas) - lógica de negocio
│   ├── main.rs                    # CLI (870 líneas) - interfaz de comandos show/edit/remove
│   ├── output.rs                  # Salida estándar (49 líneas) - macro outln! y cierre del pipe
│   └── tests.rs                   # Tests unitarios (4706 líneas)
├── tests/
│   ├── integration_test.rs        # Tests de integración (5111 líneas)
│   └── fixtures/                  # Muestras reales (sample.mp3, sample.m4a y sample.ogg) y sus scripts de regeneración, incluido generate_sample_ogg.sh
├── .github/
│   ├── copilot-instructions.md    # Guía para AI coding agents
│   └── workflows/
│       ├── ci.yml                 # CI: tests, formato y linter en cada push a development y pull request
│       ├── release-prepare.yml    # Prepare Release: prepara la release al fusionar en main
│       └── release.yml            # release: publica la release y los binarios al crear un tag v*
├── Cargo.toml                     # Dependencias y metadata
├── README.md                      # Esta documentación
└── RELEASE.md                     # Proceso de release
```

## Dependencias

- [`lofty`](https://crates.io/crates/lofty) v0.25 - Lectura/escritura de metadatos de audio
- [`id3`](https://crates.io/crates/id3) v1.16 — Solo como dependencia de desarrollo: los tests de integración releen con esta librería los tags escritos por `lofty`, de modo que la verificación es entre librerías distintas. La producción no la usa.
- [`clap`](https://crates.io/crates/clap) v4.5 - Parser de argumentos CLI con derive macros

## Arquitectura técnica

**Módulos principales:**

- **src/lib.rs** - Librería reutilizable con todas las funciones de manipulación de tags
- **src/main.rs** - CLI con clap subcommands (show/edit/remove) para parsing de argumentos y orquestación
- **src/output.rs** - Capa de escritura de la salida estándar: el macro `outln!` y el tratamiento del pipe cerrado
- **src/tests.rs** - Tests unitarios para todas las funciones de la librería

**Funciones principales (MP3):**

- `apply_metadata()` - Aplica todos los tags de metadata al archivo MP3 (14 parámetros)
- `add_cover_art()` - Embebe imagen con detección automática de MIME type
- `add_lyrics()` - Añade letras en formato USLT
- `add_url()` - Añade URL oficial en formato WOAR
- `add_apple_metadata()` - Añade metadatos específicos de Apple
- `remove_tags()` - Elimina tags específicos (acepta inglés/español)
- `remove_all_id3()` - Borra físicamente el bloque ID3v2 y el tag ID3v1
- `display_tags()` - Muestra tags formateados con emojis

**Funciones principales (MP4):**

- `edit_mp4()` - Orquesta la edición de un archivo MP4
- `apply_metadata_mp4()` - Escribe los átomos `ilst` a partir de los flags del CLI
- `read_mp4_tag()` / `write_mp4_tag()` - Lectura y escritura del tag MP4
- `remove_tags_mp4()` - Elimina átomos concretos
- `remove_all_mp4()` - Elimina el árbol de metadatos del MP4
- `display_tags_mp4()` - Muestra los átomos de un MP4
- `MP4_ATOM_MAP` - Tabla de mapeo entre cada flag y su átomo

**Funciones principales (OGG Vorbis):**

- `edit_ogg()` - Orquesta la edición de un archivo OGG Vorbis
- `apply_metadata_ogg()` - Escribe los comentarios Vorbis a partir de los flags del CLI
- `read_vorbis_tag()` / `write_vorbis_tag()` - Lectura y escritura del comentario Vorbis
- `remove_tags_ogg()` - Elimina claves concretas del comentario
- `remove_all_ogg()` - Vacía el comentario preservando el vendor y el audio
- `display_tags_ogg()` - Muestra los comentarios de un OGG
- `VORBIS_COMMENT_MAP` - Tabla de mapeo entre cada flag y su clave de comentario

**Comunes a los tres formatos:**

- `detect_format()` / `detect_format_from_bytes()` - Detecta el formato de audio por el contenido (magic bytes), no por la extensión
- `detect_mime_type()` / `detect_cover_mime_type()` - Detecta el formato de la imagen de carátula

**Patrones de diseño:**

- Arquitectura modular con separación lib/CLI
- Funciones puras para lógica testeable
- Separación entre parsing CLI (clap) y lógica de negocio
- Referencias/slices en lugar de cloning innecesario
- Manejo de errores con `Result<T, E>` y mensajes en español

## Tests

El proyecto tiene **cobertura completa** con **271 tests** (160 unitarios + 111 de integración):

```bash
cargo test              # Ejecutar todos los tests (271)
cargo test --lib        # Solo tests unitarios (160)
cargo test --test '*'   # Solo tests de integración (111)
```

**Ejemplos de tests:**

- Aplicación de metadatos básicos y extendidos
- Detección de MIME types (JPG, PNG, WEBP)
- Detección del formato de audio por contenido (magic bytes), independiente de la extensión
- Rechazo de formatos no soportados sin modificar el archivo
- Múltiples artistas con separador correcto
- Temporada (season) para podcasts
- Lyrics, URLs y metadatos de Apple
- Eliminación de tags en inglés/español
- Preservación de metadata existente
- Validación de formatos no soportados
- Cierre del pipe como final normal de la salida (código 0 y sin mensajes) y fallo real de escritura como error
- Validación de los metadatos del paquete (`keywords` y `categories`)
- Tests end-to-end del CLI completo con podcasts
- OGG Vorbis: fixture real versionado (`sample.ogg`) y detección del contenedor por el códec de su primer paquete (Vorbis), no por la cabecera `OggS`
- Total de pista y temporada (`N/T`) en los tres formatos, incluida la conservación del total preexistente y el borrado del par número/total como una unidad
- Coherencia documental: el recuento real de tests y los tres formatos figuran igual en `README.md` y `Cargo.toml`

---

## Licencia

MIT - Vea el archivo [LICENSE](LICENSE) para más detalles.

## Autor

Desarrollado con 🦀 Rust

## Contribuir

1. Fork el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/amazing`)
3. Commit tus cambios (`git commit -am 'Add amazing feature'`)
4. Push a la rama (`git push origin feature/amazing`)
5. Abre un Pull Request

## Roadmap

- [x] Soporte para más formatos de imagen (PNG, WEBP)
- [x] Eliminación de tags específicos
- [x] Soporte para lyrics (letras de canciones)
- [x] Soporte para URLs (sitio web oficial)
- [x] Metadatos de Apple (compilation, sort orders)
- [x] Metadatos para podcasts (composer, subtitle, original artist, album artist)
- [x] Temporada (season/TPOS) para organizar podcasts por temporadas
- [x] Arquitectura modular (lib.rs separado del CLI)
- [ ] Modo batch para procesar múltiples archivos
- [ ] Binarios para Windows y macOS
- [x] Soporte para MP4 / M4A (átomos `ilst` estilo iTunes: `.m4a`, `.mp4`, `.m4b`)
- [x] Soporte para OGG Vorbis (comentarios de Vorbis: `TITLE`, `ARTIST`, `TRACKNUMBER`…)
- [ ] Soporte para Opus (contenedor Ogg con el códec Opus)
- [ ] Soporte para Speex (contenedor Ogg con el códec Speex)
- [ ] Soporte para FLAC (nativo y dentro del contenedor Ogg)
- [ ] Soporte para otros formatos de audio (WAV, AIFF, APE)
- [ ] Leer lyrics desde archivo externo (.lrc, .txt)
- [ ] GUI opcional con egui o similar
