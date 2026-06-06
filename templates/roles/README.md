# Mapa de Agentes

Arnes multi-LLM con tres roles. Lee solo lo necesario para la tarea actual
(mapa progresivo): primero el plan, luego el rol, luego el codigo.

## Flujo

```
  __HREL__feature_list.json
            |
            v
   +-----------+    plan en        +--------------+   evidencia en   +------------+
   |  LIDER    |--> docs/plan-* --> | IMPLEMENTER  |--> docs/impl-* -->| REVIEWER   |
   | (planner) |                    | (1 unidad)   |                  | (verifica) |
   +-----------+                    +--------------+                  +------------+
        ^                                                                   |
        |                       changes_requested                           |
        +-------------------------------------------------------------------+
                                       |
                             approved + checkpoints OK
                                       v
                        harness_check.sh limpio  ->  cierre
```

## Roles

| Rol         | Cuando usarlo                             | Tools (Claude)          | Escribe en                |
|-------------|-------------------------------------------|-------------------------|---------------------------|
| leader      | Al iniciar: alcance, impacto, plan        | Read, Grep, Glob, Bash  | docs/plan-feature-<f>.md  |
| implementer | Escribir o modificar una unidad de codigo | Read, Edit, Write, Bash | docs/impl-<f>.md          |
| reviewer    | Antes de cerrar: tests, impacto, gates    | Read, Grep, Glob, Bash  | docs/review-<f>.md        |

Definicion completa: `__HREL__roles/leader.md`, `__HREL__roles/implementer.md`,
`__HREL__roles/reviewer.md`.

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

Sin archivo de definicion soportado (aplican `__HREL__roles/*.md` como fases
secuenciales lider -> implementer -> reviewer en una sola sesion):

- **Antigravity**: crea sus subagentes dinamicamente en runtime; lee tambien
  `AGENTS.md` / `.agents/rules/`.
- **Cualquier otro CLI** sin subagentes nativos.

Claude Code no permite subagentes anidados: delega el hilo principal, no el
subagente `leader`.

## Modelos, effort y tools por rol (tunable)

- **Claude** (`.claude/agents/*.md`): `leader` y `reviewer` con
  `model: claude-opus-4-8` (Opus 4.8); `implementer` con
  `model: claude-sonnet-4-6` (Sonnet 4.6); los tres con `effort: max`. `model:`
  acepta ID fijo o alias auto-ultima-version (`opus`, `sonnet`, `haiku`,
  `inherit`); `effort:` es `low|medium|high|xhigh|max` (`xhigh` solo Opus 4.7+).
  El `effort:` del frontmatter NO sobreescribe la env var
  `CLAUDE_CODE_EFFORT_LEVEL`.
- **Codex** (`.codex/agents/*.toml`): `model` se hereda de la sesion;
  `model_reasoning_effort = high` (tope de Codex). Read-only via
  `sandbox_mode = read-only`; el implementer usa `workspace-write`.
- **Gemini** (`.gemini/agents/*.md`): `model` y `tools` se heredan de la sesion
  (omitidos para no fijar IDs/nombres que cambian por version). Agregalos por
  rol cuando confirmes los nombres de tools/model de tu version instalada.

## Regla anti perdida de contexto

Los documentos durables se escriben en `docs/` de la raiz; `progress/` guarda
solo el estado vivo. Una respuesta corta en chat no reemplaza evidencia
persistida.
