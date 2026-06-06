# Checkpoints

Antes de cerrar una tarea:

- [ ] La feature activa en `feature_list.json` refleja el estado real.
- [ ] El plan vive en `docs/plan-feature-<feature>.md` (raiz) y refleja lo hecho.
- [ ] `progress/current.md` apunta al plan y contiene evidencia al dia.
- [ ] Se ejecuto impacto para los microservicios modificados:
      `python3 graph_memory.py impacto --microservicio <proyecto>/<servicio>`
- [ ] Si existe `graphify-out/graph.json`, se consulto `graphify query`.
- [ ] Tests relevantes ejecutados por cada microservicio afectado.
- [ ] Frontends validados con `validate_ui.sh <url>` cuando aplique.
- [ ] `docs/review-<feature>.md` contiene veredicto del reviewer.
- [ ] Repos afectados limpios o commiteados segun politica.
- [ ] Task y memorias en sync: cierre via
      `python3 harness.py close --feature <id> --status <estado>` (registra el hub
      y refresca graphify).
- [ ] `harness_check.sh` pasa o el bloqueo queda documentado.
