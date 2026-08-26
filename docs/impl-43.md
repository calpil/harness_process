# Evidencia de implementación — Feature #43

Spec: `docs/spec-feature-43-consolidar-check-sin-cuota.md` (approved)

## Cambios

- `tests/consolidar_check.sh` compila el binario del worktree si falta y usa un
  CLI falso generado dentro de cada sandbox como comportamiento por defecto.
- Los modos locales cubren propuesta, descarte, JSON malformado, fallo del
  falso, fusión bajo paraguas y no-escritura; siempre sobrescriben un
  `HARNESS_CONSOLIDAR_CMD` heredado.
- `--real backend-real` es la única ruta externa. Está documentada en el propio
  script y en `docs/verification.md`; exige explícitamente `claude` o `kimi`
  autenticado y no se ejecutó en esta validación local.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | `bash tests/consolidar_check.sh` terminó con cinco modos locales verdes y ningún backend real. |
| AC-2 | Sus modos `propuesta`, `descarte`, `error`, `paraguas` y `no-toca-nada` fijan respuestas/control observable. |
| AC-3 | `--real` queda documentado; `bash tests/consolidar_check.sh --real no-toca-nada` rechaza el modo y muestra los prerequisitos sin activar integración. |
| AC-4 | `modo_propuesta` hereda deliberadamente `HARNESS_CONSOLIDAR_CMD=/bin/false` y demuestra que el falso del sandbox prevalece. |
| AC-5 | `modo_error` cubre JSON malformado y salida 7 del falso, exige su diagnóstico local y no configura fallback. |
| AC-6 | La suite completa corre sin secretos ni red; `cargo test consolidar_` conserva 27 pruebas de invariantes y `clippy` queda verde. |

## Verificación ejecutada

- `bash -n tests/consolidar_check.sh`
- `bash tests/consolidar_check.sh`
- `cargo test consolidar_` — 15 unitarias + 12 de CLI verdes.
- `cargo clippy --all-targets -- -D warnings`

No se ejecutó `--real`: requiere interacción/cuota externa y el cambio exige
que esa intención sea separada.
