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
# ("" = corrida a mano, sin evento).
correr() {
    proyecto="$1"
    if [ -n "${2:-}" ]; then
        env HARNESS_STOP_HOOK_ACTIVE="$2" HARNESS_REPO_ROOT="$proyecto" \
            bash "$proyecto/hp/harness_check.sh" >"$TMP_ROOT/out" 2>"$TMP_ROOT/err" || return $?
    else
        env -u HARNESS_STOP_HOOK_ACTIVE HARNESS_REPO_ROOT="$proyecto" \
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
    ok "estado-degrada: ausente, vacio o con basura no rompe nada"
}

modo_payload_grande() {
    # AC-11. Este modo INLINEABA una copia del patron, asi que verificaba el
    # instrumento adyacente: cuando el patron de `run_stop` cambio (dos veces),
    # el modo siguio verde probando codigo muerto. Ahora extrae el matcher REAL
    # del instalador y lo ejercita; si `run_stop` cambia, esto lo acompaña o se
    # rompe ruidosamente, que es lo que queremos.
    matcher="$TMP_ROOT/matcher.sh"
    awk '/^run_stop\(\) \{/,/^\}/' "$REPO_ROOT/setup_harness.sh" \
        | sed -n '/if grep -qE/,/^    fi$/p' > "$matcher"
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
modo_a_mano_no_degrada() {
    # La promesa del spec: correr `bash harness_check.sh` a mano NUNCA degrada.
    # Sin evento no hay racha que contar, y el que lo corre quiere el veredicto.
    p="$(sembrar amano)"
    for _ in 1 2 3; do
        rc=0; correr "$p" "" || rc=$?
        [ "$rc" -eq 2 ] || fail "una corrida a mano degrado (rc=$rc): el veredicto se perdio"
    done
    ok "a-mano-no-degrada: sin evento, el check bloquea siempre"
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
    todos)
        modo_primera_vuelta
        modo_segunda_vuelta
        modo_degrada_todos_los_gates
        modo_centinela_sin_flag
        modo_centinela_reinicia
        modo_estado_degrada
        modo_payload_grande
        modo_a_mano_no_degrada
        ok "stop-hook: los ocho modos verdes"
        ;;
    *) echo "Modo desconocido: $MODO" >&2; exit 1 ;;
esac
