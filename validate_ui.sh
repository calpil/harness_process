#!/bin/bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd -P)"
cd "$ROOT"

TARGET_URL="${1:-http://localhost:5173}"
echo "== Validando UI en $TARGET_URL =="

if ! command -v node >/dev/null 2>&1; then
    echo "[!] Node.js no esta disponible." >&2
    exit 1
fi

if [ ! -d "node_modules/playwright" ]; then
    if ! command -v npm >/dev/null 2>&1; then
        echo "[!] Playwright no esta instalado y npm no esta disponible." >&2
        exit 1
    fi
    echo "[Harness] Instalando Playwright localmente..."
    npm install playwright --no-save
    npx playwright install chromium
fi

if ! node debug_ui.js "$TARGET_URL"; then
    echo "[!] UI con errores. Captura: $ROOT/debug-ui/ui_error_state.png" >&2
    exit 1
fi

echo "[Ok] UI verificada."
