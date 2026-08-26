# Evidencia de implementación — Feature #55

Spec: `docs/spec-feature-55-check-resuelve-el-spec-de-la-feature.md` (approved)

## Cambios

- El resumen `status` —invocado por `harness_check.sh`— crea
  `paths_feature = paths.para_feature(feature)` dentro de cada feature activa.
- Las comprobaciones de plan y de spec usan ese contexto local; el backlog y
  la bitácora permanecen en el principal.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | Con el binario ejecutado desde el principal, `#1` aprobado en su worktree se informa `approved (fresco)`, no ausente. |
| AC-2 | La suite modifica y luego borra el spec de `#2`; el resumen muestra respectivamente `draft (STALE)` y `ausente`, y `check-spec --feature 2` bloquea con el mismo estado. |
| AC-3 | Dos features activas tienen worktrees distintos: cambiar o borrar el spec de `#2` deja `#1 approved (fresco)`. |
| AC-4 | Un sandbox sin Git inicia una feature sin worktree y `status` conserva `docs/spec-feature-1-clasica.md` sin fallo. |
| AC-5 | El fixture contrasta explícitamente las líneas de `status` contra `check-spec --feature 1/2` para aprobado, draft, stale y ausente. |
| AC-6 | Los worktrees son repos Git temporales locales; no hay remoto, red ni datos del repositorio real. |

## Verificación ejecutada

- `cargo test status_should_resolve_each_spec_from_its_registered_worktree`
- `cargo test status_should_keep_the_classic_docs_fallback_without_a_worktree`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Todas verdes.
