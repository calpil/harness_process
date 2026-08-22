# Plan - Feature #49: architecture_en_el_worktree_de_la_feature

Estado: in_progress
Microservicios:
- harness

## Alcance

Que `docs/architecture.md` se resuelva contra el `docs/` de la feature igual que
el PRD y el SDD. Una linea en `documentos::alcance()` mas el test que la
sostiene. No entra: tocar otras rutas ni mover el estado del arnes.

## Impacto entre microservicios

Un solo microservicio: `harness`. Sin worktree, `paths.plans` ES el `docs/` de
la raiz, asi que el modo clasico no cambia (AC-3).

## Consulta al grafo (graphify)

No hace falta: la auditoria por grep ya acoto el cambio a
`rust/src/documentos.rs:89`, el unico lugar de produccion que arma esa ruta
contra `repo_root`.

## Delegacion (implementer)

- D1 [AC-1, AC-2, AC-3]: en `documentos::alcance()`, resolver `architecture.md`
  con `paths.plans` en vez de `paths.repo_root`, conservando `docs/architecture.md`
  como etiqueta relativa del bloque.
- D2 [AC-4]: test que arma unas `HarnessPaths` con `plans` distinto de
  `repo_root` y exige que el documento salga del primero.
- D3 [AC-5, AC-6]: los cuatro comandos oficiales, y el cierre de esta feature
  como prueba real de que los tres documentos viajan con el merge.

## Criterios de cierre (reviewer)

- Evidencia por AC-n en `docs/impl-49.md`.
- `cargo test`, `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh` limpios.
- El propio cierre integrado a `main` sin dejar el arbol principal sucio.

## Riesgos

- R1: que algun consumidor espere la ruta absoluta contra la raiz. Mitigacion:
  `rel` no cambia (sigue siendo `docs/architecture.md`) y sin worktree las dos
  rutas son la misma.

## Observaciones (decisiones pendientes)

- OBS-1 [REGISTRADA]: la deuda vino de la feature #47 y la encontro la
  verificacion de cierre, no un test; por eso el AC-4.

---
Cerrado: 2026-08-22T12:11:57Z - status=done - architecture.md se resuelve contra el docs/ de la feature igual que el PRD y el SDD: el cambio viaja con el merge en vez de quedar suelto en el principal
