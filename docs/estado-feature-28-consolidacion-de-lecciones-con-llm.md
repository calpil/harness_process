# Estado archivado - Feature #28: consolidacion_de_lecciones_con_llm
Cerrada: 2026-08-18T22:18:57Z - status=done - Consolidacion de lecciones con LLM: el modelo ve solo nombre, descripcion y triggers (nunca el cuerpo), no puede escribir (detectar no recibe HarnessPaths) y el prompt va por argv, no por shell. La fusion la pide una persona con argv. Verificada de punta a punta con dos backends reales, y aplicada al corpus real: la biblioteca paso de 9 a 8 lecciones sin perder un solo pitfall.

---

# Feature #28: consolidacion_de_lecciones_con_llm

Estado: in_progress
Plan: docs/plan-feature-28-consolidacion-de-lecciones-con-llm.md
Spec: docs/spec-feature-28-consolidacion-de-lecciones-con-llm.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-18T14:17:17Z Re-sincronizado: se reparo una corrupcion de 2 bytes al inicio de la linea 1 del spec de la #28 ('ne# Spec' -> '# Spec'). No cambio ningun AC ni el sello de aprobacion de Alan; el contenido quedo intacto (27 AC, 9 secciones) y se verifico antes de re-firmar.
- 2026-08-18T21:28:46Z Feature #28 implementada: lecciones consolidar detecta solapamientos con un LLM (solo nombre, descripcion y triggers; NUNCA el cuerpo) e informa; --aplicar toma la fusion de argv y archiva con backup. Corrida real contra el corpus: el modelo encontro el par que el analisis lexico habia identificado (0.85) y propuso un segundo a 0.60 que el Jaccard daba en 0.048. Fusion real aplicada con el si de Alan: la biblioteca paso de 9 a 8 lecciones, el cuerpo archivado quedo byte a byte identico y install_asset sigue encontrando el paraguas.
