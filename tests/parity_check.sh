#!/bin/bash
# Paridad estructural entre setup_harness.sh y setup_harness.ps1 (feature #30).
#
# Cierra la deuda que once features arrastraron ("esta maquina no tiene pwsh")
# sin instalar PowerShell: en vez de EJECUTAR el instalador de Windows, compara
# lo que los dos DECLARAN. No prueba que el .ps1 funcione — prueba que no se
# quede atras sin que nadie se entere, que es lo que venia pasando.
#
# Modos, uno por AC:
#   opciones              AC-1   las opciones de los dos coinciden o estan declaradas
#   detecta-opcion        AC-2   una opcion en un solo lado se reporta  <- prueba del rojo
#   asimetrias-declaradas AC-3   las cinco de hoy estan declaradas CON razon
#   superficies           AC-4   los dos escriben las mismas superficies
#   smokes                AC-5   los dos smokes cubren los mismos bloques
#   promesa-acotada       AC-7   verification.md no manda correr lo que nadie corre
#   en-harness-check      AC-8   el aviso corre y NO cambia el exit code
#   sin-ps1               AC-9   sin el .ps1, silencio
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

# --- Asimetrias DECLARADAS -------------------------------------------------
# Cada una con su razon. Una excepcion anonima es un agujero; una con razon
# escrita es una decision que alguien puede discutir.
#
# Las razones se verificaron UNA POR UNA contra el codigo antes de escribirlas, y
# dos salieron mal en el primer intento: `--with-postgres` no es la afirmativa de
# un default (es un no-op historico, linea `--with-postgres) ;;`) y
# `-CargoTargetDir` no tiene que ver con el PATH de rustup sino con
# CARGO_TARGET_DIR. Una razon decorativa es peor que ninguna: se cita como cierta.
#
# formato: <opcion>|<solo-en>|<razon>
asimetrias() {
    cat <<'ASIM'
--with-subagents|sh|afirmativa de un default ya encendido; en ps1 solo existe el -NoSubagents que lo apaga
--install-graphify|sh|afirmativa de un default ya encendido; en ps1 solo existe el -NoGraphify que lo apaga
--install-antigravity|sh|afirmativa de un default ya encendido; en ps1 solo existe el -NoAntigravity que lo apaga
--with-postgres|sh|no-op historico: PostgreSQL es obligatorio desde la feature #14 y el flag se mantiene solo para no romper invocaciones viejas
-CargoTargetDir|ps1|en Unix el mismo efecto se logra exportando CARGO_TARGET_DIR antes de correr el instalador; en PowerShell el flag ahorra tocar el entorno de la sesion
ASIM
}

# -PascalCase -> kebab-case. En awk y no en sed: `\L` es una extension de GNU
# sed que BSD (macOS) no tiene, y con ella el chequeo reportaba basura como
# `--LNo-LGraphify` en vez de `--no-graphify`.
a_kebab() {
    printf '%s' "$1" | sed 's/^-//' | awk '{
        out = ""
        for (i = 1; i <= length($0); i++) {
            c = substr($0, i, 1)
            if (c ~ /[A-Z]/) {
                if (i > 1) out = out "-"
                out = out tolower(c)
            } else {
                out = out c
            }
        }
        print out
    }'
}

# --kebab-case -> -PascalCase
a_pascal() {
    printf '%s' "$1" | sed 's/^--//' | awk -F- '{ for (i=1;i<=NF;i++) printf "%s%s", toupper(substr($i,1,1)), substr($i,2); print "" }' \
        | sed 's/^/-/'
}

opciones_sh() {
    # Las ramas del `case` que parsea argumentos, incluidas las que agrupan
    # varias opciones (`--dry-run|--preview)`). La primera version solo miraba
    # ramas de una sola opcion y por eso creia que al .sh le faltaba --dry-run.
    grep -oE '^[[:space:]]+--[a-z|-]+\)' "$REPO_ROOT/setup_harness.sh" \
        | tr -d ' )' | tr '|' '\n' | grep -E '^--' | sort -u
}

opciones_ps1() {
    # Solo el bloque param() de nivel superior, no los de las funciones.
    awk '/^param\(/{f=1;next} f&&/^\)/{exit} f' "$1" \
        | grep -oE '\$[A-Za-z]+' | tr -d '$' | sed 's/^/-/' | sort -u
}

declarada() {
    asimetrias | grep -q "^$1|"
}

# Compara los dos y escribe una linea por diferencia NO declarada.
diferencias() {
    ps1_file="${1:-$REPO_ROOT/setup_harness.ps1}"
    ps1_ops="$(opciones_ps1 "$ps1_file")"
    while IFS= read -r flag; do
        [ -z "$flag" ] && continue
        # --preview es alias de --dry-run; en ps1 es [Alias("Preview")].
        [ "$flag" = "--preview" ] && continue
        pascal="$(a_pascal "$flag")"
        if ! printf '%s\n' "$ps1_ops" | grep -qix -- "$pascal"; then
            declarada "$flag" || echo "$flag|falta en setup_harness.ps1 (esperaba $pascal)"
        fi
    done <<< "$(opciones_sh)"
    sh_ops="$(opciones_sh)"
    while IFS= read -r param; do
        [ -z "$param" ] && continue
        # Los que no son opciones de instalacion.
        case "$param" in -Help|-Version|-Preview) continue ;; esac
        kebab="--$(a_kebab "$param")"
        if ! printf '%s\n' "$sh_ops" | grep -qx -- "$kebab"; then
            declarada "$param" || echo "$param|falta en setup_harness.sh (esperaba $kebab)"
        fi
    done <<< "$ps1_ops"
}

modo_opciones() {
    difs="$(diferencias)"
    if [ -n "$difs" ]; then
        printf '%s\n' "$difs" | while IFS='|' read -r op donde; do
            echo "    $op: $donde" >&2
        done
        fail "opciones: hay diferencias sin declarar entre los dos instaladores"
    fi
    ok "opciones: los dos instaladores declaran las mismas, salvo las asimetrias declaradas"
}

modo_detecta_opcion() {
    # Prueba del rojo, sobre COPIAS: los archivos reales no se tocan.
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-parity.XXXXXX")"
    cp "$REPO_ROOT/setup_harness.ps1" "$tmp/con-extra.ps1"
    # Se agrega un parametro que el .sh no tiene.
    awk '/^param\(/{print; print "    [switch]$SoloEnWindows,"; next} {print}' \
        "$REPO_ROOT/setup_harness.ps1" > "$tmp/con-extra.ps1"
    difs="$(diferencias "$tmp/con-extra.ps1")"
    rm -rf "$tmp"
    printf '%s' "$difs" | grep -q "SoloEnWindows" \
        || fail "detecta-opcion: NO reporto la opcion sembrada. Reporto: ${difs:-(nada)}"
    printf '%s' "$difs" | grep -q "falta en setup_harness.sh" \
        || fail "detecta-opcion: no dice en cual instalador falta. Reporto: $difs"
    ok "detecta-opcion: reporta la opcion sembrada y dice en cual falta"
}

modo_asimetrias_declaradas() {
    n=0
    while IFS='|' read -r op donde razon; do
        [ -z "$op" ] && continue
        n=$((n + 1))
        [ -n "$razon" ] || fail "asimetrias: $op esta declarada SIN razon"
        [ "${#razon}" -ge 20 ] || fail "asimetrias: la razon de $op es demasiado corta: '$razon'"
        case "$donde" in sh|ps1) ;; *) fail "asimetrias: $op no dice en cual instalador vive" ;; esac
    done <<< "$(asimetrias)"
    [ "$n" -ge 5 ] || fail "asimetrias: esperaba las 5 conocidas, hay $n"
    ok "asimetrias-declaradas: $n, cada una con su razon y su lado"
}

modo_superficies() {
    faltantes=""
    for sup in "CLAUDE.md" "AGENTS.md" "GEMINI.md" "LLM.md"; do
        en_sh=0; en_ps1=0
        grep -q "$sup" "$REPO_ROOT/setup_harness.sh" && en_sh=1
        grep -q "$sup" "$REPO_ROOT/setup_harness.ps1" && en_ps1=1
        [ "$en_sh" = "$en_ps1" ] || faltantes="$faltantes $sup(sh=$en_sh ps1=$en_ps1)"
    done
    [ -z "$faltantes" ] || fail "superficies: solo en uno de los dos:$faltantes"
    ok "superficies: los dos instaladores escriben las mismas"
}

# Los dos smokes se escribieron distinto —el .sh marca bloques con `[Ok] <tema>`
# y el .ps1 usa 132 `Assert-True` sin secciones nombradas—, asi que contar
# bloques no compara nada. Lo que si compara es la COBERTURA: cada tema que el
# .sh declara tiene que aparecer, por su palabra clave, en el .ps1.
TEMAS="dry-run|DryRun reset|Reset version|Version subdir|Subdir root|Root graphify|Graphify kimi|Kimi atlassian|Atlassian migrate-rules|MigrateRules"

modo_smokes() {
    [ -f "$REPO_ROOT/tests/setup_smoke.ps1" ] || { ok "smokes: no hay smoke ps1, nada que comparar"; return; }
    sin_cubrir=""
    for par in $TEMAS; do
        tema="${par%%|*}"
        en_ps1="${par#*|}"
        grep -qi -- "$tema" "$REPO_ROOT/tests/setup_smoke.sh" || continue
        grep -qi -- "$en_ps1" "$REPO_ROOT/tests/setup_smoke.ps1" \
            || sin_cubrir="$sin_cubrir $tema"
    done
    [ -z "$sin_cubrir" ] || fail "smokes: el .sh prueba temas que el .ps1 no menciona:$sin_cubrir"
    asserts_ps1="$(grep -c 'Assert-True' "$REPO_ROOT/tests/setup_smoke.ps1" || true)"
    [ "$asserts_ps1" -ge 20 ] \
        || fail "smokes: el smoke ps1 solo tiene $asserts_ps1 aserciones; ¿quedo atras?"
    ok "smokes: los temas del .sh estan cubiertos en el .ps1 ($asserts_ps1 aserciones)"
}

# Feature #64: la migracion de `rules` toca el feature_list.json del USUARIO, y
# tiene que existir en los DOS instaladores. El modo `smokes` no lo detectaba:
# el reviewer de la #64 borro `Migrate-HarnessRules` entero del .ps1 y los ocho
# modos seguian en verde, o sea que el criterio del AC-9 no podia fallar.
modo_migracion_rules() {
    falta=""
    grep -q "^migrate_rules() {" "$REPO_ROOT/setup_harness.sh" || falta="$falta setup_harness.sh:migrate_rules"
    grep -q "migrate_rules feature_list.json" "$REPO_ROOT/setup_harness.sh" \
        || falta="$falta setup_harness.sh:llamada"
    if [ -f "$REPO_ROOT/setup_harness.ps1" ]; then
        grep -q "function Migrate-HarnessRules" "$REPO_ROOT/setup_harness.ps1" \
            || falta="$falta setup_harness.ps1:Migrate-HarnessRules"
        grep -q "Migrate-HarnessRules -Target" "$REPO_ROOT/setup_harness.ps1" \
            || falta="$falta setup_harness.ps1:llamada"
    fi
    [ -z "$falta" ] \
        || fail "migracion-rules: la migracion de reglas no esta en los dos lados:$falta"
    ok "migracion-rules: definida y llamada en los dos instaladores"
}

# Feature #66: el bug fue que habia DOS escritores de hooks y uno no se entero
# del contrato. `.claude/settings.json` en POSIX llamaba `harness_check.sh`
# derecho, asi que `stop_hook_active` moria antes de llegar al gate — mientras el
# mismo backend, escrito por el .ps1, ya despachaba al runtime. Este modo impide
# que vuelva a existir un cableado que se saltee `bin/harness-hook`.
modo_cableado_hooks() {
    falta=""
    sh_file="$REPO_ROOT/setup_harness.sh"
    # AFIRMACION POSITIVA, no denylist. La primera version eran tres grep
    # NEGATIVOS de las formas historicas, y el reviewer la paso por arriba con
    # tres mutantes que rompian el cableado dejando la forma nueva: el Stop
    # apuntando al check directo pero con SURFACE_BASE, el PreToolUse
    # despachando el evento equivocado, y un typo (harness-hookk) que sale 127.
    # Un chequeo que solo prohibe tres formas conocidas no afirma nada.
    grep -qF 'bin/harness-hook\" plain stop' "$sh_file" \
        || falta="$falta Stop:no-invoca-el-runtime-con-su-evento"
    grep -qF 'bin/harness-hook\" plain PreToolUse' "$sh_file" \
        || falta="$falta PreToolUse:no-invoca-el-runtime-con-su-evento"
    # Y NINGUN comando de hook corre el check o el guard por su cuenta: ahi es
    # donde muere el JSON del evento.
    # Sin clase negada: el comando trae comillas ESCAPADAS (\") y un `[^"]*` se
    # corta en la primera, dejando pasar `"command": "bash \"$X/harness_check.sh\""`
    # — que es exactamente el mutante con el que el reviewer paso por arriba la
    # primera version de este modo.
    if grep -E '"command":.*(harness_check|commit_guard)' "$sh_file" >/dev/null 2>&1; then
        falta="$falta comando-de-hook-sin-pasar-por-el-runtime"
    fi
    # El runtime es SUPERFICIE y vive en la raiz: con HOOK_BASE la ruta apunta a
    # <raiz>/<subdir>/bin/harness-hook, que en layout subdir no existe (127).
    grep -qF 'HOOK_BASE/bin/harness-hook' "$sh_file" \
        && falta="$falta runtime-con-HOOK_BASE-en-vez-de-SURFACE_BASE"
    # Cada Stop declara su timeout, como las otras cuatro superficies (AC-12).
    stops="$(grep -cF 'bin/harness-hook\" plain stop' "$sh_file" || true)"
    # JSON usa `"timeout":`, TOML (Kimi) usa `timeout =`: se aceptan los dos.
    timeouts="$(grep -A1 -F 'bin/harness-hook\" plain stop' "$sh_file" | grep -cE '"timeout"|timeout *=' || true)"
    if [ "$stops" -eq 0 ] || [ "$timeouts" -ne "$stops" ]; then
        falta="$falta Stop-sin-timeout-declarado[$timeouts-de-$stops]"
    fi
    # Los DOS instaladores, para el MISMO backend, despachan al runtime.
    if [ -f "$REPO_ROOT/setup_harness.ps1" ]; then
        grep -qF 'harness-hook.ps1" plain stop' "$REPO_ROOT/setup_harness.ps1" \
            || falta="$falta ps1:Stop-no-despacha-al-runtime"
    fi
    [ -z "$falta" ] \
        || fail "cableado-hooks: el cableado no cumple el contrato:$falta"
    ok "cableado-hooks: cada evento invoca el runtime con SU evento, con timeout, en los dos instaladores"
}

modo_promesa_acotada() {
    grep -q "no ejecuta el instalador de Windows" "$REPO_ROOT/docs/verification.md" \
        || fail "promesa-acotada: verification.md no dice que el chequeo no ejecuta el instalador de Windows"
    grep -q "si tenes Windows" "$REPO_ROOT/docs/verification.md" \
        || fail "promesa-acotada: la instruccion del smoke ps1 sigue sin condicionar"
    grep -q "parity_check.sh" "$REPO_ROOT/docs/verification.md" \
        || fail "promesa-acotada: no nombra el sustituto que si corre siempre"
    ok "promesa-acotada: la instruccion esta condicionada y nombra el sustituto"
}

modo_en_harness_check() {
    grep -q "parity_check.sh" "$REPO_ROOT/harness_check.sh" \
        || fail "en-harness-check: harness_check.sh no corre el chequeo de paridad"
    # Y NO puede cambiar el exit code: el bloque no toca failures.
    bloque="$(awk '/Paridad de los instaladores/,/^fi$/' "$REPO_ROOT/harness_check.sh")"
    [ -n "$bloque" ] || fail "en-harness-check: no se encontro el bloque de paridad"
    printf '%s' "$bloque" | grep -q "failures=" \
        && fail "en-harness-check: el bloque toca failures; decidimos que avisa sin bloquear"
    ok "en-harness-check: corre el chequeo y no cambia el exit code"
}

modo_sin_ps1() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-parity-sin.XXXXXX")"
    mkdir -p "$tmp/hp" "$tmp/docs"
    cp "$REPO_ROOT/harness_check.sh" "$tmp/hp/harness_check.sh"
    cp "$REPO_ROOT/harness_cli" "$tmp/hp/harness_cli"
    cp "$REPO_ROOT/harness" "$tmp/hp/harness" 2>/dev/null || true
    printf 'root\n' > "$tmp/hp/.harness_layout"
    printf '# Constitution\n' > "$tmp/docs/constitution.md"
    salida="$(cd "$tmp/hp" && HARNESS_CHECK_MODE=warn bash ./harness_check.sh 2>&1 || true)"
    rm -rf "$tmp"
    printf '%s' "$salida" | grep -qi "paridad" \
        && fail "sin-ps1: hablo de paridad sin setup_harness.ps1. Dijo: $salida"
    ok "sin-ps1: sin el instalador de Windows, el bloque se omite sin ruido"
}

case "$MODO" in
    opciones)              modo_opciones ;;
    detecta-opcion)        modo_detecta_opcion ;;
    asimetrias-declaradas) modo_asimetrias_declaradas ;;
    superficies)           modo_superficies ;;
    smokes)                modo_smokes ;;
    migracion-rules)       modo_migracion_rules ;;
    cableado-hooks)        modo_cableado_hooks ;;
    promesa-acotada)       modo_promesa_acotada ;;
    en-harness-check)      modo_en_harness_check ;;
    sin-ps1)               modo_sin_ps1 ;;
    todos)
        modo_opciones
        modo_detecta_opcion
        modo_asimetrias_declaradas
        modo_superficies
        modo_smokes
        modo_migracion_rules
        modo_cableado_hooks
        modo_promesa_acotada
        modo_en_harness_check
        modo_sin_ps1
        ok "paridad: los diez modos verdes"
        ;;
    *) fail "modo desconocido: $MODO" ;;
esac
