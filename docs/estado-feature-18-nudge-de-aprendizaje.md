# Estado archivado - Feature #18: nudge_de_aprendizaje
Cerrada: 2026-08-16T23:25:03Z - status=done - El arnes empuja solo: recordatorio cada N acciones (default 25, apagable por rules) y CONTRATO completo al cerrar sin declarar, leido de la guia y no duplicado en el binario (con degradacion a puntero). Ademas el aviso de 'sin feature activa' pasa a backoff 600s->3600s con reset al piso. 21 AC cubiertos, sin parciales; todo a stderr y con exit 0 invariante; un proyecto sin docs/lecciones/ no ve nada nuevo.

---

# Feature #18: nudge_de_aprendizaje

Estado: in_progress
Plan: docs/plan-feature-18-nudge-de-aprendizaje.md
Spec: docs/spec-feature-18-nudge-de-aprendizaje.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-16T23:08:41Z Plan de la #18 escrito: D1-D8 citando cada AC, impacto (hub caido, documentado), consulta al grafo y riesgos. Las 7 observaciones quedaron decididas por Alan en el acto de aprobacion. Fuera de alcance pero aplicado en la sesion: require_leccion activa en este repo y PRD-master con sus 5 hitos (#23-#27).
- 2026-08-16T23:24:47Z D1-D8 implementados: backoff con nivel en .last_nudge (600->3600, reset al piso), contador por feature en .nudge_lecciones (default 25, apagable), lector del contrato que LEE la guia con degradacion a puntero, enganche en close a stderr sin tocar stdout ni exit code, docs/roles/superficies y 19 tests nuevos (incluido el anti-drift contra la guia real). El pase de reviewer corrigio el mensaje: decia 'escrituras' y contaba tool-calls.
