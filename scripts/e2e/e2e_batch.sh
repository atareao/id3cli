#!/usr/bin/env bash
#
# e2e_batch.sh — verificación e2e local del modo batch de id3cli
# (fase GREEN, change batch-processing).
#
# Uso: scripts/e2e/e2e_batch.sh [--bin RUTA] [--keep]
#   --bin RUTA  usa el binario indicado en lugar de `cargo build --locked`.
#   --keep      conserva el temporal de la ejecución (bajo /tmp/opencode/e2e/batch/).
#
# FLUJO VERIFICADO (GREEN, implementado):
#   El modo batch despliega `show`/`edit`/`remove` sobre N>1 archivos con las
#   mismas opciones: el legible separa cada bloque con `=== <ruta> ===`, el JSON
#   emite NDJSON (una línea por archivo, orden de entrada) al final del lote, un
#   fallo por archivo NO aborta el resto (se informa al final con exit ≠ 0) y
#   `show` sigue exigiendo al menos un archivo. El flujo genera audio de DOS
#   extensiones distintas (mp3 y m4a) cambiando E2E_EXT antes de cada
#   e2e_gen_audio, y verifica, por orden:
#   1. Estado inicial: el MP3 recién generado no tiene campos (sin tags hay una
#      cabecera ID3v2 vacía de ffmpeg: `tags` es objeto todo-null; el
#      `tags:null` real se verifica tras remove --all en el punto 7).
#   2. `edit --title/--artist` batch sobre f1.mp3 y f2.m4a: exit 0.
#   3. Verificación por archivo: show legible y show --json de cada uno con los
#      valores batch escritos.
#   4. NDJSON: `show --json f1 f2` → exit 0, exactamente 2 líneas no vacías y
#      documento parseable con `jq -s` (length == 2).
#   5. Cabeceras legibles: `=== <ruta> ===` para cada archivo, con la ruta tal y
#      como se pasó al CLI (absoluta del temporal).
#   6. Exit agregado: `edit` batch con un archivo inexistente en medio → exit 1
#      con el motivo por stderr y los archivos que SÍ existían quedan editados.
#   7. `remove --all` batch: exit 0 y el MP3 vuelve a `tags: null`.
#   8. `show` sin archivos: exit 2 (clap exige `<FILES>...`).
#   Todas las aserciones alimentan el recuento PASS/FAIL de e2e_summary
#   (e2e_lib.sh) y ninguna aborta: el fallo se mide por el contador.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./e2e_lib.sh
source "$HERE/e2e_lib.sh"

main() {
    # ── Parseo: --bin RUTA, --keep ──
    local bin=""
    local keep=0
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --bin) bin="${2:?falta la ruta del binario para --bin}"; shift 2 ;;
            --keep) keep=1; shift ;;
            *) printf 'Error: argumento desconocido: %s\nUso: %s [--bin RUTA] [--keep]\n' "$1" "$0" >&2; exit 1 ;;
        esac
    done
    E2E_BIN="$bin"
    E2E_KEEP=$keep

    # El temporal usa la extensión «batch» como nombre de directorio
    # (/tmp/opencode/e2e/batch/); el audio se genera después con E2E_EXT
    # alternado (mp3 y m4a) porque e2e_gen_audio decide códec y sufijo por ella.
    e2e_setup "batch"

    # 1) Preparación: dos audios de extensiones DISTINTAS sobre el mismo
    #    temporal. e2e_gen_audio usa E2E_EXT para códec y sufijo, así que se
    #    alterna antes de cada generación. Estado inicial sin campos: el MP3
    #    recién generado por ffmpeg conserva una cabecera ID3v2 VACÍA («ID3» en
    #    el byte 0 pese a -map_metadata -1, medido), de modo que `.tags` es un
    #    OBJETO con todas las claves null — no el `tags:null` real, que solo
    #    existe sin bloque ID3 y se verifica tras remove --all en el punto 7—.
    #    El «sin campos» se comprueba por la ausencia del título.
    E2E_EXT=mp3
    e2e_gen_audio f1            # $E2E_DIR/f1.mp3
    E2E_EXT=m4a
    e2e_gen_audio f2            # $E2E_DIR/f2.m4a
    e2e_run show --json "$E2E_DIR/f1.mp3"
    check_json_null '.tags.title'

    # 2) Edit batch: una sola invocación con las mismas opciones aplica el
    #    título y el artista a ambos archivos con exit 0.
    e2e_run edit --title "Título Batch" --artist "Artista Batch" \
        "$E2E_DIR/f1.mp3" "$E2E_DIR/f2.m4a"
    check_rc 0

    # 3) Verificación por archivo: cada uno lleva los valores batch escritos,
    #    tanto en el modo legible como en el JSON por archivo.
    e2e_run show "$E2E_DIR/f1.mp3"
    check_show_contains 'Título Batch'
    check_show_contains 'Artista Batch'
    e2e_run show "$E2E_DIR/f2.m4a"
    check_show_contains 'Título Batch'
    check_show_contains 'Artista Batch'
    e2e_run show --json "$E2E_DIR/f1.mp3"
    check_json_field '.tags.title' 'Título Batch'
    e2e_run show --json "$E2E_DIR/f2.m4a"
    check_json_field '.tags.title' 'Título Batch'

    # 4) NDJSON: con N>1 el modo JSON emite una línea por archivo (orden de
    #    entrada) y el bloque termina con salto final, consumible con `jq -s`.
    #    Se guarda el stdout capturado y se valida con jq como parser
    #    independiente: 2 líneas no vacías, documento parseable como array y
    #    longitud 2.
    e2e_run show --json "$E2E_DIR/f1.mp3" "$E2E_DIR/f2.m4a"
    check_rc 0
    local ndjson="$E2E_DIR/ndjson.txt"
    cp "$E2E_OUT" "$ndjson"
    local n_lineas
    n_lineas="$(grep -c . "$ndjson")"
    if [[ "$n_lineas" -eq 2 ]]; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: NDJSON: %s líneas no vacías (una por archivo)\n' "$n_lineas"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''NDJSON conteo'\'': se esperaban 2 líneas no vacías y hay %s\n' "$n_lineas" >&2
    fi
    if jq -s -e . "$ndjson" >/dev/null 2>&1; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: NDJSON: el documento completo es parseable con jq -s\n'
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''NDJSON jq -s'\'': el documento NDJSON no es parseable con jq -s\n' >&2
    fi
    local n_obj
    n_obj="$(jq -s 'length' "$ndjson")"
    if [[ "$n_obj" -eq 2 ]]; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: NDJSON: jq -s length devuelve 2 objetos\n'
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''NDJSON length'\'': jq -s length devuelve %s y se esperaban 2\n' "$n_obj" >&2
    fi

    # 5) Cabeceras legibles: con N>1 el modo legible separa cada bloque con
    #    `=== <ruta> ===`, donde la ruta es la que se pasó al CLI (absoluta del
    #    temporal; medido).
    e2e_run show "$E2E_DIR/f1.mp3" "$E2E_DIR/f2.m4a"
    check_show_contains "=== $E2E_DIR/f1.mp3 ==="
    check_show_contains "=== $E2E_DIR/f2.m4a ==="

    # 6) Exit agregado: un archivo inexistente en medio del lote NO aborta el
    #    resto: f1 y f2 se editan con «Mixto» y el motivo se informa al final
    #    por stderr con exit 1. El motivo real es «El archivo '...' no existe»
    #    (medido): la subcadena estable es 'no existe' — el token
    #    'inexistente' solo aparece por casualidad en el nombre de la ruta
    #    falsa, no como parte del mensaje.
    e2e_run edit --title "Mixto" "$E2E_DIR/f1.mp3" /ruta/inexistente.mp3 "$E2E_DIR/f2.m4a"
    check_rc 1
    check_stderr_aviso 'no existe'
    e2e_run show "$E2E_DIR/f2.m4a"
    check_show_contains 'Mixto'          # el lote continuó tras el fallo

    # 7) Remove --all batch: exit 0 y el MP3 vuelve a un estado SIN bloque
    #    ID3v2: aquí sí el caso real de `tags: null` (medido).
    e2e_run remove --all "$E2E_DIR/f1.mp3" "$E2E_DIR/f2.m4a"
    check_rc 0
    e2e_run show --json "$E2E_DIR/f1.mp3"
    check_json_null '.tags'

    # 8) Show sin archivos: el batch sigue exigiendo al menos un archivo
    #    (`#[arg(required = true)]`), así que clap falla con exit 2.
    e2e_run show
    check_rc 2

    if e2e_summary "batch"; then
        exit 0
    fi
    exit 1
}

main "$@"