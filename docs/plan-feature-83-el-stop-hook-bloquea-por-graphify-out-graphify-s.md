# Plan - Feature #83: el Stop hook bloquea por graphify-out/.graphify_stale

Spec: docs/spec-feature-83-el-stop-hook-bloquea-por-graphify-out-graphify-s.md
(approved 2026-09-07).

## Alcance

Un bloque de `harness_check.sh` cambia de severidad (`[!]` + `sumar_fallo` ->
`[i]`), con su espejo en `templates/`, un test en fixture instalado enganchado
al smoke, y una seccion en `UPDATING.md`. Sin Rust, sin tocar el hook ni el
rebuild.

## Peldano de huella

Peldano 1: cambiar un mensaje y una severidad en un script existente. Ninguna
opcion nueva, ninguna regla nueva.

## Delegacion (implementer)

- D-1 (AC-1, AC-2, AC-4; PRIMERO): `tests/graphify_stale_check.sh` sobre un
  fixture instalado (como `tests/leccion_tope_check.sh`): sin marcador el check
  pasa limpio y no lo menciona; con el marcador, `[i]` que nombra al
  post-commit, a `/graphify --update` y dice que no bloquea, y exit 0. Correrlo
  contra el check de HEAD: tiene que caer por el `[!]` (leer el mensaje, no el
  rc).
- D-2 (AC-1..AC-3): el bloque en `harness_check.sh`, copiado a
  `templates/harness_check.sh` (el instalador copia desde ahi; `cmp`).
- D-3 (AC-4): enganche en `tests/setup_smoke.sh`, despues del bloque de la #80.
- D-4 (AC-5): seccion en `UPDATING.md` y `templates/UPDATING.md` (`cmp`).
- D-5: `tests/stop_hook_check.sh` y `tests/parity_check.sh` siguen verdes; el
  smoke completo verde.

## Criterios de cierre (reviewer)

- El test cae contra HEAD por el `[!]` del marcador, no por una precondicion.
- Con el fix, el fixture con marcador da rc 0 y NINGUN otro `[!]` cambia.

## Riesgos

- R-1: un fixture recien instalado tiene `progress/current.md` vacio, que es
  `[!]` por si mismo; el test le da contenido antes de medir, para que el rc
  refleje solo el marcador.

## Observaciones (decisiones pendientes)

- OBS-1: la rama de integracion se pregunta antes de `close --status done --to`.
- OBS-2 (propuesta: no): un marcador viejo no vuelve a ser `[!]`.

---
Cerrado: 2026-09-07T01:49:56Z - status=done - 
