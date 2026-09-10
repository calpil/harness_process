#!/usr/bin/env bash
# Feature #85: Copilot CLI como backend. El instalador escribe .github/copilot.json
# (hooks del arnes, mezclados sobre lo ajeno) y el bloque de
# .github/copilot-instructions.md SOLO si `copilot` esta en el PATH o se pide
# --copilot; --no-copilot lo omite; --reset devuelve los dos archivos a lo ajeno.
# Y bin/harness-hook copilot-json responde el JSON que Copilot espera:
# agentStop -> {"block":true,"reason":...} si el gate falla, {"block":false} si
# pasa o si stopHookActive es true; sessionStart y sessionEnd -> {}.
# Se corre solo (bash tests/copilot_hook_check.sh) o desde el smoke.
set -Eeuo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
BIN="${HARNESS_PREBUILT_BIN:-$REPO_ROOT/rust/target/debug/harness}"
[ -x "$BIN" ] || BIN="$REPO_ROOT/harness"
if [ ! -x "$BIN" ] && command -v git >/dev/null 2>&1; then
    MAIN_ROOT="$(cd "$REPO_ROOT" && git rev-parse --git-common-dir 2>/dev/null)"
    MAIN_ROOT="$(cd "$REPO_ROOT" && cd "$(dirname "$MAIN_ROOT")" 2>/dev/null && pwd -P)"
    [ -n "$MAIN_ROOT" ] && [ -x "$MAIN_ROOT/harness" ] && BIN="$MAIN_ROOT/harness"
fi
if [ ! -x "$BIN" ] && command -v cargo >/dev/null 2>&1; then
    ( cd "$REPO_ROOT/rust" && cargo build --locked >/dev/null 2>&1 ) && BIN="$REPO_ROOT/rust/target/debug/harness"
fi
[ -x "$BIN" ] || { echo "[!] copilot_hook: falta el binario (HARNESS_PREBUILT_BIN o rust/target/debug/harness)" >&2; exit 1; }
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT
fail() { echo "[!] copilot_hook: $*" >&2; exit 1; }

FAKE="$TMP_ROOT/fakebin"; mkdir -p "$FAKE"
printf '#!/bin/sh\necho "GitHub Copilot CLI 1.0.83."\n' > "$FAKE/copilot"; chmod +x "$FAKE/copilot"

# Un fixture por escenario: instala con los flags dados y el PATH dado.
instalar() { # $1 dir, $2 "con-copilot"|"sin-copilot", $3... flags extra
    d="$1"; con="$2"; shift 2
    mkdir -p "$d" "$TMP_ROOT/home"
    cp "$REPO_ROOT/setup_harness.sh" "$d/"; cp -R "$REPO_ROOT/templates" "$d/templates"; cp "$BIN" "$d/harness"; chmod +x "$d/harness"
    # "sin-copilot" tiene que valer aunque la maquina tenga copilot instalado:
    # se saca del PATH el directorio donde este.
    path="$PATH"
    if real="$(command -v copilot 2>/dev/null)"; then
        path="$(printf '%s' "$PATH" | tr ':' '\n' | grep -v -x -F "$(dirname "$real")" | paste -sd: -)"
    fi
    [ "$con" = "con-copilot" ] && path="$FAKE:$path"
    ( cd "$d" && PATH="$path" HOME="$TMP_ROOT/home" HARNESS_HUB="$TMP_ROOT/hub" KIMI_CODE_HOME="$TMP_ROOT/kimi" \
        DB_HOST=h DB_USER=u DB_PASSWORD=p DB_NAME=d DB_SSL_MODE=require \
        bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity --no-kimi "$@" >"$d/.install.log" 2>&1 ) \
        || fail "no se pudo instalar el fixture ($con $*): $(tail -3 "$d/.install.log")"
}

# 1. Con copilot en el PATH: los dos archivos, mezclados sobre lo ajeno.
A="$TMP_ROOT/a"; mkdir -p "$A/.github"
printf '{"model":"gpt-5","hooks":{"toolCall":{"command":"echo x","shell":"bash"}}}\n' > "$A/.github/copilot.json"
printf '# Mis reglas\n\nUsar tabs.\n' > "$A/.github/copilot-instructions.md"
instalar "$A" con-copilot
python3 - "$A/.github/copilot.json" "$A" <<'PY' || fail "copilot.json no quedo como se esperaba"
import json,sys
j=json.load(open(sys.argv[1])); raiz=sys.argv[2]
assert j["model"]=="gpt-5", j
assert j["hooks"]["toolCall"]["command"]=="echo x", j
for ev in ("sessionStart","agentStop","sessionEnd"):
    c=j["hooks"][ev]["command"]
    assert "bin/harness-hook" in c and c.endswith(f"copilot-json {ev}"), (ev, c)
    assert j["hooks"][ev]["shell"]=="bash", j
PY
grep -q '^# Mis reglas' "$A/.github/copilot-instructions.md" || fail "se perdio el texto ajeno de copilot-instructions.md"
[ "$(grep -c 'harness:copilot:inicio' "$A/.github/copilot-instructions.md")" = 1 ] || fail "el bloque del arnes no esta (o esta repetido) en copilot-instructions.md"
grep -q 'AGENTS.md' "$A/.github/copilot-instructions.md" || fail "el bloque no apunta a AGENTS.md"
grep -q -- '--copilot\|Copilot' "$A/AGENTS.md" || fail "AGENTS.md no nombra a Copilot"

# 2. El runtime: agentStop bloquea con el gate en rojo, no bloquea con
#    stopHookActive, ni con el repo limpio; sessionStart y sessionEnd -> {}.
( cd "$A" && git init -q svc && cd svc && git -c user.email=t@t -c user.name=t commit -q --allow-empty -m init && echo sucio > cambio.txt )
salida="$(cd "$A" && printf '{"event":"agentStop","sessionId":"s","stopHookActive":false}' | bash bin/harness-hook copilot-json agentStop 2>/dev/null)" || fail "el hook salio distinto de 0 en agentStop"
printf '%s' "$salida" | grep -q '"block":true' || fail "agentStop con el repo sucio no bloqueo: $salida"
printf '%s' "$salida" | grep -q '"reason":"Harness' || fail "el block no trae reason: $salida"
salida="$(cd "$A" && printf '{"event":"agentStop","sessionId":"s","stopHookActive":true}' | bash bin/harness-hook copilot-json agentStop 2>/dev/null)" || fail "el hook salio distinto de 0 con stopHookActive"
printf '%s' "$salida" | grep -q '"block":false' || fail "con stopHookActive true siguio bloqueando: $salida"
salida="$(cd "$A" && printf '{"event":"sessionEnd","sessionId":"s"}' | bash bin/harness-hook copilot-json sessionEnd 2>/dev/null)" || fail "sessionEnd salio distinto de 0"
[ "$(printf '%s' "$salida" | tr -d '[:space:]')" = "{}" ] || fail "sessionEnd no respondio {}: $salida"
salida="$(cd "$A" && printf '{"event":"sessionStart","sessionId":"s"}' | bash bin/harness-hook copilot-json sessionStart 2>/dev/null)" || fail "sessionStart salio distinto de 0"
[ "$(printf '%s' "$salida" | tr -d '[:space:]')" = "{}" ] || fail "sessionStart no respondio {}: $salida"
( cd "$A/svc" && rm cambio.txt )
salida="$(cd "$A" && printf '{"event":"agentStop","sessionId":"s","stopHookActive":false}' | bash bin/harness-hook copilot-json agentStop 2>/dev/null)" || fail "el hook salio distinto de 0 con el repo limpio"
printf '%s' "$salida" | grep -q '"block":false' || fail "con el repo limpio bloqueo: $salida"

# 3. --reset devuelve los dos archivos a lo ajeno.
( cd "$A" && PATH="$FAKE:$PATH" HOME="$TMP_ROOT/home" HARNESS_HUB="$TMP_ROOT/hub" KIMI_CODE_HOME="$TMP_ROOT/kimi" bash setup_harness.sh --root --reset >"$A/.reset.log" 2>&1 ) || fail "fallo el --reset: $(tail -3 "$A/.reset.log")"
python3 - "$A/.github/copilot.json" <<'PY' || fail "tras el reset copilot.json no volvio a lo ajeno"
import json,sys
j=json.load(open(sys.argv[1]))
assert j=={"model":"gpt-5","hooks":{"toolCall":{"command":"echo x","shell":"bash"}}}, j
PY
[ "$(cat "$A/.github/copilot-instructions.md")" = "$(printf '# Mis reglas\n\nUsar tabs.')" ] || fail "tras el reset copilot-instructions.md no volvio a lo ajeno: $(cat "$A/.github/copilot-instructions.md")"
# Y antes de tocarlos, el reset los respaldo en bkp/ (hallazgo de la revision).
find "$A/bkp" -name 'copilot.json*' 2>/dev/null | grep -q . || fail "el reset no respaldo .github/copilot.json en bkp/"
find "$A/bkp" -name 'copilot-instructions.md*' 2>/dev/null | grep -q . || fail "el reset no respaldo .github/copilot-instructions.md en bkp/"

# 4. Sin copilot en el PATH no se toca .github/; con --copilot se genera igual;
#    con copilot en el PATH y --no-copilot se omite.
B="$TMP_ROOT/b"; instalar "$B" sin-copilot
[ -e "$B/.github/copilot.json" ] && fail "sin copilot en el PATH igual escribio .github/copilot.json"
grep -q "Copilot CLI no detectado" "$B/.install.log" || fail "el instalador no dijo que omitio Copilot"
C="$TMP_ROOT/c"; instalar "$C" sin-copilot --copilot
[ -f "$C/.github/copilot.json" ] || fail "--copilot sin el CLI en el PATH no genero los archivos"
[ -f "$C/.github/copilot-instructions.md" ] || fail "--copilot no genero copilot-instructions.md"
D="$TMP_ROOT/d"; instalar "$D" con-copilot --no-copilot
[ -e "$D/.github/copilot.json" ] && fail "--no-copilot escribio .github/copilot.json igual"
grep -q -- "--no-copilot" "$D/.install.log" || fail "el instalador no dijo que omitio Copilot por --no-copilot"

echo "[Ok] copilot_hook: instalador (deteccion, --copilot, --no-copilot, mezcla, reset) y runtime copilot-json (block/reason, stopHookActive, sessionStart/sessionEnd) verdes"
