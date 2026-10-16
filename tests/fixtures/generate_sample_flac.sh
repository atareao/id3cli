#!/usr/bin/env bash
#
# Regenera `tests/fixtures/sample.flac`, el fixture FLAC nativo mínimo usado por
# los tests del change `flac-support` (soporte de audio FLAC).
#
# El fixture debe ser un FLAC real (no bytes fabricados) para que `lofty` lo
# parsee como `FlacFile` y `ffprobe` lo reconozca, con la marca `fLaC`, el
# bloque STREAMINFO como primer bloque de metadatos y un comentario de Vorbis
# (vendor del codificador y **sin** campos de usuario: `TITLE`, `ARTIST`…), de
# modo que la escritura ejercite la creación del bloque desde cero.
#
# Medido: `ffmpeg` escribe siempre el vendor (`ffmpeg`), el STREAMINFO como
# primer bloque y un bloque VORBIS_COMMENT sin campos de usuario; a diferencia
# de los contenedores Ogg, **no** escribe ninguna clave `ENCODER` en FLAC
# nativo, así que aquí no se exige. Por eso el fichero NO es reproducible byte
# a byte entre versiones de `ffmpeg` y la verificación de este script es
# **estructural**, nunca por igualdad de bytes contra el fixture versionado.
#
# Uso:
#   tests/fixtures/generate_sample_flac.sh                # regenera y verifica
#   tests/fixtures/generate_sample_flac.sh --check FILE   # solo verifica FILE
#
# Requisitos: ffmpeg, ffprobe y python3.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$HERE/sample.flac"

# Verificación estructural de un FLAC nativo: marca `fLaC` en el offset 0,
# primer bloque de metadatos STREAMINFO, bloque VORBIS_COMMENT con vendor no
# vacío y cero campos de usuario, `ffprobe` como audio FLAC y decodificación
# completa. Falla con su mensaje y código 1 ante cualquier caso negativo.
check_flac() {
  local file="$1"

  python3 - "$file" <<'PY'
import struct
import sys

path = sys.argv[1]
data = open(path, "rb").read()


def fail(msg):
    raise SystemExit("ERROR: " + msg)


if len(data) < 8:
    fail("el fichero es demasiado corto para ser un FLAC")

# 1. Marca de FLAC nativo en el offset 0 (`fLaC` con la `f` minúscula, no la
#    variante de FLAC encapsulado en ID3, que empieza por `ID3`).
if data[0:4] != b"fLaC":
    fail("la marca fLaC no está en el offset 0")

# 2. Recorre los bloques de metadatos desde el offset 4. Cada cabecera son 4
#    bytes: [bit «last»][7 bits tipo][longitud 24 bits big-endian], y los
#    bloques se navegan encadenando `4 + longitud` hasta el que marca `last`.
off = 4
blocks = {}
first_type = data[4] & 0x7F
if first_type != 0:
    fail("el primer bloque de metadatos no es STREAMINFO (tipo %d)" % first_type)
while True:
    header = data[off:off + 4]
    if len(header) < 4:
        fail("cabecera de bloque de metadatos truncada en el offset %d" % off)
    is_last = bool(header[0] & 0x80)
    block_type = header[0] & 0x7F
    length = int.from_bytes(header[1:4], "big")
    body = data[off + 4:off + 4 + length]
    if len(body) < length:
        fail("el bloque de metadatos de tipo %d está truncado en el offset %d"
             % (block_type, off))
    blocks.setdefault(block_type, []).append(body)

    if block_type == 0 and length != 34:
        fail("la longitud del STREAMINFO no es 34, es %d" % length)
    off += 4 + length
    if is_last:
        break

# 3. Debe existir un bloque VORBIS_COMMENT (tipo 4) con vendor no vacío y cero
#    campos de usuario. Layout igual que los comentarios de Vorbis: vendor
#    (le32) + cadena + count (le32) + campos `CLAVE=valor`.
comment_blocks = blocks.get(4)
if not comment_blocks:
    fail("no se encontró ningún bloque VORBIS_COMMENT (tipo 4)")

comment = comment_blocks[0]
if len(comment) < 8:
    fail("el bloque VORBIS_COMMENT es demasiado corto (truncado)")
vlen = struct.unpack("<I", comment[0:4])[0]
vendor = comment[4:4 + vlen]
if len(vendor) < vlen:
    fail("la cadena de vendor del comentario está truncada")
if not vendor:
    fail("el comentario no lleva vendor")

i = 4 + vlen
if i + 4 > len(comment):
    fail("el conteo de campos del comentario está truncado")
count = struct.unpack("<I", comment[i:i + 4])[0]
if count != 0:
    fail("el comentario lleva %d campos de usuario y debe llevar 0" % count)

print("vendor=%r" % vendor.decode("utf-8", "replace"))
print("tipos de bloque de metadatos=%s" % sorted(blocks))
PY

  # 4. `ffprobe` reconoce el archivo como audio FLAC y `ffmpeg` lo decodifica
  #    por completo (es reproducible).
  local codec
  codec="$(ffprobe -hide_banner -v error -select_streams a:0 \
    -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$file")" || {
    echo "ERROR: ffprobe no puede leer el fichero" >&2
    return 1
  }
  if [ "$codec" != "flac" ]; then
    echo "ERROR: ffprobe no reconoce el fichero como audio FLAC (códec '$codec')" >&2
    return 1
  fi
  if ! ffmpeg -hide_banner -loglevel error -i "$file" -f null - >/dev/null 2>&1; then
    echo "ERROR: ffmpeg no puede decodificar el fichero" >&2
    return 1
  fi
  return 0
}

# Modo `--check ARCHIVO`: solo verifica, sin generar ni tocar nada.
if [ "${1:-}" = "--check" ]; then
  target="${2:?uso: generate_sample_flac.sh [--check ARCHIVO]}"
  check_flac "$target"
  echo "OK: estructura válida: $target"
  exit 0
fi

# Modo generar: se escribe a un temporal, se verifica y solo entonces se
# sustituye el fixture versionado.
TMP="$(mktemp -t sampleXXXX.flac)"
trap 'rm -f "$TMP"' EXIT

# Audio FLAC mono de 0,6 s a 48 kHz, sin metadatos de usuario (`-map_metadata
# -1`). El vendor del codificador permanece en el comentario a propósito.
ffmpeg -hide_banner -loglevel error -y \
  -f lavfi -i "anullsrc=r=48000:cl=mono" -t 0.6 \
  -c:a flac \
  -map_metadata -1 -fflags +bitexact -flags:a +bitexact \
  "$TMP"

check_flac "$TMP"

# `mktemp` crea con permisos 0600, así que se normalizan a 0644 como el resto
# de fixtures versionados.
mv -f "$TMP" "$OUT"
chmod 644 "$OUT"
echo "OK: $OUT generado y verificado"