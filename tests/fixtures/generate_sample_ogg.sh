#!/usr/bin/env bash
#
# Regenera `tests/fixtures/sample.ogg`, el fixture Ogg Vorbis mínimo usado por
# los tests del backend `VorbisComments` (grupos 2 y 4 del cambio
# `ogg-support-and-track-totals`).
#
# El fixture debe ser un Ogg Vorbis real (no bytes fabricados) para que `lofty`
# lo parsee y `ffprobe` lo reconozca, con el vendor y la clave `ENCODER` del
# codificador y **sin** campos de usuario (`TITLE`, `ARTIST`…), de modo que
# `edit` ejercite la creación de comentarios desde cero.
#
# Medido: `ffmpeg` escribe siempre el vendor y una clave `ENCODER` (del tipo
# `encoder=Lavc libvorbis`) en el comentario, y `-fflags +bitexact` NO los
# elimina. Por eso el fichero NO es reproducible byte a byte entre versiones de
# `ffmpeg` y la verificación de este script es **estructural**, nunca por
# igualdad de bytes contra el fixture versionado. El vendor y el `ENCODER` son
# esperados y no hay que corregirlos; lo que el fixture NO debe llevar es ningún
# campo de usuario.
#
# Uso:
#   tests/fixtures/generate_sample_ogg.sh
#
# Requisitos: ffmpeg, ffprobe y python3.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$HERE/sample.ogg"
TMP="$(mktemp -t sampleXXXX.ogg)"
trap 'rm -f "$TMP"' EXIT

# Audio Vorbis mono de 0,6 s a 44,1 kHz y calidad 1 (~4 KB), sin metadatos de
# usuario (`-map_metadata -1`). El vendor y el `ENCODER` del codificador
# permanecen en el comentario a propósito.
ffmpeg -hide_banner -loglevel error -y \
  -f lavfi -i "sine=frequency=440:duration=0.6:sample_rate=44100" \
  -c:a libvorbis -q:a 1 -ac 1 \
  -map_metadata -1 -fflags +bitexact -flags:a +bitexact \
  "$TMP"

# Verificación estructural sobre el temporal: solo si pasa se sustituye el
# fixture versionado. Comprueba la cabecera de página, el primer paquete Vorbis
# y el comentario (vendor + `ENCODER`, sin campos de usuario).
python3 - "$TMP" <<'PY'
import struct
import sys

path = sys.argv[1]
data = open(path, "rb").read()


def fail(msg):
    raise SystemExit("ERROR: " + msg)


if len(data) < 28:
    fail("el fixture es demasiado corto para ser un Ogg")

# 1. Cabecera de la primera página: patrón `OggS` en el offset 0 y versión 0.
if data[0:4] != b"OggS":
    fail("el patrón OggS no está en el offset 0")
if data[4] != 0:
    fail("la versión de la página (byte 4) no es 0, es %d" % data[4])

# 2. Primer paquete en 27 + header[26] (calculado, no fijado a 28).
first = 27 + data[26]
if first != 28:
    print("AVISO: el primer paquete cae en el offset %d, no en 28" % first)
if first + 7 > len(data):
    fail("el primer paquete está truncado")

# 3. El primer paquete es la cabecera de identificación Vorbis (`\x01vorbis`).
if data[first:first + 7] != b"\x01vorbis":
    fail("el primer paquete no empieza por el identificador \\x01vorbis")

# Reensambla los paquetes recorriendo las páginas por sus lacing values para
# localizar el comentario, que va en páginas posteriores a la primera.
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

if len(packets) < 2 or packets[1][:7] != b"\x03vorbis":
    fail("no se encontró la cabecera de comentario \\x03vorbis")

# 4. El comentario lleva vendor (no vacío) y la clave `ENCODER` con valor.
comment = packets[1]
vlen = struct.unpack("<I", comment[7:11])[0]
vendor = comment[11:11 + vlen]
i = 11 + vlen
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

# 6. Sin campos de usuario en el comentario.
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

# 5. `ffprobe` reconoce el archivo como audio Vorbis y `ffmpeg` lo decodifica
# por completo (es reproducible).
codec="$(ffprobe -hide_banner -v error -select_streams a:0 \
  -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$TMP")"
if [ "$codec" != "vorbis" ]; then
  echo "ERROR: ffprobe no reconoce el fixture como audio Vorbis (códec '$codec')" >&2
  exit 1
fi
ffmpeg -hide_banner -loglevel error -i "$TMP" -f null - >/dev/null || {
  echo "ERROR: ffmpeg no puede decodificar el fixture" >&2
  exit 1
}

# Solo ahora se sustituye el fixture versionado. `mktemp` crea con permisos
# 0600, así que se normalizan a 0644 como el resto de fixtures versionados.
mv -f "$TMP" "$OUT"
chmod 644 "$OUT"
echo "OK: $OUT generado y verificado"
