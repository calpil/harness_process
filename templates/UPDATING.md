# Actualización del Harness Process

El Harness Process se mantiene actualizado **re-ejecutando el instalador** desde la carpeta fuente (`harness_process`). Esto es intencional y explícito.

No existe un comando mágico `harness_cli upgrade` dentro de tus proyectos. La forma correcta de traer mejoras es volver a correr el instalador.

## Por qué funciona así

- Las mejoras al protocolo (por ejemplo: `check-plan` para detectar si otros LLMs actualizaron planes, mejores instrucciones para implementer/reviewer, nuevos comandos, etc.) viven en este repositorio.
- Las superficies (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`) y los subagentes se **generan** desde el instalador.
- Los scripts (`harness_cli`, `harness_check.sh`, roles, etc.) se copian desde `templates/`, y el binario Rust `harness` se compila desde `rust/` durante el setup (cargo requerido).
- Re-correr el instalador asegura que todos los proyectos y todos los agentes (Claude, Gemini, Antigravity, Grok, Codex...) usen la misma versión actualizada del flujo.

## Cómo actualizar

Desde la carpeta del `harness_process` (la fuente):

```bash
# Actualización normal (recomendada)
./setup_harness.sh

# O reinstalación limpia (borra superficies anteriores y las regenera)
./setup_harness.sh --reset
```

En Windows se mantiene un instalador paralelo:

```powershell
.\setup_harness.ps1
.\setup_harness.ps1 -Reset
```

`setup_harness.ps1` configura Cargo para la sesion desde `PATH`,
`$env:CARGO_HOME\bin` o `$HOME\.cargo\bin`, compila `harness.exe` con
`cargo build --release --locked` y despliega `harness_cli.ps1`. El instalador
Bash sigue disponible y tambien copia el shim PowerShell.

El instalador hace backups automáticos de los archivos que reemplaza (en `bkp/`) a menos que uses `--force`.

Notas de robustez (2026-06):

- `$HARNESS_HUB/.env` se **parsea** línea a línea (ya no se sourcea): un
  `DB_PASSWORD` con metacaracteres no necesita quoting especial. Recomendado
  igualmente: `chmod 600 ~/.harness-hub/.env`.
- El estado por-instalación (`feature_list.json`, `progress/`) **no se
  versiona** en este repo: cada proyecto mantiene el suyo y el instalador lo
  siembra desde `templates/` solo si falta. Si actualizas el harness con
  `git pull` y choca el estado, conserva SIEMPRE tu versión local.

### Migración única (2026-06): conflicto modify/delete al hacer pull

Si tu clon instalado tenía un commit local con el estado vivo, el primer
`git pull --rebase` tras esta versión choca con
`CONFLICT (modify/delete): feature_list.json ...`. Es esperado y pasa UNA
sola vez. Resuélvelo conservando tu estado (queda en disco, sin versionar):

```bash
# dentro del clon harness_process, con el rebase en conflicto:
mkdir -p /tmp/harness-state-bkp progress
cp -f feature_list.json /tmp/harness-state-bkp/ 2>/dev/null || true
cp -f progress/current.md progress/history.md /tmp/harness-state-bkp/ 2>/dev/null || true

git rm -q -f feature_list.json progress/current.md progress/history.md 2>/dev/null || true
GIT_EDITOR=true git rebase --continue || git rebase --skip

mkdir -p progress
cp -f /tmp/harness-state-bkp/feature_list.json feature_list.json 2>/dev/null || true
cp -f /tmp/harness-state-bkp/current.md progress/current.md 2>/dev/null || true
cp -f /tmp/harness-state-bkp/history.md progress/history.md 2>/dev/null || true

git status -sb   # limpio; tu backlog sigue en disco y ya no se versiona
```

Los pulls siguientes ya no chocan: el estado quedó fuera de git en ambos
lados.
- El instalador se niega a escribir superficies en tu `$HOME` (protege
  `.claude/settings.json` y agentes globales). Escape consciente:
  `HARNESS_ALLOW_HOME_SURFACE=1`.

## Cuándo actualizar

- Después de hacer `git pull` o `git fetch` en la carpeta `harness_process`.
- Cuando `harness_status.sh` o las superficies muestren recordatorios de nuevas funcionalidades.
- Periódicamente para beneficiarte de mejoras en el manejo multi-LLM (detección de planes actualizados por otros agentes, mejores checkpoints, etc.).
- Cuando agregues un nuevo LLM al equipo (para asegurarte de que tenga los últimos roles y hooks).

## Qué se actualiza

- Superficies de instrucciones (CLAUDE.md, AGENTS.md, etc.)
- Subagentes nativos (`.claude/agents/`, `.codex/agents/`, `.gemini/agents/`)
- Scripts del arnés (`harness_cli`, `harness_check.sh`, `harness_status.sh`, roles, etc.)
- El binario Rust `harness` (recompilado con cargo si está disponible; sin cargo, `harness_cli` usa el fallback Python automáticamente)
- Hooks y launchers
- Documentación interna como `CHECKPOINTS.md` y este mismo `UPDATING.md`

## Mantenimiento Rust only (post feature #2)

El punto de entrada es **`harness_cli`** (sh y .ps1): ejecuta **exclusivamente** el binario Rust `harness` / `harness.exe` (compilado desde `rust/`). 

- Sin binario: harness_cli falla con mensaje claro pidiendo cargo/rustup + re-setup.
- No hay fallback Python, ni parity, ni .py en templates/ o raiz.
- Cambios en rust/src + shims/setup + tests + docs deben ser verificados con cargo test/clippy + setup_smoke (bash+ps1) + harness_check.sh .

Sube version en Cargo.toml cuando haya cambios de comportamiento visibles en el CLI/hub.

## Recomendación

Mantén este repositorio (`harness_process`) actualizado y re-instala en tus proyectos cuando haya cambios relevantes. Así el protocolo de trabajo multi-agente se mantiene consistente y mejora con el tiempo.

**NUNCA commitees la carpeta del harness** en los proyectos donde lo instalas. El instalador la agrega automáticamente a `.gitignore`.

Si usas `--reset` + re-instalación, las superficies se regeneran desde cero con la versión más reciente del protocolo.

## Para maintainers de este repositorio harness_process

Cuando realizas mejoras (nuevo protocolo, fixes en rust/src + shims/setup, actualizaciones por otros LLMs, etc.):

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
