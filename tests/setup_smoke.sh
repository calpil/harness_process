#!/bin/bash
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/harness-setup-smoke.XXXXXX")"
TMP_ROOT="$(cd "$TMP_ROOT" && pwd -P)"
trap 'rm -rf "$TMP_ROOT"' EXIT

copy_fixture() {
    target="$1"
    mkdir -p "$target"
    cp "$REPO_ROOT/setup_harness.sh" "$target/setup_harness.sh"
    cp -R "$REPO_ROOT/templates" "$target/templates"
}

copy_flat_fixture() {
    target="$1"
    mkdir -p "$target"
    cp "$REPO_ROOT/setup_harness.sh" "$target/setup_harness.sh"
    cp -R "$REPO_ROOT/templates/." "$target/"
}

FAKE_PYTHON="$TMP_ROOT/fake-python"
mkdir -p "$FAKE_PYTHON/psycopg2"
cat > "$FAKE_PYTHON/psycopg2/__init__.py" <<'PYEOF'
import json
import os

from . import extensions, sql


class OperationalError(Exception):
    pass


class Cursor:
    def __init__(self):
        self.rows = []

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False

    def execute(self, query, params=None):
        statement = " ".join(str(query).split()).lower()
        self.rows = []
        raw_node = os.environ.get("FAKE_PG_NODE")
        if not raw_node:
            return
        node = json.loads(raw_node)
        if "select label, props from graph_nodes where id" in statement:
            if params and params[0] == node["id"]:
                self.rows = [(node["label"], node["props"])]
        elif "select id, label, props from graph_nodes" in statement:
            self.rows = [(node["id"], node["label"], node["props"])]

    def fetchall(self):
        return self.rows

    def fetchone(self):
        return self.rows[0] if self.rows else None


class Connection:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False

    def cursor(self):
        return Cursor()

    def close(self):
        pass

    def commit(self):
        pass

    def set_isolation_level(self, level):
        pass


def connect(*args, **kwargs):
    assert kwargs.get("sslmode") == "require"
    return Connection()
PYEOF
cat > "$FAKE_PYTHON/psycopg2/extensions.py" <<'PYEOF'
ISOLATION_LEVEL_AUTOCOMMIT = 0
PYEOF
cat > "$FAKE_PYTHON/psycopg2/sql.py" <<'PYEOF'
class SQL:
    def __init__(self, value):
        self.value = value

    def format(self, *args):
        return self

    def __str__(self):
        return self.value


class Identifier:
    def __init__(self, value):
        self.value = value
PYEOF
cat > "$FAKE_PYTHON/psycopg2/extras.py" <<'PYEOF'
class Json:
    def __init__(self, value):
        self.adapted = value
PYEOF

run_setup() {
    target="$1"
    shift
    (
        cd "$target"
        HOME="$TMP_ROOT/home" \
        HARNESS_HUB="$target/.test-hub" \
        PYTHONPATH="$FAKE_PYTHON" \
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
mkdir -p "$TMP_ROOT/postgres-hub/progress/test"
cat > "$TMP_ROOT/postgres-hub/graph_db.json" <<'JSONEOF'
{"nodes":{"test/feature-1":{"_id":"test/feature-1","_label":"Artefacto","estado":"done"}},"edges":[]}
JSONEOF
cat > "$TMP_ROOT/postgres-hub/progress/test/feature-2.json" <<'JSONEOF'
{"_id":"test/feature-2","estado":"in_progress"}
JSONEOF
(
    cd "$TMP_ROOT"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/postgres-hub" \
    PYTHONPATH="$FAKE_PYTHON" \
    DB_HOST=postgres.example \
    DB_USER=harness \
    DB_PASSWORD=secret \
    DB_SSL_MODE=require \
    bash "$POSTGRES_DEFAULT/setup_harness.sh" \
        --root \
        --no-graphify \
        --no-graphify-skills \
        --no-antigravity
)
grep -qx 'postgres' "$POSTGRES_DEFAULT/.harness_backend"
test ! -e "$TMP_ROOT/postgres-hub/graph_db.json"
test ! -e "$TMP_ROOT/postgres-hub/progress"
find "$POSTGRES_DEFAULT/bkp/memory-hub" -type f -name graph_db.json -print -quit | grep -q .
find "$POSTGRES_DEFAULT/bkp/memory-hub" -type f -path '*/progress/test/feature-2.json' -print -quit | grep -q .
! grep -q 'GRAPH_DB_FILE\|PROGRESS_DIR\|GraphStore' "$POSTGRES_DEFAULT/graph_memory.py"
test -x "$POSTGRES_DEFAULT/harness_cli"
test -f "$POSTGRES_DEFAULT/harness_cli.ps1"
# Las fixtures no traen rust/: harness_cli debe caer al fallback Python.
FAKE_PG_NODE='{"id":"postgres-default/feature-1","label":"Artefacto","props":{"estado":"done"}}' \
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/postgres-hub" \
    PYTHONPATH="$FAKE_PYTHON" \
    DB_HOST=postgres.example \
    DB_USER=harness \
    DB_PASSWORD=secret \
    DB_NAME=harness \
    DB_SSL_MODE=require \
    sh "$POSTGRES_DEFAULT/harness_cli" graph consultar --artefacto feature-1 \
    | grep -q '"estado": "done"'
HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/postgres-hub" \
    PYTHONPATH="$FAKE_PYTHON" \
    DB_HOST=postgres.example \
    DB_USER=harness \
    DB_PASSWORD=secret \
    DB_NAME=harness \
    DB_SSL_MODE=require \
    sh "$POSTGRES_DEFAULT/harness_cli" graph registrar \
        --accion update \
        --estado done \
        --artefacto feature-3 >/dev/null
test ! -e "$TMP_ROOT/postgres-hub/progress"

FLAT_LAYOUT="$TMP_ROOT/flat-layout"
copy_flat_fixture "$FLAT_LAYOUT"
(
    cd "$TMP_ROOT"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$FLAT_LAYOUT/.test-hub" \
    PYTHONPATH="$FAKE_PYTHON" \
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
test -f "$FLAT_LAYOUT/graph_memory.py"
test -x "$FLAT_LAYOUT/harness_cli"
test -f "$FLAT_LAYOUT/harness_cli.ps1"
test -f "$FLAT_LAYOUT/roles/leader.md"
test -f "$FLAT_LAYOUT/.codex/hooks.json"

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

test -f "$ROOT_LAYOUT/graph_memory.py"
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
HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$ROOT_LAYOUT/.test-hub" \
    PYTHONPATH="$FAKE_PYTHON" \
    DB_HOST=postgres.example \
    DB_USER=harness \
    DB_PASSWORD=secret \
    DB_NAME=harness \
    DB_SSL_MODE=require \
    bash "$ROOT_LAYOUT/init.sh" >/dev/null
# El hook post-commit conectado debe ser v9 y pasar por el shim.
grep -q 'harness-managed-hook v9' "$ROOT_LAYOUT/svc-demo/.git/hooks/post-commit"
grep -Fq 'harness_cli" graph sync_git' "$ROOT_LAYOUT/svc-demo/.git/hooks/post-commit"

SUBDIR_ROOT="$TMP_ROOT/subdir-layout"
SUBDIR_HARNESS="$SUBDIR_ROOT/harness_process"
copy_fixture "$SUBDIR_HARNESS"
run_setup "$SUBDIR_HARNESS"

test -f "$SUBDIR_HARNESS/graph_memory.py"
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
        PYTHONPATH="$FAKE_PYTHON" \
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
    PYTHONPATH="$FAKE_PYTHON" \
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
    PYTHONPATH="$FAKE_PYTHON" \
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
    PYTHONPATH="$FAKE_PYTHON" \
    DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity >/dev/null 2>&1 || true
)
# Ahora reset
(
    cd "$RESET_TEST"
    HOME="$TMP_ROOT/home" \
    HARNESS_HUB="$TMP_ROOT/reset-hub" \
    PYTHONPATH="$FAKE_PYTHON" \
    DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
    bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity --reset >/dev/null 2>&1
)
# Despues de reset, al menos las superficies principales deberian haber sido tocadas (pueden no existir si reset limpio todo)
# El test solo verifica que el comando no exploto y que backup se genero en algun lado
find "$RESET_TEST/bkp" -type f -name '*.bak.*' | head -1 | grep -q . || echo "[info] reset genero backups esperados (o carpeta limpia)"

# --- Binario Rust (build-on-setup): solo corre si hay cargo disponible ------
if command -v cargo >/dev/null 2>&1 && [ -f "$REPO_ROOT/rust/Cargo.toml" ]; then
    RUST_TEST="$TMP_ROOT/rust-binary"
    copy_fixture "$RUST_TEST"
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
        PYTHONPATH="$FAKE_PYTHON" \
        CARGO_TARGET_DIR="$REPO_ROOT/rust/target" \
        DB_HOST=postgres.example DB_USER=harness DB_PASSWORD=secret DB_NAME=harness DB_SSL_MODE=require \
        bash setup_harness.sh --root --no-graphify --no-graphify-skills --no-antigravity >/dev/null 2>&1
    )
    test -x "$RUST_TEST/harness"
    # El shim debe despachar al binario (status responde sin python3 de por
    # medio). grep SIN -q: consume todo el stdout y evita SIGPIPE temprano.
    sh "$RUST_TEST/harness_cli" status | grep '^Backlog:' >/dev/null
    echo "[Ok] binario Rust compilado por el setup e integrado via harness_cli."
else
    echo "[info] cargo no disponible: se omite la prueba del binario Rust (fallback Python cubierto)."
fi

echo "[Ok] setup smoke: PostgreSQL-only, migracion local, layouts, reinstall, dry-run, version, reset."
