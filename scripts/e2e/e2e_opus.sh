#!/usr/bin/env bash
#
# e2e_opus.sh — verificación e2e local del formato Opus (fase GREEN: flujo completo).
#
# Uso: scripts/e2e/e2e_opus.sh [--bin RUTA] [--keep]
#   --bin RUTA  usa el binario indicado en lugar de `cargo build --locked`.
#   --keep      conserva el temporal de la ejecución (bajo /tmp/opencode/e2e/opus/).
#
# FLUJO VERIFICADO (GREEN, implementado):
#   El contenedor Ogg de Opus se verifica igual que el de Vorbis (mismo backend
#   de comentarios Vorbis en id3cli y misma exposición stream_tags en ffprobe):
#   show inicial (sin campos) → edit con los 18 tags + --url/--original-artist
#   (omitidos con aviso) → show campo a campo → ffprobe (stream_tags; track=7 +
#   TRACKTOTAL=12, disc=1 + DISCTOTAL=2) → remove de un campo (genre) → remove
#   --all (comentario vaciado PRESERVANDO el vendor) → ffprobe final de
#   reproducibilidad (~1.5 s) → variante de track/season SIN total (sobre audio
#   recién generado) → carátula JPG y PNG sobre audio recién generado (roundtrip
#   de bytes).
#
# DEGRADACIONES Opus: idénticas a OGG (--url y --original-artist omitidos con
#   aviso; nunca clave inventada; ausencia también en los bytes crudos del
#   comentario: MUST NOT aparecer WEBSITE= ni ORIGINALARTIST=).

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

    # ── Tabla de clave ffprobe del formato Opus (celdas medidas 2026-10-16) ──
    # Misma tabla que OGG: metadata en stream_tags, caja Vorbis preservada
    # (salvo ALBUMARTIST, que ffmpeg normaliza a «album_artist» — hallazgo 3.5),
    # TRACKNUMBER/DISCNUMBER normalizados por ffmpeg a track/disc, totales en
    # TRACKTOTAL/DISCTOTAL, sorts en TITLESORT/ARTISTSORT/ALBUMSORT.
    declare -A FF_TAGS=(
        [title]="TITLE"
        [artist]="ARTIST"
        [album]="ALBUM"
        [year]="YEAR"
        [date]="DATE"
        [genre]="GENRE"
        [track]="track"
        [season]="disc"
        [copyright]="COPYRIGHT"
        [composer]="COMPOSER"
        [subtitle]="SUBTITLE"
        [album_artist]="album_artist"
        [lyrics]="LYRICS"
        [compilation]="COMPILATION"
        [album_sort]="ALBUMSORT"
        [artist_sort]="ARTISTSORT"
        [title_sort]="TITLESORT"
        # [track_total]="TRACKTOTAL"  [disc_total]="DISCTOTAL"
        [url]=""                   # «—»: se omite con aviso, nunca clave inventada
        [original_artist]=""       # «—»: se omite con aviso, nunca clave inventada
        [cover]=""                 # «—»: se verifica por roundtrip de bytes
    )

    e2e_setup "opus"
    e2e_gen_audio
    e2e_gen_covers

    # 1) Estado inicial: show sobre el audio recién generado sin campos de usuario.
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_TITLE"
    check_show_absent "$E2E_V_ARTIST_JOINED"
    check_show_absent "$E2E_V_URL"

    # 2) edit con los 18 tags + --url/--original-artist (omitidos con aviso).
    e2e_run edit "$E2E_AUDIO" \
        --title "$E2E_V_TITLE" \
        --artist "$E2E_V_ARTIST1" --artist "$E2E_V_ARTIST2" \
        --album "$E2E_V_ALBUM" \
        --year "$E2E_V_YEAR" \
        --genre "$E2E_V_GENRE" \
        --track "$E2E_V_TRACK_TOTAL" \
        --season "$E2E_V_SEASON_TOTAL" \
        --date "$E2E_V_DATE" \
        --copyright "$E2E_V_COPYRIGHT" \
        --composer "$E2E_V_COMPOSER" \
        --subtitle "$E2E_V_SUBTITLE" \
        --original-artist "$E2E_V_ORIGINAL_ARTIST" \
        --album-artist "$E2E_V_ALBUM_ARTIST" \
        --lyrics "$E2E_V_LYRICS" \
        --url "$E2E_V_URL" \
        --compilation \
        --album-sort "$E2E_V_ALBUM_SORT" \
        --artist-sort "$E2E_V_ARTIST_SORT" \
        --title-sort "$E2E_V_TITLE_SORT"
    check_rc 0
    # Avisos de la degradación del contenedor Ogg (medidos en el spec):
    check_stderr_aviso "El campo '--url' no tiene clave equivalente en OGG; se omite."
    check_stderr_aviso "El campo '--original-artist' no tiene clave equivalente en OGG; se omite."

    # 3) show campo a campo: los 18 tags escritos (Año y Fecha separados),
    #    sin url/original_artist.
    e2e_run show "$E2E_AUDIO"
    check_show_contains "$E2E_V_TITLE"
    check_show_contains "$E2E_V_ARTIST_JOINED"
    check_show_contains "$E2E_V_ALBUM"
    check_show_contains "Año:"
    check_show_contains "$E2E_V_DATE"
    check_show_contains "$E2E_V_GENRE"
    check_show_contains "$E2E_V_TRACK_TOTAL"
    check_show_contains "$E2E_V_SEASON_TOTAL"
    check_show_contains "$E2E_V_COPYRIGHT"
    check_show_contains "$E2E_V_COMPOSER"
    check_show_contains "$E2E_V_SUBTITLE"
    check_show_contains "$E2E_V_ALBUM_ARTIST"
    check_show_contains "LetraE2E"
    check_show_contains "Segunda línea E2E"
    check_show_contains "Compilación: Sí"
    check_show_contains "$E2E_V_ALBUM_SORT"
    check_show_contains "$E2E_V_ARTIST_SORT"
    check_show_contains "$E2E_V_TITLE_SORT"
    check_show_absent "$E2E_V_URL"                  # no se confirman (omitidos)
    check_show_absent "$E2E_V_ORIGINAL_ARTIST"

    # 4) ffprobe (capa 2, stream_tags): celdas de FF_TAGS, totals incluidos.
    #    El YEAR se fija por el par clave+valor crudo («YEAR=2024») porque el
    #    valor «2024» es subcadena de «2024-05-01».
    local tag key
    for tag in "${!FF_TAGS[@]}"; do
        key="$(ff_key_for "$tag")"
        [[ -n "$key" ]] || continue
        case "$tag" in
            artist)      check_ffprobe_tag "$key" "$E2E_V_ARTIST_JOINED" ;;
            track)       check_ffprobe_tag "$key" "$E2E_V_TRACK" ;;
            season)      check_ffprobe_tag "$key" "$E2E_V_SEASON" ;;
            lyrics)      check_ffprobe_tag "$key" "${E2E_V_LYRICS%%$'\n'*}"
                         check_ffprobe_tag "$key" "${E2E_V_LYRICS#*$'\n'}" ;;
            compilation) check_ffprobe_tag "$key" "1" ;;
            *)           local v="E2E_V_${tag^^}"; check_ffprobe_tag "$key" "${!v}" ;;
        esac
    done
    check_ffprobe_raw_contains "YEAR=2024"
    check_ffprobe_tag TRACKTOTAL "12"
    check_ffprobe_tag DISCTOTAL "2"
    # Los campos omitidos MUST no aparecer ni en ffprobe ni en los bytes crudos:
    check_ffprobe_absent url
    check_ffprobe_absent original_artist
    grep_raw_absent "WEBSITE=" "$E2E_AUDIO"
    grep_raw_absent "ORIGINALARTIST=" "$E2E_AUDIO"

    # 5) remove de un campo concreto (genre): desaparece y el resto permanece.
    e2e_run remove "$E2E_AUDIO" genre
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_GENRE"
    check_show_contains "$E2E_V_TITLE"
    check_show_contains "$E2E_V_ARTIST_JOINED"
    check_ffprobe_absent GENRE
    check_ffprobe_tag TITLE "$E2E_V_TITLE"

    # 6) remove --all: el comentario queda VACÍO pero el VENDOR se preserva
    #    (obligatorio en el contenedor); el audio sigue siendo reproducible.
    local vendor
    vendor="$(grep -aoE 'Lavf[0-9.]+' "$E2E_AUDIO" 2>/dev/null | head -1 || true)"
    if [[ -z "$vendor" ]]; then
        check_fail "no se pudo capturar el vendor del comentario Vorbis de $E2E_AUDIO antes de remove --all"
    fi
    e2e_run remove --all "$E2E_AUDIO"
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_TITLE"
    check_show_absent "$E2E_V_ARTIST_JOINED"
    check_show_absent "$E2E_V_URL"
    check_ffprobe_no_user_tags
    if [[ -n "$vendor" ]]; then
        grep_raw_contains "$vendor" "$E2E_AUDIO"    # vendor preservado en crudo
    fi
    check_duration_approx 1.5

    # 7) Variante SIN total sobre audio RECIÉN GENERADO: --track 7 --season 1.
    #    El CLI conserva TRACKTOTAL/DISCTOTAL previos si solo se indica el
    #    número, así que la forma sin total se ejerce sobre un archivo sin
    #    totales previos. Discriminador: la AUSENCIA de «7/12» y «1/2» en show
    #    y de las claves TRACKTOTAL/DISCTOTAL en ffprobe.
    e2e_gen_audio track_only
    e2e_run edit "$E2E_AUDIO" --track "$E2E_V_TRACK" --season "$E2E_V_SEASON"
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_contains "$E2E_V_TRACK"
    check_show_absent "$E2E_V_TRACK_TOTAL"
    check_show_absent "$E2E_V_SEASON_TOTAL"
    check_ffprobe_tag track "$E2E_V_TRACK"
    check_ffprobe_tag disc "$E2E_V_SEASON"
    check_ffprobe_absent TRACKTOTAL
    check_ffprobe_absent DISCTOTAL

    # 8) Carátula JPG y PNG: cada una sobre audio RECIÉN GENERADO, roundtrip de
    #    bytes (`-map 0:v:0 -c copy -frames:v 1` + cmp).
    e2e_gen_audio cover_jpg
    e2e_run edit "$E2E_AUDIO" --cover "$E2E_COVER_JPG"
    check_rc 0
    check_cover_roundtrip "$E2E_COVER_JPG"

    e2e_gen_audio cover_png
    e2e_run edit "$E2E_AUDIO" --cover "$E2E_COVER_PNG"
    check_rc 0
    check_cover_roundtrip "$E2E_COVER_PNG"

    if e2e_summary "opus"; then
        exit 0
    fi
    exit 1
}

main "$@"