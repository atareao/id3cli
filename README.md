# id3cli

CLI en Rust para añadir tags y carátulas a archivos **MP3** (tags ID3v2.4), **MP4** (átomos `ilst` estilo iTunes: `.m4a`, `.mp4`, `.m4b`), **OGG** (comentarios Vorbis: Vorbis y Opus en el contenedor Ogg) y **FLAC** (comentarios Vorbis, `fLaC` nativo).

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-329%20passing-brightgreen.svg)](https://github.com/atareao/id3cli)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Características

✨ **Completo y fácil de usar**

- 🎧 **MP3, MP4, OGG, Opus y FLAC:** escribe tags ID3v2.4 en MP3, átomos `ilst` en MP4 (`.m4a`, `.mp4`, `.m4b`) y comentarios Vorbis en OGG (Vorbis y Opus) y en FLAC
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
| **Soportado** | OGG Opus (`OggS`, versión `0`, primer paquete `OpusHead`) | `show`, `edit` y `remove` operan sobre él; los metadatos se escriben como comentarios Vorbis (`CLAVE=valor`), con el mismo mapeo que OGG Vorbis |
| **Soportado** | FLAC nativo (`fLaC` en el offset 0) | `show`, `edit` y `remove` operan sobre él; los metadatos se escriben como comentarios Vorbis (`CLAVE=valor`), con el mismo mapeo que OGG y Opus |
| **No soportado** | Speex en contenedor Ogg; WAV (`RIFF`), AIFF (`FORM`), APE (`MAC `), WavPack (`wvpk`) | El CLI sale con código distinto de cero, avisa en español y **no modifica el archivo** |
| **Indeterminado** | Contenido no reconocible o archivo demasiado corto | El CLI sale con código distinto de cero, avisa en español y **no modifica el archivo** |

Un contenedor **Ogg** es solo un sobre, así que el formato se decide por el **códec de su primer paquete**, no por la cabecera de página. Solo se acepta si ese primer paquete empieza por `\x01vorbis` (Vorbis) o por `OpusHead` (Opus).

Si el primer paquete empieza por `Speex`, por `\x7fFLAC` (FLAC dentro de un Ogg) o si es el de Theora, el archivo se rechaza como formato fuera de alcance. Ese `\x7fFLAC` se sigue rechazando: el FLAC **nativo** (`fLaC` fuera de un Ogg) sí está soportado, pero como códec *dentro* del contenedor Ogg no entra en el alcance. Una página Ogg **truncada**, **sin primer paquete** o con un paquete **vacío** queda como *formato sin determinar*, y en ese caso tampoco se toca el archivo.

En todos los casos de error el archivo queda **intacto**: la detección ocurre antes de abrir el archivo para escritura, de modo que un formato no soportado nunca se toca.

**En resumen:**

- **Soportados:** MP3 (ID3v2.4), MP4 / M4A (`.m4a`, `.mp4`, `.m4b`, átomos `ilst` estilo iTunes), OGG Vorbis, Opus y FLAC (comentarios de Vorbis). Los CINCO admiten `show`, `edit` y `remove` completos.
- **No soportados:** Speex (en contenedor Ogg), WAV (`RIFF`), AIFF (`FORM`), APE (`MAC `) y WavPack (`wvpk`). El CLI sale con código de salida distinto de cero, avisa en español y **no modifica el archivo**.

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
# Mostrar los tags de uno o más archivos
id3cli show <FILE>...
# Mostrar todos los tags del archivo en JSON (una línea, para scripts y jq)
id3cli show <FILE> --json

# Editar tags del archivo
id3cli edit <FILE>... [OPTIONS]

# Eliminar todos los tags del archivo (ID3v1 e ID3v2 en MP3, átomos ilst en MP4 y campos de usuario en OGG: el comentario de Vorbis se vacía pero no se elimina)
id3cli remove <FILE>... --all

# Eliminar tags específicos del archivo
id3cli remove <FILE> [TAGS...]

# Generar el script de completions para el shell indicado (bash, zsh o fish)
id3cli completions <bash|zsh|fish>
```

### Opciones para el comando remove

| Argumento    | Descripción                                         |
| ------------ | --------------------------------------------------- |
| `<FILE>`     | Ruta del archivo de audio MP3, MP4 u OGG (requerido) |
| `-a, --all`  | Eliminar todos los metadatos del archivo: en MP3 desaparece el bloque ID3 (ID3v1 e ID3v2) y en MP4 el árbol de átomos `ilst`. En OGG el comentario de Vorbis es obligatorio en el contenedor, así que **no se elimina**: se **vacían** todos sus campos de usuario preservando el *vendor* (ni el *vendor* ni `ENCODER` figuran como metadatos). Si el archivo no tenía tags, muestra un aviso |
| `[TAGS...]`  | Tags a eliminar (uno o más argumentos posicionales) |

**Tags disponibles:** `title`, `artist`, `album`, `year`, `genre`, `track`, `season`, `date`, `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `cover`, `lyrics`, `url`, `compilation`, `album_sort`, `artist_sort`, `title_sort`

También acepta nombres en **español**: `título`, `artista`, `álbum`, `año`, `género`, `pista`, `temporada`, `fecha`, `compositor`, `subtítulo`, `carátula`, `letra`, `compilación`, etc.

### Opciones para el comando show

| Opción     | Descripción                                                                                                   |
| ---------- | ------------------------------------------------------------------------------------------------------------- |
| `<FILE>`   | Ruta del archivo de audio MP3, MP4, OGG, Opus o FLAC (argumento posicional, requerido)                        |
| `-j, --json` | Emite los metadatos como **un único objeto JSON compacto en una línea** con salto de línea final, adecuado para `jq` y scripts |

**Contrato del JSON** (claves estables): el objeto tiene la forma `{ "formato": "...", "tags": {...} }`. `formato` es el códec real detectado por contenido —`mp3`, `mp4`, `ogg`, `opus` o `flac`— y en el contenedor Ogg se distingue `ogg` (Vorbis) de `opus` (OpusHead). `tags` es un objeto con **todas** las claves siempre presentes (`null` si el campo está ausente): `title`, `artists`, `album`, `year`, `date`, `genre`, `track`, `season`, `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `url`, `lyrics`, `compilation`, `album_sort`, `artist_sort`, `title_sort`, `pictures`. `artists` y `pictures` son siempre arrays (nunca `null`); `compilation` es siempre booleana; `track` y `season` son objetos `{ "number": N, "total": N|null }` o `null`; `year` es número o `null`; `lyrics` es el texto completo sin recorte; cada `pictures[i]` es `{ "type": "...", "mime": "...|null", "size_bytes": N }`.

**Sin tags:** si el contenedor no tiene tag (MP3 sin ID3v2, MP4 sin `ilst`), `tags` vale `null` con exit 0 y **stderr vacío** (el aviso ⚠️ del modo legible queda suprimido: el consumidor es una máquina). En OGG/Opus/FLAC el comentario de Vorbis siempre existe, así que un comentario vacío se emite como objeto `tags` con todas las claves `null`/vacías. Los errores (formato no soportado, archivo inexistente) siguen yendo por stderr en español con exit ≠ 0 y **nada** en stdout. El cierre de pipe por el consumidor termina con exit 0 sin pánico, igual que el modo legible.

### Compleciones para bash, zsh y fish

El subcomando `completions` emite el script de completions del CLI en la salida
estándar, generado automáticamente desde la definición real del parser
(`clap_complete`), de modo que siempre refleja los subcomandos y opciones
actuales de `show`, `edit`, `remove` y el propio `completions`.

```bash
# bash: guardar el script y cargarlo desde .bashrc (o .bash_completion.d)
id3cli completions bash > ~/.bash_completion.d/id3cli
# zsh: con compinit activo
id3cli completions zsh > ~/.zsh/completions/_id3cli
# fish:
id3cli completions fish > ~/.config/fish/completions/id3cli.fish
```

El script generado es la salida nativa del generador (`bash -n` lo valida): su
línea de registro para bash es `complete -F _id3cli -o nosort -o bashdefault -o
default id3cli`. Un shell no soportado (p. ej. `powershell`) falla con un error
en español por stderr que nombra los shells válidos, y si el consumidor cierra
el pipe la escritura termina como final normal (exit 0).

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

### Modo batch (varios archivos)

`show`, `edit` y `remove` aceptan **uno o más archivos** en una sola invocación; la
operación se aplica a todos en el orden dado. Con un único archivo el comportamiento
es exactamente el de siempre (byte a byte).

```bash
# Editar varios archivos a la vez
id3cli edit --artist "Artista" *.mp3
# Mostrar varios archivos
id3cli show track1.mp3 track2.mp3
# Eliminar todos los tags de varios archivos
id3cli remove --all *.mp3
```

Con `show` y más de un archivo, cada bloque se antecede de una cabecera `=== <ruta> ===`.
Con `--json` y más de un archivo, la salida es **NDJSON**: una línea JSON por archivo,
consumible con `jq -s` (devuelve el array de objetos):

```bash
id3cli show --json *.mp3 | jq -s '.[].tags.title'
```

El modo batch termina con exit 0 solo si **todos** los archivos se procesan; si alguno
falla (ruta inexistente, formato no soportado), el resto se procesa igual, los fallos se
informan por stderr en español y la salida final es distinta de cero. Sin archivos, clap
responde con el error de argumento requerido (exit 2). Los avisos por archivo (sin tags,
degradaciones MP4/OGG) conservan su contrato y no cuentan como fallo.

### 🖥️ Salida estándar, avisos y errores

La salida normal del CLI se escribe por **stdout** y los avisos y los errores por **stderr**, y esto es idéntico en MP3, MP4 y OGG. El tratamiento del pipe cerrado también lo es:

- **Pipe cerrado = final normal.** Si el consumidor cierra la salida antes de que el CLI termine de escribir (por ejemplo `id3cli show cancion.m4a | head -1`), el proceso termina **en silencio y con código de salida 0**: cerrar el pipe no es un error y no se imprime ningún pánico ni ningún mensaje.
- **Fallo real de escritura = error.** Si la escritura falla por otra causa (por ejemplo un destino sin espacio), el CLI avisa **en español por stderr** y termina con un código de salida **distinto de cero**.

> [!note] Un pipe cerrado no es un error, es un final normal de la salida. Solo los fallos reales de escritura se tratan como error, y los avisos y los errores siguen saliendo por stderr en ambos casos.

En el modo `show --json` la salida es exclusivamente el objeto JSON: los avisos informativos del modo legible (p. ej. «no se encontraron metadatos») se suprimen para no contaminar el stdout, y el stderr queda vacío salvo errores reales (que terminan con exit ≠ 0). La ausencia de bytes ANSI y de texto fuera del objeto JSON es parte del contrato.

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

### 🎼 Editar un archivo OGG Vorbis (`.ogg`) u Opus (`.opus`)

En un `.ogg` (y en un `.opus`) la edición es igual de directa: los metadatos van a **comentarios Vorbis** (pares `CLAVE=valor`) en lugar de frames ID3 o átomos `ilst`. El contenedor Ogg cubre Vorbis y Opus, y **ambos comparten el mismo mapeo** de claves.

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

`--cover` **sustituye** la carátula frontal existente: una segunda edición sobre el mismo archivo deja exactamente **una** carátula, la de la última imagen indicada (no acumula). En MP3, OGG y Opus se conservan las carátulas de otros tipos (p. ej. `BackCover`); en MP4, donde `lofty` relee toda carátula como tipo genérico, se sustituye la lista `covr` completa. Para eliminar la carátula: `id3cli edit cancion.mp3 --remove cover`.

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
- **La carátula se sustituye por completo en MP4.** El átomo `covr` admite varias imágenes, pero el CLI relee toda carátula MP4 como tipo genérico (`PictureType::Other`), así que una segunda edición de `--cover` sustituye la **lista `covr` completa** (elimina las anteriores y escribe la nueva). En MP3, OGG y Opus solo se sustituye la carátula frontal (`CoverFront`), conservando los demás tipos.

---

## Mapeo de tags en OGG y FLAC (comentarios Vorbis)

En un `.ogg`, un `.opus` y un `.flac` los metadatos son **comentarios Vorbis** (`VORBIS_COMMENT`): pares `CLAVE=valor` en texto plano. El contenedor Ogg cubre Vorbis y Opus, y el archivo FLAC nativo usa el mismo mecanismo: **los tres comparten el mismo mapeo** y el mismo tipo de comentario. La siguiente tabla recoge el mapeo entre cada flag del CLI y la clave del comentario que se escribe:

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

- **La carátula WebP no se degrada en OGG ni en FLAC.** El bloque `METADATA_BLOCK_PICTURE` admite el tipo MIME tal cual se declara, así que un `.webp` se guarda con su tipo real y se muestra con normalidad: a diferencia del átomo `covr` de MP4, **no** queda con el tipo reservado (0) ni pierde visibilidad. En MP3 también funciona vía `APIC`.
- **La letra pierde el código de idioma.** `LYRICS` es un comentario de texto sin campo de idioma, así que la letra de un OGG se guarda sin el `spa` que sí lleva el frame `USLT` de MP3 (igual que ocurre con `©lyr` en MP4). El texto se conserva **íntegro**; lo único que se pierde es la marca de idioma.
- **`--url` y `--original-artist` se omiten con aviso.** Ninguno de los dos tiene clave equivalente en los comentarios Vorbis. Sobre un OGG el CLI **no falla ni inventa** una clave de reemplazo: avisa por stderr, omite ese campo, escribe el resto con normalidad y termina con código de salida cero. Si son los únicos flags indicados, no hay cambio real que escribir y el archivo **no se reescribe**. En MP3 se comportan con normalidad (`--url` → `WOAR`, `--original-artist` → `TOPE`, sin aviso).
- **El total de pista y temporada no tiene límite de tamaño.** `TRACKNUMBER`/`DISCNUMBER` con `TRACKTOTAL`/`DISCTOTAL` guardan el par como **texto**, así que admiten cualquier valor, incluido un total `0`, que se escribe y se muestra tal cual (`7/0`). Solo el átomo numérico de MP4 lo restringe a 16 bits (ver [Limitaciones conocidas en MP4](#limitaciones-conocidas-en-mp4)).
- **Las tres degradaciones aplican igual a Opus y a FLAC.** El contenedor Ogg cubre Vorbis y Opus, y el archivo FLAC nativo usa el mismo comentario de Vorbis, así que todo lo anterior (carátula WebP con su MIME real, `LYRICS` sin código de idioma y `--url`/`--original-artist` omitidos con aviso) se comporta idéntico en los tres.
- **`remove --all` elimina el bloque VORBIS_COMMENT completo en FLAC.** A diferencia de OGG y Opus —donde el comentario es obligatorio en el contenedor y el vendor del codificador se conserva al vaciar—, en un archivo FLAC nativo `lofty` no escribe un comentario vacío, de modo que `remove --all` deja el archivo sin bloque de comentario: los campos, las carátulas y el vendor desaparecen. El archivo sigue siendo un FLAC válido y reproducible.

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
│   ├── lib.rs                     # Librería (2702 líneas) - lógica de negocio
│   ├── main.rs                    # CLI (871 líneas) - interfaz de comandos show/edit/remove
│   ├── output.rs                  # Salida estándar (49 líneas) - macro outln! y cierre del pipe
│   └── tests.rs                   # Tests unitarios (4888 líneas)
├── tests/
│   ├── integration_test.rs        # Tests de integración (5438 líneas)
│   └── fixtures/                  # Muestras reales (sample.mp3, sample.m4a, sample.ogg y sample.opus) y sus scripts de regeneración, incluidos generate_sample_ogg.sh y generate_sample_opus.sh
├── scripts/
│   └── e2e/                       # Verificación e2e local: runner común (e2e_lib.sh) y un script por formato (e2e_mp3.sh, e2e_m4a.sh, e2e_ogg.sh, e2e_opus.sh)
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

El proyecto tiene **cobertura completa** con **329 tests** (178 unitarios + 151 de integración):

```bash
cargo test              # Ejecutar todos los tests (329)
cargo test --lib        # Solo tests unitarios (178)
cargo test --test '*'   # Solo tests de integración (151)
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
- OGG Vorbis u Opus: fixture real versionado (`sample.ogg` y `sample.opus`) y detección del contenedor por el códec de su primer paquete (Vorbis u Opus), no por la cabecera `OggS`
- Opus: mismo backend de comentarios Vorbis que OGG, con detección por `OpusHead` y flujo completo `show`/`edit`/`remove` verificado contra el fixture real
- FLAC: fixture real versionado (`sample.flac`), `show`/`edit`/`remove` completos con comentarios Vorbis, carátula como bloque PICTURE con roundtrip y sustitución, y `remove --all` sin vendor (degradación documentada)
- Total de pista y temporada (`N/T`) en los tres formatos, incluida la conservación del total preexistente y el borrado del par número/total como una unidad
- Coherencia documental: el recuento real de tests y los formatos de `Cargo.toml` (MP3, MP4, OGG y FLAC) figuran igual en `README.md` y `Cargo.toml`, y la coherencia del soporte de Opus se verifica contra el código

---

## Verificación e2e local

Además de la suite de Rust, el repositorio incluye un tooling de **verificación e2e local** (`scripts/e2e/`) que ejercita el binario real de extremo a extremo: escribe y relee los **20 tags** de `VALID_TAG_NAMES` y las carátulas **JPG y PNG** sobre los **cinco formatos soportados** (MP3, MP4/M4A, OGG, Opus y FLAC). Es una verificación de regresión de lectura/escritura del flujo completo `edit → show → remove`, contrastada con un **lector independiente**; está pensada para no sobrecargar el repositorio, así que **todo el material se genera al vuelo y nada se versiona**.

**Qué verifica** (tres capas por formato):

- **Capa 1 — `id3cli show`:** cada uno de los 20 tags de `VALID_TAG_NAMES` (`title`, `artist`, `album`, `year`, `genre`, `track`, `season`, `date`, `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `cover`, `lyrics`, `url`, `compilation`, `album_sort`, `artist_sort`, `title_sort`) se escribe con `edit` y se relee con `show` campo a campo.
- **Capa 2 — `ffprobe` como lector independiente:** la metadata se consulta con `ffprobe` (`format_tags` en MP3/MP4, `stream_tags` en OGG/Opus); en OGG/Opus se comprueba además en los **bytes crudos** del comentario Vorbis (incluido el *vendor* preservado tras `remove --all`).
- **Capa 3 — `id3cli show --json` con `jq` como parser independiente:** el runner ejecuta el modo JSON y valida el documento con `jq` (oráculo independiente, como `ffprobe`): JSON válido y sin bytes ANSI, `.formato` igual al códec real del formato (`mp3`, `m4a`→`mp4`, `ogg`, `opus`, `flac`), una muestra de campos por ruta jq (p. ej. `.tags.track.number`), el caso `tags: null` en MP3/MP4 sin tags con stderr vacío, y la falsabilidad de una aserción JSON (corromper el formato esperado → exit 1).
- **Cobertura por formato:** MP3 verifica los **20/20** tags; MP4, OGG, Opus y FLAC verifican los 18 relevantes y comprueban que `url` y `original_artist` **se omiten con aviso** (stderr) y no aparecen en `ffprobe`. En FLAC, además, `remove --all` elimina el bloque VORBIS_COMMENT completo y el vendor **no** se conserva (a diferencia de OGG/Opus, donde el vendor es obligatorio; degradación documentada en la sección de degradaciones).
- **Carátulas:** `cover.jpg` y `cover.png` (generadas con `testsrc`) se añaden **cada una sobre audio recién generado** y se comparan **byte a byte** contra la extracción de `ffmpeg` (`-map 0:v:0 -c copy`), es decir, *roundtrip* de bytes.
- **Borrado:** `remove` de un campo con el resto intacto y `remove --all` (en MP3 el bloque ID3v2 **desaparece físicamente** del archivo; en OGG/Opus el comentario Vorbis se **vacía preservando el vendor**), con el audio final **reproducible** (duración conservada).

**Cómo se ejecuta** (desde la raíz del repositorio):

```bash
just e2e            # los 5 formatos, completions y batch en serie; se detiene con el exit code del primero que falle
just e2e-mp3        # un formato concreto: e2e-mp3 | e2e-m4a | e2e-ogg | e2e-opus | e2e-flac | e2e-completions | e2e-batch
```

O directamente, sin `just`:

```bash
scripts/e2e/e2e_mp3.sh
```

**Flags** (los aceptan los scripts directos y las recipes por extensión; en `just e2e` el binario lo fija la pre-construcción de la recipe — `--keep` y otros extras sí se reenvían):

| Flag       | Efecto                                                                                                                     |
| ---------- | -------------------------------------------------------------------------------------------------------------------------- |
| `--bin RUTA` | Usa el binario indicado en lugar de construir. Por defecto se ejecuta `cargo build --locked` **una vez** y se usa `target/debug/id3cli`. `just e2e` pre-construye una vez y pasa `--bin target/debug/id3cli` a los scripts; los scripts invocados directamente construyen si no reciben `--bin` |
| `--keep`   | Conserva el directorio temporal al terminar, para depurar                                                                  |

```bash
just e2e                                              # pre-construye una vez (cargo build --locked) y verifica los 5 formatos + completions + batch
just e2e --keep                                       # lo mismo conservando los temporales
just e2e-opus --bin target/release/id3cli --keep      # opus con binario custom, conservando el temporal
scripts/e2e/e2e_m4a.sh --keep
```

**Localidad:** todo el material (audio de silencio `anullsrc` y carátulas `testsrc`) se genera bajo `/tmp/opencode/e2e/<ext>/` dentro del directorio temporal de la ejecución (`mktemp -d`) y se elimina con un `trap` al terminar; con `--keep` el temporal se conserva y el script indica dónde queda. El repositorio queda sin binarios ni imágenes generados.

**Requisitos:** `ffmpeg`, `ffprobe` y `jq` en el `PATH` (y `cargo` para el build por defecto). Si falta alguna, el script falla con exit 1 y un mensaje en español indicando la herramienta ausente.

**Códigos de salida:** `0` = todas las aserciones pasan; `1` = fallo de alguna aserción, con un mensaje en español que identifica el **nombre de la aserción fallida** y el valor esperado frente al obtenido.

> [!note] En `just` >= 1.58 las recipes reenvían sus argumentos con un parámetro variádico (`*args` + `{{args}}`), porque el shell de la recipe ya no recibe los argumentos vía `"$@"`. Por eso `just e2e --bin …` recibe `--bin` sin necesidad del separador `--`.

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
- [x] Salida JSON de `show` (`--json`, para scripts y consumidores tipo máquina)
- [x] Compleciones para bash, zsh y fish (subcomando `completions`)
- [x] Modo batch (`<FILE>...` en `show`, `edit` y `remove`)
- [ ] Binarios para Windows y macOS
- [x] Soporte para MP4 / M4A (átomos `ilst` estilo iTunes: `.m4a`, `.mp4`, `.m4b`)
- [x] Soporte para OGG Vorbis (comentarios de Vorbis: `TITLE`, `ARTIST`, `TRACKNUMBER`…)
- [x] Soporte para Opus (contenedor Ogg con el códec Opus; mismas claves de comentario Vorbis que OGG)
- [ ] Soporte para Speex (contenedor Ogg con el códec Speex)
- [x] Soporte para FLAC (nativo: `fLaC`, comentarios Vorbis)
- [ ] Soporte para FLAC dentro del contenedor Ogg (códec `\x7fFLAC`)
- [ ] Soporte para otros formatos de audio (WAV, AIFF, APE)
- [ ] Leer lyrics desde archivo externo (.lrc, .txt)
- [ ] GUI opcional con egui o similar
