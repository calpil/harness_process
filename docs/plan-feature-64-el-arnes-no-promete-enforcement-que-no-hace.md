# Plan - Feature #64: el_arnes_no_promete_enforcement_que_no_hace

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

### Avance 2026-08-28T22:23:49Z
Nucleo del gate listo y verde: require_review + veredicto_estampado + acs_sin_fila + gate en revision.rs, --veredicto en cli.rs, quinto gate en close.rs:119. 12 tests de revision:: verdes, 603 de la suite completa sin regresiones. Un test propio encontro un bug real: contains("AC-1") matcheaba AC-11, o sea el gate daba por cubierto un AC que no reviso; arreglado con match de token completo (fn menciona).

### Avance 2026-08-29T00:42:37Z
Rutas protegidas tocadas con el SI EXPLICITO del usuario (2026-08-28, en el chat): docs/prd/SDD-master.md via prd apply --yes (el texto prometia que un review de cinco segundos no se puede fabricar, y el reviewer lo desmintio con un printf de 4 lineas) y docs/prd/aprendizaje/PRD-aprendizaje.md:187 (afirmaba one_feature_at_a_time vigente; dejo de bloquear en la #47 y la #64 la borro del molde).

---
Cerrado: 2026-08-30T18:08:19Z - status=done - 
