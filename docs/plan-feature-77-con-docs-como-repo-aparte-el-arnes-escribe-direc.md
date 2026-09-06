# Plan - Feature #77: con docs/ como repo aparte, el arnes escribe directo en docs/ y no crea docs-wt

Estado: in_progress
Microservicios:
- harness_process (rust: paths/start/close)

## Alcance

Revertir la OBS-5 de la #72: el repo `docs/` aparte deja de tener worktree por
feature. Los documentos van a `<raiz>/docs/`, donde ya van el PRD, el SDD y el
sello de cierre.

## Peldano de huella

`Peldano elegido: 1 (extender lo que existe) porque el arreglo es QUITAR una
pieza (el worktree de docs) y cambiar una resolucion de ruta que ya existe.`

## Delegacion (implementer)

- D-1 (AC-1, AC-2): `commands/start.rs` — eliminar `preparar_docs` y la
  escritura de `docs_worktree`.
- D-2 (AC-1, AC-3, AC-5): `paths.rs::para_feature` — con docs aparte,
  `plans = <raiz>/docs`; el campo `docs_worktree` se ignora; con docs del repo
  principal, el worktree sigue mandando.
- D-3 (AC-4): `commands/close.rs` — quitar el commit y el aviso del worktree de
  docs.
- D-4 (AC-6, AC-7): tests — reescribir el de la #72 y agregar el de dos
  features en paralelo sobre el mismo `docs/`.

## Criterios de cierre (reviewer)

- Prueba del rojo: devolver el worktree de docs tiene que romper el test del AC-1.
- El test de la #72 no se borra: dice que regla codificaba y cual codifica ahora.

## Riesgos

- R-1: `para_feature` pasa a llamar a git (`repo_de_docs`) en cada resolucion.
  `from_root` ya lo hace (`worktree_actual`); es un costo que ya se pagaba.
- R-2: features de realestate con `docs_worktree` en el backlog. Se ignora el
  campo y pasan a `docs/`; sus artefactos se migran a mano (AC-9).

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por el usuario 2026-09-06): directo en `docs/`, sin worktree.
- OBS-2: la rama de integracion se pregunta antes de `close --status done --to`.

---
Cerrado: 2026-09-06T22:05:16Z - status=done - Con docs aparte los documentos van directo a docs/; se revierte la OBS-5 de la #72, que dejaba cada spec en una rama del repo docs que nadie mergeaba
