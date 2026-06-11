# Actualización del Harness Process

El Harness Process se mantiene actualizado **re-ejecutando el instalador** desde la carpeta fuente (`harness_process`). Esto es intencional y explícito.

No existe un comando mágico `harness_cli upgrade` dentro de tus proyectos. La forma correcta de traer mejoras es volver a correr el instalador.

## Por qué funciona así

- Las mejoras al protocolo (por ejemplo: `check-plan` para detectar si otros LLMs actualizaron planes, mejores instrucciones para implementer/reviewer, nuevos comandos, etc.) viven en este repositorio.
- Las superficies (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`) y los subagentes se **generan** desde el instalador.
- Los scripts (`harness_cli`, `harness.py`, `harness_check.sh`, roles, etc.) se copian desde `templates/`, y el binario Rust `harness` se compila desde `rust/` durante el setup (si hay cargo).
- Re-correr el instalador asegura que todos los proyectos y todos los agentes (Claude, Gemini, Antigravity, Grok, Codex...) usen la misma versión actualizada del flujo.

## Cómo actualizar

Desde la carpeta del `harness_process` (la fuente):

```bash
# Actualización normal (recomendada)
./setup_harness.sh

# O reinstalación limpia (borra superficies anteriores y las regenera)
./setup_harness.sh --reset
```

El instalador hace backups automáticos de los archivos que reemplaza (en `bkp/`) a menos que uses `--force`.

## Cuándo actualizar

- Después de hacer `git pull` o `git fetch` en la carpeta `harness_process`.
- Cuando `harness_status.sh` o las superficies muestren recordatorios de nuevas funcionalidades.
- Periódicamente para beneficiarte de mejoras en el manejo multi-LLM (detección de planes actualizados por otros agentes, mejores checkpoints, etc.).
- Cuando agregues un nuevo LLM al equipo (para asegurarte de que tenga los últimos roles y hooks).

## Qué se actualiza

- Superficies de instrucciones (CLAUDE.md, AGENTS.md, etc.)
- Subagentes nativos (`.claude/agents/`, `.codex/agents/`, `.gemini/agents/`)
- Scripts del arnés (`harness_cli`, `harness.py`, `harness_check.sh`, `harness_status.sh`, roles, etc.)
- El binario Rust `harness` (recompilado con cargo si está disponible; sin cargo, `harness_cli` usa el fallback Python automáticamente)
- Hooks y launchers
- Documentación interna como `CHECKPOINTS.md` y este mismo `UPDATING.md`

## Mantenimiento dual Rust + Python (obligatorio para maintainers)

El punto de entrada es **`harness_cli`**: ejecuta el binario Rust `harness` (compilado desde `rust/`, corre en macOS/Windows/Linux) y, si no existe, cae al fallback Python (`harness.py` / `graph_memory.py`). Ambas implementaciones conviven y DEBEN mantenerse en paridad:

1. Los `.py` son el **oráculo de comportamiento**: todo cambio de comportamiento en `templates/harness.py` o `templates/graph_memory.py` se espeja en `rust/src/` **en el mismo commit** (y viceversa).
2. `bash tests/parity_smoke.sh` debe pasar antes de cada push: ejecuta el mismo escenario en ambas implementaciones y compara exit codes, salidas y archivos de estado.
3. Sube la versión en `rust/Cargo.toml` en cada cambio de comportamiento.
4. Las máquinas sin cargo siguen funcionando con el fallback; si un binario viejo quedara desactualizado, `rm harness harness.exe` en la instalación reactiva el fallback hasta el próximo setup con cargo.

## Recomendación

Mantén este repositorio (`harness_process`) actualizado y re-instala en tus proyectos cuando haya cambios relevantes. Así el protocolo de trabajo multi-agente se mantiene consistente y mejora con el tiempo.

**NUNCA commitees la carpeta del harness** en los proyectos donde lo instalas. El instalador la agrega automáticamente a `.gitignore`.

Si usas `--reset` + re-instalación, las superficies se regeneran desde cero con la versión más reciente del protocolo.

## Para maintainers de este repositorio harness_process

Cuando realizas mejoras (nuevo protocolo, recordatorios de planes actualizados por otros LLMs, fixes en harness.py + su espejo en rust/src, etc.):

1. Haz los cambios en este repo (el "fuente").
2. Una vez hecho el commit **sin co-author** (sin `Co-Authored-By`, sin "Generated with", sin trailers de IA):
   ```bash
   git commit -m "tu mensaje limpio"
   ```
3. Haz push del cambio:
   ```bash
   git push origin main
   ```

Esto hace que el cambio esté disponible **incluso si aplica en otros proyectos**. Los demás proyectos que usan este harness_process como fuente recibirán las mejoras la próxima vez que ejecuten:

```bash
./setup_harness.sh
# o
./setup_harness.sh --reset
```

Mantener el proceso explícito asegura consistencia multi-LLM a través de todos los proyectos.

## Cómo obtener este archivo si te falta al actualizar

Si al correr `./setup_harness.sh` ves el error "Falta el recurso requerido: UPDATING.md", significa que estás usando una versión actualizada del instalador pero aún no tienes el archivo UPDATING.md en tu carpeta `harness_process`.

Solución rápida:
- Copia este archivo `UPDATING.md` a tu carpeta `harness_process/` (junto a `setup_harness.sh`).
- O, si usas la estructura con `templates/`, colócalo dentro de `templates/`.
- Luego vuelve a ejecutar `./setup_harness.sh` (recomendado con `--reset` para una actualización limpia).

Este archivo se copiará automáticamente a los proyectos destino en futuras instalaciones.