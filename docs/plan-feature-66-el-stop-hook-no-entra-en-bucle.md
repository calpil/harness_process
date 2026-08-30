# Plan - Feature #66: el_stop_hook_no_entra_en_bucle

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

### Avance 2026-08-30T20:08:56Z
Implementacion de la #66 verde: Stop y PreToolUse de Claude/POSIX cableados a bin/harness-hook via SURFACE_BASE (el runtime es superficie y vive en la raiz; con HOOK_BASE salia 127 en layout subdir), harness_check.sh degrada en la segunda vuelta imprimiendo TODO, y centinela propio progress/.stop_streak con firma del conjunto de fallos. 8 modos de tests/stop_hook_check.sh y 7 de commit_guard_check.sh verdes, prueba del rojo hecha sobre cada mecanismo. HALLAZGO: el bug del SIGPIPE que motivo el AC-11 NO se reproduce (medido hasta 8MB en bash de macOS, rc=0 siempre); el cambio a case queda como robustez y el AC-11 se corrigio para decir eso, lo que deja el spec stale y necesita re-firma del usuario.
