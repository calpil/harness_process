#!/bin/bash
# Verifica el bloque de conventions de harness_check.sh (feature #24).
#
# Cuatro modos, uno por AC, para que cada criterio tenga un comando propio que
# pueda fallar (leccion `criterios-de-cierre-que-se-pueden-fallar`):
#
#   sin-violaciones  AC-8   la suite real no tiene ningun test que lea el fuente
#   detecta-en-src   #36    la violacion sembrada en rust/src/ tambien se reporta
#   detecta          AC-10  ante una violacion sembrada, la reporta con archivo,
#                           linea y nombre del test  <- la PRUEBA DEL ROJO
#   no-bloquea       AC-11  con la violacion presente, el exit code no cambia
#   sin-rust         AC-12  sin rust/tests/ el bloque se omite sin ruido
#
# El modo `detecta` es el que hace que este script valga: sin el, "no reporto
# nada" seria indistinguible de "no sabe reportar".
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"

fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

# La violacion se siembra en un archivo propio y se borra siempre, tambien si el
# assert falla: el repo no puede quedar sucio por un test.
SEMBRADO="$REPO_ROOT/rust/tests/zz_conventions_fixture.rs"
# La unica violacion historica de la regla 2 estaba en un test UNITARIO, dentro
# de rust/src/, o sea fuera del alcance original del chequeo (feature #36).
SEMBRADO_SRC="$REPO_ROOT/rust/src/zz_conventions_fixture.rs"
sembrar_violacion() {
    cat > "$SEMBRADO" <<'RUST'
//! Fixture temporal de tests/conventions_check.sh. Se borra al terminar.
#[test]
fn fixture_que_lee_el_fuente() {
    let texto = std::fs::read_to_string("src/cli.rs").unwrap_or_default();
    assert!(!texto.is_empty() || texto.is_empty());
}
RUST
}
sembrar_violacion_en_src() {
    cat > "$SEMBRADO_SRC" <<'RUST'
//! Fixture temporal de tests/conventions_check.sh. Se borra al terminar.
#[cfg(test)]
mod tests {
    #[test]
    fn fixture_unitario_que_lee_el_fuente() {
        let texto = std::fs::read_to_string("src/cli.rs").unwrap_or_default();
        assert!(texto.is_empty() || !texto.is_empty());
    }
}
RUST
}
limpiar() { rm -f "$SEMBRADO" "$SEMBRADO_SRC"; }
trap limpiar EXIT

correr_check() {
    # `warn` para que un problema ajeno al bloque no enmascare lo que se mide.
    HARNESS_CHECK_MODE=warn bash "$REPO_ROOT/harness_check.sh" 2>&1 || true
}

modo_sin_violaciones() {
    salida="$(correr_check)"
    if printf '%s' "$salida" | grep -q "lee un archivo fuente"; then
        printf '%s\n' "$salida" | grep "lee un archivo fuente" >&2
        fail "la suite real tiene tests que leen el fuente (regla 2 de docs/conventions.md)"
    fi
    ok "sin-violaciones: ningun test de rust/tests/ lee un archivo fuente"
}

modo_detecta() {
    sembrar_violacion
    salida="$(correr_check)"
    linea="$(printf '%s\n' "$salida" | grep "lee un archivo fuente" || true)"
    limpiar
    [ -n "$linea" ] || fail "detecta: el chequeo NO reporto la violacion sembrada (no verifica nada)"
    printf '%s' "$linea" | grep -q "zz_conventions_fixture.rs" \
        || fail "detecta: no nombra el archivo. Reporto: $linea"
    printf '%s' "$linea" | grep -qE ':[0-9]+ ' \
        || fail "detecta: no nombra la linea. Reporto: $linea"
    printf '%s' "$linea" | grep -q "fixture_que_lee_el_fuente" \
        || fail "detecta: no nombra el test. Reporto: $linea"
    printf '%s' "$linea" | grep -q "conventions.md" \
        || fail "detecta: no nombra la regla. Reporto: $linea"
    ok "detecta: reporta archivo, linea, nombre del test y la regla"
}

modo_detecta_en_src() {
    sembrar_violacion_en_src
    salida="$(correr_check)"
    limpiar
    linea="$(printf '%s\n' "$salida" | grep "lee un archivo fuente" || true)"
    [ -n "$linea" ] || fail "detecta-en-src: no reporto la violacion sembrada en rust/src/"
    printf '%s' "$linea" | grep -q "zz_conventions_fixture.rs" \
        || fail "detecta-en-src: no nombra el archivo. Reporto: $linea"
    printf '%s' "$linea" | grep -q "fixture_unitario_que_lee_el_fuente" \
        || fail "detecta-en-src: no nombra el test. Reporto: $linea"
    ok "detecta-en-src: los tests unitarios de rust/src/ tambien se revisan"
}

modo_no_bloquea() {
    limpiar
    set +e
    HARNESS_CHECK_MODE=warn bash "$REPO_ROOT/harness_check.sh" >/dev/null 2>&1
    rc_limpio=$?
    sembrar_violacion
    HARNESS_CHECK_MODE=warn bash "$REPO_ROOT/harness_check.sh" >/dev/null 2>&1
    rc_sucio=$?
    set -e
    limpiar
    [ "$rc_limpio" = "$rc_sucio" ] \
        || fail "no-bloquea: el exit code cambio de $rc_limpio a $rc_sucio con una violacion presente"
    ok "no-bloquea: el aviso no cambia el exit code (sigue en $rc_limpio)"
}

modo_sin_rust() {
    # Un proyecto que no es Rust: se copia lo minimo para que harness_check
    # corra, SIN rust/tests/.
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/harness-conv.XXXXXX")"
    trap 'rm -rf "$tmp"; limpiar' EXIT
    mkdir -p "$tmp/hp" "$tmp/docs" "$tmp/progress"
    cp "$REPO_ROOT/harness_check.sh" "$tmp/hp/harness_check.sh"
    cp "$REPO_ROOT/harness_cli" "$tmp/hp/harness_cli"
    cp "$REPO_ROOT/harness" "$tmp/hp/harness" 2>/dev/null || true
    printf 'subdir\n' > "$tmp/hp/.harness_layout"
    printf '# Constitution\n' > "$tmp/docs/constitution.md"
    salida="$(cd "$tmp/hp" && HARNESS_CHECK_MODE=warn bash ./harness_check.sh 2>&1 || true)"
    rm -rf "$tmp"
    trap limpiar EXIT
    if printf '%s' "$salida" | grep -q "lee un archivo fuente\|conventions.md"; then
        fail "sin-rust: el bloque hablo en un repo sin rust/tests/"
    fi
    ok "sin-rust: sin rust/tests/ el bloque se omite sin ruido"
}

case "$MODO" in
    sin-violaciones) modo_sin_violaciones ;;
    detecta-en-src)  modo_detecta_en_src ;;
    detecta)         modo_detecta ;;
    no-bloquea)      modo_no_bloquea ;;
    sin-rust)        modo_sin_rust ;;
    todos)
        modo_sin_violaciones
        modo_detecta_en_src
        modo_detecta
        modo_no_bloquea
        modo_sin_rust
        ok "conventions check: los cuatro modos verdes"
        ;;
    *) fail "modo desconocido: $MODO (sin-violaciones | detecta | no-bloquea | sin-rust | todos)" ;;
esac
