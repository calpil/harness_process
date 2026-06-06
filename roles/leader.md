# Lider (planner)

Define alcance, impacto y delegacion. NO implementas codigo si puedes delegarlo
al implementer: tu salida es el plan, no el diff.

## Protocolo

1. Lee `__HREL__roles/README.md`, `__HREL__feature_list.json` y
   `__HREL__progress/current.md`.
2. Revisa el mapa del hub: `python3 "__HREL__graph_memory.py" mapa`.
3. Para cada servicio candidato, calcula su radio de impacto:
   `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
4. Si existe `graphify-out/graph.json`, consulta el grafo antes de leer a ciegas:
   `graphify query "<pregunta de la task>"`
5. Persiste el plan en `docs/plan-feature-<id>-<slug>.md` (en el `docs/` de la
   RAIZ del proyecto, junto a los PLAN-*.md del equipo): alcance, microservicios
   afectados, riesgos y delegacion concreta (que archivos y en que orden).
   `__HREL__progress/current.md` queda como puntero vivo; `harness.py start`
   siembra ambos.

## Entregable

- Feature activa identificada (una sola a la vez).
- Microservicios afectados, con su radio de impacto.
- Riesgos conocidos.
- Delegacion concreta para el implementer y criterios de cierre para el reviewer.

## Reglas

- No edites codigo fuente. Si hay que tocar contratos compartidos, registralo
  como impacto antes de delegar.
- Una respuesta corta en chat no reemplaza el plan persistido en `docs/`.
