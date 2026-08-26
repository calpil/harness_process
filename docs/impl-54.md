# Evidencia de implementación — Feature #54

Spec: `docs/spec-feature-54-prd-apply-escribe-en-el-docs-de-la-feature.md` (approved)

## Cambios

- `prd propose` y `prd apply` resuelven primero el `HarnessPaths` de la
  feature registrada y reutilizan esa única resolución para alcance, propuesta,
  señales, citas, escritura, sello y registro.
- Las citas de `ya-esta` parten del padre del `docs/` seleccionado, de modo que
  también se validan contra el worktree y no contra el principal.
- Si no hay worktree válido, el comando conserva el `docs/` efectivo y emite
  un aviso explícito de fallback.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | El fixture ejecuta `prd propose --feature 1` desde el principal y exige que `prd-diff-1.md` nazca solo en `feature-1/docs/`. |
| AC-2 | El mismo recorrido responde la propuesta y ejecuta `prd apply --yes` desde el principal; la escritura y el sello quedan en el worktree. |
| AC-3 | PRD y architecture del principal y del worktree difieren; una cita que solo existe en el worktree valida y el architecture principal queda byte a byte intacto. |
| AC-4 | `prd_commands_should_announce_the_no_worktree_fallback` exige el aviso y el destino `docs/` clásico. |
| AC-5 | No se genera copia documental en el principal: los cambios quedan en la rama/worktree que el merge normal incorpora. |
| AC-6 | Los fixtures no usan Git real, red ni el repositorio del desarrollador; cubren propose, apply confirmado, aislamiento y fallback. |

## Verificación ejecutada

- `cargo test prd_propose_and_apply_should_use_the_registered_feature_worktree`
- `cargo test prd_commands_should_announce_the_no_worktree_fallback`
- `cargo test prd_apply_with_yes_should_write_seal_and_log`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Todas verdes.
