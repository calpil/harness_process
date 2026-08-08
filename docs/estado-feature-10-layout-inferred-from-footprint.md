# Estado archivado - Feature #10: layout_inferred_from_footprint
Cerrada: 2026-07-30T21:21:01Z - status=done - Inferencia de layout subdir por huella del padre cuando falta .harness_layout (tres casos excluyentes: subdir/AUSENTE/root), misma regla en los 4 scripts + espejos + paths.rs; bug reproducido ANTES y fix comprobado DESPUES con fixtures; smoke/clippy/test verdes (50+27); AC-11 estatica (sin pwsh). docs/impl-10.md y docs/review-10.md (approved). Instalaciones sin marker se reparan solas al actualizar.

---

# Feature #10: layout_inferred_from_footprint

Estado: in_progress
Plan: docs/plan-feature-10-layout-inferred-from-footprint.md
Spec: docs/spec-feature-10-layout-inferred-from-footprint.md

Microservicios:
- harness

Evidencia:
- 
- 2026-07-29T04:21:02Z Re-sincronizado con plan actualizado por otro agente (feature #10, U1..U5)
- 2026-07-30T21:19:05Z Evidencia reconstruida y verificada: bug reproducido ANTES (sin marker resuelve al arnes) y fix comprobado DESPUES (infiere subdir + aviso [i]); AC-3/AC-4/AC-6/AC-7 verificados con fixtures; 4 scripts identicos a templates (AC-9); smoke rc=0 con bloque marker-ausente y [Ok] previas intactas (AC-10); clippy -D warnings y cargo test 50+27 verdes (AC-13); AC-11 estatica (sin pwsh). docs/impl-10.md y docs/review-10.md (veredicto: approved) escritos; harness_check rc=0.
