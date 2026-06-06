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

```bash
cd /ruta/al/proyecto/harness_process
./setup_harness.sh
```

Para instalar el arnes directamente en la raiz multi-repo:

```bash
./setup_harness.sh --root
```

Instalacion offline, sin descargas ni cambios globales:

```bash
./setup_harness.sh \
  --no-graphify \
  --no-graphify-skills \
  --no-antigravity \
  --json-hub
```

El Hub usa PostgreSQL por defecto. Configura la conexion en el entorno:

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
Para una instalacion local sin PostgreSQL usa `--json-hub`; ese modo guarda el
grafo en `~/.harness-hub/graph_db.json`.

## Opciones

Ejecuta `./setup_harness.sh --help` para ver todas las opciones. Las mas utiles:

- `--root` / `--subdir`: selecciona el layout.
- `--no-subagents`: omite roles y backlog ejecutable.
- `--no-graphify`: no instala el CLI de graphify.
- `--no-graphify-skills`: no modifica skills globales de agentes.
- `--no-antigravity`: no instala Antigravity CLI.
- `--json-hub`: usa almacenamiento JSON local en vez de PostgreSQL.
- `--force`: sobrescribe sin crear backups.

Los backups se guardan en `bkp/`. Usa `HARNESS_BKP_DIR` para cambiar la ruta.

## Verificacion

```bash
bash init.sh
bash harness_status.sh
bash harness_check.sh
```
