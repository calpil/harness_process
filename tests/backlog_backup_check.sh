#!/bin/bash
# Feature #78: el instalador no puede dejar el backlog sin respaldo, ni borrarlo
# con --reset, ni sembrarlo en silencio cuando falta.
#
# Cuatro corridas sobre un backlog PLANTADO (features y reglas distintas de la
# plantilla, para que "sigue igual" no pueda pasar por casualidad):
#   reinstall    reinstalar encima: backlog byte-identico y copia en bkp/
#   reset        --reset: idem — el backlog no es superficie generada
#   reset-force  --reset --force: idem — --force no saltea el respaldo de DATOS
#   faltante     se borra el backlog y se reinstala: se siembra la plantilla pero
#                con [WARN] propio que nombra el respaldo que hay en bkp/
#   todos        los cuatro
#
# Medido el 2026-09-06 contra el instalador previo a la #78: --reset lo BORRABA
# (con respaldo), reinstall no lo respaldaba nunca, y la siembra callaba.
set -Eeuo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $MODO: $*" >&2; exit 1; }
ok() { echo "[Ok] $*"; }
BIN="${HARNESS_PREBUILT_BIN:-$REPO_ROOT/harness}"
[ -x "$BIN" ] || fail "falta el binario prebuilt en $BIN (compila con cargo build --release y copialo, o exporta HARNESS_PREBUILT_BIN)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/harness-backlog-bkp.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT

fixture() {
    mkdir -p "$1"
    cp "$REPO_ROOT/setup_harness.sh" "$1/setup_harness.sh"
    cp -R "$REPO_ROOT/templates" "$1/templates"
    cp "$BIN" "$1/harness"; chmod +x "$1/harness"
}
instalar() { # $1 dir, $2 flags
    ( cd "$1" && HOME="$TMP_ROOT/home" HARNESS_HUB="$TMP_ROOT/hub" \
        DB_HOST=h DB_USER=u DB_PASSWORD=p DB_NAME=d DB_SSL_MODE=require \
        bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity $2 >"$1/.log" 2>&1 ) || true
}
plantar() {
    python3 - "$1" <<'PY'
import json,io,sys
d=sys.argv[1]; p=d+'/feature_list.json'; b=json.load(io.open(p,encoding='utf-8'))
b['features']=[{"id":1,"name":"Viva","status":"in_progress"},{"id":2,"name":"Hecha","status":"done"}]
b['rules']={"require_spec_approved":True,"require_review":True,"one_feature_at_a_time":True,"require_tests_to_close":True,"require_impact_check":True}
io.open(p,'w',encoding='utf-8').write(json.dumps(b,indent=2)+'\n')
io.open(d+'/progress/history.md','a',encoding='utf-8').write('- 2026-09-06 start feature #1 Viva\n')
PY
    cp "$1/feature_list.json" "$1/.backlog.antes"; cp "$1/progress/history.md" "$1/.history.antes"
}
sigue_igual() {
    cmp -s "$1/feature_list.json" "$1/.backlog.antes" || fail "$2: el backlog CAMBIO (o desaparecio)"
    cmp -s "$1/progress/history.md" "$1/.history.antes" || fail "$2: history.md CAMBIO (o desaparecio)"
}
respaldo_en_bkp() {
    ls "$1"/bkp/feature_list.json.bak.* >/dev/null 2>&1 || fail "$2: no hay bkp/feature_list.json.bak.* (el instalador no respaldo el backlog)"
    ls "$1"/bkp/progress/history.md.bak.* >/dev/null 2>&1 || ls "$1"/bkp/progress.bak.* >/dev/null 2>&1 \
        || fail "$2: no hay respaldo de progress/history.md en bkp/"
    # Y el respaldo es EL backlog plantado, no la plantilla.
    ultimo="$(ls -t "$1"/bkp/feature_list.json.bak.* | head -1)"
    cmp -s "$ultimo" "$1/.backlog.antes" || fail "$2: el respaldo en bkp/ no coincide con el backlog que habia"
}

modo_reinstall()   { d="$TMP_ROOT/reinstall";   fixture "$d"; instalar "$d" "";  plantar "$d"; instalar "$d" "";                sigue_igual "$d" reinstall;   respaldo_en_bkp "$d" reinstall;   ok "reinstall: backlog intacto y respaldado en bkp/"; }
modo_reset()       { d="$TMP_ROOT/reset";       fixture "$d"; instalar "$d" "";  plantar "$d"; instalar "$d" "--reset";         sigue_igual "$d" reset;       respaldo_en_bkp "$d" reset;       ok "reset: --reset no borra el backlog ni progress/, y los respalda"; }
modo_reset_force() { d="$TMP_ROOT/reset-force"; fixture "$d"; instalar "$d" "";  plantar "$d"; instalar "$d" "--reset --force"; sigue_igual "$d" reset-force; respaldo_en_bkp "$d" reset-force; ok "reset-force: --force no saltea el respaldo de los datos"; }
modo_faltante() {
    d="$TMP_ROOT/faltante"; fixture "$d"; instalar "$d" ""; plantar "$d"
    # Una corrida mas deja el respaldo en bkp/; despues el backlog "desaparece".
    instalar "$d" ""; rm "$d/feature_list.json"; instalar "$d" ""
    test -f "$d/feature_list.json" || fail "faltante: el instalador no sembro el backlog (la instalacion tiene que terminar)"
    grep -q "WARN" "$d/.log" || fail "faltante: sembro el backlog en SILENCIO, sin [WARN]"
    grep -qi "feature_list.json" "$d/.log" || fail "faltante: el aviso no nombra el backlog"
    grep -q "feature_list.json.bak" "$d/.log" || fail "faltante: el aviso no nombra el respaldo que hay en bkp/"
    grep -qiE "2 feature" "$d/.log" || fail "faltante: el aviso no dice cuantas features tiene el respaldo"
    ok "faltante: siembra, pero avisa en [WARN] y nombra el respaldo con sus features"
}

case "$MODO" in
    reinstall)   modo_reinstall ;;
    reset)       modo_reset ;;
    reset-force) modo_reset_force ;;
    faltante)    modo_faltante ;;
    todos)       modo_reinstall; modo_reset; modo_reset_force; modo_faltante; ok "backlog_backup: los cuatro modos verdes" ;;
    *) fail "modo desconocido: $MODO (reinstall | reset | reset-force | faltante | todos)" ;;
esac
