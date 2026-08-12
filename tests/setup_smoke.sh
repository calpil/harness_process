#!/bin/bash
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/harness-setup-smoke.XXXXXX")"
TMP_ROOT="$(cd "$TMP_ROOT" && pwd -P)"
trap 'rm -rf "$TMP_ROOT"' EXIT

# Aislamiento Kimi (feature #8): NINGUNA corrida del instalador dentro del
# smoke puede tocar el home real de Kimi. El default apunta a una fixture; los
# bloques que lo necesitan lo overridean con su propio KIMI_CODE_HOME.
export KIMI_CODE_HOME="$TMP_ROOT/kimi-home-default"

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
# Feature #11 / AC-2: la guia de uso eficiente se siembra en docs/ (layout root).
test -f "$ROOT_LAYOUT/docs/kimi-cli-uso-eficiente.md"
# Feature #12 / AC-5: la guia del metodo PRD se siembra en docs/prd/ (layout root).
test -f "$ROOT_LAYOUT/docs/prd/COMO-ESCRIBIR-UN-PRD.md" \
    || { echo "[FALLO] falta docs/prd/COMO-ESCRIBIR-UN-PRD.md en layout root"; exit 1; }
# Feature #11 (companion KIMI_DOTFILES): .kimiignore/.kimirules se siembran en la RAIZ.
test -f "$ROOT_LAYOUT/.kimiignore"
test -f "$ROOT_LAYOUT/.kimirules"
grep -qx 'postgres' "$ROOT_LAYOUT/.harness_backend"
# Hooks y superficies deben invocar el shim, no python3 directo.
grep -Fq 'harness_cli\" nudge' "$ROOT_LAYOUT/.claude/settings.json"
grep -Fq 'harness_cli" graph mapa' "$ROOT_LAYOUT/AGENTS.md"
python3 -m json.tool "$ROOT_LAYOUT/.codex/hooks.json" >/dev/null
python3 -m json.tool "$ROOT_LAYOUT/.gemini/settings.json" >/dev/null
python3 -c 'import pathlib, tomllib; [tomllib.loads(p.read_text()) for p in pathlib.Path("'"$ROOT_LAYOUT"'/.codex/agents").glob("*.toml")]'
# Feature #9 / AC-1: los TRES roles de Codex son workspace-write. Codex no
# ofrece allowlist de tools, asi que read-only en leader/reviewer les impedia
# escribir sus entregables en docs/ (spec, plan, veredicto). Se verifica sobre
# el TOML parseado, no por grep, para que un cambio de formato no lo falsee.
python3 - "$ROOT_LAYOUT" <<'CODEX_SANDBOX_EOF'
import pathlib, sys, tomllib
root = pathlib.Path(sys.argv[1]) / ".codex" / "agents"
roles = {}
for p in sorted(root.glob("*.toml")):
    roles[p.stem] = tomllib.loads(p.read_text()).get("sandbox_mode")
faltan = {r for r in ("leader", "implementer", "reviewer")} - set(roles)
if faltan:
    sys.exit(f"[FALLO] faltan agentes Codex: {sorted(faltan)}")
malos = {r: m for r, m in roles.items() if m != "workspace-write"}
if malos:
    sys.exit(f"[FALLO] sandbox_mode != workspace-write en Codex: {malos}")
CODEX_SANDBOX_EOF
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
# Feature #11 / AC-2: la guia de uso eficiente se siembra en docs/ (layout subdir).
test -f "$SUBDIR_ROOT/docs/kimi-cli-uso-eficiente.md"
# Feature #11 (companion KIMI_DOTFILES): .kimiignore/.kimirules en la RAIZ multi-repo.
test -f "$SUBDIR_ROOT/.kimiignore"
test -f "$SUBDIR_ROOT/.kimirules"
test ! -e "$SUBDIR_HARNESS/docs/architecture.md"
test ! -e "$SUBDIR_HARNESS/docs/conventions.md"
test ! -e "$SUBDIR_HARNESS/docs/verification.md"
test ! -d "$SUBDIR_HARNESS/docs"
# Feature #5 / AC-1: planillas maestras PRD y SDD en docs/prd/ de la RAIZ.
test -f "$SUBDIR_ROOT/docs/prd/PRD-master.md"
test -f "$SUBDIR_ROOT/docs/prd/SDD-master.md"
test ! -d "$SUBDIR_HARNESS/docs/prd"
# Feature #12 / AC-5: la guia del metodo PRD acompana a las planillas en la RAIZ.
test -f "$SUBDIR_ROOT/docs/prd/COMO-ESCRIBIR-UN-PRD.md" \
    || { echo "[FALLO] falta docs/prd/COMO-ESCRIBIR-UN-PRD.md en la raiz multi-repo"; exit 1; }
# Feature #12 / AC-4: la guia trae el metodo completo (historia, tamano, sin codigo).
grep -q '^## 2. Todo empieza con una historia' "$SUBDIR_ROOT/docs/prd/COMO-ESCRIBIR-UN-PRD.md"
grep -q '^## 3. El tamano lo decide el cambio' "$SUBDIR_ROOT/docs/prd/COMO-ESCRIBIR-UN-PRD.md"
grep -q 'NUNCA CONTIENE' "$SUBDIR_ROOT/docs/prd/COMO-ESCRIBIR-UN-PRD.md"
# AC-7 / AC-8 (feature #5) + Feature #12 / AC-1..AC-3: las planillas traen las
# secciones que las hacen utiles, ya con la anatomia del metodo.
grep -q '^## 2. La historia' "$SUBDIR_ROOT/docs/prd/PRD-master.md"
grep -q '^## 8. Pseudo-codigo (el acuerdo)' "$SUBDIR_ROOT/docs/prd/PRD-master.md"
grep -q '^## 10. Hitos -> features' "$SUBDIR_ROOT/docs/prd/PRD-master.md"
grep -q 'harness_cli add' "$SUBDIR_ROOT/docs/prd/PRD-master.md"
grep -q '^## 4. Decisiones tecnicas' "$SUBDIR_ROOT/docs/prd/SDD-master.md"
grep -q 'docs/architecture.md' "$SUBDIR_ROOT/docs/prd/SDD-master.md"
grep -q 'harness_process/init.sh' "$SUBDIR_ROOT/AGENTS.md"
grep -Fq 'harness_process/harness_cli" graph mapa' "$SUBDIR_ROOT/AGENTS.md"
# Feature #11 / AC-3: la superficie instalada enlaza la guia de uso eficiente.
grep -q 'kimi-cli-uso-eficiente' "$SUBDIR_ROOT/AGENTS.md" \
    || { echo "[FALLO] AGENTS.md instalado no enlaza docs/kimi-cli-uso-eficiente.md"; exit 1; }
# Feature #12 / AC-8: la superficie instalada enlaza el metodo para escribir PRDs.
grep -q 'COMO-ESCRIBIR-UN-PRD' "$SUBDIR_ROOT/AGENTS.md" \
    || { echo "[FALLO] AGENTS.md instalado no enlaza docs/prd/COMO-ESCRIBIR-UN-PRD.md"; exit 1; }
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
# Feature #8 / AC-2: los espejos Kimi tambien se respaldan en el reinstall.
find "$CUSTOM_BKP" -type f -path '*.kimi-code/agents/leader.md.bak.*' -print -quit | grep -q . \
    || { echo "[!] falta el backup del espejo Kimi en el reinstall." >&2; exit 1; }
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
# Feature #11 / AC-2: la guia es HARNESS_DOCS, asi que el reset tambien la limpia.
test ! -f "$RESET_TEST/docs/kimi-cli-uso-eficiente.md" \
    || { echo "[FALLO] reset no limpio el doc generado kimi-cli-uso-eficiente.md"; exit 1; }
# Feature #12 / AC-5: la guia del metodo es plantilla del arnes, asi que el reset
# tambien la limpia (el PRD/SDD del usuario, en la misma carpeta, sobreviven).
test ! -f "$RESET_TEST/docs/prd/COMO-ESCRIBIR-UN-PRD.md" \
    || { echo "[FALLO] reset no limpio la guia generada COMO-ESCRIBIR-UN-PRD.md"; exit 1; }
# Feature #11 (companion KIMI_DOTFILES): los dotfiles son documentos del USUARIO
# y sobreviven al reset (mismo criterio que PRD/SDD).
test -f "$RESET_TEST/.kimiignore" \
    || { echo "[FALLO] reset borro el .kimiignore del usuario"; exit 1; }
test -f "$RESET_TEST/.kimirules" \
    || { echo "[FALLO] reset borro el .kimirules del usuario"; exit 1; }
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
# Feature #12 / AC-5: y la guia del metodo se vuelve a sembrar en el reinstall.
test -f "$RESET_TEST/docs/prd/COMO-ESCRIBIR-UN-PRD.md" \
    || { echo "[FALLO] reinstall tras reset no resembro COMO-ESCRIBIR-UN-PRD.md"; exit 1; }
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

# --- Feature #10: layout inferido por huella cuando falta el marker ---------
# La feature #7 des-versiono .harness_layout (commit c8392f5 graba
# "D .harness_layout"), asi que toda instalacion subdir que hace 'git pull' se
# queda SIN marker y pasaba a tratar harness_process/ como raiz en silencio.
# Fixtures propias para los escenarios del AC-10 (mas la guarda de $HOME).

LOST_ROOT="$TMP_ROOT/lost-marker"
mkdir -p "$LOST_ROOT/home"

# <caso>/proyecto/harness_process con los scripts y el binario REALES. El
# proyecto lleva huella de instalacion (docs/constitution.md + CLAUDE.md) solo
# si se pide, y el marker se escribe solo si se pasa un valor.
make_lost_case() { # $1=caso  $2=huella(1|0)  $3=marker ("" = ausente)
    lost_proj="$LOST_ROOT/$1/proyecto"
    lost_h="$lost_proj/harness_process"
    mkdir -p "$lost_h/progress" "$lost_h/rust"
    for lost_f in harness_check.sh harness_status.sh init.sh commit_guard.sh harness_cli CHECKPOINTS.md; do
        cp "$REPO_ROOT/$lost_f" "$lost_h/$lost_f"
    done
    cp -R "$REPO_ROOT/templates" "$lost_h/templates"
    cp -R "$REPO_ROOT/roles" "$lost_h/roles"
    cp "$REPO_ROOT/rust/Cargo.toml" "$lost_h/rust/Cargo.toml"  # senal de fuente
    cp "$REPO_ROOT/templates/feature_list.json" "$lost_h/feature_list.json"
    cp "$REPO_ROOT/templates/progress/current.md" "$lost_h/progress/current.md"
    cp "$REPO_ROOT/templates/progress/history.md" "$lost_h/progress/history.md"
    cp "$PREBUILT_BIN" "$lost_h/harness"
    chmod +x "$lost_h/harness"
    if [ "$2" = "1" ]; then
        mkdir -p "$lost_proj/docs"
        cp "$REPO_ROOT/docs/constitution.md" "$lost_proj/docs/constitution.md"
        printf '# proyecto\n' > "$lost_proj/CLAUDE.md"
    fi
    if [ -n "$3" ]; then
        printf '%s\n' "$3" > "$lost_h/.harness_layout"
    fi
    return 0
}

lost_env() {
    env -u HARNESS_REPO_ROOT -u CLAUDE_PROJECT_DIR -u CODEX_PROJECT_DIR \
        -u GEMINI_PROJECT_DIR -u GROK_PROJECT_DIR -u ANTIGRAVITY_PROJECT_DIR \
        HOME="$LOST_ROOT/home" \
        HARNESS_HUB="$LOST_ROOT/hub" \
        DB_HOST=127.0.0.1 DB_PORT=9 DB_USER=harness DB_PASSWORD=secret \
        DB_NAME=harness DB_SSL_MODE=disable \
        "$@"
}

# init.sh imprime la raiz resuelta ("raiz=..."); la conexion al hub muere
# despues (DB inalcanzable), como en el bloque del checkout fuente.
lost_resolved_root() { # $1=harness dir  $2=log  [env extra...]
    lost_dir="$1"; lost_log="$2"; shift 2
    (cd "$lost_dir" && lost_env "$@" bash init.sh) > "$lost_log" 2>&1 || true
}

lost_assert_absent() { # $1=log  $2=patron  $3=explicacion
    if grep -q "$2" "$1"; then
        echo "[!] $3 (log: $1)" >&2
        exit 1
    fi
}

# (a) AC-1 + AC-2: sin marker y con huella en el padre -> raiz al PROYECTO, con
# aviso [i] que nombra el remedio. Es el footgun real de las 15 instalaciones.
make_lost_case a 1 ""
LOST_A_PROJ="$LOST_ROOT/a/proyecto"
LOST_A_H="$LOST_A_PROJ/harness_process"
rc=0; (cd "$LOST_A_H" && lost_env bash harness_check.sh < /dev/null) > "$TMP_ROOT/lost-a-check.log" 2>&1 || rc=$?
if [ "$rc" -ne 0 ]; then
    echo "[!] harness_check.sh debio pasar en la instalacion sin marker (rc=$rc):" >&2
    cat "$TMP_ROOT/lost-a-check.log" >&2
    exit 1
fi
grep -q '\.harness_layout ausente: layout subdir inferido por la huella de instalacion del padre' "$TMP_ROOT/lost-a-check.log"
grep -q 'para regenerar el marker' "$TMP_ROOT/lost-a-check.log"
# Sin la inferencia, REPO_ROOT seria el arnes y el check inventaria este fallo:
lost_assert_absent "$TMP_ROOT/lost-a-check.log" 'Falta docs/constitution.md' \
    "la resolucion cayo en el arnes en vez del proyecto"
rc=0; (cd "$LOST_A_H" && lost_env bash harness_status.sh --brief) > "$TMP_ROOT/lost-a-status.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] harness_status.sh --brief fallo sin marker (rc=$rc)." >&2; exit 1; }
grep -q '\.harness_layout ausente' "$TMP_ROOT/lost-a-status.log"
rc=0; (cd "$LOST_A_H" && lost_env sh commit_guard.sh </dev/null) > "$TMP_ROOT/lost-a-guard.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] commit_guard.sh fallo sin marker (rc=$rc)." >&2; exit 1; }
grep -q '\.harness_layout ausente' "$TMP_ROOT/lost-a-guard.log"
lost_resolved_root "$LOST_A_H" "$TMP_ROOT/lost-a-init.log"
grep -qF "raiz=$LOST_A_PROJ" "$TMP_ROOT/lost-a-init.log"
# AC-8: el binario aplica la MISMA regla; los artefactos van al docs/ del proyecto.
lost_env sh "$LOST_A_H/harness_cli" add --name demo > /dev/null 2>&1
lost_env sh "$LOST_A_H/harness_cli" start --feature 1 > "$TMP_ROOT/lost-a-start.log" 2>&1
test -f "$LOST_A_PROJ/docs/plan-feature-1-demo.md" \
    || { echo "[!] start debio crear el plan en el docs/ del proyecto." >&2; exit 1; }
test -f "$LOST_A_PROJ/docs/spec-feature-1-demo.md"
test ! -d "$LOST_A_H/docs" \
    || { echo "[!] el binario escribio dentro del arnes: $(ls -A "$LOST_A_H/docs")" >&2; exit 1; }

# (b) AC-4: sin marker y SIN huella en el padre -> raiz al propio arnes, sin aviso.
make_lost_case b 0 ""
LOST_B_H="$LOST_ROOT/b/proyecto/harness_process"
lost_resolved_root "$LOST_B_H" "$TMP_ROOT/lost-b-init.log"
grep -qF "raiz=$LOST_B_H" "$TMP_ROOT/lost-b-init.log"
lost_assert_absent "$TMP_ROOT/lost-b-init.log" '\.harness_layout ausente' \
    "se infirio subdir sin huella en el padre"

# (c) AC-3: marker EXPLICITO 'root' con huella en el padre -> sin inferencia y
# sin aviso (la inferencia solo aplica cuando el archivo NO existe).
make_lost_case c 1 root
LOST_C_H="$LOST_ROOT/c/proyecto/harness_process"
lost_resolved_root "$LOST_C_H" "$TMP_ROOT/lost-c-init.log"
grep -qF "raiz=$LOST_C_H" "$TMP_ROOT/lost-c-init.log"
lost_assert_absent "$TMP_ROOT/lost-c-init.log" '\.harness_layout ausente' \
    "se infirio layout sobre un marker explicito 'root'"

# (d) AC-7: el guardrail de checkout fuente de la feature #7 sigue verde
# (marker 'subdir' + senales de fuente + padre sin huella -> propio dir).
make_lost_case d 0 subdir
LOST_D_H="$LOST_ROOT/d/proyecto/harness_process"
lost_resolved_root "$LOST_D_H" "$TMP_ROOT/lost-d-init.log"
grep -qF "raiz=$LOST_D_H" "$TMP_ROOT/lost-d-init.log"
grep -q 'Checkout fuente del arnes detectado' "$TMP_ROOT/lost-d-init.log"
lost_assert_absent "$TMP_ROOT/lost-d-init.log" '\.harness_layout ausente' \
    "el marker 'subdir' no debe pasar por la inferencia"

# (e) cero regresion feature #7: marker 'subdir' + padre CON huella sigue
# resolviendo al padre, sin ningun aviso.
make_lost_case e 1 subdir
LOST_E_PROJ="$LOST_ROOT/e/proyecto"
lost_resolved_root "$LOST_E_PROJ/harness_process" "$TMP_ROOT/lost-e-init.log"
grep -qF "raiz=$LOST_E_PROJ" "$TMP_ROOT/lost-e-init.log"
lost_assert_absent "$TMP_ROOT/lost-e-init.log" '\[i\]' \
    "una instalacion subdir legitima no debe emitir avisos"

# (f) AC-5: la guarda de $HOME aplica tambien a la inferencia; con el escape
# explicito HARNESS_ALLOW_HOME_SURFACE=1 la huella vuelve a mandar.
make_lost_case f 1 ""
LOST_F_PROJ="$LOST_ROOT/f/proyecto"
LOST_F_H="$LOST_F_PROJ/harness_process"
lost_resolved_root "$LOST_F_H" "$TMP_ROOT/lost-f-init.log" env HOME="$LOST_F_PROJ"
grep -qF "raiz=$LOST_F_H" "$TMP_ROOT/lost-f-init.log"
lost_assert_absent "$TMP_ROOT/lost-f-init.log" '\.harness_layout ausente' \
    "se infirio subdir con el padre == \$HOME"
lost_resolved_root "$LOST_F_H" "$TMP_ROOT/lost-f-home-ok.log" \
    env HOME="$LOST_F_PROJ" HARNESS_ALLOW_HOME_SURFACE=1
grep -qF "raiz=$LOST_F_PROJ" "$TMP_ROOT/lost-f-home-ok.log"
grep -q '\.harness_layout ausente' "$TMP_ROOT/lost-f-home-ok.log"

# (g) AC-6: los overrides mandan sobre la inferencia, sin aviso. HARNESS_REPO_ROOT
# lo honran script y binario; las variables de agente (CLAUDE_PROJECT_DIR, ...)
# solo los scripts -el binario nunca las leyo, comportamiento previo a esta
# feature-, asi que ahi el "sin aviso" se comprueba con commit_guard.sh, que es
# shell puro y no invoca al binario.
LOST_G_TARGET="$LOST_ROOT/override-target"
mkdir -p "$LOST_G_TARGET"
(cd "$LOST_A_H" && lost_env env HARNESS_REPO_ROOT="$LOST_G_TARGET" bash init.sh) \
    > "$TMP_ROOT/lost-g-init.log" 2>&1 || true
grep -qF "raiz=$LOST_G_TARGET" "$TMP_ROOT/lost-g-init.log"
lost_assert_absent "$TMP_ROOT/lost-g-init.log" '\.harness_layout ausente' \
    "el override no debe pasar por la inferencia"
(cd "$LOST_A_H" && lost_env env CLAUDE_PROJECT_DIR="$LOST_G_TARGET" bash init.sh) \
    > "$TMP_ROOT/lost-g2-init.log" 2>&1 || true
grep -qF "raiz=$LOST_G_TARGET" "$TMP_ROOT/lost-g2-init.log"
rc=0; (cd "$LOST_A_H" && lost_env env CLAUDE_PROJECT_DIR="$LOST_G_TARGET" sh commit_guard.sh </dev/null) \
    > "$TMP_ROOT/lost-g2-guard.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] commit_guard.sh fallo con variable de agente (rc=$rc)." >&2; exit 1; }
lost_assert_absent "$TMP_ROOT/lost-g2-guard.log" '\[i\]' \
    "la variable de agente no debe pasar por la inferencia en los scripts"

echo "[Ok] marker ausente: layout subdir inferido por huella del padre (scripts + binario) con aviso [i]; sin huella, marker 'root', \$HOME y overrides no infieren; guardrail #7 intacto."

# --- Feature #8: Kimi Code CLI como backend de primera clase ----------------
# Todo lo global de Kimi usa KIMI_CODE_HOME de fixture (NUNCA el home real);
# el export del inicio del smoke ya aisla cualquier corrida del instalador.

# (a) AC-9a: espejos Kimi generados en layout root y subdir, con frontmatter
# valido (allowlist de tools por rol, decision usuario 2026-07-28) y cuerpo
# identico a roles/<rol>.md (misma extraccion que el gate del check).
kimi_agent_body() {
    awk 'started { print; next }
         inbody { if ($0 ~ /^[[:space:]]*$/) next; started=1; print; next }
         /^---[[:space:]]*$/ { fm++; if (fm == 2) inbody=1; next }' "$1"
}
for kimi_role in leader implementer reviewer; do
    test -f "$ROOT_LAYOUT/.kimi-code/agents/$kimi_role.md"
    test "$(head -n1 "$ROOT_LAYOUT/.kimi-code/agents/$kimi_role.md")" = "---"
    grep -q "^name: $kimi_role\$" "$ROOT_LAYOUT/.kimi-code/agents/$kimi_role.md"
    grep -q '^description: ' "$ROOT_LAYOUT/.kimi-code/agents/$kimi_role.md"
    test "$(kimi_agent_body "$ROOT_LAYOUT/.kimi-code/agents/$kimi_role.md")" = "$(cat "$ROOT_LAYOUT/roles/$kimi_role.md")" \
        || { echo "[!] espejo Kimi $kimi_role.md (root) no coincide con roles/$kimi_role.md." >&2; exit 1; }
    test -f "$SUBDIR_ROOT/.kimi-code/agents/$kimi_role.md"
    test "$(kimi_agent_body "$SUBDIR_ROOT/.kimi-code/agents/$kimi_role.md")" = "$(cat "$SUBDIR_HARNESS/roles/$kimi_role.md")" \
        || { echo "[!] espejo Kimi $kimi_role.md (subdir) no coincide con roles/$kimi_role.md." >&2; exit 1; }
done
grep -q '^tools: Read, Grep, Glob, Bash$' "$ROOT_LAYOUT/.kimi-code/agents/leader.md"
grep -q '^tools: Read, Grep, Glob, Bash$' "$ROOT_LAYOUT/.kimi-code/agents/reviewer.md"
grep -q '^tools: Read, Edit, Write, Bash, Grep, Glob$' "$ROOT_LAYOUT/.kimi-code/agents/implementer.md"
# Launcher + superficie multi-backend mencionan a Kimi (AC-2/AC-7).
test -x "$ROOT_LAYOUT/bin/harness-kimi"
grep -q 'AGENT="kimi"' "$ROOT_LAYOUT/bin/harness-kimi"
grep -q 'bin/harness-kimi' "$ROOT_LAYOUT/AGENTS.md"
grep -q '.kimi-code/agents' "$ROOT_LAYOUT/AGENTS.md"
grep -q 'bin/harness-kimi' "$NO_SUBAGENTS/AGENTS.md"
# AC-1: --no-subagents NO genera espejos Kimi (misma condicionalidad que
# .claude/.codex/.gemini); el launcher se genera siempre.
test ! -d "$NO_SUBAGENTS/.kimi-code"
test -x "$NO_SUBAGENTS/bin/harness-kimi"

# (b) AC-9b: espejo Kimi stale inyectado -> harness_check.sh lo reporta
# nombrando el archivo y falla (rc=2) en block; warn reporta y sale 0; off no
# evalua. Reusa la fixture check-robust y run_check de la feature #7.
cp "$CHECK_ROBUST/.kimi-code/agents/leader.md" "$TMP_ROOT/bak-kimi-leader.md"
printf '\nPROTOCOLO VIEJO INYECTADO\n' >> "$CHECK_ROBUST/.kimi-code/agents/leader.md"
rc=0; run_check > "$TMP_ROOT/check-kimi-stale.log" 2>&1 || rc=$?
test "$rc" -eq 2 || { echo "[!] el check debio fallar (rc=2) con espejo Kimi stale (rc=$rc)." >&2; exit 1; }
grep -q 'Espejo desincronizado: .kimi-code/agents/leader.md' "$TMP_ROOT/check-kimi-stale.log"
grep -q 'Re-corre el instalador' "$TMP_ROOT/check-kimi-stale.log"
rc=0; run_check HARNESS_CHECK_MODE=warn > "$TMP_ROOT/check-kimi-warn.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] modo warn debio salir 0 con espejo Kimi stale (rc=$rc)." >&2; exit 1; }
grep -q 'Espejo desincronizado: .kimi-code/agents/leader.md' "$TMP_ROOT/check-kimi-warn.log"
rc=0; run_check HARNESS_CHECK_MODE=off > "$TMP_ROOT/check-kimi-off.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] modo off debio salir 0 (rc=$rc)." >&2; exit 1; }
if grep -q 'Espejo desincronizado' "$TMP_ROOT/check-kimi-off.log"; then
    echo "[!] modo off no debe evaluar el gate de espejo Kimi." >&2
    exit 1
fi
cp "$TMP_ROOT/bak-kimi-leader.md" "$CHECK_ROBUST/.kimi-code/agents/leader.md"
rc=0; run_check > "$TMP_ROOT/check-kimi-restored.log" 2>&1 || rc=$?
test "$rc" -eq 0 || { echo "[!] el check debio volver a pasar tras restaurar el espejo Kimi (rc=$rc)." >&2; exit 1; }

# (c)/(d)/(e): bloque GLOBAL de hooks con KIMI_CODE_HOME de fixture.
KIMI_E2E="$TMP_ROOT/kimi-e2e"
copy_fixture "$KIMI_E2E"
KIMI_BKP="$TMP_ROOT/kimi-bkp"

kimi_setup() {
    kimi_home_arg="$1"
    shift
    (
        cd "$KIMI_E2E"
        HOME="$TMP_ROOT/home" \
        KIMI_CODE_HOME="$kimi_home_arg" \
        HARNESS_HUB="$KIMI_E2E/.test-hub" \
        HARNESS_BKP_DIR="$KIMI_BKP" \
        DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
        bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity "$@"
    )
}

KIMI_HOOKS_B='# >>> harness-process hooks >>>'
KIMI_HOOKS_E='# <<< harness-process hooks <<<'

# Kimi falso detectable via ${KIMI_CODE_HOME}/bin/kimi: 'doctor' valida OK.
KIMI_HOME_ON="$TMP_ROOT/kimi-home-on"
mkdir -p "$KIMI_HOME_ON/bin"
printf '#!/bin/sh\nexit 0\n' > "$KIMI_HOME_ON/bin/kimi"
chmod +x "$KIMI_HOME_ON/bin/kimi"

# (c-1) AC-9c: config global INEXISTENTE -> se crea dir+archivo con UN solo
# bloque delimitado y exactamente los tres [[hooks]] del arnes (TOML valido).
kimi_setup "$KIMI_HOME_ON" > "$TMP_ROOT/kimi-install-on.log" 2>&1
test -f "$KIMI_HOME_ON/config.toml"
test "$(grep -cF "$KIMI_HOOKS_B" "$KIMI_HOME_ON/config.toml")" -eq 1
test "$(grep -cF "$KIMI_HOOKS_E" "$KIMI_HOME_ON/config.toml")" -eq 1
test "$(grep -c '^\[\[hooks\]\]$' "$KIMI_HOME_ON/config.toml")" -eq 3
grep -q '^event = "SessionStart"$' "$KIMI_HOME_ON/config.toml"
grep -q '^event = "PostToolUse"$' "$KIMI_HOME_ON/config.toml"
grep -q '^matcher = "Edit|Write"$' "$KIMI_HOME_ON/config.toml"
grep -q '^event = "Stop"$' "$KIMI_HOME_ON/config.toml"
grep -q 'plain session-start' "$KIMI_HOME_ON/config.toml"
grep -q 'plain post-tool' "$KIMI_HOME_ON/config.toml"
grep -q 'plain stop' "$KIMI_HOME_ON/config.toml"
grep -q 'HARNESS_REPO_ROOT' "$KIMI_HOME_ON/config.toml"
if grep -q 'SessionEnd\|UserPromptSubmit' "$KIMI_HOME_ON/config.toml"; then
    echo "[!] el bloque global solo debe registrar SessionStart/PostToolUse/Stop." >&2
    exit 1
fi
python3 -c 'import tomllib,sys; tomllib.load(open(sys.argv[1],"rb"))' "$KIMI_HOME_ON/config.toml"

# (c-2) AC-9c: config con hooks PROPIOS del usuario + sentinel -> sobreviven,
# el bloque no se duplica tras re-instalar, el resto queda byte a byte y hay
# backup en el bkp/ de fixture.
KIMI_HOME_USER="$TMP_ROOT/kimi-home-user"
mkdir -p "$KIMI_HOME_USER/bin"
cp "$KIMI_HOME_ON/bin/kimi" "$KIMI_HOME_USER/bin/kimi"
KIMI_SENTINEL="SENTINEL-KIMI-USER-CONFIG-$$"
cat > "$KIMI_HOME_USER/config.toml" <<KIMIUSEREOF
# $KIMI_SENTINEL

[[hooks]]
event = "UserPromptSubmit"
command = "echo hook-del-usuario"
KIMIUSEREOF
cp "$KIMI_HOME_USER/config.toml" "$TMP_ROOT/kimi-user-config.orig"
kimi_setup "$KIMI_HOME_USER" > "$TMP_ROOT/kimi-install-user.log" 2>&1
grep -q "$KIMI_SENTINEL" "$KIMI_HOME_USER/config.toml"
grep -q 'echo hook-del-usuario' "$KIMI_HOME_USER/config.toml"
test "$(grep -cF "$KIMI_HOOKS_B" "$KIMI_HOME_USER/config.toml")" -eq 1
test "$(grep -c '^\[\[hooks\]\]$' "$KIMI_HOME_USER/config.toml")" -eq 4
kimi_setup "$KIMI_HOME_USER" > "$TMP_ROOT/kimi-install-user2.log" 2>&1
test "$(grep -cF "$KIMI_HOOKS_B" "$KIMI_HOME_USER/config.toml")" -eq 1
test "$(grep -cF "$KIMI_HOOKS_E" "$KIMI_HOME_USER/config.toml")" -eq 1
test "$(grep -c '^\[\[hooks\]\]$' "$KIMI_HOME_USER/config.toml")" -eq 4
awk -v b="$KIMI_HOOKS_B" -v e="$KIMI_HOOKS_E" '
    $0 == b { inblk=1; next }
    inblk { if ($0 == e) inblk=0; next }
    { print }
' "$KIMI_HOME_USER/config.toml" > "$TMP_ROOT/kimi-user-config.stripped"
cmp -s "$TMP_ROOT/kimi-user-config.orig" "$TMP_ROOT/kimi-user-config.stripped" \
    || { echo "[!] el contenido del usuario fuera del bloque de Kimi cambio." >&2; exit 1; }
find "$KIMI_BKP" -type f -name 'config.toml.bak.*' -print -quit | grep -q . \
    || { echo "[!] falta el backup del config.toml global de Kimi." >&2; exit 1; }

# AC-5: doctor invalido -> rollback al estado previo (o retiro del archivo
# recien creado), aviso accionable y el setup conserva exit 0. Determinista
# solo si el 'kimi' real no esta en PATH (usaria el doctor real, que si
# validaria); en ese caso queda cubierto por --no-kimi y el doctor OK de (c).
if ! command -v kimi >/dev/null 2>&1; then
    KIMI_HOME_BAD="$TMP_ROOT/kimi-home-bad"
    mkdir -p "$KIMI_HOME_BAD/bin"
    printf '#!/bin/sh\nexit 1\n' > "$KIMI_HOME_BAD/bin/kimi"
    chmod +x "$KIMI_HOME_BAD/bin/kimi"
    printf '# config previa del usuario\n' > "$KIMI_HOME_BAD/config.toml"
    cp "$KIMI_HOME_BAD/config.toml" "$TMP_ROOT/kimi-bad-config.orig"
    rc=0; kimi_setup "$KIMI_HOME_BAD" > "$TMP_ROOT/kimi-install-bad.log" 2>&1 || rc=$?
    test "$rc" -eq 0 || { echo "[!] el bloque global es best-effort: el setup no debio fallar (rc=$rc)." >&2; exit 1; }
    cmp -s "$TMP_ROOT/kimi-bad-config.orig" "$KIMI_HOME_BAD/config.toml" \
        || { echo "[!] doctor invalido debio restaurar el config.toml previo." >&2; exit 1; }
    grep -q "doctor' reporto config invalido" "$TMP_ROOT/kimi-install-bad.log"
    rm -f "$KIMI_HOME_BAD/config.toml"
    rc=0; kimi_setup "$KIMI_HOME_BAD" > "$TMP_ROOT/kimi-install-bad2.log" 2>&1 || rc=$?
    test "$rc" -eq 0
    test ! -e "$KIMI_HOME_BAD/config.toml" \
        || { echo "[!] con doctor invalido el config recien creado debio retirarse." >&2; exit 1; }
else
    echo "[info] 'kimi' esta en PATH: se omite la rama de rollback con doctor falso."
fi

# (d) AC-9d: rama de NO instalacion del bloque global.
# (d-1) --no-kimi con Kimi detectable -> no escribe nada en el home global.
KIMI_HOME_FLAG="$TMP_ROOT/kimi-home-flag"
mkdir -p "$KIMI_HOME_FLAG/bin"
cp "$KIMI_HOME_ON/bin/kimi" "$KIMI_HOME_FLAG/bin/kimi"
kimi_setup "$KIMI_HOME_FLAG" --no-kimi > "$TMP_ROOT/kimi-install-flag.log" 2>&1
test ! -e "$KIMI_HOME_FLAG/config.toml" \
    || { echo "[!] --no-kimi no debe escribir el config.toml global." >&2; exit 1; }
grep -q 'omitido (--no-kimi)' "$TMP_ROOT/kimi-install-flag.log"
# (d-2) sin Kimi detectable -> KIMI_CODE_HOME de fixture queda intacto.
# Determinista solo si 'kimi' tampoco esta en el PATH del entorno del test.
if ! command -v kimi >/dev/null 2>&1; then
    KIMI_HOME_OFF="$TMP_ROOT/kimi-home-off"
    mkdir -p "$KIMI_HOME_OFF"
    kimi_setup "$KIMI_HOME_OFF" > "$TMP_ROOT/kimi-install-off.log" 2>&1
    test -z "$(ls -A "$KIMI_HOME_OFF")" \
        || { echo "[!] sin Kimi detectado no debio escribirse nada en KIMI_CODE_HOME." >&2; exit 1; }
    grep -q 'no detectado' "$TMP_ROOT/kimi-install-off.log"
else
    echo "[info] 'kimi' esta en PATH: se omite la rama de no-deteccion (la cubre --no-kimi)."
fi
# En todas las ramas los artefactos DE PROYECTO se generan igual.
test -f "$KIMI_E2E/.kimi-code/agents/leader.md"
test -x "$KIMI_E2E/bin/harness-kimi"

# (e) AC-9e: --reset limpia .kimi-code/agents del proyecto (con backup previo)
# y NO toca el bloque global (decision usuario 2026-07-28: es compartido por
# todos los proyectos de la maquina).
cp "$KIMI_HOME_USER/config.toml" "$TMP_ROOT/kimi-user-config.pre-reset"
kimi_setup "$KIMI_HOME_USER" --reset > "$TMP_ROOT/kimi-reset.log" 2>&1
test ! -e "$KIMI_E2E/.kimi-code/agents" \
    || { echo "[!] --reset debio limpiar .kimi-code/agents del proyecto." >&2; exit 1; }
test ! -e "$KIMI_E2E/bin/harness-kimi" \
    || { echo "[!] --reset debio limpiar bin/harness-kimi." >&2; exit 1; }
find "$KIMI_BKP" -path '*.kimi-code/agents.bak.*' -print -quit | grep -q . \
    || { echo "[!] falta el backup de .kimi-code/agents en el reset." >&2; exit 1; }
cmp -s "$TMP_ROOT/kimi-user-config.pre-reset" "$KIMI_HOME_USER/config.toml" \
    || { echo "[!] --reset NO debe tocar el bloque global de hooks de Kimi." >&2; exit 1; }
echo "[Ok] Kimi Code: espejos por rol (root+subdir, allowlist de tools), gate de espejo stale/warn/off, bloque global blindado (crea/no-duplica/backup/rollback doctor), ramas --no-kimi y sin-deteccion, --reset conserva lo global."

echo "[Ok] docs del arnes en el docs/ de la RAIZ: destino, migracion, no-pisa y reset."
echo "[Ok] planillas maestras docs/prd/ (PRD + SDD): siembra, no-pisa y supervivencia al reset."
echo "[Ok] setup smoke: Rust-only, gate de credenciales, layouts, reinstall, dry-run, version, reset."
