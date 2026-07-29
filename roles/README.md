# Mapa de Agentes

Arnes multi-LLM con tres roles. Lee solo lo necesario para la tarea actual
(mapa progresivo): primero el plan, luego el rol, luego el codigo.

## Flujo

```
  harness_process/feature_list.json
            |
            v
   +-----------+  spec+plan   +-----------+  aprueba  +--------------+  evidencia  +------------+
   |  LIDER    |-> docs/  --->|  USUARIO  |-> draft ->| IMPLEMENTER  |-> docs/  -->| REVIEWER   |
   | (planner) |  spec-* +    | (aprueba) |  approved | (1 unidad)   |  impl-*     | (verifica) |
   | cita AC-n |  plan-*      |  el spec  |           |              |             |            |
   +-----------+              +-----------+           +--------------+             +------------+
        ^                                                                                       |
        |                                   changes_requested                                   |
        +---------------------------------------------------------------------------------------+
                                          |
                              approved + checkpoints OK
                                          v
                         harness_check.sh limpio  ->  cierre
```

Entre el LIDER y el IMPLEMENTER se ejecuta el **ritual de aprobacion**: el agente
lee el spec, se lo MUESTRA al usuario (chat + editor), le PREGUNTA si lo aprueba
y solo con su SI lo REGISTRA con `sh "harness_process/harness_cli" approve-spec --yes`
(que sella quien/cuando y re-firma el spec para que `check-spec` no lo reporte
como edicion de otro LLM). La decision es del usuario: ningun agente aprueba por
su cuenta, y el gate `check-spec` bloquea hasta esa aprobacion.

## Roles

| Rol         | Cuando usarlo                             | Tools (Claude)          | Escribe en                |
|-------------|-------------------------------------------|-------------------------|---------------------------|
| leader      | Al iniciar: spec (AC-n) + plan            | Read, Grep, Glob, Bash  | docs/spec-* + docs/plan-* |
| implementer | Escribir o modificar una unidad de codigo | Read, Edit, Write, Bash | docs/impl-<f>.md          |
| reviewer    | Antes de cerrar: tests, impacto, gates    | Read, Grep, Glob, Bash  | docs/review-<f>.md        |

Definicion completa: `harness_process/roles/leader.md`, `harness_process/roles/implementer.md`,
`harness_process/roles/reviewer.md`.

## Como se orquesta por herramienta

Mismos tres roles; cada CLI los recibe en su formato nativo (auto-registrados):

- **Claude Code**: `.claude/agents/*.md` (frontmatter `name`/`description`/
  `tools`/`model`/`effort`; cuerpo = system prompt). El hilo principal delega.
- **Codex CLI**: `.codex/agents/*.toml` (`name`, `description`,
  `developer_instructions`, `sandbox_mode`, `model_reasoning_effort`).
  Delegacion explicita (`/agent` o pidiendolo). No hay allowlist de tools: la
  capacidad se acota con `sandbox_mode`.
- **Gemini CLI**: `.gemini/agents/*.md` (frontmatter + cuerpo). Invocar con
  `@<rol>`; auto-delega segun `description`.
- **Grok Build (xAI)**: sin formato propio, pero LEE `.claude/agents/*.md` por
  compatibilidad con Claude Code (sin archivos extra). Puede ignorar un `model:`
  de Claude y caer al modelo por defecto de Grok.
- **Kimi Code CLI**: `.kimi-code/agents/*.md` (frontmatter `name`/`description`/
  `tools`; cuerpo = system prompt, mismo formato que Claude; verificado en
  v0.29.2). Al seleccionar un perfil su cuerpo REEMPLAZA el system prompt (cada
  rol se basta solo). Delegacion en sesion interactiva via tools
  `Agent`/`AgentSwarm`; en modo `-p`, `--agent <rol>` requiere el engine v2
  (`KIMI_CODE_EXPERIMENTAL_FLAG=1`). Lee `AGENTS.md` nativamente; sus hooks son
  SOLO globales (`KIMI_CODE_HOME/config.toml`, default `~/.kimi-code/`, bloque
  delimitado del arnes con guard por proyecto).

Sin archivo de definicion soportado (aplican `harness_process/roles/*.md` como fases
secuenciales lider -> implementer -> reviewer en una sola sesion):

- **Antigravity**: crea sus subagentes dinamicamente en runtime; lee tambien
  `AGENTS.md` / `.agents/rules/`.
- **Cualquier otro CLI** sin subagentes nativos.

Claude Code no permite subagentes anidados: delega el hilo principal, no el
subagente `leader`.

## Modelos, effort y tools por rol (tunable)

- **Claude** (`.claude/agents/*.md`): los tres roles (`leader`, `implementer`
  y `reviewer`) con `model: claude-fable-5` (Fable 5) y `effort: max`. `model:`
  acepta ID fijo o alias auto-ultima-version (`fable`, `opus`, `sonnet`,
  `haiku`, `inherit`); `effort:` es `low|medium|high|xhigh|max` (`xhigh` solo
  Opus 4.7+). El `effort:` del frontmatter NO sobreescribe la env var
  `CLAUDE_CODE_EFFORT_LEVEL`.
- **Codex** (`.codex/agents/*.toml`): `model` se hereda de la sesion;
  `model_reasoning_effort = high` (tope de Codex). El formato NO admite
  allowlist de herramientas (los subagentes usan las del chat padre), asi que
  la unica palanca es `sandbox_mode`, y los TRES roles usan `workspace-write`:
  leader y reviewer tienen que escribir sus entregables en `docs/` (spec, plan
  y veredicto), cosa que `read-only` impide (feature #9).
- **Gemini** (`.gemini/agents/*.md`): `model` y `tools` se heredan de la sesion
  (omitidos para no fijar IDs/nombres que cambian por version). Agregalos por
  rol cuando confirmes los nombres de tools/model de tu version instalada.
- **Kimi** (`.kimi-code/agents/*.md`): `model` se hereda de la sesion; `tools`
  con allowlist por rol (decision usuario 2026-07-28; nombres case-sensitive
  verificados en v0.29.2): leader/reviewer `Read, Grep, Glob, Bash`,
  implementer ademas `Edit, Write`.

**Sobre "solo lectura":** en Claude y en Kimi, leader y reviewer NO tienen
`Edit` ni `Write`, pero SI tienen `Bash`, con el que se escribe un archivo
igual — y lo necesitan, porque su entregable ES un archivo en `docs/`. En
Codex la palanca equivalente es `workspace-write`. En los tres backends la
disciplina de rol la sostiene el PROMPT de `roles/*.md` ("No edites codigo
fuente"), no la configuracion: ningun backend impide fisicamente que un rol
read-only toque codigo si decide ignorar su prompt.

## Regla anti perdida de contexto

Los documentos durables se escriben en `docs/` de la raiz; `progress/` guarda
solo el estado vivo. Una respuesta corta en chat no reemplaza evidencia
persistida.
