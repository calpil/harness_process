# Impl - Feature #77: con docs/ como repo aparte, el arnes escribe directo en docs/ y no crea docs-wt

Spec: docs/spec-feature-77-con-docs-como-repo-aparte-el-arnes-escribe-direc.md
Plan: docs/plan-feature-77-con-docs-como-repo-aparte-el-arnes-escribe-direc.md

## Lo que estaba pasando (realestate, 2026-09-06)

```
docs/          dab5879 [main]
docs-wt/131-…  02d4314 [feature/131-…]   SIN MERGEAR   ?? spec, plan
docs-wt/132-…  dab5879 [feature/132-…]   SIN MERGEAR   ?? spec, plan, impl, verify
docs-wt/142-…  dab5879 [bugfix/142-…]    SIN MERGEAR   ?? spec, plan, impl, verify
```

Los artefactos de las tres estaban sin commitear en su worktree: ni siquiera
habian llegado a la rama. `docs/`, con el PRD y el SDD, no veia ninguno. Era la
OBS-5 de la #72 (el arnes crea el worktree de docs) mas la #71 (el cierre no
integra el repo docs). El usuario revirtio la OBS-5: "el worktree era para
implementar las feat en paralelo, no para crear otro docs-wt".

## El arreglo: quitar una pieza

Con `docs/` como repo aparte, los documentos van a `<raiz>/docs/` — donde ya
iban el PRD, el SDD, `architecture.md` y el sello de cierre. No hay worktree del
repo docs, no hay rama, no hay `docs_worktree` en el backlog.

## Evidencia por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/paths.rs:59 | `para_feature` resuelve `plans = <raiz>/docs` cuando `repo_de_docs` (rust/src/git.rs:108) detecta docs aparte. Test rust/tests/cli_basics.rs:5537: el spec esta en `docs/`, no existe `docs-wt/`. |
| AC-2 | rust/src/commands/start.rs:102 | `preparar_docs` se elimino; `start` ya no escribe `docs_worktree`. El mismo test comprueba que el backlog no lleva el campo y que el repo docs no gano rama. |
| AC-3 | rust/src/paths.rs:49 | El campo `docs_worktree`, si sigue en el backlog de una feature vieja, se ignora: `para_feature` ya no lo lee. Test rust/tests/cli_basics.rs:5599 lo planta a mano y comprueba que el `advance` va a `docs/`. |
| AC-4 | rust/src/commands/close.rs:805 | El bloque que commiteaba el worktree de docs y avisaba de la rama sin integrar se quito; `PlanDeIntegracion` ya no lleva `docs_worktree`. |
| AC-5 | rust/src/paths.rs:59 | La rama `else` conserva la resolucion de siempre: con docs del repo principal, `<worktree>/docs`. Los tests de la #47 y #62 sobre el sandbox sin docs aparte siguen verdes sin tocarse. |
| AC-6 | rust/tests/cli_basics.rs:5573 | Dos features en paralelo con un solo `docs/`: las dos escriben, cada `advance` va a su plan y ninguno pisa al otro. |
| AC-7 | rust/tests/cli_basics.rs:5537 | El test de la #72 se reescribio con doc-comment que nombra el anterior y la regla que codificaba. El fixture `sandbox_git_con_docs_aparte` (rust/tests/cli_basics.rs:5515) se reuso tal cual. |
| AC-8 | rust/src/paths.rs:59 | Suite, clippy, smoke y paridad. |
| AC-9 | rust/src/commands/start.rs:266 | MANUAL: la migracion de los tres `docs-wt/` de realestate se hace con comandos, despues del cierre. `start` ahora imprime a donde van los documentos, para que no haya que adivinar. |

## La mutacion

Forzando `if false` en la rama de docs aparte de `para_feature` (vuelve al
`<worktree>/docs` vacio), caen dos tests:
`start_should_write_the_docs_of_a_separate_docs_repo_into_docs_itself` y
`two_parallel_features_with_a_separate_docs_repo_should_both_land_in_docs`.

## Los documentos que describian el docs-wt

`AGENTS.md`, `UPDATING.md` (las dos copias) y `docs/architecture.md` decian que
el repo docs recibia su worktree. Corregidos los tres; cada uno dice que la #77
lo revirtio y por que.

## Lo que NO hace

- No mueve lo que ya esta en `docs-wt/` de realestate: eso es shell sobre otro
  repo, con los comandos a la vista (AC-9).
- No cambia nada cuando `docs/` es parte del repo principal.
