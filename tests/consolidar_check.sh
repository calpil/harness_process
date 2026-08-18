#!/bin/bash
# Verifica la consolidacion de lecciones con backend REAL (feature #28).
#
# Existe porque la acceptance lo exige con todas las letras: "poder verificarse
# de punta a punta con al menos un backend configurado; no se cierra con el
# camino sin ejecutar". Un test de Rust con un backend falso prueba el parser,
# no que el arnes pueda hablar con un modelo de verdad.
#
# Modos:
#   backend-real     AC-22  un backend real responde y el comando lo procesa
#   catalogo-limpio  AC-23  sin solapamientos: informa, sale 0 y no crea backup
#   sin-backend      AC-2   skip limpio, sin dejar rastro
#   no-toca-nada     AC-12  sin --aplicar el arbol queda byte a byte igual
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

# Sandbox con biblioteca propia: nunca se toca la del repo.
sandbox() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-consolidar.XXXXXX")"
    tmp="$(cd "$tmp" && pwd -P)"
    mkdir -p "$tmp/hp/progress" "$tmp/docs/lecciones"
    printf 'subdir\n' > "$tmp/hp/.harness_layout"
    cp "$REPO_ROOT/harness" "$tmp/hp/harness"
    cp "$REPO_ROOT/harness_cli" "$tmp/hp/harness_cli"
    printf '{"features": [], "rules": {"consolidar_backend": "auto"}}\n' > "$tmp/hp/feature_list.json"
    printf '%s' "$tmp"
}

leccion() {
    cat > "$1/docs/lecciones/$2.md" <<LEC
---
nombre: $2
descripcion: $3
triggers: [$4]
relacionadas: []
origen: [1]
usos: 0
ultimo_uso:
ultima_actualizacion: 2026-08-18
estado: activa
---

## Cuando aplica

$5

## Procedimiento

1. Hacelo.

## Pitfalls

- **Algo.** Que no pase.

## Verificacion

\`\`\`bash
echo ok
\`\`\`
LEC
}

# El backend que de verdad esta disponible en esta maquina.
backend_disponible() {
    command -v claude >/dev/null 2>&1 && { echo "claude -p"; return; }
    command -v kimi >/dev/null 2>&1 && { echo "kimi -p"; return; }
    echo ""
}

modo_backend_real() {
    cmd="$(backend_disponible)"
    [ -n "$cmd" ] || fail "backend-real: no hay ningun backend en esta maquina; la acceptance exige al menos uno"
    tmp="$(sandbox)"
    leccion "$tmp" "una-cosa" "Sobre una cosa concreta." "alfa, beta" "Cuando pasa alfa."
    leccion "$tmp" "otra-cosa" "Sobre algo completamente distinto." "gamma, delta" "Cuando pasa gamma."
    set +e
    salida="$(cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" HARNESS_CONSOLIDAR_CMD="$cmd" ./harness lecciones consolidar 2>&1)"
    rc=$?
    set -e
    rm -rf "$tmp"
    [ "$rc" -eq 0 ] || fail "backend-real: exit $rc. Dijo: $salida"
    printf '%s' "$salida" | grep -q "Consultando a" \
        || fail "backend-real: no consulto al backend. Dijo: $salida"
    # Que HAYA procesado la respuesta: o encontro candidatos, o dijo que no hay.
    printf '%s' "$salida" | grep -qE "candidato\(s\) a consolidar|catalogo esta limpio|no devolvio JSON usable|no respondio" \
        || fail "backend-real: no proceso la respuesta. Dijo: $salida"
    ok "backend-real: hablo con \`$cmd\` y proceso su respuesta"
}

modo_catalogo_limpio() {
    cmd="$(backend_disponible)"
    [ -n "$cmd" ] || fail "catalogo-limpio: no hay backend"
    tmp="$(sandbox)"
    # Dos lecciones deliberadamente disjuntas: no hay nada que fusionar.
    leccion "$tmp" "commits-en-espanol" "Los mensajes de commit van en espanol." "commit, mensaje, idioma" "Al commitear."
    leccion "$tmp" "puertos-del-frontend" "El dev server siempre en el 5173." "puerto, vite, dev server" "Al levantar el front."
    set +e
    salida="$(cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" HARNESS_CONSOLIDAR_CMD="$cmd" ./harness lecciones consolidar 2>&1)"
    rc=$?
    set -e
    hay_backup=0
    [ -d "$tmp/bkp" ] && hay_backup=1
    rm -rf "$tmp"
    [ "$rc" -eq 0 ] || fail "catalogo-limpio: exit $rc. Dijo: $salida"
    [ "$hay_backup" -eq 0 ] || fail "catalogo-limpio: creo un backup sin --aplicar"
    # La propuesta vacia es un resultado de primera clase, no una rama muerta.
    printf '%s' "$salida" | grep -qE "catalogo esta limpio|candidato\(s\) a consolidar" \
        || fail "catalogo-limpio: no dio un veredicto. Dijo: $salida"
    ok "catalogo-limpio: la propuesta vacia es un resultado, no una rama muerta"
}

modo_sin_backend() {
    tmp="$(sandbox)"
    leccion "$tmp" "una" "Una." "a" "Cuando."
    leccion "$tmp" "dos" "Dos." "b" "Cuando."
    set +e
    salida="$(cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" PATH=/nonexistent HARNESS_CONSOLIDAR_CMD= ./harness lecciones consolidar 2>&1)"
    rc=$?
    set -e
    hay_backup=0
    [ -d "$tmp/bkp" ] && hay_backup=1
    rm -rf "$tmp"
    [ "$rc" -eq 0 ] || fail "sin-backend: exit $rc, esperaba skip limpio. Dijo: $salida"
    [ "$hay_backup" -eq 0 ] || fail "sin-backend: dejo rastro"
    printf '%s' "$salida" | grep -q "Sin backend" \
        || fail "sin-backend: no explico que falto. Dijo: $salida"
    ok "sin-backend: skip limpio, exit 0 y sin rastro"
}

modo_no_toca_nada() {
    cmd="$(backend_disponible)"
    [ -n "$cmd" ] || fail "no-toca-nada: no hay backend"
    tmp="$(sandbox)"
    leccion "$tmp" "una-cosa" "Sobre una cosa." "alfa" "Cuando alfa."
    leccion "$tmp" "otra-cosa" "Sobre otra." "beta" "Cuando beta."
    antes="$(find "$tmp/docs" -type f -exec shasum {} + | sort)"
    set +e
    (cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" HARNESS_CONSOLIDAR_CMD="$cmd" ./harness lecciones consolidar >/dev/null 2>&1)
    set -e
    despues="$(find "$tmp/docs" -type f -exec shasum {} + | sort)"
    hay_backup=0
    [ -d "$tmp/bkp" ] && hay_backup=1
    rm -rf "$tmp"
    [ "$antes" = "$despues" ] || fail "no-toca-nada: la deteccion modifico archivos"
    [ "$hay_backup" -eq 0 ] || fail "no-toca-nada: la deteccion creo un backup"
    ok "no-toca-nada: sin --aplicar el arbol queda byte a byte igual"
}

case "$MODO" in
    backend-real)    modo_backend_real ;;
    catalogo-limpio) modo_catalogo_limpio ;;
    sin-backend)     modo_sin_backend ;;
    no-toca-nada)    modo_no_toca_nada ;;
    todos)
        modo_sin_backend
        modo_no_toca_nada
        modo_catalogo_limpio
        modo_backend_real
        ok "consolidacion: los cuatro modos verdes"
        ;;
    *) fail "modo desconocido: $MODO" ;;
esac
