# Estado archivado - Feature #22: mapa_de_aprendizaje
Cerrada: 2026-08-17T04:54:23Z - status=done - Mapa de aprendizaje: los tres almacenes juntos con sus enlaces (declarada / origen / cita / relacionada) y sus HUECOS, cada uno con el comando que lo corrige. Solo lectura por decision: sin delete ni edit, porque serian una segunda puerta al 'nunca borra' del curador y al gate del --yes del perfil. Sin hub, sin modelo, sin escrituras. Ultimo hito del PRD de aprendizaje.

---

# Feature #22: mapa_de_aprendizaje

Estado: in_progress
Plan: docs/plan-feature-22-mapa-de-aprendizaje.md
Spec: docs/spec-feature-22-mapa-de-aprendizaje.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-17T04:39:10Z Plan de la #22 escrito: D1-D8 citando cada AC, disenado con buscar (#20) y con inspeccion de los datos reales, que mostraron que 'leccion declarada' y 'leccion de origen' NO son lo mismo (la #17 parió dos). Criterios de cierre escritos para poder fallar, incluido verificar a mano que cada hueco reportado sobre el repo real existe de verdad. Las 5 observaciones decididas; 2 reducen el alcance respecto del backlog.
- 2026-08-17T04:54:16Z D1-D8 implementados: journey.rs (Tipo/Clase/Motivo como enums, construir() solo lee, Mapa::hijos con dedup y anclaje), comando con render cronologico y --json, docs/superficies y 26 tests. Los criterios de cierre encontraron TRES bugs que los fixtures no veian: leccion declarada duplicada, perfil colgando de todas las features citadas, y 16 huecos no corregibles (features anteriores a la maquinaria) que bajaron a 0 tras acotar por era y comparar timestamps completos.
