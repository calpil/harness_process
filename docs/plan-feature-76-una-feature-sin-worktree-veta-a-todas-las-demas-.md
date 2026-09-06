# Plan - Feature #76: una feature sin worktree veta a todas las demas y mata el paralelismo

Estado: in_progress
Microservicios:
- harness_process (rust: aislamiento/start)

## Alcance

Corregir UNA decision pura (`aislamiento::decidir`) y los mensajes que la
acompañan. Es una regresion introducida por la #72 el mismo dia.

## Peldano de huella

`Peldano elegido: 1 (extender lo que existe) porque el arreglo es cambiar una
condicion dentro de una funcion que ya existe.` No hay flag, comando ni campo
nuevo: el backlog, las ramas y los worktrees se comportan igual.

## Delegacion (implementer)

- D-1 (AC-1..AC-5): `decidir` pasa del modelo "una sin aislar veta a todas" al
  modelo "el checkout compartido tiene capacidad UNO". Se elimina la variante
  `Rechazo::BypassEnParalelo` porque su condicion real es la de
  `OcupanteSinAislar`, y `Decision::Aislar` pasa a llevar la lista de features
  sin aislar con las que convive, para INFORMAR sin bloquear.
- D-2 (AC-1): `start` imprime esa lista.
- D-3 (AC-7): los tres tests de la #72 que codificaban la regla ancha se
  reescriben; cada uno dice que regla codificaba y cual codifica ahora.
- D-4: los avisos y los documentos (`UPDATING.md`, `AGENTS.md`,
  `architecture.md`) que prometian el veto se corrigen: un documento que dice
  "no arranca ninguna otra" sobre una regla que ya no lo hace es una mentira
  nueva.

## Criterios de cierre (reviewer)

- La mutacion "vuelve la regla ancha" tiene que poner en rojo el test que
  reproduce el caso reportado, no solo un test unitario.
- Ningun test de la #72 se borra: los que afirmaban el rechazo se reescriben
  diciendo por que.

## Riesgos

- R-1: relajar un gate de seguridad. Se relaja SOLO el caso que la evidencia no
  sostenia (aislada + no-aislada); el incidente real (dos sin aislar en el mismo
  arbol) sigue bloqueado y tiene su test.
- R-2: el caso sin git, que el usuario decidio explicitamente en la #72, no
  cambia: ahi todas son no aisladas y capacidad uno sigue siendo una a la vez.

## Observaciones (decisiones pendientes)

- OBS-1: la rama de integracion se pregunta antes de `close --status done --to`.

---
Cerrado: 2026-09-06T02:33:18Z - status=done - El checkout compartido tiene capacidad uno: una feature con worktree arranca siempre; se rechaza solo que dos escriban en el mismo arbol
