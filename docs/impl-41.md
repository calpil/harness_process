# Evidencia de implementación — Feature #41

Spec: `docs/spec-feature-41-consolidar-usa-relacionadas.md` (approved)

## Cambios

- `lecciones consolidar` calcula una señal local de `relacionadas` antes de
  consultar un backend: una pareja mutua se informa aun sin cuota, red o CLI.
- Las referencias unilaterales, rotas o fuera del catálogo activo y válido se
  ignoran con diagnóstico; el frontmatter se trata como nombre, nunca como ruta.
- Los pares se comparan con llave canónica: si coinciden una propuesta por
  triggers/LLM y una relación local, la salida queda una sola vez y concatena
  ambas evidencias.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | `relacionadas_mutuas_should_produce_a_local_candidate_without_shared_triggers` usa triggers disjuntos y obtiene A-B. |
| AC-2 | `relacionadas_unilaterales_or_unknown_should_not_invent_candidates` no crea candidato y conserva diagnósticos de unilateral y desconocida. |
| AC-3 | `related_and_trigger_candidates_should_merge_and_keep_both_reasons` mezcla B-A por triggers/LLM con A-B relacionada y deja una sola candidata con ambas razones. |
| AC-4 | La señal solo recibe lecciones activas/válidas; las demás se diagnostican como no elegibles y `validar` sigue descartando pinneadas. |
| AC-5 | Los motivos imprimen los dos nombres y la declaración mutua exacta; el prompt sigue omitiendo `relacionadas` y cuerpos. |
| AC-6 | `consolidar_should_report_mutual_relacionadas_without_a_backend` cubre el recorrido CLI sin red/backend y comprueba que no escribe ni genera backup. |

## Verificación ejecutada

- `cargo test relacionadas_`
- `cargo test related_and_trigger`
- `cargo test consolidar_should_report_mutual_relacionadas_without_a_backend`
- `cargo test consolidar_without_aplicar_should_not_touch_anything`
- `cargo clippy --all-targets -- -D warnings`

Todas verdes.
