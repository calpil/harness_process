#!/bin/bash
# Verifica el gate de bitacoras de los PRD (feature #60, bugs #91 y #92).
#
#   check      AC-9   harness_check.sh REPORTA los pendientes del PRD (puntero
#                     que no resuelve, cierre sin registrar) y NO bloquea por
#                     ellos: un PRD desactualizado no impide trabajar hoy.
#   repo       AC-12  el PRD maestro de ESTE repo no tiene punteros que escapen
#                     de la raiz.
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
MODO="${1:-todos}"
fail() { echo "[!] $1" >&2; exit 1; }
ok() { echo "[Ok] $1"; }

# El binario: el instalado en la raiz, o el compilado del checkout de
# desarrollo (en un worktree de la feature solo existe el segundo).
binario() {
    for candidato in \
        "$REPO_ROOT/harness" \
        "$REPO_ROOT/rust/target/release/harness" \
        "$REPO_ROOT/rust/target/debug/harness"; do
        if [ -x "$candidato" ]; then echo "$candidato"; return 0; fi
    done
    return 1
}

# Sandbox: un arnes en <tmp>/hp con marker subdir, asi que la RAIZ es <tmp>.
montar_sandbox() {
    SANDBOX="$(mktemp -d)"
    mkdir -p "$SANDBOX/hp" "$SANDBOX/docs/prd"
    printf 'subdir' > "$SANDBOX/hp/.harness_layout"
    cp "$(binario)" "$SANDBOX/hp/harness"
    cp "$REPO_ROOT/harness_cli" "$SANDBOX/hp/harness_cli"
    # Backlog con una feature cerrada como done...
    cat > "$SANDBOX/hp/feature_list.json" <<'JSON'
{
  "project": "demo",
  "rules": {},
  "features": [
    {"id": 1, "name": "cobranza", "microservicios": ["demo"], "acceptance": [],
     "status": "done", "closed_at": "2026-08-20T10:00:00Z"}
  ]
}
JSON
    # ...y un PRD que no la registro (bug #91) y que ademas tiene un puntero al
    # worktree que el cierre borro (bug #92).
    cat > "$SANDBOX/docs/prd/PRD-master.md" <<'MD'
# PRD Master - demo

## 10. Hitos -> features

| # | Hito | Slug de feature | Objetivo | Criterio | Estado |
| --- | --- | --- | --- | --- | --- |
| 1 | Cobrar | cobranza | O1 | se cierra | pendiente |

## Bitacora

- #9 vieja -> done 2026-08-01 · spec: ../demo-wt/9-vieja/docs/spec-feature-9-vieja.md
MD
}

modo_check() {
    binario >/dev/null || { ok "check: sin binario compilado (nada que verificar)"; return; }
    montar_sandbox
    trap 'rm -rf "$SANDBOX"' RETURN

    # (a) El informe detecta los dos pendientes sembrados.
    informe="$("$SANDBOX/hp/harness" prd doctor 2>&1 || true)"
    for esperado in "hallazgo" "escapa de la raiz" "sin linea de bitacora"; do
        case "$informe" in
            *"$esperado"*) ;;
            *) fail "check: el informe no menciona '$esperado':
$informe" ;;
        esac
    done

    # (b) El informe NO escribe: el documento queda byte a byte igual.
    antes="$(cat "$SANDBOX/docs/prd/PRD-master.md")"
    "$SANDBOX/hp/harness" prd doctor >/dev/null 2>&1 || true
    [ "$antes" = "$(cat "$SANDBOX/docs/prd/PRD-master.md")" ] \
        || fail "check: el informe modifico el PRD (tiene que ser solo lectura)"

    # (c) harness_check.sh lo REPORTA como informativo, no como fallo: el aviso
    #     sale con [i] y no con [!], que es lo que cuenta para el contador.
    cp "$REPO_ROOT/harness_check.sh" "$SANDBOX/hp/harness_check.sh"
    salida="$(HARNESS_CHECK_MODE=warn bash "$SANDBOX/hp/harness_check.sh" 2>&1 || true)"
    case "$salida" in
        *"Bitacoras de los PRD con pendientes"*) ;;
        *) fail "check: harness_check.sh no reporto los pendientes del PRD:
$salida" ;;
    esac
    case "$salida" in
        *"[!] Bitacoras de los PRD"*)
            fail "check: el aviso del PRD no puede contar como fallo bloqueante" ;;
        *) ;;
    esac
    ok "check: se reporta (informativo, no bloqueante) y el informe no toca el documento"
}

modo_repo() {
    prd="$REPO_ROOT/docs/prd/PRD-master.md"
    [ -f "$prd" ] || { ok "repo: sin PRD maestro"; return; }
    rotos="$(grep -c '· spec: \.\./' "$prd" || true)"
    [ "$rotos" = "0" ] \
        || fail "repo: quedan $rotos puntero(s) al worktree en $prd (corre: sh harness_cli prd doctor --reparar)"
    ok "repo: el PRD maestro no tiene punteros que escapen de la raiz"
}

case "$MODO" in
    check) modo_check ;;
    repo) modo_repo ;;
    todos) modo_check; modo_repo; ok "prd doctor: los modos verdes" ;;
    *) fail "modo desconocido: $MODO (check|repo|todos)" ;;
esac
