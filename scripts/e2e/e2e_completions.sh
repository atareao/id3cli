#!/usr/bin/env bash
#
# e2e_completions.sh — verificación e2e local del subcomando `completions`
# (fase GREEN, change shell-completions).
#
# Uso: scripts/e2e/e2e_completions.sh [--bin RUTA] [--keep]
#   --bin RUTA  usa el binario indicado en lugar de `cargo build --locked`.
#   --keep      conserva el temporal de la ejecución (bajo /tmp/opencode/e2e/completions/).
#
# FLUJO VERIFICADO (GREEN, implementado):
#   Este script NO toca archivos de audio: la capa de completions se genera
#   sobre el árbol de comandos del CLI, sin leer ningún fichero, así que no hay
#   e2e_gen_audio/e2e_gen_covers (e2e_setup los deja disponibles, pero aquí no
#   se necesitan). El flujo verifica, por orden:
#   1. `completions bash`: exit 0, stdout con el marcador estable `_id3cli` y la
#      subcadena `complete -F _id3cli` (el resto de flags del template de
#      clap_complete NO se fijan), y stderr VACÍO.
#   2. Oráculo `bash -n` sobre una copia del script generado: el propio
#      intérprete valida que el template completo es sintácticamente válido.
#   3. Los cuatro subcomandos del CLI (show, edit, remove, completions) figuran
#      en el script bash.
#   4. `completions zsh`: exit 0, `#compdef id3cli` y stderr VACÍO.
#   5. `completions fish`: exit 0, `complete -c id3cli` y stderr VACÍO.
#   6. `completions powershell` (shell NO soportado): exit 2 (error de clap con
#      el value_parser de GenerationShell), mensaje en español por stderr que
#      nombra los shells válidos (bash, zsh, fish) y stdout VACÍO.
#   Todas las aserciones alimentan el recuento PASS/FAIL de e2e_summary
#   (e2e_lib.sh) y ninguna aborta la ejecución: el fallo se mide por el
#   contador, igual que en los scripts por formato.

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

    # El temporal usa la extensión «completions» como nombre de directorio
    # (/tmp/opencode/e2e/completions/); no se genera audio ni carátulas.
    e2e_setup "completions"

    # 1) Bash: script de completions con exit 0, marcador estable `_id3cli` y la
    #    subcadena `complete -F _id3cli` (la línea completa lleva además flags
    #    del template de clap_complete 4.5 como `-o nosort -o bashdefault`, que
    #    el spec no fija), y stderr VACÍO (la salida viaja toda por stdout).
    e2e_run completions bash
    check_rc 0
    check_show_contains '_id3cli'
    check_show_contains 'complete -F _id3cli'
    if [[ -s "$E2E_ERR" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''stderr vacío'\'': completions bash escribió en stderr: «%s»\n' "$(tr -d '\n' <"$E2E_ERR")" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: stderr vacío en completions bash\n'
    fi

    # 2) Oráculo `bash -n`: una copia del script generado debe ser válida para
    #    el propio intérprete. Valida el template completo de clap_complete,
    #    más allá de las subcadenas estables comprobadas en el punto 1.
    local script="$E2E_DIR/completions_bash.sh"
    cp "$E2E_OUT" "$script"
    if bash -n "$script"; then
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: bash -n: el script bash generado es sintácticamente válido\n'
    else
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''bash -n'\'': el script bash generado no es sintácticamente válido\n' >&2
    fi

    # 3) Los cuatro subcomandos del CLI figuran en el script bash (rama del case
    #    con el dispatch `id3cli,<subcomando>)`).
    check_show_contains 'show'
    check_show_contains 'edit'
    check_show_contains 'remove'
    check_show_contains 'completions'

    # 4) Zsh: declaración `#compdef id3cli` con exit 0 y stderr VACÍO.
    e2e_run completions zsh
    check_rc 0
    check_show_contains '#compdef id3cli'
    if [[ -s "$E2E_ERR" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''stderr vacío'\'': completions zsh escribió en stderr: «%s»\n' "$(tr -d '\n' <"$E2E_ERR")" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: stderr vacío en completions zsh\n'
    fi

    # 5) Fish: líneas `complete -c id3cli` con exit 0 y stderr VACÍO.
    e2e_run completions fish
    check_rc 0
    check_show_contains 'complete -c id3cli'
    if [[ -s "$E2E_ERR" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''stderr vacío'\'': completions fish escribió en stderr: «%s»\n' "$(tr -d '\n' <"$E2E_ERR")" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: stderr vacío en completions fish\n'
    fi

    # 6) Shell NO soportado (powershell): el value_parser de GenerationShell lo
    #    rechaza con exit 2 y un error en español que nombra los shells válidos
    #    (bash, zsh y fish); el error viaja por stderr y stdout queda VACÍO.
    e2e_run completions powershell
    check_rc 2
    check_stderr_aviso 'bash'
    check_stderr_aviso 'zsh'
    check_stderr_aviso 'fish'
    if [[ -s "$E2E_OUT" ]]; then
        E2E_FAIL=$((E2E_FAIL + 1))
        printf 'FAIL: aserción '\''stdout vacío'\'': completions con shell no soportado escribió en stdout: «%s»\n' "$(tr -d '\n' <"$E2E_OUT")" >&2
    else
        E2E_PASS=$((E2E_PASS + 1))
        printf 'PASS: stdout vacío en el error de shell no soportado\n'
    fi

    if e2e_summary "completions"; then
        exit 0
    fi
    exit 1
}

main "$@"