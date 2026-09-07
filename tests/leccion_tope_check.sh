#!/usr/bin/env bash
# Feature #80: harness_check.sh avisa [i] por cada leccion de CLASE que supera
# rules.leccion_max_lineas (default 250), y NO avisa por los archivos de
# <clase>/referencias/ ni cuando la regla esta en 0. Solo avisa: el bloqueo es
# del cierre. Se corre solo (bash tests/leccion_tope_check.sh) o desde el smoke.
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
[ -x "$BIN" ] || { echo "[!] leccion_tope: falta el binario (HARNESS_PREBUILT_BIN o rust/target/debug/harness)" >&2; exit 1; }
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT
fail() { echo "[!] leccion_tope: $*" >&2; exit 1; }

d="$TMP_ROOT/fx"; mkdir -p "$d" "$TMP_ROOT/home" "$TMP_ROOT/hub"
cp "$REPO_ROOT/setup_harness.sh" "$d/"; cp -R "$REPO_ROOT/templates" "$d/templates"; cp "$BIN" "$d/harness"; chmod +x "$d/harness"
( cd "$d" && HOME="$TMP_ROOT/home" HARNESS_HUB="$TMP_ROOT/hub" \
    DB_HOST=h DB_USER=u DB_PASSWORD=p DB_NAME=d DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity >"$d/.install.log" 2>&1 ) || fail "no se pudo instalar el fixture: $(tail -3 "$d/.install.log")"
[ -f "$d/harness_check.sh" ] || fail "el instalador no dejo harness_check.sh en la raiz del fixture"

leccion() { # $1 nombre, $2 lineas de cuerpo
    mkdir -p "$d/docs/lecciones"
    { printf -- '---\nnombre: %s\ndescripcion: Leccion de prueba.\ntriggers: [prueba]\nusos: 0\nultimo_uso:\nultima_actualizacion: 2026-09-06\nestado: activa\n---\n\n## Cuando aplica\n\n' "$1"
      seq 1 "$2" | sed 's/^/linea de relleno /'; } > "$d/docs/lecciones/$1.md"
}
leccion larga 300
leccion corta 20
# Una referencia enorme NO cuenta: el tope es de la leccion de clase.
mkdir -p "$d/docs/lecciones/corta/referencias"; seq 1 400 | sed 's/^/detalle /' > "$d/docs/lecciones/corta/referencias/detalle.md"

correr() { ( cd "$d" && bash harness_check.sh >"$d/.check.out" 2>"$d/.check.err" || true ); }

correr
grep -q 'docs/lecciones/larga.md tiene 3[0-9][0-9] lineas y el tope es 250' "$d/.check.err" || fail "sin la regla, no aviso por la leccion larga con el default 250: $(grep -c . "$d/.check.err") lineas de stderr"
grep -q 'referencias/' "$d/.check.err" && grep -q 'partirla' "$d/.check.err" || fail "el aviso no dice a donde va el detalle"
grep -q 'lecciones/corta.md tiene' "$d/.check.err" && fail "aviso por la leccion corta, que esta bajo el tope"
grep -q 'detalle.md' "$d/.check.err" && fail "aviso por un archivo de referencias/, que no cuenta"

python3 - "$d/feature_list.json" 0 <<'PY'
import json,sys
p=sys.argv[1]; j=json.load(open(p)); j.setdefault('rules',{})['leccion_max_lineas']=int(sys.argv[2]); json.dump(j,open(p,'w'),indent=2)
PY
correr
grep -q 'el tope es' "$d/.check.err" && fail "con la regla en 0 sigue avisando"

python3 - "$d/feature_list.json" 400 <<'PY'
import json,sys
p=sys.argv[1]; j=json.load(open(p)); j['rules']['leccion_max_lineas']=int(sys.argv[2]); json.dump(j,open(p,'w'),indent=2)
PY
correr
grep -q 'el tope es' "$d/.check.err" && fail "con el tope en 400 avisa por una leccion de 3xx lineas"

echo "[Ok] leccion_tope: el check avisa [i] por la leccion sobre el tope, no por referencias/, y calla con la regla en 0 o mas alta"
