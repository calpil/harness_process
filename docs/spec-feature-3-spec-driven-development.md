# Spec - Feature #3: spec_driven_development

Estado: draft
Plan: docs/plan-feature-3-spec-driven-development.md
Constitution: docs/constitution.md

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como maintainer del harness, quiero que `harness_cli start` genere el spec
  (draft) ademas del plan de la feature, para arrancar cada feature con criterios
  de aceptacion explicitos antes de tocar codigo.
- P1: Como usuario (Alan), quiero aprobar el spec editando yo mismo
  `Estado: draft` -> `Estado: approved`, para que ninguna implementacion avance
  sin mi decision (los agentes tienen PROHIBIDO auto-aprobar).
- P1: Como implementer, quiero quedar bloqueado (advance / close --status done /
  harness_check.sh fallan con mensaje accionable) mientras el spec no este
  aprobado, para no implementar nada fuera de un spec acordado.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given una feature pending, When corro `harness_cli start --feature <id>`,
  Then se crea `docs/spec-feature-<id>-<slug>.md` en el `docs/` de la RAIZ
  (layout plano junto al plan, sin carpetas tipo Spec Kit `specs/NNN/`) con
  plantilla `Estado: draft`, recorridos de usuario priorizados (P1/P2,
  independientemente testeables), criterios de aceptacion AC-n en
  Given/When/Then, no funcionales (SLOs, seguridad, observabilidad), fuera de
  alcance y observaciones con el mismo protocolo de decision que el plan.
- AC-2: Given un spec firmado en la feature (`last_spec_sig` reusa la mecanica
  de `plan_signature()`: dict path/mtime/size/hash), When otro LLM edita el
  spec o el plan, Then `harness_cli check-plan` sale con exit 2 y su stdout
  distingue cual de los dos esta stale (plan vs spec).
- AC-3: Given la regla `require_spec_approved: true` en `feature_list.json` y
  un spec sin `Estado: approved`, When corro `advance`, `close --status done` o
  `harness_check.sh` (via el nuevo subcomando `check-spec`), Then bloquean con
  mensaje claro y accionable; Given la regla ausente o en false, Then el gate
  queda apagado (compatibilidad con instalaciones previas y features #1/#2).
- AC-4: Given un proyecto sin `docs/constitution.md`, When corro
  `setup_harness.sh` o `setup_harness.ps1`, Then se siembra
  `docs/constitution.md` (docs de la RAIZ) desde `templates/docs/constitution.md`
  SOLO si falta (nunca pisa el del usuario), queda referenciado por las
  superficies (CLAUDE/AGENTS/GEMINI/LLM) y los roles, y `harness_check.sh`
  verifica su existencia.
- AC-5: Given los roles leader/implementer/reviewer (en `roles/`,
  `templates/roles/` y los subagentes generados `.claude/agents` /
  `.codex/agents` / `.gemini/agents` de ambos instaladores), When un agente
  trabaja una feature, Then el implementer exige spec aprobado antes de
  implementar, cada item de Delegacion del plan cita su AC-n y el reviewer
  exige evidencia por AC en el veredicto.
- AC-6: Given el repo fuente, When corro `cargo test` y
  `cargo clippy -- -D warnings`, Then salen limpios e incluyen tests de la
  firma y del gate del spec; y `tests/setup_smoke.sh` verifica la siembra del
  spec (via `start` e2e) y de la constitution (incluida la semantica no-pisa).
- AC-7: Given la feature cerrada, When leo README.md, UPDATING.md
  (+ `templates/UPDATING.md`), AGENTS.md y `docs/architecture.md`, Then
  documentan el flujo SDD (start genera spec draft -> aprobacion del usuario ->
  implementacion con gate) y el opt-in para instalaciones existentes.

## No funcionales
- SLOs: el gate (`check-spec` y `spec_gate` en advance/close) resuelve en <1s
  en local y SIN red (solo filesystem + feature_list.json; el hub nunca
  participa del gate).
- Seguridad: solo el USUARIO aprueba specs (draft -> approved); los agentes no
  pueden auto-aprobar; con la regla activa el gate falla cerrado (spec
  ausente/draft/desconocido => bloqueo).
- Observabilidad: mensajes accionables (ruta del spec, estado actual, accion
  requerida) y exit codes estables 0/1/2 para hooks, roles y harness_check.sh.

## Fuera de alcance
- Carpetas por spec estilo Spec Kit (`specs/NNN/`): el layout es plano, en el
  `docs/` de la raiz junto a los planes.
- Aprobacion de specs por un LLM (auto-aprobacion o aprobacion delegada): la
  transicion draft -> approved es exclusiva del usuario.
- Migracion retroactiva de specs para las features #1/#2 (done): el gate solo
  actua sobre la feature activa y no exige firmas historicas.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- (ninguna)
