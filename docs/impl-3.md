# Implementacion - Feature #3: spec_driven_development

Evidencia por unidad (plan: `docs/plan-feature-3-spec-driven-development.md`).
Cada cambio cita el AC-n del spec `docs/spec-feature-3-spec-driven-development.md`.

## U0 - Spec retroactivo (AC-1, AC-5)

Archivos:

- `docs/spec-feature-3-spec-driven-development.md` (NUEVO): plantilla del plan
  rellenada para esta feature. `Estado: draft` (PROHIBIDO aprobarlo un agente),
  recorridos P1 (maintainer obtiene spec+plan en start; usuario aprueba;
  implementer bloqueado sin aprobacion), AC-1..AC-7 en Given/When/Then desde el
  Alcance del plan, no funcionales (gate <1s local sin red, mensajes
  accionables, exit codes estables 0/1/2), fuera de alcance (no `specs/NNN/`,
  no aprobacion por LLM, no migracion retroactiva #1/#2), observaciones:
  (ninguna).

Decisiones:

- La regla `require_spec_approved` NO se activo en el feature_list.json vivo:
  eso ocurre tras la aprobacion de Alan (bootstrapping decidido en el plan).
- El commit U0 incluye SOLO el spec (el plan del lider quedo untracked; lo
  versiona el orquestador/reviewer al cierre).

Commit: `2bb2ce8` `docs(spec): spec retroactivo feature #3 (SDD, Estado: draft)`.

**Pendiente de Alan**: aprobar el spec editando `Estado: draft` ->
`Estado: approved` (ningun agente puede hacerlo).

## U1 - Nucleo Rust (AC-1, AC-2, AC-3, AC-6)

Archivos:

- `rust/src/spec.rs` (NUEVO, espejo de plan.rs): `spec_path` (paths.plans +
  plan::slugify, `spec-feature-<id>-<slug>.md`, layout plano => AC-1),
  `spec_template` (plantilla verbatim del plan, misma tecnica join que
  plan_template => AC-1), `write_spec` (solo si no existe), `update_spec_sig`/
  `get_spec_sig` sobre `last_spec_sig` REUSANDO `plan::plan_signature` (sin
  duplicar hashing => AC-2), `is_spec_stale`/`spec_staleness_message` (mismas
  tolerancias que el plan: hash distinto o drift mtime > 1s; falso sin archivo
  o sin firma previa => AC-2 y compat #1/#2), `spec_state` (enum Missing/Draft/
  Approved/Other; primera linea `Estado:` en las primeras 10 lineas, valor trim
  case-insensitive), `require_spec_approved` (rules, default false),
  `close_requires_spec` (solo "done" gatea) y `spec_gate` (Exit::msg accionable
  con ruta, estado y "aprobar editando `Estado: approved`" => AC-3).
- `rust/src/plan.rs`: `sig_mtime` pasa a `pub(crate)` para compartir la
  tolerancia de drift con spec.rs (unico cambio; el hashing ya se reusa).
- `rust/src/main.rs`: `mod spec;`.
- `rust/src/commands/start.rs`: tras write_plan+update_plan_sig, `write_spec` +
  `update_spec_sig` SIEMPRE (la regla solo controla el gate); linea
  `Spec: <rel_spec>` en current.md; println final anuncia spec draft + gate
  (=> AC-1).
- `rust/src/commands/check_plan.rs`: vigila plan Y spec; stale cualquiera =>
  exit 2 con stdout que distingue cual; linea informativa
  `[spec] Estado: <label>` (=> AC-2).
- `rust/src/commands/check_spec.rs` (NUEVO) + `commands/mod.rs` + `cli.rs`
  (variante `CheckSpec`, name `check-spec`): exit 0 = regla apagada (informa) o
  spec aprobado y fresco; exit 1 = sin feature in_progress; exit 2 = spec stale
  O (regla activa y ausente/draft/no aprobado) (=> AC-3).
- `rust/src/commands/advance.rs`: tras validar in_progress aplica `spec_gate`;
  `update_spec_sig` junto a `update_plan_sig` (=> AC-2, AC-3).
- `rust/src/commands/close.rs`: si `--status done` aplica `spec_gate` ANTES de
  mutar la feature; blocked/pending no gatean (=> AC-3).
- `rust/src/commands/autocheck.rs`: suma `update_spec_sig` al bloque de
  re-firma; sigue best-effort, nunca bloquea (=> AC-2).
- `rust/src/commands/status.rs`: linea `[spec] #<id> <estado> (fresco|STALE)`
  junto a la frescura del plan (=> AC-2).
- `rust/tests/cli_basics.rs`: test de start actualizado (spec draft sembrado,
  current.md con `Spec:`, stdout exacto de check-plan) + nuevos: check-spec
  exit 1 sin feature, exit 0 informando regla apagada, check-plan/check-spec
  exit 2 con spec editado por otro agente, gate e2e (advance y close done
  bloquean en draft, usuario aprueba -> advance pasa y re-firma -> check-spec
  rc=0, status muestra `[spec] #1 approved (fresco)`), close blocked pasa sin
  aprobar (=> AC-3, AC-6).
- `rust/src/spec.rs` #[cfg(test)]: plantilla (Estado: draft + secciones + fin
  `-\n`), spec_path, spec_state (draft/approved linea 3, case-insensitive,
  `Estado:` tras la linea 10 NO cuenta, primera linea manda, ausente,
  desconocido), is_spec_stale (drift 1s tolerado, edicion => stale, falso sin
  archivo/firma), firma con orden de claves path/mtime/size/hash, gate (regla
  false/ausente pasa sin spec; regla true bloquea ausente y draft con mensaje
  accionable; approved abre), close_requires_spec solo done (=> AC-6).
- `rust/Cargo.toml` + `rust/Cargo.lock`: version 0.2.0 -> 0.3.0.

Decisiones:

- Mensajes nuevos del spec usan `sh harness_cli ...` (no existe legado
  harness.py que preservar verbatim, a diferencia de los mensajes del plan).
- Gate fail-closed: con regla activa, spec ausente o con `Estado:` no
  reconocible bloquea igual que draft (solo `approved` abre).
- `check-spec` evalua frescura ANTES que aprobacion: un spec editado por otro
  LLM invalida cualquier estado hasta re-firmar (start/advance/autocheck).
- No se corrio `cargo fmt` global (el repo tiene diff rustfmt preexistente en
  cli.rs); el codigo nuevo sigue el estilo manual existente.

Comandos ejecutados (todos en verde):

- `(cd rust && cargo test)` -> 34 unit + 14 integracion, 0 fallos.
- `(cd rust && cargo clippy --all-targets -- -D warnings)` -> limpio.
- `(cd rust && cargo build --release --locked) && cp rust/target/release/harness ./harness`
  -> `./harness --version` = `harness 0.3.0`.
- `HARNESS_REPO_ROOT=$PWD sh harness_cli status` -> rc=0, muestra
  `[plan] #3 fresco` y `[spec] #3 draft (fresco)`.
- `HARNESS_REPO_ROOT=$PWD sh harness_cli check-spec` -> rc=0:
  "Regla require_spec_approved apagada: gate no aplica. Estado del spec: draft".
- `HARNESS_REPO_ROOT=$PWD sh harness_cli check-plan` -> rc=0 (plan fresco;
  "[!] Spec sin firma previa" esperado: la feature #3 arranco con el binario
  0.2.0; la firma `last_spec_sig` la crean advance/autocheck).

Riesgos para el reviewer:

- La feature #3 aun no tiene `last_spec_sig` en el feature_list vivo (se firma
  con el primer advance/autocheck del binario nuevo); mientras tanto
  is_spec_stale=false por diseno (sin firma previa no hay stale).
- La regla `require_spec_approved` sigue APAGADA en el feature_list vivo y en
  `templates/feature_list.json` (se activa en U2 tras la aprobacion de Alan).
- `nudge.rs` no vigila el spec (el plan U1 no lo pide; el aviso multi-LLM del
  spec vive en check-plan/check-spec/status).

## U2 - Gates de shell + regla + constitution (pendiente)

-

## U3 - Instaladores (pendiente)

-

## U4 - Roles, templates espejados y docs (pendiente)

-

## U5 - Smoke tests (pendiente)

-
