# Mapa de Agentes

Este arnes usa un mapa progresivo: lee solo lo necesario para la tarea actual.

## Orden de trabajo

1. Lider revisa `feature_list.json`, `progress/current.md`, hub e impacto.
2. Implementer trabaja en una unidad concreta y escribe evidencia en `progress/`.
3. Reviewer verifica impacto, tests, checkpoints y estado Git.
4. El cierre requiere `harness_check.sh` limpio o decision explicita de bloqueo.

## Archivos principales

- `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `GROK.md`, `LLM.md`: superficies
  raiz para distintos agentes.
- `CHECKPOINTS.md`: criterios de cierre.
- `feature_list.json`: backlog ejecutable.
- `progress/current.md`: estado vivo de la tarea.
- `progress/history.md`: bitacora append-only.
- `docs/architecture.md`: mapa de arquitectura.
- `docs/conventions.md`: convenciones del equipo.
- `docs/verification.md`: comandos de validacion.
- `.claude/agents/leader.md`: rol lider.
- `.claude/agents/implementer.md`: rol implementador.
- `.claude/agents/reviewer.md`: rol revisor.

## Regla anti perdida de contexto

Todo hallazgo relevante se escribe en `progress/`. Una respuesta corta en chat
no reemplaza evidencia persistida.
