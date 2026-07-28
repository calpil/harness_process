# Estado archivado - Feature #7: harness_check_robustness
Cerrada: 2026-07-28T18:58:03Z - status=done - Gate de espejo roles->agentes (bloquea, read-only) + resolucion robusta de REPO_ROOT en 4 scripts, sus espejos y paths.rs; marker des-versionado. AC-11 (pwsh) verificado estaticamente

---

# Feature #7: harness_check_robustness

Estado: in_progress
Plan: docs/plan-feature-7-harness-check-robustness.md
Spec: docs/spec-feature-7-harness-check-robustness.md

Microservicios:
- harness

Evidencia:
- 2026-07-28 lider: spec completo (AC-1..AC-13, Estado: draft) y plan con
  delegacion U0..U8; 4 observaciones PENDIENTE DE DECISION esperan al usuario
  antes de implementar (ritual de aprobacion pendiente).
- 2026-07-28T18:11:34Z Re-sincronizado con plan actualizado por otro agente (U0 cerrada, 4 decisiones registradas); inicio U1
- 2026-07-28T18:25:09Z F7 U1-U5: guardrail checkout fuente en 4 scripts + espejos, gate de espejo roles/agentes + sub-gate templates en harness_check, regla en paths.rs con tests (44u+22i verdes, clippy limpio), marker des-versionado + gitignore + UPDATING (raiz y template)
- 2026-07-28T18:29:41Z F7 U6: smoke con check limpio post-install, stale inyectado en Claude/Gemini/Codex + drift templates detectados (block/warn/off), y checkout fuente simulado con resolucion local sin escrituras fuera del clon; setup_smoke.sh rc=0
- 2026-07-28T18:35:17Z F7 U7: paridad ps1 - bloques del gate de espejo y checkout fuente en tests/setup_smoke.ps1 (extraccion portada + check real via bash si existe) + fix here-strings rotos preexistentes (apertura $fakeCargo perdida en feature #2); revision estatica, sin pwsh en la maquina
- 2026-07-28T18:40:47Z F7 U8 + evidencia: docs (README/AGENTS/architecture) y docs/impl-7.md con evidencia AC-1..AC-13; verificacion final: cargo test 44u+22i ok, clippy -D warnings ok, setup_smoke.sh rc=0, dogfooding check limpio con [i] y AC-10 via checkout-index
