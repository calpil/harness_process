#!/bin/bash
set -Eeuo pipefail

HARNESS_DIR="$(cd "$(dirname "$0")" && pwd -P)"
AGENT_PROJECT_DIR="${CLAUDE_PROJECT_DIR:-${CODEX_PROJECT_DIR:-${GEMINI_PROJECT_DIR:-${GROK_PROJECT_DIR:-${ANTIGRAVITY_PROJECT_DIR:-}}}}}"
REPO_ROOT="${HARNESS_REPO_ROOT:-$AGENT_PROJECT_DIR}"
if [ -z "$REPO_ROOT" ]; then
    if [ "$(cat "$HARNESS_DIR/.harness_layout" 2>/dev/null)" = "subdir" ]; then
        REPO_ROOT="$(dirname "$HARNESS_DIR")"
    else
        REPO_ROOT="$HARNESS_DIR"
    fi
fi
BRIEF=0
[ "${1:-}" = "--brief" ] && BRIEF=1

dirty=""
for repo in "$REPO_ROOT"/*; do
    [ -d "$repo" ] || continue
    repo_abs="$(cd "$repo" && pwd -P)"
    [ "$repo_abs" = "$HARNESS_DIR" ] && continue
    git -C "$repo" rev-parse --show-toplevel >/dev/null 2>&1 || continue
    git_top="$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null || true)"
    [ "$git_top" = "$repo_abs" ] || continue
    if [ -n "$(git -C "$repo" status --porcelain 2>/dev/null)" ]; then
        dirty="$dirty $(basename "$repo")"
    fi
done

if [ "$BRIEF" -eq 1 ]; then
    [ -n "$dirty" ] && echo "[Harness] Repos dirty:$dirty" || echo "[Harness] Repos limpios"
    exit 0
fi

echo "== Harness Status =="
if [ -f "$HARNESS_DIR/feature_list.json" ]; then
    python3 "$HARNESS_DIR/harness.py" status || true
fi
if [ -n "$dirty" ]; then
    echo "Repos con cambios:$dirty"
else
    echo "Repos con cambios: ninguno"
fi
python3 "$HARNESS_DIR/graph_memory.py" mapa || true

# Chequeo rapido de frescura de plan (multi-LLM) - no bloqueante aqui
# Solo mostramos advertencia si check-plan sale con 2 (stale real).
# Exit 1 = sin feature activa → no es un problema de staleness.
if [ -f "$HARNESS_DIR/feature_list.json" ]; then
    python3 "$HARNESS_DIR/harness.py" check-plan 2>/dev/null || {
        rc=$?
        if [ "$rc" -eq 2 ]; then
            echo "[Harness] Plan puede estar desactualizado (ver 'harness.py check-plan')"
        fi
    }
fi

# Recordatorio fuerte de actualizacion (proceso explicito)
# Se muestra en casi todas las sesiones porque status se llama al inicio.
# Mantiene la filosofia de que para obtener mejoras hay que re-correr el instalador.
if [ "$BRIEF" -eq 0 ]; then
    echo ""
    echo "[Harness] Recordatorio de actualizacion:"
    echo "  El protocolo y herramientas se actualizan re-correndo el instalador"
    echo "  desde la carpeta fuente del harness_process."
    echo "  Ejemplo:  cd /ruta/al/harness_process && ./setup_harness.sh"
    echo "  (o con --reset para regenerar superficies)."
    echo "  Lee 'UPDATING.md' (disponible en tu instalacion) para mas detalles."
fi
