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

## U2 - Gates de shell + regla + constitution (AC-3, AC-4)

Archivos:

- `templates/docs/constitution.md` (NUEVO): principios no negociables en 6
  articulos (calidad/tests primero; specs aprobadas antes de implementar;
  trazabilidad AC-n; seguridad/observabilidad minimas; decisiones del usuario
  mandan; puente a ADRs: "ninguna dependencia nueva de runtime sin ADR").
  Nota explicita "documento del USUARIO: sembrado una vez, nunca pisado". (AC-4)
- `docs/constitution.md` (raiz de este repo): copia ajustada al harness. (AC-4)
- `harness_check.sh` + `templates/harness_check.sh` (identicos, `diff` vacio):
  (a) gate check-spec tras el bloque check-plan (rc 2 => failure accionable;
  rc 1 => no falla); (b) check de existencia de `$REPO_ROOT/docs/constitution.md`
  gateado por `[ -d roles ]` (misma condicion de instalacion completa que el
  bloque de roles, para no romper instalaciones minimas). (AC-3, AC-4)
- `templates/feature_list.json`: `"require_spec_approved": true` en `rules`. La
  regla del feature_list VIVO de este repo se activa al cierre (tras aprobacion
  del spec por Alan), no aqui. (AC-3)

Verificaciones:

- `HARNESS_REPO_ROOT=/Users/alan/harness_process bash harness_check.sh` => rc=0
  limpio (check-spec rc=0 porque la regla viva sigue apagada + spec draft;
  constitution ahora existe).
- `diff harness_check.sh templates/harness_check.sh` => vacio (espejados).
- `templates/feature_list.json` valida como JSON.

## U3 - Instaladores (AC-4, AC-5)

Archivos:

- `setup_harness.sh`:
  - `required_assets` (bloque `WITH_SUBAGENTS`): + `docs/constitution.md`, para
    que el gate de recursos exija el asset antes de instalar (AC-4).
  - mkdirs del layout: `[ "$WITH_SUBAGENTS" -eq 1 ] && do_mkdir "$SURFACE_DIR/docs"`
    con el helper `do_mkdir` existente; la constitution vive en el docs/ de la
    RAIZ (SURFACE_DIR), que en subdir puede no existir aun (AC-4).
  - Siembra NO destructiva (junto a la de `feature_list.json`/`progress`, dentro
    del `if [ "$WITH_SUBAGENTS" -eq 1 ]`):
    `if [ ! -f "$SURFACE_DIR/docs/constitution.md" ]; then install_asset "docs/constitution.md" "$SURFACE_DIR/docs/constitution.md"; fi`.
    Usa el mismo helper `install_asset` (con destino explicito) que el resto del
    script; solo-si-falta, no pisa la del usuario. NO se agrega a la lista
    `generated` (no se respalda ni regenera) (AC-4).
  - Heredoc `write_agent_surface` (superficie completa): nuevo paso `0.2` en
    "ANTES DE IMPLEMENTAR CODIGO": `sh "__HREL__harness_cli" check-spec`; `start`
    genera spec+plan y `check-plan`/`check-spec` vigilan AMBOS; si el spec sigue
    en `Estado: draft` => DETENTE y pide al USUARIO aprobar `docs/spec-feature-*.md`
    (`Estado: approved`), PROHIBIDO auto-aprobar o tocar la linea `Estado:`; spec
    y plan deben cumplir `docs/constitution.md`. En "Archivos principales" sumados
    `docs/constitution.md` (RAIZ) y `docs/spec-feature-<id>-<slug>.md` (RAIZ)
    (AC-5).
  - Heredoc `write_basic_agent_surface` (superficie basica): version corta del
    paso `0.2` (check-spec + mencion a constitution y spec draft) (AC-5).
  - Descripciones de subagentes: `desc_leader` menciona "spec + plan con AC-n";
    `desc_rev` menciona "evidencia por AC". `desc_impl` sin cambios (el plan solo
    pide leader y reviewer) (AC-5).
  - "Comandos utiles": linea `sh ${HREL}harness_cli check-spec` en el listado
    general y en el bloque de subagentes (AC-5).
  - `HARNESS_VERSION`: `2026.06-harness-process` -> `2026.07-harness-process`
    (bump trivial y consistente, opcional del plan).
- `setup_harness.ps1` (paridad CONCEPTUAL, no literal):
  - `Assert-HarnessAssets` (bloque `WithSubagents`): + `docs/constitution.md`
    (AC-4).
  - Bloque principal: `Ensure-Directory` de `(Join-Path $script:SurfaceDir "docs")`
    en el loop de dirs con subagentes; siembra if-missing con destino EXPLICITO
    (`Install-HarnessAssetIfMissing` solo apunta a `HarnessDir`, no sirve):
    `$constitutionDest = Join-Path $script:SurfaceDir "docs/constitution.md"` +
    `if (-not (Test-Path -LiteralPath $constitutionDest)) { Install-HarnessAsset -Asset "docs/constitution.md" -Destination $constitutionDest }`,
    dentro del `if ($script:WithSubagents)` de siembra, fuera de `$generatedAssets`
    (AC-4).
  - `Write-AgentSurface`: nuevo paso `check-spec` (renumerado 1..8) con referencia
    a spec (`docs/spec-feature-*.md`, aprobar `Estado: approved`, never self-approve)
    y a `docs/constitution.md` (AC-5).
  - `Write-AgentDefinitions`: `leader` = "spec + plan with AC-n"; `reviewer` =
    "per-AC evidence" (AC-5).
  - `$script:HarnessVersion`: `2026.06` -> `2026.07-harness-process`.

Decisiones:

- La siembra de la constitution NO entra en `generated`/`$generatedAssets`: es
  documento del usuario, se instala solo-si-falta y un reinstall no lo respalda
  ni lo pisa (mismo criterio que `feature_list.json`/`progress`).
- Destino explicito `$SURFACE_DIR/docs/constitution.md` (no el default
  `$HARNESS_DIR/$asset` de `install_asset`): en layout subdir la constitution
  vive en el docs/ de la RAIZ (padre), no en el docs/ interno del arnes.
- En root layout `SURFACE_DIR == HARNESS_DIR`, asi que el mkdir y la siembra son
  idempotentes; `install_asset` ya evita copiar `source == destination`.
- ASSET_DIR resuelve a `templates/` en instalacion normal (y a la raiz en
  distribucion aplanada); `docs/constitution.md` existe en ambos (`templates/`
  la generica que se siembra, raiz la del harness), por lo que el gate de
  `required_assets`/`Assert-HarnessAssets` pasa en las dos rutas.
- No se corrio `setup_harness.sh` ni `setup_harness.ps1` en este checkout
  (footgun del marker subdir): validacion por `bash -n` + revision manual del
  ps1 (no hay `pwsh` en el entorno).

Verificaciones:

- `bash -n setup_harness.sh` => rc=0 (sintaxis limpia).
- `grep -n "constitution" setup_harness.sh setup_harness.ps1` => en sh:
  `required_assets` (1428), comentario del mkdir (1532) + linea
  `do_mkdir "$SURFACE_DIR/docs"` (1534), siembra if-missing (1875-1876),
  superficies (597, 713, 866); en ps1: `Assert-HarnessAssets` (359), surface
  (564), siembra if-missing (1194-1196).
- `grep -n "check-spec" setup_harness.sh setup_harness.ps1` => sh: ambas
  superficies (594 basica, 706/709 completa) + "Comandos utiles" (2046, 2063);
  ps1: `Write-AgentSurface` paso 5 (560).
- Siembra de constitution FUERA de `generated` (sh, `init.sh..UPDATING.md` sin
  constitution) y de `$generatedAssets` (ps1, `CHECKPOINTS.md..roles/reviewer.md`
  sin constitution): confirmado por inspeccion de ambos arrays.
- ps1: bloque de siembra bien anidado y cerrado (`if ($script:WithSubagents) {`
  cierra en 1198 antes de `Build-HarnessBinary`); lista de `Write-AgentSurface`
  renumerada 1..8 sin huecos.

Riesgos para el reviewer:

- `setup_harness.ps1` no se ejecuto (sin `pwsh` en el entorno): revision manual;
  la ejecucion Windows real queda para cuando haya entorno (como en feature #1).
- Los cuerpos de los subagentes `.claude/.codex/.gemini` se ensamblan desde
  `roles/*.md` (spec aprobado + AC-n): eso lo cubre U4; aqui solo cambian las
  descripciones (frontmatter) y las superficies.
- Si alguien actualiza solo `setup_harness.sh`/`.ps1` sin copiar
  `templates/docs/constitution.md`, el gate de `required_assets` falla con
  mensaje claro (no hay bootstrap especial como el de `UPDATING.md`, y el plan
  no lo pide).

## U4 - Roles, templates espejados y docs (pendiente)

-

## U5 - Smoke tests (pendiente)

-
