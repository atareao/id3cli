#!/usr/bin/env bash
#
# Regenera `tests/fixtures/sample.m4a`, el fixture MP4 mínimo usado por los
# tests del backend `ilst` (grupo 4 del cambio `m4a-support`).
#
# El fixture debe ser un MP4 real, con el átomo `ftyp` en el offset 4 y **sin**
# bloque `ilst`, para poder probar la creación del árbol `moov.udta.meta.ilst`
# desde cero.
#
# `ffmpeg` siempre escribe un `udta.meta.ilst` (aunque sea vacío) al usar el
# muxer `mov`, así que el script genera sin `faststart` (de modo que `moov` sea
# el último átomo del archivo, con `udta` como su último hijo) y después elimina
# el átomo `udta` truncando el archivo y ajustando el tamaño de `moov`. Al estar
# `moov` al final, quitar `udta` no desplaza `mdat` ni invalida los offsets de
# `stco`.
#
# Uso:
#   tests/fixtures/generate_sample_m4a.sh
#
# Requisitos: ffmpeg, ffprobe y python3.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$HERE/sample.m4a"
TMP="$(mktemp -t sampleXXXX.m4a)"
trap 'rm -f "$TMP"' EXIT

# Audio AAC mono de 0,6 s a 32 kbps (~3 KB), sin metadatos (`bitexact` evita el
# tag `©too` del encoder y `-map_metadata -1` descarta cualquier metadato).
ffmpeg -hide_banner -loglevel error -y \
  -f lavfi -i "sine=frequency=440:duration=0.6:sample_rate=44100" \
  -c:a aac -b:a 32k -ac 1 \
  -map_metadata -1 -fflags +bitexact -flags:a +bitexact \
  "$TMP"

# Elimina el átomo `udta` (que contiene `meta.ilst`) de `moov`.
python3 - "$TMP" "$OUT" <<'PY'
import struct
import sys

src, dst = sys.argv[1], sys.argv[2]
data = bytearray(open(src, "rb").read())


def atom_children(buf, start, end):
    off = start
    while off + 8 <= end:
        size = struct.unpack(">I", buf[off:off + 4])[0]
        ident = bytes(buf[off + 4:off + 8])
        if size == 1:
            size = struct.unpack(">Q", buf[off + 8:off + 16])[0]
        elif size == 0:
            size = end - off
        yield off, size, ident
        off += size


# Localiza `moov` (átomo de nivel superior).
moov = next((o, s, i) for o, s, i in atom_children(data, 0, len(data)) if i == b"moov")
moov_off, moov_size, _ = moov
moov_end = moov_off + moov_size

# Localiza `udta`, que debe ser el último hijo de `moov`.
udta = None
for off, size, ident in atom_children(data, moov_off + 8, moov_end):
    if ident == b"udta":
        udta = (off, size)
if udta is None:
    raise SystemExit("no se encontró el átomo udta en el fixture generado")
udta_off, udta_size = udta
if udta_off + udta_size != moov_end:
    raise SystemExit("udta no es el último hijo de moov; el fixture no es determinista")

# Encoge `moov` y trunca el archivo para eliminar `udta`.
new_moov_size = moov_size - udta_size
data[moov_off:moov_off + 4] = struct.pack(">I", new_moov_size)
open(dst, "wb").write(bytes(data[:udta_off]))
PY

# Verificación: ftyp en el offset 4, sin ilst y reproducible por ffprobe.
head="$(head -c 8 "$OUT" | tail -c 4)"
if [ "$head" != "ftyp" ]; then
  echo "ERROR: ftyp no está en el offset 4" >&2
  exit 1
fi
if grep -q "ilst" "$OUT"; then
  echo "ERROR: el fixture todavía contiene un bloque ilst" >&2
  exit 1
fi
ffprobe -hide_banner -v error "$OUT" >/dev/null || {
  echo "ERROR: ffprobe no reconoce el fixture" >&2
  exit 1
}
echo "OK: $OUT generado y verificado"
