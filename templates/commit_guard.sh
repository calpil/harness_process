#!/bin/sh
# harness-managed-hook v4
# Uso normal: COMO hook. El agente manda el JSON del evento por la entrada y de
# ahi sale el unico dato que se lee, `stop_hook_active`. Con stdin en una
# terminal no hay JSON que esperar y `cat` se quedaria pidiendo un EOF que nadie
# va a escribir: el guard se cuelga en vez de fallar (feature #52). Quien no es
# un hook —harness_check.sh— cierra la entrada y pasa el dato por entorno.
if [ -t 0 ]; then
    INPUT=""
else
    INPUT=$(cat 2>/dev/null)
fi
MODE="${HARNESS_COMMIT_GUARD_MODE:-block}" # block | warn | off

[ "$MODE" = "off" ] && exit 0

STOP_HOOK_ACTIVE=0
[ "${HARNESS_STOP_HOOK_ACTIVE:-0}" = "1" ] && STOP_HOOK_ACTIVE=1
printf '%s' "$INPUT" | grep -q '"stop_hook_active"[[:space:]]*:[[:space:]]*true' && STOP_HOOK_ACTIVE=1

HARNESS_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
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
# Feature #58: el arnes no se bloquea a si mismo.
#
# El guard trata como microservicio a todo subdirectorio que sea un repo git, y
# en un proyecto donde `docs/` es su PROPIO repo eso incluye los 137 artefactos
# que el arnes escribe ahi. Resultado: cada start/advance/prd apply terminaba el
# turno pidiendo "commit por microservicio" de archivos que el propio `close` va
# a commitear, y la salida facil era apagar el guard tambien para el codigo.
#
# La regla ya estaba escrita en docs/rutas-protegidas.md ("la proteccion es
# contra las herramientas del agente, no contra el binario"); aca se aplica.
# La exencion es POR ARTEFACTO, no por carpeta (decision del usuario): alcanza
# UN archivo que no sea del arnes para que el repo vuelva a contar como sucio.
es_artefacto_del_arnes() {
    # $1 ruta relativa al repo, $2 basename del repo.
    #
    # No alcanza con el nombre: los artefactos del arnes viven en `docs/`, o sea
    # que la ruta arranca con `docs/` o el repo sucio ES el `docs/`. Sin esta
    # condicion, un `impl-notas.md` dentro de un microservicio se eximia como si
    # fuera del arnes y el guard dejaba de mirar un documento real (encontrado
    # intentando romper esta misma feature).
    case "$1" in
        docs/*) ruta="${1#docs/}" ;;
        *)
            [ "$2" = "docs" ] || return 1
            ruta="$1"
            ;;
    esac
    case "$ruta" in
        spec-feature-*.md|plan-feature-*.md|impl-*.md|review-*.md|verify-*.md) return 0 ;;
        estado-feature-*.md|prd-diff-*.md) return 0 ;;
        prd/*|lecciones/*) return 0 ;;
        architecture.md|perfil-usuario.md) return 0 ;;
    esac
    return 1
}

# 0 solo si hubo cambios y TODOS son artefactos del arnes.
# Feature #66: ademas de decidir, junta los archivos que NO estan exentos. El
# mensaje nombraba solo el repo ("Cambios sin commitear en: docs"), y con eso el
# agente no puede saber si lo sucio es suyo: la unica salida que le quedaba era
# commitear a ciegas trabajo que puede ser de otra sesion. Ver
# docs/lecciones/remedios-que-la-herramienta-sugiere.md.
AJENOS=""
solo_artefactos_del_arnes() {
    hubo=1
    ajenos_del_repo=""
    while IFS= read -r linea; do
        [ -n "$linea" ] || continue
        ruta=${linea#???}                                   # XY + espacio
        case "$ruta" in *" -> "*) ruta=${ruta##* -> } ;; esac  # renombrados
        ruta=${ruta#\"}
        ruta=${ruta%\"}
        if es_artefacto_del_arnes "$ruta" "$(basename "$1")"; then
            hubo=0
        else
            ajenos_del_repo="$ajenos_del_repo
    $(basename "$1")/$ruta"
        fi
    done <<EOF
$(git -C "$1" status --porcelain 2>/dev/null)
EOF
    if [ -n "$ajenos_del_repo" ]; then
        AJENOS="$AJENOS$ajenos_del_repo"
        return 1
    fi
    return $hubo
}

DIRTY=""
for repo in "$REPO_ROOT"/*; do
    [ -d "$repo" ] || continue
    repo_abs=$(cd "$repo" && pwd -P)
    [ "$repo_abs" = "$HARNESS_DIR" ] && continue
    # ¿Es este dir la RAIZ de un repo, y no un subdirectorio de uno? Se le
    # pregunta a git en vez de comparar rutas: en Git Bash `pwd -P` devuelve
    # /c/Users/... y `rev-parse --show-toplevel` devuelve C:/Users/..., asi que
    # la comparacion de textos NUNCA daba igual y el guard no encontraba ningun
    # repo en Windows: salia verde sin haber mirado nada. `--show-prefix` vacio
    # dice lo mismo sin depender de la forma de la ruta.
    git -C "$repo" rev-parse --is-inside-work-tree >/dev/null 2>&1 || continue
    [ -z "$(git -C "$repo" rev-parse --show-prefix 2>/dev/null)" ] || continue
    if [ -n "$(git -C "$repo" status --porcelain 2>/dev/null)" ]; then
        if solo_artefactos_del_arnes "$repo"; then
            # Se dice: un guard que se calla en silencio es como no tenerlo.
            echo "[i] $(basename "$repo"): solo artefactos del arnes sin commitear (los commitea 'close'); no cuenta como sucio."
        else
            DIRTY="$DIRTY $(basename "$repo")"
        fi
    fi
done

if [ -n "$DIRTY" ]; then
    echo "Cambios sin commitear en:$DIRTY" >&2
    if [ -n "$AJENOS" ]; then
        echo "Archivos que no son artefactos del arnes:$AJENOS" >&2
    fi
    {
        echo "Tres salidas, en este orden:"
        echo "  1. Si el cambio es TUYO: commitealo por microservicio con Conventional Commits."
        echo "  2. Si NO es tuyo (otra sesion, otro repo, trabajo del usuario): NO lo commitees."
        echo "     Decilo y segui: commitear trabajo ajeno a medio hacer es peor que dejarlo sucio."
        echo "  3. Solo si molesta de forma sostenida: HARNESS_COMMIT_GUARD_MODE=warn"
        echo "     (avisa sin bloquear). \`off\` apaga el guard para TODO el repo, incluido tu codigo."
    } >&2
    if [ "$MODE" = "warn" ] || [ "$STOP_HOOK_ACTIVE" -eq 1 ]; then
        exit 0
    fi
    exit 2
fi
exit 0
