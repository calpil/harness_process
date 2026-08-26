# Evidencia de implementación — Feature #53

Spec: `docs/spec-feature-53-check-no-se-cuelga-por-stdin.md` (approved)

## Cambios

- La única llamada de `harness_check.sh` a `commit_guard.sh` ahora redirige
  `</dev/null`; el guard no se modifica y por ello conserva stdin cuando un
  hook lo invoca directamente.
- Se aplica el mismo cambio textual a `templates/harness_check.sh`.
- `tests/commit_guard_stdin_check.sh` crea un sandbox con un servicio sucio y
  mantiene una tubería viva para demostrar que el check termina y sigue
  bloqueando; después verifica el payload directo del hook.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | El fixture deja stdin vivo con `yes`, sondea solo hasta 1 s y observa que el check termina. |
| AC-2 | El fixture ejecuta un servicio limpio con stdin cerrado y exige exit 0 antes del caso bloqueante. |
| AC-3 | El mismo fixture usa un servicio Git sucio y exige exit 2 + `Cambios sin commitear`. |
| AC-4 | `guard_directo_conserva_payload` entrega `stop_hook_active:true` y confirma que el guard distingue un payload vacío. |
| AC-5 | `cmp` exige paridad byte a byte entre fuente y plantilla; no se redirige stdin dentro del guard. |
| AC-6 | `bash tests/commit_guard_stdin_check.sh` cubre terminación, limpio/bloqueante y hook sin red ni espera larga. |

## Verificación ejecutada

- `bash -n harness_check.sh templates/harness_check.sh commit_guard.sh tests/commit_guard_stdin_check.sh`
- `bash tests/commit_guard_stdin_check.sh`

Todas verdes.
