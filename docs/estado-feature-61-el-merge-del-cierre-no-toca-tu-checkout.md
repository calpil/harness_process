# Estado archivado - Feature #61: el_merge_del_cierre_no_toca_tu_checkout
Cerrada: 2026-08-27T18:35:58Z - status=done - El merge del cierre corre siempre en un worktree temporal --detach: el checkout del usuario nunca participa. La rama destino avanza con reset --keep, que conserva lo que tenga sin commitear. La colision irreductible se detecta ANTES de commitear o mergear y nombra los archivos. 7/7 AC ejecutables verdes.

---

# Feature #61: el_merge_del_cierre_no_toca_tu_checkout

Estado: in_progress
Plan: docs/plan-feature-61-el-merge-del-cierre-no-toca-tu-checkout.md
Spec: docs/spec-feature-61-el-merge-del-cierre-no-toca-tu-checkout.md

Microservicios:
- harness

Evidencia:
- 
