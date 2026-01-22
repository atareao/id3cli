# Crear una Release

Este proyecto usa GitHub Actions para generar automáticamente binarios de Linux cuando se crea una nueva release.

## Pasos para crear una release:

### 1. Actualizar la versión en Cargo.toml

```toml
[package]
version = "0.2.0"  # Actualiza la versión
```

### 2. Crear commit con los cambios

```bash
git add Cargo.toml
git commit -m "Bump version to 0.2.0"
git push
```

### 3. Crear y subir un tag

```bash
# Crear tag anotado
git tag -a v0.2.0 -m "Release v0.2.0"

# Subir el tag
git push origin v0.2.0
```

### 4. Crear la release en GitHub

Opción A - Desde la interfaz web:
1. Ve a https://github.com/TU_USUARIO/id3cli/releases/new
2. Selecciona el tag `v0.2.0`
3. Añade título: "Release v0.2.0"
4. Añade descripción con los cambios
5. Click en "Publish release"

Opción B - Desde la línea de comandos (con GitHub CLI):
```bash
gh release create v0.2.0 \
  --title "Release v0.2.0" \
  --notes "## Cambios
  - Feature 1
  - Feature 2
  - Bug fixes"
```

### 5. El workflow se ejecutará automáticamente

GitHub Actions compilará el proyecto y subirá:
- `id3cli-linux-x86_64` - Binario ejecutable para Linux
- `id3cli-linux-x86_64.sha256` - Checksum SHA256 para verificación

## Verificar la build

Puedes ver el progreso en:
- https://github.com/TU_USUARIO/id3cli/actions

## Descargar el binario

Los usuarios pueden descargar el binario desde:
```bash
# Descargar última release
wget https://github.com/TU_USUARIO/id3cli/releases/latest/download/id3cli-linux-x86_64

# Dar permisos de ejecución
chmod +x id3cli-linux-x86_64

# Verificar checksum (opcional)
wget https://github.com/TU_USUARIO/id3cli/releases/latest/download/id3cli-linux-x86_64.sha256
sha256sum -c id3cli-linux-x86_64.sha256

# Usar el binario
./id3cli-linux-x86_64 --help
```

## Ejemplo de changelog

```markdown
## v0.2.0 (2026-01-22)

### Nuevas funcionalidades
- Soporte para múltiples artistas separados por ";"
- Añadido campo de fecha de grabación
- Añadido campo de copyright
- Comando `--show` para mostrar tags

### Mejoras
- 35 tests unitarios y de integración
- Mejor manejo de errores

### Cambios
- Los artistas ahora se separan con ";" en lugar de "/"
```
