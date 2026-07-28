#!/bin/sh
# harness-managed-hook v4
INPUT=$(cat 2>/dev/null)
MODE="${HARNESS_COMMIT_GUARD_MODE:-block}" # block | warn | off

[ "$MODE" = "off" ] && exit 0

STOP_HOOK_ACTIVE=0
printf '%s' "$INPUT" | grep -q '"stop_hook_active"[[:space:]]*:[[:space:]]*true' && STOP_HOOK_ACTIVE=1

HARNESS_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
AGENT_PROJECT_DIR="${CLAUDE_PROJECT_DIR:-${CODEX_PROJECT_DIR:-${GEMINI_PROJECT_DIR:-${GROK_PROJECT_DIR:-${ANTIGRAVITY_PROJECT_DIR:-}}}}}"
REPO_ROOT="${HARNESS_REPO_ROOT:-$AGENT_PROJECT_DIR}"
if [ -z "$REPO_ROOT" ]; then
    if [ "$(cat "$HARNESS_DIR/.harness_layout" 2>/dev/null)" = "subdir" ]; then
        REPO_ROOT=$(dirname "$HARNESS_DIR")
        # Guardrail checkout fuente (decision usuario 2026-07-28): un clon de
        # la fuente es identico a una instalacion subdir; solo el ENTORNO los
        # distingue. Con senales de fuente en este dir (templates/harness_cli
        # + rust/) y un padre sin huella de instalacion (o $HOME sin
        # HARNESS_ALLOW_HOME_SURFACE=1), el marker 'subdir' es incoherente:
        # fallback al propio arnes con aviso informativo (ni fallo duro ni
        # silencioso).
        if [ -f "$HARNESS_DIR/templates/harness_cli" ] && [ -d "$HARNESS_DIR/rust" ]; then
            harness_parent_footprint=0
            for harness_fp in "docs/constitution.md" "CLAUDE.md" "AGENTS.md" ".claude/settings.json"; do
                if [ -f "$REPO_ROOT/$harness_fp" ]; then
                    harness_parent_footprint=1
                    break
                fi
            done
            harness_parent_is_home=0
            if [ "${HARNESS_ALLOW_HOME_SURFACE:-0}" != "1" ] && [ -n "${HOME:-}" ] \
                && [ "$(cd "$REPO_ROOT" 2>/dev/null && pwd -P)" = "$(cd "$HOME" 2>/dev/null && pwd -P)" ]; then
                harness_parent_is_home=1
            fi
            if [ "$harness_parent_footprint" -eq 0 ] || [ "$harness_parent_is_home" -eq 1 ]; then
                echo "[i] Checkout fuente del arnes detectado (.harness_layout=subdir sin huella de instalacion en el padre): REPO_ROOT=$HARNESS_DIR" >&2
                REPO_ROOT="$HARNESS_DIR"
            fi
        fi
    else
        REPO_ROOT="$HARNESS_DIR"
    fi
fi
DIRTY=""
for repo in "$REPO_ROOT"/*; do
    [ -d "$repo" ] || continue
    repo_abs=$(cd "$repo" && pwd -P)
    [ "$repo_abs" = "$HARNESS_DIR" ] && continue
    git -C "$repo" rev-parse --show-toplevel >/dev/null 2>&1 || continue
    git_top=$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null || true)
    [ "$git_top" = "$repo_abs" ] || continue
    if [ -n "$(git -C "$repo" status --porcelain 2>/dev/null)" ]; then
        DIRTY="$DIRTY $(basename "$repo")"
    fi
done

if [ -n "$DIRTY" ]; then
    echo "Cambios sin commitear en:$DIRTY" >&2
    echo "Haz commit por microservicio con Conventional Commits o usa HARNESS_COMMIT_GUARD_MODE=warn/off." >&2
    if [ "$MODE" = "warn" ] || [ "$STOP_HOOK_ACTIVE" -eq 1 ]; then
        exit 0
    fi
    exit 2
fi
exit 0
