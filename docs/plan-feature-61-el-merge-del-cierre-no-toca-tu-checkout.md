# Plan - Feature #61: el_merge_del_cierre_no_toca_tu_checkout

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
Cerrado: 2026-08-27T18:35:58Z - status=done - El merge del cierre corre siempre en un worktree temporal --detach: el checkout del usuario nunca participa. La rama destino avanza con reset --keep, que conserva lo que tenga sin commitear. La colision irreductible se detecta ANTES de commitear o mergear y nombra los archivos. 7/7 AC ejecutables verdes.
