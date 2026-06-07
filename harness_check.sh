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

MODE="${HARNESS_CHECK_MODE:-block}" # block | warn | off
[ "$MODE" = "off" ] && exit 0

failures=0

echo "== Harness Check =="

if [ -f "$HARNESS_DIR/feature_list.json" ]; then
    python3 "$HARNESS_DIR/harness.py" status || failures=$((failures + 1))

    # NUEVO: Gate de frescura del plan (multi-LLM).
    # Si hay feature in_progress y el plan en docs/ fue modificado por otro LLM
    # (Claude, Gemini, Antigravity/agy, Grok, Codex...), bloqueamos hasta que
    # el implementer re-lee el plan actualizado.
    if python3 "$HARNESS_DIR/harness.py" check-plan >/dev/null 2>&1; then
        : # plan fresco
    else
        echo "[!] Plan desactualizado (modificado por otro LLM). Ejecuta 'python3 harness.py check-plan' y re-lee el plan antes de continuar." >&2
        failures=$((failures + 1))
    fi
fi

if [ -f "$HARNESS_DIR/CHECKPOINTS.md" ] && [ ! -s "$HARNESS_DIR/progress/current.md" ]; then
    echo "[!] progress/current.md esta vacio; registra estado antes de cerrar." >&2
    failures=$((failures + 1))
fi

if [ -f "$REPO_ROOT/graphify-out/.graphify_stale" ]; then
    echo "[!] graphify-out/.graphify_stale existe; corre /graphify --update cuando aplique." >&2
    failures=$((failures + 1))
fi

if ! bash "$HARNESS_DIR/commit_guard.sh"; then
    failures=$((failures + 1))
fi

# Integridad del mapa de agentes (solo si la capa de subagentes esta instalada).
# roles/ vive junto a los scripts; los subagentes nativos de Claude viven en la
# raiz multi-repo (REPO_ROOT/.claude/agents) para que Claude Code los registre.
if [ -d "$HARNESS_DIR/roles" ]; then
    for role in leader implementer reviewer; do
        if [ ! -f "$HARNESS_DIR/roles/$role.md" ]; then
            echo "[!] Falta roles/$role.md; el mapa de agentes esta incompleto." >&2
            failures=$((failures + 1))
        fi
        agent_md="$REPO_ROOT/.claude/agents/$role.md"
        if [ -f "$agent_md" ]; then
            if [ "$(head -n1 "$agent_md")" != "---" ]; then
                echo "[!] .claude/agents/$role.md sin frontmatter YAML; Claude Code no lo registrara como subagente." >&2
                failures=$((failures + 1))
            elif ! grep -q '^name:' "$agent_md" || ! grep -q '^description:' "$agent_md"; then
                echo "[!] .claude/agents/$role.md: frontmatter sin name: o description:." >&2
                failures=$((failures + 1))
            fi
        fi
        codex_toml="$REPO_ROOT/.codex/agents/$role.toml"
        if [ -f "$codex_toml" ] && ! grep -q '^developer_instructions' "$codex_toml"; then
            echo "[!] .codex/agents/$role.toml sin developer_instructions." >&2
            failures=$((failures + 1))
        fi
        gemini_md="$REPO_ROOT/.gemini/agents/$role.md"
        if [ -f "$gemini_md" ] && [ "$(head -n1 "$gemini_md")" != "---" ]; then
            echo "[!] .gemini/agents/$role.md sin frontmatter YAML." >&2
            failures=$((failures + 1))
        fi
    done
fi

if [ "$failures" -gt 0 ]; then
    echo "[Harness] Check fallo con $failures problema(s)." >&2
    [ "$MODE" = "warn" ] && exit 0
    exit 2
fi

echo "[Ok] Harness Check limpio."
