#!/bin/bash
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/harness-setup-smoke.XXXXXX")"
TMP_ROOT="$(cd "$TMP_ROOT" && pwd -P)"
trap 'rm -rf "$TMP_ROOT"' EXIT

# Diseño Rust-only: el binario es requerido, asi que el smoke tambien
# requiere cargo. Se compila UNA vez y se siembra en cada fixture (rama
# "binario preexistente" del instalador; el build real se prueba al final).
command -v cargo >/dev/null 2>&1 || { echo "[!] cargo es requerido para el smoke (harness Rust-only)." >&2; exit 2; }
(cd "$REPO_ROOT/rust" && cargo build --release --quiet)
PREBUILT_BIN="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}/release/harness"
test -x "$PREBUILT_BIN"

copy_fixture() {
    target="$1"
    mkdir -p "$target"
    cp "$REPO_ROOT/setup_harness.sh" "$target/setup_harness.sh"
    cp -R "$REPO_ROOT/templates" "$target/templates"
    cp "$PREBUILT_BIN" "$target/harness"
    chmod +x "$target/harness"
}

copy_flat_fixture() {
    target="$1"
    mkdir -p "$target"
    cp "$REPO_ROOT/setup_harness.sh" "$target/setup_harness.sh"
    cp -R "$REPO_ROOT/templates/." "$target/"
    cp "$PREBUILT_BIN" "$target/harness"
    chmod +x "$target/harness"
}

run_setup() {
    target="$1"
    shift
    (
        cd "$target"
        HOME="$TMP_ROOT/home" \
        HARNESS_HUB="$target/.test-hub" \
                DB_HOST=postgres.example \
        DB_USER=harness \
        DB_PASSWORD=secret \
        DB_NAME=harness \
        DB_SSL_MODE=require \
        bash setup_harness.sh \
            --no-graphify \
            --no-graphify-skills \
            --no-antigravity \
            "$@"
    )
}

POSTGRES_PREFLIGHT="$TMP_ROOT/postgres-preflight"
copy_fixture "$POSTGRES_PREFLIGHT"
if (
    unset DB_HOST DB_USER DB_PASSWORD DB_NAME
    cd "$POSTGRES_PREFLIGHT"
    HOME="$TMP_ROOT/empty-home" \
    HARNESS_HUB="$TMP_ROOT/empty-hub" \
    bash setup_harness.sh \
        --root \
        --no-graphify \
        --no-graphify-skills \
        --no-antigravity
) >/dev/null 2>&1; then
    echo "[!] El setup PostgreSQL debio fallar sin credenciales." >&2
    exit 1
fi
test ! -e "$POSTGRES_PREFLIGHT/.harness_layout"

POSTGRES_DEFAULT="$TMP_ROOT/postgres-default"
copy_fixture "$POSTGRES_DEFAULT"
# Credenciales SOLO via $HARNESS_HUB/.env, con un password lleno de
# metacaracteres: el setup debe PARSEARLO (sourcearlo abortaba en silencio).
mkdir -p "$TMP_ROOT/postgres-hub"
cat > "$TMP_ROOT/postgres-hub/.env" <<'ENVEOF'
# comentario y linea vacia a proposito

DB_HOST=postgres.example
DB_USER=harness
DB_PASSWORD=we!rd)pa'ss$(word)&;`uh
DB_SSL_MODE=require
ENVEOF
(
    cd "$TMP_ROOT"
    env -u DB_HOST -u DB_USER -u DB_PASSWORD -u DB_NAME -u DB_SSL_MODE \
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/postgres-hub" \
    bash "$POSTGRES_DEFAULT/setup_harness.sh" \
        --root \
        --no-graphify \
        --no-graphify-skills \
        --no-antigravity
)
grep -qx 'postgres' "$POSTGRES_DEFAULT/.harness_backend"
test -x "$POSTGRES_DEFAULT/harness_cli"
test -f "$POSTGRES_DEFAULT/harness_cli.ps1"
test -x "$POSTGRES_DEFAULT/harness"
# El shim despacha al binario; status es 100% local (no toca la DB).
sh "$POSTGRES_DEFAULT/harness_cli" status | grep '^Backlog:' >/dev/null

FLAT_LAYOUT="$TMP_ROOT/flat-layout"
copy_flat_fixture "$FLAT_LAYOUT"
(
    cd "$TMP_ROOT"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$FLAT_LAYOUT/.test-hub" \
        DB_HOST=postgres.example \
    DB_USER=harness \
    DB_PASSWORD=secret \
    DB_NAME=harness \
    DB_SSL_MODE=require \
    bash "$FLAT_LAYOUT/setup_harness.sh" \
        --root \
        --no-graphify \
        --no-graphify-skills \
        --no-antigravity
)
test ! -d "$FLAT_LAYOUT/templates"
test -x "$FLAT_LAYOUT/harness_cli"
test -f "$FLAT_LAYOUT/harness_cli.ps1"
test -f "$FLAT_LAYOUT/roles/leader.md"
test -f "$FLAT_LAYOUT/.codex/hooks.json"
sh "$FLAT_LAYOUT/harness_cli" status | grep '^Backlog:' >/dev/null

NO_SUBAGENTS="$TMP_ROOT/no-subagents"
copy_fixture "$NO_SUBAGENTS"
run_setup "$NO_SUBAGENTS" --root --no-subagents
test ! -d "$NO_SUBAGENTS/roles"
test ! -d "$NO_SUBAGENTS/.codex/agents"
! grep -q 'roles/README' "$NO_SUBAGENTS/AGENTS.md"
grep -q 'WITH_SUBAGENTS="0"' "$NO_SUBAGENTS/bin/harness-hook"

ROOT_LAYOUT="$TMP_ROOT/root-layout"
copy_fixture "$ROOT_LAYOUT"
run_setup "$ROOT_LAYOUT" --root

# test -f "$ROOT_LAYOUT/graph_memory.py"  # py removed
test -x "$ROOT_LAYOUT/harness_cli"
test -f "$ROOT_LAYOUT/harness_cli.ps1"
test -f "$ROOT_LAYOUT/AGENTS.md"
test -f "$ROOT_LAYOUT/.codex/hooks.json"
test -d "$ROOT_LAYOUT/templates"
# Constitution SDD sembrada en el docs/ de la RAIZ (en root, RAIZ == harness dir).
test -f "$ROOT_LAYOUT/docs/constitution.md"
grep -qx 'postgres' "$ROOT_LAYOUT/.harness_backend"
# Hooks y superficies deben invocar el shim, no python3 directo.
grep -Fq 'harness_cli\" nudge' "$ROOT_LAYOUT/.claude/settings.json"
grep -Fq 'harness_cli" graph mapa' "$ROOT_LAYOUT/AGENTS.md"
python3 -m json.tool "$ROOT_LAYOUT/.codex/hooks.json" >/dev/null
python3 -m json.tool "$ROOT_LAYOUT/.gemini/settings.json" >/dev/null
python3 -c 'import pathlib, tomllib; [tomllib.loads(p.read_text()) for p in pathlib.Path("'"$ROOT_LAYOUT"'/.codex/agents").glob("*.toml")]'
grep -Fq "$ROOT_LAYOUT/bin/harness-hook" "$ROOT_LAYOUT/.codex/hooks.json"
git init -q "$ROOT_LAYOUT/svc-demo"
# DB inexistente en localhost: rechazo instantaneo (sin timeout de 10s).
# Los hooks se conectan ANTES de los comandos graph, que aqui fallan rapido.
HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$ROOT_LAYOUT/.test-hub" \
    DB_HOST=127.0.0.1 \
    DB_PORT=9 \
    DB_USER=harness \
    DB_PASSWORD=secret \
    DB_NAME=harness \
    DB_SSL_MODE=disable \
    bash "$ROOT_LAYOUT/init.sh" >/dev/null 2>&1 || true
# El hook post-commit conectado debe ser v9 y pasar por el shim.
grep -q 'harness-managed-hook v9' "$ROOT_LAYOUT/svc-demo/.git/hooks/post-commit"
grep -Fq 'harness_cli" graph sync_git' "$ROOT_LAYOUT/svc-demo/.git/hooks/post-commit"

SUBDIR_ROOT="$TMP_ROOT/subdir-layout"
SUBDIR_HARNESS="$SUBDIR_ROOT/harness_process"
copy_fixture "$SUBDIR_HARNESS"
run_setup "$SUBDIR_HARNESS"

# test -f "$SUBDIR_HARNESS/graph_memory.py"  # py removed
test -x "$SUBDIR_HARNESS/harness_cli"
test -f "$SUBDIR_HARNESS/harness_cli.ps1"
test -f "$SUBDIR_ROOT/AGENTS.md"
test -f "$SUBDIR_ROOT/bin/harness-hook"
test -d "$SUBDIR_HARNESS/templates"
grep -qx 'postgres' "$SUBDIR_HARNESS/.harness_backend"
# Constitution SDD en el docs/ de la RAIZ multi-repo, NO en la subcarpeta del arnes.
test -f "$SUBDIR_ROOT/docs/constitution.md"
# Feature #4 / AC-1: TODA la doc del proceso vive en el docs/ de la RAIZ. Los tres
# docs del arnes se instalan ahi y la subcarpeta ya no tiene docs/ propio.
test -f "$SUBDIR_ROOT/docs/architecture.md"
test -f "$SUBDIR_ROOT/docs/conventions.md"
test -f "$SUBDIR_ROOT/docs/verification.md"
test ! -e "$SUBDIR_HARNESS/docs/architecture.md"
test ! -e "$SUBDIR_HARNESS/docs/conventions.md"
test ! -e "$SUBDIR_HARNESS/docs/verification.md"
test ! -d "$SUBDIR_HARNESS/docs"
# Feature #5 / AC-1: planillas maestras PRD y SDD en docs/prd/ de la RAIZ.
test -f "$SUBDIR_ROOT/docs/prd/PRD-master.md"
test -f "$SUBDIR_ROOT/docs/prd/SDD-master.md"
test ! -d "$SUBDIR_HARNESS/docs/prd"
# AC-7 / AC-8: las planillas traen las secciones que las hacen utiles.
grep -q '^## 7. Hitos -> features' "$SUBDIR_ROOT/docs/prd/PRD-master.md"
grep -q 'harness_cli add' "$SUBDIR_ROOT/docs/prd/PRD-master.md"
grep -q '^## 4. Decisiones tecnicas' "$SUBDIR_ROOT/docs/prd/SDD-master.md"
grep -q 'docs/architecture.md' "$SUBDIR_ROOT/docs/prd/SDD-master.md"
grep -q 'harness_process/init.sh' "$SUBDIR_ROOT/AGENTS.md"
grep -Fq 'harness_process/harness_cli" graph mapa' "$SUBDIR_ROOT/AGENTS.md"
grep -Fq "$SUBDIR_ROOT/bin/harness-hook" "$SUBDIR_ROOT/.codex/hooks.json"
mkdir -p "$SUBDIR_ROOT/service"
codex_start="$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["hooks"]["SessionStart"][0]["hooks"][0]["command"])' "$SUBDIR_ROOT/.codex/hooks.json")"
(
    cd "$SUBDIR_ROOT/service"
    HOME="$TMP_ROOT/home" \
        HARNESS_HUB="$SUBDIR_HARNESS/.test-hub" \
                DB_HOST=postgres.example \
        DB_USER=harness \
        DB_PASSWORD=secret \
        DB_NAME=harness \
        DB_SSL_MODE=require \
        bash -c "$codex_start" >/dev/null 2>&1
)

printf 'contenido previo\n' > "$SUBDIR_ROOT/AGENTS.md"
# No-pisa: la constitution es documento del USUARIO; el instalador la siembra
# solo-si-falta. Un sentinel debe SOBREVIVIR al reinstall de abajo.
CONST_SENTINEL="SENTINEL-CONSTITUTION-NO-PISA-$$"
printf '\n<!-- %s -->\n' "$CONST_SENTINEL" >> "$SUBDIR_ROOT/docs/constitution.md"
# Feature #4 / AC-4 (reinstall): los docs del arnes comparten carpeta con la
# documentacion del equipo, asi que siguen la misma regla que la constitution
# (sembrar solo-si-falta). Un sentinel debe SOBREVIVIR al reinstall.
DOCS_SENTINEL="SENTINEL-DOCS-ARNES-NO-PISA-$$"
printf '\n<!-- %s -->\n' "$DOCS_SENTINEL" >> "$SUBDIR_ROOT/docs/conventions.md"
# Feature #5 / AC-3: el PRD del proyecto es del USUARIO; el reinstall no lo pisa.
PRD_SENTINEL="SENTINEL-PRD-NO-PISA-$$"
printf '\n<!-- %s -->\n' "$PRD_SENTINEL" >> "$SUBDIR_ROOT/docs/prd/PRD-master.md"
CUSTOM_BKP="$TMP_ROOT/custom-backups"
(
    cd "$SUBDIR_HARNESS"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$SUBDIR_HARNESS/.test-hub" \
    HARNESS_BKP_DIR="$CUSTOM_BKP" \
        DB_HOST=postgres.example \
    DB_USER=harness \
    DB_PASSWORD=secret \
    DB_NAME=harness \
    DB_SSL_MODE=require \
    bash setup_harness.sh \
        --no-graphify \
        --no-graphify-skills \
        --no-antigravity
)
find "$CUSTOM_BKP" -type f -name 'AGENTS.md.bak.*' -print -quit | grep -q .
# El reinstall NO pisa la constitution existente: el sentinel sigue ahi.
grep -q "$CONST_SENTINEL" "$SUBDIR_ROOT/docs/constitution.md"
# Feature #4 / AC-4: tampoco pisa los docs del arnes ya presentes en la raiz.
grep -q "$DOCS_SENTINEL" "$SUBDIR_ROOT/docs/conventions.md"
# Feature #5 / AC-3: el PRD ya escrito sobrevive intacto al reinstall.
grep -q "$PRD_SENTINEL" "$SUBDIR_ROOT/docs/prd/PRD-master.md"

# --- Feature #4 / AC-3 + AC-4: migracion de instalaciones previas -----------
# Fixture con los docs del arnes en la ubicacion VIEJA (<harness>/docs/). El
# instalador debe MOVER los que faltan en la raiz y CONSERVAR intacto (sin pisar
# ni mover encima) el que el equipo ya tiene en la raiz.
MIGRATE_ROOT="$TMP_ROOT/migrate-layout"
MIGRATE_HARNESS="$MIGRATE_ROOT/harness_process"
copy_fixture "$MIGRATE_HARNESS"
mkdir -p "$MIGRATE_HARNESS/docs" "$MIGRATE_ROOT/docs"
printf 'VIEJO-ARCHITECTURE\n' > "$MIGRATE_HARNESS/docs/architecture.md"
printf 'VIEJO-VERIFICATION\n' > "$MIGRATE_HARNESS/docs/verification.md"
printf 'VIEJO-CONVENTIONS\n' > "$MIGRATE_HARNESS/docs/conventions.md"
TEAM_SENTINEL="SENTINEL-CONVENTIONS-DEL-EQUIPO-$$"
printf '%s\n' "$TEAM_SENTINEL" > "$MIGRATE_ROOT/docs/conventions.md"
run_setup "$MIGRATE_HARNESS" > "$TMP_ROOT/migrate.log" 2>&1
# AC-3: los que faltaban en la raiz se movieron con su contenido (no se
# regeneraron desde la plantilla) y desaparecieron de la subcarpeta.
grep -qx 'VIEJO-ARCHITECTURE' "$MIGRATE_ROOT/docs/architecture.md"
grep -qx 'VIEJO-VERIFICATION' "$MIGRATE_ROOT/docs/verification.md"
test ! -e "$MIGRATE_HARNESS/docs/architecture.md"
test ! -e "$MIGRATE_HARNESS/docs/verification.md"
# AC-4: el doc del equipo en la raiz queda intacto y la copia vieja se conserva
# (no se pisa ni se borra nada); ademas el instalador lo avisa.
grep -qx "$TEAM_SENTINEL" "$MIGRATE_ROOT/docs/conventions.md"
grep -qx 'VIEJO-CONVENTIONS' "$MIGRATE_HARNESS/docs/conventions.md"
grep -q 'ya existe' "$TMP_ROOT/migrate.log"
grep -q 'Migrado al docs/ de la raiz' "$TMP_ROOT/migrate.log"

# --- E2E: gate de spec (SDD) con el binario ya sembrado --------------------
# Fixture root-layout dedicado. La regla require_spec_approved la trae el
# template ya sembrado; el gate se ejercita de punta a punta. DB inalcanzable
# (rechazo TCP instantaneo, mismo patron que el init.sh de ROOT_LAYOUT): el hub
# es best-effort y NO altera los exit codes del gate. HARNESS_REPO_ROOT ancla el
# binario al fixture (evita markers residuales del entorno).
SPEC_E2E="$TMP_ROOT/spec-e2e"
copy_fixture "$SPEC_E2E"
run_setup "$SPEC_E2E" --root
test -f "$SPEC_E2E/docs/constitution.md"
# El template sembrado trae la regla activa (require_spec_approved: true).
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if d.get("rules",{}).get("require_spec_approved") is True else 1)' "$SPEC_E2E/feature_list.json"

spec_cli() {
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$SPEC_E2E/.test-hub" \
    HARNESS_REPO_ROOT="$SPEC_E2E" \
    DB_HOST=127.0.0.1 \
    DB_PORT=9 \
    DB_USER=harness \
    DB_PASSWORD=secret \
    DB_NAME=harness \
    DB_SSL_MODE=disable \
    sh "$SPEC_E2E/harness_cli" "$@"
}

spec_cli add --name demo >/dev/null 2>&1
spec_cli start --feature 1 >/dev/null 2>&1
# start siembra el spec DRAFT en el docs/ de la RAIZ, junto al plan (layout plano).
SPEC_FILE="$SPEC_E2E/docs/spec-feature-1-demo.md"
test -f "$SPEC_FILE"
grep -q '^Estado: draft' "$SPEC_FILE"

# Regla activa + spec draft => advance BLOQUEA (el gate corre antes del hub).
rc=0; spec_cli advance --nota "sin aprobar" --no-graphify >/dev/null 2>&1 || rc=$?
test "$rc" -ne 0 || { echo "[!] advance debio bloquear con el spec en draft." >&2; exit 1; }
# check-spec => rc 2 con el spec sin aprobar.
rc=0; spec_cli check-spec >/dev/null 2>&1 || rc=$?
test "$rc" -eq 2 || { echo "[!] check-spec debio dar rc=2 con spec draft (rc=$rc)." >&2; exit 1; }

# AC-3: sin la confirmacion explicita del usuario, approve-spec SE NIEGA (rc 2)
# y el spec queda intacto en draft. Es la barrera del Articulo 2 en codigo.
rc=0; spec_cli approve-spec >/dev/null 2>&1 || rc=$?
test "$rc" -eq 2 || { echo "[!] approve-spec sin --yes debio dar rc=2 (rc=$rc)." >&2; exit 1; }
grep -q '^Estado: draft' "$SPEC_FILE" \
    || { echo "[!] approve-spec sin --yes NO debe tocar el spec." >&2; exit 1; }

# AC-1: el usuario dice que si (en uso real: tras ver el spec en el chat y en su
# editor) y el agente lo REGISTRA. El comando escribe el estado y el sello.
rc=0; spec_cli approve-spec --yes --nota "aprobado en el chat" >/dev/null 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] approve-spec --yes debio dar rc=0 (rc=$rc)." >&2; exit 1; }
grep -q '^Estado: approved' "$SPEC_FILE"
grep -q '^Aprobado: .* por USUARIO (confirmacion explicita) - aprobado en el chat' "$SPEC_FILE" \
    || { echo "[!] falta el sello de aprobacion en el spec." >&2; exit 1; }

# AC-2: check-spec sale limpio INMEDIATAMENTE (sin advance de por medio). Antes,
# aprobar a mano dejaba el spec stale y check-spec gritaba "otro LLM".
rc=0; spec_cli check-spec >/dev/null 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] tras approve-spec, check-spec debio dar rc=0 sin re-firma manual (rc=$rc)." >&2; exit 1; }

# AC-4: idempotente (no duplica el sello).
spec_cli approve-spec --yes >/dev/null 2>&1
test "$(grep -c '^Aprobado: ' "$SPEC_FILE")" -eq 1 \
    || { echo "[!] approve-spec duplico el sello de aprobacion." >&2; exit 1; }

# AC-10: la superficie sembrada describe el flujo nuevo, no el manual.
grep -q 'approve-spec' "$SPEC_E2E/docs/constitution.md" \
    || { echo "[!] la constitution sembrada no menciona approve-spec." >&2; exit 1; }
grep -q 'auto-aprobar' "$SPEC_E2E/docs/constitution.md" \
    && { echo "[!] la constitution sembrada conserva el texto del flujo viejo." >&2; exit 1; }
grep -q 'approve-spec' "$SPEC_E2E/harness_check.sh" \
    || { echo "[!] harness_check.sh sembrado no menciona approve-spec." >&2; exit 1; }

# Aprobado => advance PASA (el fallo del hub best-effort no altera el exit code).
rc=0; spec_cli advance --nota "ok" --no-graphify >/dev/null 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] advance debio pasar con el spec aprobado (rc=$rc)." >&2; exit 1; }
# check-spec => rc 0 con el spec aprobado y fresco.
rc=0; spec_cli check-spec >/dev/null 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] check-spec debio dar rc=0 con spec aprobado (rc=$rc)." >&2; exit 1; }
echo "[Ok] gate de spec (SDD): draft bloquea advance/check-spec; aprobado abre; constitution sembrada + no-pisa."
echo "[Ok] approve-spec: exige --yes, sella la aprobacion del usuario, re-firma (check-spec limpio) y es idempotente."

if bash "$REPO_ROOT/setup_harness.sh" --json-hub >/dev/null 2>&1; then
    echo "[!] --json-hub ya no debe estar soportado." >&2
    exit 1
fi

# --- Nuevas pruebas para mejoras 2026 ---
DRY_TEST="$TMP_ROOT/dry-run-test"
copy_fixture "$DRY_TEST"
(
    cd "$DRY_TEST"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/dry-hub" \
        DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity --dry-run --json > /tmp/dry.json 2>&1
)
grep -q '"dry_run": true' /tmp/dry.json || { echo "[!] --dry-run no emitio JSON correcto"; exit 1; }
test ! -f "$DRY_TEST/.harness_layout"   # nada debe haberse escrito

VERSION_OUT=$(bash "$REPO_ROOT/setup_harness.sh" --version)
test -n "$VERSION_OUT"

# Reset en temp: NO debe borrar la constitution del usuario (layout root).
RESET_TEST="$TMP_ROOT/reset-test"
copy_fixture "$RESET_TEST"
(
    cd "$RESET_TEST"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/reset-hub" \
        DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity >/dev/null 2>&1 || true
)
# La constitution es un documento del USUARIO: se sembro y el usuario la edita.
test -f "$RESET_TEST/docs/constitution.md"
RESET_SENTINEL="SENTINEL-CONSTITUTION-RESET-$$"
printf '\n<!-- %s -->\n' "$RESET_SENTINEL" >> "$RESET_TEST/docs/constitution.md"
# Feature #4 / AC-6: los artefactos de feature comparten carpeta con los docs
# generados; el reset NO puede llevarselos por delante.
printf '# spec\n' > "$RESET_TEST/docs/spec-feature-1-demo.md"
printf '# plan\n' > "$RESET_TEST/docs/plan-feature-1-demo.md"
printf '# review\n' > "$RESET_TEST/docs/review-1.md"
# Feature #5 / AC-4: las planillas maestras se sembraron y el usuario las
# completo; el reset NO puede borrarlas (no son superficie generada).
test -f "$RESET_TEST/docs/prd/PRD-master.md"
test -f "$RESET_TEST/docs/prd/SDD-master.md"
PRD_RESET_SENTINEL="SENTINEL-PRD-RESET-$$"
printf '\n<!-- %s -->\n' "$PRD_RESET_SENTINEL" >> "$RESET_TEST/docs/prd/PRD-master.md"
# Ahora reset (respalda y limpia SOLO las superficies/docs generados)
(
    cd "$RESET_TEST"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/reset-hub" \
        DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity --reset >/dev/null 2>&1
)
# Garantia "un reinstall NUNCA lo pisa": el reset conserva la constitution editada.
grep -q "$RESET_SENTINEL" "$RESET_TEST/docs/constitution.md" \
    || { echo "[FALLO] reset borro la constitution del usuario (layout root)"; exit 1; }
# ...pero SI limpia los docs generados (architecture.md viene de templates/docs/).
test ! -f "$RESET_TEST/docs/architecture.md" \
    || { echo "[FALLO] reset no limpio el doc generado architecture.md"; exit 1; }
test ! -f "$RESET_TEST/docs/conventions.md" \
    || { echo "[FALLO] reset no limpio el doc generado conventions.md"; exit 1; }
test ! -f "$RESET_TEST/docs/verification.md" \
    || { echo "[FALLO] reset no limpio el doc generado verification.md"; exit 1; }
# Feature #4 / AC-6: los artefactos de feature sobreviven al reset.
for artifact in spec-feature-1-demo.md plan-feature-1-demo.md review-1.md; do
    test -f "$RESET_TEST/docs/$artifact" \
        || { echo "[FALLO] reset borro el artefacto de feature $artifact"; exit 1; }
done
# Feature #5 / AC-4: las planillas maestras PRD/SDD sobreviven al reset, con el
# contenido que escribio el usuario.
grep -q "$PRD_RESET_SENTINEL" "$RESET_TEST/docs/prd/PRD-master.md" \
    || { echo "[FALLO] reset borro o piso el PRD del proyecto"; exit 1; }
test -f "$RESET_TEST/docs/prd/SDD-master.md" \
    || { echo "[FALLO] reset borro el SDD master del proyecto"; exit 1; }
# Reinstall tras reset: la siembra if-missing tampoco pisa la constitution.
(
    cd "$RESET_TEST"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/reset-hub" \
        DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity >/dev/null 2>&1 || true
)
grep -q "$RESET_SENTINEL" "$RESET_TEST/docs/constitution.md" \
    || { echo "[FALLO] reinstall tras reset piso la constitution del usuario"; exit 1; }
find "$RESET_TEST/bkp" -type f -name '*.bak.*' | head -1 | grep -q . || echo "[info] reset genero backups esperados (o carpeta limpia)"
echo "[Ok] reset preserva la constitution del usuario (root) y limpia docs generados."

# --- Binario Rust: build real durante el setup (sin binario sembrado) -------
RUST_TEST="$TMP_ROOT/rust-binary"
copy_fixture "$RUST_TEST"
rm -f "$RUST_TEST/harness"   # fuerza la rama de compilacion, no la preexistente
mkdir -p "$RUST_TEST/rust"
cp "$REPO_ROOT/rust/Cargo.toml" "$REPO_ROOT/rust/Cargo.lock" "$RUST_TEST/rust/"
cp -R "$REPO_ROOT/rust/src" "$RUST_TEST/rust/src"
# El HOME falso del sandbox dejaria a rustup/cargo sin toolchain ni
# cache: capturamos los reales ANTES de pisar HOME.
REAL_RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
REAL_CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
(
    cd "$RUST_TEST"
    HOME="$TMP_ROOT/home" \
    RUSTUP_HOME="$REAL_RUSTUP_HOME" \
    CARGO_HOME="$REAL_CARGO_HOME" \
    HARNESS_HUB="$TMP_ROOT/rust-hub" \
    CARGO_TARGET_DIR="$REPO_ROOT/rust/target" \
    DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity >/dev/null 2>&1
)
test -x "$RUST_TEST/harness"
# grep SIN -q: consume todo el stdout y evita SIGPIPE temprano.
sh "$RUST_TEST/harness_cli" status | grep '^Backlog:' >/dev/null
echo "[Ok] binario Rust compilado por el setup e integrado via harness_cli."

# --- Feature #7: harness_check robusto (gate de espejo + checkout fuente) ---
# (a) AC-12b: harness_check.sh corre y pasa LIMPIO en una fixture recien
# instalada (espejos generados por el instalador vigente = cero falsos
# positivos del gate de espejo, AC-2).
CHECK_ROBUST="$TMP_ROOT/check-robust"
copy_fixture "$CHECK_ROBUST"
run_setup "$CHECK_ROBUST" --root

run_check() {
    (
        cd "$CHECK_ROBUST"
        env -u HARNESS_REPO_ROOT -u CLAUDE_PROJECT_DIR -u CODEX_PROJECT_DIR \
            -u GEMINI_PROJECT_DIR -u GROK_PROJECT_DIR -u ANTIGRAVITY_PROJECT_DIR \
            HOME="$TMP_ROOT/home" \
            HARNESS_HUB="$CHECK_ROBUST/.test-hub" \
            "$@" bash harness_check.sh < /dev/null
    )
}

rc=0; run_check > "$TMP_ROOT/check-clean.log" 2>&1 || rc=$?
if [ "$rc" -ne 0 ]; then
    echo "[!] harness_check.sh debio pasar limpio en la fixture recien instalada (rc=$rc):" >&2
    cat "$TMP_ROOT/check-clean.log" >&2
    exit 1
fi
grep -q 'Harness Check limpio' "$TMP_ROOT/check-clean.log"
if grep -q 'Espejo desincronizado' "$TMP_ROOT/check-clean.log"; then
    echo "[!] falso positivo del gate de espejo sobre espejos recien generados (AC-2)." >&2
    exit 1
fi

# (b) AC-12c: espejos stale INYECTADOS en los tres formatos -> el check los
# reporta nombrando cada archivo y falla (rc=2) en modo block (AC-1, AC-3).
cp "$CHECK_ROBUST/.claude/agents/implementer.md" "$TMP_ROOT/bak-claude-implementer.md"
cp "$CHECK_ROBUST/.gemini/agents/leader.md" "$TMP_ROOT/bak-gemini-leader.md"
cp "$CHECK_ROBUST/.codex/agents/reviewer.toml" "$TMP_ROOT/bak-codex-reviewer.toml"
printf '\nPROTOCOLO VIEJO INYECTADO\n' >> "$CHECK_ROBUST/.claude/agents/implementer.md"
printf '\nPROTOCOLO VIEJO INYECTADO\n' >> "$CHECK_ROBUST/.gemini/agents/leader.md"
# Codex: el drift va DENTRO del bloque developer_instructions (antes del cierre ''').
awk -v q="'''" '{ if ($0 == q && !done) { print "PROTOCOLO VIEJO INYECTADO"; done=1 } print }' \
    "$CHECK_ROBUST/.codex/agents/reviewer.toml" > "$TMP_ROOT/reviewer.toml.tmp" \
    && mv "$TMP_ROOT/reviewer.toml.tmp" "$CHECK_ROBUST/.codex/agents/reviewer.toml"

rc=0; run_check > "$TMP_ROOT/check-stale.log" 2>&1 || rc=$?
test "$rc" -eq 2 || { echo "[!] el check debio fallar (rc=2) con espejos stale (rc=$rc)." >&2; exit 1; }
grep -q 'Espejo desincronizado: .claude/agents/implementer.md' "$TMP_ROOT/check-stale.log"
grep -q 'Espejo desincronizado: .gemini/agents/leader.md' "$TMP_ROOT/check-stale.log"
grep -q 'Espejo desincronizado: .codex/agents/reviewer.toml' "$TMP_ROOT/check-stale.log"
grep -q 'Re-corre el instalador' "$TMP_ROOT/check-stale.log"
# AC-5: HARNESS_CHECK_MODE degrada igual que el resto de los checks.
rc=0; run_check HARNESS_CHECK_MODE=warn > "$TMP_ROOT/check-warn.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] modo warn debio salir 0 (rc=$rc)." >&2; exit 1; }
grep -q 'Espejo desincronizado' "$TMP_ROOT/check-warn.log"
rc=0; run_check HARNESS_CHECK_MODE=off > "$TMP_ROOT/check-off.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] modo off debio salir 0 (rc=$rc)." >&2; exit 1; }
if grep -q 'Espejo desincronizado' "$TMP_ROOT/check-off.log"; then
    echo "[!] modo off no debe evaluar el gate de espejo." >&2
    exit 1
fi
cp "$TMP_ROOT/bak-claude-implementer.md" "$CHECK_ROBUST/.claude/agents/implementer.md"
cp "$TMP_ROOT/bak-gemini-leader.md" "$CHECK_ROBUST/.gemini/agents/leader.md"
cp "$TMP_ROOT/bak-codex-reviewer.toml" "$CHECK_ROBUST/.codex/agents/reviewer.toml"

# AC-4: divergencia roles/ vs templates/roles/ (modulo __HREL__) tambien falla.
cp "$CHECK_ROBUST/templates/roles/leader.md" "$TMP_ROOT/bak-tpl-leader.md"
printf '\nDIVERGENCIA INYECTADA\n' >> "$CHECK_ROBUST/templates/roles/leader.md"
rc=0; run_check > "$TMP_ROOT/check-tpl.log" 2>&1 || rc=$?
test "$rc" -eq 2 || { echo "[!] el check debio fallar (rc=2) con templates/roles divergente (rc=$rc)." >&2; exit 1; }
grep -q 'Divergencia roles/leader.md vs templates/roles/leader.md' "$TMP_ROOT/check-tpl.log"
cp "$TMP_ROOT/bak-tpl-leader.md" "$CHECK_ROBUST/templates/roles/leader.md"

# Sanity: restaurados los espejos, el check vuelve a quedar limpio.
rc=0; run_check > "$TMP_ROOT/check-restored.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] el check debio volver a pasar tras restaurar los espejos (rc=$rc)." >&2; exit 1; }
echo "[Ok] gate de espejo: limpio post-install; stale en Claude/Gemini/Codex y drift de templates detectados; warn/off degradan."

# (c) AC-12d: checkout FUENTE simulado (marker subdir versionado historico +
# senales de fuente, padre sin huella): resolucion LOCAL, sin el falso "Falta
# docs/constitution.md", y CERO escrituras fuera del clon (AC-6, AC-7, AC-8).
SOURCE_PARENT="$TMP_ROOT/source-sim"
SOURCE_CLONE="$SOURCE_PARENT/harness_process"
SOURCE_HOME="$TMP_ROOT/source-home"
mkdir -p "$SOURCE_CLONE" "$SOURCE_HOME"
for f in harness_check.sh harness_status.sh init.sh commit_guard.sh harness_cli setup_harness.sh CHECKPOINTS.md; do
    cp "$REPO_ROOT/$f" "$SOURCE_CLONE/$f"
done
cp -R "$REPO_ROOT/templates" "$SOURCE_CLONE/templates"
cp -R "$REPO_ROOT/roles" "$SOURCE_CLONE/roles"
mkdir -p "$SOURCE_CLONE/rust" "$SOURCE_CLONE/docs" "$SOURCE_CLONE/progress" "$SOURCE_CLONE/.claude/agents"
cp "$REPO_ROOT/rust/Cargo.toml" "$SOURCE_CLONE/rust/Cargo.toml"
cp "$REPO_ROOT/docs/constitution.md" "$SOURCE_CLONE/docs/constitution.md"
cp "$REPO_ROOT"/.claude/agents/*.md "$SOURCE_CLONE/.claude/agents/"
cp "$REPO_ROOT/templates/feature_list.json" "$SOURCE_CLONE/feature_list.json"
cp "$REPO_ROOT/templates/progress/current.md" "$SOURCE_CLONE/progress/current.md"
cp "$REPO_ROOT/templates/progress/history.md" "$SOURCE_CLONE/progress/history.md"
cp "$PREBUILT_BIN" "$SOURCE_CLONE/harness"
chmod +x "$SOURCE_CLONE/harness"
printf 'subdir\n' > "$SOURCE_CLONE/.harness_layout" # estado historico del footgun

src_env() {
    env -u HARNESS_REPO_ROOT -u CLAUDE_PROJECT_DIR -u CODEX_PROJECT_DIR \
        -u GEMINI_PROJECT_DIR -u GROK_PROJECT_DIR -u ANTIGRAVITY_PROJECT_DIR \
        HOME="$SOURCE_HOME" \
        HARNESS_HUB="$SOURCE_CLONE/.test-hub" \
        DB_HOST=127.0.0.1 DB_PORT=9 DB_USER=harness DB_PASSWORD=secret \
        DB_NAME=harness DB_SSL_MODE=disable \
        "$@"
}

# AC-6: harness_check.sh resuelve local, avisa con [i] y NO inventa fallos.
rc=0; (cd "$SOURCE_CLONE" && src_env bash harness_check.sh < /dev/null) > "$TMP_ROOT/source-check.log" 2>&1 || rc=$?
if [ "$rc" -ne 0 ]; then
    echo "[!] harness_check.sh debio pasar en el checkout fuente simulado (rc=$rc):" >&2
    cat "$TMP_ROOT/source-check.log" >&2
    exit 1
fi
grep -q 'Checkout fuente del arnes detectado' "$TMP_ROOT/source-check.log"
if grep -q 'Falta docs/constitution.md' "$TMP_ROOT/source-check.log"; then
    echo "[!] falso fallo de constitution en el checkout fuente (resolvio al padre)." >&2
    exit 1
fi
# AC-7: harness_status.sh y commit_guard.sh aplican la misma resolucion.
rc=0; (cd "$SOURCE_CLONE" && src_env bash harness_status.sh --brief) > "$TMP_ROOT/source-status.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] harness_status.sh --brief fallo en el checkout fuente (rc=$rc)." >&2; exit 1; }
grep -q 'Checkout fuente del arnes detectado' "$TMP_ROOT/source-status.log"
rc=0; (cd "$SOURCE_CLONE" && src_env sh commit_guard.sh </dev/null) > "$TMP_ROOT/source-guard.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] commit_guard.sh fallo en el checkout fuente (rc=$rc)." >&2; exit 1; }
grep -q 'Checkout fuente del arnes detectado' "$TMP_ROOT/source-guard.log"
# init.sh: la resolucion es la misma; la conexion al hub muere rapido (DB
# inalcanzable) y NADA debe quedar fuera del clon.
(cd "$SOURCE_CLONE" && src_env bash init.sh) > "$TMP_ROOT/source-init.log" 2>&1 || true
grep -q 'Checkout fuente del arnes detectado' "$TMP_ROOT/source-init.log"
grep -qF "raiz=$SOURCE_CLONE" "$TMP_ROOT/source-init.log"
# AC-8: start (binario Rust) crea los artefactos DENTRO del clon.
src_env sh "$SOURCE_CLONE/harness_cli" add --name demo > /dev/null 2>&1
src_env sh "$SOURCE_CLONE/harness_cli" start --feature 1 > "$TMP_ROOT/source-start.log" 2>&1
test -f "$SOURCE_CLONE/docs/plan-feature-1-demo.md" \
    || { echo "[!] start debio crear el plan dentro del clon." >&2; exit 1; }
test -f "$SOURCE_CLONE/docs/spec-feature-1-demo.md"
# Cero basura fuera del clon: el padre solo contiene el clon y el HOME de la
# fixture queda intacto (el incidente real creaba $HOME/docs).
test "$(ls -A "$SOURCE_PARENT")" = "harness_process" \
    || { echo "[!] aparecieron rutas fuera del clon: $(ls -A "$SOURCE_PARENT")" >&2; exit 1; }
test -z "$(ls -A "$SOURCE_HOME")" \
    || { echo "[!] el \$HOME de la fixture fue modificado: $(ls -A "$SOURCE_HOME")" >&2; exit 1; }
echo "[Ok] checkout fuente simulado: resolucion local con aviso [i] en check/status/guard/init, start sin escrituras fuera del clon."

echo "[Ok] docs del arnes en el docs/ de la RAIZ: destino, migracion, no-pisa y reset."
echo "[Ok] planillas maestras docs/prd/ (PRD + SDD): siembra, no-pisa y supervivencia al reset."
echo "[Ok] setup smoke: Rust-only, gate de credenciales, layouts, reinstall, dry-run, version, reset."
