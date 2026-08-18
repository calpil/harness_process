# Estado archivado - Feature #17: lecciones_memoria_procedural
Cerrada: 2026-08-16T22:14:34Z - status=done - Memoria procedural del arnes: docs/lecciones/<clase>.md ordenado por clase de trabajo, comando leccion list|show|nueva|usar, nombres de clase sin escape hatch, gate opcional require_leccion en el cierre, reglas de captura portadas a la guia y a los tres roles, y gate de integridad en harness_check.sh. 20 AC cubiertos (AC-20 parcial: smoke ps1 sin correr, sin PowerShell). Cero dependencias nuevas y funciona con el hub caido.

---

# Feature #17: lecciones_memoria_procedural

Estado: in_progress
Plan: docs/plan-feature-17-lecciones-memoria-procedural.md
Spec: docs/spec-feature-17-lecciones-memoria-procedural.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-16T20:03:11Z Plan de la #17 escrito por el lider: alcance, impacto (hub inalcanzable, documentado), consulta a graphify, D1-D10 citando cada AC-n, criterios de cierre y riesgos. Las 5 observaciones quedaron decididas por Alan en el acto de aprobacion del spec, asi que no hay ninguna abierta.
- 2026-08-16T22:14:21Z D1-D10 implementados: guia de lecciones (orden de preferencia + lista anti-veneno), entrada unica en HARNESS_DOCS de ambos instaladores, modulo lecciones.rs (validacion de nombre de clase sin escape hatch, frontmatter con round-trip, telemetria, scan y gate), comando leccion list|show|nueva|usar, gate opcional require_leccion en close, reglas en los tres roles + espejos, tres almacenes en architecture.md, README/UPDATING/superficies, bloque de integridad en harness_check.sh y 26 tests nuevos. El pase de reviewer encontro y corrigio el bug de CRLF en render(). Evidencia por AC en docs/impl-17.md, veredicto en docs/review-17.md.
