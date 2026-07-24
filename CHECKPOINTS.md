# Checkpoints

Antes de cerrar una tarea:

- [ ] La feature activa en `feature_list.json` refleja el estado real.
- [ ] **Plan frescura verificada**: `sh harness_cli check-plan` pasa (sin "PLAN ACTUALIZADO POR OTRO LLM").
  Esto detecta ediciones concurrentes hechas por Claude/Gemini/Antigravity/Grok/etc.
- [ ] **Spec aprobado y fresco**: `sh harness_cli check-spec` pasa
  (`Estado: approved` y sin ediciones multi-LLM sin refirmar). Solo el usuario
  aprueba el spec: el agente se lo muestra, le pregunta y registra su SI con
  `harness_cli approve-spec --yes` (sello + re-firma). Ningun agente aprueba solo.
- [ ] **Sin observaciones pendientes**: cada observacion del plan tiene la
  decision del usuario registrada (`advance --nota "Decision usuario: ..."`);
  no se implemento nada con decisiones abiertas.
- [ ] El plan vive en `docs/plan-feature-<feature>.md` (raiz) y refleja lo hecho.
- [ ] `progress/current.md` apunta al plan y contiene evidencia al dia.
- [ ] Se ejecuto impacto para los microservicios modificados:
      `sh harness_cli graph impacto --microservicio <proyecto>/<servicio>`
- [ ] Si existe `graphify-out/graph.json`, se consulto `graphify query`.
- [ ] Tests relevantes ejecutados por cada microservicio afectado.
- [ ] Frontends validados con `validate_ui.sh <url>` cuando aplique.
- [ ] `docs/review-<feature>.md` contiene veredicto del reviewer.
- [ ] **Evidencia por AC-n**: `docs/impl-<feature>.md` y `docs/review-<feature>.md`
  mapean cada AC-n del spec a su evidencia/test; el veredicto lista AC-1..AC-n.
- [ ] Repos afectados limpios o commiteados segun politica.
- [ ] Task y memorias en sync: cierre via
      `sh harness_cli close --feature <id> --status <estado>` (registra el hub
      y refresca graphify).
- [ ] `harness_check.sh` pasa o el bloqueo queda documentado.
