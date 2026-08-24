# Evidencia de implementación — Feature #39

Spec: `docs/spec-feature-39-prd-senales-mas-alla-del-nombre.md` (approved)

## Cambios

- Las señales de `prd propose` combinan nombre de feature, palabras específicas
  del spec y módulos/rutas del diff que también alimentan el candidato #38.
- Cada hallazgo informa archivo, línea, fuente (`nombre`, `spec` o `módulo`) y
  término; se ordenan, deduplican y acotan a tres.
- Un filtro de ruido evita que palabras estructurales como `arquitectura`,
  `feature` o `documento` se vuelvan falsos presentes.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | `prd_propose_should_find_a_document_by_terms_in_its_spec_not_only_its_name` detecta `facturacion` aun cuando el PRD no dice `demo`. |
| AC-2 | Los documentos sin coincidencias muestran `sin señales de nombre, spec o módulo`. |
| AC-3 | `palabras_significativas` normaliza, deduplica y filtra ruido; la suite conserva el caso de nombre `demo`. |
| AC-4 | Las señales se ordenan, deduplican y se limitan a tres con excedente explícito. |
| AC-5 | El test exige que el hallazgo cite la línea y la fuente textual; no hay citas generadas fuera del documento. |
| AC-6 | Cinco pruebas focalizadas de `prd propose` y clippy corrieron sin red ni LLM. |

## Verificación ejecutada

- `cargo test prd_propose_should` — 5 integración verdes.
- `cargo clippy --all-targets -- -D warnings` — verde.
