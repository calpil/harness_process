# Review - Feature #3: spec_driven_development

Veredicto: approved (implementación) — cierre BLOQUEADO por una acción humana
pendiente (aprobación del spec por el usuario). Ver "Gate de cierre".

Revisión reforzada con un pase adversarial multi-agente (6 dimensiones ×
verificación por refutación): 3 hallazgos CONFIRMED corregidos en U6, 2 REFUTED
descartados, dimensión shell-gate re-revisada a mano sin hallazgos.

## Veredicto por criterio de aceptación

- **AC-1 (spec en start, layout plano)** — OK. `start` siembra
  `docs/spec-feature-<id>-<slug>.md` junto al plan (`rust/src/spec.rs`
  `spec_path`/`write_spec`, integrado en `commands/start.rs`). Sin carpetas
  `specs/NNN/`. Verificado por el E2E del smoke (`add`+`start` genera el spec en
  `Estado: draft`).
- **AC-2 (firma + vigilancia multi-LLM)** — OK. `last_spec_sig` reusa
  `plan::plan_signature` (mismo orden de claves path/mtime/size/hash, sin duplicar
  hashing). `check-plan` vigila plan Y spec; `is_spec_stale` es false sin firma
  previa (compat features #1/#2). Tests unit + integración cubren edición por otro
  agente => exit 2.
- **AC-3 (gate duro)** — OK, reforzado en U6. Con `require_spec_approved: true` y
  spec != approved, `advance` y `close --status done` bloquean; `check-spec` da
  exit 2; `harness_check.sh` falla (bloque check-spec). Gate fail-closed: Missing /
  Draft / Other bloquean, solo Approved abre (test nuevo `..._unrecognized_estado_...`).
  **Fix U6 (HIGH):** el stop-hook de PowerShell corría solo `check-plan` y no
  aplicaba el gate en Windows; ahora corre `check-plan` + `check-spec` con `throw`
  en `$LASTEXITCODE == 2`.
- **AC-4 (constitution)** — OK, reforzado en U6. `docs/constitution.md` sembrado
  if-missing por ambos instaladores, fuera de las listas generated/backup,
  verificado por `harness_check.sh`. **Fix U6 (MEDIUM):** `--reset` ya no barre
  `docs/` entero (borraba la constitution del usuario en layout root); elimina solo
  los docs generados. Cubierto por assert nuevo en el smoke (sentinel sobrevive
  reset + reinstall).
- **AC-5 (roles con trazabilidad)** — OK. leader/implementer/reviewer exigen spec
  aprobado y trazabilidad AC-n; espejado `templates/roles/*` ↔ `roles/*` intacto
  (solo `__HREL__` difiere), superficies de ambos instaladores con paso
  `check-spec`.
- **AC-6 (tests)** — OK. `cargo test` 35 unit + 14 integración, `cargo clippy
  -D warnings` limpio, `bash tests/setup_smoke.sh` rc 0 (siembra root/subdir +
  no-pisa + E2E del gate + reset preserva constitution).
- **AC-7 (docs)** — OK, reforzado en U6. README/UPDATING(×2)/AGENTS/architecture con
  el flujo SDD; **Fix U6 (LOW):** eliminada la deuda viva del feature #2 (fallback
  Python inexistente + comando `parity_smoke.sh` roto).

## Hallazgos REFUTED (correctamente descartados)

- "check-plan exit 2 confunde plan vs spec y los consumidores tragan stdout":
  FALSO (`harness_status.sh` usa `2>/dev/null`, stdout fluye; mensajes distinguen).
- "SpecState::Other nunca se prueba bajo regla activa": era una brecha de cobertura
  real pero no un defecto de comportamiento (el código ya es fail-closed); brecha
  cerrada igual con el test nuevo.

## Riesgo residual

- **Windows real**: los fixes del hook ps1 y del reset ps1 se validaron por
  revisión manual + paridad conceptual (sin `pwsh` en este entorno). Re-ejecutar el
  smoke ps1 en Windows/CI.
- **Bootstrapping**: el `feature_list.json` VIVO tiene `require_spec_approved`
  APAGADA a propósito; el path regla=true se ejercita en el fixture del smoke. Se
  activa al cierre, tras la aprobación del usuario.
- **Instalaciones existentes**: el seed es if-missing, así que no reciben la regla
  automáticamente (opt-in documentado en UPDATING).

## Gate de cierre (acción humana pendiente — no la ejecuta ningún agente)

Por diseño (Artículo 2 de la constitution: solo el USUARIO aprueba), el cierre
requiere, EN ORDEN:

1. El usuario edita `docs/spec-feature-3-spec-driven-development.md`:
   `Estado: draft` -> `Estado: approved`.
2. Activar la regla en el `feature_list.json` VIVO: `"require_spec_approved": true`.
3. `sh harness_cli advance --nota "Spec aprobado; regla activada"` (re-firma) y
   confirmar `sh harness_cli check-spec` rc 0 con la regla activa.
4. `sh harness_cli close --feature 3 --status done` (ahora el gate pasa).
5. Versionar el plan del líder (`docs/plan-feature-3-*.md`, hoy untracked).

## Criterios checklist (del plan)

- [x] Alcance AC-1..AC-7 implementado y verificado.
- [x] cargo test (35+14) + clippy -D warnings limpios.
- [x] tests/setup_smoke.sh rc 0 (incluye gate E2E + reset preserva constitution).
- [x] harness_check.sh rc 0.
- [x] Espejado templates/ ↔ raíz intacto (harness_check, CHECKPOINTS, roles).
- [x] Trazabilidad: impl-3 mapea cambios -> AC-n; review-3 da veredicto por AC.
- [x] Revisión adversarial: 3 CONFIRMED corregidos, 2 REFUTED, shell-gate limpio.
- [x] Commits Conventional sin trailers de IA (commit_guard limpio).
- [ ] Spec aprobado por el usuario (Estado: approved) — PENDIENTE.
- [ ] Regla activada + `close --status done` — PENDIENTE (tras aprobación).

Aprobado en calidad de implementación. Cierre en espera de la aprobación del spec.
