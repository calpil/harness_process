# Impl - Feature #76: una feature sin worktree veta a todas las demas y mata el paralelismo

Spec: docs/spec-feature-76-una-feature-sin-worktree-veta-a-todas-las-demas-.md
Plan: docs/plan-feature-76-una-feature-sin-worktree-veta-a-todas-las-demas-.md

## Lo que estaba roto

Una regresion de la #72, del mismo dia. Con una feature abierta sin worktree,
`start` rechazaba a TODAS las demas, incluidas las que traian su propio arbol.
El usuario lo reporto con una captura: *"avisame cuando la #99 libere y
arranca"*. Reproducido en fixture:

```
$ harness start --feature 1 --sin-worktree     # ok, declarada NO AISLADA
$ harness start --feature 2                    # quiere su worktree
[GATE] No se arranca la feature #2: la feature #1 Uno esta abierta SIN worktree.
```

La #1 escribe en `<raiz>`; la #2 escribiria en `<raiz>-wt/2-dos`. **No se
solapan.** El incidente que motivo la #72 fueron cuatro features `--sin-worktree`
sobre el MISMO arbol; yo lo traduje a una regla mas ancha de lo que eso
sostenia.

## El modelo corregido

**El checkout compartido tiene capacidad UNO.** Lo unico que se rechaza es que
dos features escriban ahi. Una feature con worktree no comparte nada con una que
no lo tiene, asi que arranca siempre — y se le INFORMA con quien convive.

## Evidencia por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/aislamiento.rs:196 | `Decision::Aislar` lleva `conviven_sin_aislar`: la nueva arranca y `rust/src/commands/start.rs:82` informa. Test unitario en rust/src/aislamiento.rs:271 y de comportamiento en rust/tests/cli_basics.rs:5487, que afirma el aviso, la rama creada y `aislada=true`. |
| AC-2 | rust/src/aislamiento.rs:172 | `--sin-worktree` se rechaza solo si ya hay otra sin aislar (`ocupante`, rust/src/aislamiento.rs:157). Test rust/tests/cli_basics.rs:5424: dos sin aislar, el backlog no se toca. |
| AC-3 | rust/src/aislamiento.rs:172 | `--sin-worktree` junto a una aislada arranca serial. Test rust/tests/cli_basics.rs:5403. |
| AC-4 | rust/src/aislamiento.rs:147 | Sin git, la rama del `let Some(_repo) = ctx.repo else` no cambio: todas son no aisladas, una a la vez. Test rust/tests/cli_basics.rs:5606, intacto desde la #72. |
| AC-5 | rust/src/aislamiento.rs:355 | Dos al mismo worktree: sin cambios, test intacto. |
| AC-6 | rust/tests/cli_basics.rs:5449 | Fallo de git: sin cambios, test intacto. |
| AC-7 | rust/src/aislamiento.rs:306 | Tres tests de la #72 reescritos, ninguno borrado; cada uno dice en su doc-comment que regla codificaba y cual codifica ahora. `Rechazo::BypassEnParalelo` se elimino (rust/src/aislamiento.rs:92) porque su condicion real era la de `OcupanteSinAislar`. |
| AC-8 | rust/src/aislamiento.rs:327 | Suite, clippy, smoke y paridad. `la_tabla_del_checkout_de_capacidad_uno` codifica la tabla entera del spec en un solo test. |
| AC-9 | rust/tests/cli_basics.rs:5487 | MANUAL: el escenario reportado, reproducido y verde con el binario nuevo. |

## La mutacion

Devolver la regla ancha (`if let Some(o) = ocupante { return Rechazar(...) }`
antes del chequeo de worktree) pone en rojo tres tests: los dos unitarios
(`con_worktree_propio_convive_con_una_sin_aislar`,
`la_tabla_del_checkout_de_capacidad_uno`) y el de comportamiento que reproduce
el caso reportado (`start_with_worktree_should_coexist_with_an_unisolated_feature`).

## Los documentos que prometian el veto

`UPDATING.md` (las dos copias), `AGENTS.md` y `docs/architecture.md` decian
"mientras siga abierta, no arranca ninguna otra". Un documento que describe una
regla que ya no existe es una mentira nueva: se corrigieron los tres, y cada uno
dice que la #76 acoto la regla y por que.

## Lo que se relajo, y lo que no

| Se relajo | Sigue bloqueado |
| --- | --- |
| aislada + no-aislada conviven | dos no-aisladas en el mismo arbol |
| `--sin-worktree` junto a una aislada arranca | sin git: una a la vez (decision del usuario del 2026-09-05) |
| | dos features al mismo worktree |
| | fallo de git cancela el arranque |
