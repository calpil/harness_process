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
- [ ] **Documentos al dia**: el PRD de origen, sus padres, `docs/prd/SDD-master.md`
      y `docs/architecture.md` reflejan lo implementado. Se siembra la propuesta
      con `sh harness_cli prd propose --feature <id>`, se contesta cada bloque
      (`cambio` / `ya-esta <archivo>:<L1>-<L2>` / `no-aplica <razon>`), se le
      MUESTRA al usuario y solo con su SI: `prd apply --feature <id> --yes`. Con
      `require_docs_al_dia` activa el cierre lo exige. Son documentos del
      USUARIO: el arnes propone, nunca escribe por su cuenta.
- [ ] Repos afectados limpios o commiteados segun politica.
- [ ] **Aprendizaje declarado**: el cierre dice que se aprendio
      (`--leccion <clase>`) o por que no (`--leccion ninguna --leccion-motivo
      "..."`). Con la regla `require_leccion` activa el comando lo exige; sin
      ella sigue siendo criterio de cierre. `ninguna` tras una feature con
      correcciones del usuario o forks de diseno NO es honesto.
- [ ] Task y memorias en sync: cierre via
      `sh harness_cli close --feature <id> --status <estado>` (registra el hub
      y refresca graphify).
- [ ] `harness_check.sh` pasa o el bloqueo queda documentado.
