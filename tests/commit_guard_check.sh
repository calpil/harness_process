#!/bin/bash
# El commit_guard no cuelga a quien no es un hook.
#
# El guard arranca con `INPUT=$(cat)` porque su uso normal es COMO hook: el
# agente le manda el JSON del evento por la entrada. harness_check.sh lo invoca
# sin ser un hook, y con stdin abierto `cat` espera un EOF que nadie va a
# mandar: el check entero se cuelga (medido en la feature #52: 18 minutos hasta
# matarlo, en una corrida en segundo plano). El arreglo es cerrarle la entrada y
# pasarle por entorno el unico dato que ese JSON traia.
#
# Modos, uno por criterio:
#   no-cuelga       harness_check.sh termina con stdin abierto que nunca cierra
#   prueba-del-rojo la version PREVIA si se cuelga (si no, este test no mide nada)
#   stop-por-env    el guard respeta HARNESS_STOP_HOOK_ACTIVE=1 y no bloquea
#   stop-por-json   como hook, el JSON por stdin sigue mandando
#   bloquea         sin ninguna de las dos senales, un repo sucio sigue bloqueando
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

# Sandbox: un arnes con layout root y UN microservicio sucio colgando, que es lo
# que hace hablar al guard.
sandbox() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-guard.XXXXXX")"
    mkdir -p "$tmp/hp/docs" "$tmp/hp/miservicio"
    cp "$REPO_ROOT/commit_guard.sh" "$tmp/hp/commit_guard.sh"
    cp "$REPO_ROOT/harness_check.sh" "$tmp/hp/harness_check.sh"
    cp "$REPO_ROOT/harness_cli" "$tmp/hp/harness_cli"
    printf 'root\n' > "$tmp/hp/.harness_layout"
    printf '# Constitution\n' > "$tmp/hp/docs/constitution.md"
    git -C "$tmp/hp/miservicio" init -q
    printf 'sin commitear\n' > "$tmp/hp/miservicio/pendiente.txt"
    echo "$tmp"
}

# Corre un comando con una entrada que NUNCA cierra, y responde si termino solo.
# `sleep | cmd` deja el pipe abierto sin mandar un byte: exactamente la corrida
# en segundo plano / CI del hallazgo.
termina_con_stdin_abierto() {
    script="$1"
    dir="$(dirname "$script")"
    (cd "$dir" && sleep 20 | timeout 10 bash "$script" >/dev/null 2>&1) && rc=0 || rc=$?
    # 124 es el timeout: se colgo.
    [ "$rc" != "124" ]
}

modo_no_cuelga() {
    tmp="$(sandbox)"
    if termina_con_stdin_abierto "$tmp/hp/harness_check.sh"; then
        rm -rf "$tmp"
        ok "no-cuelga: harness_check.sh termina con la entrada abierta"
    else
        rm -rf "$tmp"
        fail "no-cuelga: harness_check.sh se colgo esperando EOF (la regresion de la #52)"
    fi
}

modo_prueba_del_rojo() {
    tmp="$(sandbox)"
    # Se reconstruye la invocacion PREVIA al arreglo: sin `</dev/null`.
    sed 's#bash "$HARNESS_DIR/commit_guard.sh" </dev/null#bash "$HARNESS_DIR/commit_guard.sh"#' \
        "$tmp/hp/harness_check.sh" > "$tmp/hp/viejo.sh"
    grep -q 'commit_guard.sh"; then' "$tmp/hp/viejo.sh" \
        || { rm -rf "$tmp"; fail "prueba-del-rojo: no se pudo reconstruir la version previa"; }
    if termina_con_stdin_abierto "$tmp/hp/viejo.sh"; then
        rm -rf "$tmp"
        fail "prueba-del-rojo: la version previa NO se colgo; este test no esta midiendo el cuelgue"
    else
        rm -rf "$tmp"
        ok "prueba-del-rojo: la version previa si se cuelga, asi que el modo no-cuelga mide algo"
    fi
}

modo_stop_por_env() {
    tmp="$(sandbox)"
    salida="$(cd "$tmp/hp" && HARNESS_STOP_HOOK_ACTIVE=1 bash ./commit_guard.sh </dev/null 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "0" ] || fail "stop-por-env: exit $rc, esperaba 0 (no puede bloquear dos veces el mismo turno). Dijo: $salida"
    ok "stop-por-env: con HARNESS_STOP_HOOK_ACTIVE=1 avisa pero no bloquea"
}

modo_stop_por_json() {
    tmp="$(sandbox)"
    salida="$(cd "$tmp/hp" && printf '{"stop_hook_active": true}' | bash ./commit_guard.sh 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "0" ] || fail "stop-por-json: exit $rc, esperaba 0. El camino de hook tiene que seguir intacto. Dijo: $salida"
    ok "stop-por-json: como hook, el JSON por stdin sigue mandando"
}

modo_bloquea() {
    tmp="$(sandbox)"
    salida="$(cd "$tmp/hp" && bash ./commit_guard.sh </dev/null 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "2" ] || fail "bloquea: exit $rc, esperaba 2 con un repo sucio y sin senal de stop. Dijo: $salida"
    printf '%s' "$salida" | grep -q "miservicio" \
        || fail "bloquea: no nombra el repo sucio. Dijo: $salida"
    ok "bloquea: sin senal de stop, un repo sucio sigue bloqueando y se nombra"
}

case "$MODO" in
    no-cuelga)       modo_no_cuelga ;;
    prueba-del-rojo) modo_prueba_del_rojo ;;
    stop-por-env)    modo_stop_por_env ;;
    stop-por-json)   modo_stop_por_json ;;
    bloquea)         modo_bloquea ;;
    todos)
        modo_no_cuelga
        modo_prueba_del_rojo
        modo_stop_por_env
        modo_stop_por_json
        modo_bloquea
        ok "commit_guard: los cinco modos verdes"
        ;;
    *) fail "modo desconocido: $MODO (no-cuelga | prueba-del-rojo | stop-por-env | stop-por-json | bloquea | todos)" ;;
esac
