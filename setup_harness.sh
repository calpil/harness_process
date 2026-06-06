#!/usr/bin/env bash
# shellcheck shell=bash
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
#
# Mejoras aplicadas (best practices 2025-2026):
#   - #!/usr/bin/env bash + shellcheck directive
#   - Logging estructurado con colores + --log-file
#   - --dry-run / --preview completo
#   - --reset (uninstall / limpieza de artefactos generados)
#   - Descarga verificada (no pipe directo) para Antigravity + retry
#   - --version + --json + reporte de idempotencia
#   - Lockfile anti-concurrencia
#   - Soporte config file (.harness.env / HARNESS_CONFIG)
#   - PATH guidance post --user pip
#   - Traps centralizados para temps + reintentos en red
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

# Nuevas opciones globales (mejoras)
DRY_RUN=0
RESET=0
JSON_OUTPUT=0
LOG_FILE=""
HARNESS_VERSION="2026.06-harness-process"
LOCK_FILE=""
# Contadores para reporte de idempotencia
COUNT_BACKED_UP=0
COUNT_CREATED=0
COUNT_SKIPPED=0
COUNT_INSTALLED=0
COUNT_REMOVED=0
# Lista de temps para cleanup centralizado
TEMP_PATHS=()

# Colores (solo si hay TTY)
if [ -t 1 ]; then
    C_RESET="\033[0m"
    C_BOLD="\033[1m"
    C_RED="\033[0;31m"
    C_GREEN="\033[0;32m"
    C_YELLOW="\033[0;33m"
    C_BLUE="\033[0;34m"
    C_CYAN="\033[0;36m"
else
    C_RESET=""
    C_BOLD=""
    C_RED=""
    C_GREEN=""
    C_YELLOW=""
    C_BLUE=""
    C_CYAN=""
fi

log() {
    local level="$1"; shift
    local msg="$*"
    local prefix ts
    ts="$(date +%Y-%m-%dT%H:%M:%S%z)"
    case "$level" in
        INFO)  prefix="${C_BLUE}[INFO]${C_RESET}" ;;
        WARN)  prefix="${C_YELLOW}[WARN]${C_RESET}" ;;
        ERROR) prefix="${C_RED}[ERROR]${C_RESET}" ;;
        OK| SUCCESS) prefix="${C_GREEN}[OK]${C_RESET}" ;;
        *)     prefix="[$level]" ;;
    esac
    local line="[$ts] $prefix $msg"
    echo -e "$line"
    if [ -n "$LOG_FILE" ]; then
        echo "$line" | sed 's/\x1b\[[0-9;]*m//g' >> "$LOG_FILE"
    fi
}

log_info() { log INFO "$@"; }
log_warn() { log WARN "$@"; }
log_error() { log ERROR "$@"; }
log_success() { log SUCCESS "$@"; }

# Registrar temp para cleanup
register_temp() {
    TEMP_PATHS+=("$1")
}

cleanup_temps() {
    for p in "${TEMP_PATHS[@]:-}"; do
        if [ -e "$p" ]; then
            rm -rf "$p" 2>/dev/null || true
        fi
    done
}

# Trap maestro para temps (mejora)
trap 'cleanup_temps' EXIT INT TERM

# Carga config file (soporte .harness.env / HARNESS_CONFIG) antes de flags/env
load_config_file() {
    local cfg="${HARNESS_CONFIG:-}"
    # HARNESS_DIR puede aun no estar resuelto en el momento de parseo inicial;
    # usamos dirname $0 como fallback seguro.
    local hdir="${HARNESS_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" 2>/dev/null && pwd -P || echo .)}"
    if [ -z "$cfg" ]; then
        if [ -f "$hdir/.harness.env" ]; then
            cfg="$hdir/.harness.env"
        elif [ -f "$HOME/.config/harness/config" ]; then
            cfg="$HOME/.config/harness/config"
        elif [ -f "$HOME/.harnessrc" ]; then
            cfg="$HOME/.harnessrc"
        fi
    fi
    if [ -n "$cfg" ] && [ -f "$cfg" ]; then
        # shellcheck disable=SC1090
        set -a
        # shellcheck disable=SC1090
        . "$cfg"
        set +a
        log_info "Config cargada desde: $cfg"
    fi
}

# Lockfile simple (anti-concurrencia)
acquire_lock() {
    local lockdir="${TMPDIR:-/tmp}/harness-setup.lock.$$"
    LOCK_FILE="$lockdir"
    if mkdir "$lockdir" 2>/dev/null; then
        register_temp "$lockdir"
        echo $$ > "$lockdir/pid"
        return 0
    else
        if [ -f "$lockdir/pid" ]; then
            local other_pid
            other_pid=$(cat "$lockdir/pid" 2>/dev/null || echo "?")
            log_error "Otro setup_harness.sh esta corriendo (pid $other_pid). Usa --force para ignorar (no recomendado)."
        fi
        return 1
    fi
}

release_lock() {
    if [ -n "$LOCK_FILE" ] && [ -d "$LOCK_FILE" ]; then
        rm -rf "$LOCK_FILE" 2>/dev/null || true
    fi
}
trap 'release_lock; cleanup_temps' EXIT INT TERM

# Resolver HARNESS_DIR lo mas temprano posible (para config y lock)
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"

# Retry con backoff para operaciones de red (graphify, antigravity)
retry_cmd() {
    local attempts=4
    local delay=2
    local cmd=("$@")
    local i=1
    while [ $i -le $attempts ]; do
        if "${cmd[@]}"; then
            return 0
        fi
        if [ $i -lt $attempts ]; then
            log_warn "Intento $i fallido para: ${cmd[*]}. Reintentando en ${delay}s..."
            sleep "$delay"
            delay=$((delay * 2))
        fi
        i=$((i + 1))
    done
    return 1
}

print_final_report() {
    local status="success"
    if [ "$DRY_RUN" -eq 1 ]; then
        status="dry-run"
    fi

    log_success "========================================================"
    log_info "Harness Process - Reporte final (version: $HARNESS_VERSION)"
    log_info "  layout: $LAYOUT"
    log_info "  dry-run: $([ "$DRY_RUN" -eq 1 ] && echo si || echo no)"
    log_info "  subagentes: $([ "$WITH_SUBAGENTS" -eq 1 ] && echo si || echo no)"
    log_info "  graphify: $([ "$INSTALL_GRAPHIFY" -eq 1 ] && echo si || echo no)"
    log_info "  /graphify skills: $([ "$INSTALL_GRAPHIFY_SKILLS" -eq 1 ] && echo si || echo no)"
    log_info "  antigravity: $([ "$INSTALL_ANTIGRAVITY" -eq 1 ] && echo si || echo no)"
    log_info ""
    log_info "Acciones:"
    log_info "  Backups realizados: $COUNT_BACKED_UP"
    log_info "  Archivos/acciones creados: $COUNT_CREATED"
    log_info "  Skipped (ya existian o no aplico): $COUNT_SKIPPED"
    log_info "  Herramientas instaladas/aseguradas: $COUNT_INSTALLED"
    log_info "  Elementos eliminados (reset): $COUNT_REMOVED"
    log_success "========================================================"

    if [ "$JSON_OUTPUT" -eq 1 ]; then
        # Emitir JSON al final (stdout limpio para parsers)
        python3 - "$HARNESS_VERSION" "$LAYOUT" "$DRY_RUN" "$WITH_SUBAGENTS" \
                "$COUNT_BACKED_UP" "$COUNT_CREATED" "$COUNT_SKIPPED" "$COUNT_INSTALLED" "$COUNT_REMOVED" "$status" <<'PYJSON'
import sys, json
ver, lay, dr, sub, b, c, s, i, r, st = sys.argv[1:]
print(json.dumps({
    "version": ver,
    "layout": lay,
    "dry_run": bool(int(dr)),
    "with_subagents": bool(int(sub)),
    "actions": {
        "backed_up": int(b),
        "created": int(c),
        "skipped": int(s),
        "installed": int(i),
        "removed": int(r)
    },
    "status": st
}, indent=2))
PYJSON
    fi
}

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
  --with-postgres      PostgreSQL es obligatorio; se mantiene por compatibilidad.
  --subdir             (DEFAULT) El arnes vive en esta subcarpeta y orquesta el
                       directorio PADRE: escribe superficies multi-LLM en el
                       padre (raiz multi-repo) y mantiene aqui los scripts.
                       Correlo desde dentro de la subcarpeta del arnes.
  --root               El arnes vive EN la raiz multi-repo (hermano de los
                       microservicios). Layout clasico; desactiva el default.
  --force              Sobrescribe archivos sin crear backup.
  --dry-run            Modo preview: no escribe nada, no instala, solo muestra acciones.
  --reset              Limpia artefactos generados por Harness (superficies, hooks,
                       binarios, roles, etc.). Usa backups en bkp/ para recuperar si
                       hace falta. No toca tu codigo fuente.
  --version            Muestra version y sale.
  --json               Salida final en JSON (reporte de acciones + estado).
  --log-file <path>    Escribe log (sin colores) a archivo ademas de stdout.
  --config <path>      Carga variables extra desde archivo (antes de .env del hub).
  -h, --help           Muestra esta ayuda.

El layout por defecto es subdir (usa --root para el layout clasico). Por defecto
instala todas las superficies y hooks LLM conocidos (sin --target), la capa de
subagentes, asegura graphify y Antigravity CLI (los instala si faltan).
Respalda archivos existentes bajo bkp/ (configurable con HARNESS_BKP_DIR).

Mejoras 2026: dry-run, reset, logging con colores, lockfile, reintentos, PATH
guidance, config file, shellcheck-ready, shebang portable, reporte idempotencia.
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
        --with-postgres) ;;
        --subdir) LAYOUT=subdir ;;
        --root) LAYOUT=root ;;
        --force) FORCE=1 ;;
        --dry-run|--preview) DRY_RUN=1 ;;
        --reset) RESET=1 ;;
        --version) echo "$HARNESS_VERSION"; exit 0 ;;
        --json) JSON_OUTPUT=1 ;;
        --log-file)
            shift
            LOG_FILE="$1"
            mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null || true
            ;;
        --config)
            shift
            HARNESS_CONFIG="$1"
            ;;
        -h|--help) usage; exit 0 ;;
        *)
            log_error "Opcion desconocida: $1"
            usage >&2
            exit 2
            ;;
    esac
    shift
done

# Cargar config file lo antes posible (despues de parsear --config)
load_config_file

# Adquirir lock (salvo dry-run o force explicito en reset)
if [ "$DRY_RUN" -eq 0 ]; then
    if ! acquire_lock; then
        if [ "$FORCE" -eq 0 ] && [ "$RESET" -eq 0 ]; then
            exit 1
        fi
        log_warn "Continuando a pesar de lock (FORCE o RESET)."
    fi
fi

# Si es --reset, manejar temprano (antes de resolver paths completos)
if [ "$RESET" -eq 1 ]; then
    # Todavia necesitamos paths basicos; resolvemos lo minimo
    HARNESS_DIR="$(cd "$(dirname "$0")" && pwd -P)"
    cd "$HARNESS_DIR" || exit 1
    if [ "$LAYOUT" = "subdir" ]; then
        REPO_ROOT="$(dirname "$HARNESS_DIR")"
    else
        REPO_ROOT="$HARNESS_DIR"
    fi
    SURFACE_DIR="$REPO_ROOT"
    BKP_DIR="${HARNESS_BKP_DIR:-$HARNESS_DIR/bkp}"

    log_info "Modo RESET activado. Limpiando artefactos generados por Harness..."
    if [ "$DRY_RUN" -eq 1 ]; then
        log_warn "[DRY-RUN] No se eliminara nada."
    fi

    # Lista de artefactos a limpiar (superficies, hooks, agentes, bin, markers, etc.)
    reset_targets=(
        "$SURFACE_DIR/CLAUDE.md"
        "$SURFACE_DIR/AGENTS.md"
        "$SURFACE_DIR/GEMINI.md"
        "$SURFACE_DIR/LLM.md"
        "$SURFACE_DIR/.claude/settings.json"
        "$SURFACE_DIR/.claude/agents"
        "$SURFACE_DIR/.codex/hooks.json"
        "$SURFACE_DIR/.codex/agents"
        "$SURFACE_DIR/.gemini/settings.json"
        "$SURFACE_DIR/.gemini/commands"
        "$SURFACE_DIR/.grok/hooks"
        "$SURFACE_DIR/.grok/GROK.md"
        "$SURFACE_DIR/bin/harness-hook"
        "$SURFACE_DIR/bin/harness-claude"
        "$SURFACE_DIR/bin/harness-codex"
        "$SURFACE_DIR/bin/harness-gemini"
        "$SURFACE_DIR/bin/harness-grok"
        "$SURFACE_DIR/bin/harness-antigravity"
        "$HARNESS_DIR/.harness_layout"
        "$HARNESS_DIR/.harness_backend"
        "$HARNESS_DIR/roles"
        "$HARNESS_DIR/docs"
        "$HARNESS_DIR/progress"
        "$HARNESS_DIR/CHECKPOINTS.md"
        "$HARNESS_DIR/feature_list.json"
        # No tocamos graph_memory.py ni los scripts base del harness en reset
        # (el usuario puede querer mantener los scripts del arnes)
    )

    for t in "${reset_targets[@]}"; do
        if [ -e "$t" ]; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "[DRY-RUN] Eliminaria: $t"
                COUNT_REMOVED=$((COUNT_REMOVED + 1))
            else
                backup_file "$t"   # respalda antes de borrar
                rm -rf "$t"
                log_success "Eliminado: $t"
                COUNT_REMOVED=$((COUNT_REMOVED + 1))
            fi
        else
            COUNT_SKIPPED=$((COUNT_SKIPPED + 1))
        fi
    done

    log_success "Reset completado. Archivos eliminados: $COUNT_REMOVED (se respaldaron)."
    print_final_report
    exit 0
fi

timestamp() {
    date +%Y%m%d%H%M%S
}

BKP_DIR=""

# Calcula la ruta del backup dentro de bkp/, preservando la estructura de
# subcarpetas del archivo original para evitar colisiones de nombres.
backup_path() {
    target="$1"
    case "$target" in
        "$HARNESS_DIR"/*) rel="${target#"$HARNESS_DIR"/}" ;;
        "$SURFACE_DIR"/*) rel="surface/${target#"$SURFACE_DIR"/}" ;;
        /*) rel="external/${target#/}" ;;
        *) rel="${target#./}" ;;
    esac
    dest="$BKP_DIR/${rel}.bak.$(timestamp)"
    mkdir -p "$(dirname "$dest")"
    echo "$dest"
}

# Registra accion y (si no dry-run) hace backup real
backup_file() {
    target="$1"
    if [ "$FORCE" -eq 0 ] && [ -e "$target" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            log_info "[DRY-RUN] Backup de: $target"
            COUNT_BACKED_UP=$((COUNT_BACKED_UP + 1))
            return 0
        fi
        backup="$(backup_path "$target")"
        cp -p "$target" "$backup"
        log_info "Backup creado: $backup"
        COUNT_BACKED_UP=$((COUNT_BACKED_UP + 1))
    else
        COUNT_SKIPPED=$((COUNT_SKIPPED + 1))
    fi
}

archive_legacy_file() {
    target="$1"
    reason="$2"
    if [ -f "$target" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            log_info "[DRY-RUN] Archivaria (legacy): $target ($reason)"
            COUNT_BACKED_UP=$((COUNT_BACKED_UP + 1))
            return 0
        fi
        backup="$(backup_path "$target")"
        mv "$target" "$backup"
        log_info "$reason; archivado como $backup"
        COUNT_BACKED_UP=$((COUNT_BACKED_UP + 1))
    fi
}

write_file_notice() {
    local name="$1"
    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY-RUN] Escribiria: $name"
        COUNT_CREATED=$((COUNT_CREATED + 1))
    else
        log_success "   -> $name"
        COUNT_CREATED=$((COUNT_CREATED + 1))
    fi
}

# Helper para acciones idempotentes (mkdir, copy de assets, etc)
track_action() {
    local what="$1"
    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY-RUN] $what"
        COUNT_CREATED=$((COUNT_CREATED + 1))
    else
        COUNT_CREATED=$((COUNT_CREATED + 1))
    fi
}

json_value() {
    python3 -c 'import json, sys; print(json.dumps(sys.argv[1]))' "$1"
}

write_basic_agent_surface() {
    target="$1"
    mkdir -p "$(dirname "$target")"
    cat <<'SURFACE_EOF' > "$target"
# Harness Process

Estas operando en la raiz de un arnes multi-repo. Los hooks y launchers
inicializan el mapa compartido antes de trabajar:

```bash
bash "__HREL__init.sh"
bash "__HREL__harness_status.sh"
```

**⚠️ ANTES DE IMPLEMENTAR CÓDIGO (multi-LLM):**

0. Verifica frescura del plan (puede haber sido actualizado por Claude, Gemini,
   Antigravity, Grok, etc.):
   ```bash
   python3 "__HREL__harness.py" check-plan
   ```
   Si está desactualizado, re-lee el plan en `docs/` y sincroniza con `advance`.

Antes de modificar codigo:

1. Revisa el mapa: `python3 "__HREL__graph_memory.py" mapa`.
2. Revisa impacto: `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`.
3. Si existe `graphify-out/graph.json`, consulta `graphify query "<pregunta>"`.
4. Verifica plan: `python3 "__HREL__harness.py" check-plan`.
5. Trabaja y valida dentro del microservicio afectado.

El Hub usa exclusivamente PostgreSQL y graphify mantiene el grafo del codigo.
Para declarar dependencias usa:

```bash
python3 "__HREL__graph_memory.py" vincular \
  --microservicio <consumidor> \
  --destino <proyecto>/<servicio>
```

Commitea por microservicio con Conventional Commits. El cierre ejecuta
`__HREL__commit_guard.sh`; controla su severidad con
`HARNESS_COMMIT_GUARD_MODE=block|warn|off`.

Launchers disponibles: `bin/harness-claude`, `bin/harness-codex`,
`bin/harness-gemini`, `bin/harness-grok` y `bin/harness-antigravity`.

## Actualizacion

Para actualizar el harness (protocolo + herramientas) vuelve a la carpeta
fuente `harness_process` y ejecuta `./setup_harness.sh` (o con `--reset`).
Las superficies y scripts se regeneran desde la fuente.
SURFACE_EOF

    surface_tmp="$target.harness.tmp"
    sed -e "s|__HREL__|$HREL|g" "$target" > "$surface_tmp" && mv "$surface_tmp" "$target"
    write_file_notice "$(basename "$target") ($SURFACE_DIR)"
}

write_agent_surface() {
    target="$1"
    if [ "$WITH_SUBAGENTS" -eq 0 ]; then
        write_basic_agent_surface "$target"
        return
    fi
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

**⚠️ ANTES DE IMPLEMENTAR CÓDIGO (multi-LLM):**

0. Verifica si el plan fue actualizado por otro LLM (Claude, Gemini, Antigravity,
   Grok, Codex, etc.):
   ```bash
   python3 "__HREL__harness.py" check-plan
   ```
   - Si reporta **PLAN ACTUALIZADO POR OTRO LLM** → **DETENTE**, re-lee
     completamente el plan en `docs/plan-feature-*.md`.
   - Luego sincroniza: `python3 "__HREL__harness.py" advance --nota "Re-sincronizado con plan actualizado por otro agente"`.
   - Repite `check-plan` hasta que salga limpio.

Antes de tocar codigo, arquitectura o dependencias entre servicios, en ESTE
orden:

1. Revisa el mapa del hub:
   `python3 "__HREL__graph_memory.py" mapa`
2. Si vas a modificar un servicio, revisa su radio de impacto:
   `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
3. Si existe `graphify-out/graph.json`, consulta el grafo antes de leer a
   ciegas: `graphify query "<pregunta de la task>"`
4. **Verifica frescura del plan** (obligatorio antes de implementar):
   `python3 "__HREL__harness.py" check-plan`
5. Trabaja dentro del microservicio correspondiente; no programes en la raiz
   multi-repo salvo que la tarea sea del arnes.
6. Valida los servicios afectados. Los documentos durables (plan, investigacion,
   evidencia de implementacion/review) se guardan en `docs/` de la RAIZ del
   proyecto, no en chat ni en `__HREL__progress/` (solo estado vivo).
7. Al cerrar la feature usa
   `python3 "__HREL__harness.py" close --feature <id> --status <estado>`: mueve la
   task y las memorias juntas (registra el hub y refresca graphify). Luego corre
   `bash "__HREL__harness_check.sh"`.

Los comandos anteriores usan rutas relativas a la raiz multi-repo. Si estas
dentro de un microservicio, vuelve a la raiz o usa la ruta absoluta del arnes.

## Hub de memoria y graphify

Son sistemas separados:

- **Hub** (solo PostgreSQL; `HARNESS_HUB/.env` guarda su configuracion):
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
- **Verificar si el plan fue actualizado por otro LLM** (Claude/Gemini/Antigravity/Grok/etc.):
  `python3 "__HREL__harness.py" check-plan`
  **Obligatorio antes de implementar cualquier tarea.** Si detecta cambios,
  re-lee el plan en `docs/` y sincroniza con `advance`.
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

## Actualizacion del Harness Process

Este protocolo y las herramientas (`harness.py`, `check-plan`, roles, hooks, etc.)
viven en la carpeta `harness_process` (la fuente).

Para traer mejoras (nuevos recordatorios de planes actualizados por otros LLMs,
cambios en el protocolo, fixes, etc.):

1. Ve a la carpeta del harness_process.
2. Ejecuta `./setup_harness.sh` (o `./setup_harness.sh --reset` para limpiar antes).
3. Los archivos de instrucciones (CLAUDE.md, AGENTS.md, subagentes, etc.) y los
   scripts se regeneran automaticamente con la version actualizada.

No uses comandos magicos dentro del proyecto para actualizar. El proceso es
explicito: re-correr el instalador desde la fuente.

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
WITH_SUBAGENTS="__WITH_SUBAGENTS__"
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
    if [ "$WITH_SUBAGENTS" -eq 1 ]; then
        # Aviso no bloqueante si no hay feature activa.
        HARNESS_REPO_ROOT="$ROOT" python3 "$HARNESS_DIR/harness.py" nudge || true
    fi
    HARNESS_REPO_ROOT="$ROOT" bash "$HARNESS_DIR/harness_status.sh" --brief
}

run_stop() {
    if [ "$WITH_SUBAGENTS" -eq 1 ]; then
        # Checkpoint automatico de avance; harness_check conserva el exit code.
        HARNESS_REPO_ROOT="$ROOT" python3 "$HARNESS_DIR/harness.py" autocheck 1>&2 || true
        HARNESS_REPO_ROOT="$ROOT" bash "$HARNESS_DIR/harness_check.sh"
    else
        HARNESS_REPO_ROOT="$ROOT" bash "$HARNESS_DIR/commit_guard.sh"
    fi
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
    sed "s|__WITH_SUBAGENTS__|$WITH_SUBAGENTS|g" \
        "$SURFACE_DIR/bin/harness-hook" > "$hook_runtime_tmp" \
        && mv "$hook_runtime_tmp" "$SURFACE_DIR/bin/harness-hook"
    chmod +x "$SURFACE_DIR/bin/harness-hook"
    write_file_notice "bin/harness-hook ($SURFACE_DIR)"
}

write_codex_hooks() {
    mkdir -p "$SURFACE_DIR/.codex"
    codex_start_command="$(json_value "bash \"$SURFACE_DIR/bin/harness-hook\" codex-json session-start")"
    codex_post_command="$(json_value "bash \"$SURFACE_DIR/bin/harness-hook\" codex-json post-tool")"
    codex_stop_command="$(json_value "bash \"$SURFACE_DIR/bin/harness-hook\" codex-json stop")"
    cat <<CODEX_HOOKS_EOF > "$SURFACE_DIR/.codex/hooks.json"
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear|compact",
        "hooks": [
          {
            "type": "command",
            "command": $codex_start_command,
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
            "command": $codex_post_command,
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
            "command": $codex_stop_command,
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
    gemini_start_command="$(json_value "bash \"$SURFACE_DIR/bin/harness-hook\" gemini-json session-start")"
    gemini_post_command="$(json_value "bash \"$SURFACE_DIR/bin/harness-hook\" gemini-json post-tool")"
    gemini_stop_command="$(json_value "bash \"$SURFACE_DIR/bin/harness-hook\" gemini-json stop")"
    cat <<GEMINI_SETTINGS_EOF > "$SURFACE_DIR/.gemini/settings.json"
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
            "command": $gemini_start_command,
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
            "command": $gemini_post_command,
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
            "command": $gemini_stop_command,
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
# HARNESS_DIR : carpeta donde vive setup_harness.sh.
# REPO_ROOT   : raiz multi-repo (donde estan los microservicios). En 'subdir' es
#               el padre; en 'root' es el propio HARNESS_DIR.
# SURFACE_DIR : donde van CLAUDE.md, AGENTS.md, GEMINI.md, LLM.md y
#               .claude/settings.json (= REPO_ROOT).
# HARNESS_EXEC: prefijo historico para superficies Claude (sin llaves).
# HOOK_BASE   : prefijo para las rutas en .claude/settings.json (con llaves).
# HREL        : prefijo relativo de archivos del arnes vistos desde REPO_ROOT.
HARNESS_DIR="$(cd "$(dirname "$0")" && pwd -P)"
cd "$HARNESS_DIR"
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
BKP_DIR="${HARNESS_BKP_DIR:-$HARNESS_DIR/bkp}"

if [ -f "$HARNESS_DIR/templates/graph_memory.py" ]; then
    ASSET_DIR="$HARNESS_DIR/templates"
else
    ASSET_DIR="$HARNESS_DIR"
fi

required_assets=(
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
    required_assets+=(
        "CHECKPOINTS.md"
        "feature_list.json"
        "progress/current.md"
        "progress/history.md"
        "docs/architecture.md"
        "docs/conventions.md"
        "docs/verification.md"
        "roles/README.md"
        "roles/leader.md"
        "roles/implementer.md"
        "roles/reviewer.md"
    )
fi
for asset in "${required_assets[@]}"; do
    if [ ! -f "$ASSET_DIR/$asset" ]; then
        echo "[!] Falta el recurso requerido: $asset (buscado en $ASSET_DIR)" >&2
        exit 2
    fi
done

install_asset() {
    asset="$1"
    destination="${2:-$HARNESS_DIR/$asset}"
    source="$ASSET_DIR/$asset"
    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY-RUN] instalaria asset: $asset -> $destination"
        COUNT_CREATED=$((COUNT_CREATED + 1))
        return 0
    fi
    mkdir -p "$(dirname "$destination")"
    if [ "$source" != "$destination" ]; then
        cp "$source" "$destination"
    fi
    COUNT_CREATED=$((COUNT_CREATED + 1))
}

for command_name in bash cp git python3 sed; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        log_error "Comando requerido no disponible: $command_name"
        exit 2
    fi
done

# Preflight DB siempre se valida (incluso en dry-run, es barato y detecta config mala)
python3 - <<'EOF'
import os
import sys

hub_dir = os.environ.get("HARNESS_HUB") or os.path.join(
    os.path.expanduser("~"), ".harness-hub"
)
env_file = os.path.join(hub_dir, ".env")
if os.path.exists(env_file):
    with open(env_file, encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if line and not line.startswith("#") and "=" in line:
                key, _, value = line.partition("=")
                os.environ.setdefault(key.strip(), value.strip().strip("'\""))

required = ("DB_HOST", "DB_USER", "DB_PASSWORD")
missing = [key for key in required if not os.environ.get(key)]
if missing:
    print(
        "[!] PostgreSQL es el Hub predeterminado. Faltan variables: "
        + ", ".join(missing)
        + ". Configuralas en el entorno o en $HARNESS_HUB/.env.",
        file=sys.stderr,
    )
    sys.exit(2)
EOF

if [ "$DRY_RUN" -eq 1 ]; then
    log_warn "MODO DRY-RUN: no se realizaran escrituras ni instalaciones."
fi

log_info "== Instalando Harness Process en: $HARNESS_DIR =="
log_info "   proyecto:   $PROJECT_NAME"
log_info "   layout:     $LAYOUT$([ "$LAYOUT" = "subdir" ] && echo " (raiz multi-repo: $REPO_ROOT)")"
log_info "   superficies/hooks: Claude, Codex, Gemini, Grok, Antigravity, generica"
log_info "   subagentes: $([ "$WITH_SUBAGENTS" -eq 1 ] && echo si || echo no)"
log_info "   graphify:   $([ "$INSTALL_GRAPHIFY" -eq 1 ] && echo asegurar || echo no)"
log_info "   /graphify por agente: $([ "$INSTALL_GRAPHIFY_SKILLS" -eq 1 ] && echo "Claude/Codex/Gemini/Antigravity" || echo no)"
log_info "   antigravity: $([ "$INSTALL_ANTIGRAVITY" -eq 1 ] && echo "asegurar" || echo "no")"
log_info "   hub:        PostgreSQL (obligatorio)"
if [ -n "$LOG_FILE" ]; then
    log_info "   log-file:   $LOG_FILE"
fi

if [ "$LAYOUT" = "subdir" ] && [ "$REPO_ROOT" = "$HARNESS_DIR" ]; then
    log_error "--subdir requiere correr el instalador DESDE la subcarpeta del arnes, de modo que su padre sea la raiz multi-repo. Aborto."
    exit 2
fi

# mkdirs: siempre creamos dirs (barato y evita fallos downstream en dry-run simulado).
# En dry-run solo logueamos la intencion adicionalmente.
do_mkdir() {
    local d="$1"
    mkdir -p "$d"
    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY-RUN] mkdir -p $d"
    fi
    COUNT_CREATED=$((COUNT_CREATED + 1))
}

do_mkdir ".claude"
[ "$WITH_SUBAGENTS" -eq 1 ] && do_mkdir "roles" && do_mkdir "docs" && do_mkdir "progress"
do_mkdir "$SURFACE_DIR/.claude"
do_mkdir "$SURFACE_DIR/.codex"
do_mkdir "$SURFACE_DIR/.gemini"
do_mkdir "$SURFACE_DIR/.grok"
do_mkdir "$SURFACE_DIR/bin"
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    do_mkdir "$SURFACE_DIR/.claude/agents"
    do_mkdir "$SURFACE_DIR/.codex/agents"
    do_mkdir "$SURFACE_DIR/.gemini/agents"
fi

# Marcador de layout (respetando dry-run)
if [ "$DRY_RUN" -eq 1 ]; then
    log_info "[DRY-RUN] Escribiria marcadores .harness_layout y .harness_backend"
    COUNT_CREATED=$((COUNT_CREATED + 2))
else
    printf '%s\n' "$LAYOUT" > "$HARNESS_DIR/.harness_layout"
    printf 'postgres\n' > "$HARNESS_DIR/.harness_backend"
    track_action "layout markers"
fi

archive_legacy_file ".claudemd" ".claudemd es obsoleto; Claude Code lee CLAUDE.md"
archive_legacy_file "validate_aks.sh" "validate_aks.sh quedo obsoleto"
archive_legacy_file "$SURFACE_DIR/GROK.md" "GROK.md no se usa; Grok Build lee AGENTS.md/CLAUDE.md"
archive_legacy_file "$SURFACE_DIR/ANTIGRAVITY.md" "ANTIGRAVITY.md no se usa; Antigravity lee AGENTS.md/.agents/rules"

# Dry-run early exit: ya logueamos el plan (mkdirs, backups planeados, contadores). 
# Evitamos todo el codigo de generacion de contenido (cat heredoc, writes grandes).
if [ "$DRY_RUN" -eq 1 ]; then
    log_success "DRY-RUN: simulacion completa. Cero efectos en el filesystem de destino."
    print_final_report
    exit 0
fi

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

log_info "Generando .claude/settings.json..."
if [ "$DRY_RUN" -eq 1 ]; then
    log_info "[DRY-RUN] Se generarian .claude/settings.json + .codex + .gemini + .grok hooks + launchers + superficies LLM + subagentes"
    COUNT_CREATED=$((COUNT_CREATED + 20))
else
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
fi  # cierra el if DRY_RUN de la generacion de settings

echo "Instalando scripts desde: $ASSET_DIR"
install_asset "graph_memory.py"
install_asset "init.sh"
install_asset "validate_ui.sh"
install_asset "debug_ui.js"
install_asset "commit_guard.sh"
install_asset "harness_status.sh"
install_asset "harness_check.sh"
install_asset "harness.py"
write_file_notice "scripts base ($HARNESS_DIR)"

echo "Asegurando permisos de ejecucion en HARNESS_DIR..."
chmod +x "$HARNESS_DIR/graph_memory.py"
chmod +x "$HARNESS_DIR/init.sh"
chmod +x "$HARNESS_DIR/validate_ui.sh"
chmod +x "$HARNESS_DIR/harness_status.sh"
chmod +x "$HARNESS_DIR/commit_guard.sh"
chmod +x "$HARNESS_DIR/harness_check.sh"
chmod +x "$HARNESS_DIR/harness.py"

echo "Generando capa de subagentes (mapa de agentes)..."
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
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

    install_asset "roles/leader.md"
    install_asset "roles/implementer.md"
    install_asset "roles/reviewer.md"
    install_asset "roles/README.md"

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

    install_asset "CHECKPOINTS.md"

    # Backlog vivo: solo se siembra si falta. Un reinstall NO debe vaciar las
    # features ya cargadas.
    if [ ! -f feature_list.json ]; then
        install_asset "feature_list.json"
    fi

    # Estado vivo: solo se siembra si falta. Un reinstall NO debe pisar la tarea
    # en curso ni la bitacora ya escrita.
    if [ ! -f progress/current.md ]; then
        install_asset "progress/current.md"
    fi

    if [ ! -f progress/history.md ]; then
        install_asset "progress/history.md"
    fi

    install_asset "docs/architecture.md"
    install_asset "docs/conventions.md"
    install_asset "docs/verification.md"

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

log_info "Asegurando graphify..."
if command -v graphify >/dev/null 2>&1; then
    log_success "   -> graphify ya esta disponible."
    COUNT_INSTALLED=$((COUNT_INSTALLED + 1))
elif [ "$INSTALL_GRAPHIFY" -eq 1 ]; then
    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY-RUN] Se instalaria graphify (graphifyy via uv/pipx/pip --user)"
        COUNT_INSTALLED=$((COUNT_INSTALLED + 1))
    else
        local installed_via=""
        if retry_cmd command -v uv >/dev/null 2>&1; then
            if retry_cmd uv tool install --upgrade graphifyy >/dev/null 2>&1; then
                installed_via="uv"
            fi
        elif retry_cmd command -v pipx >/dev/null 2>&1; then
            if retry_cmd pipx install graphifyy >/dev/null 2>&1; then
                installed_via="pipx"
            fi
        fi
        if [ -z "$installed_via" ]; then
            if retry_cmd python3 -m pip install --user graphifyy >/dev/null 2>&1; then
                installed_via="pip-user"
            fi
        fi
        if [ -n "$installed_via" ]; then
            log_success "   -> graphify instalado via $installed_via."
            COUNT_INSTALLED=$((COUNT_INSTALLED + 1))
            # PATH guidance (mejora clave)
            if [[ ":$PATH:" != *":$HOME/.local/bin:"* && ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
                log_warn "   graphify instalado con --user. Agrega a PATH si no esta:"
                log_warn "     export PATH=\"\$HOME/.local/bin:\$PATH\""
            fi
        else
            log_warn "   -> aviso: no se pudo instalar graphify automaticamente. Instala manualmente: pipx/uv/pip install graphifyy"
        fi
    fi
else
    log_info "   -> graphify no instalado (--no-graphify activo)."
fi

log_info "Asegurando psycopg2 para el Hub en PostgreSQL..."
if python3 -c "import psycopg2" >/dev/null 2>&1; then
    log_success "   -> psycopg2 ya esta disponible."
else
    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY-RUN] Se instalaria psycopg2-binary via pip --user"
        COUNT_INSTALLED=$((COUNT_INSTALLED + 1))
    else
        if retry_cmd python3 -m pip install --user psycopg2-binary >/dev/null 2>&1; then
            log_success "   -> psycopg2-binary instalado via pip --user."
            COUNT_INSTALLED=$((COUNT_INSTALLED + 1))
            if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
                log_warn "   psycopg2 via --user. Asegura PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
            fi
        else
            log_error "No se pudo instalar psycopg2-binary."
            exit 1
        fi
    fi
fi

log_info "Verificando y migrando el Memory Hub PostgreSQL..."
python3 - << 'EOF'
import json
import os
import sys
from pathlib import Path

try:
    import psycopg2
    import psycopg2.extensions
    from psycopg2 import sql
    from psycopg2.extras import Json
except ImportError:
    print("[!] psycopg2 no esta disponible.", file=sys.stderr)
    sys.exit(1)

hub_dir = Path(
    os.environ.get("HARNESS_HUB")
    or os.path.join(os.path.expanduser("~"), ".harness-hub")
)
env_file = hub_dir / ".env"
if env_file.exists():
    with env_file.open(encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if line and not line.startswith("#") and "=" in line:
                key, _, value = line.partition("=")
                os.environ.setdefault(key.strip(), value.strip().strip("'\""))

required = ("DB_HOST", "DB_USER", "DB_PASSWORD")
missing = [key for key in required if not os.environ.get(key)]
if missing:
    print("[!] PostgreSQL requiere: " + ", ".join(missing), file=sys.stderr)
    sys.exit(2)

db_name = os.environ.get("DB_NAME", "postgres")
connection = {
    "user": os.environ["DB_USER"],
    "password": os.environ["DB_PASSWORD"],
    "host": os.environ["DB_HOST"],
    "port": os.environ.get("DB_PORT", "5432"),
    "sslmode": os.environ.get("DB_SSL_MODE", "require"),
    "connect_timeout": 10,
}

try:
    conn = psycopg2.connect(dbname=db_name, **connection)
except psycopg2.OperationalError as exc:
    if "does not exist" not in str(exc) and "no existe" not in str(exc).lower():
        print("[!] Fallo de conexion PostgreSQL: " + str(exc).strip(), file=sys.stderr)
        sys.exit(1)
    try:
        print("   -> Creando base de datos " + db_name + "...")
        admin = psycopg2.connect(dbname="postgres", **connection)
        admin.set_isolation_level(psycopg2.extensions.ISOLATION_LEVEL_AUTOCOMMIT)
        with admin.cursor() as cur:
            cur.execute(sql.SQL("CREATE DATABASE {}").format(sql.Identifier(db_name)))
        admin.close()
        conn = psycopg2.connect(dbname=db_name, **connection)
    except Exception as create_exc:
        print("[!] Fallo al crear base de datos: " + str(create_exc), file=sys.stderr)
        sys.exit(1)

legacy_nodes = {}
legacy_edges = []
graph_file = hub_dir / "graph_db.json"
if graph_file.exists():
    try:
        data = json.loads(graph_file.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print("[!] No se puede migrar graph_db.json: " + str(exc), file=sys.stderr)
        conn.close()
        sys.exit(1)
    if not isinstance(data, dict):
        print("[!] graph_db.json no contiene un objeto JSON valido.", file=sys.stderr)
        conn.close()
        sys.exit(1)
    legacy_nodes.update(data.get("nodes") or {})
    legacy_edges.extend(data.get("edges") or [])

progress_dir = hub_dir / "progress"
if progress_dir.exists():
    for progress_file in progress_dir.rglob("*.json"):
        try:
            node = json.loads(progress_file.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            print(
                "[!] No se puede migrar " + str(progress_file) + ": " + str(exc),
                file=sys.stderr,
            )
            conn.close()
            sys.exit(1)
        if not isinstance(node, dict) or not node.get("_id"):
            continue
        node_id = node["_id"]
        merged = dict(node)
        merged.update(legacy_nodes.get(node_id) or {})
        legacy_nodes[node_id] = merged

with conn:
    with conn.cursor() as cur:
        cur.execute(
            """
            CREATE TABLE IF NOT EXISTS graph_nodes (
                id TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                props JSONB NOT NULL DEFAULT '{}'::jsonb
            );
            """
        )
        cur.execute(
            """
            CREATE TABLE IF NOT EXISTS graph_edges (
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                type TEXT NOT NULL,
                props JSONB NOT NULL DEFAULT '{}'::jsonb,
                PRIMARY KEY (source, target, type)
            );
            """
        )
        for node_id, raw_props in legacy_nodes.items():
            props = dict(raw_props or {})
            label = props.pop("_label", "Artefacto")
            props.setdefault("_id", node_id)
            cur.execute(
                """
                INSERT INTO graph_nodes (id, label, props)
                VALUES (%s, %s, %s)
                ON CONFLICT (id) DO UPDATE SET
                    label = EXCLUDED.label,
                    props = graph_nodes.props || EXCLUDED.props;
                """,
                (node_id, label, Json(props)),
            )
        for edge in legacy_edges:
            props = dict(edge)
            source = props.pop("source")
            target = props.pop("target")
            edge_type = props.pop("type")
            cur.execute(
                """
                INSERT INTO graph_edges (source, target, type, props)
                VALUES (%s, %s, %s, %s)
                ON CONFLICT (source, target, type) DO UPDATE SET
                    props = graph_edges.props || EXCLUDED.props;
                """,
                (source, target, edge_type, Json(props)),
            )

conn.close()
print("   -> Base de datos " + db_name + " lista.")
if legacy_nodes or legacy_edges:
    print(
        "   -> Memoria local migrada: "
        + str(len(legacy_nodes))
        + " nodos, "
        + str(len(legacy_edges))
        + " relaciones."
    )
EOF

archive_local_hub_memory() {
    hub_dir="${HARNESS_HUB:-$HOME/.harness-hub}"
    graph_file="$hub_dir/graph_db.json"
    progress_dir="$hub_dir/progress"
    if [ ! -f "$graph_file" ] && [ ! -d "$progress_dir" ]; then
        return
    fi

    memory_backup="$BKP_DIR/memory-hub/$(timestamp)-$$"
    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY-RUN] Archivaria memoria local hub en $memory_backup"
        COUNT_BACKED_UP=$((COUNT_BACKED_UP + 1))
        return
    fi
    mkdir -p "$memory_backup"
    if [ -f "$graph_file" ]; then
        cp -p "$graph_file" "$memory_backup/graph_db.json"
    fi
    if [ -d "$progress_dir" ]; then
        cp -R "$progress_dir" "$memory_backup/progress"
    fi

    rm -f "$graph_file"
    rm -rf "$progress_dir"
    log_info "Memoria local respaldada en $memory_backup y eliminada del Hub activo."
    COUNT_BACKED_UP=$((COUNT_BACKED_UP + 1))
}

archive_local_hub_memory

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
    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "[DRY-RUN] Se desplegaria /graphify para claude/codex/antigravity (tmp aislado)"
        COUNT_CREATED=$((COUNT_CREATED + 1))
    else
        log_info "Desplegando el comando /graphify por agente..."
        gx_tmp="$(mktemp -d 2>/dev/null || echo "${TMPDIR:-/tmp}/harness-graphify.$$")"
        register_temp "$gx_tmp"
        mkdir -p "$gx_tmp"
        for gx_plat in claude codex antigravity; do
            if ( cd "$gx_tmp" && graphify install --platform "$gx_plat" ) >/dev/null 2>&1; then
                log_success "   -> /graphify disponible en $gx_plat."
            else
                log_warn "   -> aviso: no se pudo desplegar /graphify en $gx_plat."
            fi
        done
        log_info "   -> Grok: sin plataforma propia; usa el CLI ('graphify update .' / 'graphify query')."
    fi
elif [ "$INSTALL_GRAPHIFY_SKILLS" -eq 0 ]; then
    log_info "   -> Comando /graphify por agente omitido (--no-graphify-skills)."
fi

ensure_antigravity_cli

echo ""
log_success "========================================================"
log_success "Harness Process instalado exitosamente (layout: $LAYOUT)."
log_success "========================================================"
log_info ""
log_info "Superficies multi-LLM escritas en la raiz:"
log_info "  $SURFACE_DIR/CLAUDE.md"
log_info "  $SURFACE_DIR/AGENTS.md"
log_info "  $SURFACE_DIR/GEMINI.md"
log_info "  $SURFACE_DIR/LLM.md"
log_info "  $SURFACE_DIR/.claude/settings.json (hooks automaticos para Claude Code)"
log_info "  $SURFACE_DIR/.codex/hooks.json (hooks automaticos para Codex; confiar con /hooks)"
log_info "  $SURFACE_DIR/.gemini/settings.json (hooks automaticos para Gemini CLI)"
log_info "  $SURFACE_DIR/.grok/hooks/harness.sh (hooks automaticos para Grok; confiar con /hooks-trust)"
log_info "  $SURFACE_DIR/bin/harness-claude|codex|gemini|grok|antigravity"
if [ "$LAYOUT" = "subdir" ]; then
    log_info ""
    log_info "Scripts del arnes en: $HARNESS_DIR"
    log_info "IMPORTANTE: lanza tu agente DESDE la raiz ($REPO_ROOT) para que"
    log_info "descubra la superficie correspondiente."
fi
log_info ""
log_info "Comandos utiles:"
log_info "  bash ${HREL}init.sh"
log_info "  bash ${HREL}harness_status.sh"
log_info "  bash ${HREL}harness_check.sh"
log_info "  python3 ${HREL}graph_memory.py mapa"
log_info "  python3 ${HREL}harness.py status"
log_info "  python3 ${HREL}harness.py check-plan     # <-- OBLIGATORIO antes de implementar (detecta planes actualizados por otros LLMs)"
log_info "  bin/harness-codex"
log_info "  bin/harness-gemini"
log_info "  bin/harness-grok"
log_info "  bin/harness-antigravity"
log_info "  /graphify           (comando nativo en Claude/Codex/Gemini/Antigravity)"
log_info "  graphify query \"...\"  (CLI; funciona en cualquier agente, incl. Grok)"
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    log_info ""
    log_info "Modo subagentes activo:"
    log_info "  Mapa de agentes:    ${HREL}roles/README.md"
    log_info "  Subagentes nativos: .claude/agents/*.md, .codex/agents/*.toml, .gemini/agents/*.md"
    log_info "  Grok Build:         lee .claude/agents/*.md (compat Claude Code)"
    log_info "  Antigravity/otros:  ${HREL}roles/*.md como fases secuenciales"
    log_info "  python3 ${HREL}harness.py add --name \"mi_feature\" --service \"$PROJECT_NAME/servicio\""
    log_info "  python3 ${HREL}harness.py start --feature 1"
    log_info "  python3 ${HREL}harness.py check-plan     # verifica si otro LLM actualizo el plan"
    log_info "  python3 ${HREL}harness.py close --feature 1 --status done"
fi

log_info ""
log_info "Actualizacion futura:"
log_info "  Para traer mejoras (nuevo protocolo, recordatorios de planes actualizados por otros LLMs,"
log_info "  nuevos comandos, etc.) vuelve a ejecutar este instalador desde la carpeta fuente:"
log_info "    cd /ruta/al/harness_process"
log_info "    ./setup_harness.sh"
log_info "  O usa --reset para una reinstalacion limpia de las superficies."

print_final_report
