# id3cli

CLI en Rust para añadir tags ID3 y carátulas a archivos MP3.

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
|--------|-------------|
| `-f, --file <FILE>` | Ruta del archivo MP3 (requerido) |
| `-t, --title <TITLE>` | Título de la canción |
| `-a, --artist <ARTIST>` | Artista (se puede repetir para múltiples artistas) |
| `-A, --album <ALBUM>` | Álbum |
| `-y, --year <YEAR>` | Año |
| `-g, --genre <GENRE>` | Género |
| `-T, --track <TRACK>` | Número de pista |
| `-d, --date <DATE>` | Fecha de grabación (YYYY-MM-DD o YYYY) |
| `-C, --copyright <COPYRIGHT>` | Copyright |
| `-c, --cover <COVER>` | Ruta del archivo JPG para la carátula |
| `-s, --show` | Mostrar todos los tags del archivo |
| `-h, --help` | Mostrar ayuda |

## Ejemplos

### Ver tags existentes

```bash
id3cli -f cancion.mp3 --show
```

### Añadir metadatos básicos

```bash
id3cli -f cancion.mp3 \
  --title "Bohemian Rhapsody" \
  --artist "Queen" \
  --album "A Night at the Opera" \
  --year 1975 \
  --genre "Rock" \
  --track 11
```

### Múltiples artistas (colaboraciones)

```bash
id3cli -f remix.mp3 \
  --title "Despacito Remix" \
  --artist "Luis Fonsi" \
  --artist "Daddy Yankee" \
  --artist "Justin Bieber"
```

Resultado: `Luis Fonsi; Daddy Yankee; Justin Bieber`

### Añadir carátula

```bash
id3cli -f cancion.mp3 --cover portada.jpg
```

### Metadata completa

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

### Solo actualizar algunos campos

Los tags existentes se preservan:

```bash
# Solo cambiar el año
id3cli -f cancion.mp3 --year 2026

# Solo añadir carátula
id3cli -f cancion.mp3 --cover nueva_portada.jpg
```

## Desarrollo

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

```
id3cli/
├── src/
│   └── main.rs          # Código principal
├── tests/
│   └── integration_test.rs  # Tests de integración
├── .github/
│   └── workflows/
│       └── release.yml  # Workflow para releases
├── Cargo.toml           # Dependencias
└── README.md
```

## Dependencias

- [`id3`](https://crates.io/crates/id3) - Lectura/escritura de tags ID3
- [`clap`](https://crates.io/crates/clap) - Parser de argumentos CLI

## Tests

El proyecto incluye 35 tests (22 unitarios + 13 de integración):

```bash
cargo test              # Ejecutar todos los tests
cargo test --lib        # Solo tests unitarios
cargo test --test '*'   # Solo tests de integración
```

## Licencia

MIT

## Contribuir

1. Fork el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/amazing`)
3. Commit tus cambios (`git commit -am 'Add amazing feature'`)
4. Push a la rama (`git push origin feature/amazing`)
5. Abre un Pull Request

## Roadmap

- [ ] Soporte para más formatos de imagen (PNG, WEBP)
- [ ] Modo batch para procesar múltiples archivos
- [ ] Eliminación de tags específicos
- [ ] Soporte para lyrics
- [ ] Binarios para Windows y macOS
