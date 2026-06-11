#!/bin/bash
set -Eeuo pipefail
IFS=$'\n\t'

HARNESS_DIR="$(cd "$(dirname "$0")" && pwd -P)"
# REPO_ROOT: raiz multi-repo. Prioriza override explicito y variables de agente;
# en runs manuales usa el marcador .harness_layout.
AGENT_PROJECT_DIR="${CLAUDE_PROJECT_DIR:-${CODEX_PROJECT_DIR:-${GEMINI_PROJECT_DIR:-${GROK_PROJECT_DIR:-${ANTIGRAVITY_PROJECT_DIR:-}}}}}"
REPO_ROOT="${HARNESS_REPO_ROOT:-$AGENT_PROJECT_DIR}"
if [ -z "$REPO_ROOT" ]; then
    if [ "$(cat "$HARNESS_DIR/.harness_layout" 2>/dev/null)" = "subdir" ]; then
        REPO_ROOT="$(dirname "$HARNESS_DIR")"
    else
        REPO_ROOT="$HARNESS_DIR"
    fi
fi
cd "$REPO_ROOT"

# Carpeta donde se archivan los respaldos *.bak.* generados por el harness.
BKP_DIR="${HARNESS_BKP_DIR:-$HARNESS_DIR/bkp}"

echo "== Inicializando Harness Process: arnes=$HARNESS_DIR raiz=$REPO_ROOT =="

if ! command -v git >/dev/null 2>&1; then
    echo "[!] git no esta disponible." >&2
    exit 1
fi

if [ ! -x "$HARNESS_DIR/harness" ] && [ ! -x "$HARNESS_DIR/harness.exe" ]; then
    echo "[!] Binario 'harness'/'harness.exe' no disponible; ejecuta setup con cargo/rustup para compilarlo. (sin fallback Python)" >&2
    exit 1
fi

OS="$(uname -s)"
DESIRED_AUTOCRLF=""
case "$OS" in
    Darwin|Linux) DESIRED_AUTOCRLF="input" ;;
    MINGW*|MSYS*|CYGWIN*) DESIRED_AUTOCRLF="true" ;;
esac

echo "== Sincronizando hooks de microservicios =="
for repo in */; do
    REPO_DIR="${repo%/}"
    [ -d "$REPO_DIR" ] || continue
    if ! git -C "$REPO_DIR" rev-parse --show-toplevel >/dev/null 2>&1; then
        continue
    fi
    REPO_ABS="$(cd "$REPO_DIR" && pwd -P)"
    [ "$REPO_ABS" = "$HARNESS_DIR" ] && continue  # el propio arnes no es microservicio
    GIT_TOP="$(git -C "$REPO_DIR" rev-parse --show-toplevel 2>/dev/null || true)"
    [ "$GIT_TOP" = "$REPO_ABS" ] || continue

    if [ -n "$DESIRED_AUTOCRLF" ]; then
        CURRENT_AUTOCRLF="$(git -C "$REPO_DIR" config --local --get core.autocrlf || true)"
        if [ "$CURRENT_AUTOCRLF" != "$DESIRED_AUTOCRLF" ]; then
            git -C "$REPO_DIR" config core.autocrlf "$DESIRED_AUTOCRLF"
            echo "   -> [Git] $REPO_DIR core.autocrlf=$DESIRED_AUTOCRLF"
        fi
    fi

    GIT_DIR="$(git -C "$REPO_DIR" rev-parse --git-dir)"
    case "$GIT_DIR" in
        /*) ;;
        *) GIT_DIR="$REPO_DIR/$GIT_DIR" ;;
    esac
    HOOKS_DIR="$GIT_DIR/hooks"
    mkdir -p "$HOOKS_DIR"
    POST_COMMIT="$HOOKS_DIR/post-commit"
    COMMIT_MSG="$HOOKS_DIR/commit-msg"

    for hook in "$POST_COMMIT" "$COMMIT_MSG"; do
        if [ -f "$hook" ] && ! grep -q "harness-managed-hook" "$hook"; then
            hook_bkp="$BKP_DIR/git-hooks"
            mkdir -p "$hook_bkp"
            backup="$hook_bkp/$(basename "$hook").bak.$(date +%Y%m%d%H%M%S)"
            cp "$hook" "$backup"
            echo "   -> [Backup] hook previo: $backup"
        elif [ ! -f "$hook" ]; then
            echo "#!/bin/bash" > "$hook"
        fi
    done

    if ! grep -q "harness-managed-hook v9" "$POST_COMMIT"; then
        tmp_hook="${POST_COMMIT}.tmp"
        if [ -f "$POST_COMMIT" ]; then
            sed '/harness-managed-hook/,$d' "$POST_COMMIT" > "$tmp_hook" && mv "$tmp_hook" "$POST_COMMIT"
        fi
        cat >> "$POST_COMMIT" <<HOOKEOF

# harness-managed-hook v9
set -u
HARNESS_DIR="$HARNESS_DIR"
REPO_ROOT="$REPO_ROOT"
MICROSERVICIO=\$(basename "\$(git rev-parse --show-toplevel)")
COMMIT_HASH=\$(git rev-parse HEAD)
ARCHIVOS=\$(git diff-tree --no-commit-id --name-only -r --root "\$COMMIT_HASH" | paste -sd "," -)
COMMIT_MSG_BODY=\$(git log -1 --format=%B "\$COMMIT_HASH")
sh "\$HARNESS_DIR/harness_cli" graph sync_git --artefacto "\$COMMIT_HASH" --meta "\$ARCHIVOS" --microservicio "\$MICROSERVICIO" \
  || echo "[Harness] Aviso: no se pudo sincronizar memoria para \$MICROSERVICIO." >&2

export PATH="\$HOME/.local/bin:\$PATH"
if command -v graphify >/dev/null 2>&1 && [ -f "\$REPO_ROOT/graphify-out/graph.json" ]; then
    if mkdir "\$REPO_ROOT/graphify-out/.update.lock" 2>/dev/null; then
        (
            exec >> "\$REPO_ROOT/graphify-out/background_update.log" 2>&1
            echo "[$(date)] Iniciando graphify update para commit \$COMMIT_HASH"
            trap 'rmdir "\$REPO_ROOT/graphify-out/.update.lock" 2>/dev/null || true' EXIT
            cd "\$REPO_ROOT" || exit 0
            graphify update "\$REPO_ROOT" || true
            if printf '%s' "\$ARCHIVOS" | grep -qiE '(^|,)(README|AGENTS|[^,]+[.]md)(,|\$)'; then
                touch "\$REPO_ROOT/graphify-out/.graphify_stale" 2>/dev/null || true
            fi
            # Rebuild semantico debounced (comunidades + god nodes + report). Multi-LLM:
            #   1) GRAPHIFY_SEMANTIC_BACKEND si esta seteado (gemini|openai|claude-cli|...);
            #   2) si hay API key configurada (Gemini/OpenAI/Anthropic/DeepSeek/Kimi) o
            #      Ollama/Bedrock, deja que graphify auto-detecte el backend;
            #   3) si no hay ninguna, cae al CLI de Claude Code (claude-cli, sin API key);
            #   4) si nada de eso existe, salta (queda .graphify_stale para /graphify manual).
            # Debounced: solo si pasaron > GRAPHIFY_SEMANTIC_DEBOUNCE seg (default 1800=30min).
            SEM_BACKEND="\${GRAPHIFY_SEMANTIC_BACKEND:-}"
            if [ -z "\$SEM_BACKEND" ]; then
                if [ -n "\${GEMINI_API_KEY:-}\${GOOGLE_API_KEY:-}\${OPENAI_API_KEY:-}\${ANTHROPIC_API_KEY:-}\${DEEPSEEK_API_KEY:-}\${MOONSHOT_API_KEY:-}\${OLLAMA_BASE_URL:-}\${AWS_PROFILE:-}\${AWS_REGION:-}\${AWS_DEFAULT_REGION:-}" ]; then
                    SEM_BACKEND="auto"
                elif command -v claude >/dev/null 2>&1; then
                    SEM_BACKEND="claude-cli"
                fi
            fi
            if [ -n "\$SEM_BACKEND" ]; then
                SEM_DEBOUNCE=\${GRAPHIFY_SEMANTIC_DEBOUNCE:-1800}
                SEM_STAMP="\$REPO_ROOT/graphify-out/.last_semantic"
                sem_now=\$(date +%s); sem_last=0
                [ -f "\$SEM_STAMP" ] && sem_last=\$(stat -c %Y "\$SEM_STAMP" 2>/dev/null || stat -f %m "\$SEM_STAMP" 2>/dev/null || echo 0)
                [ -z "\$sem_last" ] && sem_last=0
                FORCE_REBUILD=0
                if printf '%s' "\$COMMIT_MSG_BODY" | grep -qF "[rebuild graph]"; then
                    FORCE_REBUILD=1
                fi
                if [ \$(( sem_now - sem_last )) -ge "\$SEM_DEBOUNCE" ] || [ "\$FORCE_REBUILD" -eq 1 ]; then
                    [ "\$SEM_BACKEND" = "auto" ] && SEM_BARG="" || SEM_BARG="--backend=\$SEM_BACKEND"
                    echo "Ejecutando rebuild semantico (backend: \$SEM_BACKEND, force: \$FORCE_REBUILD)..."
                    if graphify cluster-only "\$REPO_ROOT" \$SEM_BARG --no-viz; then
                        touch "\$SEM_STAMP" 2>/dev/null || true
                        rm -f "\$REPO_ROOT/graphify-out/.graphify_stale" 2>/dev/null || true
                    else
                        echo "Error en rebuild semantico."
                    fi
                else
                    echo "Skip rebuild semantico (debounce activo)."
                fi
            fi
            sh "\$HARNESS_DIR/harness_cli" graph vincular-grafo || true
            echo "Actualizacion en segundo plano finalizada."
        ) &
    fi
fi
HOOKEOF
    fi
    chmod +x "$POST_COMMIT"

    if ! grep -q "harness-managed-hook v4" "$COMMIT_MSG"; then
        tmp_hook="${COMMIT_MSG}.tmp"
        if [ -f "$COMMIT_MSG" ]; then
            sed '/harness-managed-hook/,$d' "$COMMIT_MSG" > "$tmp_hook" && mv "$tmp_hook" "$COMMIT_MSG"
        fi
        cat >> "$COMMIT_MSG" <<'CMEOF'

# harness-managed-hook v4
set -u
msg_file="${1:?commit message file missing}"
tmp="${msg_file}.harness.$$"
sed -E '/^Co-Authored-By:.*([Cc]laude|[Cc]odex|[Gg]emini|[Gg]rok|[Aa]ntigravity|[Oo]pen[Aa][Ii]|[Aa]nthropic|[Gg]oogle|[Xx][Aa][Ii]|[Aa][Ii])/d; /^Generated with .*([Cc]laude|[Cc]odex|[Gg]emini|[Gg]rok|[Aa]ntigravity|[Oo]pen[Aa][Ii]|[Aa]nthropic|[Gg]oogle|[Xx][Aa][Ii]|[Aa][Ii])/d' "$msg_file" > "$tmp" && mv "$tmp" "$msg_file"
rm -f "$tmp" "$msg_file.bak"
CMEOF
    fi
    chmod +x "$COMMIT_MSG"
    echo "   -> [Ok] $REPO_DIR conectado"
done

sh "$HARNESS_DIR/harness_cli" graph descubrir

if [ -f "$REPO_ROOT/graphify-out/graph.json" ]; then
    sh "$HARNESS_DIR/harness_cli" graph vincular-grafo || true
    if [ -f "$REPO_ROOT/graphify-out/.graphify_stale" ]; then
        echo "[graphify] Grafo desactualizado: corre '/graphify --update' y luego borra graphify-out/.graphify_stale."
    else
        echo "[graphify] Grafo de conocimiento al dia."
    fi
else
    echo "[graphify] Sin graphify-out/graph.json. Primera construccion manual: /graphify"
fi

echo ""
sh "$HARNESS_DIR/harness_cli" graph mapa
echo ""
echo "== [Ok] Harness listo =="
