# Evidencia de implementacion - Feature #49: architecture_en_el_worktree_de_la_feature

Spec: `docs/spec-feature-49-architecture-en-el-worktree-de-la-feature.md` (approved, 6 AC)
Plan: `docs/plan-feature-49-architecture-en-el-worktree-de-la-feature.md`

## Que se cambio

Una linea en `rust/src/documentos.rs`, en `alcance()`:

```
antes:   let arch = paths.repo_root.join(ARCHITECTURE);
despues: let arch = paths.plans.join("architecture.md");
```

`paths.plans` es el `docs/` de la feature — el mismo del que ya salian el PRD y
el SDD desde la feature #47 —, asi que ahora los tres documentos del alcance se
leen y se escriben en el mismo lugar. Sin worktree, `plans` ES el `docs/` de la
raiz: el modo clasico no cambia.

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 el bloque apunta al worktree | OK | Unit `architecture_should_come_from_the_feature_docs_not_the_repo_root`: con `plans` en otro directorio, el documento sale de ahi — y encima hay un `architecture.md` VIEJO en la raiz que NO gana |
| AC-2 `apply` escribe en el worktree | OK | `prd apply` escribe en `Documento.path`, que es lo que verifica AC-1. Comprobado ademas en el cierre real de esta feature (AC-6) |
| AC-3 sin worktree, cero regresion | OK | Unit `architecture_should_fall_back_to_the_root_without_worktree` (la ruta vuelve a ser `repo_root/docs/architecture.md`) y los tests previos del modulo, que siguen verdes sin tocarlos |
| AC-4 test anti-regresion | OK | El test de AC-1 falla si alguien vuelve a armar la ruta contra `repo_root`: el `architecture.md` de la raiz esta sembrado a proposito para que esa version pase el `is_file()` y elija el archivo equivocado |
| AC-5 comandos oficiales | OK | `cargo test` en verde, `clippy --all-targets -- -D warnings` limpio, `setup_smoke.sh` exit 0 y `harness_check.sh` limpio |
| AC-6 el propio cierre | OK | Ver abajo |

## Detalle: por que el test detecta la regresion

El fixture siembra DOS `architecture.md`: uno en el `docs/` de la feature (con
el contenido bueno) y otro en el `docs/` de la raiz. La version vieja del codigo
encontraba el de la raiz y pasaba `is_file()` sin quejarse — por eso la deuda
sobrevivio a 515 tests. Ahora el assert compara la ruta exacta, asi que elegir
la raiz falla.

## Nota

`Documento.rel` sigue siendo `docs/architecture.md`: es la etiqueta con la que
el bloque se nombra en `docs/prd-diff-<id>.md`, no el destino. Ningun formato
cambio.
