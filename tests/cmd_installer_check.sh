#!/bin/bash
# El instalador para cmd.exe (setup_harness.cmd).
#
# Es un lanzador, no un tercer instalador: todo lo que hace es encontrar
# PowerShell, saltear la ExecutionPolicy y delegar en setup_harness.ps1. Lo que
# se prueba, entonces, es lo unico que puede romper por su cuenta: que traduzca
# las opciones estilo .sh, que devuelva el exit code de verdad y que sus errores
# nombren el remedio.
#
# Los modos que necesitan cmd.exe se saltean con un [Ok] explicito fuera de
# Windows: un skip silencioso se lee igual que un verde, y no lo es.
#
# Modos:
#   existe        el .cmd esta, delega en el .ps1 y nombra el remedio de Git Bash
#   traduce       --dry-run llega al .ps1 como -DryRun (y --no-subagents como -NoSubagents)
#   exit-code     el exit code del .ps1 sale intacto por el .cmd
#   sin-ps1       sin el .ps1 al lado: exit 127 nombrando que falta
#   ci-windows    el smoke runtime y su workflow Windows estan versionados
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

CMD="$REPO_ROOT/setup_harness.cmd"
WINDOWS_SMOKE="$REPO_ROOT/tests/cmd_installer_check.ps1"
WINDOWS_WORKFLOW="$REPO_ROOT/.github/workflows/windows-cmd-installer.yml"

hay_cmd() { command -v cmd.exe >/dev/null 2>&1 && command -v cygpath >/dev/null 2>&1; }

# Sandbox con el .cmd real y un .ps1 falso que solo declara lo que recibio.
sandbox() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-cmd.XXXXXX")"
    cp "$CMD" "$tmp/setup_harness.cmd"
    printf '%s\n' \
        'param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Rest)' \
        '$code = 0' \
        'foreach ($a in $Rest) { if ($a -eq "-Salir3") { $code = 3 } }' \
        '"ARGS:" + ($Rest -join " ")' \
        'exit $code' \
        > "$tmp/setup_harness.ps1"
    echo "$tmp"
}

# `//c` es la forma de pasarle /c desde Git Bash sin que MSYS lo convierta en
# ruta. Y el .cmd se invoca por ruta Windows COMPLETA a proposito: con
# NoDefaultCurrentDirectoryInExePath activo —lo esta en instalaciones de
# Windows endurecidas— cmd.exe no busca en el directorio actual, y el test
# reportaba "no se reconoce el comando" como si el .cmd estuviera roto.
correr_cmd() {
    dir="$1"; shift
    cmd.exe //c "$(cygpath -w "$dir/setup_harness.cmd")" "$@" 2>&1
}

modo_existe() {
    [ -f "$CMD" ] || fail "existe: falta setup_harness.cmd en la raiz"
    grep -q "setup_harness.ps1" "$CMD" \
        || fail "existe: el .cmd no delega en setup_harness.ps1"
    grep -q "ExecutionPolicy Bypass" "$CMD" \
        || fail "existe: no saltea la ExecutionPolicy; un .ps1 sin firmar no arranca"
    grep -q "bash setup_harness.sh" "$CMD" \
        || fail "existe: sin PowerShell no nombra el remedio alternativo (Git Bash)"
    ok "existe: el .cmd delega en el .ps1, saltea la policy y nombra el remedio"
}

modo_traduce() {
    hay_cmd || { ok "traduce: sin cmd.exe (no es Windows), nada que ejecutar"; return; }
    tmp="$(sandbox)"
    salida="$(correr_cmd "$tmp" --dry-run --no-subagents -Force)"
    rm -rf "$tmp"
    printf '%s' "$salida" | grep -q -- "-DryRun" \
        || fail "traduce: --dry-run no llego como -DryRun. Dijo: $salida"
    printf '%s' "$salida" | grep -q -- "-NoSubagents" \
        || fail "traduce: --no-subagents no llego como -NoSubagents. Dijo: $salida"
    printf '%s' "$salida" | grep -q -- "-Force" \
        || fail "traduce: una opcion ya en estilo .ps1 no paso tal cual. Dijo: $salida"
    ok "traduce: --dry-run -> -DryRun, --no-subagents -> -NoSubagents, y lo demas pasa igual"
}

modo_exit_code() {
    hay_cmd || { ok "exit-code: sin cmd.exe (no es Windows), nada que ejecutar"; return; }
    tmp="$(sandbox)"
    correr_cmd "$tmp" -Salir3 >/dev/null 2>&1 && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "3" ] || fail "exit-code: exit $rc, esperaba 3. El del .ps1 tiene que salir intacto: de el dependen los gates."
    ok "exit-code: el exit code del .ps1 sale intacto por el .cmd"
}

modo_sin_ps1() {
    hay_cmd || { ok "sin-ps1: sin cmd.exe (no es Windows), nada que ejecutar"; return; }
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-cmd-sin.XXXXXX")"
    cp "$CMD" "$tmp/setup_harness.cmd"
    salida="$(correr_cmd "$tmp" -Version)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "127" ] || fail "sin-ps1: exit $rc, esperaba 127. Dijo: $salida"
    printf '%s' "$salida" | grep -q "setup_harness.ps1" \
        || fail "sin-ps1: no dice que falta. Dijo: $salida"
    ok "sin-ps1: exit 127 y dice cual archivo falta"
}

modo_ci_windows() {
    [ -f "$WINDOWS_SMOKE" ] \
        || fail "ci-windows: falta el smoke PowerShell que ejecuta cmd.exe real"
    [ -f "$WINDOWS_WORKFLOW" ] \
        || fail "ci-windows: falta el workflow windows-latest"
    grep -q 'Windows_NT' "$WINDOWS_SMOKE" \
        || fail "ci-windows: el smoke no falla fuera de Windows"
    grep -Fq '& $env:ComSpec /d /c' "$WINDOWS_SMOKE" \
        || fail "ci-windows: el smoke no invoca cmd.exe real"
    grep -q 'runs-on: windows-latest' "$WINDOWS_WORKFLOW" \
        || fail "ci-windows: el workflow no usa un runner Windows"
    grep -q 'cmd_installer_check.ps1' "$WINDOWS_WORKFLOW" \
        || fail "ci-windows: el workflow no ejecuta el smoke runtime"
    ok "ci-windows: smoke runtime y workflow Windows presentes"
}

case "$MODO" in
    existe)    modo_existe ;;
    traduce)   modo_traduce ;;
    exit-code) modo_exit_code ;;
    sin-ps1)   modo_sin_ps1 ;;
    ci-windows) modo_ci_windows ;;
    todos)
        modo_existe
        modo_traduce
        modo_exit_code
        modo_sin_ps1
        modo_ci_windows
        ok "instalador cmd: los cinco modos verdes"
        ;;
    *) fail "modo desconocido: $MODO (existe | traduce | exit-code | sin-ps1 | ci-windows | todos)" ;;
esac
