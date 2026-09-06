# Review - Feature #77: con docs/ como repo aparte, el arnes escribe directo en docs/ y no crea docs-wt
Revisado: approved · 2026-09-06T22:02:14Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento, y la que diseño la pieza que aca se
quita (OBS-5 de la #72). Metodo: inventariar realestate antes de escribir el
spec, y mutar produccion para confirmar que los tests caen.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/paths.rs:59 | CUBIERTO. Probado en rojo: con la rama de docs aparte forzada a `false`, el test cae. |
| AC-2 | rust/src/commands/start.rs:102 | CUBIERTO. `preparar_docs` no existe mas; el test afirma que no hay `docs_worktree` en el backlog y que el repo docs no gano ramas (`git branch --list` sobre el repo docs). |
| AC-3 | rust/src/paths.rs:49 | CUBIERTO. El test planta `docs_worktree` a mano en el JSON —como quedo en realestate— y comprueba que el `advance` termina en `docs/` y que no se creo `docs-wt/`. |
| AC-4 | rust/src/commands/close.rs:805 | CUBIERTO por ausencia: no queda ninguna referencia a `docs_worktree` en `close.rs` (grep en el impl). |
| AC-5 | rust/src/paths.rs:59 | CUBIERTO. Los tests existentes sobre `sandbox_git` (docs dentro del repo principal) pasan sin cambios: la rama `else` es la de antes. |
| AC-6 | rust/tests/cli_basics.rs:5573 | CUBIERTO. Dos features, un `docs/`, cada `advance` a su plan. |
| AC-7 | rust/tests/cli_basics.rs:5537 | CUBIERTO. Un test reescrito con su historia, ninguno borrado. |
| AC-8 | rust/src/paths.rs:59 | CUBIERTO. |
| AC-9 | rust/src/commands/start.rs:266 | MANUAL, pendiente hasta despues del cierre: los comandos para realestate se entregan al usuario. |

## Lo que el review tiene que decir

La pieza que esta feature quita la puse yo hace un dia, y le pedi al usuario que
la aprobara con un argumento que sonaba bien: "evita que el spec se escriba en
un `docs/` vacio". Era cierto. Lo que no dije, porque no lo vi, es que un
worktree del repo docs por feature tiene ciclo de vida —una rama, un merge, un
borrado— y que nadie iba a hacerse cargo de el. El resultado fue exactamente lo
que la #71 habia arreglado para el sello de cierre, reintroducido para todos los
demas documentos.

El repo ya habia resuelto esto dos veces (#60: la bitacora del PRD; #71: el
sello) con la misma respuesta: lo compartido va a la raiz. No la busque.

## Riesgo declarado

Los documentos de una feature quedan SIN COMMITEAR en el repo docs hasta que el
usuario commitee, igual que la bitacora del PRD. Antes tampoco llegaban: quedaban
sin commitear en un worktree que nadie miraba.

## Veredicto

Los nueve AC tienen cobertura. La mutacion pone en rojo dos tests. Los tres
documentos que describian el `docs-wt` estan corregidos.
