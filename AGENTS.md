# Mapa de Agentes

Este arnes usa un mapa progresivo: lee solo lo necesario para la tarea actual.

## Orden de trabajo

1. Lider revisa `feature_list.json`, `progress/current.md`, hub e impacto, y
   completa el spec (`docs/spec-feature-<id>-<slug>.md`, `Estado: draft`) con
   los AC-n en Given/When/Then ANTES del plan; cada item de la Delegacion cita
   su AC-n.
2. Ritual de aprobacion: el agente le MUESTRA el spec al USUARIO (contenido en
   el chat + abierto en su editor), le PREGUNTA si lo aprueba y solo con su SI
   lo REGISTRA con `sh harness_cli approve-spec --yes` (escribe
   `Estado: approved`, sella quien/cuando y re-firma el spec). Sin `--yes` el
   comando se niega: ningun agente aprueba por su cuenta. Con la regla
   `require_spec_approved` activa, el gate (`check-spec`, `advance`,
   `close --status done`) bloquea hasta esa aprobacion.
3. Implementer verifica `check-spec` limpio, trabaja una unidad concreta y
   escribe evidencia por AC-n en `docs/impl-<feature>.md`.
4. Reviewer verifica spec aprobado y fresco, evidencia por AC, impacto, tests,
   checkpoints y estado Git.
5. El cierre requiere `harness_check.sh` limpio o decision explicita de bloqueo.
   El check incluye el gate de espejo de roles: los cuerpos embebidos de
   `.claude/agents/*.md`, `.gemini/agents/*.md` y `.codex/agents/*.toml` deben
   coincidir con `roles/*.md` (fuente unica), y `roles/*.md` con
   `templates/roles/*.md` modulo `__HREL__`; un espejo stale bloquea y el
   remedio es re-correr el instalador.

## Archivos principales

- `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `GROK.md`, `LLM.md`: superficies
  raiz para distintos agentes.
- `CHECKPOINTS.md`: criterios de cierre.
- `feature_list.json`: backlog ejecutable.
- `progress/current.md`: estado vivo de la tarea.
- `progress/history.md`: bitacora append-only.
- `docs/constitution.md`: principios no negociables (el spec y el plan los
  cumplen; el reviewer los verifica).
- `docs/spec-feature-<id>-<slug>.md`: spec de la feature con AC-n
  (Given/When/Then); se aprueba (draft -> approved) antes de implementar, via
  `harness_cli approve-spec --yes` tras el si explicito del usuario.
- `docs/prd/PRD-master.md` (RAIZ): planilla maestra de producto; sus hitos
  alimentan `feature_list.json`.
- `docs/prd/SDD-master.md` (RAIZ): planilla maestra de diseno tecnico del
  proyecto (distinta de `docs/architecture.md`, que mapea lo que ya existe).
- `docs/architecture.md` (RAIZ): mapa de arquitectura.
- `docs/conventions.md` (RAIZ): convenciones del equipo.
- `docs/verification.md` (RAIZ): comandos de validacion.
- `.claude/agents/leader.md`: rol lider.
- `.claude/agents/implementer.md`: rol implementador.
- `.claude/agents/reviewer.md`: rol revisor.

## Regla anti perdida de contexto

Todo hallazgo relevante se escribe en `progress/`. Una respuesta corta en chat
no reemplaza evidencia persistida.
