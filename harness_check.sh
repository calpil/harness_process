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
        harness_layout="$(cat "$harness_marker" 2>/dev/null || true)"
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

MODE="${HARNESS_CHECK_MODE:-block}" # block | warn | off
[ "$MODE" = "off" ] && exit 0

failures=0

echo "== Harness Check =="

if [ -f "$HARNESS_DIR/feature_list.json" ]; then
    sh "$HARNESS_DIR/harness_cli" status || failures=$((failures + 1))

    # Gate de frescura del plan (multi-LLM).
    # check-plan sale con:
    #   0 = OK (plan fresco)
    #   2 = genuinamente stale (plan modificado por otro LLM → fallo)
    #   1 = sin feature in_progress (no lo tratamos como fallo de stale aquí)
    # Solo bloqueamos si realmente hay feature activa y el plan está desactualizado.
    if sh "$HARNESS_DIR/harness_cli" check-plan >/dev/null 2>&1; then
        : # exit 0: fresco
    else
        rc=$?
        if [ "$rc" -eq 2 ]; then
            echo "[!] Plan desactualizado (modificado por otro LLM). Ejecuta 'sh harness_cli check-plan' y re-lee el plan antes de continuar." >&2
            failures=$((failures + 1))
        fi
        # rc=1 (sin feature) u otros: no incrementamos failures para este gate
    fi

    # Gate de spec aprobado (Spec-Driven Development).
    # check-spec sale con:
    #   0 = OK (regla require_spec_approved apagada, o spec aprobado y fresco)
    #   2 = spec stale, o regla activa con spec ausente/draft/no aprobado → fallo
    #   1 = sin feature in_progress (no lo tratamos como fallo aquí)
    if sh "$HARNESS_DIR/harness_cli" check-spec >/dev/null 2>&1; then
        : # exit 0: regla apagada o spec aprobado y fresco
    else
        rc=$?
        if [ "$rc" -eq 2 ]; then
            echo "[!] Spec sin aprobar o modificado. Ejecuta 'sh harness_cli check-spec'; si esta en draft, mostrale el spec al USUARIO, preguntale si lo aprueba y con su SI registra 'sh harness_cli approve-spec --yes'." >&2
            failures=$((failures + 1))
        fi
        # rc=1 (sin feature) u otros: no incrementamos failures para este gate
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

# Cuerpo embebido de un espejo Markdown (Claude/Gemini): todo lo que sigue al
# cierre del frontmatter (segundo '---'), sin las lineas en blanco iniciales
# (el instalador inserta una entre frontmatter y cuerpo).
extract_agent_body() {
    awk 'started { print; next }
         inbody { if ($0 ~ /^[[:space:]]*$/) next; started=1; print; next }
         /^---[[:space:]]*$/ { fm++; if (fm == 2) inbody=1; next }' "$1"
}

# Cuerpo embebido de un espejo Codex: el bloque developer_instructions entre
# comillas triples (envoltorio fijo de build_codex_agent en el instalador).
extract_codex_body() {
    awk -v q="'''" 'started { if ($0 == q) exit; print; next }
         inblock { if ($0 == q) exit; if ($0 ~ /^[[:space:]]*$/) next; started=1; print; next }
         $0 ~ /^developer_instructions[[:space:]]*=/ { inblock=1 }' "$1"
}

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
        kimi_md="$REPO_ROOT/.kimi-code/agents/$role.md"
        if [ -f "$kimi_md" ]; then
            if [ "$(head -n1 "$kimi_md")" != "---" ]; then
                echo "[!] .kimi-code/agents/$role.md sin frontmatter YAML; Kimi Code no lo registrara como subagente." >&2
                failures=$((failures + 1))
            elif ! grep -q '^name:' "$kimi_md" || ! grep -q '^description:' "$kimi_md"; then
                echo "[!] .kimi-code/agents/$role.md: frontmatter sin name: o description:." >&2
                failures=$((failures + 1))
            fi
        fi

        # Gate de espejo (decision usuario 2026-07-28): roles/ es la fuente
        # unica; los espejos generados por el instalador deben llevar el MISMO
        # cuerpo. Solo se comparan espejos existentes (una instalacion
        # --no-subagents o un checkout sin .gemini/.codex no falla). El check
        # SOLO reporta (read-only): el remedio es re-correr el instalador, o
        # propagar el cambio a roles/ si lo editado fue el espejo.
        if [ -f "$HARNESS_DIR/roles/$role.md" ]; then
            role_body="$(cat "$HARNESS_DIR/roles/$role.md")"
            if [ -f "$agent_md" ] && [ "$(extract_agent_body "$agent_md")" != "$role_body" ]; then
                echo "[!] Espejo desincronizado: .claude/agents/$role.md (leido por Claude y Grok) no coincide con roles/$role.md. Re-corre el instalador (setup_harness.sh / setup_harness.ps1) para regenerarlo; si lo que editaste fue el espejo, propaga el cambio a roles/$role.md." >&2
                failures=$((failures + 1))
            fi
            if [ -f "$gemini_md" ] && [ "$(extract_agent_body "$gemini_md")" != "$role_body" ]; then
                echo "[!] Espejo desincronizado: .gemini/agents/$role.md no coincide con roles/$role.md. Re-corre el instalador (setup_harness.sh / setup_harness.ps1) para regenerarlo; si lo que editaste fue el espejo, propaga el cambio a roles/$role.md." >&2
                failures=$((failures + 1))
            fi
            if [ -f "$kimi_md" ] && [ "$(extract_agent_body "$kimi_md")" != "$role_body" ]; then
                echo "[!] Espejo desincronizado: .kimi-code/agents/$role.md (leido por Kimi Code) no coincide con roles/$role.md. Re-corre el instalador (setup_harness.sh / setup_harness.ps1) para regenerarlo; si lo que editaste fue el espejo, propaga el cambio a roles/$role.md." >&2
                failures=$((failures + 1))
            fi
            if [ -f "$codex_toml" ] && [ "$(extract_codex_body "$codex_toml")" != "$role_body" ]; then
                echo "[!] Espejo desincronizado: .codex/agents/$role.toml no coincide con roles/$role.md. Re-corre el instalador (setup_harness.sh / setup_harness.ps1) para regenerarlo; si lo que editaste fue el espejo, propaga el cambio a roles/$role.md." >&2
                failures=$((failures + 1))
            fi
        fi
    done

    # Sub-gate raiz <-> templates (Articulo 6), modulo __HREL__: roles/<f>.md
    # debe equivaler a templates/roles/<f>.md bajo ALGUNA de las dos
    # expansiones validas del placeholder (prefijo "<basename del arnes>/" en
    # layout subdir, o vacio en layout root/flat). Condicional a que
    # templates/roles/ exista (la distribucion aplanada no lo trae).
    if [ -d "$HARNESS_DIR/templates/roles" ]; then
        harness_hrel="$(basename "$HARNESS_DIR")/"
        for role_file in leader implementer reviewer README; do
            role_src="$HARNESS_DIR/roles/$role_file.md"
            role_tpl="$HARNESS_DIR/templates/roles/$role_file.md"
            if [ ! -f "$role_src" ] || [ ! -f "$role_tpl" ]; then
                echo "[!] Espejo roles/ <-> templates/roles/ incompleto: falta roles/$role_file.md o templates/roles/$role_file.md." >&2
                failures=$((failures + 1))
                continue
            fi
            role_src_body="$(cat "$role_src")"
            if [ "$role_src_body" != "$(sed "s|__HREL__|$harness_hrel|g" "$role_tpl")" ] \
                && [ "$role_src_body" != "$(sed "s|__HREL__||g" "$role_tpl")" ]; then
                echo "[!] Divergencia roles/$role_file.md vs templates/roles/$role_file.md (modulo __HREL__). Propaga el cambio al otro lado en el mismo commit (regla de espejo del Articulo 6)." >&2
                failures=$((failures + 1))
            fi
        done
    fi
fi

# Constitution del proyecto: vive en el docs/ de la RAIZ (junto a planes y
# specs) y el instalador la siembra solo si falta. Mismo criterio que el bloque
# de roles ([ -d roles ] = instalacion completa) para no romper instalaciones
# minimas sin esa capa.
if [ -d "$HARNESS_DIR/roles" ] && [ ! -f "$REPO_ROOT/docs/constitution.md" ]; then
    echo "[!] Falta docs/constitution.md (principios del proyecto). Re-corre el instalador (setup_harness.sh / setup_harness.ps1) para sembrarla." >&2
    failures=$((failures + 1))
fi

if [ "$failures" -gt 0 ]; then
    echo "[Harness] Check fallo con $failures problema(s)." >&2
    [ "$MODE" = "warn" ] && exit 0
    exit 2
fi

echo "[Ok] Harness Check limpio."
