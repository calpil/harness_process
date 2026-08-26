# Evidencia de implementación — Feature #57

Spec: `docs/spec-feature-57-verify-corre-en-el-worktree-de-la-feature.md` (approved)

## Cambios

- `verify` resuelve `HarnessPaths::para_feature(feature)` antes de leer el spec,
  y obtiene del padre de ese `docs/` la única raíz de ejecución para todos los
  AC.
- Spec, reporte y comandos usan el mismo worktree; el reporte y el JSON
  incorporan `Raiz de ejecucion` / `raiz_ejecucion` para que la medida sea
  auditable.
- Sin worktree válido, conserva la raíz documental efectiva y muestra un
  diagnóstico antes de ejecutar.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | El fixture llama `verify --feature 1` desde el principal; `test "$(cat origen.txt)" = feature` solo puede pasar en el worktree y queda verde. |
| AC-2 | Principal y feature tienen `origen.txt` distinto, y `solo-worktree.txt` no existe en principal; ambos AC verdes prueban que no se ejecutó código viejo. |
| AC-3 | Cinco AC reciben la misma raíz y el reporte `worktree/docs/verify-1.md` declara esa ruta; no se crea `docs/verify-1.md` en principal. |
| AC-4 | El fixture sin Git no registra worktree, exige el diagnóstico de fallback y confirma comando, spec y reporte en la raíz efectiva clásica. |
| AC-5 | En la corrida desde worktree se conservan `rojo`, `timeout` (1 s) y `vacio`; además se reejecutan las regresiones independientes de timeout y vacío. |
| AC-6 | Los fixtures usan repos Git temporales locales y archivos controlados, sin remoto, red ni datos del repositorio real. |

## Verificación ejecutada

- `cargo test verify_should_run_and_report_from_the_registered_feature_worktree`
- `cargo test verify_should_keep_the_effective_root_when_the_feature_has_no_worktree`
- `cargo test verify_should_time_out_a_hung_command`
- `cargo test verify_should_mark_an_ac_that_ran_nothing_as_vacio`
- `cargo test render_should_summarize_and_detail_failures`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Todas verdes.
