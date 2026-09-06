# Review - Feature #76: una feature sin worktree veta a todas las demas y mata el paralelismo
Revisado: approved · 2026-09-06T02:25:42Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento, y la misma que introdujo la regresion en
la #72 horas antes. Metodo: reproducir el reporte del usuario en fixture, medir
quien escribe donde, y mutar produccion.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/aislamiento.rs:196 | CUBIERTO. La decision es pura y el test de comportamiento (rust/tests/cli_basics.rs:5487) afirma las tres cosas: que informa, que arranca (rama creada) y que queda `aislada=true`. Probado en rojo con la regla ancha devuelta. |
| AC-2 | rust/src/aislamiento.rs:172 | CUBIERTO. Dos sin aislar se rechaza y el backlog queda intacto (rust/tests/cli_basics.rs:5424). Es el incidente real de realestate, ahora como test. |
| AC-3 | rust/src/aislamiento.rs:172 | CUBIERTO (rust/tests/cli_basics.rs:5403). |
| AC-4 | rust/src/aislamiento.rs:147 | CUBIERTO SIN CAMBIOS. La rama sin git no se toco y su test de la #72 sigue verde tal cual. |
| AC-5 | rust/src/aislamiento.rs:355 | CUBIERTO SIN CAMBIOS. |
| AC-6 | rust/tests/cli_basics.rs:5449 | CUBIERTO SIN CAMBIOS. |
| AC-7 | rust/src/aislamiento.rs:306 | CUBIERTO. Tres tests reescritos con doc-comment que nombra el test anterior y la regla que codificaba. Uno mas agregado (`la_tabla_del_checkout_de_capacidad_uno`) que codifica la tabla del spec entera. |
| AC-8 | rust/src/aislamiento.rs:327 | CUBIERTO. |
| AC-9 | rust/tests/cli_basics.rs:5487 | CUBIERTO como test de comportamiento del escenario reportado. No se corrio contra realestate: su instalacion se actualiza aparte. |

## Lo que el review tiene que decir sobre la #72

La regla ancha no fue un typo: fue una lectura de mas sobre una frase del spec
("no habilitan paralelo de escritura") y una generalizacion de mas sobre el
incidente (cuatro sin worktree en el mismo arbol -> "una sin worktree bloquea a
todas"). Los tests de la #72 la codificaron con precision, y por eso pasaron:
median lo que el codigo hacia, no lo que tenia que hacer. Es la misma familia
que la #73 documento (el test que acompaña a la implementacion), en la variante
"el test codifica una regla mas estricta de lo que la evidencia justifica".

Lo detecto el usuario usando el arnes de verdad, no la suite. Vale anotarlo: el
gate era coherente consigo mismo y con sus tests, y estaba mal igual.

## Riesgo declarado

Se relaja un gate de seguridad. Se relaja SOLO el par (aislada, no-aislada), que
escriben en directorios distintos; el par (no-aislada, no-aislada), que es el
incidente original, sigue bloqueado y tiene su test. El caso sin git no cambio.

## Veredicto

Los nueve AC tienen cobertura. La mutacion que devuelve la regla ancha pone en
rojo tres tests, incluido el que reproduce el caso del usuario. Los documentos
que prometian el veto se corrigieron, para no dejar una mentira nueva en lugar
de la anterior.
