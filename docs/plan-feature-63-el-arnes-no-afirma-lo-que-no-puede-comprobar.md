# Plan - Feature #63: el_arnes_no_afirma_lo_que_no_puede_comprobar

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

---
Cerrado: 2026-08-27T19:41:21Z - status=done - El test del commit_guard vuelve a medir en macOS (timeout(1) no existe ahi: ahora elige entre timeout/gtimeout/perl alarm, se auto-prueba, y falla si no hay ninguno en vez de saltear en verde) y la prueba-del-rojo revierte las DOS defensas contra el cuelgue, no una. Y la ruta del estado archivado que imprime el cierre apunta a donde el archivo queda despues del merge, no al worktree que el propio cierre borro. 7/7 AC verdes.
