# Review - Feature #6: interactive_spec_approval

Spec: docs/spec-feature-6-interactive-spec-approval.md (Estado: approved)
Plan: docs/plan-feature-6-interactive-spec-approval.md
Impl: docs/impl-6.md

## Veredicto global

**approved**, con una salvedad heredada de las features #4 y #5: AC-11 (paridad
Windows) queda verificado por revision estatica, no por ejecucion, porque no hay
`pwsh` ni `powershell` en esta maquina (verificado con `command -v`).

## Estado por AC

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | `spec::approve_spec` + test unitario que compara el archivo completo byte a byte, y test de integracion sobre el binario |
| AC-2 | cubierto | `approve_spec_plus_resign_should_leave_the_spec_fresh` + bloque del smoke que corre `check-spec` inmediatamente despues de aprobar (rc=0, sin `advance`) |
| AC-3 | cubierto | `approve_spec_should_refuse_without_explicit_user_confirmation` (rc=2, spec intacto en draft) + smoke; verificado tambien a mano en este repo |
| AC-4 | cubierto | 3 verificaciones independientes (unitaria, integracion, smoke con `grep -c` == 1) |
| AC-5 | cubierto | `approve_spec_should_exit_one_without_active_feature_and_two_without_spec` (1 y 2 segun el caso, con mensaje accionable) |
| AC-6 | cubierto | sello con nota (unitario) + linea en `progress/history.md` (integracion); confirmado en la bitacora real: `approve-spec feature #6 estado=approved nota=...` |
| AC-7 | cubierto | los 4 roles y sus dos juegos de espejos describen el ritual (leer -> mostrar -> abrir editor -> preguntar -> registrar); no queda la instruccion de editar `Estado:` a mano |
| AC-8 | cubierto | Articulo 2 reescrito en la constitution de raiz y en la plantilla; solo se toco ese articulo |
| AC-9 | cubierto | `spec_gate`, `check-spec`, `start` y `harness_check.sh` instruyen el flujo nuevo; los tests del gate exigen el texto nuevo |
| AC-10 | cubierto | el smoke valida sobre una instalacion REAL que la constitution y `harness_check.sh` sembrados contienen `approve-spec` y no `auto-aprobar` |
| AC-11 | **parcial** | `setup_harness.ps1` y `tests/setup_smoke.ps1` portados y revisados estaticamente; SIN ejecutar (no hay PowerShell en la maquina) |
| AC-12 | cubierto | `cargo test` 40+19 en verde, `cargo clippy --all-targets -- -D warnings` rc=0, `bash tests/setup_smoke.sh` rc=0 con la linea `[Ok] approve-spec: ...` |
| AC-13 | cubierto | `approve-spec` documentado en README, UPDATING (raiz + template), AGENTS.md, CHECKPOINTS.md (+ template) y architecture.md |

## Trazabilidad y constitution

- Cada unidad de la Delegacion (U1-U9) cita sus AC y aparece en `impl-6.md` con
  evidencia: **Articulo 3 cumplido**.
- **Articulo 1**: tests cercanos al codigo tocado (5 unitarios nuevos en
  `spec.rs`, 5 de integracion en `cli_basics.rs`, bloque nuevo en el smoke) y los
  tres comandos oficiales de `docs/verification.md` en verde.
- **Articulo 2**: el spec de esta feature se aprobo con confirmacion explicita
  del usuario en el chat, previa lectura (mostrado en el chat + abierto en su
  editor). El propio cambio refuerza el articulo: sin `--yes` el comando se
  niega con exit 2.
- **Articulo 4**: exit codes 0/1/2 estables y mensajes accionables (los tres
  pasos del ritual); sin secretos.
- **Articulo 5**: las cuatro decisiones del usuario (quien registra, como se
  muestra el spec, entrega como feature #6, flag `--yes`) estan registradas en
  las Observaciones del spec y del plan.
- **Articulo 6**: sin dependencias nuevas en `rust/Cargo.toml`; raiz y
  `templates/` espejados (verificado con `diff` para roles, CHECKPOINTS y
  harness_check.sh); feature backend-agnostica (ningun paso depende de una CLI
  de proveedor).

## Impacto y gates

- Microservicio unico `harness`; sin contratos compartidos. `graph impacto` no
  se pudo ejecutar contra el hub (PostgreSQL inalcanzable en esta maquina: los
  comandos degradan best-effort con el aviso de credenciales). El impacto real
  era de superficie y se verifico por `diff` raiz/templates y por el smoke.
- `graphify query "flujo de aprobacion del spec en el harness"` ejecutado: 14
  nodos, confirmando que la superficie del flujo vive en `roles/README.md` y su
  espejo (ambos actualizados).
- `bash harness_check.sh` limpio: `[plan] #6 fresco`, `[spec] #6 approved
  (fresco)`, `[Ok] Harness Check limpio.`

## Hallazgos (no bloquean el cierre)

1. **`.claude/agents/*.md` estaban stale desde la feature #3.** Quedaron al dia
   al regenerar los espejos. Recomendacion: chequeo automatico de espejo
   `roles/` -> `.claude/agents/` en una feature futura.
2. **El spec de la feature #5 fue aprobado a mano por el usuario el 2026-07-24
   18:21:38** (mtime confirmado), lo que disparo la falsa alarma que origino
   esta feature. Ese spec sigue `approved` pero con firma vieja: al retomar la
   #5 basta `harness_cli approve-spec --yes` para re-firmarlo (el comando es
   idempotente y no duplica el sello).
3. **Residuos en `$HOME/docs`** (`plan-feature-1`, `plan-feature-2`) por el
   footgun del layout del checkout fuente. Fuera de alcance; espera decision del
   usuario.
4. **Sin commit**: el arbol queda modificado a proposito; el commit es decision
   del usuario.
