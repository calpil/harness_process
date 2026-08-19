#!/bin/bash
# Verifica el estado `superseded` sobre el backlog REAL (feature #37).
#
#   migradas   AC-10  las seis que absorbio la #36 estan en superseded, con su
#                     referencia, y prd tree dejo de contarlas
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

modo_migradas() {
    lista="$REPO_ROOT/feature_list.json"
    [ -f "$lista" ] || { ok "migradas: sin feature_list.json (instalacion nueva)"; return; }
    malas=""
    for id in 27 31 32 33 34 35; do
        linea="$(python3 - "$lista" "$id" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
for f in d.get("features", []):
    if str(f.get("id")) == sys.argv[2]:
        print(f"{f.get('status','')}|{f.get('superseded_by','')}")
        break
else:
    print("ausente|")
PY
)"
        estado="${linea%%|*}"
        por="${linea#*|}"
        case "$estado" in
            ausente) ;;
            superseded)
                [ "$por" = "36" ] || malas="$malas #$id(absorbida-por='$por')"
                ;;
            *) malas="$malas #$id($estado)" ;;
        esac
    done
    [ -z "$malas" ] || fail "migradas: no quedaron como superseded por #36:$malas"
    ok "migradas: las seis absorbidas por la #36 quedaron en superseded, con su referencia"
}

case "$MODO" in
    migradas) modo_migradas ;;
    todos) modo_migradas; ok "superseded: los modos verdes" ;;
    *) fail "modo desconocido: $MODO" ;;
esac
