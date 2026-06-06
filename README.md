# Harness Process

Instalador de un arnes multi-repo para Claude Code, Codex, Gemini, Grok,
Antigravity y otros agentes CLI. Genera superficies de instrucciones, hooks,
launchers, memoria compartida y una capa opcional de subagentes.

## Requisitos

- Bash 3.2 o superior
- Git
- Python 3
- `curl`, `uv` o `pipx` solo cuando se instalan herramientas opcionales

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

Para instalar el arnes directamente en la raiz multi-repo:

```bash
./setup_harness.sh --root
```

Instalacion sin graphify ni cambios globales adicionales:

```bash
./setup_harness.sh \
  --no-graphify \
  --no-graphify-skills \
  --no-antigravity
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

## Verificacion

```bash
bash init.sh
bash harness_status.sh
bash harness_check.sh
```
