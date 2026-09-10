#!/usr/bin/env bash
# Feature #86: el Stop de .claude/settings.json invoca `bin/harness-hook claude-json stop`,
# que con el gate en rojo emite por stdout UNA linea {"decision":"block","reason":...}
# con exit 0 (Claude Code y Copilot CLI lo honran; el exit 2 solo Claude), con el
# detalle del check en el reason y lo legible por stderr; con el gate verde o con
# stop_hook_active no imprime nada. Se corre solo o desde el smoke.
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
[ -x "$BIN" ] || { echo "[!] hook_runtime: falta el binario (HARNESS_PREBUILT_BIN o rust/target/debug/harness)" >&2; exit 1; }
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT
fail() { echo "[!] hook_runtime: $*" >&2; exit 1; }

d="$TMP_ROOT/fx"; mkdir -p "$d" "$TMP_ROOT/home"
cp "$REPO_ROOT/setup_harness.sh" "$d/"; cp -R "$REPO_ROOT/templates" "$d/templates"; cp "$BIN" "$d/harness"; chmod +x "$d/harness"
( cd "$d" && HOME="$TMP_ROOT/home" HARNESS_HUB="$TMP_ROOT/hub" KIMI_CODE_HOME="$TMP_ROOT/kimi" \
    DB_HOST=h DB_USER=u DB_PASSWORD=p DB_NAME=d DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity --no-kimi >"$d/.install.log" 2>&1 ) \
    || fail "no se pudo instalar el fixture: $(tail -3 "$d/.install.log")"

# AC-2 (sustancia): el Stop de Claude invoca el modo claude-json.
grep -q 'claude-json stop' "$d/.claude/settings.json" \
    || fail ".claude/settings.json no invoca 'claude-json stop': $(grep -n 'harness-hook' "$d/.claude/settings.json" | head -3)"

# Un repo hermano sucio: el commit guard tiene que ponerse en rojo.
( cd "$d" && git init -q svc && cd svc && git -c user.email=t@t -c user.name=t commit -q --allow-empty -m init && echo sucio > cambio.txt )

correr() { # $1 payload, $2 evento
    out="$TMP_ROOT/out.txt"; err="$TMP_ROOT/err.txt"
    rc=0
    ( cd "$d" && printf '%s' "$1" | bash bin/harness-hook claude-json "$2" >"$out" 2>"$err" ) || rc=$?
}

# 1. Gate rojo -> una linea JSON decision:block, con el detalle, exit 0, stderr con lo legible.
correr '{"hook_event_name":"Stop","stop_hook_active":false}' stop
[ "$rc" -eq 0 ] || fail "claude-json stop con el gate rojo salio $rc (tiene que ser 0: el bloqueo va en el JSON)"
[ "$(grep -c . "$TMP_ROOT/out.txt")" = 1 ] || fail "stdout no es una sola linea: $(cat "$TMP_ROOT/out.txt")"
python3 - "$TMP_ROOT/out.txt" <<'PY' || fail "el JSON del bloqueo no es el esperado: $(cat "$TMP_ROOT/out.txt")"
import json,sys
j=json.load(open(sys.argv[1]))
assert j.get("decision")=="block", j
assert j["reason"].startswith("Harness check fallo"), j["reason"][:80]
assert "sin commitear" in j["reason"], j["reason"]
PY
grep -q "sin commitear" "$TMP_ROOT/err.txt" || fail "lo legible del gate no salio por stderr"

# 2. stop_hook_active (Claude y Copilot lo mandan igual): no bloquea, sin stdout.
correr '{"hook_event_name":"Stop","stop_hook_active":true}' stop
[ "$rc" -eq 0 ] || fail "con stop_hook_active salio $rc"
[ ! -s "$TMP_ROOT/out.txt" ] || fail "con stop_hook_active imprimio: $(cat "$TMP_ROOT/out.txt")"

# 3. Gate verde: sin stdout, exit 0.
( cd "$d/svc" && rm cambio.txt )
correr '{"hook_event_name":"Stop","stop_hook_active":false}' stop
[ "$rc" -eq 0 ] || fail "con el repo limpio salio $rc"
[ ! -s "$TMP_ROOT/out.txt" ] || fail "con el repo limpio imprimio: $(cat "$TMP_ROOT/out.txt")"

# 4. session-start: el estado va por stdout (es contexto), exit 0.
correr '{"hook_event_name":"SessionStart"}' session-start
[ "$rc" -eq 0 ] || fail "claude-json session-start salio $rc"
[ -s "$TMP_ROOT/out.txt" ] || fail "session-start no imprimio el estado por stdout"

echo "[Ok] hook_runtime: claude-json stop bloquea con decision:block (detalle en reason, exit 0), calla con stop_hook_active y con el repo limpio; session-start imprime contexto"
