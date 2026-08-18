# Estado archivado - Feature #20: buscar_en_el_historial
Cerrada: 2026-08-17T03:54:24Z - status=done - buscar: hace preguntable la memoria del arnes. Recorre docs/**/*.md + history.md y ordena de lo mas curado (lecciones, perfil) a lo mas crudo (bitacora), con encabezados, frases contiguas y frescura como desempate. Sin indice (~10 ms medidos sobre 1,1 MB), sin LLM, sin hub, sin dependencias nuevas y de solo lectura. 19 AC cubiertos; el criterio de la consulta real fallo primero y corrigio dos bugs de ranking.

---

# Feature #20: buscar_en_el_historial

Estado: in_progress
Plan: docs/plan-feature-20-buscar-en-el-historial.md
Spec: docs/spec-feature-20-buscar-en-el-historial.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-17T03:45:54Z Plan de la #20 escrito: D1-D9 citando cada AC, impacto (el hub es irrelevante por diseno tras OBS-1), medicion real del corpus (113 archivos, 28.391 lineas, 1,1 MB) que justifica no tener indice, y la decision explicita de NO compartir codigo con perfil::recolectar. Las 5 observaciones quedaron decididas por Alan.
- 2026-08-17T03:54:17Z D1-D9 implementados: modulo buscar.rs (Fuente como enum ordenado por relevancia, score puro y testeable, corpus que excluye bkp), comando buscar con --json y --todos, docs/superficies/roles y 30 tests nuevos. El criterio de cierre de la consulta real FALLO en la primera corrida y destapo dos bugs de clasificacion (la guia de lecciones cobraba peso de conocimiento curado; el ADR pesaba como doc generico); ambos corregidos y con test.
