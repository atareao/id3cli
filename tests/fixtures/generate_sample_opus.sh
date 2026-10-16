#!/usr/bin/env bash
#
# Regenera `tests/fixtures/sample.opus`, el fixture Ogg Opus mínimo usado por
# los tests del change `opus-support` (soporte de Ogg Opus sobre el backend de
# comentarios de Vorbis).
#
# El fixture debe ser un Ogg Opus real (no bytes fabricados) para que `lofty`
# lo parsee como `OpusFile` y `ffprobe` lo reconozca, con el vendor y la clave
# `ENCODER` del codificador y **sin** campos de usuario (`TITLE`, `ARTIST`…),
# de modo que `edit` ejercite la creación de comentarios desde cero.
#
# Medido: `ffmpeg` escribe siempre el vendor (`ffmpeg`) y una clave `ENCODER`
# del tipo `encoder=Lavc libopus` en el comentario, y `-fflags +bitexact` NO
# los elimina. Por eso el fichero NO es reproducible byte a byte entre
# versiones de `ffmpeg` y la verificación de este script es **estructural**,
# nunca por igualdad de bytes contra el fixture versionado. El vendor y el
# `ENCODER` son esperados y no hay que corregirlos; lo que el fixture NO debe
# llevar es ningún campo de usuario.
#
# Uso:
#   tests/fixtures/generate_sample_opus.sh                # regenera y verifica
#   tests/fixtures/generate_sample_opus.sh --check FILE   # solo verifica FILE
#
# Requisitos: ffmpeg, ffprobe y python3.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$HERE/sample.opus"

# Verificación estructural de un Ogg Opus: cabecera de página, primer paquete
# `OpusHead`, segundo `OpusTags` con su comentario (vendor + `ENCODER`, sin
# campos de usuario), `ffprobe` como audio Opus y decodificación completa. Falla
# con su mensaje y código 1 ante cualquier caso negativo.
check_opus() {
  local file="$1"

  python3 - "$file" <<'PY'
import struct
import sys

path = sys.argv[1]
data = open(path, "rb").read()


def fail(msg):
    raise SystemExit("ERROR: " + msg)


if len(data) < 28:
    fail("el fichero es demasiado corto para ser un Ogg")

# 1. Cabecera de la primera página: patrón `OggS` en el offset 0 y versión 0.
if data[0:4] != b"OggS":
    fail("el patrón OggS no está en el offset 0")
if data[4] != 0:
    fail("la versión de la página (byte 4) no es 0, es %d" % data[4])

# 2. Primer paquete en 27 + header[26] (calculado, no fijado a 28).
first = 27 + data[26]
if first != 28:
    print("AVISO: el primer paquete cae en el offset %d, no en 28" % first)
if first + 8 > len(data):
    fail("el primer paquete está truncado")

# 3. El primer paquete es la cabecera de identificación Opus (`OpusHead`).
if data[first:first + 8] != b"OpusHead":
    fail("el primer paquete no empieza por el identificador OpusHead")

# Reensambla los paquetes recorriendo las páginas por sus lacing values para
# localizar el comentario, que va en la segunda página (BOS completo).
off = 0
packets = []
partial = b""
while off + 27 <= len(data):
    if data[off:off + 4] != b"OggS":
        fail("se esperaba una página OggS en el offset %d" % off)
    nsegs = data[off + 26]
    body = off + 27 + nsegs
    for seg in data[off + 27:body]:
        partial += data[body:body + seg]
        body += seg
        if seg < 255:
            packets.append(partial)
            partial = b""
    off = body

if len(packets) < 2 or packets[1][:8] != b"OpusTags":
    fail("no se encontró la cabecera de comentario OpusTags")

# 4. El comentario lleva vendor (no vacío) y la clave `ENCODER` con valor.
# Layout de OpusTags: tras el mágico de 8 bytes, vendor (le32) + count (le32) +
# campos `CLAVE=valor`, igual que los comentarios de Vorbis.
comment = packets[1]
vlen = struct.unpack("<I", comment[8:12])[0]
vendor = comment[12:12 + vlen]
i = 12 + vlen
count = struct.unpack("<I", comment[i:i + 4])[0]
i += 4
fields = []
for _ in range(count):
    flen = struct.unpack("<I", comment[i:i + 4])[0]
    i += 4
    fields.append(comment[i:i + flen].decode("utf-8", "replace"))
    i += flen

if not vendor:
    fail("el comentario no lleva vendor")

keys = {}
for field in fields:
    key, _, value = field.partition("=")
    keys[key.upper()] = value
if not keys.get("ENCODER"):
    fail("el comentario no lleva la clave ENCODER con valor")

# 5. Sin campos de usuario en el comentario.
USER_FIELDS = {
    "TITLE", "ARTIST", "ALBUM", "ALBUMARTIST", "TRACKNUMBER", "TRACKTOTAL",
    "DISCNUMBER", "DISCTOTAL", "DATE", "YEAR", "GENRE", "COMPOSER",
    "PERFORMER", "COMMENT", "DESCRIPTION", "LYRICS", "LYRICIST", "CONDUCTOR",
    "PUBLISHER", "COPYRIGHT", "SUBTITLE", "URL",
}
present = sorted(key for key in keys if key in USER_FIELDS)
if present:
    fail("el comentario contiene campos de usuario: " + ", ".join(present))

print("vendor=%r" % vendor.decode("utf-8", "replace"))
print("campos del comentario=%s" % fields)
PY

  # 6. `ffprobe` reconoce el archivo como audio Opus y `ffmpeg` lo decodifica
  # por completo (es reproducible).
  local codec
  codec="$(ffprobe -hide_banner -v error -select_streams a:0 \
    -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$file")" || {
    echo "ERROR: ffprobe no puede leer el fichero" >&2
    return 1
  }
  if [ "$codec" != "opus" ]; then
    echo "ERROR: ffprobe no reconoce el fichero como audio Opus (códec '$codec')" >&2
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
  target="${2:?uso: generate_sample_opus.sh [--check ARCHIVO]}"
  check_opus "$target"
  echo "OK: estructura válida: $target"
  exit 0
fi

# Modo generar: se escribe a un temporal, se verifica y solo entonces se
# sustituye el fixture versionado.
TMP="$(mktemp -t sampleXXXX.opus)"
trap 'rm -f "$TMP"' EXIT

# Audio Opus mono de 0,6 s a 48 kHz (la frecuencia nativa de Opus), sin
# metadatos de usuario (`-map_metadata -1`). El vendor y el `ENCODER` del
# codificador permanecen en el comentario a propósito.
ffmpeg -hide_banner -loglevel error -y \
  -f lavfi -i "anullsrc=r=48000:cl=mono" -t 0.6 \
  -c:a libopus \
  -map_metadata -1 -fflags +bitexact -flags:a +bitexact \
  "$TMP"

check_opus "$TMP"

# `mktemp` crea con permisos 0600, así que se normalizan a 0644 como el resto
# de fixtures versionados.
mv -f "$TMP" "$OUT"
chmod 644 "$OUT"
echo "OK: $OUT generado y verificado"