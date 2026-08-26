# Evidencia de implementación — Feature #38

Spec: `docs/spec-feature-38-prd-propose-texto-candidato.md` (approved)

## Cambios

- `rust/src/commands/prd.rs`: cada bloque recién sembrado contiene
  `Candidato despues:`. El texto usa rutas cambiadas de commits, cambios sin
  commitear y archivos nuevos de la feature, se acota a cuatro rutas y descarta
  los artefactos que escribe el propio arnés.
- El candidato no modifica `Veredicto:` ni toca el documento de destino; sin
  rutas atribuibles declara explícitamente que no hay candidato.
- `rust/tests/cli_basics.rs`: fixture Git con un archivo nuevo real, además de
  la cobertura existente de siembra e idempotencia.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | `prd_propose_should_seed_an_editable_candidate_from_uncommitted_paths` encuentra `rust/src/nuevo.rs` y conserva `Veredicto: PENDIENTE`. |
| AC-2 | `prd_propose_should_not_clobber_existing_verdicts` sigue verde; un bloque respondido no se regenera. |
| AC-3 | `prd_propose_should_seed_one_block_per_document` comprueba el texto explícito `sin candidato` fuera de Git. |
| AC-4 | La ruta de `prd apply` no cambió y el candidato no participa en `parsear` ni `planificar`; las pruebas de propuesta siguen ejecutando el ritual existente. |
| AC-5 | El test de propuesta solo crea `docs/prd-diff-1.md`; no muta PRD, SDD ni architecture. |
| AC-6 | Tests focalizados y unitarios verdes, sin red ni backend LLM. |

## Verificación ejecutada

- `cargo test prd_propose_should` — 4 pruebas de integración verdes.
- `cargo test candidato_de_rutas` — verde.
- `cargo test artefactos_del_arnes_no_entran` — verde.
- `cargo clippy --all-targets -- -D warnings` — verde.

`cargo fmt --check` global aún reporta diferencias preexistentes en numerosos
archivos no tocados; se ejecutó `rustfmt` únicamente sobre los dos archivos
modificados por esta feature.
