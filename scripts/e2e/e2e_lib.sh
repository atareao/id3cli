#!/usr/bin/env bash
#
# e2e_lib.sh — runner común de verificación e2e local de id3cli (fase GREEN).
#
# USO (desde un script por extensión):
#   HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "$HERE/e2e_lib.sh"
#   E2E_BIN=""                 # opcional: vacío = cargo build --locked una vez y target/debug/id3cli
#   E2E_KEEP=0                 # 1 = conservar el temporal (--keep)
#   e2e_setup "<ext>"          # ext ∈ {mp3, m4a, ogg, opus, flac}
#   e2e_gen_audio [nombre]     # genera $E2E_AUDIO (silencio anullsrc, -map_metadata -1); nombre por defecto «audio»
#   e2e_gen_covers             # genera $E2E_COVER_JPG y $E2E_COVER_PNG (testsrc)
#   e2e_run <args...>          # ejecuta el binario: stdout→$E2E_OUT, stderr→$E2E_ERR, código→$E2E_RC
#   check_rc <código>          # el último e2e_run devolvió el código esperado
#   check_show_contains <valor>     # grep -a del valor en stdout de show
#   check_show_absent <valor>       # grep -a inverso (estado inicial / tras remove --all)
#   check_stderr_aviso <subcadena>  # grep -a del aviso en stderr
#   check_fail <motivo>             # caso negativo deliberado («FAIL inyectado»)
#   check_ffprobe_tag <clave> <valor>   # ffprobe expone la clave con ese valor
#   check_ffprobe_absent <clave>        # ffprobe NO expone la clave
#   check_ffprobe_raw_contains <patrón> # grep -a del patrón en la salida cruda de ffprobe
#   check_ffprobe_no_user_tags          # tras remove --all: ffprobe sin ninguna clave
#   check_cover_roundtrip <imagen>      # roundtrip de bytes de la carátula extraída
#   check_cover_not_roundtrip <imagen>  # la carátula extraída NO coincide con <imagen> (sustitución)
#   grep_raw_absent <patrón> <archivo>  # el patrón no aparece en los bytes crudos
#   grep_raw_contains <patrón> <archivo># el patrón aparece en los bytes crudos (vendor)
#   check_duration_approx <seg> [tol]   # duración ffprobe ≈ esperada (reproducibilidad)
#   check_no_id3v2_header               # MP3: el bloque ID3v2 desapareció del archivo
#   e2e_summary "<formato>" && exit 0 || exit 1
#
# FASE GREEN (2026-10-16): el runner queda completo. La capa 2 consulta ffprobe
# con el conjunto de tags correcto por formato —format_tags en MP3/MP4 (claves
# minúsculas), stream_tags en OGG/Opus (caja Vorbis) y format_tags en FLAC
# (caja Vorbis, medido tarea 6.3)— mediante la sintaxis
# «-show_entries <formato>=<clave>» verificada con ffmpeg 9.0.2 (la sintaxis con
# «:» da «No match for section», no se expone la clave), y la carátula se extrae
# con «ffmpeg -map 0:v:0 -c copy -frames:v 1» (sin -c copy ffmpeg re-codifica y
# los bytes difieren, medido: 9039 frente a 9056). Todas las aserciones
# alimentan el recuento de e2e_summary; ninguna aborta la ejecución.

set -euo pipefail

# ── Variables globales (exportadas por e2e_setup / e2e_run / aserciones) ──
E2E_BIN=""          # binario del CLI ("" = construir con cargo build --locked)
E2E_DIR=""          # temporal de la ejecución (bajo /tmp/opencode/e2e/<ext>/)
E2E_EXT=""          # extensión/formato: mp3 | m4a | ogg | opus | flac
E2E_OUT=""          # RUTA al fichero con el stdout del último e2e_run
E2E_ERR=""          # RUTA al fichero con el stderr del último e2e_run
E2E_RC=0            # código de salida del último e2e_run
E2E_PASS=0          # contador de aserciones pasadas
E2E_FAIL=0          # contador de aserciones fallidas
E2E_AUDIO=""        # audio generado por e2e_gen_audio
E2E_COVER_JPG=""    # carátula JPG generada por e2e_gen_covers
E2E_COVER_PNG=""    # carátula PNG generada por e2e_gen_covers
E2E_KEEP=0          # 1 = no limpiar el temporal (--keep)
E2E_TMP_BASE="${E2E_TMP_BASE:-/tmp/opencode/e2e}"   # sobreescribible para pruebas

# ── Rutas derivadas (resolución robusta de la ruta del propio script) ──
E2E_SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
E2E_ROOT="$(cd "$E2E_SCRIPTS_DIR/../.." && pwd)"

# ── Tabla de códec por extensión (medida; spec e2e-verification) ──
# mp3 → libmp3lame, m4a → aac, ogg → libvorbis, opus → libopus, flac → flac
declare -A E2E_CODEC=([mp3]=libmp3lame [m4a]=aac [ogg]=libvorbis [opus]=libopus [flac]=flac)

# ── Valores de prueba fijados (spec e2e-verification, «Valores de prueba») ──
# Elegidos para que ningún valor sea subcadena de otro y el grep sea inequívoco
# (salvo los pares «2024» ⊂ «2024-05-01» y «7» ⊂ «7/12» que el propio spec fija:
# el discriminador de esos pares es la ausencia del valor largo, ver scripts).
E2E_V_TITLE="TítuloE2E"
E2E_V_ARTIST1="Uno E2E"
E2E_V_ARTIST2="Dos E2E"
E2E_V_ARTIST_JOINED="Uno E2E; Dos E2E"     # separador multi-artista "; " (sin espacios extra)
E2E_V_ALBUM="ÁlbumE2E"
E2E_V_YEAR="2024"
E2E_V_GENRE="RockE2E"
E2E_V_TRACK="7"
E2E_V_TRACK_TOTAL="7/12"
E2E_V_SEASON="1"
E2E_V_SEASON_TOTAL="1/2"
E2E_V_DATE="2024-05-01"
E2E_V_COPYRIGHT="©2024E2E"
E2E_V_COMPOSER="CompositorE2E"
E2E_V_SUBTITLE="SubtítuloE2E"
E2E_V_ORIGINAL_ARTIST="OriginalArtistE2E"
E2E_V_ALBUM_ARTIST="AlbumArtistE2E"
E2E_V_LYRICS=$'LetraE2E\nSegunda línea E2E'
E2E_V_URL="https://e2e.example.org/id3cli"
E2E_V_ALBUM_SORT="AlbumSortE2E"
E2E_V_ARTIST_SORT="ArtistSortE2E"
E2E_V_TITLE_SORT="TitleSortE2E"

# ── Helper: error fatal en español ──
e2e_die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

# ── e2e_setup <ext>: temporal + trap de limpieza + decisión del binario ──
# Antes de llamarla, el script por extensión puede fijar E2E_BIN (--bin) y
# E2E_KEEP (--keep); vacío/0 es el comportamiento por defecto.
e2e_setup() {
    local ext="${1:?e2e_setup: falta la extensión (mp3|m4a|ogg|opus|flac)}"
    E2E_EXT="$ext"

    # Herramientas obligatorias (spec: ausencia → exit 1 con mensaje en español)
    local tool
    for tool in ffmpeg ffprobe; do
        command -v "$tool" >/dev/null 2>&1 \
            || e2e_die "la herramienta obligatoria «${tool}» no está en el PATH"
    done

    # Temporal no versionado: /tmp/opencode/e2e/<ext>/e2e.<aleatorio>
    local base="${E2E_TMP_BASE}/${E2E_EXT}"
    mkdir -p "$base"
    E2E_DIR="$(mktemp -d "$base/e2e.XXXXXX")"
    trap e2e_cleanup EXIT

    # Ficheros de captura de la última ejecución
    E2E_OUT="$E2E_DIR/stdout.txt"
    E2E_ERR="$E2E_DIR/stderr.txt"

    # Decisión del binario: --bin lo fija el script en E2E_BIN; si está vacío
    # se construye UNA vez por ejecución con cargo build --locked.
    # (El «una sola vez para just e2e» se resuelve en GREEN, grupo 4.)
    if [[ -z "$E2E_BIN" ]]; then
        printf '→ Construyendo id3cli (cargo build --locked)…\n' >&2
        (cd "$E2E_ROOT" && cargo build --locked) \
            || e2e_die "falló cargo build --locked en $E2E_ROOT"
        E2E_BIN="$E2E_ROOT/target/debug/id3cli"
    fi
    [[ -x "$E2E_BIN" ]] \
        || e2e_die "el binario «${E2E_BIN}» no existe o no es ejecutable"

    printf '→ e2e de %s con el binario %s (temporal: %s)\n' "$E2E_EXT" "$E2E_BIN" "$E2E_DIR" >&2
}

# ── Limpieza del temporal (trap EXIT) respetando --keep ──
e2e_cleanup() {
    if [[ "${E2E_KEEP:-0}" -eq 1 ]]; then
        printf '→ Temporal conservado (--keep): %s\n' "${E2E_DIR:-}" >&2
        return 0
    fi
    if [[ -n "${E2E_DIR:-}" && -d "$E2E_DIR" ]]; then
        rm -rf "$E2E_DIR"
    fi
}

# ── Generación del audio de prueba (silencio, sin metadata de origen) ──
# e2e_gen_audio [nombre]: por defecto «audio»; la fase GREEN genera además un
# audio fresco por carátula («cada una sobre audio recién generado», spec), así
# que el nombre del fichero es parametrizable (cover_jpg / cover_png).
# ffmpeg -f lavfi -i "anullsrc=r=48000:cl=mono" -t 1.5 -map_metadata -1 -c:a <códec>
e2e_gen_audio() {
    local codec="${E2E_CODEC[$E2E_EXT]:-}"
    if [[ -z "$codec" ]]; then
        e2e_die "e2e_gen_audio: extensión «${E2E_EXT}» sin códec conocido (mp3|m4a|ogg|opus|flac)"
    fi
    local nombre="${1:-audio}"
    E2E_AUDIO="$E2E_DIR/${nombre}.$E2E_EXT"
    local extra_args=()
    case "$E2E_EXT" in
        mp3)  extra_args=(-q:a 2) ;;
        m4a)  extra_args=(-b:a 128k) ;;
        ogg)  extra_args=(-q:a 3) ;;
        opus) extra_args=() ;;
        flac) extra_args=() ;;
        *)    e2e_die "e2e_gen_audio: extensión «${E2E_EXT}» no soportada" ;;
    esac
    ffmpeg -y -f lavfi -i "anullsrc=r=48000:cl=mono" -t 1.5 -map_metadata -1 \
        -c:a "$codec" "${extra_args[@]}" "$E2E_AUDIO" >/dev/null 2>&1 \
        || e2e_die "e2e_gen_audio: no se pudo generar el audio ${E2E_EXT} con ffmpeg"
    printf '→ Audio generado: %s\n' "$E2E_AUDIO" >&2
}

# ── Generación de las carátulas JPG y PNG (testsrc; sin ImageMagick) ──
e2e_gen_covers() {
    E2E_COVER_JPG="$E2E_DIR/cover.jpg"
    E2E_COVER_PNG="$E2E_DIR/cover.png"
    ffmpeg -y -f lavfi -i "testsrc=duration=0.1:size=320x240:rate=1" -frames:v 1 "$E2E_COVER_JPG" >/dev/null 2>&1 \
        || e2e_die "e2e_gen_covers: no se pudo generar ${E2E_COVER_JPG} con ffmpeg"
    ffmpeg -y -f lavfi -i "testsrc=duration=0.1:size=320x240:rate=1" -frames:v 1 "$E2E_COVER_PNG" >/dev/null 2>&1 \
        || e2e_die "e2e_gen_covers: no se pudo generar ${E2E_COVER_PNG} con ffmpeg"
    printf '→ Carátulas generadas: %s y %s\n' "$E2E_COVER_JPG" "$E2E_COVER_PNG" >&2
}

# ── e2e_run <args...>: ejecuta el binario sin abortar por sus errores ──
# Captura stdout → $E2E_OUT (ruta), stderr → $E2E_ERR (ruta) y código → $E2E_RC.
e2e_run() {
    E2E_RC=0
    "$E2E_BIN" "$@" >"$E2E_OUT" 2>"$E2E_ERR" || E2E_RC=$?
    return 0
}

# ── Aserciones de la capa 1 (id3cli show): grep -a del VALOR, nunca de ANSI ──
check_show_contains() {
    local expected="${1:?check_show_contains: falta el valor esperado}"
    if grep -aqF -- "$expected" "$E2E_OUT"; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_show_contains: valor «%s» encontrado en show\n' "$expected"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_show_contains'\'': el valor esperado «%s» NO aparece en show (grep -a sobre %s)\n' "$expected" "$E2E_OUT" >&2
    fi
    return 0
}

# Aserción de ausencia en show: estado inicial sin campos de usuario y post remove --all.
check_show_absent() {
    local unexpected="${1:?check_show_absent: falta el valor que no debe aparecer}"
    if grep -aqF -- "$unexpected" "$E2E_OUT"; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_show_absent'\'': el valor «%s» aparece en show y no debería\n' "$unexpected" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_show_absent: el valor «%s» NO aparece en show\n' "$unexpected"
    fi
    return 0
}

# ── Aserción de avisos de degradación por stderr ──
check_stderr_aviso() {
    local expected="${1:?check_stderr_aviso: falta el aviso esperado}"
    if grep -aqF -- "$expected" "$E2E_ERR"; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_stderr_aviso: aviso «%s» presente en stderr\n' "$expected"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_stderr_aviso'\'': el aviso esperado «%s» NO aparece en stderr\n' "$expected" >&2
    fi
    return 0
}

# ── check_fail: caso negativo deliberado («FAIL inyectado») ──
# Fuerza un fallo contabilizado; se usa para demostrar la falsabilidad (exit 1).
check_fail() {
    local motivo="${1:-motivo no especificado}"
    E2E_FAIL=$((E2E_FAIL + 1))
    printf 'FAIL: aserción '\''check_fail'\'' (caso negativo deliberado): %s\n' "$motivo" >&2
    return 0
}

# ── check_rc <código>: el último e2e_run devolvió el código esperado ──
# El CLI debe terminar 0 en cada operación del flujo; un fallo de escritura o de
# lectura se detecta aquí y no solo por las aserciones de valor.
check_rc() {
    local expected="${1:?check_rc: falta el código esperado}"
    if [[ "$E2E_RC" -eq "$expected" ]]; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_rc: el CLI devolvió %d como se esperaba\n' "$E2E_RC"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_rc'\'': se esperaba salida %d y el CLI devolvió %d\n' "$expected" "$E2E_RC" >&2
    fi
    return 0
}

# ── Aserciones de la capa 2 (ffprobe) ──
# El conjunto de tags lo decide la extensión (medido 2026-10-16, ffmpeg 9.0.2):
# MP3 y MP4 exponen la metadata en format_tags con claves normalizadas a
# minúsculas; OGG y Opus en stream_tags con la caja Vorbis preservada
# (format_tags vacío en el contenedor Ogg); FLAC en format_tags con la caja
# Vorbis preservada (medido en la tarea 6.3 del change flac-support, igual que
# OGG/Opus salvo el conjunto de entradas). La sintaxis de -show_entries para
# una clave concreta es «format_tags=clave» / «stream_tags=clave» (con «:» la
# versión 9.0.2 responde «No match for section» y no se expone la clave).

# Devuelve el conjunto de entradas de ffprobe donde expone la metadata un formato.
ffprobe_entries_for() {
    case "${1:?ffprobe_entries_for: falta la extensión}" in
        mp3|m4a|flac)  printf 'format_tags' ;;
        ogg|opus)      printf 'stream_tags' ;;
        *)        e2e_die "ffprobe_entries_for: extensión «$1» sin conjunto de tags conocido (mp3|m4a|ogg|opus|flac)" ;;
    esac
}

# ffprobe_tag <ext> <clave>: imprime el valor de la clave para el formato
# (vacío si no está expuesta). Se compara el VALOR: en stream_tags la caja la
# mantiene la clave, no el valor.
ffprobe_tag() {
    local ext="${1:?ffprobe_tag: falta la extensión}"
    local key="${2:?ffprobe_tag: falta la clave}"
    local entries
    entries="$(ffprobe_entries_for "$ext")"
    ffprobe -hide_banner -v error -show_entries "${entries}=${key}" \
        -of default=noprint_wrappers=1:nokey=1 "$E2E_AUDIO" 2>/dev/null || true
}

check_ffprobe_tag() {
    local key="${1:?check_ffprobe_tag: falta la clave ffprobe}"
    local expected="${2:?check_ffprobe_tag: falta el valor esperado}"
    local entries
    entries="$(ffprobe_entries_for "$E2E_EXT")"
    local value
    value="$(ffprobe_tag "$E2E_EXT" "$key")"
    if grep -aqF -- "$expected" <<<"$value"; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_ffprobe_tag: «%s»=%s en %s\n' "$key" "$expected" "$entries"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_ffprobe_tag'\'': se esperaba %s=%s y ffprobe devolvió «%s» (%s)\n' "$key" "$expected" "$value" "$entries" >&2
    fi
    return 0
}

check_ffprobe_absent() {
    local key="${1:?check_ffprobe_absent: falta la clave ffprobe}"
    local entries
    entries="$(ffprobe_entries_for "$E2E_EXT")"
    local value
    value="$(ffprobe_tag "$E2E_EXT" "$key")"
    if [[ -n "$value" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_ffprobe_absent'\'': la clave «%s» aparece en %s con valor «%s» y no debería\n' "$key" "$entries" "$value" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_ffprobe_absent: la clave «%s» NO está expuesta en %s\n' "$key" "$entries"
    fi
    return 0
}

# check_ffprobe_raw_contains <patrón>: grep -aF del patrón (clave y valor, p. ej.
# «YEAR=2024») sobre la salida cruda de ffprobe con claves. Sirve cuando el valor
# es subcadena de otro (2024 ⊂ 2024-05-01) y solo el par clave+valor discrimina.
check_ffprobe_raw_contains() {
    local pattern="${1:?check_ffprobe_raw_contains: falta el patrón}"
    local entries
    entries="$(ffprobe_entries_for "$E2E_EXT")"
    local dump="$E2E_DIR/ffprobe_raw.txt"
    ffprobe -hide_banner -v error -show_entries "$entries" \
        -of default=noprint_wrappers=1 "$E2E_AUDIO" >"$dump" 2>/dev/null || true
    if grep -aqF -- "$pattern" "$dump"; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_ffprobe_raw_contains: patrón «%s» presente en %s\n' "$pattern" "$entries"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_ffprobe_raw_contains'\'': el patrón «%s» NO aparece en %s\n' "$pattern" "$entries" >&2
    fi
    return 0
}

# check_ffprobe_no_user_tags: tras remove --all, ffprobe no expone NINGUNA clave
# de usuario (sin líneas «CLAVE=valor»). Válido para OGG/Opus, donde el
# comentario vacío deja stream_tags sin salida, y para FLAC (medido en la tarea
# 6.3: format_tags queda vacío porque el bloque VORBIS_COMMENT se elimina
# completo — vendor no conservado); en MP4 quedan claves de formato
# (major_brand, …), así que para MP4 se usan check_ffprobe_absent de cada campo.
check_ffprobe_no_user_tags() {
    local entries
    entries="$(ffprobe_entries_for "$E2E_EXT")"
    local salida
    salida="$(ffprobe -hide_banner -v error -show_entries "$entries" \
        -of default=noprint_wrappers=1 "$E2E_AUDIO" 2>/dev/null || true)"
    if [[ -n "$salida" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_ffprobe_no_user_tags'\'': %s sigue exponiendo claves tras remove --all: «%s»\n' "$entries" "$salida" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_ffprobe_no_user_tags: %s sin claves tras remove --all\n' "$entries"
    fi
    return 0
}

# ── Roundtrip de carátula ──
# Extrae el stream de imagen con `ffmpeg -map 0:v:0 -c copy -frames:v 1` (¡sin
# -c copy re-codifica y rompe la comparación!, medido: 9039 frente a 9056 bytes)
# y compara byte a byte contra la original.
check_cover_roundtrip() {
    local imagen="${1:?check_cover_roundtrip: falta la ruta de la imagen original}"
    if [[ ! -f "$imagen" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_cover_roundtrip'\'': la imagen original «%s» no existe\n' "$imagen" >&2
        return 0
    fi
    local ext="${imagen##*.}"
    local extraida="$E2E_DIR/extraida.${ext}"
    if ! ffmpeg -y -i "$E2E_AUDIO" -map 0:v:0 -c copy -frames:v 1 "$extraida" >/dev/null 2>&1; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_cover_roundtrip'\'': no se pudo extraer el stream de imagen de %s con ffmpeg\n' "$E2E_AUDIO" >&2
        return 0
    fi
    if cmp -s "$imagen" "$extraida"; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_cover_roundtrip: %s idéntica a la extracción (%s bytes, byte a byte)\n' "$imagen" "$(wc -c <"$extraida")"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_cover_roundtrip'\'': %s difiere de la extracción (hash original %s, extraído %s)\n' "$imagen" "$(sha256sum "$imagen" | cut -c1-16)" "$(sha256sum "$extraida" | cut -c1-16)" >&2
    fi
    return 0
}

# ── Anti-roundtrip de carátula: la imagen extraída NO debe coincidir ──
# Mismo mecanismo que check_cover_roundtrip (`ffmpeg -map 0:v:0 -c copy
# -frames:v 1` + cmp byte a byte) pero con la aserción INVERTIDA: PASA cuando
# los bytes extraídos DIFIEREN de <imagen> (esa imagen ya no es la carátula del
# archivo) y FALLA cuando coinciden. Fija la semántica de sustitución del fix
# 1.2: si un formato acumulara carátulas, la extracción seguiría devolviendo la
# PRIMERA imagen y esta aserción fallaría con exit 1 (e2e RED sin el fix).
check_cover_not_roundtrip() {
    local imagen="${1:?check_cover_not_roundtrip: falta la ruta de la imagen que ya no debe estar}"
    if [[ ! -f "$imagen" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_cover_not_roundtrip'\'': la imagen «%s» no existe\n' "$imagen" >&2
        return 0
    fi
    local ext="${imagen##*.}"
    local extraida="$E2E_DIR/extraida_no_roundtrip.${ext}"
    if ! ffmpeg -y -i "$E2E_AUDIO" -map 0:v:0 -c copy -frames:v 1 "$extraida" >/dev/null 2>&1; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_cover_not_roundtrip'\'': no se pudo extraer el stream de imagen de %s con ffmpeg\n' "$E2E_AUDIO" >&2
        return 0
    fi
    if cmp -s "$imagen" "$extraida"; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_cover_not_roundtrip'\'': %s SIGUE siendo la carátula extraída (acumulación; la extracción coincide byte a byte, %s bytes)\n' "$imagen" "$(wc -c <"$extraida")" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_cover_not_roundtrip: %s ya NO es la carátula (la extracción difiere byte a byte, %s bytes)\n' "$imagen" "$(wc -c <"$extraida")"
    fi
    return 0
}

# ── Greps en bytes crudos del contenedor ──
# OGG/Opus: la ausencia de WEBSITE=/ORIGINALARTIST= se comprueba en los bytes
# crudos del comentario (capa 3, más allá de ffprobe); el vendor preservado tras
# remove --all se comprueba con grep_raw_contains.
grep_raw_absent() {
    local pattern="${1:?grep_raw_absent: falta el patrón}"
    local file="${2:?grep_raw_absent: falta el archivo}"
    if grep -aqF -- "$pattern" "$file"; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''grep_raw_absent'\'': el patrón «%s» aparece en crudo en %s y no debería\n' "$pattern" "$file" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: grep_raw_absent: el patrón «%s» NO aparece en crudo en %s\n' "$pattern" "$file"
    fi
    return 0
}

grep_raw_contains() {
    local pattern="${1:?grep_raw_contains: falta el patrón}"
    local file="${2:?grep_raw_contains: falta el archivo}"
    if grep -aqF -- "$pattern" "$file"; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: grep_raw_contains: el patrón «%s» aparece en crudo en %s\n' "$pattern" "$file"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''grep_raw_contains'\'': el patrón «%s» NO aparece en crudo en %s\n' "$pattern" "$file" >&2
    fi
    return 0
}

# ── Reproducibilidad: duración de ffprobe ≈ esperada (con tolerancia) ──
# El audio generado dura 1.5 s; tras remove --all el archivo debe seguir
# reconociéndose con la misma duración aprox.
check_duration_approx() {
    local expected="${1:?check_duration_approx: falta la duración esperada}"
    local tol="${2:-0.15}"
    local dur
    dur="$(ffprobe -hide_banner -v error -show_entries format=duration \
        -of default=noprint_wrappers=1:nokey=1 "$E2E_AUDIO" 2>/dev/null || true)"
    if [[ -z "$dur" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_duration_approx'\'': ffprobe no devolvió duración para %s\n' "$E2E_AUDIO" >&2
        return 0
    fi
    if awk -v d="$dur" -v e="$expected" -v t="$tol" 'BEGIN { exit !(d >= e - t && d <= e + t) }'; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_duration_approx: duración %ss ≈ esperada %ss (±%s)\n' "$dur" "$expected" "$tol"
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_duration_approx'\'': duración %ss fuera de %s±%s (audio no reproducible tras la operación)\n' "$dur" "$expected" "$tol" >&2
    fi
    return 0
}

# ── MP3: el bloque ID3v2 desaparece FÍSICAMENTE al eliminar el último campo ──
# El archivo ya no empieza por la firma «ID3» (medido; idéntico a remove --all).
check_no_id3v2_header() {
    if [[ "$(head -c 3 "$E2E_AUDIO" 2>/dev/null)" == "ID3" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''check_no_id3v2_header'\'': el archivo sigue empezando por «ID3» tras remove --all (bloque ID3v2 no eliminado)\n' >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: check_no_id3v2_header: el bloque ID3v2 desapareció del archivo (no empieza por «ID3»)\n'
    fi
    return 0
}

# ── Resumen PASS/FAIL con recuento: exit 0 solo si ninguna aserción falló ──
e2e_summary() {
    local formato="${1:?e2e_summary: falta el nombre del formato}"
    local total=$((E2E_PASS + E2E_FAIL))
    printf 'PASS/FAIL: %s: %d aserciones, %d fallidas\n' "$formato" "$total" "$E2E_FAIL"
    [[ "$E2E_FAIL" -eq 0 ]]
}

# ── ff_key_for <tag>: clave ffprobe del tag según la tabla FF_TAGS del script ──
# Devuelve "" para las celdas «—» (sin clave distinguible: cobertura por show).
ff_key_for() {
    local tag="${1:?ff_key_for: falta el nombre del tag}"
    printf '%s' "${FF_TAGS[$tag]:-}"
}