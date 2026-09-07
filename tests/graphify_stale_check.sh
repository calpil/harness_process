#!/usr/bin/env bash
# Feature #83: graphify-out/.graphify_stale es un marcador de enriquecimiento
# best-effort (lo deja el hook post-commit o autocheck; lo limpian ellos). El
# check lo AVISA con [i] y no lo cuenta como fallo: un Stop hook no puede
# bloquear por un rebuild semantico pendiente. Se corre solo o desde el smoke.
set -Eeuo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
BIN="${HARNESS_PREBUILT_BIN:-$REPO_ROOT/rust/target/debug/harness}"
[ -x "$BIN" ] || BIN="$REPO_ROOT/harness"
# En un worktree no hay binario propio (feature #83, leccion #78: un test que
# cae por una precondicion no es rojo): se toma el del checkout principal, y si
# tampoco esta, se compila.
if [ ! -x "$BIN" ] && command -v git >/dev/null 2>&1; then
    MAIN_ROOT="$(cd "$REPO_ROOT" && git rev-parse --git-common-dir 2>/dev/null)"
    MAIN_ROOT="$(cd "$REPO_ROOT" && cd "$(dirname "$MAIN_ROOT")" 2>/dev/null && pwd -P)"
    [ -n "$MAIN_ROOT" ] && [ -x "$MAIN_ROOT/harness" ] && BIN="$MAIN_ROOT/harness"
fi
if [ ! -x "$BIN" ] && command -v cargo >/dev/null 2>&1; then
    ( cd "$REPO_ROOT/rust" && cargo build --locked >/dev/null 2>&1 ) && BIN="$REPO_ROOT/rust/target/debug/harness"
fi
[ -x "$BIN" ] || { echo "[!] graphify_stale: falta el binario (HARNESS_PREBUILT_BIN o rust/target/debug/harness)" >&2; exit 1; }
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT
fail() { echo "[!] graphify_stale: $*" >&2; exit 1; }

d="$TMP_ROOT/fx"; mkdir -p "$d" "$TMP_ROOT/home" "$TMP_ROOT/hub"
cp "$REPO_ROOT/setup_harness.sh" "$d/"; cp -R "$REPO_ROOT/templates" "$d/templates"; cp "$BIN" "$d/harness"; chmod +x "$d/harness"
( cd "$d" && HOME="$TMP_ROOT/home" HARNESS_HUB="$TMP_ROOT/hub" \
    DB_HOST=h DB_USER=u DB_PASSWORD=p DB_NAME=d DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity >"$d/.install.log" 2>&1 ) || fail "no se pudo instalar el fixture: $(tail -3 "$d/.install.log")"
[ -f "$d/harness_check.sh" ] || fail "el instalador no dejo harness_check.sh"
# Un proyecto sano: el check tiene que pasar limpio ANTES de plantar el marcador.
# current.md vacio es [!] en un fixture recien instalado: se le da contenido.
printf '# Estado\n\nfixture del test #83\n' > "$d/progress/current.md"
correr() { ( cd "$d" && bash harness_check.sh >"$d/.out" 2>"$d/.err"; echo $? > "$d/.rc" ); cat "$d/.rc"; }

rc="$(correr)"
[ "$rc" = "0" ] || fail "el fixture no pasa el check ni sin marcador (rc=$rc): $(grep -E '^\[!\]' "$d/.err" | head -3)"
grep -q 'graphify_stale' "$d/.err" && fail "sin marcador, el check lo menciona"

mkdir -p "$d/graphify-out"; touch "$d/graphify-out/.graphify_stale"
rc="$(correr)"
grep -q '^\[i\] graphify-out/.graphify_stale' "$d/.err" || fail "con el marcador no sale el [i] (stderr: $(grep -c . "$d/.err") lineas; primer [!]: $(grep -m1 -E '^\[!\]' "$d/.err"))"
grep -qE '^\[!\].*graphify_stale' "$d/.err" && fail "el marcador sigue saliendo como [!]"
[ "$rc" = "0" ] || fail "con el marcador (y nada mas) el check falla con rc=$rc: bloquea el Stop"
grep -q 'post-commit' "$d/.err" && grep -q 'graphify --update' "$d/.err" && grep -qi 'no bloquea' "$d/.err" \
    || fail "el aviso no dice quien lo limpia, como forzarlo y que no bloquea"

echo "[Ok] graphify_stale: el marcador avisa con [i], dice quien lo limpia y no hace fallar el check"
