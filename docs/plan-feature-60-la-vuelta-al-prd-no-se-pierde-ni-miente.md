# Plan - Feature #60: la_vuelta_al_prd_no_se_pierde_ni_miente

Estado: in_progress
Microservicios:
- harness

## Alcance

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

## Delegacion (implementer)
- 

## Criterios de cierre (reviewer)
- 

## Riesgos
- 

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- 

### Avance 2026-08-27T16:27:08Z
La vuelta al PRD sale del branch: se escribe en el docs/prd/ de la RAIZ y DESPUES de integrar. Punteros validados antes de escribirse (decidir_vuelta pura + aplicar_vuelta). Nuevo prd doctor [--reparar] que deriva el pendiente del backlog. Reparados los 18 punteros rotos y 13 bitacoras faltantes del PRD real. AC-1..AC-12 con evidencia en docs/impl-60.md.

---
Cerrado: 2026-08-27T18:04:26Z - status=done - La vuelta al PRD sale del branch: se escribe en el docs/prd/ de la RAIZ y DESPUES de integrar, y ningun puntero se escribe sin verificar que resuelve. Nuevo prd doctor [--reparar]. Reparados los 18 punteros rotos y 13 bitacoras faltantes. 12/12 AC verdes, 576 tests.
