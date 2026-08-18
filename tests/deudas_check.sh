#!/bin/bash
# Verifica el cierre de las deudas de la feature #36.
#
#   backlog-cerrado  AC-13  las entradas #27 y #31-#35 no quedan duplicadas
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

modo_backlog_cerrado() {
    lista="$REPO_ROOT/feature_list.json"
    [ -f "$lista" ] || { ok "backlog-cerrado: sin feature_list.json (instalacion nueva)"; return; }
    abiertas=""
    for id in 27 31 32 33 34 35; do
        estado="$(python3 - "$lista" "$id" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
for f in d.get("features", []):
    if str(f.get("id")) == sys.argv[2]:
        print(f.get("status", ""))
        break
else:
    print("ausente")
PY
)"
        # `blocked` es el estado correcto: el trabajo esta hecho pero en OTRA
        # feature. `done` lo rechaza el gate de spec, y con razon — estas
        # entradas nunca tuvieron spec propio.
        case "$estado" in
            done|blocked|ausente) ;;
            *) abiertas="$abiertas #$id($estado)" ;;
        esac
    done
    [ -z "$abiertas" ] \
        || fail "backlog-cerrado: quedan entradas abiertas que esta feature ya pago:$abiertas"
    ok "backlog-cerrado: las seis entradas quedaron cerradas (absorbidas por la #36), sin duplicar el trabajo"
}

case "$MODO" in
    backlog-cerrado) modo_backlog_cerrado ;;
    todos) modo_backlog_cerrado; ok "deudas: los modos verdes" ;;
    *) fail "modo desconocido: $MODO" ;;
esac
