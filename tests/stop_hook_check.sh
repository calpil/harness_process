#!/bin/bash
# tests/stop_hook_check.sh - el fin de turno no puede quedar sin salida (#66)
#
# El bug que estos modos defienden: el Stop fallaba, el CLI re-invocaba, y cuando
# lo que fallaba no lo podia resolver el agente —un repo hermano sucio de otra
# sesion, un espejo de rol cuyo remedio es re-correr el instalador, un spec en
# draft que EXIGE el si del usuario— no habia ninguna accion que lo satisficiera.
#
# Cada modo monta un proyecto de mentira en un temporal y corre harness_check.sh
# como lo correria el hook. Nada toca el repo real.
set -Eeuo pipefail

MODO="${1:-todos}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/harness-stop-hook.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT

fail() { echo "[!] $MODO: $*" >&2; exit 1; }
ok()   { echo "[Ok] $*"; }

# Un proyecto minimo donde el check tenga algo que reportar que el agente NO
# puede resolver: un repo hermano sucio con un archivo que no es artefacto.
sembrar() {
    proyecto="$TMP_ROOT/$1"
    mkdir -p "$proyecto/hp" "$proyecto/ajeno"
    cp "$REPO_ROOT/harness_check.sh" "$REPO_ROOT/commit_guard.sh" "$proyecto/hp/"
    ( cd "$proyecto/ajeno" && git init -q . && : > TRABAJO-DE-OTRA-SESION.md )
    printf '%s' "$proyecto"
}

# Corre el check como lo corre el hook. $2 = valor de HARNESS_STOP_HOOK_ACTIVE
# ("" = corrida A MANO, sin evento).
#
# Un hook declara HARNESS_HOOK_EVENT ademas del flag: esa es la señal de "vengo
# de un evento" desde la #66. Antes se miraba si HARNESS_STOP_HOOK_ACTIVE estaba
# definida, y un `=0` residual en la terminal del usuario convertia una corrida a
# mano en evento.
correr() {
    proyecto="$1"
    if [ -n "${2:-}" ]; then
        env HARNESS_HOOK_EVENT=stop HARNESS_STOP_HOOK_ACTIVE="$2" HARNESS_REPO_ROOT="$proyecto" \
            bash "$proyecto/hp/harness_check.sh" >"$TMP_ROOT/out" 2>"$TMP_ROOT/err" || return $?
    else
        env -u HARNESS_HOOK_EVENT -u HARNESS_STOP_HOOK_ACTIVE HARNESS_REPO_ROOT="$proyecto" \
            bash "$proyecto/hp/harness_check.sh" >"$TMP_ROOT/out" 2>"$TMP_ROOT/err" || return $?
    fi
    return 0
}

modo_primera_vuelta() {
    # AC-2: la primera vuelta bloquea. Es la unica chance del agente de arreglar
    # lo que SI es suyo, y no se le saca.
    p="$(sembrar primera)"
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 2 ] || fail "la primera vuelta no bloqueo (rc=$rc); se le saco al agente su chance"
    grep -q "TRABAJO-DE-OTRA-SESION.md" "$TMP_ROOT/err" \
        || fail "el mensaje no nombra el archivo sucio (ver AC-8)"
    ok "primera-vuelta: bloquea con exit 2 y nombra el archivo"
}

modo_segunda_vuelta() {
    # AC-3: con la señal del CLI, imprime TODO y deja cerrar.
    p="$(sembrar segunda)"
    rc=0; correr "$p" 1 || rc=$?
    [ "$rc" -eq 0 ] || fail "la segunda vuelta siguio bloqueando (rc=$rc): el bucle sigue"
    grep -q "No bloqueo el cierre del turno" "$TMP_ROOT/err" \
        || fail "no dijo por que deja cerrar"
    grep -q "SIGUEN ahi" "$TMP_ROOT/err" \
        || fail "oculto los problemas en vez de mostrarlos: la segunda vuelta imprime MAS, no menos"
    grep -q "Cambios sin commitear" "$TMP_ROOT/err" \
        || fail "no imprimio el detalle de los gates"
    ok "segunda-vuelta: sale 0, imprime todo y dice de quien es la decision"
}

modo_degrada_todos_los_gates() {
    # AC-4: el corte es del CHECK ENTERO, no del commit_guard. Se prueba con un
    # proyecto SIN nada sucio pero con otro gate en rojo (falta el binario).
    p="$TMP_ROOT/gates"
    mkdir -p "$p/hp"
    cp "$REPO_ROOT/harness_check.sh" "$REPO_ROOT/commit_guard.sh" "$p/hp/"
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 2 ] || fail "sin señal no bloqueo (rc=$rc): no hay nada que degradar despues"
    grep -q "Cambios sin commitear" "$TMP_ROOT/err" \
        && fail "el fallo vino del guard; este modo tiene que probar OTRO gate"
    rc=0; correr "$p" 1 || rc=$?
    [ "$rc" -eq 0 ] || fail "un gate que no es el guard sigue bloqueando en la segunda vuelta (rc=$rc)"
    ok "degrada-todos-los-gates: la degradacion es del check entero"
}

modo_centinela_sin_flag() {
    # AC-5: un backend que NUNCA manda stop_hook_active. El corte no puede
    # depender de que el CLI se acuerde: el arnes es multi-LLM.
    p="$(sembrar centinela)"
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 2 ] || fail "la primera vuelta no bloqueo (rc=$rc)"
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 0 ] || fail "con el MISMO fallo repetido y sin flag, el check sigue en bucle (rc=$rc)"
    grep -q "veces seguidas" "$TMP_ROOT/err" \
        || fail "no explico que corto por racha propia"
    ok "centinela-sin-flag: corta aunque el CLI no mande nada"
}

modo_centinela_reinicia() {
    # AC-6: si cambia lo que falla, la racha se reinicia. Un problema NUEVO
    # siempre merece su vuelta.
    p="$(sembrar reinicia)"
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 2 ] || fail "la primera vuelta no bloqueo"
    printf '%s\n' "otra-firma-completamente-distinta:9" > "$p/progress/.stop_streak"
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 2 ] || fail "con una firma distinta no volvio a bloquear (rc=$rc): la racha no se reinicia"
    ok "centinela-reinicia: un conjunto de fallos distinto vuelve a bloquear"
}

modo_estado_degrada() {
    # AC-7: el estado local NUNCA puede hacer fallar un comando.
    p="$(sembrar degrada)"
    mkdir -p "$p/progress"
    for basura in "" "no-es-una-firma" ":::" "$(printf 'a\nb')"; do
        printf '%s' "$basura" > "$p/progress/.stop_streak"
        rc=0; correr "$p" 0 || rc=$?
        case "$rc" in
            0|2) : ;;
            *) fail "un .stop_streak con basura hizo fallar el check (rc=$rc)" ;;
        esac
    done
    # Los casos que el AC nombra y el test NO cubria ("sin permisos"), y que
    # escondian un bloqueante: con el .stop_streak como symlink y progress/ en
    # solo-lectura, el `rm` fallaba y —por ser el ultimo comando de una lista
    # `&&` bajo pipefail— MATABA el check con rc=1. Un Stop con rc=1 no bloquea:
    # el turno cerraba sin veredicto.
    rm -f "$p/progress/.stop_streak" 2>/dev/null || true
    printf 'CONTENIDO DEL USUARIO\n' > "$p/notas-del-usuario.txt"
    ln -s "$p/notas-del-usuario.txt" "$p/progress/.stop_streak"
    chmod 555 "$p/progress"
    rc=0; correr "$p" 0 || rc=$?
    chmod 755 "$p/progress"
    case "$rc" in
        0|2) : ;;
        *) fail "symlink + progress solo-lectura mato el check (rc=$rc): un Stop con rc=1 no bloquea" ;;
    esac
    [ "$(cat "$p/notas-del-usuario.txt")" = "CONTENIDO DEL USUARIO" ] \
        || fail "escribio A TRAVES del symlink y piso un archivo del usuario"

    # Y el symlink que SI se puede reemplazar: el archivo apuntado queda intacto.
    rm -f "$p/progress/.stop_streak"
    ln -s "$p/notas-del-usuario.txt" "$p/progress/.stop_streak"
    rc=0; correr "$p" 0 || rc=$?
    [ "$(cat "$p/notas-del-usuario.txt")" = "CONTENIDO DEL USUARIO" ] \
        || fail "reemplazando el symlink igual piso el archivo del usuario"
    [ -L "$p/progress/.stop_streak" ] \
        && fail "no reemplazo el symlink por el archivo real"

    ok "estado-degrada: ausente, vacio, basura, symlink y sin permisos: decide siempre y no toca nada del usuario"
}

modo_payload_grande() {
    # AC-11. Este modo INLINEABA una copia del patron, asi que verificaba el
    # instrumento adyacente: cuando el patron de `run_stop` cambio (dos veces),
    # el modo siguio verde probando codigo muerto. Ahora extrae el matcher REAL
    # del instalador y lo ejercita; si `run_stop` cambia, esto lo acompaña o se
    # rompe ruidosamente, que es lo que queremos.
    matcher="$TMP_ROOT/matcher.sh"
    awk '/^run_stop\(\) \{/,/^\}/' "$REPO_ROOT/setup_harness.sh" \
        | sed -n '/ultimo_valor=/,/^    esac$/p' > "$matcher"
    grep -q 'stop_hook_active' "$matcher" \
        || fail "payload-grande: no se pudo extraer el matcher de run_stop; ¿cambio su forma?"

    probar() {
        stop_input="$1"
        HARNESS_STOP_HOOK_ACTIVE=""
        # shellcheck disable=SC1090
        . "$matcher"
        [ "$HARNESS_STOP_HOOK_ACTIVE" = "$2" ] \
            || fail "payload-grande: flag=$HARNESS_STOP_HOOK_ACTIVE esperaba=$2 con: $(printf '%.60s' "$1")"
    }

    # Los cuatro falsos positivos que costaron una vuelta de revision: el JSON
    # real del Stop trae `cwd`, y un `case` que aceptaba cualquier `true`
    # posterior a la clave se comia la primera vuelta del agente.
    probar '{"stop_hook_active":false,"cwd":"/Users/alan/truenorth"}' 0
    probar '{"stop_hook_active":false,"note":"construed"}' 0
    probar '{"stop_hook_active":false,"verbose":true}' 0
    probar '{"stop_hook_active":false,"msg":"True story"}' 0
    probar '{"stop_hook_activeX":true}' 0
    probar '{"other":true,"stop_hook_active":false}' 0
    probar '{"session_id":"a"}' 0
    probar '' 0
    probar '{"stop_hook_active":true,"cwd":"/Users/alan/truenorth"}' 1
    probar '{"stop_hook_active": true}' 1
    probar '{"stop_hook_active" : true}' 1
    probar '{"stop_hook_active":True}' 1
    # Clave duplicada: gana la ULTIMA, como en cualquier parser JSON.
    probar '{"stop_hook_active":false,"stop_hook_active":true}' 1
    probar '{"stop_hook_active":true,"stop_hook_active":false}' 0

    # Y que no dependa del tamaño NI se vuelva lento: el arreglo intermedio
    # (recorte de prefijo, cuadratico en bash) tardaba 20 s con 200 KB y ~8 min
    # con 1 MB, contra un timeout de hook de 120 s. Un hook que no termina es
    # peor que uno que decide mal.
    pad="$(head -c 204800 /dev/zero | tr '\0' 'x')"
    inicio="$(date +%s)"
    probar "{\"pad\":\"$pad\",\"stop_hook_active\":true}" 1
    probar "{\"pad\":\"$pad\",\"stop_hook_active\":false}" 0
    fin="$(date +%s)"
    [ "$((fin - inicio))" -le 5 ] \
        || fail "payload-grande: 200 KB tardo $((fin - inicio))s; el matcher volvio a ser no-lineal"
    ok "payload-grande: adyacencia clave-valor, sin falsos positivos y lineal (200 KB en $((fin - inicio))s)"
}
modo_centinela_problema_nuevo() {
    # AC-6, el escenario de verdad. `centinela-reinicia` fabrica la firma a mano,
    # asi que no protege el mecanismo: si alguien borra el DETALLE que
    # `sumar_fallo` mete en la firma (la regresion exacta de B2), aquel modo
    # sigue verde. Este hace lo que pasa en la vida real: se arregla un archivo y
    # aparece OTRO.
    p="$(sembrar problema_nuevo)"
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 2 ] || fail "la primera vuelta no bloqueo (rc=$rc)"
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 0 ] || fail "el mismo problema repetido no degrado (rc=$rc)"
    # Se arregla lo viejo y aparece algo nuevo: tiene que volver a bloquear.
    ( cd "$p/ajeno" \
        && git add -A >/dev/null 2>&1 \
        && git -c user.email=t@t -c user.name=t commit -qm "arreglado" >/dev/null 2>&1 \
        && : > PROBLEMA-NUEVO.md )
    rc=0; correr "$p" 0 || rc=$?
    [ "$rc" -eq 2 ] \
        || fail "un problema NUEVO no volvio a bloquear (rc=$rc): la firma no mira el contenido"
    grep -q "PROBLEMA-NUEVO.md" "$TMP_ROOT/err" \
        || fail "no nombro el archivo nuevo"
    ok "centinela-problema-nuevo: arreglar uno y ensuciar otro reinicia la racha"
}

modo_a_mano_no_degrada() {
    # La promesa del spec: correr `bash harness_check.sh` a mano NUNCA degrada.
    # Sin evento no hay racha que contar, y el que lo corre quiere el veredicto.
    p="$(sembrar amano)"
    for _ in 1 2 3; do
        rc=0; correr "$p" "" || rc=$?
        [ "$rc" -eq 2 ] || fail "una corrida a mano degrado (rc=$rc): el veredicto se perdio"
    done
    # Y con un HARNESS_STOP_HOOK_ACTIVE=0 RESIDUAL en el entorno —lo que queda
    # tras debuggear un hook— tampoco: el evento lo declara el hook, no una
    # variable que quedo colgada.
    for _ in 1 2 3; do
        rc=0
        env -u HARNESS_HOOK_EVENT HARNESS_STOP_HOOK_ACTIVE=0 HARNESS_REPO_ROOT="$p" \
            bash "$p/hp/harness_check.sh" >"$TMP_ROOT/out" 2>"$TMP_ROOT/err" || rc=$?
        [ "$rc" -eq 2 ] \
            || fail "con HARNESS_STOP_HOOK_ACTIVE=0 residual, una corrida a mano degrado (rc=$rc)"
    done
    ok "a-mano-no-degrada: sin evento el check bloquea siempre, aun con el flag residual"
}

case "$MODO" in
    primera-vuelta)            modo_primera_vuelta ;;
    segunda-vuelta)            modo_segunda_vuelta ;;
    degrada-todos-los-gates)   modo_degrada_todos_los_gates ;;
    centinela-sin-flag)        modo_centinela_sin_flag ;;
    centinela-reinicia)        modo_centinela_reinicia ;;
    estado-degrada)            modo_estado_degrada ;;
    payload-grande)            modo_payload_grande ;;
    a-mano-no-degrada)         modo_a_mano_no_degrada ;;
    centinela-problema-nuevo)  modo_centinela_problema_nuevo ;;
    todos)
        modo_primera_vuelta
        modo_segunda_vuelta
        modo_degrada_todos_los_gates
        modo_centinela_sin_flag
        modo_centinela_reinicia
        modo_centinela_problema_nuevo
        modo_estado_degrada
        modo_payload_grande
        modo_a_mano_no_degrada
        ok "stop-hook: los nueve modos verdes"
        ;;
    *) echo "Modo desconocido: $MODO" >&2; exit 1 ;;
esac
