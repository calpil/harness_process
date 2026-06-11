# Harness Process

Instalador de un arnes multi-repo para Claude Code, Codex, Gemini, Grok,
Antigravity y otros agentes CLI. Genera superficies de instrucciones, hooks,
launchers, memoria compartida y una capa opcional de subagentes.

## Requisitos

- Bash 3.2 o superior para `setup_harness.sh` y los hooks POSIX existentes
- Windows PowerShell 5.1 o PowerShell 7 para `setup_harness.ps1`
- Git
- Rust + cargo (requerido): compila el binario nativo `harness` durante el setup.
  `harness_cli` (sh/ps1) despacha exclusivamente al binario; sin el binario falla.
- `curl`, `uv` o `pipx` solo cuando se instalan herramientas opcionales (graphify etc)

## Instalacion

El layout predeterminado es `subdir`: este repositorio vive dentro de la raiz
multi-repo y escribe las superficies de agente en el directorio padre.
La carpeta `templates/` pertenece a este repositorio fuente. Una distribucion
aplanada puede dejar esos archivos junto a `setup_harness.sh`; el instalador no
exige ni crea `templates/` en el proyecto destino.

```bash
cd /ruta/al/proyecto/harness_process
./setup_harness.sh
```

En Windows:

```powershell
cd C:\ruta\al\proyecto\harness_process
.\setup_harness.ps1
```

PowerShell busca `cargo.exe` en `PATH`, `$env:CARGO_HOME\bin` y
`$HOME\.cargo\bin`. Si rustup todavia no actualizo la sesion, agrega la carpeta
de Cargo al `PATH` del proceso antes de compilar. Se puede fijar el target:

```powershell
$env:CARGO_HOME = "$HOME\.cargo"
.\setup_harness.ps1 -CargoTargetDir "$PWD\.cargo-target"
```

El instalador agrega `harness_cli.ps1`, que ejecuta `harness.exe` (Rust). Git for Windows Bash sigue siendo necesario para scripts/hook POSIX historicos; ambos instaladores se mantienen. (Sin fallback Python desde feature #2).

Para instalar el arnes directamente en la raiz multi-repo:

```bash
./setup_harness.sh --root
```

```powershell
.\setup_harness.ps1 -Root
```

Instalacion sin graphify ni cambios globales adicionales:

```bash
./setup_harness.sh \
  --no-graphify \
  --no-graphify-skills \
  --no-antigravity
```

```powershell
.\setup_harness.ps1 -NoGraphify -NoGraphifySkills -NoAntigravity
```

El Memory Hub usa exclusivamente PostgreSQL. Configura la conexion en el entorno:

```bash
export DB_HOST=localhost
export DB_USER=harness
export DB_PASSWORD='...'
export DB_NAME=harness
export DB_SSL_MODE=require
./setup_harness.sh
```

```powershell
$env:DB_HOST = "localhost"
$env:DB_USER = "harness"
$env:DB_PASSWORD = "..."
$env:DB_NAME = "harness"
$env:DB_SSL_MODE = "require"
.\setup_harness.ps1
```

Tambien se pueden guardar esas variables en `$HARNESS_HUB/.env`.
`DB_SSL_MODE` usa `require` por defecto.

Al actualizar una instalacion antigua, `graph_db.json` y `progress/` se migran
a PostgreSQL. Luego se respaldan bajo `bkp/memory-hub/` y se eliminan del Hub
activo. Las consultas posteriores se realizan solo en PostgreSQL.

## Opciones

Ejecuta `./setup_harness.sh --help` para ver todas las opciones. Las mas utiles:

- `--root` / `--subdir`: selecciona el layout.
- `--no-subagents`: omite roles y backlog ejecutable.
- `--no-graphify`: no instala el CLI de graphify.
- `--no-graphify-skills`: no modifica skills globales de agentes.
- `--no-antigravity`: no instala Antigravity CLI.
- `--force`: sobrescribe sin crear backups.
- `--dry-run` (o `--preview`): modo simulado, no escribe ni instala nada (ideal para auditar).
- `--reset`: limpia todas las superficies, hooks, agentes, binarios y marcadores generados por el arnes (respaldando primero). No toca tu codigo.
- `--version`: muestra la version del instalador.
- `--json`: emite al final un reporte JSON con contadores de acciones.
- `--log-file <ruta>`: escribe log plano (sin ANSI) a un archivo.
- `--config <ruta>`: carga variables de entorno extra desde un archivo (se evalua temprano).

PowerShell usa los equivalentes `-Root`, `-Subdir`, `-NoSubagents`,
`-NoGraphify`, `-NoGraphifySkills`, `-NoAntigravity`, `-Force`, `-DryRun`,
`-Reset`, `-Version`, `-Help`, `-Json`, `-LogFile`, `-Config` y
`-CargoTargetDir`.

Los backups se guardan en `bkp/`. Usa `HARNESS_BKP_DIR` para cambiar la ruta.

Nuevas mejoras (2026 best practices aplicadas):
- shebang portable (`#!/usr/bin/env bash`) + shellcheck-ready
- logging con colores + niveles
- lockfile anti-concurrencia
- reintentos con backoff en descargas
- descarga verificada (no pipe ciego) para Antigravity CLI
- guidance de PATH despues de installs --user
- soporte config file + dry-run + reset + reporte de idempotencia

Ejemplo dry-run:
```bash
./setup_harness.sh --dry-run --json
```

## Actualizacion (proceso explicito)

El Harness Process se actualiza **re-correndo el instalador**. Esto es intencional y explicito:

- Las superficies (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`) y los subagentes se generan desde los heredocs del instalador.
- Los scripts (`harness_cli`, `harness.py`, `harness_check.sh`, roles, etc.) se copian desde `templates/`.
- El binario Rust `harness` se compila desde `rust/` con cargo (si esta disponible) y `harness_cli` lo prefiere sobre el fallback Python.

Para recibir mejoras (nuevo protocolo de `check-plan`, recordatorios de planes actualizados por otros LLMs, fixes, nuevas opciones, etc.):

```bash
# Ve a la carpeta del harness_process (la fuente)
cd /ruta/al/harness_process

# Actualizacion normal (hace backups de lo anterior)
./setup_harness.sh

# O para una reinstalacion limpia de las superficies:
./setup_harness.sh --reset
```

El instalador respalda archivos existentes en `bkp/` (a menos que uses `--force`).

**NUNCA commitees la carpeta del harness** (`harness_process/` o el subdirectorio donde está `setup_harness.sh`). 

El instalador la agrega automáticamente a `.gitignore` del proyecto. El harness es una **herramienta** que vive en su propio repositorio fuente separado. No forma parte del código de tu proyecto.

No existe (ni se recomienda) un comando magico `harness_cli upgrade` dentro del proyecto. La forma correcta y explícita de actualizar es volver a ejecutar el instalador desde la carpeta fuente de `harness_process`.

## harness_cli: binario Rust + fallback Python

Todos los hooks, scripts y docs invocan `sh .../harness_cli <cmd>`:

- Si existe el binario `harness` (o `harness.exe` en Windows), lo ejecuta.
  Es un solo ejecutable multi-OS (macOS/Windows/Linux) con los comandos de
  ciclo de vida al tope (`status`, `start`, `check-plan`, ...) y el Memory
  Hub bajo `harness graph <cmd>` (`mapa`, `impacto`, `vincular`, ...).
- Si no, cae a `python3 harness.py` / `graph_memory.py` (mismos comandos,
  mismos mensajes, mismos exit codes).

Regla de mantenedor: los `.py` son el oraculo; cualquier cambio de
comportamiento se espeja en `rust/src/` en el mismo commit y
`bash tests/parity_smoke.sh` debe pasar antes de push (compara ambas
implementaciones paso a paso). Detalles en `templates/UPDATING.md`.

## Verificacion

```bash
bash init.sh
bash harness_status.sh
bash harness_check.sh

# Suites del repo fuente
bash tests/setup_smoke.sh     # instalador (layouts, hooks, build-on-setup)
bash tests/parity_smoke.sh    # paridad Rust vs Python (oraculo)
(cd rust && cargo clippy --all-targets -- -D warnings && cargo test)
```

```powershell
.\tests\setup_smoke.ps1
.\harness_cli.ps1 status
```
