# Evidencia de implementación — Feature #40

Spec: `docs/spec-feature-40-prd-sello-se-invalida-al-editar.md` (approved)

## Cambios

- `Aplicado:` ahora distingue sello presente de aplicación vigente: para cada
  bloque `cambio` comprueba que su texto literal `Despues:` siga presente en el
  documento real de su alcance.
- `prd propose` y `prd apply` quitan solamente el sello obsoleto y conservan
  los veredictos; el siguiente `apply` vuelve a pedir `--yes`.
- El gate de cierre falla cerrado para una propuesta sellada pero incomprobable
  o cuyo contenido propio fue retirado, sin castigar una edición ajena del PRD.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | `gate_should_validate_only_the_text_applied_by_this_proposal` acepta un `Despues:` todavía presente. |
| AC-2 | El mismo test reemplaza el fragmento y el gate responde que ya no está vigente; la integración `prd_propose_should_invalidate_a_seal_when_its_applied_text_is_removed` elimina el sello visible. |
| AC-3 | La prueba agrega una nota ajena al mismo PRD y conserva el sello de la propuesta, pues solo se revisa su `Despues:`. |
| AC-4 | La comprobación usa `contains(despues)` por bloque y la prueba conserva el fragmento frente a cambios en el resto del archivo. |
| AC-5 | Un sello heredado sin bloques comprobables se rechaza explícitamente en `gate_should_validate_only_the_text_applied_by_this_proposal`. |
| AC-6 | Los fixtures de CLI y los unit tests del gate cubren aplicación, edición ajena, reemplazo y formato antiguo, todos en `tempfile`. |

## Verificación ejecutada

- `cargo test docs_gate_should`
- `cargo test gate_should_be_off_by_default`
- `cargo test gate_should_validate_only`
- `cargo test prd_propose_should_invalidate`
- `cargo clippy --all-targets -- -D warnings`

Todas verdes.
