# Estado archivado - Feature #8: kimi_cli_backend
Cerrada: 2026-07-29T00:06:56Z - status=done - Kimi Code CLI como backend de primera clase: subagentes en .kimi-code/agents/, superficie AGENTS.md nativa, hook global blindado (backup + marcadores + idempotencia + rollback por doctor), gate de espejo extendido. AC-10 (pwsh) verificado estaticamente

---

# Feature #8: kimi_cli_backend

Estado: in_progress
Plan: docs/plan-feature-8-kimi-cli-backend.md
Spec: docs/spec-feature-8-kimi-cli-backend.md

Microservicios:
- harness

Evidencia:
- 2026-07-28: spec completado por el lider (AC-1..AC-12) con investigacion
  empirica de Kimi Code CLI v0.29.2 (AGENTS.md de proyecto SI se carga; hooks
  solo globales; Stop exit 2 bloquea; cwd del hook = proyecto). Plan con
  delegacion U0..U7. Spec en DRAFT: pendiente ritual de aprobacion del
  usuario + 3 decisiones PENDIENTES en Observaciones (reset del bloque
  global; deteccion/flag --no-kimi; tools del frontmatter).
- 2026-07-28T23:20:44Z Re-sincronizado con plan actualizado por otro agente (U0 cerrada: 3 decisiones registradas por Alan)
- 2026-07-28T23:42:28Z Feature #8 U1-U7 implementadas: build_kimi_agent + write_kimi_hooks (bloque global blindado) + gate espejo Kimi + superficies/README + smoke sh en verde + paridad ps1 + docs
- 2026-07-28T23:47:14Z Evidencia por AC-1..AC-12 escrita en docs/impl-8.md; cargo test 44+22/0, clippy limpio, smoke rc=0 con bloque Kimi, prueba negativa propia del gate, dogfood limpio, home real de Kimi intacto (181 bytes, 0 hooks)
