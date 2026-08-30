#!/bin/bash
# El commit_guard no cuelga a quien no es un hook.
#
# El guard arranca con `INPUT=$(cat)` porque su uso normal es COMO hook: el
# agente le manda el JSON del evento por la entrada. harness_check.sh lo invoca
# sin ser un hook, y con stdin abierto `cat` espera un EOF que nadie va a
# mandar: el check entero se cuelga (medido en la feature #52: 18 minutos hasta
# matarlo, en una corrida en segundo plano). El arreglo es cerrarle la entrada y
# pasarle por entorno el unico dato que ese JSON traia.
#
# Modos, uno por criterio:
#   limite          el mecanismo de limite de tiempo de este test funciona
#   no-cuelga       harness_check.sh termina con stdin abierto que nunca cierra
#   prueba-del-rojo la version PREVIA si se cuelga (si no, este test no mide nada)
#   stop-por-env    el guard respeta HARNESS_STOP_HOOK_ACTIVE=1 y no bloquea
#   stop-por-json   como hook, el JSON por stdin sigue mandando
#   bloquea         sin ninguna de las dos senales, un repo sucio sigue bloqueando
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

# Sandbox: un arnes con layout root y UN microservicio sucio colgando, que es lo
# que hace hablar al guard.
sandbox() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-guard.XXXXXX")"
    mkdir -p "$tmp/hp/docs" "$tmp/hp/miservicio"
    cp "$REPO_ROOT/commit_guard.sh" "$tmp/hp/commit_guard.sh"
    cp "$REPO_ROOT/harness_check.sh" "$tmp/hp/harness_check.sh"
    cp "$REPO_ROOT/harness_cli" "$tmp/hp/harness_cli"
    printf 'root\n' > "$tmp/hp/.harness_layout"
    printf '# Constitution\n' > "$tmp/hp/docs/constitution.md"
    git -C "$tmp/hp/miservicio" init -q
    printf 'sin commitear\n' > "$tmp/hp/miservicio/pendiente.txt"
    echo "$tmp"
}

# --- Limite de tiempo, sin depender de coreutils ---------------------------
#
# Este test decide si algo se colgo cortandolo por tiempo. Durante un tiempo
# uso `timeout 10` a secas, y `timeout(1)` es de coreutils: NO viene en macOS.
# Ahi el subshell devolvia 127, el test solo consideraba colgado al 124, y el
# modo `no-cuelga` salia VERDE pase lo que pase — un criterio que no se puede
# fallar (leccion criterios-de-cierre-que-se-pueden-fallar). `perl` si esta en
# macOS y en Linux, y `alarm` hace exactamente lo mismo.
#
# Codigos: 124 lo pone timeout(1); 142 es 128+14 (SIGALRM) y lo pone perl.
LIMITE_CORTADO_TIMEOUT=124
LIMITE_CORTADO_ALARM=142

# Que mecanismo hay en esta maquina. Sin ninguno devuelve 1: quien llama FALLA,
# nunca saltea en verde.
mecanismo_de_limite() {
    for cand in timeout gtimeout perl; do
        if command -v "$cand" >/dev/null 2>&1; then
            echo "$cand"
            return 0
        fi
    done
    return 1
}

# Corre un comando con limite de `$1` segundos. Devuelve 0 si termino solo y 1
# si hubo que cortarlo.
con_limite() {
    seg="$1"
    shift
    mec="$(mecanismo_de_limite)" || fail "limite: no hay timeout, gtimeout ni perl en esta maquina; instala coreutils o perl (este test no puede medir sin uno de los tres, y no se saltea en verde)"
    case "$mec" in
        timeout)  "$mec" "$seg" "$@" >/dev/null 2>&1 && rc=0 || rc=$? ;;
        gtimeout) "$mec" "$seg" "$@" >/dev/null 2>&1 && rc=0 || rc=$? ;;
        # Perl vigila a un hijo y sale EL NORMALMENTE con el codigo: si en
        # cambio se dejara matar por SIGALRM, el shell imprimiria un
        # "Alarm clock" que ensucia la salida del test.
        perl)     rc="$(perl -e '
            my $seg = shift;
            my $pid = fork();
            if (!defined $pid) { print 127; exit 0 }
            if ($pid == 0) {
                open(STDOUT, ">", "/dev/null");
                open(STDERR, ">", "/dev/null");
                exec(@ARGV);
                exit 127;
            }
            $SIG{ALRM} = sub { kill("KILL", $pid); waitpid($pid, 0); print '"$LIMITE_CORTADO_ALARM"'; exit 0 };
            alarm($seg);
            waitpid($pid, 0);
            alarm(0);
            print($? >> 8);
        ' "$seg" "$@")" ;;
    esac
    [ "$rc" != "$LIMITE_CORTADO_TIMEOUT" ] && [ "$rc" != "$LIMITE_CORTADO_ALARM" ]
}

# El auto-test del andamiaje: sin esto, "ahora si mide" seria otra afirmacion
# sin comprobar, que es justo el bug que este test arrastraba.
modo_limite() {
    mec="$(mecanismo_de_limite)" || fail "limite: no hay timeout, gtimeout ni perl en esta maquina; instala coreutils o perl (este test no puede medir sin uno de los tres, y no se saltea en verde)"
    con_limite 5 sleep 0 \
        || fail "limite: con '$mec', un comando que termina solo se reporto como colgado"
    if con_limite 1 sleep 30; then
        fail "limite: con '$mec', un comando que se cuelga NO se corto (este test no puede medir nada)"
    fi
    # Y sin NINGUN mecanismo la deteccion tiene que fallar, para que quien la
    # llama se detenga: un test que no puede medir se pone rojo, no verde
    # (leccion criterios-de-cierre-que-se-pueden-fallar).
    if ( PATH="/nonexistent"; mecanismo_de_limite ) >/dev/null 2>&1; then
        fail "limite: sin timeout, gtimeout ni perl en el PATH, la deteccion tiene que fallar y no devolver un mecanismo"
    fi
    ok "limite: '$mec' corta lo que se cuelga y no corta lo que termina; sin ninguno, falla"
}

# Corre un comando con una entrada que NUNCA cierra, y responde si termino solo.
# `sleep | cmd` deja el pipe abierto sin mandar un byte: exactamente la corrida
# en segundo plano / CI del hallazgo.
termina_con_stdin_abierto() {
    script="$1"
    dir="$(dirname "$script")"
    # El `sleep` que alimenta el pipe vive un poco mas que el limite, para que
    # el pipe siga abierto todo el tiempo que dura la medicion.
    (cd "$dir" && sleep 12 | con_limite 6 bash "$script")
}

modo_no_cuelga() {
    tmp="$(sandbox)"
    if termina_con_stdin_abierto "$tmp/hp/harness_check.sh"; then
        rm -rf "$tmp"
        ok "no-cuelga: harness_check.sh termina con la entrada abierta"
    else
        rm -rf "$tmp"
        fail "no-cuelga: harness_check.sh se colgo esperando EOF (la regresion de la #52)"
    fi
}

modo_prueba_del_rojo() {
    tmp="$(sandbox)"
    # Contra el cuelgue hay DOS defensas, una en cada lado, y de features
    # distintas: la #52 le cerro la entrada al guard en la invocacion
    # (`</dev/null`), y la #53 le puso al guard su propia guarda de terminal
    # (`[ -t 0 ]`). Reconstruir solo una deja la otra en pie y el rojo no
    # aparece — que es como este modo dejo de medir sin que nadie lo notara.
    # La #66 cambio la invocacion (captura la salida y apaga la señal del guard),
    # asi que este sed se actualizo con ella. Que el test HAYA FALLADO al
    # cambiarla es la señal de que sigue midiendo: si el sed no reconstruye nada,
    # el modo `no-cuelga` deja de probar el cuelgue sin que nadie lo note.
    sed 's#commit_guard.sh" </dev/null#commit_guard.sh"#' \
        "$tmp/hp/harness_check.sh" > "$tmp/hp/viejo.sh"
    grep -q 'commit_guard.sh" 2>&1' "$tmp/hp/viejo.sh" \
        || { rm -rf "$tmp"; fail "prueba-del-rojo: no se pudo reconstruir la invocacion previa (#52)"; }
    # `if false` deja el `cat` sin guarda, como antes de la #53.
    sed 's/^if \[ -t 0 \]; then$/if false; then/' \
        "$tmp/hp/commit_guard.sh" > "$tmp/hp/guard_viejo.sh"
    grep -q '^if false; then$' "$tmp/hp/guard_viejo.sh" \
        || { rm -rf "$tmp"; fail "prueba-del-rojo: no se pudo reconstruir la guarda previa del guard (#53)"; }
    mv "$tmp/hp/guard_viejo.sh" "$tmp/hp/commit_guard.sh"

    if termina_con_stdin_abierto "$tmp/hp/viejo.sh"; then
        rm -rf "$tmp"
        fail "prueba-del-rojo: la version previa NO se colgo; este test no esta midiendo el cuelgue"
    else
        rm -rf "$tmp"
        ok "prueba-del-rojo: la version previa si se cuelga, asi que el modo no-cuelga mide algo"
    fi
}

modo_stop_por_env() {
    tmp="$(sandbox)"
    salida="$(cd "$tmp/hp" && HARNESS_STOP_HOOK_ACTIVE=1 bash ./commit_guard.sh </dev/null 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "0" ] || fail "stop-por-env: exit $rc, esperaba 0 (no puede bloquear dos veces el mismo turno). Dijo: $salida"
    ok "stop-por-env: con HARNESS_STOP_HOOK_ACTIVE=1 avisa pero no bloquea"
}

modo_stop_por_json() {
    tmp="$(sandbox)"
    salida="$(cd "$tmp/hp" && printf '{"stop_hook_active": true}' | bash ./commit_guard.sh 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "0" ] || fail "stop-por-json: exit $rc, esperaba 0. El camino de hook tiene que seguir intacto. Dijo: $salida"
    ok "stop-por-json: como hook, el JSON por stdin sigue mandando"
}

modo_bloquea() {
    tmp="$(sandbox)"
    salida="$(cd "$tmp/hp" && bash ./commit_guard.sh </dev/null 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "2" ] || fail "bloquea: exit $rc, esperaba 2 con un repo sucio y sin senal de stop. Dijo: $salida"
    printf '%s' "$salida" | grep -q "miservicio" \
        || fail "bloquea: no nombra el repo sucio. Dijo: $salida"
    ok "bloquea: sin senal de stop, un repo sucio sigue bloqueando y se nombra"
}

# Feature #66: el mensaje nombraba el REPO y nada mas ("Cambios sin commitear
# en: docs"), asi que el agente no podia saber si lo sucio era suyo y la unica
# salida que le quedaba era commitear a ciegas trabajo que podia ser de otra
# sesion. Ver docs/lecciones/remedios-que-la-herramienta-sugiere.md.
modo_nombra_archivos() {
    tmp="$(sandbox)"
    # Un artefacto del arnes (exento) y uno ajeno. El artefacto va bajo `docs/`
    # porque la exencion exige la UBICACION, no solo el nombre (commit_guard.sh:97-108):
    # un `impl-notas.md` suelto dentro de un microservicio es un documento real.
    mkdir -p "$tmp/hp/miservicio/docs"
    : > "$tmp/hp/miservicio/docs/spec-feature-9-algo.md"
    salida="$(cd "$tmp/hp" && bash ./commit_guard.sh </dev/null 2>&1)" && rc=0 || rc=$?
    rm -rf "$tmp"
    [ "$rc" = "2" ] || fail "nombra-archivos: exit $rc, esperaba 2. Dijo: $salida"
    printf '%s' "$salida" | grep -q "pendiente.txt" \
        || fail "nombra-archivos: no nombra el archivo ajeno. Dijo: $salida"
    printf '%s' "$salida" | grep -q "spec-feature-9-algo.md" \
        && fail "nombra-archivos: nombro un artefacto del arnes, que esta exento. Dijo: $salida"
    printf '%s' "$salida" | grep -q "NO lo commitees" \
        || fail "nombra-archivos: falta la salida 'si no es tuyo'. Dijo: $salida"
    printf '%s' "$salida" | grep -q "para TODO el repo" \
        || fail "nombra-archivos: no dice que \`off\` apaga el guard entero. Dijo: $salida"
    ok "nombra-archivos: nombra los ajenos, respeta los exentos y ofrece la tercera salida"
}

case "$MODO" in
    limite)          modo_limite ;;
    no-cuelga)       modo_no_cuelga ;;
    prueba-del-rojo) modo_prueba_del_rojo ;;
    stop-por-env)    modo_stop_por_env ;;
    stop-por-json)   modo_stop_por_json ;;
    bloquea)         modo_bloquea ;;
    nombra-archivos) modo_nombra_archivos ;;
    todos)
        modo_limite
        modo_no_cuelga
        modo_prueba_del_rojo
        modo_stop_por_env
        modo_stop_por_json
        modo_bloquea
        modo_nombra_archivos
        ok "commit_guard: los siete modos verdes"
        ;;
    *) fail "modo desconocido: $MODO (limite | no-cuelga | prueba-del-rojo | stop-por-env | stop-por-json | bloquea | nombra-archivos | todos)" ;;
esac
