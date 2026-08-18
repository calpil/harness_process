#!/bin/bash
# Verifica las tres capas de rutas protegidas (feature #26).
#
# Modos, uno por AC, para que cada criterio pueda fallar solo:
#   previene         AC-5   el PreToolUse deniega una escritura protegida
#   detecta          AC-6   el PostToolUse avisa con el comando de reversion
#   red-de-seguridad AC-7   harness_check.sh bloquea (exit 2)
#   no-se-autobloquea AC-10 lo que escribe el arnes no cuenta como violacion
#   compatible       AC-14  sin la feature, harness_check se comporta igual
#   sin-costo        AC-15  sin violacion, el hook no bloquea ni tarda
#
# LIMITE DECLARADO (OBS-3): `previene` verifica el JSON que el hook emite y el
# cableado que el instalador escribe, NO una denegacion real de Claude Code.
# Probar eso de punta a punta exige correr Claude Code, que no esta disponible
# en esta maquina. Es la razon de que las capas 2 y 3 no dependan de la 1.
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

# Sandbox: un repo git con el arnes en subdir, como una instalacion real.
sandbox() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-deny.XXXXXX")"
    tmp="$(cd "$tmp" && pwd -P)"
    mkdir -p "$tmp/hp/progress" "$tmp/docs/prd"
    printf 'subdir\n' > "$tmp/hp/.harness_layout"
    cp "$REPO_ROOT/harness_cli" "$tmp/hp/harness_cli"
    cp "$REPO_ROOT/harness" "$tmp/hp/harness"
    cp "$REPO_ROOT/harness_check.sh" "$tmp/hp/harness_check.sh"
    cp "$REPO_ROOT/commit_guard.sh" "$tmp/hp/commit_guard.sh" 2>/dev/null || true
    cp "$REPO_ROOT/harness_status.sh" "$tmp/hp/harness_status.sh" 2>/dev/null || true
    cp "$REPO_ROOT/init.sh" "$tmp/hp/init.sh" 2>/dev/null || true
    printf '{"features": [], "rules": {}}\n' > "$tmp/hp/feature_list.json"
    printf '# Constitution\n' > "$tmp/docs/constitution.md"
    printf '# PRD\n' > "$tmp/docs/prd/PRD-master.md"
    printf 'CLAUDE\n' > "$tmp/CLAUDE.md"
    (cd "$tmp" && git init -q && git add -A && git -c user.email=t@t -c user.name=t commit -qm base)
    printf '%s' "$tmp"
}

# El runtime de hooks, extraido del instalador tal como lo escribe.
hook_runtime() {
    dest="$1/hp/bin"
    mkdir -p "$dest"
    awk '/^write_harness_hook_runtime\(\)/{f=1} f&&/^HOOK_RUNTIME_EOF$/{exit} f' \
        "$REPO_ROOT/setup_harness.sh" \
        | sed -n "/^#!\/bin\/bash/,\$p" \
        | sed 's|__HREL_NOSLASH__|hp|g; s|__WITH_SUBAGENTS__|1|g' > "$dest/harness-hook"
    chmod +x "$dest/harness-hook"
}

modo_previene() {
    # 1) El instalador cablea PreToolUse sobre Edit|Write|MultiEdit.
    grep -q '"PreToolUse"' "$REPO_ROOT/setup_harness.sh" \
        || fail "previene: el instalador no cablea PreToolUse"
    grep -q 'plain PreToolUse' "$REPO_ROOT/setup_harness.sh" \
        || fail "previene: PreToolUse no invoca el runtime de hooks"
    # 2) El runtime deniega una ruta protegida y deja pasar una que no lo es.
    tmp="$(sandbox)"; hook_runtime "$tmp"
    protegida="$(printf '{"tool_input":{"file_path":"%s/docs/constitution.md"}}' "$tmp" \
        | HARNESS_REPO_ROOT="$tmp" bash "$tmp/hp/bin/harness-hook" plain PreToolUse 2>/dev/null)"
    libre="$(printf '{"tool_input":{"file_path":"%s/src/main.rs"}}' "$tmp" \
        | HARNESS_REPO_ROOT="$tmp" bash "$tmp/hp/bin/harness-hook" plain PreToolUse 2>/dev/null)"
    rm -rf "$tmp"
    printf '%s' "$protegida" | grep -q '"permissionDecision":"deny"' \
        || fail "previene: no denego una ruta protegida. Emitio: $protegida"
    printf '%s' "$protegida" | grep -q "documento del USUARIO" \
        || fail "previene: la razon no explica por que. Emitio: $protegida"
    printf '%s' "$libre" | grep -q '"deny"' \
        && fail "previene: denego una ruta NO protegida. Emitio: $libre"
    ok "previene: PreToolUse cableado, deniega lo protegido y deja pasar el resto"
}

modo_detecta() {
    tmp="$(sandbox)"; hook_runtime "$tmp"
    printf '\n<!-- toque del agente -->\n' >> "$tmp/docs/constitution.md"
    salida="$(HARNESS_REPO_ROOT="$tmp" bash "$tmp/hp/bin/harness-hook" plain PostToolUse 2>&1 || true)"
    rm -rf "$tmp"
    printf '%s' "$salida" | grep -q "RUTA PROTEGIDA" \
        || fail "detecta: no aviso. Dijo: $salida"
    printf '%s' "$salida" | grep -q "docs/constitution.md" \
        || fail "detecta: no nombra la ruta. Dijo: $salida"
    printf '%s' "$salida" | grep -q "git checkout -- " \
        || fail "detecta: no da el comando de reversion. Dijo: $salida"
    printf '%s' "$salida" | grep -q "DESCARTA" \
        || fail "detecta: no avisa que el comando descarta lo no commiteado. Dijo: $salida"
    ok "detecta: avisa con la ruta, el comando y lo que ese comando descarta"
}

modo_red_de_seguridad() {
    tmp="$(sandbox)"
    printf '\n<!-- toque del agente -->\n' >> "$tmp/docs/prd/PRD-master.md"
    set +e
    salida="$(HARNESS_CHECK_MODE=block bash "$tmp/hp/harness_check.sh" 2>&1)"
    rc=$?
    set -e
    rm -rf "$tmp"
    [ "$rc" -eq 2 ] || fail "red-de-seguridad: exit $rc, esperaba 2. Dijo: $salida"
    printf '%s' "$salida" | grep -q "PROTEGIDAS" \
        || fail "red-de-seguridad: no lo reporto. Dijo: $salida"
    printf '%s' "$salida" | grep -q "docs/prd/PRD-master.md" \
        || fail "red-de-seguridad: no nombra la ruta. Dijo: $salida"
    ok "red-de-seguridad: bloquea con exit 2 y nombra la ruta"
}

modo_no_se_autobloquea() {
    # Lo que escribe el ARNES queda exento: se simula con el mismo comando que
    # usa el binario al marcar un hito.
    tmp="$(sandbox)"
    printf '\n<!-- hito marcado por el arnes -->\n' >> "$tmp/docs/prd/PRD-master.md"
    HARNESS_REPO_ROOT="$tmp" sh "$tmp/hp/harness_cli" rutas --aceptar-estado-actual >/dev/null 2>&1
    set +e
    HARNESS_CHECK_MODE=block bash "$tmp/hp/harness_check.sh" >/dev/null 2>&1
    rc=$?
    set -e
    rm -rf "$tmp"
    [ "$rc" -ne 2 ] || fail "no-se-autobloquea: bloqueo por una escritura del propio arnes"
    ok "no-se-autobloquea: lo que escribe el arnes no cuenta como violacion"
}

modo_compatible() {
    # Proteccion apagada (lista vacia): harness_check se comporta como antes.
    tmp="$(sandbox)"
    printf '{"features": [], "rules": {"rutas_protegidas": []}}\n' > "$tmp/hp/feature_list.json"
    printf '\n<!-- toque -->\n' >> "$tmp/docs/constitution.md"
    set +e
    salida="$(HARNESS_CHECK_MODE=block bash "$tmp/hp/harness_check.sh" 2>&1)"
    rc=$?
    set -e
    rm -rf "$tmp"
    printf '%s' "$salida" | grep -q "PROTEGIDAS" \
        && fail "compatible: reporto con la proteccion apagada. Dijo: $salida"
    [ "$rc" -ne 2 ] || fail "compatible: bloqueo con la proteccion apagada"
    ok "compatible: con la lista vacia no reporta ni bloquea"
}

modo_sin_costo() {
    tmp="$(sandbox)"; hook_runtime "$tmp"
    inicio=$(date +%s)
    set +e
    salida="$(HARNESS_REPO_ROOT="$tmp" bash "$tmp/hp/bin/harness-hook" plain PostToolUse 2>&1)"
    rc=$?
    set -e
    fin=$(date +%s)
    rm -rf "$tmp"
    [ "$rc" -eq 0 ] || fail "sin-costo: el hook fallo sin violaciones (rc=$rc): $salida"
    printf '%s' "$salida" | grep -q "RUTA PROTEGIDA" \
        && fail "sin-costo: aviso sin que hubiera violacion. Dijo: $salida"
    [ $((fin - inicio)) -le 5 ] \
        || fail "sin-costo: el hook tardo $((fin - inicio))s sin violaciones"
    ok "sin-costo: sin violacion no avisa, no falla y no tarda"
}

case "$MODO" in
    previene)          modo_previene ;;
    detecta)           modo_detecta ;;
    red-de-seguridad)  modo_red_de_seguridad ;;
    no-se-autobloquea) modo_no_se_autobloquea ;;
    compatible)        modo_compatible ;;
    sin-costo)         modo_sin_costo ;;
    todos)
        modo_previene
        modo_detecta
        modo_red_de_seguridad
        modo_no_se_autobloquea
        modo_compatible
        modo_sin_costo
        ok "rutas protegidas: los seis modos verdes"
        ;;
    *) fail "modo desconocido: $MODO" ;;
esac
