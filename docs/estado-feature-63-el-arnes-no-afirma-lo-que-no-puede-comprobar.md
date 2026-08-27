# Estado archivado - Feature #63: el_arnes_no_afirma_lo_que_no_puede_comprobar
Cerrada: 2026-08-27T19:41:21Z - status=done - El test del commit_guard vuelve a medir en macOS (timeout(1) no existe ahi: ahora elige entre timeout/gtimeout/perl alarm, se auto-prueba, y falla si no hay ninguno en vez de saltear en verde) y la prueba-del-rojo revierte las DOS defensas contra el cuelgue, no una. Y la ruta del estado archivado que imprime el cierre apunta a donde el archivo queda despues del merge, no al worktree que el propio cierre borro. 7/7 AC verdes.

---

# Feature #63: el_arnes_no_afirma_lo_que_no_puede_comprobar

Estado: in_progress
Plan: docs/plan-feature-63-el-arnes-no-afirma-lo-que-no-puede-comprobar.md
Spec: docs/spec-feature-63-el-arnes-no-afirma-lo-que-no-puede-comprobar.md

Microservicios:
- harness

Evidencia:
- 
