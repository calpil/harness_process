#!/usr/bin/env bash
# Feature #53: `harness_check` no puede heredar un stdin vivo hacia el guard.
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/harness-stdin.XXXXXX")"
cleanup() { rm -rf "$TMP_ROOT"; }
trap cleanup EXIT

fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

preparar() {
    local raiz="$1"
    mkdir -p "$raiz/hp/progress" "$raiz/servicio"
    cp "$REPO_ROOT/harness_check.sh" "$raiz/hp/harness_check.sh"
    cp "$REPO_ROOT/commit_guard.sh" "$raiz/hp/commit_guard.sh"
    printf '{"features": [], "rules": {}}\n' > "$raiz/hp/feature_list.json"
    # Stub: el caso prueba la frontera stdin/guard, no el binario Rust.
    cat > "$raiz/hp/harness_cli" <<'CLI'
#!/bin/sh
case "${1:-}" in
  status) exit 0 ;;
  check-plan|check-spec) exit 1 ;;
  *) exit 0 ;;
esac
CLI
    chmod +x "$raiz/hp/harness_cli"
    git -C "$raiz/servicio" init -q
    printf 'base\n' > "$raiz/servicio/base.txt"
    git -C "$raiz/servicio" add base.txt
    git -C "$raiz/servicio" -c user.email=t@t -c user.name=t commit -qm base
}

ensuciar_servicio() {
    local raiz="$1"
    printf 'cambio sin commit\n' >> "$raiz/servicio/base.txt"
}

check_no_interactivo_limpio() {
    local raiz="$1" salida rc
    set +e
    HARNESS_REPO_ROOT="$raiz" HARNESS_CHECK_MODE=block \
        bash "$raiz/hp/harness_check.sh" </dev/null > "$raiz/limpio" 2>&1
    rc=$?
    set -e
    salida="$(cat "$raiz/limpio")"
    [ "$rc" -eq 0 ] || fail "check limpio no mantuvo su resultado (rc=$rc): $salida"
    ok "check no interactivo limpio termina sin cambios"
}

check_no_interactivo_termina() {
    local raiz="$1" fifo salida pid escritor rc vivo=1 intento
    fifo="$raiz/stdin-vivo"
    salida="$raiz/salida"
    mkfifo "$fifo"
    # En la versión defectuosa `cat` consume este flujo para siempre. Con stdin
    # cerrado solo para el guard, `harness_check` sale aunque el escritor siga.
    yes '{"payload":"vivo"}' > "$fifo" & escritor=$!
    HARNESS_REPO_ROOT="$raiz" HARNESS_CHECK_MODE=block \
        bash "$raiz/hp/harness_check.sh" < "$fifo" > "$salida" 2>&1 & pid=$!
    for intento in 1 2 3 4 5 6 7 8 9 10; do
        if ! kill -0 "$pid" 2>/dev/null; then
            vivo=0
            break
        fi
        sleep 0.1
    done
    if [ "$vivo" -ne 0 ]; then
        kill "$pid" 2>/dev/null || true
        kill "$escritor" 2>/dev/null || true
        fail "harness_check siguio esperando stdin vivo"
    fi
    set +e
    wait "$pid"; rc=$?
    wait "$escritor" 2>/dev/null
    set -e
    [ "$rc" -eq 2 ] || fail "check no interactivo: exit $rc, esperaba bloqueo finito. Dijo: $(cat "$salida")"
    grep -q "Cambios sin commitear" "$salida" \
        || fail "check no interactivo: dejo de detectar cambios bloqueantes. Dijo: $(cat "$salida")"
    ok "check no interactivo termina con stdin vivo y conserva el bloqueo"
}

guard_directo_conserva_payload() {
    local raiz="$1" salida rc
    # El stop hook es un payload real: con él el guard en modo block deja salir
    # aunque haya un servicio sucio. Sin él, debe bloquear. Si el guard dejara
    # de leer stdin, estos dos casos se volverían indistinguibles.
    salida="$(printf '%s' '{"stop_hook_active":true}' | HARNESS_REPO_ROOT="$raiz" HARNESS_COMMIT_GUARD_MODE=block sh "$raiz/hp/commit_guard.sh" 2>&1)"
    [ -z "$salida" ] || true
    set +e
    printf '%s' '{}' | HARNESS_REPO_ROOT="$raiz" HARNESS_COMMIT_GUARD_MODE=block \
        sh "$raiz/hp/commit_guard.sh" > "$raiz/guard-sin-stop" 2>&1
    rc=$?
    set -e
    [ "$rc" -eq 2 ] || fail "guard directo no evaluó el payload vacío (rc=$rc)"
    grep -q "Cambios sin commitear" "$raiz/guard-sin-stop" \
        || fail "guard directo no informó el servicio sucio"
    ok "guard directo conserva y evalúa el payload del hook"
}

raiz="$TMP_ROOT/proyecto"
preparar "$raiz"
check_no_interactivo_limpio "$raiz"
ensuciar_servicio "$raiz"
check_no_interactivo_termina "$raiz"
guard_directo_conserva_payload "$raiz"
cmp -s "$REPO_ROOT/harness_check.sh" "$REPO_ROOT/templates/harness_check.sh" \
    || fail "fuente y plantilla de harness_check divergen"
ok "stdin del check aislado; hook y espejo intactos"
