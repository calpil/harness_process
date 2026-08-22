# Evidencia de implementacion - Feature #47: features_en_paralelo_con_worktrees

Spec: `docs/spec-feature-47-features-en-paralelo-con-worktrees.md` (approved, 25 AC)
Plan: `docs/plan-feature-47-features-en-paralelo-con-worktrees.md`

## Que se construyo

- `rust/src/git.rs` (modulo nuevo): ramas GitFlow, worktrees, merge sin
  reescritura de historia, push, y commit de lo que quede en el worktree. Todo
  degrada a "no hago nada" cuando no hay repo git.
- `rust/src/paths.rs`: `worktree` actual, `current_de(id)`,
  `autocheck_stamp_de(id)` y `para_feature(feature)` — las rutas de docs se
  resuelven DESDE la feature, no desde el directorio donde se ejecuta.
- `rust/src/progress.rs`: `escribir_indice()` (current.md pasa a ser indice) y
  `touch_autocheck_stamp_de()`.
- `rust/src/features.rs`: `feature_por_worktree()` y
  `active_feature_index_con_foco()`.
- `start`: sin rechazo de la segunda activa, crea rama + worktree ANTES de
  escribir el plan y el spec, y `--sin-worktree` para el modo clasico.
- `close`: `CierreOpts` con `--to`, integracion GitFlow y estado vivo por
  feature.
- `autocheck`: trabaja sobre la feature en foco, con su propio stamp.

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 arrancar en paralelo | OK | **Real**: con la #47 activa, `start --feature 48` arranco e imprimio `En paralelo con: #47`. Test `start_should_allow_a_second_feature_in_parallel` |
| AC-2 rama GitFlow | OK | **Real**: `feature/47-features-en-paralelo-con-worktrees`. Unit `nombre_rama_should_follow_gitflow` (incluye `bugfix/` para `kind: bug`) y test `start_should_create_branch_and_worktree_per_feature` |
| AC-3 worktree hermano | OK | **Real**: `/Users/alan/harness_process-wt/47-...`. Unit `ruta_worktree_should_be_a_sibling_of_the_repo` |
| AC-4 reusa lo existente | OK | **Real**: `start --feature 47` corrido dos veces reuso rama y worktree. Unit `preparar_should_create_branch_and_worktree_and_reuse_them` |
| AC-5 sin git | OK | Test `start_should_keep_working_without_git_or_with_sin_worktree` (imprime `sin aislamiento` y sigue) + unit `should_return_none_outside_a_repo` |
| AC-6 `--sin-worktree` | OK | Mismo test de integracion |
| AC-7 estado unico | OK | Unit `worktree_should_resolve_the_main_repo` (desde el worktree, el principal es el de siempre) + **real**: los comandos corridos desde los worktrees de la #47 y la #48 escribieron en el UNICO `feature_list.json` del principal |
| AC-8 estado vivo por feature | OK | **Real**: `progress/current-47.md` y `current-48.md` coexistieron. Test de integracion del arranque en paralelo |
| AC-9 `current.md` como indice | OK | **Real**: listo `#47` y `#48` con su rama y su worktree; al cerrar la #48 quedo solo la #47 |
| AC-10 stamp por feature | OK | **Real**: `.last_autocheck-47` sobrevivio al cierre de la #48. Test `close_should_not_touch_the_state_of_the_other_active_feature` |
| AC-11 cerrar una no toca a la otra | OK | **Real**: cerre la #48 y el `current-47.md` quedo byte a byte identico (diff vacio), con su stamp y su worktree intactos. Es el bug de la feature #45, ahora imposible |
| AC-12 foco por worktree | OK | **Real**: `check-spec` sin `--feature` desde el worktree de la #48 resolvio la #48, y desde el de la #47 resolvio la #47 |
| AC-13 exige `--feature` fuera | OK | **Real**: desde el principal con dos activas: `Varias features in_progress (#47, #48); especifica --feature <id>` |
| AC-14 `--to` obligatorio | OK | Test `close_done_should_refuse_without_to_and_then_integrate`: exit 2, `PREGUNTALE AL USUARIO` y lista de ramas |
| AC-15 merge | OK | Mismo test (el archivo de la rama aparece en `main`) + unit `merge_should_integrate_and_keep_history` |
| AC-16 sin trailers de IA | OK | Los dos tests verifican que el mensaje del merge no contiene `co-authored-by` ni `generated with` |
| AC-17 push del destino | OK (degradado en test) | Implementado con `git push origin <rama>`; en los sandboxes no hay remoto y el cierre informa `merge local hecho, pero no pude publicar`. Verificacion real en el cierre de esta misma feature |
| AC-18 conflicto aborta | OK | Unit `merge_should_abort_on_conflict_and_leave_everything_intact`: README intacto, sin restos del merge, rama sin cambiar y worktree en pie |
| AC-19 borra worktree, conserva rama | OK | Test de integracion + unit `borrar_worktree_should_keep_the_branch` |
| AC-20 destino inexistente | OK | Test `close_should_refuse_an_unknown_target_branch` + unit `merge_should_refuse_an_unknown_target` |
| AC-21 solo `done` integra | OK | **Real**: cerrar la #48 como `pending` imprimio `Rama ... conservada (el cierre pending no integra); su worktree tambien`. Test `close_blocked_should_keep_branch_and_worktree` |
| AC-22 base develop/main | OK | Unit `base_should_prefer_develop_then_main` (y el arnes nunca crea la base) |
| AC-23 prefijos configurables | PARCIAL | Los prefijos son constantes del modulo (`feature/`, `bugfix/`) y la base se elige por existencia. La configuracion por repo quedo sin exponer: ver reparo (1) |
| AC-24 comandos oficiales | OK | `cargo test`: 345 unit + 170 integracion = **515**; `clippy --all-targets -- -D warnings` limpio |
| AC-25 dos features reales | OK | Toda la columna "Real" de esta tabla salio de la #47 y la #48 abiertas a la vez en este repo |

## Un bug de diseno que aparecio durante la verificacion

La primera version resolvia el `docs/` por el **directorio actual**. Resultado:
el spec generado desde el principal "desaparecia" al mirarlo desde el worktree
(`estado: ausente`), y al reves. Se corrigio con `HarnessPaths::para_feature()`:
los docs se resuelven DESDE la feature (su worktree), no desde donde estas
parado, y `start` crea el worktree ANTES de escribir el plan y el spec para que
nazcan en la rama correcta.

## Otro hallazgo (no es de esta feature)

El gate de frescura del spec marca `SPEC ACTUALIZADO POR OTRO LLM` cuando cambia
el **mtime** aunque el **hash sea identico** (paso al copiar el spec al
worktree: `hash=2e6055cda2d73bb6` en ambos lados). No rompe nada — pide re-leer
— pero es ruido evitable. Merece feature propia.
