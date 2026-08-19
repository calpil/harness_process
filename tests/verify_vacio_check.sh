#!/bin/bash
# Verifica, de punta a punta y con un `cargo test` REAL, que un AC cuyo comando
# no ejecuta ningun caso salga `vacio` y frene el cierre (feature #44).
#
#   filtro-vacio   AC-12  cargo test <nombre-inexistente> -> vacio -> close sale 2
#   filtro-real    AC-12  cargo test <nombre-que-existe>  -> verde -> close pasa
#
# Se usa cargo de verdad a proposito: el falso verde de la #28 nacio de lo que
# `cargo test` imprime cuando el filtro no matchea, y una salida inventada no
# probaria que seguimos leyendo bien lo que cargo imprime HOY.
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

[ -x "$REPO_ROOT/harness" ] || fail "no hay binario en $REPO_ROOT/harness"

# Aislamiento del hub, igual que el helper de rust/tests/cli_basics.rs. Sin esto
# los sandboxes le hablan al Memory Hub PostgreSQL REAL de la maquina (sin
# HARNESS_HUB el binario cae en $HOME/.harness-hub y de ahi saca las DB_*):
# el chequeo escribia en una base compartida y tardaba minutos en vez de
# segundos, con "connection reset by peer" de por medio.
HUB_AISLADO="$(mktemp -d "${TMPDIR:-/tmp}/harness-vacio-hub.XXXXXX")"
trap 'rm -rf "$HUB_AISLADO"' EXIT
export HARNESS_HUB="$HUB_AISLADO"
unset DB_HOST DB_USER DB_PASSWORD DB_NAME DB_PORT

# Sandbox con su propio backlog: no se toca el del repo.
sandbox() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-vacio.XXXXXX")"
    tmp="$(cd "$tmp" && pwd -P)"
    mkdir -p "$tmp/hp/progress" "$tmp/docs"
    printf 'subdir\n' > "$tmp/hp/.harness_layout"
    cp "$REPO_ROOT/harness" "$tmp/hp/harness"
    cp "$REPO_ROOT/harness_cli" "$tmp/hp/harness_cli"
    printf '{"features": [], "rules": {"require_verify_green": true}}\n' > "$tmp/hp/feature_list.json"
    printf '%s' "$tmp"
}

# Deja el spec de la feature 1 con un unico AC que declara $1 como comando.
preparar() {
    tmp="$1"; comando="$2"
    (cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" ./harness add --name Demo >/dev/null)
    (cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" ./harness start --feature 1 >/dev/null)
    spec="$tmp/docs/spec-feature-1-demo.md"
    [ -f "$spec" ] || fail "el sandbox no genero el spec"
    python3 - "$spec" "$comando" <<'PY'
import sys
spec, comando = sys.argv[1], sys.argv[2]
texto = open(spec).read()
marca = "- AC-1: Given <contexto>, When <accion>, Then <resultado observable>.\n  Comando: `<como se prueba, ejecutable desde la raiz>`"
assert marca in texto, "cambio la plantilla del spec"
open(spec, "w").write(texto.replace(marca, f"- AC-1: el unico.\n  Comando: `{comando}`"))
PY
    (cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" ./harness approve-spec --yes >/dev/null)
}

modo_filtro_vacio() {
    tmp="$(sandbox)"
    # Un nombre que no existe en ningun binario de test del repo.
    preparar "$tmp" "cd $REPO_ROOT/rust && cargo test --locked no_existe_este_test_en_ningun_lado_44"
    set +e
    (cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" ./harness verify --feature 1 >/dev/null 2>&1)
    verify_exit=$?
    (cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" ./harness close --feature 1 --status done >"$tmp/close.out" 2>&1)
    close_exit=$?
    set -e
    reporte="$(cat "$tmp/docs/verify-1.md" 2>/dev/null || true)"
    close_out="$(cat "$tmp/close.out" 2>/dev/null || true)"
    rm -rf "$tmp"
    case "$reporte" in
        *"| AC-1 | vacio |"*) ;;
        *) fail "filtro-vacio: el reporte no marco vacio:
$reporte" ;;
    esac
    [ "$verify_exit" -eq 1 ] || fail "filtro-vacio: verify salio $verify_exit, se esperaba 1"
    [ "$close_exit" -eq 2 ] || fail "filtro-vacio: close salio $close_exit, se esperaba 2"
    case "$close_out" in
        *AC-1*) ;;
        *) fail "filtro-vacio: el cierre no nombro el AC:
$close_out" ;;
    esac
    ok "filtro-vacio: cargo test sin coincidencias sale 0, queda vacio y frena el cierre"
}

modo_filtro_real() {
    tmp="$(sandbox)"
    # El mismo comando, con un nombre que SI existe: el contraste es lo que
    # prueba que el chequeo discrimina en vez de rechazar todo `cargo test`.
    preparar "$tmp" "cd $REPO_ROOT/rust && cargo test --locked casos_corridos_should_count_zero_on_the_real_empty_filter"
    set +e
    (cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" ./harness verify --feature 1 >/dev/null 2>&1)
    verify_exit=$?
    (cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" ./harness close --feature 1 --status done >/dev/null 2>&1)
    close_exit=$?
    set -e
    reporte="$(cat "$tmp/docs/verify-1.md" 2>/dev/null || true)"
    rm -rf "$tmp"
    case "$reporte" in
        *"| AC-1 | verde |"*) ;;
        *) fail "filtro-real: un test que SI corre tendria que quedar verde:
$reporte" ;;
    esac
    [ "$verify_exit" -eq 0 ] || fail "filtro-real: verify salio $verify_exit, se esperaba 0"
    [ "$close_exit" -eq 0 ] || fail "filtro-real: close salio $close_exit, se esperaba 0"
    ok "filtro-real: un cargo test que si corre casos sigue verde y deja cerrar"
}

case "$MODO" in
    filtro-vacio) modo_filtro_vacio ;;
    filtro-real) modo_filtro_real ;;
    todos) modo_filtro_vacio; modo_filtro_real; ok "verify-vacio: los dos modos verdes" ;;
    *) fail "modo desconocido: $MODO" ;;
esac
