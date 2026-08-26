#!/usr/bin/env bash
# Check de consolidación: local y sin cuota por defecto (feature #43).
#
# Uso:
#   tests/consolidar_check.sh [modo]
#   tests/consolidar_check.sh --real [backend-real|todos]
#
# Sin `--real` todos los modos usan el backend falso escrito en el sandbox; no
# importan credenciales ni HARNESS_CONSOLIDAR_CMD heredado. `--real` es la única
# puerta a un CLI autenticado (`claude -p` o `kimi -p`) y queda fuera de CI.
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
HARNESS_BIN="${HARNESS_BIN:-$REPO_ROOT/rust/target/debug/harness}"
REAL=0
if [ "${1:-}" = "--real" ]; then
    REAL=1
    shift
fi
MODO="${1:-todos}"
[ "$#" -le 1 ] || { echo "[!] Uso: $0 [--real] [modo]" >&2; exit 2; }

fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

asegurar_harness() {
    if [ ! -x "$HARNESS_BIN" ]; then
        echo "[i] Compilando harness local para el sandbox..."
        (cd "$REPO_ROOT/rust" && cargo build --quiet)
    fi
    [ -x "$HARNESS_BIN" ] || fail "no se encontro el binario local: $HARNESS_BIN"
}

sandbox() {
    local tmp
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-consolidar.XXXXXX")"
    tmp="$(cd "$tmp" && pwd -P)"
    mkdir -p "$tmp/hp/progress" "$tmp/docs/lecciones"
    printf 'subdir\n' > "$tmp/hp/.harness_layout"
    cp "$HARNESS_BIN" "$tmp/hp/harness"
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

# Recibe el prompt como argv igual que un CLI real, pero responde solo según
# HARNESS_CONSOLIDAR_FAKE. No hay URL, token ni fallback a servicios externos.
backend_falso() {
    local tmp="$1"
    local falso="$tmp/backend-falso.sh"
    cat > "$falso" <<'FAKE'
#!/usr/bin/env sh
case "${HARNESS_CONSOLIDAR_FAKE:-propuesta}" in
  propuesta) printf '%s\n' '{"candidatos":[{"miembros":["una-cosa","otra-cosa"],"motivo":"respuesta controlada","confianza":0.91}]}' ;;
  descarte) printf '%s\n' '{"candidatos":[{"miembros":["una-cosa","no-existe"],"motivo":"respuesta controlada","confianza":0.20}]}' ;;
  malformado) printf '%s\n' '{esto no es json' ;;
  falla) printf '%s\n' 'falla controlada del backend falso' >&2; exit 7 ;;
  *) printf '%s\n' '{"candidatos":[]}' ;;
esac
FAKE
    chmod +x "$falso"
    printf '%s' "$falso"
}

ejecutar_falso() {
    local tmp="$1"
    local caso="$2"
    local falso
    falso="$(backend_falso "$tmp")"
    # Esta asignación por proceso prevalece incluso si el entorno trae un
    # comando real. La ejecución normal jamás invoca ese valor heredado.
    (cd "$tmp/hp" && \
        HARNESS_REPO_ROOT="$tmp" \
        HARNESS_CONSOLIDAR_CMD="$falso" \
        HARNESS_CONSOLIDAR_FAKE="$caso" \
        ./harness lecciones consolidar 2>&1)
}

modo_propuesta() {
    local tmp salida rc
    tmp="$(sandbox)"
    leccion "$tmp" "una-cosa" "Una cosa." "alfa" "Cuando alfa."
    leccion "$tmp" "otra-cosa" "Otra cosa." "beta" "Cuando beta."
    set +e
    salida="$(HARNESS_CONSOLIDAR_CMD='/bin/false no-debe-usarse' ejecutar_falso "$tmp" propuesta)"
    rc=$?
    set -e
    rm -rf "$tmp"
    [ "$rc" -eq 0 ] || fail "propuesta: exit $rc. Dijo: $salida"
    printf '%s' "$salida" | grep -q "una-cosa + otra-cosa" \
        || fail "propuesta: no proceso la respuesta falsa. Dijo: $salida"
    printf '%s' "$salida" | grep -q "triggers/LLM" \
        || fail "propuesta: no conserva la evidencia observable. Dijo: $salida"
    ok "propuesta: backend falso por defecto, aun con variable real heredada"
}

modo_descarte() {
    local tmp salida
    tmp="$(sandbox)"
    leccion "$tmp" "una-cosa" "Una cosa." "alfa" "Cuando alfa."
    leccion "$tmp" "otra-cosa" "Otra cosa." "beta" "Cuando beta."
    salida="$(ejecutar_falso "$tmp" descarte)"
    rm -rf "$tmp"
    printf '%s' "$salida" | grep -q "Candidato descartado" \
        || fail "descarte: no rechazo el miembro falso. Dijo: $salida"
    printf '%s' "$salida" | grep -q "no existe" \
        || fail "descarte: falta diagnostico local. Dijo: $salida"
    ok "descarte: respuesta inválida rechazada localmente"
}

modo_error() {
    local tmp salida rc
    tmp="$(sandbox)"
    leccion "$tmp" "una-cosa" "Una cosa." "alfa" "Cuando alfa."
    leccion "$tmp" "otra-cosa" "Otra cosa." "beta" "Cuando beta."
    set +e
    salida="$(ejecutar_falso "$tmp" malformado)"
    rc=$?
    set -e
    [ "$rc" -eq 0 ] || fail "error/malformado: exit $rc. Dijo: $salida"
    printf '%s' "$salida" | grep -q "no devolvio JSON usable" \
        || fail "error/malformado: falta diagnostico. Dijo: $salida"
    set +e
    salida="$(ejecutar_falso "$tmp" falla)"
    rc=$?
    set -e
    rm -rf "$tmp"
    [ "$rc" -eq 0 ] || fail "error/falla: exit $rc. Dijo: $salida"
    printf '%s' "$salida" | grep -q "falla controlada del backend falso" \
        || fail "error/falla: no expuso la falla local. Dijo: $salida"
    ok "error: malformado y falla del falso no hacen fallback real"
}

modo_paraguas() {
    local tmp salida
    tmp="$(sandbox)"
    leccion "$tmp" "paraguas" "Paraguas." "alfa, beta" "Ver [[miembro]]."
    leccion "$tmp" "miembro" "Miembro." "beta" "Cuando beta."
    salida="$(cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" ./harness lecciones consolidar --aplicar --en paraguas --de miembro --motivo controlado 2>&1)"
    [ -f "$tmp/docs/lecciones/archivo/miembro.md" ] \
        || fail "paraguas: no archivo la miembro en el sandbox. Dijo: $salida"
    rm -rf "$tmp"
    printf '%s' "$salida" | grep -q "Consolidacion aplicada" \
        || fail "paraguas: no verifico el contrato de fusion. Dijo: $salida"
    ok "paraguas: la fusión aislada conserva su contrato"
}

modo_no_toca_nada() {
    local tmp antes despues salida
    tmp="$(sandbox)"
    leccion "$tmp" "una-cosa" "Una cosa." "alfa" "Cuando alfa."
    leccion "$tmp" "otra-cosa" "Otra cosa." "beta" "Cuando beta."
    antes="$(find "$tmp/docs" -type f -exec shasum {} + | sort)"
    salida="$(ejecutar_falso "$tmp" propuesta)"
    despues="$(find "$tmp/docs" -type f -exec shasum {} + | sort)"
    [ "$antes" = "$despues" ] || fail "no-toca-nada: el falso modifico documentos"
    [ ! -d "$tmp/bkp" ] || fail "no-toca-nada: creo un backup sin --aplicar"
    rm -rf "$tmp"
    printf '%s' "$salida" | grep -q "candidato(s) a consolidar" \
        || fail "no-toca-nada: no ejercio la propuesta. Dijo: $salida"
    ok "no-toca-nada: propuesta falsa sin escrituras ni backup"
}

backend_disponible() {
    command -v claude >/dev/null 2>&1 && { echo "claude -p"; return; }
    command -v kimi >/dev/null 2>&1 && { echo "kimi -p"; return; }
    echo ""
}

modo_backend_real() {
    local cmd tmp salida rc
    cmd="$(backend_disponible)"
    [ -n "$cmd" ] || fail "--real requiere un CLI autenticado (claude o kimi). Sin --real la suite es local y no requiere secretos."
    tmp="$(sandbox)"
    leccion "$tmp" "una-cosa" "Una cosa." "alfa" "Cuando alfa."
    leccion "$tmp" "otra-cosa" "Otra cosa." "beta" "Cuando beta."
    set +e
    salida="$(cd "$tmp/hp" && HARNESS_REPO_ROOT="$tmp" HARNESS_CONSOLIDAR_CMD="$cmd" ./harness lecciones consolidar 2>&1)"
    rc=$?
    set -e
    rm -rf "$tmp"
    [ "$rc" -eq 0 ] || fail "backend-real: exit $rc. Dijo: $salida"
    printf '%s' "$salida" | grep -q "Consultando a" \
        || fail "backend-real: no consulto al backend. Dijo: $salida"
    ok "backend-real: integración explícita con \`$cmd\`"
}

if [ "$REAL" -eq 1 ]; then
    case "$MODO" in
        backend-real|todos) modo_backend_real ;;
        *) fail "--real solo admite backend-real o todos; requisitos: CLI autenticado claude/kimi" ;;
    esac
    exit 0
fi

asegurar_harness

case "$MODO" in
    propuesta) modo_propuesta ;;
    descarte) modo_descarte ;;
    error) modo_error ;;
    paraguas) modo_paraguas ;;
    no-toca-nada) modo_no_toca_nada ;;
    todos)
        modo_propuesta
        modo_descarte
        modo_error
        modo_paraguas
        modo_no_toca_nada
        ok "consolidacion: suite local completa, sin red ni cuota"
        ;;
    *) fail "modo desconocido: $MODO (propuesta|descarte|error|paraguas|no-toca-nada|todos)" ;;
esac
