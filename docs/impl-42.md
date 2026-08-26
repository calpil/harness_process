# Evidencia de implementación — Feature #42

Spec: `docs/spec-feature-42-consolidar-esqueleto-del-paraguas.md` (approved)

## Cambios

- `lecciones consolidar --preparar --en <paraguas> --de <a,b>` crea un único
  borrador nuevo; es incompatible con `--aplicar`, que sigue siendo la acción
  que archiva miembros.
- El borrador une triggers en minúscula, ordenada y sin duplicados, y deja un
  `[[miembro]]` por cada selección válida.
- Si el destino existe, el comando informa que lo preserva y no lo reescribe;
  por tanto nunca toca prosa humana ni hace backups/archivos durante preparar.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | `preparar_paraguas_should_union_triggers_and_satisfy_structural_review` crea la unión `alfa, beta, zeta`. |
| AC-2 | La misma prueba y la integración verifican un puntero único para cada miembro. |
| AC-3 | El unit test usa `Beta`, `alfa`, `ALFA`, `zeta` y produce orden canónico sin repetidos. |
| AC-4 | `consolidar_preparar_should_create_a_deterministic_umbrella_without_overwriting` añade prosa humana y confirma que la segunda ejecución preserva exactamente el archivo. |
| AC-5 | El unit test pasa el cuerpo y triggers generados por `revisar_paraguas` sin faltas estructurales. |
| AC-6 | La integración cubre creación, selección duplicada, no-backend, ausencia de archivo/backups y no-sobrescritura. |

## Verificación ejecutada

- `cargo test preparar_paraguas_should`
- `cargo test consolidar_preparar_should`
- `cargo test consolidar_aplicar_should_take_the_merge_from_argv`
- `cargo clippy --all-targets -- -D warnings`

Todas verdes.
