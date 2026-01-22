# id3cli

CLI en Rust para añadir tags ID3 y carátulas a archivos MP3.

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-92%20passing-brightgreen.svg)](https://github.com/TU_USUARIO/id3cli)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Características

✨ **Completo y fácil de usar**

- 📝 Añadir/modificar metadatos ID3: título, artista, álbum, año, género, pista, fecha, copyright
- 🎵 **Soporte completo para podcasts:** compositor, subtítulo, artista original, artista del álbum
- 📃 Soporte para letras de canciones (lyrics) en formato USLT
- 🌐 Soporte para URL (sitio web oficial del artista) en formato WOAR
- 🍎 Soporte para metadatos de Apple: compilation, album sort, artist sort, title sort
- 🎨 Soporte para carátulas en **JPG, PNG y WEBP** con detección automática de tipo MIME
- 👥 Soporte para múltiples artistas (colaboraciones)
- 🗑️ Eliminar tags específicos con nombres en inglés o español
- 👀 Visualizar todos los tags existentes con formato legible
- 🔄 Preserva metadatos existentes al actualizar campos específicos

## Instalación

### Desde binario (Linux)

Descarga la última release:

```bash
wget https://github.com/TU_USUARIO/id3cli/releases/latest/download/id3cli-linux-x86_64
chmod +x id3cli-linux-x86_64
sudo mv id3cli-linux-x86_64 /usr/local/bin/id3cli
```

### Compilar desde código fuente

```bash
git clone https://github.com/TU_USUARIO/id3cli.git
cd id3cli
cargo build --release
sudo cp target/release/id3cli /usr/local/bin/
```

## Uso

```bash
id3cli [OPTIONS] --file <FILE>
```

### Opciones disponibles

| Opción | Descripción |
| ------ | ----------- |
| `-f, --file <FILE>` | Ruta del archivo MP3 (requerido) |
| `-t, --title <TITLE>` | Título de la canción |
| `-a, --artist <ARTIST>` | Artista (se puede repetir para múltiples artistas) |
| `-A, --album <ALBUM>` | Álbum |
| `-y, --year <YEAR>` | Año |
| `-g, --genre <GENRE>` | Género |
| `-T, --track <TRACK>` | Número de pista |
| `-d, --date <DATE>` | Fecha de grabación (YYYY-MM-DD o YYYY) |
| `-C, --copyright <COPYRIGHT>` | Copyright |
| `--composer <COMPOSER>` | Compositor (TCOM) |
| `--subtitle <SUBTITLE>` | Subtítulo o descripción (TIT3) |
| `--original-artist <ORIGINAL_ARTIST>` | Artista original (TOPE) |
| `--album-artist <ALBUM_ARTIST>` | Artista del álbum / Publisher (TPE2) |
| `-c, --cover <COVER>` | Ruta del archivo de imagen para la carátula (JPG, PNG, WEBP) |
| `-L, --lyrics <LYRICS>` | Letra de la canción (lyrics) |
| `-u, --url <URL>` | URL asociada (sitio web del artista, página oficial, etc.) |
| `--compilation` | Marcar como compilación (Apple TCMP) |
| `--album-sort <ALBUM_SORT>` | Orden de clasificación del álbum (Apple TSOA) |
| `--artist-sort <ARTIST_SORT>` | Orden de clasificación del artista (Apple TSOP) |
| `--title-sort <TITLE_SORT>` | Orden de clasificación del título (Apple TSOT) |
| `-r, --remove <TAG>` | Eliminar tags específicos (se puede repetir) |
| `-s, --show` | Mostrar todos los tags del archivo |
| `-h, --help` | Mostrar ayuda |

## Ejemplos de uso

### 👀 Ver tags existentes

```bash
id3cli -f cancion.mp3 --show
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

### ✏️ Añadir metadatos básicos

```bash
id3cli -f cancion.mp3 \
  --title "Bohemian Rhapsody" \
  --artist "Queen" \
  --album "A Night at the Opera" \
  --year 1975 \
  --genre "Rock" \
  --track 11
```

### 👥 Múltiples artistas (colaboraciones)

```bash
id3cli -f remix.mp3 \
  --title "Despacito Remix" \
  --artist "Luis Fonsi" \
  --artist "Daddy Yankee" \
  --artist "Justin Bieber"
```

**Resultado:** Los artistas se unen con `"; "` → `Luis Fonsi; Daddy Yankee; Justin Bieber`

### 🎨 Añadir carátula

Soporta **JPG, PNG y WEBP** con detección automática de tipo MIME:

```bash
# JPG o JPEG
id3cli -f cancion.mp3 --cover portada.jpg

# PNG
id3cli -f cancion.mp3 --cover portada.png

# WEBP
id3cli -f cancion.mp3 --cover portada.webp
```

### 🎶 Añadir letra (lyrics)

```bash
# Letra simple
id3cli -f cancion.mp3 -L "Primera línea
Segunda línea
Coro completo"

# Desde archivo
id3cli -f cancion.mp3 -L "$(cat letra.txt)"

# Con otros metadatos
id3cli -f cancion.mp3 -t "Canción" -a "Artista" -L "Letra completa..."
```

### 🌐 Añadir URL

```bash
# URL del sitio oficial del artista
id3cli -f cancion.mp3 -u "https://artista.com"

# Con otros metadatos
id3cli -f cancion.mp3 -t "Canción" -a "Artista" -u "https://artista.com/official"
```

### 🍎 Metadatos de Apple

Los metadatos de Apple son útiles para organizar bibliotecas musicales en iTunes y otros reproductores compatibles:

```bash
# Marcar como compilación (álbum recopilatorio)
id3cli -f cancion.mp3 --compilation

# Especificar orden de clasificación personalizado
id3cli -f cancion.mp3 \
  --title "A Hard Day's Night" \
  --artist "The Beatles" \
  --album "A Hard Day's Night" \
  --album-sort "Hard Day's Night, A" \
  --artist-sort "Beatles, The" \
  --title-sort "Hard Day's Night, A"

# Compilación con orden de clasificación
id3cli -f cancion.mp3 \
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
id3cli -f episodio42.mp3 \
  --title "Episodio 42: Introducción a Rust" \
  --subtitle "Aprendiendo sobre ownership y borrowing" \
  --artist "Lorenzo" \
  --album "atareao con Linux" \
  --album-artist "Lorenzo" \
  --composer "Lorenzo" \
  --original-artist "Lorenzo" \
  --genre "Podcast" \
  --track 42 \
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
- `--date` → TDRC (fecha de publicación)
- `--copyright` → TCOP (licencia)

### 📦 Metadata completa

```bash
id3cli -f cancion.mp3 \
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
id3cli -f cancion.mp3 --year 2026

# Solo añadir carátula (preserva título, artista, etc.)
id3cli -f cancion.mp3 --cover nueva_portada.jpg

# Cambiar artista sin afectar otros tags
id3cli -f cancion.mp3 -a "Nuevo Artista"
```

### 🗑️ Eliminar tags específicos

Acepta nombres en **inglés o español**:

```bash
# Eliminar un tag
id3cli -f cancion.mp3 --remove title

# Eliminar varios tags a la vez
id3cli -f cancion.mp3 -r title -r artist -r album

# Usar nombres en español
id3cli -f cancion.mp3 -r título -r artista

# Eliminar carátula
id3cli -f cancion.mp3 --remove cover

# Eliminar letra
id3cli -f cancion.mp3 -r lyrics

# Eliminar URL
id3cli -f cancion.mp3 -r url

# Eliminar metadatos de Apple
id3cli -f cancion.mp3 -r compilation
id3cli -f cancion.mp3 -r album_sort -r artist_sort -r title_sort

# Usar nombres en español para metadatos Apple
id3cli -f cancion.mp3 -r compilación
id3cli -f cancion.mp3 -r orden-album -r orden-artista -r orden-titulo
```

**Tags disponibles para eliminar:**
`title`, `artist`, `album`, `year`, `genre`, `track`, `date`, `copyright`, `composer`, `subtitle`, `original_artist`, `album_artist`, `cover`, `lyrics`, `url`, `compilation`, `album_sort`, `artist_sort`, `title_sort`

---

## Referencia Rápida de Tags ID3v2

| Frame ID3v2 | Opción CLI | Descripción | Uso en Podcasts |
|-------------|------------|-------------|-----------------|
| TIT2 | `--title` | Título principal | Nombre del episodio |
| TIT3 | `--subtitle` | Subtítulo/Descripción | Descripción corta |
| TPE1 | `--artist` | Artista/Intérprete | Host/Presentador |
| TPE2 | `--album-artist` | Artista del álbum | Publisher/Creador |
| TALB | `--album` | Álbum | Nombre del podcast |
| TCOM | `--composer` | Compositor | Autor de la obra |
| TOPE | `--original-artist` | Artista original | Creador original |
| TCON | `--genre` | Género | "Podcast" |
| TRCK | `--track` | Número de pista | Número de episodio |
| TDRC | `--date` | Fecha de grabación | Fecha de publicación |
| TCOP | `--copyright` | Copyright | Licencia (CC BY 4.0) |
| TYER | `--year` | Año | Año de publicación |
| USLT | `--lyrics` | Letras | Transcripción |
| WOAR | `--url` | URL oficial | Sitio web |
| APIC | `--cover` | Carátula | Logo del podcast |
| TCMP | `--compilation` | Compilación (Apple) | - |
| TSOA | `--album-sort` | Orden álbum (Apple) | - |
| TSOP | `--artist-sort` | Orden artista (Apple) | - |
| TSOT | `--title-sort` | Orden título (Apple) | - |

---

## Para Desarrolladores

### Requisitos

- Rust 1.70+ (edition 2024)
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
cargo run -- -f test.mp3 --title "Test"
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
│   └── main.rs                    # Código principal (761 líneas)
├── tests/
│   └── integration_test.rs        # Tests de integración (19 tests)
├── .github/
│   ├── copilot-instructions.md    # Guía para AI coding agents
│   └── workflows/
│       └── release.yml            # CI/CD para releases automáticas
├── Cargo.toml                     # Dependencias y metadata
├── README.md                      # Esta documentación
└── RELEASE.md                     # Proceso de release
```

## Dependencias

- [`id3`](https://crates.io/crates/id3) v1.16.4 - Lectura/escritura de tags ID3v2
- [`clap`](https://crates.io/crates/clap) v4.5 - Parser de argumentos CLI con derive macros

## Arquitectura técnica

**Funciones principales:**

- `apply_metadata()` - Aplica todos los tags de metadata al archivo
- `add_cover_art()` - Embebe imagen con detección automática de MIME type
- `remove_tags()` - Elimina tags específicos (acepta inglés/español)
- `detect_mime_type()` - Detecta formato de imagen por extensión
- `display_tags()` - Muestra tags formateados con emojis

**Patrones de diseño:**

- Funciones puras para lógica testeable
- Separación entre parsing CLI (clap) y lógica de negocio
- Referencias/slices en lugar de cloning innecesario
- Manejo de errores con `Result<T, E>` y mensajes en español

## Tests

El proyecto tiene **cobertura completa** con 63 tests (38 unitarios + 25 de integración):

```bash
cargo test              # Ejecutar todos los tests (52)
cargo test --lib        # Solo tests unitarios (33)
cargo test --test '*'   # Solo tests de integración (19)
```

**Ejemplos de tests:**

- Detección de MIME types (JPG, PNG, WEBP)
- Múltiples artistas con separador correcto
- Eliminación de tags en inglés/español
- Preservación de metadata existente
- Validación de formatos no soportados
- Tests end-to-end del CLI completo

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
- [ ] Modo batch para procesar múltiples archivos
- [ ] Binarios para Windows y macOS
- [ ] Soporte para otros formatos de audio (FLAC, M4A)
- [ ] Leer lyrics desde archivo externo (.lrc, .txt)
