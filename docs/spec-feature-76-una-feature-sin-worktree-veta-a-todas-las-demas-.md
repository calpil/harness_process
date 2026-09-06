# Spec - Feature #76: una feature sin worktree veta a todas las demas y mata el paralelismo

Estado: approved
Aprobado: 2026-09-06T02:17:45Z por USUARIO (confirmacion explicita) - Aprobado por Alan en chat: el checkout compartido tiene capacidad uno; una feature con worktree arranca siempre
Plan: docs/plan-feature-76-una-feature-sin-worktree-veta-a-todas-las-demas-.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md

## La historia (antes -> despues)

ANTES: Alan trabaja en paralelo. Una feature quedo abierta sin worktree —la #99,
la #130 de realestate— y desde entonces **ninguna otra puede arrancar**, aunque
la nueva traiga su propio arbol. Termina escribiendole al agente "avisame cuando
la #99 libere y arranca". El paralelismo con worktrees, que es lo que la #47
construyo y la #72 vino a REFORZAR, dejo de funcionar.

DESPUES: una feature con su worktree arranca siempre. Lo que sigue bloqueado es
lo unico que de verdad se pisa: dos features escribiendo en el mismo arbol.

## Lo que se midio

Reproducido en un fixture con git real:

```
$ harness start --feature 1 --sin-worktree
  [!] Feature NO AISLADA (--sin-worktree): se escribe en el checkout compartido.

$ harness start --feature 2            # esta SI quiere su worktree
[GATE] No se arranca la feature #2: la feature #1 Uno esta abierta SIN worktree.
```

Y donde escribiria cada una:

| Feature | Escribe en |
| --- | --- |
| #1 (sin worktree) | `<raiz>` — el checkout compartido |
| #2 (con worktree) | `<raiz>-wt/2-dos` — arbol propio |

**No se solapan.** Son directorios distintos.

## De donde salio el error

El incidente que motivo la #72 (diagnostico 2026-09-04, seccion 3) fueron
**cuatro** features corriendo `start --sin-worktree` sobre el **mismo** arbol:
#98, #121, #122 y #126. Lo traduje a una regla mas ancha de lo que la evidencia
aguantaba —"una feature sin aislar bloquea a todas"— cuando lo que el caso
mostraba es "dos features en el mismo arbol se pisan".

El spec de la #72 decia: *"El uso serial sin worktree y los proyectos sin Git se
identifican como no aislados y **no habilitan paralelo de escritura**"*. La
lectura correcta es que la NO AISLADA no participa del paralelo de escritura; no
que tenga derecho de veto sobre las que si estan aisladas.

## El modelo correcto

**El checkout compartido es un recurso de capacidad UNO.**

| Nueva feature | Ya hay otra sin aislar | Resultado |
| --- | --- | --- |
| con worktree | si | ARRANCA (se informa) |
| con worktree | no | ARRANCA |
| sin aislar | si | RECHAZO: dos en el mismo arbol |
| sin aislar | no | ARRANCA, declarada NO AISLADA |

Sin repo git **todas** las features son no aisladas, asi que la capacidad uno
sigue significando una a la vez: el caso que el usuario decidio explicitamente el
2026-09-05 no cambia.

## Objetivos y no objetivos

- O-1: Que una feature con su worktree pueda arrancar siempre.
- O-2: Que siga bloqueado lo que de verdad se pisa: dos en el arbol compartido.
- NO-1: No se relaja el rechazo por fallo de git ni el de dos features al mismo
  worktree.
- NO-2: No se toca el comportamiento sin git.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una feature abierta SIN aislar, When se arranca otra que SI obtiene
  su worktree, Then arranca, y el arnes INFORMA que hay una sin aislar sin
  bloquear.
  Comando: `cd rust && cargo test --locked aislamiento`
- AC-2: Given una feature abierta SIN aislar, When se arranca otra tambien sin
  aislar (`--sin-worktree`), Then se rechaza: son dos en el mismo arbol.
- AC-3: Given una feature abierta CON worktree, When se arranca otra con
  `--sin-worktree`, Then arranca: la nueva ocupa el checkout compartido sola.
- AC-4: Given un proyecto sin repo git, When ya hay una feature abierta, Then la
  segunda se sigue rechazando, porque ahi todas son no aisladas. El caso decidido
  por el usuario el 2026-09-05 no cambia.
- AC-5: Given dos features que resolverian al MISMO worktree, When se arranca la
  segunda, Then se sigue rechazando.
- AC-6: Given un fallo de `git worktree add`, When se arranca, Then se sigue
  cancelando el arranque y la feature NO queda `in_progress`.
- AC-7: Given los tests de la #72 que codificaban la regla ancha, When se aplica
  esta correccion, Then se reescriben contra la regla nueva; ninguno se borra y
  cada uno dice que cambio y por que.
- AC-8: Given el cambio completo, When se corre la suite, Then quedan verdes los
  tests, clippy, el smoke y el gate de paridad.
  Comando: `cd rust && cargo test --locked`
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`
  Comando: `bash tests/parity_check.sh`
- AC-9 (MANUAL): Given el escenario que Alan reporto, When se corre con el
  binario nuevo, Then la feature que espera arranca sin que la otra tenga que
  cerrarse.

## Los datos que se tocan

Ninguno. Cambia una decision PURA (`aislamiento::decidir`) y los mensajes. El
backlog, los worktrees y las ramas se comportan igual.

## Pseudo-codigo (el acuerdo)

```
AL ARRANCAR:
  sin git            -> es no-aislada; si hay OTRA abierta, rechazar
  --sin-worktree     -> es no-aislada; si hay OTRA no-aislada, rechazar
  con worktree       -> arrancar; si hay una no-aislada, INFORMAR
  mismo worktree     -> rechazar (sin cambios)
  fallo de git       -> rechazar (sin cambios)
```

Promesa: el checkout compartido lo ocupa como maximo UNA feature; tener un arbol
propio nunca depende de lo que hagan las demas.

## No funcionales y verificacion

- Verificacion: la decision es una funcion PURA con tests exhaustivos de la
  tabla, mas tests de comportamiento sobre el binario para las dos direcciones.
- Prueba del rojo: cada test nuevo tiene que fallar contra el codigo actual.
- Los tests de la #72 se reescriben, no se borran: cada uno tiene que decir que
  regla codificaba antes y cual codifica ahora.

## Alcance de instalacion y fuera de alcance

Se corrige `harness_process`. La instalacion de realestate se actualiza con su
propio comando, aparte; esta feature no la toca.

## Observaciones (decisiones pendientes)

- OBS-1: la rama de integracion se pregunta antes de `close --status done --to`.
