# Impl - Feature #6: interactive_spec_approval

Spec: docs/spec-feature-6-interactive-spec-approval.md (Estado: approved)
Plan: docs/plan-feature-6-interactive-spec-approval.md

## Que cambio

Aprobar un spec dejo de ser "edita este Markdown a mano" y paso a ser un ritual
conversacional con rastro auditable:

```
LIDER completa el spec
   -> lee el spec entero
   -> lo MUESTRA al usuario (contenido en el chat + abierto en su editor)
   -> le PREGUNTA si lo aprueba
   -> con su SI: harness_cli approve-spec --yes --nota "<como aprobo>"
        - escribe `Estado: approved`
        - inserta el sello `Aprobado: <stamp> por USUARIO (confirmacion explicita)`
        - RE-FIRMA last_spec_sig  <- mata la falsa alarma multi-LLM
        - registra la linea en progress/history.md
   -> sin --yes: exit 2, el spec no se toca
```

El bug que motivo la feature: `check_spec` valida frescura ANTES que estado
(`rust/src/commands/check_spec.rs:22`), asi que la aprobacion manual del propio
usuario cambiaba el hash del spec y salia reportada como "SPEC ACTUALIZADO POR
OTRO LLM", obligando a un `advance` para resincronizar. Reproducido en vivo
sobre la feature #5 antes de empezar.

## Unidades

| Unidad | AC | Archivos |
| --- | --- | --- |
| U1 nucleo | AC-1, AC-4, AC-6 | `rust/src/spec.rs` (`ApprovalOutcome`, `approval_stamp_line`, `approve_spec`) |
| U2 comando | AC-1, AC-3, AC-5, AC-6 | `rust/src/commands/approve_spec.rs` (nuevo), `rust/src/commands/mod.rs`, `rust/src/cli.rs` |
| U3 re-firma | AC-2 | `rust/src/commands/approve_spec.rs` (`update_spec_sig` + `save_features`) |
| U4 gates | AC-9 | `rust/src/spec.rs` (`spec_gate`), `rust/src/commands/check_spec.rs`, `rust/src/commands/start.rs`, `harness_check.sh`, `templates/harness_check.sh` |
| U5 constitution | AC-8 | `docs/constitution.md`, `templates/docs/constitution.md` (solo Articulo 2) |
| U6 roles | AC-7 | `roles/{leader,implementer,reviewer,README}.md`, `templates/roles/*`, `.claude/agents/*` |
| U7 instaladores | AC-10, AC-11 | `setup_harness.sh`, `setup_harness.ps1` |
| U8 tests | AC-12 | `rust/src/spec.rs` (5 tests), `rust/tests/cli_basics.rs` (5 tests), `tests/setup_smoke.sh`, `tests/setup_smoke.ps1` |
| U9 docs | AC-13 | `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `AGENTS.md`, `CHECKPOINTS.md`, `templates/CHECKPOINTS.md`, `docs/architecture.md` |

## Evidencia por AC

- **AC-1** (registra la aprobacion): `spec::approve_spec` reescribe la PRIMERA
  linea `Estado:` de la ventana de 10 lineas preservando su indentacion e
  inserta el sello debajo. Tests: `approve_spec_should_write_approved_with_stamp_preserving_indent`
  (compara el archivo completo, byte a byte) y el de integracion
  `approve_spec_should_register_approval_and_leave_check_spec_clean`.
- **AC-2** (sin falsa alarma): el comando llama `update_spec_sig` + persiste
  `feature_list.json`. Tests: `approve_spec_plus_resign_should_leave_the_spec_fresh`
  (unitario: aprobar -> stale -> re-firmar -> fresco) y, end to end, el bloque
  nuevo de `tests/setup_smoke.sh`, que corre `check-spec` INMEDIATAMENTE despues
  de aprobar (sin `advance` de por medio) y exige rc=0.
- **AC-3** (barrera): sin `--yes` el comando imprime el `[GATE]` con los 3 pasos
  del ritual y sale 2 sin tocar el archivo. Tests:
  `approve_spec_should_refuse_without_explicit_user_confirmation` (verifica
  ademas que el spec sigue en draft y sin sello) y el smoke sh.
  La validacion es propia (no `required` de clap) para dar mensaje accionable.
- **AC-4** (idempotencia): `approve_spec` retorna `AlreadyApproved` y no
  reescribe. Tests: `approve_spec_should_be_idempotent_without_duplicating_the_stamp`
  (compara contenido antes/despues y cuenta 1 sello), `approve_spec_should_be_idempotent`
  (integracion) y el smoke (`grep -c '^Aprobado: '` == 1).
- **AC-5** (exit codes): 1 sin feature `in_progress` (via `active_feature_index`),
  2 con spec ausente y mensaje que indica `start --feature <id>`. Test:
  `approve_spec_should_exit_one_without_active_feature_and_two_without_spec`.
- **AC-6** (nota + bitacora): la nota entra al sello (`approval_stamp_line`, con
  trim) y `log()` escribe `approve-spec feature #<id> estado=approved nota=<...>`
  en `progress/history.md`. Tests: `approve_spec_should_record_nota_in_the_stamp`
  y el assert sobre `history.md` en el test de integracion.
- **AC-7** (roles): `roles/leader.md` gana el paso 5.1 con el ritual completo
  (leer -> mostrar en chat -> abrir editor -> preguntar -> registrar);
  `roles/implementer.md` reescribe el paso 0.2 con los mismos 4 pasos;
  `roles/reviewer.md` ahora exige verificar el SELLO y la linea de `history.md`;
  `roles/README.md` describe el ritual bajo el diagrama. Espejos regenerados
  desde la raiz (`templates/roles/` con `__HREL__`, `.claude/agents/` con su
  frontmatter). Ya no queda la instruccion de editar `Estado:` a mano.
- **AC-8** (constitution): Articulo 2 reescrito en `docs/constitution.md` y
  `templates/docs/constitution.md`. Dice que la DECISION es exclusiva del
  usuario, que el agente muestra/pregunta/registra con `approve-spec --yes`, y
  que esta PROHIBIDO aprobar sin ese si o editar `Estado:` para saltear el flujo.
  Se toco SOLO el Articulo 2 (el resto es documento del usuario).
- **AC-9** (mensajes de gates): `spec_gate` (usado por `advance` y
  `close --status done`), `check-spec`, la salida de `start` y `harness_check.sh`
  instruyen los 3 pasos del ritual. Los tests del gate se actualizaron para
  exigir el texto nuevo (`Mostrale el spec al USUARIO`, `approve-spec --yes`).
- **AC-10** (superficie sembrada): bloques 0.2 (corto y largo) del heredoc de
  superficie y el resumen final de `setup_harness.sh` mencionan `approve-spec`.
  El smoke verifica sobre la instalacion REAL que `docs/constitution.md` y
  `harness_check.sh` sembrados contienen `approve-spec` y NO contienen
  `auto-aprobar`.
- **AC-11** (paridad Windows): `setup_harness.ps1` actualizado en su bloque de
  superficie (en ingles) y en el `throw` del gate de spec.
  `tests/setup_smoke.ps1` gana los asserts de superficie equivalentes
  (constitution, harness_check.sh, roles/implementer.md).
  **PENDIENTE DE EJECUCION**: no hay `pwsh` ni `powershell` en esta maquina
  (verificado). Revision estatica unicamente, igual que el AC-7 de la feature #4
  y el AC-6 de la #5.
- **AC-12** (tests): `cargo test` 40 unit + 19 integracion en verde;
  `cargo clippy --all-targets -- -D warnings` exit 0; `bash tests/setup_smoke.sh`
  exit 0 con la linea nueva
  `[Ok] approve-spec: exige --yes, sella la aprobacion del usuario, re-firma (check-spec limpio) y es idempotente.`
- **AC-13** (docs): `README.md` (flujo SDD), `UPDATING.md` + `templates/UPDATING.md`
  (seccion nueva "Aprobación interactiva del spec (`approve-spec`)" que explica
  el bug de la falsa alarma y el rescate de specs aprobados a mano),
  `AGENTS.md` (orden de trabajo paso 2 + lista de archivos), `CHECKPOINTS.md` +
  su template, y `docs/architecture.md` (paso 3 del flujo SDD).

## Decisiones tomadas

- **Flag `--yes`** (decision del usuario, 2026-07-24): se descarto
  `--confirmado-por-usuario`. Registrado en spec y plan.
- **El binario NO abre el editor**: abrir el spec es responsabilidad del AGENTE
  por shell (`open`/`xdg-open`/`start`/`code`), declarado en los roles. Mantiene
  el binario sin lanzar procesos externos (fuera de alcance del spec).
- **Hub best-effort, sin graphify**: `approve-spec` usa `hub_register` (no
  `update_memories`), asi que no dispara refresh sincrono de graphify. Cumple el
  SLO del spec: la aprobacion no depende de la red para completarse.
- **Re-firma tambien cuando ya estaba aprobado**: cubre el caso del usuario que
  ya habia editado `Estado: approved` a mano y quedo con la falsa alarma
  pendiente. Test dedicado: `approve_spec_should_resign_a_spec_approved_by_hand`.

## Hallazgos colaterales

- **`.claude/agents/*.md` estaban STALE desde la feature #3**: conservaban el
  protocolo anterior al SDD (sin spec, sin AC-n, sin constitution). Al regenerar
  los espejos quedaron al dia. Vale la pena un check automatico de espejo
  raiz -> `.claude/agents/` en una feature futura.
- **Footgun del checkout fuente**: con `.harness_layout=subdir`, `repo_root` es
  `/Users/alan`, asi que todo comando del arnes en este repo se corrio con
  `HARNESS_REPO_ROOT=/Users/alan/harness_process`. Sin eso, `check-spec` reporta
  el spec como "ausente" y los comandos escriben en `$HOME/docs`. Hay residuos
  viejos ahi (`plan-feature-1`, `plan-feature-2`) que NO se tocaron (decision
  pendiente del usuario, registrada en el plan).

## Riesgos pendientes para el reviewer

- AC-11 sin ejecucion real en Windows (ver arriba).
- La feature #5 quedo aparcada como `pending`: su spec sigue en draft y ahora
  puede aprobarse con el flujo nuevo.
- El commit queda a criterio del usuario (no se commiteo nada).
