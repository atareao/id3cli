#!/usr/bin/env bash
#
# e2e_mp3.sh — verificación e2e local del formato MP3 (fase GREEN: flujo completo).
#
# Uso: scripts/e2e/e2e_mp3.sh [--bin RUTA] [--keep]
#   --bin RUTA  usa el binario indicado en lugar de `cargo build --locked`.
#   --keep      conserva el temporal de la ejecución (bajo /tmp/opencode/e2e/mp3/).
#
# FLUJO VERIFICADO (GREEN, implementado):
#   show inicial (sin campos de usuario) → edit con los 20 tags en una sola
#   invocación → show campo a campo (los 20, incluidos url y original_artist,
#   que en MP3 NO se omiten) → ffprobe (tabla FF_TAGS, format_tags) → remove de
#   un campo (genre) con el resto intacto → remove --all (el bloque ID3v2
#   desaparece físicamente del archivo) → ffprobe final de reproducibilidad
#   (duración ~1.5 s) → variante de track/season SIN total (sobre audio recién
#   generado) → carátula JPG y PNG, cada una sobre audio recién generado, con
#   roundtrip de bytes → segunda edición de --cover con imagen DISTINTA:
#   sustitución en vez de acumulación (1 carátula, roundtrip de la última,
#   anti-roundtrip de la primera; bug 1.2: antes «Carátulas: 2 imagen(es)»).

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

    # ── Tabla de clave ffprobe del formato MP3 (celdas medidas 2026-10-16) ──
    # MP3 expone la metadata en format_tags con claves normalizadas a minúsculas.
    # Medido sobre archivo real escrito por id3cli (tarea 3.5): title, artist,
    # album, genre, track («7/12»), disc («1/2»), copyright, composer, TIT3
    # (subtitle), TOPE (original_artist), album_artist, lyrics-spa (USLT con
    # idioma, valor multilínea), compilation («1»), album-sort/artist-sort/
    # title-sort y date. year pliega en date (TYER/TDRC comparten clave; con
    # ambos presentes ffprobe expone el AÑO, ver hallazgo en 3.5). url (TXXX)
    # NO se expone → celda «—», se verifica solo por show.
    declare -A FF_TAGS=(
        [title]="title"
        [artist]="artist"
        [album]="album"
        [year]="date"              # plegado: TYER/TDRC comparten clave
        [genre]="genre"
        [track]="track"
        [season]="disc"
        [date]="date"
        [copyright]="copyright"
        [composer]="composer"
        [subtitle]="TIT3"
        [original_artist]="TOPE"
        [album_artist]="album_artist"
        [lyrics]="lyrics-spa"
        [compilation]="compilation"
        [album_sort]="album-sort"
        [artist_sort]="artist-sort"
        [title_sort]="title-sort"
        [url]=""                   # «—»: TXXX no expuesto; solo capa show
        [cover]=""                 # «—»: se verifica por roundtrip de bytes
    )

    e2e_setup "mp3"
    e2e_gen_audio
    e2e_gen_covers

    # 1) Estado inicial: show sobre el audio recién generado sin campos de usuario.
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_TITLE"
    check_show_absent "$E2E_V_ARTIST_JOINED"
    check_show_absent "$E2E_V_URL"

    # 2) edit con los 20 tags en una invocación (MP3 sin degradaciones).
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

    # 3) show campo a campo: los 20 valores. El año se ancla por el rótulo
    #    («Año:») porque «2024» es subcadena de la fecha «2024-05-01» (valores
    #    fijados del spec); el valor del año lo fija ffprobe en el paso 4.
    e2e_run show "$E2E_AUDIO"
    check_show_contains "$E2E_V_TITLE"
    check_show_contains "$E2E_V_ARTIST_JOINED"      # multi-artista unido con "; "
    check_show_contains "$E2E_V_ALBUM"
    check_show_contains "Año:"
    check_show_contains "$E2E_V_GENRE"
    check_show_contains "$E2E_V_TRACK_TOTAL"        # Pista: 7/12
    check_show_contains "$E2E_V_SEASON_TOTAL"       # Temporada: 1/2
    check_show_contains "$E2E_V_DATE"               # Fecha: 2024-05-01
    check_show_contains "$E2E_V_COPYRIGHT"
    check_show_contains "$E2E_V_COMPOSER"
    check_show_contains "$E2E_V_SUBTITLE"
    check_show_contains "$E2E_V_ORIGINAL_ARTIST"
    check_show_contains "$E2E_V_ALBUM_ARTIST"
    check_show_contains "LetraE2E"                  # letra: 2 líneas
    check_show_contains "Segunda línea E2E"
    check_show_contains "$E2E_V_URL"                # MP3 sí escribe la URL (WOAR)
    check_show_contains "Compilación: Sí"
    check_show_contains "$E2E_V_ALBUM_SORT"
    check_show_contains "$E2E_V_ARTIST_SORT"
    check_show_contains "$E2E_V_TITLE_SORT"

    # 4) ffprobe (capa 2, format_tags): las celdas de FF_TAGS con clave medida.
    #    year/date comparten la clave «date» y se verifican explícitamente
    #    abajo; lyrics lleva dos líneas y se comprueba línea a línea.
    local tag key
    for tag in "${!FF_TAGS[@]}"; do
        key="$(ff_key_for "$tag")"
        [[ -n "$key" ]] || continue
        case "$tag" in
            year|date) continue ;;   # clave compartida «date», verificada abajo
            artist)      check_ffprobe_tag "$key" "$E2E_V_ARTIST_JOINED" ;;
            track)       check_ffprobe_tag "$key" "$E2E_V_TRACK_TOTAL" ;;
            season)      check_ffprobe_tag "$key" "$E2E_V_SEASON_TOTAL" ;;
            lyrics)      check_ffprobe_tag "$key" "${E2E_V_LYRICS%%$'\n'*}"
                         check_ffprobe_tag "$key" "${E2E_V_LYRICS#*$'\n'}" ;;
            compilation) check_ffprobe_tag "$key" "1" ;;
            *)           local v="E2E_V_${tag^^}"; check_ffprobe_tag "$key" "${!v}" ;;
        esac
    done
    # Hallazgo medido (3.5): con TYER (--year) y TDRC (--date) presentes,
    # ffprobe expone la clave compartida «date» con el valor del AÑO (TYER gana
    # al último TDRC). El par se verifica por show (Año + Fecha); ffprobe fija
    # el valor ganador del plegado.
    check_ffprobe_tag date "$E2E_V_YEAR"
    # url (TXXX, WOAR) no se expone en format_tags: celda «—» (solo capa show).
    check_ffprobe_absent url

    # 5) remove de un campo concreto (genre): desaparece y el resto permanece.
    e2e_run remove "$E2E_AUDIO" genre
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_GENRE"
    check_show_contains "$E2E_V_TITLE"
    check_show_contains "$E2E_V_ARTIST_JOINED"
    check_ffprobe_absent genre
    check_ffprobe_tag title "$E2E_V_TITLE"

    # 6) remove --all: el bloque ID3v2 desaparece FÍSICAMENTE (el archivo ya no
    #    empieza por «ID3», medido) y el audio sigue siendo reproducible.
    e2e_run remove --all "$E2E_AUDIO"
    check_rc 0
    check_no_id3v2_header
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_TITLE"
    check_show_absent "$E2E_V_ARTIST_JOINED"
    check_show_absent "$E2E_V_URL"
    check_duration_approx 1.5

    # 7) Variante SIN total sobre audio RECIÉN GENERADO: --track 7 --season 1.
    #    El CLI conserva el total previo del frame TRCK si solo se indica el
    #    número (documentado en apply_metadata), así que la forma sin total se
    #    ejerce sobre un archivo sin totales previos. Discriminador: la
    #    AUSENCIA de «7/12» y «1/2» (7 ⊂ 7/12, valores fijados del spec).
    e2e_gen_audio track_only
    e2e_run edit "$E2E_AUDIO" --track "$E2E_V_TRACK" --season "$E2E_V_SEASON"
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_contains "$E2E_V_TRACK"
    check_show_absent "$E2E_V_TRACK_TOTAL"
    check_show_absent "$E2E_V_SEASON_TOTAL"
    check_ffprobe_tag track "$E2E_V_TRACK"
    check_ffprobe_tag disc "$E2E_V_SEASON"

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

    # 9) Sustitución de carátula (bug 1.2): una segunda edición de --cover con
    #    una imagen DISTINTA sustituye en vez de acumular. Antes del fix, este
    #    bloque habría sido RED: show reportaría «Carátulas: 2 imagen(es)», el
    #    roundtrip de la PRIMERA imagen (JPG) seguiría pasando y
    #    check_cover_not_roundtrip del JPG fallaría con exit 1.
    e2e_gen_audio cover_sust
    e2e_run edit "$E2E_AUDIO" --cover "$E2E_COVER_JPG"
    check_rc 0
    e2e_run edit "$E2E_AUDIO" --cover "$E2E_COVER_PNG"
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_contains "Carátulas: 1 imagen(es)"
    check_cover_roundtrip "$E2E_COVER_PNG"
    check_cover_not_roundtrip "$E2E_COVER_JPG"

    if e2e_summary "mp3"; then
        exit 0
    fi
    exit 1
}

main "$@"