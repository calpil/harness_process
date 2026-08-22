#!/bin/bash
set -Eeuo pipefail

HARNESS_DIR="$(cd "$(dirname "$0")" && pwd -P)"
AGENT_PROJECT_DIR="${CLAUDE_PROJECT_DIR:-${CODEX_PROJECT_DIR:-${GEMINI_PROJECT_DIR:-${GROK_PROJECT_DIR:-${ANTIGRAVITY_PROJECT_DIR:-}}}}}"
REPO_ROOT="${HARNESS_REPO_ROOT:-$AGENT_PROJECT_DIR}"
if [ -z "$REPO_ROOT" ]; then
    # Resolucion de la raiz multi-repo segun el marker .harness_layout. Tres
    # casos EXCLUYENTES (feature #10): marker 'subdir' -> el padre (con el
    # guardrail de checkout fuente de la feature #7 intacto); marker AUSENTE ->
    # inferencia por huella de instalacion del padre; marker presente con
    # cualquier otro valor ('root') -> el propio dir del arnes, sin inferencia.
    harness_marker="$HARNESS_DIR/.harness_layout"
    harness_parent="$(dirname "$HARNESS_DIR")"
    # Huella de instalacion del arnes en el padre (las MISMAS cuatro rutas y la
    # MISMA guarda de $HOME para el guardrail y para la inferencia).
    harness_parent_footprint=0
    for harness_fp in "docs/constitution.md" "CLAUDE.md" "AGENTS.md" ".claude/settings.json"; do
        if [ -f "$harness_parent/$harness_fp" ]; then
            harness_parent_footprint=1
            break
        fi
    done
    harness_parent_is_home=0
    if [ "${HARNESS_ALLOW_HOME_SURFACE:-0}" != "1" ] && [ -n "${HOME:-}" ] \
        && [ "$(cd "$harness_parent" 2>/dev/null && pwd -P)" = "$(cd "$HOME" 2>/dev/null && pwd -P)" ]; then
        harness_parent_is_home=1
    fi
    harness_layout=""
    if [ -f "$harness_marker" ]; then
        # Sin el `tr`: un marker escrito en Windows llega como "subdir\r", no
        # matchea, y la resolucion de raiz se va al camino equivocado SIN decir
        # nada. La version Rust ya hacia trim(); estos cuatro scripts no.
        harness_layout="$(tr -d '\r' < "$harness_marker" 2>/dev/null || true)"
    fi
    if [ "$harness_layout" = "subdir" ]; then
        REPO_ROOT="$harness_parent"
        # Guardrail checkout fuente (decision usuario 2026-07-28): un clon de
        # la fuente es identico a una instalacion subdir; solo el ENTORNO los
        # distingue. Con senales de fuente en este dir (templates/harness_cli
        # + rust/) y un padre sin huella de instalacion (o $HOME sin
        # HARNESS_ALLOW_HOME_SURFACE=1), el marker 'subdir' es incoherente:
        # fallback al propio arnes con aviso informativo (ni fallo duro ni
        # silencioso).
        if [ -f "$HARNESS_DIR/templates/harness_cli" ] && [ -d "$HARNESS_DIR/rust" ]; then
            if [ "$harness_parent_footprint" -eq 0 ] || [ "$harness_parent_is_home" -eq 1 ]; then
                echo "[i] Checkout fuente del arnes detectado (.harness_layout=subdir sin huella de instalacion en el padre): REPO_ROOT=$HARNESS_DIR" >&2
                REPO_ROOT="$HARNESS_DIR"
            fi
        fi
    elif [ ! -f "$harness_marker" ]; then
        # Marker AUSENTE (decision usuario 2026-07-29): la feature #7 des-versiono
        # .harness_layout, asi que toda instalacion subdir que hace 'git pull' se
        # queda sin marker. Si el padre tiene huella de instalacion (y no es
        # $HOME), se infiere layout subdir y la raiz es el padre, avisando con
        # [i]. Sin huella no hay evidencia para inferir nada. Los scripts son
        # READ-ONLY: regenerar el marker es trabajo del instalador.
        if [ "$harness_parent_footprint" -eq 1 ] && [ "$harness_parent_is_home" -eq 0 ]; then
            echo "[i] .harness_layout ausente: layout subdir inferido por la huella de instalacion del padre: REPO_ROOT=$harness_parent. Re-corre el instalador (setup_harness.sh / setup_harness.ps1) para regenerar el marker." >&2
            REPO_ROOT="$harness_parent"
        else
            REPO_ROOT="$HARNESS_DIR"
        fi
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
    sh "$HARNESS_DIR/harness_cli" status || true
fi
if [ -n "$dirty" ]; then
    echo "Repos con cambios:$dirty"
else
    echo "Repos con cambios: ninguno"
fi
sh "$HARNESS_DIR/harness_cli" graph mapa || true

# Chequeo rapido de frescura de plan (multi-LLM) - no bloqueante aqui
# Solo mostramos advertencia si check-plan sale con 2 (stale real).
# Exit 1 = sin feature activa → no es un problema de staleness.
if [ -f "$HARNESS_DIR/feature_list.json" ]; then
    sh "$HARNESS_DIR/harness_cli" check-plan 2>/dev/null || {
        rc=$?
        if [ "$rc" -eq 2 ]; then
            echo "[Harness] Plan puede estar desactualizado (ver 'sh harness_cli check-plan')"
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
