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
- Los scripts (`harness_cli`, `harness_check.sh`, roles, etc.) se copian desde `templates/`.
- El binario Rust `harness` se compila desde `rust/` con cargo (requerido); `harness_cli` despacha exclusivamente a el.

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

## harness_cli: binario Rust

Todos los hooks, scripts y docs invocan `sh .../harness_cli <cmd>`:

- `harness_cli` (sh/ps1) despacha exclusivamente al binario `harness` (o
  `harness.exe` en Windows). Es un solo ejecutable multi-OS
  (macOS/Windows/Linux) con los comandos de ciclo de vida al tope (`status`,
  `start`, `check-plan`, `check-spec`, ...) y el Memory Hub bajo
  `harness graph <cmd>` (`mapa`, `impacto`, `vincular`, ...).
- Sin el binario (cargo/rustup ausente) `harness_cli` falla pidiendo compilar;
  no hay fallback Python desde la feature #2.

Regla de mantenedor: cualquier cambio de comportamiento vive en `rust/src/` con
sus tests (`cargo test`, `cargo clippy -- -D warnings`) verdes antes de push.
Detalles en `templates/UPDATING.md`.

## Spec-Driven Development (SDD)

Cada feature arranca con un spec antes de tocar codigo (inspirado en spec-kit,
adaptado y en layout plano):

- `sh harness_cli start --feature <id>` genera, ademas del plan,
  `docs/spec-feature-<id>-<slug>.md` en el `docs/` de la RAIZ (junto a los
  planes, sin carpetas `specs/NNN/`) con `Estado: draft`: recorridos de usuario
  priorizados (P1/P2), criterios de aceptacion AC-n en Given/When/Then, no
  funcionales y fuera de alcance.
- El LIDER completa el spec y el plan (cada item de la Delegacion cita su AC-n)
  y ejecuta el **ritual de aprobacion**: le MUESTRA el spec al usuario (contenido
  en el chat + abierto en su editor), le PREGUNTA si lo aprueba y solo con su SI
  lo REGISTRA con `sh harness_cli approve-spec --yes [--nota "<como aprobo>"]`.
  El comando escribe `Estado: approved`, sella quien/cuando y re-firma el spec
  (por eso aprobar no dispara la alarma de "spec actualizado por otro LLM").
  Sin `--yes` el comando se niega con exit 2: ningun agente aprueba por su
  cuenta, y la decision sigue siendo exclusivamente del usuario.
- Gate `require_spec_approved`: con la regla `"require_spec_approved": true` en
  `rules` de `feature_list.json` y el spec sin aprobar, `advance`,
  `close --status done` y `harness_check.sh` bloquean con mensaje accionable.
  Sin la regla (o en `false`) el gate queda apagado (compat con instalaciones
  previas).
- `sh harness_cli check-spec` reporta el estado del gate (exit 0 aprobado o
  regla apagada, 1 sin feature activa, 2 spec stale o sin aprobar con la regla
  activa); `check-plan` vigila la frescura de spec y plan frente a ediciones de
  otros LLMs.
- `docs/constitution.md` (principios del proyecto) lo siembra el instalador
  (`setup_harness.sh` / `setup_harness.ps1`) solo si falta y nunca lo pisa;
  specs y planes deben cumplirlo y el reviewer lo verifica.

## harness_check.sh: gates de integridad

`bash harness_check.sh` (exit 0 limpio / 2 con fallos; `HARNESS_CHECK_MODE=block|warn|off`)
valida estado del backlog, frescura de plan/spec, checkpoints, commit guard y el
mapa de agentes. Desde la feature #7 incluye ademas:

- **Gate de espejo de roles**: `roles/*.md` es la fuente unica. El check compara
  el cuerpo embebido de `.claude/agents/*.md` (tambien leidos por Grok),
  `.gemini/agents/*.md` y `.codex/agents/*.toml` contra `roles/<rol>.md`, y
  `roles/*.md` contra `templates/roles/*.md` (modulo `__HREL__`). Un espejo
  desincronizado bloquea, nombrando el archivo; el remedio es re-correr el
  instalador (o propagar el cambio a `roles/` si lo editado fue el espejo). Los
  espejos que no existen no fallan.
- **Resolucion de raiz robusta**: los cuatro scripts (`harness_check.sh`,
  `harness_status.sh`, `init.sh`, `commit_guard.sh`) y el binario Rust resuelven
  `REPO_ROOT` con la misma regla: overrides primero (`HARNESS_REPO_ROOT`,
  variables de agente), luego el marker `.harness_layout`; y si el marker dice
  `subdir` pero el directorio es un checkout FUENTE (tiene
  `templates/harness_cli` + `rust/`) con un padre sin huella de instalacion (o
  el padre es `$HOME` sin `HARNESS_ALLOW_HOME_SURFACE=1`), la raiz es el propio
  checkout, con aviso informativo `[i]`. El marker ya no esta versionado (es
  estado local que escribe el instalador).

## Documentacion del proceso: toda en el `docs/` de la RAIZ

Con el arnes en una subcarpeta (`<proyecto>/harness_process/`, layout por
defecto), TODA la documentacion del proceso vive en el `docs/` de la RAIZ del
proyecto, junto a los docs del equipo:

```
miproyecto/
|-- docs/
|   |-- constitution.md              principios del proyecto (documento del usuario)
|   |-- architecture.md              mapa de arquitectura
|   |-- conventions.md               convenciones del equipo
|   |-- verification.md              comandos de validacion
|   |-- prd/
|   |   |-- PRD-master.md            que se construye y por que (planilla)
|   |   `-- SDD-master.md            como se construye, a nivel proyecto (planilla)
|   |-- spec-feature-<id>-<slug>.md  spec de la feature (AC-n)
|   |-- plan-feature-<id>-<slug>.md  plan del lider
|   `-- impl-<id>.md / review-<id>.md
`-- harness_process/                 binario, roles, progress/ (estado vivo)
```

### Proyectos que arrancan de cero: `docs/prd/`

`docs/prd/PRD-master.md` y `docs/prd/SDD-master.md` son planillas para completar
antes de cargar la primera feature. El flujo queda encadenado:

```
docs/prd/PRD-master.md   (hitos priorizados)
        |  sh harness_cli add --name <slug> --service <svc> --acceptance "<criterio>"
        v
feature_list.json        (backlog ejecutable)
        |  sh harness_cli start --feature <id>
        v
docs/spec-feature-<id>-<slug>.md  +  docs/plan-feature-<id>-<slug>.md
        |  aprobacion del usuario (Estado: approved)
        v
implementacion -> docs/impl-<id>.md -> docs/review-<id>.md
```

`docs/prd/` son documentos **tuyos**: se siembran una sola vez si faltan, ningun
reinstall los pisa y **`--reset` no los borra** (a diferencia de los tres docs del
arnes, que si se limpian por ser plantillas regenerables).

Los cuatro docs base se siembran **solo si faltan** y un reinstall **nunca los
pisa**: si tu equipo ya tiene un `docs/conventions.md`, queda intacto. Para
refrescar una plantilla, borra el archivo y reinstala (o usa `--force`, que por
contrato sobrescribe sin backup). `--reset` limpia solo los tres docs generados
y conserva la constitution y los artefactos de feature.

Instalaciones anteriores que tengan esos docs en `harness_process/docs/` se
migran solas al reinstalar: se mueven a la raiz si alli no existen, y si ya
existen no se pisa nada (el instalador avisa y deja la copia vieja donde esta).

## Verificacion

```bash
bash init.sh
bash harness_status.sh
bash harness_check.sh

# Suites del repo fuente
bash tests/setup_smoke.sh     # instalador (layouts, hooks, build-on-setup)
(cd rust && cargo clippy --all-targets -- -D warnings && cargo test)
```

```powershell
.\tests\setup_smoke.ps1
.\harness_cli.ps1 status
```
