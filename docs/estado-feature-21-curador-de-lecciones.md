# Estado archivado - Feature #21: curador_de_lecciones
Cerrada: 2026-08-17T04:22:41Z - status=done - Curador de lecciones: ciclo activa->stale->archivada determinista y sin modelo (30/90 dias configurables), pin que congela, y cuatro garantias probadas pudiendo fallar: nunca borra (archivar es mover), nada se mueve sin --aplicar, toda pasada mutante respalda y el rollback tambien es reversible, y archivar no la saca de buscar. La consolidacion con LLM salio a la #28 por no ser verificable aqui.

---

# Feature #21: curador_de_lecciones

Estado: in_progress
Plan: docs/plan-feature-21-curador-de-lecciones.md
Spec: docs/spec-feature-21-curador-de-lecciones.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-17T04:09:52Z Plan de la #21 escrito: D1-D10 citando cada AC, criterios de cierre escritos para poder fallar (modo informe con stat antes/despues, ciclo completo con fechas falsas + rollback con diff vacio, archivada sigue en buscar por debajo de una activa, pin sobrevive a 200 dias). Primera feature que uso 'buscar' (#20) en su propio diseno en vez de graphify query. Las 5 observaciones decididas; 3 corrigen el backlog.
- 2026-08-17T04:22:34Z D1-D10 implementados: ciclo de vida determinista en lecciones.rs (Transicion como enum, umbrales configurables, piso de gracia, pin), curador.rs con planificar/aplicar separados (lo que hace estructural la promesa de 'no toca nada'), backup + rollback reversible, reporte por pasada, subcomandos lecciones *, integracion con buscar (archivada visible pero por debajo) y 29 tests nuevos. Los 4 criterios de cierre corridos end-to-end con fechas falsas.
