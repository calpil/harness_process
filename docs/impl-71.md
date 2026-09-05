# Impl - Feature #71: El close archiva el sello de cierre en el worktree que acaba de borrar, y lo pierde

Spec: docs/spec-feature-71-el-close-archiva-el-sello-de-cierre-en-el-worktr.md
Plan: docs/plan-feature-71-el-close-archiva-el-sello-de-cierre-en-el-worktr.md

## Lo que la ficha decia y lo que se midio

La ficha describia DOS perdidas —el sello y las lineas de Evidencia de
`progress/current-<id>.md`— y son UNA: `archivar_estado` copia el cuerpo entero
del estado vivo DENTRO del sello. Comprobado sobre el cierre real de la #72,
cuyo `docs/estado-feature-72-*.md` conserva sus 15 lineas de Evidencia. Si el
sello sobrevive, la evidencia sobrevive; si se pierde, se pierden las dos.

Y el estado del bug cambio bajo nuestros pies: la feature #72, cerrada horas
antes, le dio al repo `docs/` su propio worktree y dejo de borrarlo. Medido con
un fixture de repo docs aparte, el 2026-09-05:

    Feature #1 cerrada como done. Estado archivado en docs/estado-feature-1-se-pierde.md.
    $ find . -name "estado-feature-1-*.md"
    ./docs-wt/1-se-pierde/estado-feature-1-se-pierde.md

O sea que la #72 tapo la perdida de datos sin proponerselo, y lo que quedaba era
peor de explicar: **el cierre nombraba una ruta que no existe**, y el archivo
real vivia en una rama del repo docs que nadie mergea. Es "el cierre no declara
hecho lo que no hizo" (#62) un nivel mas abajo.

## Evidencia por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/commands/close.rs:438 | El sello se escribe en `raiz_del_prd(paths)/docs`, la misma raiz que ya usaba la vuelta al PRD. El test `close_should_not_archive_the_state_into_the_worktree_it_deletes` (rust/tests/cli_basics.rs:8014) afirma que el ARCHIVO existe despues del cierre, no que el cierre salio 0 — el exit code ya era 0 con el bug. |
| AC-2 | rust/tests/cli_basics.rs:8014 | El fixture es un proyecto donde `docs/` es un repo git aparte, que es el caso de realestate y el que perdio la #124. Probado en rojo: devolviendo la escritura a `paths.plans`, el test FALLA nombrando la ruta que falta. |
| AC-3 | rust/src/commands/close.rs:538 | `ruta_del_estado_archivado` (feature #63) se BORRO, no se dejo devolviendo siempre lo mismo: existia para elegir entre dos rutas y ahora hay una sola. El mensaje (rust/src/commands/close.rs:339) nombra esa ruta y el test la resuelve contra el disco. |
| AC-4 | rust/tests/cli_basics.rs:8014 | El test afirma sobre el CONTENIDO —que el sello contiene el cuerpo del estado vivo y su seccion `Evidencia`— y ademas que `progress/current-1.md` se borro, que es lo que hace del sello la unica copia. |
| AC-5 | rust/src/commands/close.rs:262 | La escritura se movio de la FASE 1 a la FASE 3, despues de integrar. `close_should_not_leave_a_seal_for_an_integration_that_failed` (rust/tests/cli_basics.rs:8064) fuerza un conflicto de merge real y afirma que no queda sello. Probado en rojo: devolviendola a la fase 1, FALLA. |
| AC-6 | rust/src/commands/close.rs:440 | No hay una sola linea de migracion: el unico lugar del cierre que nombra `estado-feature` es el que ESCRIBE el sello nuevo. Los sellos ya escritos no se leen, no se mueven y no se borran. |
| AC-7 | rust/tests/cli_basics.rs:8014 | Suite completa, clippy, smoke del instalador y gate de paridad. |
| AC-8 | rust/src/commands/close.rs:438 | Los sellos nuevos caen en `<raiz>/docs/`, junto a los 63 que ya estan ahi (contados, no estimados). No hay una segunda ubicacion que haya que conocer. |

## Las dos mutaciones, y lo que ponen en rojo

| Mutacion | Test que cae |
| --- | --- |
| el sello vuelve a `paths.plans` (el docs de la feature) | `close_should_not_archive_the_state_into_the_worktree_it_deletes` |
| el sello vuelve a escribirse en la FASE 1, antes de integrar | `close_should_not_leave_a_seal_for_an_integration_that_failed` |

## Efectos colaterales, dichos

1. **Seis tests existentes cambiaron de lugar de observacion**, ninguno se
   borro: los de la #62 y la #63 comprobaban el sello con `git show main:...` y
   ahora lo comprueban en disco; el del mensaje incorpora el "sin commitear"; el
   que la #72 habia apuntado al `docs/` del worktree vuelve a la raiz; y el del
   corpus de la #68 gano una entrada. El detalle esta en `docs/review-71.md`.
   Se BORRO uno: `ruta_del_estado_archivado_es_pura`, junto con su funcion.
2. **El corpus real de specs gano un AC.** El `AC-8 (MANUAL)` de esta feature
   tiene la forma que el parser viejo descartaba, asi que aparece en la lista
   que mide `los_siete_que_faltaban_y_ninguno_mas`. Se agrego a la lista
   esperada: es evidencia de que el arreglo de la #68 sigue haciendo falta en el
   uso normal, no solo en el caso que la #68 escribio a proposito.

## Lo que NO hace

- **No commitea el sello.** Vive en la raiz, fuera de la rama, asi que ningun
  merge se lo lleva. El cierre lo dice en el mensaje en vez de dejarlo implicito.
- **No recupera lo perdido.** El sello de la #124 de realestate es irrecuperable
  y se reconstruyo a mano en su momento.
- **No integra el repo `docs/`.** Eso lo sigue decidiendo el usuario (#72).
