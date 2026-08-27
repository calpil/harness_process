# Spec - Feature #62: el_cierre_no_declara_hecho_lo_que_no_hizo

Estado: approved
Aprobado: 2026-08-27T18:50:00Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-62-el-cierre-no-declara-hecho-lo-que-no-hizo.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: reordenar `close::run` para que el estado del cierre se escriba
DESPUES de integrar. Tercera de la serie #60/#61/#62, y la que cierra la
familia: las tres son la misma promesa hecha antes de tiempo.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Alan cierra la #60 y el merge falla. Cuando mira el estado, el arnes ya
le habia dicho "Feature #60 cerrada como done", la feature figura `done` en el
backlog, la transicion ya salio hacia Jira, el plan tiene su linea "Cerrado:",
el estado vivo se archivo y se borro, `current.md` ya no la lista,
`history.md` tiene "close feature #60 status=done" y el hub tiene la memoria del
cierre. Nueve afirmaciones sobre un trabajo que NO esta integrado. Para retomar
tiene que confiar en que re-correr el cierre arregle todo eso, sin ninguna
garantia de que asi sea.

DESPUES: cuando la integracion falla, no hay nada que deshacer, porque no se
escribio nada: la feature sigue `in_progress`, su estado vivo sigue donde
estaba, `history.md` no tiene la linea y a Jira no salio nada. Alan resuelve lo
que sea (commitear, `git stash`, el conflicto) y vuelve a correr el mismo
comando. El cierre solo dice "cerrada" cuando lo esta.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                               DESPUES
gates                             gates
mutar + save_features             validar --to y colisiones   <-- lo que puede negarse
atlassian on_close                anotar plan (idempotente)   <-- lo que viaja
anotar plan                       archivar estado                 en la rama
archivar estado                   integrar (commit+merge+push+borrar wt)
borrar current                     |__ ¿fallo? -> Err: NADA del estado se escribio
escribir indice                   mutar + save_features        <-- recien ahora
history.md                        atlassian on_close
memorias del hub                  borrar current + escribir indice
"Feature #N cerrada"              history.md + memorias + stamp
integrar   <-- ACA FALLA          "Feature #N cerrada"
 |__ las 9 cosas ya mintieron     vuelta al PRD (#60)
```

Dos cosas NO se pueden mover al final, y por eso quedan antes de integrar: la
anotacion del plan y el estado archivado viven en el `docs/` del worktree, y el
merge borra ese worktree. Escribirlas despues seria no escribirlas nunca. Se
hacen IDEMPOTENTES para que un reintento no las duplique.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario cuyo cierre fallo al integrar, quiero que el backlog siga
  diciendo la verdad, para poder reintentar sin auditar que quedo a medias.
- P1: Como usuario que reintenta el cierre despues de resolver, quiero que
  funcione igual que la primera vez y no me deje artefactos duplicados.
- P2: Como usuario cuyo cierre salio bien, quiero exactamente el mismo
  resultado que antes: esta feature no cambia lo que pasa cuando todo anda.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->

- AC-1: Given un cierre `done` cuya integracion falla por falta de `--to`, When
  corre, Then la feature sigue `in_progress` en `feature_list.json`, sin
  `closed_at` y sin `leccion`.
  Comando: `cd rust && cargo test sin_to_el_backlog_no_queda_en_done`
- AC-2: Given un cierre `done` cuya integracion falla por colision con trabajo
  sin commitear (feature #61), When corre, Then la feature sigue `in_progress`,
  su `progress/current-<id>.md` sigue existiendo, `progress/history.md` NO tiene
  la linea del cierre y NO se imprime "Feature #N cerrada".
  Comando: `cd rust && cargo test integracion_fallida_no_escribe_nada_del_estado`
- AC-3: Given un cierre `done` cuya integracion falla por conflicto REAL de
  merge, When corre, Then vale lo mismo que AC-2: el estado no se toco.
  Comando: `cd rust && cargo test conflicto_de_merge_no_deja_el_backlog_en_done`
- AC-4: Given un cierre que fallo al integrar, When el usuario resuelve y vuelve
  a correr el MISMO comando, Then el cierre se completa y no quedan artefactos
  duplicados: el plan tiene UNA sola linea "Cerrado:" y `history.md` UNA sola
  entrada de cierre.
  Comando: `cd rust && cargo test reintentar_el_cierre_no_duplica_artefactos`
- AC-5: Given un cierre `done` que integra bien, When corre, Then el resultado
  es el mismo de siempre: backlog `done`, estado archivado, `current-<id>`
  borrado, indice reescrito, bitacora en el PRD y worktree borrado.
  Comando: `cd rust && cargo test cierre_exitoso_hace_todo_lo_de_siempre`
- AC-6: Given un cierre que NO integra (`blocked`, `pending`, `superseded`),
  When corre, Then se comporta igual que antes: escribe el estado, conserva rama
  y worktree e informa lo que conserva.
  Comando: `cd rust && cargo test cierres_que_no_integran_no_cambian`
- AC-7: Given la anotacion del plan, When el cierre corre dos veces sobre la
  misma feature y el mismo estado, Then la linea "Cerrado:" aparece UNA sola
  vez (es lo que hace seguro escribirla antes de integrar).
  Comando: `cd rust && cargo test anotar_plan_es_idempotente`
- AC-8: Given `close::run`, When se lee, Then el orden esta escrito como fases
  con su razon, y ninguna escritura de ESTADO (backlog, Jira, progress/,
  history.md, memorias, mensaje de exito) ocurre antes de `integrar`.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: `close --feature <id> --status <estado> [--to <rama>]`.
- ESTADO (se escribe recien despues de integrar): `feature_list.json`
  (`status`, `closed_at`, `note`, `leccion`, `superseded_by`), el outbox de
  Atlassian, `progress/current-<id>.md`, `progress/current.md`,
  `progress/history.md`, `.last_autocheck-<id>`, las memorias del hub y el
  mensaje de exito.
- ARTEFACTOS DE LA RAMA (se escriben antes, porque el merge borra el worktree):
  la linea "Cerrado:" del plan y `docs/estado-feature-<id>-<slug>.md`.
- candado: los dos artefactos son idempotentes — la linea del plan solo se
  agrega si no esta, el estado archivado se sobrescribe.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO se cierra una feature

  FASE 0 — lo que puede negarse, antes de escribir nada
     gates (spec, verify, docs, leccion) + --to + colisiones

  FASE 1 — lo que tiene que viajar en la rama
     anotar el plan (si no estaba ya)
     archivar el estado vivo en el docs/ de la feature

  FASE 2 — integrar
     commitear el worktree, mergear, publicar, borrar el worktree
     ¿fallo? -> salir con el error. NADA del estado se escribio.

  FASE 3 — recien ahora, el estado
     backlog, Atlassian, progress/, history.md, memorias, checkpoints
     decir "cerrada"
     volver al PRD

  ENTONCES el arnes solo afirma un cierre que ocurrio.
```
Promesas: si no integro, el estado no se toca · reintentar no duplica · un
cierre exitoso hace exactamente lo mismo que antes.

## No funcionales
- SLOs: mismo costo; solo cambia el orden.
- Seguridad: sin rollback y sin deshacer nada — no hay estado que revertir
  porque no se escribe hasta el final. El intent de Jira y la memoria del hub,
  que no se pueden deshacer, pasan a emitirse solo cuando el cierre ocurrio.
- Observabilidad: "Feature #N cerrada" pasa a imprimirse DESPUES de la salida
  de `[GitFlow]`, que es el orden real de los hechos.

## Fuera de alcance
- Rollback o compensacion de efectos ya emitidos: la decision fue reordenar
  para no necesitarlos (USUARIO, 2026-08-27).
- Cambiar el algoritmo de merge, los gates o la vuelta al PRD.
- Que el cierre sea atomico ante una caida del proceso a mitad de la FASE 3
  (kill -9, disco lleno): fuera de lo que este cambio promete.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Reordenar vs rollback — DECIDIDO (USUARIO, 2026-08-27): reordenar. Un rollback
  quedaria parcial (el intent de Jira y la memoria del hub no se deshacen) y
  ademas habria que acordarse de mantenerlo al agregar cada efecto nuevo.
- Los dos artefactos que no se pueden mover — DECIDIDO (USUARIO, 2026-08-27):
  se escriben antes de integrar y se hacen idempotentes.
- Cambia el ORDEN de la salida: "Feature #N cerrada" pasa a ir despues de
  `[GitFlow] integrando`. Es visible para quien mire la consola o parsee la
  salida, y es el orden verdadero de los hechos.
