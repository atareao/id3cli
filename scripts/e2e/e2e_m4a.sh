#!/usr/bin/env bash
#
# e2e_m4a.sh — verificación e2e local del formato MP4/M4A (fase GREEN: flujo completo).
#
# Uso: scripts/e2e/e2e_m4a.sh [--bin RUTA] [--keep]
#   --bin RUTA  usa el binario indicado en lugar de `cargo build --locked`.
#   --keep      conserva el temporal de la ejecución (bajo /tmp/opencode/e2e/m4a/).
#
# FLUJO VERIFICADO (GREEN, implementado):
#   show inicial (sin campos) → edit con los 18 tags + --url/--original-artist
#   (que se OMITEN con aviso por stderr) + --year/--date (colisión: gana date con
#   aviso) → show campo a campo (18 valores, sin url/original_artist) → ffprobe
#   (tabla FF_TAGS, format_tags, con ausencia de url/original_artist) → remove
#   de un campo (genre) → remove --all (ilst vacío) → ffprobe final de
#   reproducibilidad (~1.5 s) → variante de track/season SIN total (sobre audio
#   recién generado) → carátula JPG y PNG sobre audio recién generado (roundtrip
#   de bytes) → secuencia edit → remove --all → edit sobre audio recién generado
#   con show + ffprobe format_tags de title/artist (bug 1.1: sin el fix, el meta
#   reescrito queda sin hijo hdlr y ffprobe no expone las etiquetas) → segunda
#   edición de --cover con imagen DISTINTA: sustitución en vez de acumulación
#   (1 carátula, roundtrip de la última, anti-roundtrip de la primera; bug 1.2).
#
# DEGRADACIONES MP4 (medidas):
#   - --url y --original-artist se omiten con aviso por stderr y NUNCA se
#     escriben como átomo freeform (ausencia comprobada por ffprobe).
#   - --year y --date comparten el átomo ©day: si se indican ambos gana --date
#     con aviso («--year 2023 queda absorbido por --date ...»).

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

    # ── Tabla de clave ffprobe del formato MP4/M4A (celdas medidas 2026-10-16) ──
    # MP4 expone la metadata en format_tags con claves normalizadas a minúsculas.
    # Medido sobre archivo real escrito por id3cli (tarea 3.5): title, artist,
    # album, album_artist, genre, copyright, composer, SUBTITLE (el freeform
    # «----:com.apple.iTunes:SUBTITLE» se expone con esa caja), lyrics (©lyr,
    # valor multilínea), compilation («1»), sort_name/sort_artist/sort_album
    # (sonm/soar/soal, NO «*-sort»), date (©day; --year pliega en la misma clave
    # y gana la fecha) y track/disc («7/12», «1/2»). url/original_artist → «—»
    # (omitidos con aviso; MUST NO aparecer en ffprobe).
    declare -A FF_TAGS=(
        [title]="title"
        [artist]="artist"
        [album]="album"
        [year]="date"              # plegado: comparte ©day con date
        [genre]="genre"
        [track]="track"
        [season]="disc"
        [date]="date"
        [copyright]="copyright"
        [composer]="composer"
        [subtitle]="SUBTITLE"
        [album_artist]="album_artist"
        [lyrics]="lyrics"
        [compilation]="compilation"
        [album_sort]="sort_album"
        [artist_sort]="sort_artist"
        [title_sort]="sort_name"
        [url]=""                   # «—»: se omite con aviso, nunca freeform
        [original_artist]=""       # «—»: se omite con aviso, nunca freeform
        [cover]=""                 # «—»: se verifica por roundtrip de bytes
    )

    e2e_setup "m4a"
    e2e_gen_audio
    e2e_gen_covers

    # 1) Estado inicial: show sobre el audio recién generado sin campos de usuario.
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_TITLE"
    check_show_absent "$E2E_V_ARTIST_JOINED"
    check_show_absent "$E2E_V_URL"

    # 1b) Etapa JSON (audio fresco): el M4A recién generado por ffmpeg conserva
    #     metadatos ilst VACÍOS (medido: `tags` es un OBJETO con todas las
    #     claves null, no el `tags:null` tras remove --all del paso 6b). En
    #     modo --json no hay aviso ⚠️: stderr vacío con exit 0.
    check_json_valid
    check_json_null '.tags.title'
    check_json_field '.tags.pictures | length' '0'
    check_rc 0
    if [[ -s "$E2E_ERR" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''stderr vacío'\'': el modo --json no debe escribir en stderr sin tags: «%s»\n' "$(tr -d '\n' <"$E2E_ERR")" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: stderr vacío en el modo --json sin tags\n'
    fi

    # 2) edit con los 18 tags + --url/--original-artist (omitidos con aviso) +
    #    --year 2023 --date 2024-05-01 (colisión de ©day: gana la fecha).
    e2e_run edit "$E2E_AUDIO" \
        --title "$E2E_V_TITLE" \
        --artist "$E2E_V_ARTIST1" --artist "$E2E_V_ARTIST2" \
        --album "$E2E_V_ALBUM" \
        --year 2023 \
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
    # Avisos de las tres degradaciones (medidos en el spec):
    check_stderr_aviso "El campo '--url' no tiene átomo equivalente en MP4; se omite."
    check_stderr_aviso "El campo '--original-artist' no tiene átomo equivalente en MP4; se omite."
    check_stderr_aviso "--year 2023 queda absorbido por --date"

    # 3) show campo a campo: los 18 tags escritos, con la fecha ganadora de la
    #    colisión (©day = 2024-05-01) y SIN confirmaciones de url/original_artist.
    e2e_run show "$E2E_AUDIO"
    check_show_contains "$E2E_V_TITLE"
    check_show_contains "$E2E_V_ARTIST_JOINED"
    check_show_contains "$E2E_V_ALBUM"
    check_show_contains "$E2E_V_DATE"               # Fecha: 2024-05-01 (gana date)
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

    # 3b) Etapa JSON (parser independiente jq): show --json sobre el M4A ya
    #     editado con los 18 tags. Igual que en MP3: artistas como ARRAY (se
    #     verifica el primer elemento), track como objeto {number, total} y la
    #     letra por SUBcadena de la primera línea (el JSON escapa los saltos
    #     como `\n` literal; ver check_json_field en e2e_lib.sh).
    check_json_valid
    check_json_format 'mp4'
    check_json_field '.tags.title' "$E2E_V_TITLE"
    check_json_field '.tags.artists[0]' "$E2E_V_ARTIST1"     # multi-artista
    check_json_field '.tags.album' "$E2E_V_ALBUM"
    check_json_field '.tags.track.number' "$E2E_V_TRACK"     # = '7'
    check_json_field '.tags.compilation' 'true'               # ©cmt/ilst → bool
    check_json_field '.tags.lyrics' 'LetraE2E'                # 1.ª línea (subcadena)

    # 4) ffprobe (capa 2, format_tags): celdas de FF_TAGS con clave medida.
    #    year/date comparten ©day → verificadas explícitamente (gana la fecha).
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
    # ©day gana la fecha (medido): el valor de la clave compartida es 2024-05-01.
    check_ffprobe_tag date "$E2E_V_DATE"
    # Los campos omitidos MUST no aparecer en el lector independiente:
    check_ffprobe_absent url
    check_ffprobe_absent original_artist

    # 5) remove de un campo concreto (genre): desaparece y el resto permanece.
    e2e_run remove "$E2E_AUDIO" genre
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_GENRE"
    check_show_contains "$E2E_V_TITLE"
    check_show_contains "$E2E_V_ARTIST_JOINED"
    check_ffprobe_absent genre
    check_ffprobe_tag title "$E2E_V_TITLE"

    # 6) remove --all: ilst queda vacío; el audio sigue siendo reproducible.
    e2e_run remove --all "$E2E_AUDIO"
    check_rc 0
    # 6b) Etapa JSON sin tags: tras remove --all el M4A queda sin ilst → el
    #     caso real de `tags:null` (spec: MP4 sin ilst → tags null con exit 0 y
    #     stderr VACÍO; el aviso ⚠️ se suprime en modo --json; medido).
    check_json_valid
    check_json_null '.tags'
    check_rc 0
    if [[ -s "$E2E_ERR" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''stderr vacío'\'': el modo --json no debe escribir en stderr sin tags: «%s»\n' "$(tr -d '\n' <"$E2E_ERR")" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: stderr vacío en el modo --json sin tags (tags:null)\n'
    fi
    e2e_run show "$E2E_AUDIO"
    check_show_absent "$E2E_V_TITLE"
    check_show_absent "$E2E_V_ARTIST_JOINED"
    check_show_absent "$E2E_V_URL"
    check_ffprobe_absent title
    check_ffprobe_absent date
    check_duration_approx 1.5

    # 7) Variante SIN total sobre audio RECIÉN GENERADO: --track 7 --season 1.
    #    El CLI conserva el total previo del átomo trkn/disk si solo se indica
    #    el número, así que la forma sin total se ejerce sobre un archivo sin
    #    totales previos. Discriminador: la AUSENCIA de «7/12» y «1/2».
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

    # 9) Secuencia edit → remove --all → edit (bug 1.1, hdlr en MP4): sobre un
    #    audio RECIÉN GENERADO se edita, se eliminan TODOS los metadatos y se
    #    vuelve a editar con los mismos campos. Es la secuencia exacta que
    #    destapó el bug: lofty 0.25.4 reconstruye el átomo meta SIN su hijo
    #    hdlr cuando reescribe un archivo cuyo ilst fue eliminado con remove
    #    --all, y el demuxer de ffmpeg exige ese hdlr para parsear ilst. Sin el
    #    fix, ffprobe format_tags NO expondría title/artist (solo los brands
    #    major_brand/minor_version/compatible_brands) y check_ffprobe_tag
    #    fallaría con exit 1 (PASS/FAIL > 0 → script RED).
    e2e_gen_audio reedit
    e2e_run edit "$E2E_AUDIO" \
        --title "$E2E_V_TITLE" \
        --artist "$E2E_V_ARTIST1" --artist "$E2E_V_ARTIST2" \
        --album "$E2E_V_ALBUM"
    check_rc 0
    e2e_run remove --all "$E2E_AUDIO"
    check_rc 0
    e2e_run edit "$E2E_AUDIO" \
        --title "$E2E_V_TITLE" \
        --artist "$E2E_V_ARTIST1" --artist "$E2E_V_ARTIST2" \
        --album "$E2E_V_ALBUM"
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_contains "$E2E_V_TITLE"
    check_show_contains "$E2E_V_ARTIST_JOINED"
    # ffprobe format_tags (claves minúsculas en MP4): el metalista title y
    # artist con el artista multi unido por «; » ($E2E_V_ARTIST_JOINED).
    check_ffprobe_tag title "$E2E_V_TITLE"
    check_ffprobe_tag artist "$E2E_V_ARTIST_JOINED"

    # 10) Sustitución de carátula (bug 1.2): una segunda edición de --cover con
    #     una imagen DISTINTA sustituye en vez de acumular. Antes del fix, este
    #     bloque habría sido RED: show reportaría «Carátulas: 2 imagen(es)», el
    #     roundtrip de la PRIMERA imagen (JPG) seguiría pasando y
    #     check_cover_not_roundtrip del JPG fallaría con exit 1.
    e2e_gen_audio cover_sust
    e2e_run edit "$E2E_AUDIO" --cover "$E2E_COVER_JPG"
    check_rc 0
    e2e_run edit "$E2E_AUDIO" --cover "$E2E_COVER_PNG"
    check_rc 0
    e2e_run show "$E2E_AUDIO"
    check_show_contains "Carátulas: 1 imagen(es)"
    check_cover_roundtrip "$E2E_COVER_PNG"
    check_cover_not_roundtrip "$E2E_COVER_JPG"

    # 10b) Etapa JSON de la carátula: fixture con 1 imagen (la PNG final). El
    #      tipo del átomo covr es «Other» — NO «CoverFront» como MP3/OGG/FLAC:
    #      lofty mapea el covr a PictureType::Other (medido 2026-10-16). El
    #      count se verifica con `jq | length`.
    check_json_valid
    check_json_format 'mp4'
    check_json_field '.tags.pictures | length' '1'
    check_json_field '.tags.pictures[0].type' 'Other'

    if e2e_summary "m4a"; then
        exit 0
    fi
    exit 1
}

main "$@"