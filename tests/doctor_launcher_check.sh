#!/bin/bash
# Verifica el lanzador `harness_cli` (feature #25, AC-16).
#
# Es la mitad del diagnostico que `harness_cli doctor` NO puede cubrir: un
# doctor que vive en el binario no puede diagnosticar un binario ausente ni uno
# demasiado viejo para conocer el subcomando. Este test es de shell y no de Rust
# porque lo que se prueba es el script, no el binario.
#
# Modos (uno por criterio, para que cada uno pueda fallar solo):
#   sin-binario   el lanzador nombra el remedio y sale 127
#   binario-viejo un binario que no conoce el subcomando -> remedio, no error de clap
#   no-molesta    un subcomando conocido que falla (gate) NO dispara el aviso
#   no-buferea    stdout/stderr salen mientras el comando corre, no al final
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

# Sandbox con el lanzador y un binario falso que imita a clap.
sandbox() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-launcher.XXXXXX")"
    cp "$REPO_ROOT/harness_cli" "$tmp/harness_cli"
    echo "$tmp"
}

binario_falso_viejo() {
    # Conoce `status` y nada mas: como un binario anterior a la feature nueva.
    cat > "$1/harness" <<'SH'
#!/bin/sh
case "$1" in
    status) echo "status ok"; exit 0 ;;
    help)   [ "$2" = "status" ] && exit 0; echo "error: unrecognized subcommand '$2'" >&2; exit 2 ;;
    *)      echo "error: unrecognized subcommand '$1'" >&2; exit 2 ;;
esac
SH
    chmod +x "$1/harness"
}

modo_sin_binario() {
    tmp="$(sandbox)"
    salida="$(cd "$tmp" && sh ./harness_cli status 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "127" ] || fail "sin-binario: exit $rc, esperaba 127"
    printf '%s' "$salida" | grep -q "setup_harness.sh" \
        || fail "sin-binario: no nombra el remedio. Dijo: $salida"
    ok "sin-binario: exit 127 y nombra el remedio"
}

modo_binario_viejo() {
    tmp="$(sandbox)"
    binario_falso_viejo "$tmp"
    salida="$(cd "$tmp" && sh ./harness_cli doctor 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "2" ] || fail "binario-viejo: exit $rc, esperaba 2 (el del binario, sin alterarlo)"
    printf '%s' "$salida" | grep -q "mas viejo" \
        || fail "binario-viejo: no explica la causa. Dijo: $salida"
    printf '%s' "$salida" | grep -q "Remedio: bash setup_harness.sh" \
        || fail "binario-viejo: no da el remedio copiable. Dijo: $salida"
    ok "binario-viejo: traduce el error de clap al remedio, sin cambiar el exit code"
}

modo_no_molesta() {
    # Un subcomando CONOCIDO que sale 2 (un gate) no puede disparar el aviso:
    # seria un falso positivo en el camino mas transitado del arnes.
    tmp="$(sandbox)"
    cat > "$tmp/harness" <<'SH'
#!/bin/sh
case "$1" in
    help)  exit 0 ;;                       # conoce todo
    close) echo "[GATE] algo falta" >&2; exit 2 ;;
    *)     exit 0 ;;
esac
SH
    chmod +x "$tmp/harness"
    salida="$(cd "$tmp" && sh ./harness_cli close 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "2" ] || fail "no-molesta: exit $rc, esperaba 2"
    printf '%s' "$salida" | grep -q "mas viejo" \
        && fail "no-molesta: aviso de binario viejo en un gate normal. Dijo: $salida"
    printf '%s' "$salida" | grep -q "GATE" \
        || fail "no-molesta: se perdio el mensaje del gate. Dijo: $salida"
    ok "no-molesta: un gate normal pasa intacto y sin aviso espurio"
}

modo_no_buferea() {
    # Con el hub sin responder, `close` tarda ~90s. Si el lanzador capturara la
    # salida para inspeccionarla, el usuario no veria nada hasta el final.
    tmp="$(sandbox)"
    cat > "$tmp/harness" <<'SH'
#!/bin/sh
[ "$1" = "help" ] && exit 0
echo "arranque"
sleep 2
echo "fin"
SH
    chmod +x "$tmp/harness"
    # Se corre en segundo plano y se mira el archivo MIENTRAS el comando sigue
    # vivo. Sin pipe: `| head -n 1` con `pipefail` mata el script por SIGPIPE en
    # vez de medir nada (se descubrio corriendolo).
    salida_file="$tmp/salida.txt"
    (cd "$tmp" && sh ./harness_cli algo >"$salida_file" 2>/dev/null) &
    hijo=$!
    sleep 1
    temprano="$(cat "$salida_file" 2>/dev/null || true)"
    vivo=0
    kill -0 "$hijo" 2>/dev/null && vivo=1
    wait "$hijo" 2>/dev/null || true
    rm -rf "$tmp"
    [ "$vivo" = "1" ] || fail "no-buferea: el comando ya habia terminado; el test no mide nada"
    printf '%s' "$temprano" | grep -q "arranque" \
        || fail "no-buferea: al segundo 1 no habia salida todavia; esta bufereada"
    printf '%s' "$temprano" | grep -q "fin" \
        && fail "no-buferea: el comando termino antes de tiempo; el test no mide nada"
    ok "no-buferea: la salida sale mientras el comando corre"
}

case "$MODO" in
    sin-binario)   modo_sin_binario ;;
    binario-viejo) modo_binario_viejo ;;
    no-molesta)    modo_no_molesta ;;
    no-buferea)    modo_no_buferea ;;
    todos)
        modo_sin_binario
        modo_binario_viejo
        modo_no_molesta
        modo_no_buferea
        ok "lanzador: los cuatro modos verdes"
        ;;
    *) fail "modo desconocido: $MODO (sin-binario | binario-viejo | no-molesta | no-buferea | todos)" ;;
esac
