#!/bin/bash
# Harness Process - instalador canonico (best-of).
# Unifica las variantes previas (setup basico + improved) en un solo instalador:
#   - Memoria hub compartida (graph_memory.py) con mapa/impacto/vincular/
#     desmarcar/sync_git/vincular-grafo + registrar/consultar.
#   - Integracion graphify (estructura automatica, rebuild semantico, hub) con el
#     comando /graphify nativo desplegado por agente (Claude/Codex/Gemini/Antigravity).
#   - Superficies y hooks multi-LLM auto-instalados (Claude, Codex, Gemini,
#     Grok, Antigravity, generica) sin flag --target.
#   - Capa opcional de subagentes (lider/implementer/reviewer, harness.py).
#   - Respaldos *.bak.* archivados bajo bkp/ (HARNESS_BKP_DIR para overridear).
set -Eeuo pipefail
IFS=$'\n\t'

# Subagentes, graphify y Antigravity quedan activos por defecto.
INSTALL_GRAPHIFY=1
# Despliega el comando /graphify nativo por agente (Claude/Codex/Gemini/Antigravity)
# via `graphify install --platform <agente>`. Asi el rebuild semantico deja de ser
# exclusivo de Claude. Se instala a nivel usuario (HOME); usa --no-graphify-skills
# para asegurar solo el binario sin tocar la config global de cada agente.
INSTALL_GRAPHIFY_SKILLS=1
INSTALL_ANTIGRAVITY=1
WITH_SUBAGENTS=1
FORCE=0
# Layout: 'subdir' (DEFAULT) = el arnes vive en una subcarpeta y orquesta el
# directorio PADRE; las superficies LLM se escriben en el padre y los scripts
# resuelven el padre como raiz multi-repo. 'root' (--root) = el arnes vive EN la
# raiz multi-repo, hermano de los microservicios.
LAYOUT=subdir

usage() {
    cat <<'USAGE'
Uso: ./setup_harness.sh [opciones]

Opciones:
  --no-subagents       Omite la capa lider/implementer/reviewer (se instala por defecto).
  --no-graphify        No asegura graphify (por defecto se asegura, instalandolo si falta).
  --no-graphify-skills No despliega el comando /graphify por agente (Claude/Codex/
                       Gemini/Antigravity); deja solo el CLI graphify.
  --with-subagents     Ya es el default; se mantiene por compatibilidad.
  --install-graphify   Ya es el default; se mantiene por compatibilidad.
  --install-antigravity Ya es el default; asegura Antigravity CLI si falta.
  --no-antigravity     No instala Antigravity CLI.
  --subdir             (DEFAULT) El arnes vive en esta subcarpeta y orquesta el
                       directorio PADRE: escribe superficies multi-LLM en el
                       padre (raiz multi-repo) y mantiene aqui los scripts.
                       Correlo desde dentro de la subcarpeta del arnes.
  --root               El arnes vive EN la raiz multi-repo (hermano de los
                       microservicios). Layout clasico; desactiva el default.
  --force              Sobrescribe archivos sin crear backup.
  -h, --help           Muestra esta ayuda.

El layout por defecto es subdir (usa --root para el layout clasico). Por defecto
instala todas las superficies y hooks LLM conocidos (sin --target), la capa de
subagentes, asegura graphify y Antigravity CLI (los instala si faltan).
Respalda archivos existentes bajo bkp/ (configurable con HARNESS_BKP_DIR).
USAGE
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --with-subagents) WITH_SUBAGENTS=1 ;;
        --install-graphify) INSTALL_GRAPHIFY=1 ;;
        --install-antigravity) INSTALL_ANTIGRAVITY=1 ;;
        --no-subagents) WITH_SUBAGENTS=0 ;;
        --no-graphify) INSTALL_GRAPHIFY=0 ;;
        --no-graphify-skills) INSTALL_GRAPHIFY_SKILLS=0 ;;
        --no-antigravity) INSTALL_ANTIGRAVITY=0 ;;
        --subdir) LAYOUT=subdir ;;
        --root) LAYOUT=root ;;
        --force) FORCE=1 ;;
        -h|--help) usage; exit 0 ;;
        *)
            echo "[!] Opcion desconocida: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

timestamp() {
    date +%Y%m%d%H%M%S
}

BKP_DIR="bkp"

# Calcula la ruta del backup dentro de bkp/, preservando la estructura de
# subcarpetas del archivo original para evitar colisiones de nombres.
backup_path() {
    target="$1"
    rel="${target#./}"
    dest="$BKP_DIR/${rel}.bak.$(timestamp)"
    mkdir -p "$(dirname "$dest")"
    echo "$dest"
}

backup_file() {
    target="$1"
    if [ "$FORCE" -eq 0 ] && [ -e "$target" ]; then
        backup="$(backup_path "$target")"
        cp -p "$target" "$backup"
        echo "[Harness] Backup creado: $backup"
    fi
}

archive_legacy_file() {
    target="$1"
    reason="$2"
    if [ -f "$target" ]; then
        backup="$(backup_path "$target")"
        mv "$target" "$backup"
        echo "[Harness] $reason; archivado como $backup"
    fi
}

write_file_notice() {
    echo "   -> $1"
}

write_agent_surface() {
    target="$1"
    mkdir -p "$(dirname "$target")"
    cat <<'SURFACE_EOF' > "$target"
# Harness Process

Estas operando en la raiz de un arnes multi-repo compatible con Claude Code,
Codex, Gemini, Grok, Antigravity y otros agentes CLI. No elijas proveedor ni
target: sigue este mismo protocolo desde la raiz del proyecto.

## Arranque automatico

El instalador deja hooks nativos cuando la herramienta los soporta:

- Claude Code: `.claude/settings.json`
- Codex: `.codex/hooks.json` (revisa y confia con `/hooks` si lo pide)
- Gemini CLI: `.gemini/settings.json`
- Grok Build: `.grok/hooks/` (confia con `/hooks-trust` si lo pide)
- Antigravity CLI: sin hooks nativos conocidos; usa `bin/harness-antigravity`
  para inicializar el arnes antes de abrir el agente.

Tambien quedan launchers en `bin/` para arrancar desde la raiz y ejecutar
`init.sh` antes de abrir el agente:

```bash
bin/harness-claude
bin/harness-codex
bin/harness-gemini
bin/harness-grok
bin/harness-antigravity
```

Si tu agente no ejecuta hooks o no ves el mapa del hub, corre manualmente:

```bash
bash "__HREL__init.sh"
bash "__HREL__harness_status.sh"
```

Antes de tocar codigo, arquitectura o dependencias entre servicios, en ESTE
orden:

1. Revisa el mapa del hub:
   `python3 "__HREL__graph_memory.py" mapa`
2. Si vas a modificar un servicio, revisa su radio de impacto:
   `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
3. Si existe `graphify-out/graph.json`, consulta el grafo antes de leer a
   ciegas: `graphify query "<pregunta de la task>"`
4. Trabaja dentro del microservicio correspondiente; no programes en la raiz
   multi-repo salvo que la tarea sea del arnes.
5. Valida los servicios afectados. Los documentos durables (plan, investigacion,
   evidencia de implementacion/review) se guardan en `docs/` de la RAIZ del
   proyecto, no en chat ni en `__HREL__progress/` (solo estado vivo).
6. Al cerrar la feature usa
   `python3 "__HREL__harness.py" close --feature <id> --status <estado>`: mueve la
   task y las memorias juntas (registra el hub y refresca graphify). Luego corre
   `bash "__HREL__harness_check.sh"`.

Los comandos anteriores usan rutas relativas a la raiz multi-repo. Si estas
dentro de un microservicio, vuelve a la raiz o usa la ruta absoluta del arnes.

## Hub de memoria y graphify

Son sistemas separados:

- **Hub** (`~/.harness-hub/graph_db.json`, configurable con `HARNESS_HUB`):
  rastrea proyectos, microservicios, commits y dependencias entre servicios.
  Los ids se namespacean como `<proyecto>/<servicio>`.
- **graphify** (`graphify-out/`): grafo del contenido del codigo. Para preguntas
  de arquitectura o "como funciona X", consulta primero `graphify query`.

Servicios transversales:

- Ver dependencias de todos los proyectos:
  `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
- Declarar una dependencia:
  `python3 "__HREL__graph_memory.py" vincular --microservicio <consumidor> --destino <proyecto>/<servicio>`
- Marcar destino transversal:
  agrega `--transversal` al comando `vincular`.
- Quitar marca transversal:
  `python3 "__HREL__graph_memory.py" desmarcar --microservicio <servicio>`
- Registrar un avance de la feature activa (mueve hub + graphify + history.md +
  current.md de una vez):
  `python3 "__HREL__harness.py" advance --nota "<que avanzaste>"`
  El arnes ademas hace este checkpoint AUTOMATICAMENTE al cierre de cada turno
  (hook multi-LLM en Claude/Codex/Gemini/Grok) si el plan o la evidencia
  cambiaron; `advance` queda para dejar la nota explicita.
- Registrar progreso (evento de bajo nivel en el hub):
  `python3 "__HREL__graph_memory.py" registrar --accion <accion> --estado <estado> --artefacto <nombre> [--meta ...]`
- Consultar progreso:
  `python3 "__HREL__graph_memory.py" consultar --artefacto <nombre> [--microservicio <servicio>]`

## graphify

- El instalador deja el comando `/graphify` nativo en cada agente soportado
  (Claude, Codex, Gemini, Antigravity) via `graphify install --platform <agente>`.
  Grok no tiene plataforma propia: usa el CLI (`graphify update .`,
  `graphify query "..."`) o lee la skill de Claude si tu build lo permite.
- El hook `post-commit` de cada microservicio, en segundo plano y cuando hay grafo
  existente: (1) corre `graphify update` (estructural, sin LLM, agnostico) y (2)
  relanza el rebuild semantico (comunidades + god nodes + report) con `graphify
  cluster-only --no-viz`. Backend multi-LLM: usa `GRAPHIFY_SEMANTIC_BACKEND` si esta
  seteado; si no, auto-detecta segun la API key configurada (Gemini/OpenAI/Anthropic/
  DeepSeek/Kimi, u Ollama/Bedrock); si no hay ninguna, cae al CLI de Claude Code
  (`claude-cli`, login Pro/Max sin API key); y si tampoco, salta. El paso (2) es
  *debounced*: solo si pasaron >30 min (configurable con `GRAPHIFY_SEMANTIC_DEBOUNCE`,
  en segundos) desde el ultimo, en `.last_semantic`. Al lograrlo borra `.graphify_stale`.
- Asi comunidades/god-nodes/report se refrescan solos. El rebuild semantico
  *completo* con descripciones por nodo (mas caro) y la viz `graph.html` siguen en
  `/graphify` (o `/graphify --update`); el hook usa `--no-viz` para no regenerarla.
- Manual sin comando nativo: `graphify update .` (estructural) o `graphify
  cluster-only . --backend=claude-cli` (semantico headless). `graphify query "..."`
  consulta el grafo en cualquier agente. Tras un rebuild manual refresca el hub:
  `python3 "__HREL__graph_memory.py" vincular-grafo`

## Commits

- Prohibido incluir firmas o trailers de IA (`Co-Authored-By`, `Generated with
  Claude/Codex/Gemini/Grok/Antigravity/OpenAI/Anthropic/xAI`, etc.). El hook
  `commit-msg` intenta limpiarlos automaticamente.
- Usa Conventional Commits desde terminal.
- Commitea cada microservicio afectado antes de cerrar la task, salvo decision
  explicita de bloqueo documentada en `__HREL__progress/`.
- La politica de cierre se controla con `HARNESS_COMMIT_GUARD_MODE=block|warn|off`.

## Mapa de agentes

Mapa completo y diagrama: `__HREL__roles/README.md`. Tres roles leidos como
mapa progresivo (lee solo lo necesario para la tarea actual):

1. **Lider** (`__HREL__roles/leader.md`): fija alcance e impacto y escribe el
   plan en `docs/` de la raiz. No implementa codigo.
2. **Implementer** (`__HREL__roles/implementer.md`): modifica una unidad
   concreta y deja evidencia durable en `docs/` de la raiz.
3. **Reviewer** (`__HREL__roles/reviewer.md`): verifica tests, impacto,
   checkpoints y estado Git; veredicto en `docs/` de la raiz.

Orquestacion (mismos roles, formato nativo por herramienta):

- **Claude Code**: subagentes nativos en `.claude/agents/`.
- **Codex CLI**: subagentes nativos en `.codex/agents/*.toml`.
- **Gemini CLI**: subagentes nativos en `.gemini/agents/`.
- **Grok Build**: lee `.claude/agents/` por compatibilidad con Claude Code.
- **Antigravity y otros**: aplica `__HREL__roles/*.md` como fases secuenciales.

Detalle por herramienta (formatos, modelos, effort): `__HREL__roles/README.md`.

Archivos principales:

- `__HREL__CHECKPOINTS.md`: criterios de cierre.
- `__HREL__feature_list.json`: backlog ejecutable.
- `docs/` (RAIZ del proyecto): planes, investigaciones y evidencia durable
  (`plan-feature-<f>.md`, `impl-<f>.md`, `review-<f>.md`), junto a los docs del
  equipo.
- `__HREL__progress/current.md`: estado vivo de la tarea (apunta al plan).
- `__HREL__progress/history.md`: bitacora append-only.
- `__HREL__docs/architecture.md`: mapa de arquitectura.
- `__HREL__docs/conventions.md`: convenciones del equipo.
- `__HREL__docs/verification.md`: comandos de validacion.

Los documentos durables (plan, investigacion, evidencia) se escriben en `docs/`
de la raiz; `__HREL__progress/` guarda solo el estado vivo. Una respuesta corta
en chat no reemplaza evidencia persistida.
SURFACE_EOF

    surface_tmp="$target.harness.tmp"
    sed -e "s|__HREL__|$HREL|g" "$target" > "$surface_tmp" && mv "$surface_tmp" "$target"
    write_file_notice "$(basename "$target") ($SURFACE_DIR)"
}

harness_rel_without_slash() {
    printf '%s' "${HREL%/}"
}

write_harness_hook_runtime() {
    mkdir -p "$SURFACE_DIR/bin"
    cat <<'HOOK_RUNTIME_EOF' > "$SURFACE_DIR/bin/harness-hook"
#!/bin/bash
set -Eeuo pipefail

MODE="${1:-plain}"   # plain | gemini-json | codex-json
EVENT="${2:-${GROK_HOOK_EVENT:-unknown}}"
ROOT="${HARNESS_REPO_ROOT:-${GROK_WORKSPACE_ROOT:-}}"
if [ -z "$ROOT" ]; then
    ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
fi

HARNESS_REL="__HREL_NOSLASH__"
if [ -n "$HARNESS_REL" ]; then
    HARNESS_DIR="$ROOT/$HARNESS_REL"
else
    HARNESS_DIR="$ROOT"
fi

run_session_start() {
    HARNESS_REPO_ROOT="$ROOT" bash "$HARNESS_DIR/init.sh"
    HARNESS_REPO_ROOT="$ROOT" bash "$HARNESS_DIR/harness_status.sh"
}

run_post_tool() {
    # Aviso (no bloqueante) si no hay feature activa: empuja a registrar el trabajo
    # (harness.py start) para que el ciclo plan+autocheck no se salte. A stderr.
    HARNESS_REPO_ROOT="$ROOT" python3 "$HARNESS_DIR/harness.py" nudge || true
    HARNESS_REPO_ROOT="$ROOT" bash "$HARNESS_DIR/harness_status.sh" --brief
}

run_stop() {
    # Checkpoint automatico de avance (multi-LLM: corre al cierre de turno en
    # Claude/Codex/Gemini/Grok). Si el plan o la evidencia cambiaron, sincroniza
    # hub + graphify. No-fatal y sin tocar el exit code: el gate sigue siendo
    # harness_check (que manda en el resultado del hook).
    HARNESS_REPO_ROOT="$ROOT" python3 "$HARNESS_DIR/harness.py" autocheck 1>&2 || true
    HARNESS_REPO_ROOT="$ROOT" bash "$HARNESS_DIR/harness_check.sh"
}

run_event() {
    case "$EVENT" in
        session-start|SessionStart|InstructionsLoaded|BeforeAgent)
            run_session_start
            ;;
        post-tool|PostToolUse|AfterTool|Tool)
            run_post_tool
            ;;
        stop|Stop|AfterAgent|SessionEnd|SessionStop)
            run_stop
            ;;
        *)
            return 0
            ;;
    esac
}

if [ "$MODE" = "gemini-json" ]; then
    if run_event >&2; then
        case "$EVENT" in
            session-start|SessionStart)
                printf '{"systemMessage":"Harness inicializado.","suppressOutput":true}\n'
                ;;
            *)
                printf '{"suppressOutput":true}\n'
                ;;
        esac
    else
        case "$EVENT" in
            stop|AfterAgent)
                printf '{"continue":false,"stopReason":"Harness check fallo; corrige el estado del repo antes de continuar.","systemMessage":"Harness check fallo; revisa la salida del hook."}\n'
                exit 0
                ;;
            *)
                printf '{"systemMessage":"Harness hook fallo; revisa la salida del hook.","suppressOutput":false}\n'
                exit 1
                ;;
        esac
    fi
elif [ "$MODE" = "codex-json" ]; then
    # Codex parsea el stdout del hook como JSON: cualquier texto plano que empiece
    # con '{' o '[' (p.ej. lineas "[Harness] ...") rompe con "invalid ... JSON
    # output". Mandamos la salida legible a stderr y dejamos stdout vacio, que es
    # un no-op valido para todos los eventos. Solo el gate Stop emite JSON.
    if run_event >&2; then
        exit 0
    else
        case "$EVENT" in
            stop|Stop|AfterAgent|SessionEnd|SessionStop)
                printf '{"decision":"block","reason":"Harness check fallo; corrige el estado del repo antes de cerrar."}\n'
                exit 0
                ;;
            *)
                echo "Harness hook fallo; revisa la salida del hook." >&2
                exit 0
                ;;
        esac
    fi
else
    run_event
fi
HOOK_RUNTIME_EOF

    hook_runtime_tmp="$SURFACE_DIR/bin/harness-hook.tmp"
    sed "s|__HREL_NOSLASH__|$(harness_rel_without_slash)|g" \
        "$SURFACE_DIR/bin/harness-hook" > "$hook_runtime_tmp" \
        && mv "$hook_runtime_tmp" "$SURFACE_DIR/bin/harness-hook"
    chmod +x "$SURFACE_DIR/bin/harness-hook"
    write_file_notice "bin/harness-hook ($SURFACE_DIR)"
}

write_codex_hooks() {
    mkdir -p "$SURFACE_DIR/.codex"
    cat <<'CODEX_HOOKS_EOF' > "$SURFACE_DIR/.codex/hooks.json"
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear|compact",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"bin/harness-hook\" codex-json session-start",
            "timeout": 120,
            "statusMessage": "Inicializando Harness"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Bash|Edit|Write|apply_patch",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"bin/harness-hook\" codex-json post-tool",
            "timeout": 30,
            "statusMessage": "Actualizando Harness"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"bin/harness-hook\" codex-json stop",
            "timeout": 120,
            "statusMessage": "Verificando Harness"
          }
        ]
      }
    ]
  }
}
CODEX_HOOKS_EOF
    write_file_notice ".codex/hooks.json ($SURFACE_DIR)"
}

write_gemini_hooks() {
    mkdir -p "$SURFACE_DIR/.gemini/commands/harness"
    cat <<'GEMINI_SETTINGS_EOF' > "$SURFACE_DIR/.gemini/settings.json"
{
  "hooksConfig": {
    "enabled": true,
    "notifications": true
  },
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "name": "harness-session-start",
            "description": "Inicializa el Harness Process y muestra el mapa del hub.",
            "command": "bash \"bin/harness-hook\" gemini-json session-start",
            "timeout": 120000
          }
        ]
      }
    ],
    "AfterTool": [
      {
        "hooks": [
          {
            "type": "command",
            "name": "harness-status",
            "description": "Muestra estado breve del harness despues de herramientas.",
            "command": "bash \"bin/harness-hook\" gemini-json post-tool",
            "timeout": 30000
          }
        ]
      }
    ],
    "AfterAgent": [
      {
        "hooks": [
          {
            "type": "command",
            "name": "harness-check",
            "description": "Verifica checkpoints y estado Git al terminar el turno.",
            "command": "bash \"bin/harness-hook\" gemini-json stop",
            "timeout": 120000
          }
        ]
      }
    ]
  }
}
GEMINI_SETTINGS_EOF

    cat <<'GEMINI_CHECK_EOF' > "$SURFACE_DIR/.gemini/commands/harness/check.toml"
description = "Ejecuta el cierre del Harness Process."
prompt = """
Ejecuta este comando y corrige cualquier bloqueo antes de cerrar:

```bash
!{bash bin/harness-hook plain stop}
```
"""
GEMINI_CHECK_EOF

    cat <<'GEMINI_STATUS_EOF' > "$SURFACE_DIR/.gemini/commands/harness/status.toml"
description = "Muestra el estado actual del Harness Process."
prompt = """
Resume el estado del Harness Process usando esta salida:

```text
!{bash bin/harness-hook plain session-start}
```
"""
GEMINI_STATUS_EOF

    write_file_notice ".gemini/settings.json / commands ($SURFACE_DIR)"
}

write_grok_hooks() {
    mkdir -p "$SURFACE_DIR/.grok/hooks"
    cat <<'GROK_HOOK_EOF' > "$SURFACE_DIR/.grok/hooks/harness.sh"
#!/bin/bash
set -Eeuo pipefail

ROOT="${GROK_WORKSPACE_ROOT:-}"
if [ -z "$ROOT" ]; then
    ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
fi
export HARNESS_REPO_ROOT="$ROOT"

case "${GROK_HOOK_EVENT:-}" in
    SessionStart|InstructionsLoaded|BeforeAgent)
        exec "$ROOT/bin/harness-hook" plain session-start
        ;;
    PostToolUse|AfterTool|Tool)
        exec "$ROOT/bin/harness-hook" plain post-tool
        ;;
    Stop|AfterAgent|SessionEnd|SessionStop)
        exec "$ROOT/bin/harness-hook" plain stop
        ;;
    *)
        exit 0
        ;;
esac
GROK_HOOK_EOF
    chmod +x "$SURFACE_DIR/.grok/hooks/harness.sh"

    cat <<'GROK_MD_EOF' > "$SURFACE_DIR/.grok/GROK.md"
# Harness Process para Grok

Este proyecto instala hooks del Harness Process en `.grok/hooks/`.
Si Grok solicita confianza, abre `/hooks` o ejecuta `/hooks-trust`.
Tambien puedes iniciar con `bin/harness-grok`.
GROK_MD_EOF
    write_file_notice ".grok/hooks/harness.sh / .grok/GROK.md ($SURFACE_DIR)"
}

write_launchers() {
    mkdir -p "$SURFACE_DIR/bin"
    for agent in claude codex gemini grok antigravity; do
        launcher="$SURFACE_DIR/bin/harness-$agent"
        cat <<'LAUNCHER_EOF' > "$launcher"
#!/bin/bash
set -Eeuo pipefail

AGENT="__AGENT__"
ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
HARNESS_REL="__HREL_NOSLASH__"
if [ -n "$HARNESS_REL" ]; then
    HARNESS_DIR="$ROOT/$HARNESS_REL"
else
    HARNESS_DIR="$ROOT"
fi

export HARNESS_REPO_ROOT="$ROOT"
export CLAUDE_PROJECT_DIR="$ROOT"
export CODEX_PROJECT_DIR="$ROOT"
export GEMINI_PROJECT_DIR="$ROOT"
export GROK_PROJECT_DIR="$ROOT"
export GROK_WORKSPACE_ROOT="$ROOT"
export ANTIGRAVITY_PROJECT_DIR="$ROOT"

bash "$HARNESS_DIR/init.sh"
cd "$ROOT"

if ! command -v "$AGENT" >/dev/null 2>&1; then
    echo "[Harness] No encontre el comando '$AGENT' en PATH." >&2
    exit 127
fi

exec "$AGENT" "$@"
LAUNCHER_EOF
        launcher_tmp="$launcher.tmp"
        sed -e "s|__AGENT__|$agent|g" \
            -e "s|__HREL_NOSLASH__|$(harness_rel_without_slash)|g" \
            "$launcher" > "$launcher_tmp" && mv "$launcher_tmp" "$launcher"
        chmod +x "$launcher"
    done
    write_file_notice "bin/harness-claude|codex|gemini|grok|antigravity ($SURFACE_DIR)"
}

ensure_antigravity_cli() {
    echo "Asegurando Antigravity CLI..."
    if command -v antigravity >/dev/null 2>&1; then
        echo "   -> antigravity ya esta disponible."
    elif [ "$INSTALL_ANTIGRAVITY" -eq 1 ]; then
        if ! command -v curl >/dev/null 2>&1; then
            echo "   -> aviso: curl no esta disponible; instala Antigravity manualmente."
            return 0
        fi

        set +e
        tmp_install=$(mktemp)
        if curl -fsSL https://antigravity.google/cli/install.sh -o "$tmp_install"; then
            bash "$tmp_install"
            status=$?
        else
            status=1
        fi
        rm -f "$tmp_install"
        set -e

        if [ "$status" -eq 0 ]; then
            echo "   -> Antigravity CLI instalado."
        else
            echo "   -> aviso: no se pudo instalar Antigravity CLI."
        fi
    else
        echo "   -> Antigravity CLI no instalado (--no-antigravity activo)."
    fi
}

# --- Resolucion de layout -----------------------------------------------------
# HARNESS_DIR : carpeta donde viven los scripts del arnes (= cwd del instalador).
# REPO_ROOT   : raiz multi-repo (donde estan los microservicios). En 'subdir' es
#               el padre; en 'root' es el propio HARNESS_DIR.
# SURFACE_DIR : donde van CLAUDE.md, AGENTS.md, GEMINI.md, LLM.md y
#               .claude/settings.json (= REPO_ROOT).
# HARNESS_EXEC: prefijo historico para superficies Claude (sin llaves).
# HOOK_BASE   : prefijo para las rutas en .claude/settings.json (con llaves).
# HREL        : prefijo relativo de archivos del arnes vistos desde REPO_ROOT.
HARNESS_DIR="$(pwd -P)"
if [ "$LAYOUT" = "subdir" ]; then
    REPO_ROOT="$(dirname "$HARNESS_DIR")"
    HARNESS_SUBDIR="$(basename "$HARNESS_DIR")"
    HARNESS_EXEC='$CLAUDE_PROJECT_DIR/'"$HARNESS_SUBDIR"
    HOOK_BASE='${CLAUDE_PROJECT_DIR}/'"$HARNESS_SUBDIR"
    HREL="$HARNESS_SUBDIR/"
else
    REPO_ROOT="$HARNESS_DIR"
    HARNESS_SUBDIR=""
    HARNESS_EXEC='$CLAUDE_PROJECT_DIR'
    HOOK_BASE='${CLAUDE_PROJECT_DIR}'
    HREL=""
fi
SURFACE_DIR="$REPO_ROOT"
PROJECT_NAME="${HARNESS_PROJECT:-$(basename "$REPO_ROOT")}"

echo "== Instalando Harness Process en: $HARNESS_DIR =="
echo "   proyecto:   $PROJECT_NAME"
echo "   layout:     $LAYOUT$([ "$LAYOUT" = "subdir" ] && echo " (raiz multi-repo: $REPO_ROOT)")"
echo "   superficies/hooks: Claude, Codex, Gemini, Grok, Antigravity, generica"
echo "   subagentes: $([ "$WITH_SUBAGENTS" -eq 1 ] && echo si || echo no)"
echo "   graphify:   $([ "$INSTALL_GRAPHIFY" -eq 1 ] && echo asegurar || echo no)"
echo "   /graphify por agente: $([ "$INSTALL_GRAPHIFY_SKILLS" -eq 1 ] && echo "Claude/Codex/Gemini/Antigravity" || echo no)"
echo "   antigravity:$([ "$INSTALL_ANTIGRAVITY" -eq 1 ] && echo " asegurar" || echo " no")"

if [ "$LAYOUT" = "subdir" ] && [ "$REPO_ROOT" = "$HARNESS_DIR" ]; then
    echo "[!] --subdir requiere correr el instalador DESDE la subcarpeta del arnes," >&2
    echo "    de modo que su padre sea la raiz multi-repo. Aborto." >&2
    exit 2
fi

mkdir -p .claude
[ "$WITH_SUBAGENTS" -eq 1 ] && mkdir -p roles docs progress
mkdir -p "$SURFACE_DIR/.claude" "$SURFACE_DIR/.codex" "$SURFACE_DIR/.gemini" "$SURFACE_DIR/.grok" "$SURFACE_DIR/bin"
# Los subagentes nativos de Claude Code se registran desde la raiz (SURFACE_DIR),
# no desde la subcarpeta del arnes; por eso viven junto a .claude/settings.json.
[ "$WITH_SUBAGENTS" -eq 1 ] && mkdir -p "$SURFACE_DIR/.claude/agents" "$SURFACE_DIR/.codex/agents" "$SURFACE_DIR/.gemini/agents"

# Marcador de layout: los scripts lo leen para resolver REPO_ROOT en runtime.
printf '%s\n' "$LAYOUT" > "$HARNESS_DIR/.harness_layout"

archive_legacy_file ".claudemd" ".claudemd es obsoleto; Claude Code lee CLAUDE.md"
archive_legacy_file "validate_aks.sh" "validate_aks.sh quedo obsoleto"
archive_legacy_file "$SURFACE_DIR/GROK.md" "GROK.md no se usa; Grok Build lee AGENTS.md/CLAUDE.md"
archive_legacy_file "$SURFACE_DIR/ANTIGRAVITY.md" "ANTIGRAVITY.md no se usa; Antigravity lee AGENTS.md/.agents/rules"

generated=(
    "graph_memory.py"
    "init.sh"
    "validate_ui.sh"
    "debug_ui.js"
    "commit_guard.sh"
    "harness_status.sh"
    "harness_check.sh"
    "harness.py"
)
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    generated+=(
        "CHECKPOINTS.md"
        "docs/architecture.md"
        "docs/conventions.md"
        "docs/verification.md"
        "roles/README.md"
        "roles/leader.md"
        "roles/implementer.md"
        "roles/reviewer.md"
    )
    # feature_list.json, progress/current.md e history.md NO se listan aqui: son
    # estado vivo (backlog, bitacora, tarea en curso), no plantillas. Se crean
    # solo si faltan (mas abajo) para no pisarlos al reinstalar.
fi

for f in "${generated[@]}"; do
    backup_file "$f"
done
# Las superficies LLM pueden vivir en el padre (subdir).
backup_file "$SURFACE_DIR/CLAUDE.md"
backup_file "$SURFACE_DIR/AGENTS.md"
backup_file "$SURFACE_DIR/GEMINI.md"
backup_file "$SURFACE_DIR/LLM.md"
backup_file "$SURFACE_DIR/.claude/settings.json"
backup_file "$SURFACE_DIR/.claude/agents/leader.md"
backup_file "$SURFACE_DIR/.claude/agents/implementer.md"
backup_file "$SURFACE_DIR/.claude/agents/reviewer.md"
backup_file "$SURFACE_DIR/.codex/agents/leader.toml"
backup_file "$SURFACE_DIR/.codex/agents/implementer.toml"
backup_file "$SURFACE_DIR/.codex/agents/reviewer.toml"
backup_file "$SURFACE_DIR/.gemini/agents/leader.md"
backup_file "$SURFACE_DIR/.gemini/agents/implementer.md"
backup_file "$SURFACE_DIR/.gemini/agents/reviewer.md"
backup_file "$SURFACE_DIR/.codex/hooks.json"
backup_file "$SURFACE_DIR/.gemini/settings.json"
backup_file "$SURFACE_DIR/.gemini/commands/harness/check.toml"
backup_file "$SURFACE_DIR/.gemini/commands/harness/status.toml"
backup_file "$SURFACE_DIR/.grok/hooks/harness.sh"
backup_file "$SURFACE_DIR/.grok/GROK.md"
backup_file "$SURFACE_DIR/bin/harness-hook"
backup_file "$SURFACE_DIR/bin/harness-claude"
backup_file "$SURFACE_DIR/bin/harness-codex"
backup_file "$SURFACE_DIR/bin/harness-gemini"
backup_file "$SURFACE_DIR/bin/harness-grok"
backup_file "$SURFACE_DIR/bin/harness-antigravity"

echo "Generando .claude/settings.json..."
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    cat <<SETTINGS_EOF > "$SURFACE_DIR/.claude/settings.json"
{
  "attribution": {
    "commit": "",
    "pr": ""
  },
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"$HOOK_BASE/init.sh\" && bash \"$HOOK_BASE/harness_status.sh\""
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"$HOOK_BASE/harness.py\" nudge || true; bash \"$HOOK_BASE/harness_status.sh\" --brief"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"$HOOK_BASE/harness.py\" autocheck >/dev/null 2>&1 || true; bash \"$HOOK_BASE/harness_check.sh\""
          }
        ]
      }
    ]
  }
}
SETTINGS_EOF
else
    cat <<SETTINGS_EOF > "$SURFACE_DIR/.claude/settings.json"
{
  "attribution": {
    "commit": "",
    "pr": ""
  },
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"$HOOK_BASE/init.sh\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"$HOOK_BASE/commit_guard.sh\""
          }
        ]
      }
    ]
  }
}
SETTINGS_EOF
fi
write_file_notice ".claude/settings.json ($SURFACE_DIR)"

echo "Generando graph_memory.py..."
cp "$HARNESS_DIR/templates/graph_memory.py" "graph_memory.py"
chmod +x graph_memory.py
write_file_notice "graph_memory.py"

echo "Generando init.sh..."
cp "$HARNESS_DIR/templates/init.sh" "init.sh"
chmod +x init.sh
write_file_notice "init.sh"

echo "Generando validate_ui.sh y debug_ui.js..."
cp "$HARNESS_DIR/templates/validate_ui.sh" "validate_ui.sh"
chmod +x validate_ui.sh

cp "$HARNESS_DIR/templates/debug_ui.js" "debug_ui.js"
write_file_notice "validate_ui.sh / debug_ui.js"

echo "Generando guardas y estado..."
cp "$HARNESS_DIR/templates/harness_status.sh" "harness_status.sh"
chmod +x harness_status.sh

cp "$HARNESS_DIR/templates/commit_guard.sh" "commit_guard.sh"
chmod +x commit_guard.sh

cp "$HARNESS_DIR/templates/harness_check.sh" "harness_check.sh"
chmod +x harness_check.sh
write_file_notice "harness_status.sh / commit_guard.sh / harness_check.sh"

echo "Generando harness.py..."
cp "$HARNESS_DIR/templates/harness.py" "harness.py"
chmod +x harness.py
write_file_notice "harness.py"

if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    echo "Generando capa de subagentes (mapa de agentes)..."

    # Sustituye el placeholder __HREL__ por el prefijo real del arnes, in-place.
    subst_hrel_inplace() {
        local f="$1" tmp
        tmp="$f.harness.tmp"
        sed -e "s|__HREL__|$HREL|g" "$f" > "$tmp" && mv "$tmp" "$f"
    }

    # Ensambla un subagente nativo de Claude Code = frontmatter YAML + cuerpo del
    # rol (fuente unica en roles/). Args: role name model effort tools description.
    # Se escribe en SURFACE_DIR/.claude/agents para que Claude Code lo registre
    # desde la raiz multi-repo (no desde la subcarpeta del arnes).
    build_claude_agent() {
        local role="$1" aname="$2" amodel="$3" aeffort="$4" atools="$5" adesc="$6"
        local out="$SURFACE_DIR/.claude/agents/$role.md"
        {
            printf -- '---\n'
            printf 'name: %s\n' "$aname"
            printf 'description: %s\n' "$adesc"
            printf 'tools: %s\n' "$atools"
            printf 'model: %s\n' "$amodel"
            printf 'effort: %s\n' "$aeffort"
            printf -- '---\n\n'
        } > "$out"
        cat "roles/$role.md" >> "$out"
        subst_hrel_inplace "$out"
    }

    # --- Fuente unica de roles (legible por cualquier CLI) ----------------------
    cp "$HARNESS_DIR/templates/roles/leader.md" "roles/leader.md"

    cp "$HARNESS_DIR/templates/roles/implementer.md" "roles/implementer.md"

    cp "$HARNESS_DIR/templates/roles/reviewer.md" "roles/reviewer.md"

    cp "$HARNESS_DIR/templates/roles/README.md" "roles/README.md"

    subst_hrel_inplace roles/leader.md
    subst_hrel_inplace roles/implementer.md
    subst_hrel_inplace roles/reviewer.md
    subst_hrel_inplace roles/README.md

    # Codex CLI: subagentes nativos en .codex/agents/*.toml (auto-registrados).
    # No hay allowlist de tools; la capacidad se acota con sandbox_mode.
    # Args: role sandbox_mode reasoning_effort description.
    build_codex_agent() {
        local role="$1" asandbox="$2" aeffort="$3" adesc="$4"
        local out="$SURFACE_DIR/.codex/agents/$role.toml"
        {
            printf 'name = "%s"\n' "$role"
            printf 'description = "%s"\n' "$adesc"
            printf 'sandbox_mode = "%s"\n' "$asandbox"
            printf 'model_reasoning_effort = "%s"\n' "$aeffort"
            printf 'developer_instructions = %s\n' "'''"
            cat "roles/$role.md"
            printf '%s\n' "'''"
        } > "$out"
    }

    # Gemini CLI: subagentes nativos en .gemini/agents/*.md (Markdown+frontmatter,
    # cuerpo = system prompt; auto-descubiertos). tools/model se omiten -> hereda
    # de la sesion (evita fijar nombres de tools/model que varian por version).
    # Args: role description.
    build_gemini_agent() {
        local role="$1" adesc="$2"
        local out="$SURFACE_DIR/.gemini/agents/$role.md"
        {
            printf -- '---\n'
            printf 'name: %s\n' "$role"
            printf 'description: %s\n' "$adesc"
            printf -- '---\n\n'
        } > "$out"
        cat "roles/$role.md" >> "$out"
    }

    # Descripciones compartidas por las tres superficies de subagentes nativos.
    desc_leader="Coordinador del harness. Usalo al INICIAR una tarea para fijar alcance, calcular impacto entre microservicios y producir el plan en docs/ de la raiz. No implementa codigo."
    desc_impl="Implementa UNA unidad concreta del plan del lider dentro del microservicio asignado y deja evidencia durable en docs/ de la raiz. Usalo para escribir o modificar codigo."
    desc_rev="Verifica tests, impacto, checkpoints y estado Git antes de cerrar una feature; escribe veredicto en docs/ de la raiz. Solo lectura; no implementa."

    # --- Claude Code: .claude/agents/*.md (frontmatter + cuerpo de rol) ----------
    build_claude_agent leader      leader      claude-opus-4-8   max "Read, Grep, Glob, Bash"              "$desc_leader"
    build_claude_agent implementer implementer claude-sonnet-4-6 max "Read, Edit, Write, Bash, Grep, Glob" "$desc_impl"
    build_claude_agent reviewer    reviewer    claude-opus-4-8   max "Read, Grep, Glob, Bash"              "$desc_rev"

    # --- Codex CLI: .codex/agents/*.toml (sandbox por rol; effort high = tope) ---
    build_codex_agent leader      read-only       high "$desc_leader"
    build_codex_agent implementer workspace-write high "$desc_impl"
    build_codex_agent reviewer    read-only       high "$desc_rev"

    # --- Gemini CLI: .gemini/agents/*.md (hereda tools/model de la sesion) -------
    build_gemini_agent leader      "$desc_leader"
    build_gemini_agent implementer "$desc_impl"
    build_gemini_agent reviewer    "$desc_rev"

    # Grok Build (xAI) lee .claude/agents/*.md por compatibilidad con Claude Code;
    # no requiere archivos propios. Antigravity crea subagentes en runtime (sin
    # archivo de definicion soportado): usa roles/*.md como fases secuenciales.

    cp "$HARNESS_DIR/templates/CHECKPOINTS.md" "CHECKPOINTS.md"

    # Backlog vivo: solo se siembra si falta. Un reinstall NO debe vaciar las
    # features ya cargadas.
    if [ ! -f feature_list.json ]; then
        cp "$HARNESS_DIR/templates/feature_list.json" "feature_list.json"
    fi

    # Estado vivo: solo se siembra si falta. Un reinstall NO debe pisar la tarea
    # en curso ni la bitacora ya escrita.
    if [ ! -f progress/current.md ]; then
        cp "$HARNESS_DIR/templates/progress/current.md" "progress/current.md"
    fi

    if [ ! -f progress/history.md ]; then
        cp "$HARNESS_DIR/templates/progress/history.md" "progress/history.md"
    fi

    cp "$HARNESS_DIR/templates/docs/architecture.md" "docs/architecture.md"

    cp "$HARNESS_DIR/templates/docs/conventions.md" "docs/conventions.md"

    cp "$HARNESS_DIR/templates/docs/verification.md" "docs/verification.md"

    write_file_notice "roles/ + .claude/agents + .codex/agents + .gemini/agents / CHECKPOINTS.md / feature_list.json / docs / progress"
fi

echo "Generando superficies multi-LLM..."
write_agent_surface "$SURFACE_DIR/CLAUDE.md"
write_agent_surface "$SURFACE_DIR/AGENTS.md"
write_agent_surface "$SURFACE_DIR/GEMINI.md"
write_agent_surface "$SURFACE_DIR/LLM.md"
# GROK.md / ANTIGRAVITY.md no se generan: Grok Build lee AGENTS.md/CLAUDE.md y
# Antigravity lee AGENTS.md/.agents/rules. Ambos toman el AGENTS.md de arriba.

echo "Generando hooks y launchers multi-LLM..."
write_harness_hook_runtime
write_codex_hooks
write_gemini_hooks
write_grok_hooks
write_launchers

chmod +x init.sh validate_ui.sh commit_guard.sh harness_status.sh harness_check.sh harness.py

echo "Asegurando graphify..."
if command -v graphify >/dev/null 2>&1; then
    echo "   -> graphify ya esta disponible."
elif [ "$INSTALL_GRAPHIFY" -eq 1 ]; then
    set +e
    if command -v uv >/dev/null 2>&1; then
        uv tool install --upgrade graphifyy >/dev/null 2>&1 \
            && echo "   -> graphify instalado via uv." \
            || echo "   -> aviso: no se pudo instalar via uv."
    elif command -v pipx >/dev/null 2>&1; then
        pipx install graphifyy >/dev/null 2>&1 \
            && echo "   -> graphify instalado via pipx." \
            || echo "   -> aviso: no se pudo instalar via pipx."
    else
        python3 -m pip install --user graphifyy >/dev/null 2>&1 \
            && echo "   -> graphify instalado via pip --user." \
            || echo "   -> aviso: instala manualmente graphifyy."
    fi
    set -e
else
    echo "   -> graphify no instalado (--no-graphify activo). Quita ese flag para asegurarlo."
fi

echo "Asegurando psycopg2 para el Hub en Postgres..."
if python3 -c "import psycopg2" >/dev/null 2>&1; then
    echo "   -> psycopg2 ya esta disponible."
else
    set +e
    python3 -m pip install --user psycopg2-binary >/dev/null 2>&1 \
        && echo "   -> psycopg2-binary instalado via pip --user." \
        || echo "   -> aviso: no se pudo instalar psycopg2-binary. Usa modo JSON fallback."
    set -e
fi

# Despliega el comando /graphify nativo en cada agente que graphify soporta, para
# que el rebuild semantico no sea exclusivo de Claude. Se corre desde un directorio
# AISLADO con scope global (HOME): las skills quedan en ~/.claude (Claude),
# ~/.agents (estandar compartido que tambien lee Gemini) y ~/.gemini/config
# (Antigravity); los archivos que `graphify install` escribe en el cwd (GEMINI.md,
# .gemini/settings.json) caen en el tmp descartable y NO pisan la superficie.
#
# NO instalamos --platform gemini a proposito: Gemini CLI lee la skill desde el
# estandar compartido ~/.agents/skills/ (que puebla --platform codex) ADEMAS de su
# ~/.gemini/skills/ nativo. Instalar en ambos la deja duplicada y Gemini avisa
# "Skill conflict: graphify from ~/.agents ... overriding ... ~/.gemini". Con solo
# codex, Gemini toma /graphify de ~/.agents/ y no hay conflicto.
if [ "$INSTALL_GRAPHIFY_SKILLS" -eq 1 ] && command -v graphify >/dev/null 2>&1; then
    echo "Desplegando el comando /graphify por agente..."
    gx_tmp="$(mktemp -d 2>/dev/null || echo "${TMPDIR:-/tmp}/harness-graphify.$$")"
    mkdir -p "$gx_tmp"
    for gx_plat in claude codex antigravity; do
        if ( cd "$gx_tmp" && graphify install --platform "$gx_plat" ) >/dev/null 2>&1; then
            echo "   -> /graphify disponible en $gx_plat."
        else
            echo "   -> aviso: no se pudo desplegar /graphify en $gx_plat."
        fi
    done
    rm -rf "$gx_tmp"
    echo "   -> Grok: sin plataforma propia; usa el CLI ('graphify update .' / 'graphify query')."
elif [ "$INSTALL_GRAPHIFY_SKILLS" -eq 0 ]; then
    echo "   -> Comando /graphify por agente omitido (--no-graphify-skills)."
fi

ensure_antigravity_cli

echo ""
echo "========================================================"
echo "Harness Process instalado exitosamente (layout: $LAYOUT)."
echo ""
echo "Superficies multi-LLM escritas en la raiz:"
echo "  $SURFACE_DIR/CLAUDE.md"
echo "  $SURFACE_DIR/AGENTS.md"
echo "  $SURFACE_DIR/GEMINI.md"
echo "  $SURFACE_DIR/LLM.md"
echo "  $SURFACE_DIR/.claude/settings.json (hooks automaticos para Claude Code)"
echo "  $SURFACE_DIR/.codex/hooks.json (hooks automaticos para Codex; confiar con /hooks)"
echo "  $SURFACE_DIR/.gemini/settings.json (hooks automaticos para Gemini CLI)"
echo "  $SURFACE_DIR/.grok/hooks/harness.sh (hooks automaticos para Grok; confiar con /hooks-trust)"
echo "  $SURFACE_DIR/bin/harness-claude|codex|gemini|grok|antigravity"
if [ "$LAYOUT" = "subdir" ]; then
    echo ""
    echo "Scripts del arnes en: $HARNESS_DIR"
    echo "IMPORTANTE: lanza tu agente DESDE la raiz ($REPO_ROOT) para que"
    echo "descubra la superficie correspondiente."
fi
echo ""
echo "Comandos utiles:"
echo "  bash ${HREL}init.sh"
echo "  bash ${HREL}harness_status.sh"
echo "  bash ${HREL}harness_check.sh"
echo "  python3 ${HREL}graph_memory.py mapa"
echo "  python3 ${HREL}harness.py status"
echo "  bin/harness-codex"
echo "  bin/harness-gemini"
echo "  bin/harness-grok"
echo "  bin/harness-antigravity"
echo "  /graphify           (comando nativo en Claude/Codex/Gemini/Antigravity)"
echo "  graphify query \"...\"  (CLI; funciona en cualquier agente, incl. Grok)"
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    echo ""
    echo "Modo subagentes activo:"
    echo "  Mapa de agentes:    ${HREL}roles/README.md"
    echo "  Subagentes nativos: .claude/agents/*.md, .codex/agents/*.toml, .gemini/agents/*.md"
    echo "  Grok Build:         lee .claude/agents/*.md (compat Claude Code)"
    echo "  Antigravity/otros:  ${HREL}roles/*.md como fases secuenciales"
    echo "  python3 ${HREL}harness.py add --name \"mi_feature\" --service \"$PROJECT_NAME/servicio\""
    echo "  python3 ${HREL}harness.py start --feature 1"
    echo "  python3 ${HREL}harness.py close --feature 1 --status done"
fi
echo "========================================================"
