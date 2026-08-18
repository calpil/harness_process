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

# Integridad del arbol de PRDs anidados (docs/prd/ de la RAIZ). La identidad de
# un PRD es su cadena de segmentos: la carpeta lleva el segmento propio y el
# archivo la cadena completa (docs/prd/cobranza/mora/PRD-cobranza-mora.md). El
# FILESYSTEM es la fuente de verdad; el `Padre:` del encabezado es una
# declaracion que se contrasta contra la ubicacion real. Sin docs/prd/ (proyecto
# sin PRDs o instalacion minima) el bloque entero se omite.
prd_root="$REPO_ROOT/docs/prd"
if [ -d "$prd_root" ]; then
    # Nombre de archivo esperado para una cadena de segmentos ("" -> master).
    prd_expected_file() {
        if [ -z "$1" ]; then
            echo "PRD-master.md"
        else
            echo "PRD-$(printf '%s' "$1" | tr '/' '-').md"
        fi
    }

    # 1) Cada PRD-*.md tiene que estar donde dice su cadena.
    while IFS= read -r prd_file; do
        [ -z "$prd_file" ] && continue
        prd_rel="${prd_file#"$prd_root"/}"
        prd_base="$(basename "$prd_rel")"
        prd_chain="$(dirname "$prd_rel")"
        [ "$prd_chain" = "." ] && prd_chain=""
        prd_want="$(prd_expected_file "$prd_chain")"
        if [ "$prd_base" != "$prd_want" ]; then
            echo "[!] PRD fuera de lugar: docs/prd/$prd_rel deberia llamarse $prd_want segun su carpeta. Movelo a docs/prd/$(printf '%s' "${prd_base#PRD-}" | sed 's/\.md$//' | tr '-' '/')/$prd_base o renombralo." >&2
            failures=$((failures + 1))
            continue
        fi
        # 3) El `Padre:` declarado tiene que coincidir con la ubicacion real.
        prd_declared="$(sed -n '1,15p' "$prd_file" | sed -n 's/^Padre:[[:space:]]*//p' | head -n 1)"
        if [ -n "$prd_declared" ]; then
            prd_parent="$(dirname "$prd_chain")"
            if [ "$prd_parent" = "." ] || [ -z "$prd_chain" ]; then
                prd_parent="master"
            fi
            if [ "$prd_declared" != "$prd_parent" ]; then
                echo "[!] docs/prd/$prd_rel declara 'Padre: $prd_declared' pero su lugar en el arbol dice '$prd_parent'. Corregi el encabezado o mové el archivo." >&2
                failures=$((failures + 1))
            fi
        fi
        # 5) Un PRD sin hitos avisa, pero NO bloquea: puede estar recien creado.
        prd_hitos="$(awk -F'|' '
            /^## 10\. Hitos -> features/ { inside = 1; next }
            /^## / { inside = 0 }
            inside && /^\|/ {
                cell = $3
                gsub(/^[ \t]+|[ \t]+$/, "", cell)
                if (cell == "Hito" || cell ~ /^-+$/ || cell == "") next
                if (cell ~ /^</ && cell ~ />$/) next
                n++
            }
            END { print n + 0 }' "$prd_file")"
        if [ "$prd_hitos" -eq 0 ]; then
            echo "[i] docs/prd/$prd_rel no declara hitos todavia (tabla '10. Hitos -> features' vacia): ninguna feature puede salir de el." >&2
        fi
    done <<EOF
$(find "$prd_root" -type f -name 'PRD-*.md' | sort)
EOF

    # 2) Cada carpeta del arbol tiene que contener su PRD.
    while IFS= read -r prd_dir; do
        [ -z "$prd_dir" ] && continue
        prd_rel="${prd_dir#"$prd_root"/}"
        prd_want="$(prd_expected_file "$prd_rel")"
        if [ ! -f "$prd_dir/$prd_want" ]; then
            echo "[!] docs/prd/$prd_rel no contiene su $prd_want: es una carpeta del arbol sin PRD. Crealo con 'sh harness_cli prd add' o borra la carpeta." >&2
            failures=$((failures + 1))
        fi
    done <<EOF
$(find "$prd_root" -mindepth 1 -type d | sort)
EOF

    # 4) Ninguna feature puede apuntar a un PRD que no existe.
    if [ -f "$HARNESS_DIR/feature_list.json" ]; then
        while IFS= read -r prd_ref; do
            [ -z "$prd_ref" ] && continue
            if [ "$prd_ref" = "master" ]; then
                prd_target="$prd_root/PRD-master.md"
            else
                prd_target="$prd_root/$prd_ref/$(prd_expected_file "$prd_ref")"
            fi
            if [ ! -f "$prd_target" ]; then
                echo "[!] feature_list.json declara 'prd: $prd_ref' y ese PRD no existe. Crealo con 'sh harness_cli prd add' o corregi la feature." >&2
                failures=$((failures + 1))
            fi
        done <<EOF
$(grep -o '"prd"[[:space:]]*:[[:space:]]*"[^"]*"' "$HARNESS_DIR/feature_list.json" 2>/dev/null | sed -E 's/.*"([^"]*)"$/\1/' | sort -u)
EOF
    fi
fi

# Integridad de las lecciones (docs/lecciones/ de la RAIZ, feature #17). Una
# leccion es memoria procedural del proyecto y su identidad es su nombre de
# clase: el frontmatter tiene que ser legible y su `nombre:` tiene que coincidir
# con el archivo, porque es lo que hace que se encuentre. Frontmatter roto o
# nombre que no coincide BLOQUEAN (decision usuario 2026-08-16, OBS-4 de la #17);
# la falta de `triggers` solo avisa. Sin docs/lecciones/ el bloque se omite.
lec_root="$REPO_ROOT/docs/lecciones"
if [ -d "$lec_root" ]; then
    while IFS= read -r lec_file; do
        [ -z "$lec_file" ] && continue
        lec_base="$(basename "$lec_file")"
        # La guia es plantilla del arnes, no una leccion.
        [ "$lec_base" = "COMO-ESCRIBIR-UNA-LECCION.md" ] && continue
        lec_name="${lec_base%.md}"
        if [ "$(head -n 1 "$lec_file")" != "---" ]; then
            echo "[!] docs/lecciones/$lec_base no empieza con el frontmatter '---'. Formato en docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md." >&2
            failures=$((failures + 1))
            continue
        fi
        lec_close="$(awk 'NR > 1 && /^---[[:space:]]*$/ { print NR; exit }' "$lec_file")"
        if [ -z "$lec_close" ]; then
            echo "[!] docs/lecciones/$lec_base tiene el frontmatter sin cerrar (falta el '---' de cierre)." >&2
            failures=$((failures + 1))
            continue
        fi
        lec_head="$(sed -n "2,${lec_close}p" "$lec_file")"
        lec_decl="$(printf '%s\n' "$lec_head" | sed -n 's/^nombre:[[:space:]]*//p' | head -n 1)"
        if [ -z "$lec_decl" ]; then
            echo "[!] docs/lecciones/$lec_base no declara 'nombre:' en su frontmatter." >&2
            failures=$((failures + 1))
        elif [ "$lec_decl" != "$lec_name" ]; then
            echo "[!] docs/lecciones/$lec_base declara 'nombre: $lec_decl' y el archivo se llama '$lec_name.md'. Corregi el frontmatter o renombra el archivo." >&2
            failures=$((failures + 1))
        fi
        lec_trig="$(printf '%s\n' "$lec_head" | sed -n 's/^triggers:[[:space:]]*//p' | head -n 1 | tr -d '[] ')"
        if [ -z "$lec_trig" ]; then
            echo "[i] docs/lecciones/$lec_base no declara 'triggers': nadie la va a encontrar por tema." >&2
        fi
    done <<EOF
$(find "$lec_root" -maxdepth 2 -type f -name '*.md' | sort)
EOF
fi

# Integridad del perfil de usuario (docs/perfil-usuario.md, feature #19). El
# limite es duro porque el perfil se INYECTA en el prompt de cada agente: si
# crece sin control, el costo se paga en todas las sesiones de todos los
# backends. La validacion vive en el binario (`perfil check`) y no aca: contar
# caracteres UTF-8 en awk es poco confiable y el limite tiene que ser EXACTAMENTE
# el mismo que aplica al escribir. Sin el archivo, `perfil check` calla y sale 0.
#
# Tolerancia a instalacion parcial: quien hace `git pull` sin re-correr el
# instalador se queda con este script nuevo y el binario viejo, que no conoce
# `perfil check`. Eso NO puede reportarse como un problema del perfil: se detecta
# el "unrecognized subcommand" y se avisa con `[i]` nombrando el remedio.
if [ -f "$REPO_ROOT/docs/perfil-usuario.md" ]; then
    perfil_out="$(sh "$HARNESS_DIR/harness_cli" perfil check 2>&1)" && perfil_rc=0 || perfil_rc=$?
    if [ "$perfil_rc" -ne 0 ]; then
        case "$perfil_out" in
            *"unrecognized subcommand"*|*"invalid value"*)
                echo "[i] El binario instalado no conoce 'perfil check' (es anterior a la feature #19): re-corre el instalador para validar docs/perfil-usuario.md." >&2
                ;;
            *)
                printf '%s\n' "$perfil_out" >&2
                failures=$((failures + 1))
                ;;
        esac
    elif [ -n "$perfil_out" ]; then
        printf '%s\n' "$perfil_out" >&2
    fi
fi

# Regla de tests: prohibido leer el codigo fuente en un test (feature #24,
# docs/conventions.md). Un test que lee el texto de un .rs/.sh/.ps1 prueba la
# FORMA del codigo: pasa con la implementacion rota y falla ante un refactor
# correcto.
#
# AVISA Y NO BLOQUEA (decision usuario 2026-08-17, OBS-2 de la #24): la regla
# admite la excepcion de "dato de entrada" y un gate duro empujaria a inventar
# un --force, que es peor que el aviso. Por eso NO toca `failures`.
#
# Las otras dos reglas (snapshots, detector-de-cambios) no se chequean solas:
# saber que dato "se espera que cambie" no se grepea. Las verifica el reviewer.
#
# Sin rust/tests/ el bloque se omite entero: un proyecto que no es Rust no ve
# ninguna diferencia.
conv_tests="$REPO_ROOT/rust/tests"
if [ -d "$conv_tests" ]; then
    conv_hits="$(grep -nE 'read_to_string\([^)]*\.(rs|sh|ps1)"' "$conv_tests"/*.rs 2>/dev/null || true)"
    if [ -n "$conv_hits" ]; then
        printf '%s\n' "$conv_hits" | while IFS= read -r conv_line; do
            [ -z "$conv_line" ] && continue
            conv_file="${conv_line%%:*}"
            conv_rest="${conv_line#*:}"
            conv_num="${conv_rest%%:*}"
            # El nombre del test es el ultimo `fn ...` antes de esa linea.
            conv_fn="$(head -n "$conv_num" "$conv_file" \
                | grep -E '^(pub )?fn [a-z_0-9]+' \
                | tail -n 1 \
                | sed -E 's/^(pub )?fn ([a-z_0-9]+).*/\2/')"
            [ -z "$conv_fn" ] && conv_fn="(fuera de un test)"
            echo "[i] rust/tests/$(basename "$conv_file"):$conv_num ($conv_fn) lee un archivo fuente. Regla 2 de docs/conventions.md: prohibido leer el codigo fuente en un test. Si es dato de entrada del codigo bajo prueba, dejalo dicho en el test." >&2
        done
    fi
fi

# Rutas protegidas (feature #26, docs/rutas-protegidas.md). Red de seguridad: la
# ultima de las tres capas. No actua en el momento del dano —para eso estan el
# PreToolUse y el PostToolUse— pero impide cerrar con una ruta protegida tocada.
#
# BLOQUEA (decision usuario 2026-08-17, OBS-4): a diferencia del aviso de
# convenciones de la #24, aca no hay excepcion legitima. Nadie deberia estar
# editando un PRD o la constitution desde un agente. La valvula sigue siendo
# HARNESS_CHECK_MODE=warn.
#
# Las escrituras del PROPIO arnes (el hito que marca `close`, el PRD que crea
# `prd add`) quedan exentas: el binario las registra al hacerlas. Sin eso, el
# arnes se reportaria a si mismo despues de cada cierre.
#
# Tolerancia a instalacion parcial: binario viejo que no conoce `rutas` -> se
# avisa con [i] y no se cuenta como fallo (mismo patron que `perfil check`).
# stdout y stderr POR SEPARADO: el binario emite avisos informativos por stderr
# (la resolucion de raiz, por ejemplo) y mezclarlos hacia que una linea de aviso
# apareciera como si fuera una ruta violada.
rutas_err="$(mktemp "${TMPDIR:-/tmp}/harness-rutas.XXXXXX")"
rutas_out="$(sh "$HARNESS_DIR/harness_cli" rutas --violaciones 2>"$rutas_err")" && rutas_rc=0 || rutas_rc=$?
rutas_err_text="$(cat "$rutas_err" 2>/dev/null || true)"
rm -f "$rutas_err"
if [ "$rutas_rc" -ne 0 ]; then
    case "$rutas_err_text" in
        *"unrecognized subcommand"*|*"invalid value"*)
            echo "[i] El binario instalado no conoce 'rutas' (es anterior a la feature #26): re-corre el instalador para activar las rutas protegidas." >&2
            ;;
        *)
            echo "[!] Rutas PROTEGIDAS modificadas y sin commitear:" >&2
            printf '%s\n' "$rutas_out" | while IFS="$(printf '\t')" read -r rp_ruta rp_remedio; do
                # Solo las lineas con tab son hallazgos; cualquier otra cosa que
                # se cuele no se imprime como si lo fuera.
                [ -z "$rp_ruta" ] && continue
                [ -z "$rp_remedio" ] && continue
                echo "    $rp_ruta" >&2
                echo "        $rp_remedio" >&2
            done
            echo "    Son documentos del USUARIO (docs/rutas-protegidas.md). Si el cambio es tuyo y" >&2
            echo "    querias hacerlo, commitealo; si no, revertilo con el comando de arriba." >&2
            failures=$((failures + 1))
            ;;
    esac
fi

if [ "$failures" -gt 0 ]; then
    echo "[Harness] Check fallo con $failures problema(s)." >&2
    [ "$MODE" = "warn" ] && exit 0
    exit 2
fi

echo "[Ok] Harness Check limpio."
