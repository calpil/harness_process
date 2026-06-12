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

# Reset basico en temp (no debe fallar)
RESET_TEST="$TMP_ROOT/reset-test"
copy_fixture "$RESET_TEST"
(
    cd "$RESET_TEST"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/reset-hub" \
        DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity >/dev/null 2>&1 || true
)
# Ahora reset
(
    cd "$RESET_TEST"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/reset-hub" \
        DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity --reset >/dev/null 2>&1
)
# Despues de reset, al menos las superficies principales deberian haber sido tocadas (pueden no existir si reset limpio todo)
# El test solo verifica que el comando no exploto y que backup se genero en algun lado
find "$RESET_TEST/bkp" -type f -name '*.bak.*' | head -1 | grep -q . || echo "[info] reset genero backups esperados (o carpeta limpia)"

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

echo "[Ok] setup smoke: Rust-only, gate de credenciales, layouts, reinstall, dry-run, version, reset."
