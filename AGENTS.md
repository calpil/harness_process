# Mapa de Agentes

Este arnes usa un mapa progresivo: lee solo lo necesario para la tarea actual.

## Orden de trabajo

1. Lider revisa `feature_list.json`, `progress/current.md`, hub e impacto, y
   completa el spec (`docs/spec-feature-<id>-<slug>.md`, `Estado: draft`) con
   los AC-n en Given/When/Then ANTES del plan; cada item de la Delegacion cita
   su AC-n.
2. El USUARIO aprueba el spec (`Estado: draft` -> `Estado: approved`); ningun
   agente auto-aprueba. Con la regla `require_spec_approved` activa, el gate
   (`check-spec`, `advance`, `close --status done`) bloquea hasta la aprobacion.
3. Implementer verifica `check-spec` limpio, trabaja una unidad concreta y
   escribe evidencia por AC-n en `docs/impl-<feature>.md`.
4. Reviewer verifica spec aprobado y fresco, evidencia por AC, impacto, tests,
   checkpoints y estado Git.
5. El cierre requiere `harness_check.sh` limpio o decision explicita de bloqueo.

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
  (Given/When/Then); se aprueba (draft -> approved) antes de implementar.
- `docs/architecture.md`: mapa de arquitectura.
- `docs/conventions.md`: convenciones del equipo.
- `docs/verification.md`: comandos de validacion.
- `.claude/agents/leader.md`: rol lider.
- `.claude/agents/implementer.md`: rol implementador.
- `.claude/agents/reviewer.md`: rol revisor.

## Regla anti perdida de contexto

Todo hallazgo relevante se escribe en `progress/`. Una respuesta corta en chat
no reemplaza evidencia persistida.
