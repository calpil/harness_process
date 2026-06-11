#!/usr/bin/env bash
# Suite de paridad Rust vs Python: ejecuta el MISMO escenario en dos sandboxes
# (A = harness.py/graph_memory.py como oraculo, B = binario Rust) y compara
# exit codes, stdout/stderr y archivos de estado normalizados.
#
# No requiere base de datos ni red: con DB_* sin setear, el registro al hub
# degrada en ambas implementaciones con el mismo mensaje best-effort a stderr.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PY="$(command -v python3)" || { echo "[parity] python3 es requerido (es el oraculo)"; exit 2; }

# --- Binario Rust (release; el incremental de cargo es la cache) -------------
if ! command -v cargo >/dev/null 2>&1; then
    echo "[parity] cargo no esta disponible; no hay binario que comparar"
    exit 2
fi
(cd "$REPO_ROOT/rust" && cargo build --release --quiet)
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}"
RUST_BIN="$TARGET_DIR/release/harness"
[ -x "$RUST_BIN" ] || { echo "[parity] no se encontro $RUST_BIN"; exit 2; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# --- Fake psycopg2: fuerza la rama "faltan DB_*" (no la de "no instalado") ---
FAKE="$WORK/fake_py"
mkdir -p "$FAKE/psycopg2"
cat > "$FAKE/psycopg2/__init__.py" <<'EOF'
def connect(**kwargs):
    raise RuntimeError("fake psycopg2: sin conexion real en la suite de paridad")
EOF
: > "$FAKE/psycopg2/extensions.py"
cat > "$FAKE/psycopg2/extras.py" <<'EOF'
class Json:
    def __init__(self, adapted):
        self.adapted = adapted
EOF

# --- Normalizador: enmascara timestamps/mtimes/rutas para comparar ----------
NORM="$WORK/normalize.py"
cat > "$NORM" <<'EOF'
import json, re, sys

root = sys.argv[1]
path = sys.argv[2]
is_json = path.endswith(".json")

with open(path, "r", encoding="utf-8") as fh:
    text = fh.read()

if is_json:
    data = json.loads(text)
    for f in data.get("features", []):
        for key in ("started_at", "closed_at"):
            if key in f:
                f[key] = "@TS@"
        sig = f.get("last_plan_sig")
        if isinstance(sig, dict) and "mtime" in sig:
            sig["mtime"] = 0
    # sin sort_keys: el ORDEN de claves tambien es parte de la paridad
    text = json.dumps(data, indent=2, ensure_ascii=False)

text = re.sub(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", "@TS@", text)
text = re.sub(r"mtime=\d+(\.\d+)?", "mtime=@M@", text)
text = text.replace(root, "@ROOT@")
sys.stdout.write(text)
EOF

# --- Sandboxes: <root>/{docs,hp/{harness.py,graph_memory.py,progress}} ------
make_sandbox() {
    local root="$1" kind="$2"
    mkdir -p "$root/hp/progress" "$root/docs" "$root/hub" "$root/home"
    cp "$REPO_ROOT/templates/harness.py" "$root/hp/harness.py"
    cp "$REPO_ROOT/templates/graph_memory.py" "$root/hp/graph_memory.py"
    printf 'subdir' > "$root/hp/.harness_layout"
    if [ "$kind" = rust ]; then
        cp "$RUST_BIN" "$root/hp/harness"
    fi
}

A="$WORK/a"
B="$WORK/b"
make_sandbox "$A" python
make_sandbox "$B" rust

# Entorno controlado por sandbox: DB_* fuera, hub y HOME propios.
run_in() {
    local root="$1" kind="$2"
    shift 2
    local -a cmd
    if [ "$kind" = python ]; then
        if [ "${1:-}" = graph ]; then
            shift
            cmd=("$PY" "$root/hp/graph_memory.py" "$@")
        else
            cmd=("$PY" "$root/hp/harness.py" "$@")
        fi
    else
        cmd=("$root/hp/harness" "$@")
    fi
    env -u DB_HOST -u DB_USER -u DB_PASSWORD -u DB_NAME -u DB_PORT -u DB_SSL_MODE \
        -u HARNESS_REPO_ROOT -u HARNESS_PROJECT \
        HOME="$root/home" HARNESS_HUB="$root/hub" PYTHONPATH="$FAKE" \
        "${cmd[@]}"
}

FAILURES=0
STEP_N=0

normalized() { "$PY" "$NORM" "$1" "$2"; }

compare_streams() {
    local label="$1" file_a="$2" file_b="$3"
    local na nb
    na="$(normalized "$A" "$file_a" 2>/dev/null || true)"
    nb="$(normalized "$B" "$file_b" 2>/dev/null || true)"
    if [ "$na" != "$nb" ]; then
        echo "[FAIL] paso $STEP_N: $label difiere"
        diff <(printf '%s' "$na") <(printf '%s' "$nb") | head -30 || true
        FAILURES=$((FAILURES + 1))
    fi
}

compare_state_files() {
    # Mismos archivos de estado en ambos sandboxes, normalizados.
    local rel files_a files_b
    files_a="$(cd "$A" && ls docs/*.md hp/feature_list.json hp/progress/current.md hp/progress/history.md 2>/dev/null | sort || true)"
    files_b="$(cd "$B" && ls docs/*.md hp/feature_list.json hp/progress/current.md hp/progress/history.md 2>/dev/null | sort || true)"
    if [ "$files_a" != "$files_b" ]; then
        echo "[FAIL] paso $STEP_N: listas de archivos difieren"
        diff <(printf '%s\n' "$files_a") <(printf '%s\n' "$files_b") || true
        FAILURES=$((FAILURES + 1))
        return
    fi
    for rel in $files_a; do
        compare_streams "archivo $rel" "$A/$rel" "$B/$rel"
    done
}

# step <modo> <args...>   modo: full (todo) | code (solo exit code)
step() {
    local mode="$1"
    shift
    STEP_N=$((STEP_N + 1))
    local ec_a=0 ec_b=0
    run_in "$A" python "$@" > "$WORK/a.out" 2> "$WORK/a.err" || ec_a=$?
    run_in "$B" rust "$@" > "$WORK/b.out" 2> "$WORK/b.err" || ec_b=$?
    if [ "$ec_a" != "$ec_b" ]; then
        echo "[FAIL] paso $STEP_N ($*): exit $ec_a (py) vs $ec_b (rust)"
        echo "  py stderr:  $(head -2 "$WORK/a.err")"
        echo "  rust stderr: $(head -2 "$WORK/b.err")"
        FAILURES=$((FAILURES + 1))
    elif [ "$mode" = full ]; then
        compare_streams "stdout de '$*'" "$WORK/a.out" "$WORK/b.out"
        compare_streams "stderr de '$*'" "$WORK/a.err" "$WORK/b.err"
    fi
    compare_state_files
}

# Mutacion identica en ambos sandboxes (simula a "otro LLM" editando).
mutate_both() {
    local rel="$1" text="$2"
    printf '%s' "$text" >> "$A/$rel"
    printf '%s' "$text" >> "$B/$rel"
}

echo "[parity] escenario de ciclo de vida (sin DB)..."

step full status
step full next
step full add --name "Pago con QR" --service demo/ms-pagos-service --acceptance "boleta emitida"
step full add --name "Reportes ñ UTF-8"
step full next
step full status
step full start --feature 1
step full check-plan
step full nudge
# Otro agente edita el plan -> stale (hash distinto)
mutate_both "docs/plan-feature-1-pago-con-qr.md" $'\n## Editado por otro agente\n'
step full check-plan
step full nudge
step full status
step full advance --nota "hito uno"
step full check-plan
# Un doc nuevo del proyecto dispara el autocheck
printf 'notas\n' > "$A/docs/notas.md"
printf 'notas\n' > "$B/docs/notas.md"
step full autocheck --no-graphify
step full autocheck --no-graphify
step full close --feature 1 --status done --note "listo"
step full nudge
step full nudge
step full start --feature 2
step full start --feature 1
step full advance --feature 2 --nota "avance con ñ"
step full close --feature 2 --status blocked
step full check-plan --feature 99
# Errores de uso: argparse y clap difieren en texto, no en exit code
step code close --feature 1 --status bogus
step code advance
# Comandos graph sin DB: el manager se construye ANTES de validar args,
# asi que TODOS salen 1 con el mismo mensaje (incluso vincular sin --destino)
step full graph mapa
step full graph registrar --accion implementar --estado WIP --artefacto feature-9
step full graph vincular
step full graph sync_git --artefacto abc1234def --meta "x.go,y.md"

if [ "$FAILURES" -gt 0 ]; then
    echo "[parity] FALLO: $FAILURES diferencia(s) en $STEP_N pasos"
    exit 1
fi
echo "[parity] OK: $STEP_N pasos identicos entre Python y Rust"
