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
- `docs/architecture.md`, `docs/conventions.md`, `docs/verification.md`,
  `docs/kimi-cli-uso-eficiente.md` y `docs/prd/COMO-ESCRIBIR-UN-PRD.md` en el
  `docs/` de la **raíz** del proyecto (mismo criterio: solo si faltan)
- `docs/prd/PRD-master.md` y `docs/prd/SDD-master.md` (planillas maestras del
  proyecto; solo si faltan, y `--reset` no las borra)
- `.kimiignore` y `.kimirules` en la **raíz** del proyecto (exclusiones de
  contexto y reglas fijas para agentes; solo si faltan, y ni `--force` ni
  `--reset` los toca)
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

Además cambió el criterio de refresco: estos docs del arnés ahora se siembran
**solo si faltan** (igual que la constitution), porque comparten carpeta con la
documentación del equipo y un `docs/conventions.md` propio no debe perderse en un
reinstall. Para refrescar una plantilla: borra el archivo y reinstala, o usa
`--force` (que por contrato sobrescribe **sin** backup).

`--reset` sigue limpiando solo lo generado —los docs del arnés, en su ubicación
nueva y en la vieja— y conserva la constitution y los artefactos de feature
(`spec-*`, `plan-*`, `impl-*`, `review-*`).

## Planillas maestras PRD y SDD (`docs/prd/`)

Desde esta versión el instalador siembra dos planillas para proyectos que
arrancan de cero, en el `docs/prd/` de la **raíz del proyecto**:

- `docs/prd/COMO-ESCRIBIR-UN-PRD.md` — **el método** para escribir un PRD: qué
  contiene y qué nunca contiene, la historia (antes/después) como corazón del
  documento, el tamaño que decide el cambio (1 página un ajuste, 3-8 una
  funcionalidad, 10+ una grande, PRDs anidados para un producto nuevo), la
  anatomía sección por sección con un ejemplo, y la regla dura: el PRD fija la
  estructura en pseudo-código y explicaciones, **nunca** en código final.
  A diferencia de las otras dos, **es plantilla del arnés**: se refresca
  reinstalando (o con `--force`) y entra en los reset targets.
- `docs/prd/PRD-master.md` — qué se construye y por qué: resumen hoy → después,
  **la historia** (antes/después, con nombre y momento), objetivos y no-objetivos
  numerados (`O1`, `NO1`), usuarios y jobs-to-be-done, métricas de éxito, el flujo
  dibujado dos veces (hoy → cómo va a funcionar), **los datos** (disparador,
  interruptor, candado) y el **pseudo-código del acuerdo** a nivel producto,
  restricciones, tabla **Hitos → features** (cada fila se carga al backlog con
  `harness_cli add`), riesgos y decisiones abiertas.
- `docs/prd/SDD-master.md` — cómo se construye, a nivel proyecto: arquitectura
  objetivo, stack, contratos entre componentes, decisiones técnicas tipo ADR,
  datos, no funcionales, estrategia de verificación, riesgos y decisiones
  abiertas. Es distinto de `docs/architecture.md`, que mapea lo que **ya** existe.

Garantías (mismo criterio que `docs/constitution.md`):

- Se siembran **solo si faltan**. Un reinstall nunca las pisa, y **ni `--force`**
  las sobrescribe: lo que hay escrito ahí es tu proyecto, no una plantilla
  refrescable.
- **`--reset` no las borra.** No son superficie generada del arnés. Los docs
  del arnés (`architecture.md`, `conventions.md`, `verification.md`,
  `kimi-cli-uso-eficiente.md` y la guía `prd/COMO-ESCRIBIR-UN-PRD.md`) sí se
  limpian con `--reset`; el PRD y el SDD del proyecto no.

El mismo método baja un nivel: cada `docs/spec-feature-<id>-<slug>.md` que genera
`harness_cli start` es **el PRD de ese cambio** y nace con las secciones
`La historia (antes -> despues)`, `Hoy -> Como va a funcionar`,
`Los datos que se tocan` y `Pseudo-codigo (el acuerdo)`, además de los recorridos
priorizados y los AC-n. Los specs ya existentes no se reescriben: la plantilla
nueva rige para los que se generen de aquí en adelante.

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
  tracked). **Ya no hace falta re-correr el instalador para que la raíz vuelva a
  ser tu proyecto**: desde la feature #10 el layout `subdir` se infiere de la
  huella del padre (ver la sección siguiente). Re-correr el instalador
  (`./setup_harness.sh` / `.\setup_harness.ps1`) sigue siendo el flujo canónico
  tras un pull y es lo que **regenera el marker** —y con él, el aviso `[i]`
  desaparece—, pero ya no es un requisito para no perder la raíz.
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

## Layout `subdir` inferido cuando falta `.harness_layout`

Desde esta versión (feature #10, decisión del usuario 2026-07-29). Corrige el
efecto colateral de des-versionar el marker: el commit que lo sacó de git graba
`D .harness_layout`, así que **cualquier instalación que hizo `git pull` se
quedó sin marker** y pasaba a tratar `harness_process/` como raíz, en silencio
(specs, planes y veredictos escritos dentro del arnés en vez de en el `docs/` de
tu proyecto).

Cómo se resuelve ahora la raíz (misma regla en los cuatro scripts —
`harness_check.sh`, `harness_status.sh`, `init.sh`, `commit_guard.sh`— y en el
binario Rust), en orden:

1. `HARNESS_REPO_ROOT` o una variable de agente (`CLAUDE_PROJECT_DIR`,
   `CODEX_PROJECT_DIR`, ...) siguen mandando sobre todo lo demás, sin avisos.
2. **Marker `subdir`**: la raíz es el padre, con el guardrail de checkout fuente
   de la feature #7 intacto.
3. **Marker ausente**: si el padre tiene huella de instalación
   (`docs/constitution.md`, `CLAUDE.md`, `AGENTS.md` o `.claude/settings.json`) y
   no es `$HOME` (sin `HARNESS_ALLOW_HOME_SURFACE=1`), se **infiere layout
   `subdir`** y la raíz es el padre, con un aviso informativo:

   ```
   [i] .harness_layout ausente: layout subdir inferido por la huella de
       instalacion del padre: REPO_ROOT=<tu proyecto>. Re-corre el instalador
       (setup_harness.sh / setup_harness.ps1) para regenerar el marker.
   ```

   No es un fallo: los exit codes no cambian. Sin huella en el padre no se
   infiere nada (la raíz es el directorio del arnés, como antes).
4. **Marker con cualquier otro valor** (`root`): se respeta al pie de la letra.
   La inferencia aplica **solo** cuando el archivo NO existe, así que una
   instalación en layout root nunca cambia de raíz.

Qué tienes que hacer: **nada**. Tu instalación se repara sola al actualizar. Si
quieres que el aviso `[i]` desaparezca, re-corre el instalador: es lo único que
escribe el marker (los scripts son de solo lectura y nunca lo regeneran).

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
