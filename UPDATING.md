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

## Cuándo actualizar

- Después de hacer `git pull` o `git fetch` en la carpeta `harness_process`.
- Cuando `harness_status.sh` o las superficies muestren recordatorios de nuevas funcionalidades.
- Periódicamente para beneficiarte de mejoras en el manejo multi-LLM (detección de planes actualizados por otros agentes, mejores checkpoints, etc.).
- Cuando agregues un nuevo LLM al equipo (para asegurarte de que tenga los últimos roles y hooks).

## Qué se actualiza

- Superficies de instrucciones (CLAUDE.md, AGENTS.md, etc.)
- Subagentes nativos (`.claude/agents/`, `.codex/agents/`, `.gemini/agents/`)
- Scripts del arnés (`harness_cli`, `harness_check.sh`, `harness_status.sh`, roles, etc.)
- El binario Rust `harness` (recompilado con cargo, requerido; sin cargo/rustup `harness_cli` falla pidiendo instalarlo y re-correr el setup)
- Hooks y launchers
- Documentación interna como `CHECKPOINTS.md` y este mismo `UPDATING.md`
- `docs/constitution.md` (sembrada solo si falta; nunca pisa la del usuario)
- `docs/architecture.md`, `docs/conventions.md` y `docs/verification.md` en el
  `docs/` de la **raíz** del proyecto (mismo criterio: solo si faltan)
- `docs/prd/PRD-master.md` y `docs/prd/SDD-master.md` (planillas maestras del
  proyecto; solo si faltan, y `--reset` no las borra)
- El subcomando `harness_cli check-spec` y el gate de spec aprobado
  (`require_spec_approved`) en `advance`, `close --status done` y
  `harness_check.sh`
- El subcomando `harness_cli approve-spec --yes`, que registra la aprobación del
  usuario (sello + re-firma del spec)
- El gate de espejo de roles en `harness_check.sh` (compara el cuerpo embebido de
  `.claude/agents/*.md`, `.gemini/agents/*.md`, `.codex/agents/*.toml` y
  `.kimi-code/agents/*.md` contra `roles/*.md`) y la resolución de raíz robusta
  ante el checkout fuente
- El soporte de Kimi Code CLI: subagentes `.kimi-code/agents/*.md`, launcher
  `bin/harness-kimi` y el bloque de hooks globales en
  `KIMI_CODE_HOME/config.toml` (solo si se detecta Kimi; `--no-kimi` lo excluye)

## Spec-Driven Development (opt-in en instalaciones existentes)

Desde esta versión, `setup_harness.sh` / `setup_harness.ps1` siembran
`docs/constitution.md` (solo si falta) y `harness_cli` incorpora el subcomando
`check-spec` y el gate de spec aprobado. En instalaciones **nuevas** la regla
llega activada desde `templates/feature_list.json`.

En instalaciones **existentes** el gate queda **apagado por defecto**: el
`feature_list.json` de cada proyecto no se versiona ni se pisa, y el seed es
solo-si-falta, así que re-correr el instalador NO agrega la regla. Para activar
el gate hay que editar a mano el `feature_list.json` del proyecto y agregar la
regla a `rules`:

```json
{
  "rules": {
    "require_spec_approved": true
  }
}
```

Con la regla activa, `advance`, `close --status done` y `harness_check.sh`
exigen un spec `docs/spec-feature-<id>-<slug>.md` con `Estado: approved`. Sin la
regla, el flujo sigue como antes (gate apagado, compatibilidad total).

### Aprobación interactiva del spec (`approve-spec`)

La aprobación dejó de ser una edición manual del Markdown. El agente ejecuta el
**ritual de aprobación**: lee el spec, se lo **muestra** al usuario (contenido en
el chat y abierto en su editor), le **pregunta** si lo aprueba y solo con su
**sí** explícito lo **registra**:

```bash
sh harness_cli approve-spec --yes --nota "aprobado en el chat"
```

El comando escribe `Estado: approved`, inserta el sello
`Aprobado: <fecha> por USUARIO (confirmacion explicita)` y **re-firma**
`last_spec_sig`. Esa re-firma corrige un problema real del flujo manual: al
editar el spec a mano cambiaba su hash y `check-spec` reportaba la aprobación del
propio usuario como *"SPEC ACTUALIZADO POR OTRO LLM"*, obligando a un `advance`
para resincronizar. Si ya aprobaste un spec a mano, correr `approve-spec --yes`
lo re-firma y limpia esa falsa alarma sin duplicar el sello (es idempotente).

La decisión sigue siendo **exclusivamente del usuario**: sin `--yes` el comando
se niega (exit 2) y ningún agente puede aprobar por su cuenta. Exit codes: `0`
aprobado o ya aprobado, `1` sin feature `in_progress`, `2` sin confirmación o
spec ausente.

## Docs del arnés en el `docs/` de la raíz (migración automática)

Desde esta versión, `docs/architecture.md`, `docs/conventions.md` y
`docs/verification.md` se instalan en el `docs/` de la **raíz del proyecto**,
junto a `docs/constitution.md`, los specs y los planes. Antes vivían en
`<proyecto>/harness_process/docs/`, partiendo la documentación en dos lugares.

**No hay que hacer nada manualmente.** Al re-correr el instalador
(`setup_harness.sh` o `setup_harness.ps1`):

- Si el doc está en `harness_process/docs/` y **no** existe en el `docs/` de la
  raíz, se **mueve** (conserva tu contenido, no se regenera desde la plantilla) y
  el instalador lo informa con una línea `Migrado al docs/ de la raiz: ...`.
- Si **ya existe** en la raíz, no se pisa nada: se conserva tu archivo, la copia
  vieja queda donde está y el instalador avisa con un `WARN`.
- Si `harness_process/docs/` queda vacío tras la migración, se elimina.

Además cambió el criterio de refresco: estos tres docs ahora se siembran
**solo si faltan** (igual que la constitution), porque comparten carpeta con la
documentación del equipo y un `docs/conventions.md` propio no debe perderse en un
reinstall. Para refrescar una plantilla: borra el archivo y reinstala, o usa
`--force` (que por contrato sobrescribe **sin** backup).

`--reset` sigue limpiando solo lo generado —los tres docs, en su ubicación nueva
y en la vieja— y conserva la constitution y los artefactos de feature
(`spec-*`, `plan-*`, `impl-*`, `review-*`).

## Planillas maestras PRD y SDD (`docs/prd/`)

Desde esta versión el instalador siembra dos planillas para proyectos que
arrancan de cero, en el `docs/prd/` de la **raíz del proyecto**:

- `docs/prd/PRD-master.md` — qué se construye y por qué: problema, usuarios y
  jobs-to-be-done, métricas de éxito, alcance y no-objetivos, restricciones,
  recorridos P1/P2, tabla **Hitos → features** (cada fila se carga al backlog con
  `harness_cli add`), riesgos y decisiones abiertas.
- `docs/prd/SDD-master.md` — cómo se construye, a nivel proyecto: arquitectura
  objetivo, stack, contratos entre componentes, decisiones técnicas tipo ADR,
  datos, no funcionales, estrategia de verificación, riesgos y decisiones
  abiertas. Es distinto de `docs/architecture.md`, que mapea lo que **ya** existe.

Garantías (mismo criterio que `docs/constitution.md`):

- Se siembran **solo si faltan**. Un reinstall nunca las pisa, y **ni `--force`**
  las sobrescribe: lo que hay escrito ahí es tu proyecto, no una plantilla
  refrescable.
- **`--reset` no las borra.** No son superficie generada del arnés. Los tres docs
  del arnés (`architecture.md`, `conventions.md`, `verification.md`) sí se
  limpian con `--reset`; `docs/prd/` no.

En instalaciones existentes aparecen al re-correr el instalador, sin tocar nada
de lo que ya tengas.

## Marker `.harness_layout` des-versionado + gate de espejo de roles

Desde esta versión (feature #7, decisión del usuario 2026-07-28):

- **`.harness_layout` ya no está versionado** en el repo fuente. Es estado
  **local** de instalación (no código): versionado con valor `subdir` hacía que
  todo clon naciera declarando "mi raíz es mi padre", y el checkout fuente
  resolvía su raíz a `$HOME` (falsos fallos en `harness_check.sh` y basura en
  `$HOME/docs`). El instalador lo escribe en cada instalación, como siempre.
- **Migración en instalaciones subdir existentes**: al hacer `git pull` en tu
  `harness_process`, Git elimina el `.harness_layout` local (dejó de estar
  tracked). Entre el `pull` y el siguiente setup el arnés opera como layout
  `root` (efectos acotados al propio clon; no toca tu proyecto ni `$HOME`).
  **Re-corre el instalador** (`./setup_harness.sh` / `.\setup_harness.ps1`) —el
  flujo canónico de siempre tras un pull— y el marker se regenera con el layout
  correcto.
- **Guardrail de checkout fuente**: aunque el marker diga `subdir`, si el
  directorio tiene señales de fuente (`templates/harness_cli` + `rust/`) y el
  padre no tiene huella de instalación (`docs/constitution.md`, `CLAUDE.md`,
  `AGENTS.md`, `.claude/settings.json`) o el padre es `$HOME` (sin
  `HARNESS_ALLOW_HOME_SURFACE=1`), los scripts y el binario resuelven la raíz al
  **propio checkout** con un aviso informativo `[i]`. Las instalaciones subdir
  legítimas (padre con huella) no cambian, y `HARNESS_REPO_ROOT` /
  `CLAUDE_PROJECT_DIR` (y variables de agente) siguen mandando sobre cualquier
  detección.
- **Gate de espejo de roles en `harness_check.sh`**: `roles/*.md` es la fuente
  única; el check ahora compara el cuerpo embebido de `.claude/agents/*.md`
  (también leídos por Grok), `.gemini/agents/*.md` y `.codex/agents/*.toml`
  contra `roles/<rol>.md`, y `roles/*.md` contra `templates/roles/*.md` (módulo
  `__HREL__`). Un espejo desincronizado **bloquea** como los demás checks
  (`HARNESS_CHECK_MODE=warn|off` degradan igual que siempre). El check solo
  **reporta**: el remedio es re-correr el instalador (o propagar el cambio a
  `roles/` si lo que editaste fue el espejo).

## Kimi Code CLI: hooks globales (única excepción de escritura en `$HOME`)

Desde esta versión (feature #8) Kimi Code CLI (v0.29.x) es backend de primera
clase: lee el `AGENTS.md` generado (verificado empíricamente contra v0.29.2:
lo inyecta al system prompt), recibe los roles como subagentes nativos en
`.kimi-code/agents/*.md` (frontmatter `name`/`description`/`tools` con
allowlist por rol: leader/reviewer `Read, Grep, Glob, Bash`; implementer
además `Edit, Write`), y tiene launcher `bin/harness-kimi`. Todo eso vive en
el proyecto, como con los demás backends.

**La excepción**: Kimi **no soporta hooks por proyecto** (verificado: un
`[[hooks]]` en el config del proyecto no se ejecuta). El único lugar donde
existen es el config **global** `${KIMI_CODE_HOME:-~/.kimi-code}/config.toml`.
El usuario decidió (2026-07-28) aceptar esa única excepción a la regla de no
escribir fuera del proyecto, con estas salvaguardas:

- **Detección previa**: el bloque solo se escribe si Kimi está presente
  (`kimi` en PATH o `KIMI_CODE_HOME/bin/kimi` ejecutable). `--no-kimi`
  (`-NoKimi` en PowerShell) lo excluye explícitamente.
- **Backup previo** del `config.toml` en `bkp/` (mecanismo `HARNESS_BKP_DIR`
  de siempre) antes de cualquier escritura.
- **Bloque delimitado** por los marcadores `# >>> harness-process hooks >>>` y
  `# <<< harness-process hooks <<<`, con **reemplazo idempotente** solo entre
  marcadores: re-instalar no duplica nada y los hooks/config propios del
  usuario quedan intactos byte a byte.
- **Validación best-effort**: tras escribir se corre `kimi doctor`; si reporta
  config inválido se restaura el estado previo (o se retira el archivo recién
  creado) con un aviso accionable, sin cambiar el exit code del setup.
- **Guard por proyecto**: cada `command` del bloque solo actúa si
  `$PWD/bin/harness-hook` existe (el hook corre con cwd = proyecto,
  verificado); en proyectos sin arnés es un no-op silencioso que cuesta un
  `stat`.

Eventos registrados: `SessionStart` (timeout 120), `PostToolUse` con matcher
`Edit|Write` (timeout 30) y `Stop` (timeout 120), despachando a
`bin/harness-hook plain <evento>`. `SessionEnd` NO se registra a propósito
(el runtime lo trataría como otro `stop` y el check correría dos veces por
turno). El `Stop` con exit 2 + stderr bloquea el cierre del turno en Kimi
(verificado), igual que en Claude Code.

**`--reset` NO remueve el bloque global** (decisión del usuario 2026-07-28):
el bloque es compartido por TODOS los proyectos con arnés de la máquina y
`--reset` es por-proyecto — removerlo desde el proyecto A dejaría sin hooks al
proyecto B. Es inofensivo en máquinas que abandonan el arnés (el guard sale 0
en silencio). Para quitarlo a mano:

1. Abre `${KIMI_CODE_HOME:-~/.kimi-code}/config.toml`.
2. Borra desde la línea `# >>> harness-process hooks >>>` hasta la línea
   `# <<< harness-process hooks <<<` (inclusive).
3. Si algo sale mal, hay backups `config.toml.bak.*` bajo `bkp/` del arnés
   desde la primera instalación.

Instalaciones existentes: re-corre el instalador (`./setup_harness.sh` /
`.\setup_harness.ps1`) y aparecen los subagentes, el launcher y —si Kimi está
instalado— el bloque global. Nota de acoplamiento: los nombres de eventos y
tools están verificados contra v0.29.2; si una versión futura de Kimi los
renombra, el fallo es benigno (el hook no matchea) y el gate de espejo de
roles no depende de Kimi.

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
