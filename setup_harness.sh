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
- El hook `post-commit` de cada microservicio corre `graphify update` en segundo
  plano cuando hay grafo existente y marca `graphify-out/.graphify_stale` si
  detecta cambios que requieren rebuild semantico.
- Primera construccion o rebuild semantico: usa `/graphify` (o `/graphify
  --update`). Sin el comando nativo: `graphify update .` (estructural, sin LLM) o
  `graphify extract .` (headless AST + semantico). `graphify query "..."` consulta
  el grafo en cualquier agente.
- Despues del rebuild, borra `graphify-out/.graphify_stale` y refresca el hub:
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
        curl -fsSL https://antigravity.google/cli/install.sh | bash
        status=$?
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
    cat <<'SETTINGS_EOF' > "$SURFACE_DIR/.claude/settings.json"
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
            "command": "bash \"${CLAUDE_PROJECT_DIR}/init.sh\" && bash \"${CLAUDE_PROJECT_DIR}/harness_status.sh\""
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
            "command": "python3 \"${CLAUDE_PROJECT_DIR}/harness.py\" nudge || true; bash \"${CLAUDE_PROJECT_DIR}/harness_status.sh\" --brief"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"${CLAUDE_PROJECT_DIR}/harness.py\" autocheck >/dev/null 2>&1 || true; bash \"${CLAUDE_PROJECT_DIR}/harness_check.sh\""
          }
        ]
      }
    ]
  }
}
SETTINGS_EOF
else
    cat <<'SETTINGS_EOF' > "$SURFACE_DIR/.claude/settings.json"
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
            "command": "bash \"${CLAUDE_PROJECT_DIR}/init.sh\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR}/commit_guard.sh\""
          }
        ]
      }
    ]
  }
}
SETTINGS_EOF
fi
# Inserta el prefijo del subdir en las rutas de hooks segun el layout
# (en 'root' HOOK_BASE == ${CLAUDE_PROJECT_DIR}, asi que es un no-op).
settings_tmp="$SURFACE_DIR/.claude/settings.json.harness.tmp"
sed "s|\${CLAUDE_PROJECT_DIR}|$HOOK_BASE|g" \
    "$SURFACE_DIR/.claude/settings.json" > "$settings_tmp" \
    && mv "$settings_tmp" "$SURFACE_DIR/.claude/settings.json"
write_file_notice ".claude/settings.json ($SURFACE_DIR)"

echo "Generando graph_memory.py..."
cat <<'GM_PY_EOF' > graph_memory.py
#!/usr/bin/env python3
"""Memoria distribuida del Harness Process.

Mantiene un hub compartido por defecto en ~/.harness-hub/graph_db.json.
Ids de microservicio: <proyecto>/<servicio>.
"""
import argparse
import hashlib
import json
import os
import subprocess
from contextlib import contextmanager

try:
    import fcntl
    HAVE_FCNTL = True
except ImportError:
    HAVE_FCNTL = False

BASE_DIR = os.path.dirname(os.path.abspath(__file__))


def _repo_root():
    """Raiz multi-repo: en layout 'subdir' es el padre de BASE_DIR."""
    env = os.environ.get("HARNESS_REPO_ROOT")
    if env:
        return os.path.abspath(env)
    try:
        with open(os.path.join(BASE_DIR, ".harness_layout"), encoding="utf-8") as fh:
            if fh.read().strip() == "subdir":
                return os.path.dirname(BASE_DIR)
    except OSError:
        pass
    return BASE_DIR


REPO_ROOT = _repo_root()
PROJECT = os.environ.get("HARNESS_PROJECT") or os.path.basename(REPO_ROOT)
HUB_DIR = os.environ.get("HARNESS_HUB") or os.path.join(os.path.expanduser("~"), ".harness-hub")
GRAPH_DB_FILE = os.path.join(HUB_DIR, "graph_db.json")
PROGRESS_DIR = os.path.join(HUB_DIR, "progress")
LOCK_FILE = os.path.join(HUB_DIR, ".lock")


@contextmanager
def hub_lock():
    os.makedirs(HUB_DIR, exist_ok=True)
    with open(LOCK_FILE, "w", encoding="utf-8") as f:
        if HAVE_FCNTL:
            fcntl.flock(f.fileno(), fcntl.LOCK_EX)
        try:
            yield
        finally:
            if HAVE_FCNTL:
                fcntl.flock(f.fileno(), fcntl.LOCK_UN)


class GraphStore:
    def __init__(self):
        self.nodes = {}
        self.edges = []

    def add_node(self, label, props):
        nid = props["_id"]
        node = self.nodes.get(nid, {})
        node.update(props)
        node["_label"] = label
        self.nodes[nid] = node

    def add_edge(self, etype, source, target, **props):
        edge = {"type": etype, "source": source, "target": target}
        edge.update(props)
        if edge not in self.edges:
            self.edges.append(edge)

    def load(self, data):
        self.nodes = data.get("nodes", {})
        self.edges = data.get("edges", [])

    def to_dict(self):
        return {"nodes": self.nodes, "edges": self.edges}


def qualify(name):
    return name if "/" in name else f"{PROJECT}/{name}"


def artifact_id(path):
    safe = path.replace(os.sep, "__").replace("/", "__").replace(".", "_")
    digest = hashlib.sha1(path.encode("utf-8")).hexdigest()[:8]
    return f"{safe}_{digest}"


def is_repo_root(path):
    try:
        result = subprocess.run(
            ["git", "-C", path, "rev-parse", "--show-toplevel"],
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            check=False,
        )
        return result.returncode == 0 and os.path.realpath(result.stdout.strip()) == os.path.realpath(path)
    except OSError:
        return os.path.isdir(os.path.join(path, ".git"))


class GraphMemoryManager:
    def __init__(self):
        self.graph = GraphStore()
        os.makedirs(HUB_DIR, exist_ok=True)
        os.makedirs(PROGRESS_DIR, exist_ok=True)

    def load(self):
        if os.path.exists(GRAPH_DB_FILE):
            with open(GRAPH_DB_FILE, "r", encoding="utf-8") as f:
                self.graph.load(json.load(f))
        else:
            self.graph = GraphStore()

    def save(self):
        tmp = GRAPH_DB_FILE + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(self.graph.to_dict(), f, indent=2, ensure_ascii=False)
        os.replace(tmp, GRAPH_DB_FILE)

    def discover(self):
        found = []
        with hub_lock():
            self.load()
            project_props = {"_id": PROJECT, "path": REPO_ROOT}
            graphify_out = os.path.join(REPO_ROOT, "graphify-out")
            if os.path.exists(os.path.join(graphify_out, "graph.json")):
                project_props["graphify_out"] = graphify_out
            self.graph.add_node("Proyecto", project_props)
            for entry in sorted(os.listdir(REPO_ROOT)):
                path = os.path.join(REPO_ROOT, entry)
                if os.path.realpath(path) == os.path.realpath(BASE_DIR):
                    continue  # el propio arnes no es un microservicio
                if os.path.isdir(path) and is_repo_root(path):
                    qid = f"{PROJECT}/{entry}"
                    self.graph.add_node("Microservicio", {"_id": qid, "proyecto": PROJECT, "servicio": entry, "path": path})
                    self.graph.add_edge("CONTIENE", PROJECT, qid)
                    found.append(entry)
            self.save()
        listing = ", ".join(found) if found else "(ninguno)"
        print(f"[Memoria] Proyecto '{PROJECT}' en {HUB_DIR}: {len(found)} microservicio(s): {listing}")

    def sync_git(self, commit_hash, files, microservice):
        qserv = f"{PROJECT}/{microservice}"
        with hub_lock():
            self.load()
            self.graph.add_node("Commit", {"_id": commit_hash, "proyecto": PROJECT, "microservicio": microservice})
            self.graph.add_node("Agente", {"_id": "Agente_Implementador"})
            self.graph.add_edge("REALIZO", "Agente_Implementador", commit_hash)
            progress_path = os.path.join(PROGRESS_DIR, PROJECT, microservice)
            os.makedirs(progress_path, exist_ok=True)
            for file_path in files:
                if not file_path:
                    continue
                aid = artifact_id(file_path)
                nid = f"{qserv}:{aid}"
                self.graph.add_node(
                    "Artefacto",
                    {"_id": nid, "ruta": file_path, "estado": "MODIFICADO_GIT", "proyecto": PROJECT, "microservicio": microservice},
                )
                self.graph.add_edge("MODIFICO", commit_hash, nid)
                with open(os.path.join(progress_path, f"{aid}.json"), "w", encoding="utf-8") as f:
                    json.dump({"_id": nid, "ruta": file_path, "estado": "MODIFICADO_GIT", "commit": commit_hash}, f, indent=2, ensure_ascii=False)
            self.save()
        print(f"[Memoria] Commit {commit_hash[:7]} sincronizado para {qserv}")

    def link(self, consumer, target, transversal=False, origin="manual"):
        c = qualify(consumer)
        t = qualify(target)
        with hub_lock():
            self.load()
            self.graph.add_node("Microservicio", {"_id": c})
            props = {"_id": t}
            if transversal:
                props["tipo"] = "transversal"
            self.graph.add_node("Microservicio", props)
            self.graph.add_edge("DEPENDE_DE", c, t, origen=origin)
            self.save()
        suffix = " [transversal]" if transversal else ""
        print(f"[Memoria] {c} depende de {t}{suffix}")

    def unmark(self, service):
        s = qualify(service)
        with hub_lock():
            self.load()
            node = self.graph.nodes.get(s)
            if node and node.get("tipo") == "transversal":
                del node["tipo"]
                self.save()
                print(f"[Memoria] {s} ya no esta marcado como transversal")
            else:
                print(f"[Memoria] {s} no estaba marcado como transversal")

    def impact(self, service):
        t = qualify(service)
        with hub_lock():
            self.load()
        affected = sorted(e["source"] for e in self.graph.edges if e.get("type") == "DEPENDE_DE" and e.get("target") == t)
        if affected:
            print(f"[Impacto] Si modificas '{t}', revisa: {', '.join(affected)}")
        else:
            print(f"[Impacto] Ningun microservicio registrado depende de '{t}'")

    def record_event(self, agent, action, artefacto, estado, metadata=None):
        qart = f"{PROJECT}/{artefacto}"
        with hub_lock():
            self.load()
            self.graph.add_node("Agente", {"_id": agent})
            node = {"_id": qart, "estado": estado, "proyecto": PROJECT}
            if metadata:
                node["metadata"] = metadata
            self.graph.add_node("Artefacto", node)
            self.graph.add_edge(action.upper(), agent, qart)
            ruta = os.path.join(PROGRESS_DIR, PROJECT)
            os.makedirs(ruta, exist_ok=True)
            with open(os.path.join(ruta, f"{artefacto}.json"), "w", encoding="utf-8") as f:
                json.dump(node, f, indent=2, ensure_ascii=False)
            self.save()
        print(f"[Memoria] {agent} --[{action}]--> {qart} ({estado})")

    def query_state(self, artefacto, microservice="raiz"):
        if microservice == "raiz":
            ruta = os.path.join(PROGRESS_DIR, PROJECT, f"{artefacto}.json")
        else:
            ruta = os.path.join(PROGRESS_DIR, PROJECT, microservice, f"{artefacto}.json")
        if os.path.exists(ruta):
            with open(ruta, "r", encoding="utf-8") as f:
                print(f.read())
        else:
            print(f"Error: Artefacto '{artefacto}' no encontrado en {PROJECT}/{microservice}.")

    def derive_from_graphify(self, graph_path=None):
        import re
        graph_path = graph_path or os.path.join(REPO_ROOT, "graphify-out", "graph.json")
        if not os.path.exists(graph_path):
            print(f"[graphify->hub] No existe {graph_path}; nada que derivar.")
            return
        with open(graph_path, "r", encoding="utf-8") as f:
            graph = json.load(f)
        nodes = {n.get("id"): n for n in graph.get("nodes", [])}
        dep_relations = {"references", "implements", "shares_data_with", "depends_on", "uses", "cites"}
        convention = re.compile(r"convention|policy|guideline|standard|lint|layout|\\badr\\b", re.I)

        def service_for(node_id):
            source = (nodes.get(node_id) or {}).get("source_file", "") or ""
            match = re.search(r"(ms-[a-z0-9-]+-service|[a-z0-9-]+-ui)", source)
            return match.group(1) if match else None

        # Denylist de dependencias espurias (falsos positivos de la extraccion
        # semantica: contratos compartidos JWT/audit/infra leidos como deps).
        # Una linea "a->b" por par a suprimir; '#' para comentarios.
        denylist = set()
        deny_path = os.path.join(REPO_ROOT, "harness_deps_deny.txt")
        if os.path.exists(deny_path):
            with open(deny_path, "r", encoding="utf-8") as fh:
                for line in fh:
                    line = line.split("#", 1)[0]
                    line = re.sub(r"\s+", "", line)
                    if line:
                        denylist.add(line)

        pairs = {}
        consumers_by_target_node = {}
        raw = []
        for edge in graph.get("links", []):
            a = service_for(edge.get("source"))
            b = service_for(edge.get("target"))
            if not (a and b and a != b):
                continue
            if edge.get("relation") not in dep_relations:
                continue
            if f"{a}->{b}" in denylist:
                continue
            target_node = edge.get("target")
            if convention.search((nodes.get(target_node) or {}).get("label", "")):
                continue
            consumers_by_target_node.setdefault(target_node, set()).add(a)
            raw.append((a, b, target_node))

        for a, b, target_node in raw:
            pairs[(a, b)] = pairs.get((a, b), False) or len(consumers_by_target_node.get(target_node, set())) >= 3

        if not pairs:
            print("[graphify->hub] Sin dependencias derivables.")
            return

        with hub_lock():
            self.load()
            self.graph.edges = [
                e for e in self.graph.edges
                if not (e.get("type") == "DEPENDE_DE" and e.get("origen") == "graphify")
            ]
            existing = {(e.get("source"), e.get("target")) for e in self.graph.edges if e.get("type") == "DEPENDE_DE"}
            for (a, b), transversal in sorted(pairs.items()):
                c, d = qualify(a), qualify(b)
                self.graph.add_node("Microservicio", {"_id": c})
                props = {"_id": d}
                if transversal:
                    props["tipo"] = "transversal"
                self.graph.add_node("Microservicio", props)
                if (c, d) not in existing:
                    self.graph.add_edge("DEPENDE_DE", c, d, origen="graphify")
            self.save()
        summary = ", ".join(f"{a}->{b}{' [tv]' if tv else ''}" for (a, b), tv in sorted(pairs.items()))
        print(f"[graphify->hub] {len(pairs)} dependencia(s): {summary}")

    def map(self):
        with hub_lock():
            self.load()
        nodes = self.graph.nodes
        edges = self.graph.edges
        micros = {nid: n for nid, n in nodes.items() if n.get("_label") == "Microservicio"}
        deps = [e for e in edges if e.get("type") == "DEPENDE_DE"]
        projects = {}
        for nid in micros:
            projects.setdefault(nid.split("/", 1)[0], []).append(nid)
        for nid, node in nodes.items():
            if node.get("_label") == "Proyecto":
                projects.setdefault(nid, [])

        dependents = {}
        for edge in deps:
            dependents.setdefault(edge["target"], []).append(edge["source"])

        commit_count = sum(1 for node in nodes.values() if node.get("_label") == "Commit")
        print(f"== Mapa del Hub ({HUB_DIR}) ==")
        print(f"Proyectos: {len(projects)} | Microservicios: {len(micros)} | Dependencias: {len(deps)} | Commits: {commit_count}")
        print()
        for project in sorted(projects):
            graphify_out = (nodes.get(project) or {}).get("graphify_out")
            print(f"[{project}]" + (f"  [graphify: {graphify_out}]" if graphify_out else ""))
            services = sorted(projects[project])
            if not services:
                print("   (sin microservicios registrados)")
            for sid in services:
                short = sid.split("/", 1)[1] if "/" in sid else sid
                tags = []
                if (micros.get(sid) or {}).get("tipo") == "transversal":
                    tags.append("transversal")
                if dependents.get(sid):
                    tags.append(f"{len(dependents[sid])} dependiente(s)")
                outgoing = sorted(e["target"] for e in deps if e["source"] == sid)
                suffix = f" ({', '.join(tags)})" if tags else ""
                dep_text = " -> depende de: " + ", ".join(outgoing) if outgoing else ""
                print(f"   - {short}{suffix}{dep_text}")
            print()

        cross = [e for e in deps if e["source"].split("/", 1)[0] != e["target"].split("/", 1)[0]]
        if cross:
            print("Dependencias entre proyectos:")
            for edge in sorted(cross, key=lambda x: (x["source"], x["target"])):
                print(f"   {edge['source']} --> {edge['target']}")
        else:
            print("Dependencias entre proyectos: ninguna")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["descubrir", "mapa", "impacto", "vincular", "desmarcar", "sync_git", "vincular-grafo", "registrar", "consultar"])
    parser.add_argument("--microservicio", default="raiz")
    parser.add_argument("--destino")
    parser.add_argument("--transversal", action="store_true")
    parser.add_argument("--artefacto")
    parser.add_argument("--meta")
    parser.add_argument("--agente", default="AgentCLI")
    parser.add_argument("--accion")
    parser.add_argument("--estado")
    args = parser.parse_args()
    manager = GraphMemoryManager()

    if args.command == "registrar":
        if not (args.accion and args.estado and args.artefacto):
            parser.error("registrar requiere --accion, --estado y --artefacto")
        manager.record_event(args.agente, args.accion, args.artefacto, args.estado, args.meta)
    elif args.command == "consultar":
        if not args.artefacto:
            parser.error("--artefacto es requerido para consultar")
        manager.query_state(args.artefacto, args.microservicio)
    elif args.command == "descubrir":
        manager.discover()
    elif args.command == "mapa":
        manager.map()
    elif args.command == "impacto":
        manager.impact(args.microservicio)
    elif args.command == "vincular":
        if not args.destino:
            parser.error("--destino es requerido para vincular")
        manager.link(args.microservicio, args.destino, args.transversal)
    elif args.command == "desmarcar":
        manager.unmark(args.microservicio)
    elif args.command == "sync_git":
        if not args.artefacto:
            parser.error("--artefacto es requerido para sync_git")
        files = args.meta.split(",") if args.meta else []
        manager.sync_git(args.artefacto, files, args.microservicio)
    elif args.command == "vincular-grafo":
        manager.derive_from_graphify()


if __name__ == "__main__":
    main()
GM_PY_EOF
chmod +x graph_memory.py
write_file_notice "graph_memory.py"

echo "Generando init.sh..."
cat <<'INIT_SH_EOF' > init.sh
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

if ! command -v python3 >/dev/null 2>&1; then
    echo "[!] python3 no esta disponible; graph_memory.py lo requiere." >&2
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
        fi
    done

    cat > "$POST_COMMIT" <<HOOKEOF
#!/bin/bash
# harness-managed-hook v7
set -u
HARNESS_DIR="$HARNESS_DIR"
REPO_ROOT="$REPO_ROOT"
MICROSERVICIO=\$(basename "\$(git rev-parse --show-toplevel)")
COMMIT_HASH=\$(git rev-parse HEAD)
ARCHIVOS=\$(git diff-tree --no-commit-id --name-only -r --root "\$COMMIT_HASH" | paste -sd "," -)
python3 "\$HARNESS_DIR/graph_memory.py" sync_git --artefacto "\$COMMIT_HASH" --meta "\$ARCHIVOS" --microservicio "\$MICROSERVICIO" \
  || echo "[Harness] Aviso: no se pudo sincronizar memoria para \$MICROSERVICIO." >&2

export PATH="\$HOME/.local/bin:\$PATH"
if command -v graphify >/dev/null 2>&1 && [ -f "\$REPO_ROOT/graphify-out/graph.json" ]; then
    if mkdir "\$REPO_ROOT/graphify-out/.update.lock" 2>/dev/null; then
        (
            trap 'rmdir "\$REPO_ROOT/graphify-out/.update.lock" 2>/dev/null || true' EXIT
            cd "\$REPO_ROOT" || exit 0
            graphify update "\$REPO_ROOT" >/dev/null 2>&1 || true
            if printf '%s' "\$ARCHIVOS" | grep -qiE '(^|,)(README|AGENTS|[^,]+[.]md)(,|\$)'; then
                touch "\$REPO_ROOT/graphify-out/.graphify_stale" 2>/dev/null || true
            fi
            python3 "\$HARNESS_DIR/graph_memory.py" vincular-grafo >/dev/null 2>&1 || true
        ) &
    fi
fi
HOOKEOF
    chmod +x "$POST_COMMIT"

    cat > "$COMMIT_MSG" <<'CMEOF'
#!/bin/sh
# harness-managed-hook v4
set -u
msg_file="${1:?commit message file missing}"
tmp="${msg_file}.harness.$$"
sed -E '/^Co-Authored-By:.*([Cc]laude|[Cc]odex|[Gg]emini|[Gg]rok|[Aa]ntigravity|[Oo]pen[Aa][Ii]|[Aa]nthropic|[Gg]oogle|[Xx][Aa][Ii]|[Aa][Ii])/d; /^Generated with .*([Cc]laude|[Cc]odex|[Gg]emini|[Gg]rok|[Aa]ntigravity|[Oo]pen[Aa][Ii]|[Aa]nthropic|[Gg]oogle|[Xx][Aa][Ii]|[Aa][Ii])/d' "$msg_file" > "$tmp" && mv "$tmp" "$msg_file"
rm -f "$tmp" "$msg_file.bak"
CMEOF
    chmod +x "$COMMIT_MSG"
    echo "   -> [Ok] $REPO_DIR conectado"
done

python3 "$HARNESS_DIR/graph_memory.py" descubrir

if [ -f "$REPO_ROOT/graphify-out/graph.json" ]; then
    python3 "$HARNESS_DIR/graph_memory.py" vincular-grafo || true
    if [ -f "$REPO_ROOT/graphify-out/.graphify_stale" ]; then
        echo "[graphify] Grafo desactualizado: corre '/graphify --update' y luego borra graphify-out/.graphify_stale."
    else
        echo "[graphify] Grafo de conocimiento al dia."
    fi
else
    echo "[graphify] Sin graphify-out/graph.json. Primera construccion manual: /graphify"
fi

echo ""
python3 "$HARNESS_DIR/graph_memory.py" mapa
echo ""
echo "== [Ok] Harness listo =="
INIT_SH_EOF
chmod +x init.sh
write_file_notice "init.sh"

echo "Generando validate_ui.sh y debug_ui.js..."
cat <<'VAL_UI_EOF' > validate_ui.sh
#!/bin/bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd -P)"
cd "$ROOT"

TARGET_URL="${1:-http://localhost:5173}"
echo "== Validando UI en $TARGET_URL =="

if ! command -v node >/dev/null 2>&1; then
    echo "[!] Node.js no esta disponible." >&2
    exit 1
fi

if [ ! -d "node_modules/playwright" ]; then
    if ! command -v npm >/dev/null 2>&1; then
        echo "[!] Playwright no esta instalado y npm no esta disponible." >&2
        exit 1
    fi
    echo "[Harness] Instalando Playwright localmente..."
    npm install playwright --no-save
    npx playwright install chromium
fi

if ! node debug_ui.js "$TARGET_URL"; then
    echo "[!] UI con errores. Captura: $ROOT/debug-ui/ui_error_state.png" >&2
    exit 1
fi

echo "[Ok] UI verificada."
VAL_UI_EOF
chmod +x validate_ui.sh

cat <<'DEBUG_JS_EOF' > debug_ui.js
const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

(async () => {
  const targetUrl = process.argv[2] || 'http://localhost:5173';
  let hasErrors = false;
  console.log(`[Playwright] Verificando UI en ${targetUrl}...`);

  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  page.on('console', msg => {
    if (msg.type() === 'error') {
      console.error(`[Console Error] ${msg.text()}`);
      hasErrors = true;
    }
  });

  page.on('pageerror', exception => {
    console.error(`[Exception] ${exception}`);
    hasErrors = true;
  });

  page.on('requestfailed', request => {
    const failure = request.failure();
    console.error(`[Request Failed] ${request.method()} ${request.url()}${failure ? `: ${failure.errorText}` : ''}`);
    hasErrors = true;
  });

  try {
    await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 20000 });
    await page.waitForLoadState('networkidle', { timeout: 5000 }).catch(() => {});
    await page.waitForSelector('body', { timeout: 5000 });
  } catch (error) {
    console.error(`[Timeout] ${error.message}`);
    hasErrors = true;
  }

  if (hasErrors) {
    const outDir = path.join(__dirname, 'debug-ui');
    fs.mkdirSync(outDir, { recursive: true });
    const shot = path.join(outDir, 'ui_error_state.png');
    try {
      await page.screenshot({ path: shot });
      console.log(`[Playwright] Captura guardada en ${shot}`);
    } catch (error) {
      console.error(`[Playwright] No se pudo guardar captura: ${error.message}`);
    }
    await browser.close();
    process.exit(1);
  }

  await browser.close();
  console.log('[Playwright] Exito: sin errores visibles.');
})();
DEBUG_JS_EOF
write_file_notice "validate_ui.sh / debug_ui.js"

echo "Generando guardas y estado..."
cat <<'STATUS_EOF' > harness_status.sh
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
STATUS_EOF
chmod +x harness_status.sh

cat <<'GUARD_EOF' > commit_guard.sh
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
GUARD_EOF
chmod +x commit_guard.sh

cat <<'CHECK_SH_EOF' > harness_check.sh
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
CHECK_SH_EOF
chmod +x harness_check.sh
write_file_notice "harness_status.sh / commit_guard.sh / harness_check.sh"

echo "Generando harness.py..."
cat <<'HARNESS_PY_EOF' > harness.py
#!/usr/bin/env python3
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone

ROOT = os.path.dirname(os.path.abspath(__file__))
FEATURES = os.path.join(ROOT, "feature_list.json")
PROGRESS = os.path.join(ROOT, "progress")
CURRENT = os.path.join(PROGRESS, "current.md")
HISTORY = os.path.join(PROGRESS, "history.md")
GRAPH_MEM = os.path.join(ROOT, "graph_memory.py")


def _repo_root():
    """Raiz multi-repo: en layout 'subdir' es el padre del arnes (igual criterio
    que graph_memory.py). docs/ y graphify-out viven en esa raiz."""
    layout = os.path.join(ROOT, ".harness_layout")
    try:
        with open(layout, "r", encoding="utf-8") as fh:
            if fh.read().strip() == "subdir":
                return os.path.dirname(ROOT)
    except OSError:
        pass
    return ROOT


REPO_ROOT = os.environ.get("HARNESS_REPO_ROOT") or _repo_root()
# Los planes (y demas docs durables del proyecto) viven en el docs/ de la RAIZ
# multi-repo, junto a los PLAN-*.md / RUNBOOK del equipo, NO en la subcarpeta del
# arnes. progress/ queda solo para el estado vivo (current.md + history.md).
PLANS = os.path.join(REPO_ROOT, "docs")
# Linea base del checkpoint automatico (mtime = ultimo autocheck del hook).
AUTOCHECK_STAMP = os.path.join(PROGRESS, ".last_autocheck")
# Debounce del aviso "sin feature activa" (mtime = ultimo nudge emitido).
NUDGE_STAMP = os.path.join(PROGRESS, ".last_nudge")


def load_features():
    if not os.path.exists(FEATURES):
        return {"project": os.path.basename(ROOT), "features": []}
    with open(FEATURES, "r", encoding="utf-8") as f:
        return json.load(f)


def save_features(data):
    tmp = FEATURES + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
    os.replace(tmp, FEATURES)


def log(line):
    os.makedirs(PROGRESS, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with open(HISTORY, "a", encoding="utf-8") as f:
        f.write(f"- {stamp} {line}\n")


def find_feature(data, fid):
    for feature in data.get("features", []):
        if str(feature.get("id")) == str(fid):
            return feature
    raise SystemExit(f"Feature no encontrada: {fid}")


def slugify(text):
    s = re.sub(r"[^a-z0-9]+", "-", (text or "").lower()).strip("-")
    return (s or "feature")[:48]


def plan_path(feature):
    return os.path.join(PLANS, f"plan-feature-{feature.get('id')}-{slugify(feature.get('name', ''))}.md")


def plan_template(feature):
    services = feature.get("microservicios", []) or ["(sin servicios)"]
    lines = [
        f"# Plan - Feature #{feature.get('id')}: {feature.get('name')}",
        "",
        "Estado: in_progress",
        "Microservicios:",
    ]
    lines += [f"- {s}" for s in services]
    lines += [
        "",
        "## Alcance",
        "",
        "## Impacto entre microservicios",
        "<!-- python3 graph_memory.py impacto --microservicio <proyecto>/<servicio> -->",
        "",
        "## Consulta al grafo (graphify)",
        '<!-- graphify query "<pregunta de la task>" -->',
        "",
        "## Delegacion (implementer)",
        "- ",
        "",
        "## Criterios de cierre (reviewer)",
        "- ",
        "",
        "## Riesgos",
        "- ",
        "",
    ]
    return "\n".join(lines)


def write_plan(feature):
    """Persiste el plan en el docs/ de la RAIZ multi-repo (archivo permanente por
    feature). No pisa un plan ya escrito por el lider."""
    os.makedirs(PLANS, exist_ok=True)
    path = plan_path(feature)
    if not os.path.exists(path):
        with open(path, "w", encoding="utf-8") as f:
            f.write(plan_template(feature))
    return path


def _graphify_refresh():
    """Refresca graphify si esta instalado y hay grafo. Best-effort, bajo el
    mismo lock que el hook post-commit para no duplicar ni corromper la salida."""
    graph_json = os.path.join(REPO_ROOT, "graphify-out", "graph.json")
    if not (shutil.which("graphify") and os.path.exists(graph_json)):
        return
    lock = os.path.join(REPO_ROOT, "graphify-out", ".update.lock")
    try:
        os.mkdir(lock)
    except OSError:
        return  # ya hay un update en curso (p.ej. el hook); no dupliques
    stale = os.path.join(REPO_ROOT, "graphify-out", ".graphify_stale")
    try:
        rc = subprocess.run(
            ["graphify", "update", REPO_ROOT],
            check=False, timeout=300,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        ).returncode
        if rc == 0:
            try:
                os.remove(stale)  # grafo fresco: limpia el marcador
            except OSError:
                pass
        else:
            open(stale, "a").close()  # update fallo: marca stale
    except Exception:
        try:
            open(stale, "a").close()  # timeout u otro error: marca stale
        except OSError:
            pass
    finally:
        try:
            os.rmdir(lock)
        except OSError:
            pass


def _hub_register(accion, estado, artefacto, meta=""):
    try:
        subprocess.run(
            [sys.executable, GRAPH_MEM, "registrar",
             "--accion", accion, "--estado", estado,
             "--artefacto", artefacto, "--meta", meta, "--agente", "harness.py"],
            check=False,
        )
    except Exception as exc:
        print(f"[memoria] hub no actualizado: {exc}")


def _graphify_refresh_bg():
    """Como _graphify_refresh pero detached: lanza el rebuild en segundo plano y
    retorna de inmediato, para no colgar el turno cuando lo dispara un hook. Usa
    el mismo lock que el hook post-commit; el proceso hijo lo libera al terminar."""
    graph_json = os.path.join(REPO_ROOT, "graphify-out", "graph.json")
    if not (shutil.which("graphify") and os.path.exists(graph_json)):
        return
    lock = os.path.join(REPO_ROOT, "graphify-out", ".update.lock")
    try:
        os.mkdir(lock)
    except OSError:
        return  # ya hay un refresh en curso
    stale = os.path.join(REPO_ROOT, "graphify-out", ".graphify_stale")
    script = (
        "trap 'rmdir \"$LOCK\" 2>/dev/null || true' EXIT; "
        "if graphify update \"$ROOT\" >/dev/null 2>&1; then rm -f \"$STALE\"; "
        "else : > \"$STALE\"; fi"
    )
    env = dict(os.environ, ROOT=REPO_ROOT, STALE=stale, LOCK=lock)
    try:
        subprocess.Popen(
            ["bash", "-c", script], env=env,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            stdin=subprocess.DEVNULL, start_new_session=True,
        )
    except Exception:
        try:
            os.rmdir(lock)
        except OSError:
            pass


def _touch_stamp():
    """Fija la linea base del checkpoint automatico (mtime = ahora)."""
    try:
        os.makedirs(PROGRESS, exist_ok=True)
        open(AUTOCHECK_STAMP, "w").close()
    except OSError:
        pass


def update_memories(accion, estado, artefacto, meta="", refresh_graphify=False):
    """Mueve las memorias junto con la task: registra el evento en el hub y, en
    cierres/avances manuales, refresca graphify (sincrono). Best-effort."""
    _hub_register(accion, estado, artefacto, meta)
    if refresh_graphify:
        _graphify_refresh()


def cmd_status(_args):
    data = load_features()
    features = data.get("features", [])
    active = [f for f in features if f.get("status") == "in_progress"]
    pending = [f for f in features if f.get("status") == "pending"]
    blocked = [f for f in features if f.get("status") == "blocked"]
    done = [f for f in features if f.get("status") == "done"]
    print(f"Backlog: {len(features)} feature(s) | active={len(active)} pending={len(pending)} blocked={len(blocked)} done={len(done)}")
    for f in features:
        services = ", ".join(f.get("microservicios", [])) or "sin servicios"
        print(f"  #{f.get('id')} [{f.get('status')}] {f.get('name')} ({services})")
    if os.path.exists(CURRENT):
        with open(CURRENT, "r", encoding="utf-8") as f:
            content = f.read().strip()
        if content:
            print("\nprogress/current.md:")
            print(content)


def cmd_next(_args):
    data = load_features()
    for f in data.get("features", []):
        if f.get("status") == "pending":
            print(json.dumps(f, indent=2, ensure_ascii=False))
            return
    print("No hay features pending.")


def cmd_start(args):
    data = load_features()
    feature = find_feature(data, args.feature)
    active = [f for f in data.get("features", []) if f.get("status") == "in_progress" and str(f.get("id")) != str(args.feature)]
    if active:
        raise SystemExit(f"Ya hay feature in_progress: #{active[0].get('id')} {active[0].get('name')}")
    feature["status"] = "in_progress"
    feature["started_at"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    save_features(data)
    plan = write_plan(feature)
    rel_plan = os.path.relpath(plan, REPO_ROOT)
    os.makedirs(PROGRESS, exist_ok=True)
    with open(CURRENT, "w", encoding="utf-8") as f:
        f.write(f"# Feature #{feature.get('id')}: {feature.get('name')}\n\n")
        f.write("Estado: in_progress\n")
        f.write(f"Plan: {rel_plan}\n\n")
        f.write("Microservicios:\n")
        for service in feature.get("microservicios", []):
            f.write(f"- {service}\n")
        f.write("\nEvidencia:\n- \n")
    log(f"start feature #{feature.get('id')} {feature.get('name')}")
    update_memories("start", "in_progress", f"feature-{feature.get('id')}", feature.get("name", ""))
    _touch_stamp()  # linea base: el plan recien creado no dispara autocheck
    print(f"Feature #{feature.get('id')} iniciada. Plan: {rel_plan}")


def cmd_close(args):
    data = load_features()
    feature = find_feature(data, args.feature)
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    feature["status"] = args.status
    feature["closed_at"] = stamp
    if args.note:
        feature["note"] = args.note
    save_features(data)
    plan = plan_path(feature)
    if os.path.exists(plan):
        with open(plan, "a", encoding="utf-8") as f:
            f.write(f"\n---\nCerrado: {stamp} - status={args.status} - {args.note or ''}\n")
    os.makedirs(PROGRESS, exist_ok=True)
    with open(CURRENT, "w", encoding="utf-8") as f:
        f.write("# Estado Actual\n\nSin feature activa.\n\n## Evidencia\n\n-\n")
    log(f"close feature #{feature.get('id')} status={args.status} note={args.note or ''}")
    update_memories("close", args.status, f"feature-{feature.get('id')}", args.note or "", refresh_graphify=True)
    try:
        os.remove(AUTOCHECK_STAMP)  # cierra el ciclo de checkpoints automaticos
    except OSError:
        pass
    print(f"Feature #{feature.get('id')} cerrada como {args.status}.")


def active_feature(data, fid=None):
    """La feature objetivo de un avance: la indicada por --feature, o la unica
    in_progress si se omite."""
    if fid is not None:
        return find_feature(data, fid)
    active = [f for f in data.get("features", []) if f.get("status") == "in_progress"]
    if not active:
        raise SystemExit("No hay feature in_progress. Inicia una: harness.py start --feature <id>")
    if len(active) > 1:
        ids = ", ".join(f"#{f.get('id')}" for f in active)
        raise SystemExit(f"Varias features in_progress ({ids}); especifica --feature <id>.")
    return active[0]


def cmd_advance(args):
    """Registra un hito intermedio de la feature activa SIN cerrarla: mueve plan,
    current.md, history.md y las memorias (hub + graphify) de una sola vez."""
    data = load_features()
    feature = active_feature(data, args.feature)
    if feature.get("status") != "in_progress":
        raise SystemExit(f"Feature #{feature.get('id')} no esta in_progress (status={feature.get('status')}); usa start.")
    fid = feature.get("id")
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    # 1) Plan: deja rastro del hito en el cuerpo del plan (append, no pisa).
    plan = plan_path(feature)
    if os.path.exists(plan):
        with open(plan, "a", encoding="utf-8") as f:
            f.write(f"\n### Avance {stamp}\n{args.nota}\n")
    # 2) current.md: suma el avance a la evidencia (append, no reescribe).
    if os.path.exists(CURRENT):
        with open(CURRENT, "a", encoding="utf-8") as f:
            f.write(f"- {stamp} {args.nota}\n")
    # 3) history.md: una linea append-only.
    log(f"advance feature #{fid} {args.nota}")
    # 4) Memorias: hub (in_progress, con la nota) + graphify (best-effort).
    update_memories("advance", "in_progress", f"feature-{fid}", args.nota,
                    refresh_graphify=not args.no_graphify)
    _touch_stamp()  # un advance manual tambien resetea la linea base del auto
    extra = "" if args.no_graphify else " (hub + graphify)"
    print(f"Avance registrado en feature #{fid}{extra}.")


def cmd_autocheck(args):
    """Checkpoint automatico para los hooks (fin de turno, multi-LLM): si hay UNA
    feature in_progress y su plan/evidencia cambio desde el ultimo checkpoint,
    registra un avance auto (hub + graphify en segundo plano + history.md).
    Silencioso, idempotente y best-effort: jamas debe romper un turno."""
    try:
        data = load_features()
        active = [f for f in data.get("features", []) if f.get("status") == "in_progress"]
        if len(active) != 1:
            return
        feature = active[0]
        fid = feature.get("id")
        last = os.path.getmtime(AUTOCHECK_STAMP) if os.path.exists(AUTOCHECK_STAMP) else 0.0
        watched = []
        plan = plan_path(feature)
        if os.path.exists(plan):
            watched.append(plan)
        if os.path.isdir(PLANS):
            for name in os.listdir(PLANS):
                if name.endswith(".md") and name.startswith(("impl-", "review-")):
                    watched.append(os.path.join(PLANS, name))
        changed = sorted({os.path.basename(p) for p in watched if os.path.getmtime(p) > last})
        if not changed:
            return
        nota = "auto: " + ", ".join(changed)
        log(f"autocheck feature #{fid} {nota}")
        _hub_register("advance", "in_progress", f"feature-{fid}", nota)
        if not args.no_graphify:
            _graphify_refresh_bg()
        _touch_stamp()
        print(f"[autocheck] avance auto en feature #{fid}: {nota}")
    except Exception as exc:
        # Best-effort absoluto: corre al cierre de cada turno; nunca abortes.
        print(f"[autocheck] omitido: {exc}")


def cmd_nudge(_args):
    """Aviso (no bloqueante) para los hooks post-tool: si NO hay feature
    in_progress, recuerda registrar el trabajo antes de seguir editando, para que
    se active el ciclo (plan en docs/ + autocheck, que duerme sin feature activa).
    Debounced (~10 min) y best-effort: escribe a stderr y nunca falla."""
    try:
        data = load_features()
        if any(f.get("status") == "in_progress" for f in data.get("features", [])):
            return  # hay feature activa: nada que recordar
        last = os.path.getmtime(NUDGE_STAMP) if os.path.exists(NUDGE_STAMP) else 0.0
        if datetime.now(timezone.utc).timestamp() - last < 600:
            return  # ya avisamos hace poco
        os.makedirs(PROGRESS, exist_ok=True)
        open(NUDGE_STAMP, "w").close()
        sys.stderr.write(
            "[harness] Sin feature activa: el avance NO se esta capturando "
            "(autocheck duerme sin una feature in_progress). Antes de seguir, "
            "consulta graphify, corre impacto y registra el trabajo con "
            "'harness.py add' + 'harness.py start'.\n"
        )
    except Exception:
        pass


def cmd_add(args):
    data = load_features()
    ids = [int(f.get("id", 0)) for f in data.get("features", []) if str(f.get("id", "")).isdigit()]
    fid = max(ids, default=0) + 1
    feature = {
        "id": fid,
        "name": args.name,
        "microservicios": args.service or [],
        "acceptance": args.acceptance or [],
        "status": "pending",
    }
    data.setdefault("features", []).append(feature)
    save_features(data)
    log(f"add feature #{fid} {args.name}")
    print(f"Feature #{fid} agregada.")


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("status").set_defaults(func=cmd_status)
    sub.add_parser("next").set_defaults(func=cmd_next)

    start = sub.add_parser("start")
    start.add_argument("--feature", required=True)
    start.set_defaults(func=cmd_start)

    close = sub.add_parser("close")
    close.add_argument("--feature", required=True)
    close.add_argument("--status", choices=["done", "blocked", "pending"], required=True)
    close.add_argument("--note")
    close.set_defaults(func=cmd_close)

    adv = sub.add_parser("advance")
    adv.add_argument("--feature")
    adv.add_argument("--nota", required=True)
    adv.add_argument("--no-graphify", action="store_true")
    adv.set_defaults(func=cmd_advance)

    autochk = sub.add_parser("autocheck")
    autochk.add_argument("--no-graphify", action="store_true")
    autochk.set_defaults(func=cmd_autocheck)

    sub.add_parser("nudge").set_defaults(func=cmd_nudge)

    add = sub.add_parser("add")
    add.add_argument("--name", required=True)
    add.add_argument("--service", action="append")
    add.add_argument("--acceptance", action="append")
    add.set_defaults(func=cmd_add)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
HARNESS_PY_EOF
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
    cat <<'LEADER_ROLE_EOF' > roles/leader.md
# Lider (planner)

Define alcance, impacto y delegacion. NO implementas codigo si puedes delegarlo
al implementer: tu salida es el plan, no el diff.

## Protocolo

1. Lee `__HREL__roles/README.md`, `__HREL__feature_list.json` y
   `__HREL__progress/current.md`.
2. Revisa el mapa del hub: `python3 "__HREL__graph_memory.py" mapa`.
3. Para cada servicio candidato, calcula su radio de impacto:
   `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
4. Si existe `graphify-out/graph.json`, consulta el grafo antes de leer a ciegas:
   `graphify query "<pregunta de la task>"`
5. Persiste el plan en `docs/plan-feature-<id>-<slug>.md` (en el `docs/` de la
   RAIZ del proyecto, junto a los PLAN-*.md del equipo): alcance, microservicios
   afectados, riesgos y delegacion concreta (que archivos y en que orden).
   `__HREL__progress/current.md` queda como puntero vivo; `harness.py start`
   siembra ambos.

## Entregable

- Feature activa identificada (una sola a la vez).
- Microservicios afectados, con su radio de impacto.
- Riesgos conocidos.
- Delegacion concreta para el implementer y criterios de cierre para el reviewer.

## Reglas

- No edites codigo fuente. Si hay que tocar contratos compartidos, registralo
  como impacto antes de delegar.
- Una respuesta corta en chat no reemplaza el plan persistido en `docs/`.
LEADER_ROLE_EOF

    cat <<'IMPLEMENTER_ROLE_EOF' > roles/implementer.md
# Implementer

Implementas UNA unidad concreta del plan del lider.

## Protocolo

1. Lee el plan en `docs/plan-feature-<id>-<slug>.md` (apuntado desde
   `__HREL__progress/current.md`) y, si lo necesitas, tu rol en
   `__HREL__roles/implementer.md`.
2. Trabaja solo en los microservicios asignados. No cambies contratos
   compartidos sin registrar impacto:
   `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
3. Haz cambios pequenos y verificables. Ejecuta los tests cercanos al cambio
   (ver `__HREL__docs/verification.md`).
4. Deja evidencia en `docs/impl-<feature>.md` (en el `docs/` de la RAIZ).
5. Registra hitos intermedios con
   `python3 "__HREL__harness.py" advance --nota "<que avanzaste>"`: mueve hub,
   graphify, history.md y current.md sin esperar al cierre. (Al cerrar cada turno
   el hook hace un checkpoint automatico si el plan/evidencia cambio; usa
   `advance` para la nota explicita de que hiciste.)

## Reporte minimo (docs/impl-<feature>.md)

- Archivos modificados.
- Decisiones tomadas.
- Comandos ejecutados y su resultado.
- Riesgos pendientes para el reviewer.

## Reglas

- No cierres la feature: eso es del reviewer mas los checkpoints.
- Sin firmas de IA en commits; `commit_guard.sh` las bloquea.
IMPLEMENTER_ROLE_EOF

    cat <<'REVIEWER_ROLE_EOF' > roles/reviewer.md
# Reviewer

Verificas calidad, impacto y criterios de cierre. NO implementas.

## Verifica

- Impacto ejecutado para cada servicio modificado:
  `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
- Tests relevantes ejecutados y en verde (ver `__HREL__docs/verification.md`).
- Frontends validados cuando aplique: `bash "__HREL__validate_ui.sh" <url>`.
- `graphify query` usado, o justificacion si no hay grafo.
- Plan archivado en `docs/` de la raiz y al dia con lo implementado.
- Task y memorias en sync: cierra con
  `python3 "__HREL__harness.py" close --feature <id> --status <estado>`, que
  registra el hub y refresca graphify automaticamente.
- Checkpoints completos (`__HREL__CHECKPOINTS.md`).
- Repos afectados limpios o commiteados segun politica.
- `bash "__HREL__harness_check.sh"` limpio.

## Veredicto (docs/review-<feature>.md)

- `approved`
- `changes_requested` (con lista accionable)
- `blocked` (con causa y desbloqueo propuesto)

## Reglas

- Solo lectura mas ejecucion de validaciones. No edites codigo fuente.
REVIEWER_ROLE_EOF

    cat <<'AGENTMAP_EOF' > roles/README.md
# Mapa de Agentes

Arnes multi-LLM con tres roles. Lee solo lo necesario para la tarea actual
(mapa progresivo): primero el plan, luego el rol, luego el codigo.

## Flujo

```
  __HREL__feature_list.json
            |
            v
   +-----------+    plan en        +--------------+   evidencia en   +------------+
   |  LIDER    |--> docs/plan-* --> | IMPLEMENTER  |--> docs/impl-* -->| REVIEWER   |
   | (planner) |                    | (1 unidad)   |                  | (verifica) |
   +-----------+                    +--------------+                  +------------+
        ^                                                                   |
        |                       changes_requested                           |
        +-------------------------------------------------------------------+
                                       |
                             approved + checkpoints OK
                                       v
                        harness_check.sh limpio  ->  cierre
```

## Roles

| Rol         | Cuando usarlo                             | Tools (Claude)          | Escribe en                |
|-------------|-------------------------------------------|-------------------------|---------------------------|
| leader      | Al iniciar: alcance, impacto, plan        | Read, Grep, Glob, Bash  | docs/plan-feature-<f>.md  |
| implementer | Escribir o modificar una unidad de codigo | Read, Edit, Write, Bash | docs/impl-<f>.md          |
| reviewer    | Antes de cerrar: tests, impacto, gates    | Read, Grep, Glob, Bash  | docs/review-<f>.md        |

Definicion completa: `__HREL__roles/leader.md`, `__HREL__roles/implementer.md`,
`__HREL__roles/reviewer.md`.

## Como se orquesta por herramienta

Mismos tres roles; cada CLI los recibe en su formato nativo (auto-registrados):

- **Claude Code**: `.claude/agents/*.md` (frontmatter `name`/`description`/
  `tools`/`model`/`effort`; cuerpo = system prompt). El hilo principal delega.
- **Codex CLI**: `.codex/agents/*.toml` (`name`, `description`,
  `developer_instructions`, `sandbox_mode`, `model_reasoning_effort`).
  Delegacion explicita (`/agent` o pidiendolo). No hay allowlist de tools: la
  capacidad se acota con `sandbox_mode`.
- **Gemini CLI**: `.gemini/agents/*.md` (frontmatter + cuerpo). Invocar con
  `@<rol>`; auto-delega segun `description`.
- **Grok Build (xAI)**: sin formato propio, pero LEE `.claude/agents/*.md` por
  compatibilidad con Claude Code (sin archivos extra). Puede ignorar un `model:`
  de Claude y caer al modelo por defecto de Grok.

Sin archivo de definicion soportado (aplican `__HREL__roles/*.md` como fases
secuenciales lider -> implementer -> reviewer en una sola sesion):

- **Antigravity**: crea sus subagentes dinamicamente en runtime; lee tambien
  `AGENTS.md` / `.agents/rules/`.
- **Cualquier otro CLI** sin subagentes nativos.

Claude Code no permite subagentes anidados: delega el hilo principal, no el
subagente `leader`.

## Modelos, effort y tools por rol (tunable)

- **Claude** (`.claude/agents/*.md`): `leader` y `reviewer` con
  `model: claude-opus-4-8` (Opus 4.8); `implementer` con
  `model: claude-sonnet-4-6` (Sonnet 4.6); los tres con `effort: max`. `model:`
  acepta ID fijo o alias auto-ultima-version (`opus`, `sonnet`, `haiku`,
  `inherit`); `effort:` es `low|medium|high|xhigh|max` (`xhigh` solo Opus 4.7+).
  El `effort:` del frontmatter NO sobreescribe la env var
  `CLAUDE_CODE_EFFORT_LEVEL`.
- **Codex** (`.codex/agents/*.toml`): `model` se hereda de la sesion;
  `model_reasoning_effort = high` (tope de Codex). Read-only via
  `sandbox_mode = read-only`; el implementer usa `workspace-write`.
- **Gemini** (`.gemini/agents/*.md`): `model` y `tools` se heredan de la sesion
  (omitidos para no fijar IDs/nombres que cambian por version). Agregalos por
  rol cuando confirmes los nombres de tools/model de tu version instalada.

## Regla anti perdida de contexto

Los documentos durables se escriben en `docs/` de la raiz; `progress/` guarda
solo el estado vivo. Una respuesta corta en chat no reemplaza evidencia
persistida.
AGENTMAP_EOF

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

    cat <<'CHECKPOINTS_EOF' > CHECKPOINTS.md
# Checkpoints

Antes de cerrar una tarea:

- [ ] La feature activa en `feature_list.json` refleja el estado real.
- [ ] El plan vive en `docs/plan-feature-<feature>.md` (raiz) y refleja lo hecho.
- [ ] `progress/current.md` apunta al plan y contiene evidencia al dia.
- [ ] Se ejecuto impacto para los microservicios modificados:
      `python3 graph_memory.py impacto --microservicio <proyecto>/<servicio>`
- [ ] Si existe `graphify-out/graph.json`, se consulto `graphify query`.
- [ ] Tests relevantes ejecutados por cada microservicio afectado.
- [ ] Frontends validados con `validate_ui.sh <url>` cuando aplique.
- [ ] `docs/review-<feature>.md` contiene veredicto del reviewer.
- [ ] Repos afectados limpios o commiteados segun politica.
- [ ] Task y memorias en sync: cierre via
      `python3 harness.py close --feature <id> --status <estado>` (registra el hub
      y refresca graphify).
- [ ] `harness_check.sh` pasa o el bloqueo queda documentado.
CHECKPOINTS_EOF

    # Backlog vivo: solo se siembra si falta. Un reinstall NO debe vaciar las
    # features ya cargadas.
    if [ ! -f feature_list.json ]; then
        cat <<FEATURES_EOF > feature_list.json
{
  "project": "$PROJECT_NAME",
  "rules": {
    "one_feature_at_a_time": true,
    "require_tests_to_close": true,
    "require_impact_check": true
  },
  "features": []
}
FEATURES_EOF
    fi

    # Estado vivo: solo se siembra si falta. Un reinstall NO debe pisar la tarea
    # en curso ni la bitacora ya escrita.
    if [ ! -f progress/current.md ]; then
        cat <<'CURRENT_EOF' > progress/current.md
# Estado Actual

Sin feature activa.

## Evidencia

-
CURRENT_EOF
    fi

    if [ ! -f progress/history.md ]; then
        cat <<'HISTORY_EOF' > progress/history.md
# Historial
HISTORY_EOF
    fi

    cat <<'ARCH_EOF' > docs/architecture.md
# Arquitectura

Completa este archivo con:

- Microservicios y responsabilidades.
- Dependencias internas y externas.
- Servicios transversales.
- Riesgos conocidos.
- Flujos criticos.
ARCH_EOF

    cat <<'CONV_EOF' > docs/conventions.md
# Convenciones

- Usa Conventional Commits.
- No agregues `Co-Authored-By` ni firmas generadas por IA.
- Trabaja dentro del microservicio afectado.
- Prefiere cambios pequenos, verificables y documentados.
- Registra decisiones relevantes en `progress/`.
CONV_EOF

    cat <<'VERIF_EOF' > docs/verification.md
# Verificacion

Registra aqui los comandos oficiales por tipo de proyecto.

Ejemplos:

```bash
go test ./...
npm test
npm run lint
bash validate_ui.sh http://localhost:5173
```
VERIF_EOF

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
