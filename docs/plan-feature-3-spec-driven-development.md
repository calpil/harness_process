# Plan - Feature #3: spec_driven_development

Estado: in_progress
Microservicios:
- (sin servicios)

## Alcance

Integrar Spec-Driven Development (SDD, inspirado en github/spec-kit, adaptado) al harness.
Los 7 puntos del acceptance, con su AC-n (el spec U0 los formaliza en Given/When/Then):

- **AC-1 - Spec generado en start (layout plano)**: `harness_cli start` genera
  `docs/spec-feature-<id>-<slug>.md` ADEMAS del plan, en el `docs/` de la RAIZ,
  plano junto a los planes (NO carpetas tipo Spec Kit `specs/NNN/`). Plantilla:
  `Estado: draft`, recorridos de usuario priorizados (P1/P2, independientemente
  testeables), criterios de aceptacion AC-n en Given/When/Then, no funcionales
  (SLOs, seguridad, observabilidad), fuera de alcance, observaciones (mismo
  protocolo de decision que el plan).
- **AC-2 - Firma y vigilancia multi-LLM**: `last_spec_sig` en la feature reusa la
  mecanica de `plan_signature()` (mismo dict path/mtime/size/hash); `check-plan`
  vigila spec+plan contra ediciones de otros LLMs (exit 2 si cualquiera esta stale).
- **AC-3 - Gate duro**: con la regla `require_spec_approved: true` en
  `feature_list.json` y spec sin `Estado: approved`, `advance` y `close --status done`
  bloquean con mensaje claro, y `harness_check.sh` falla (via nuevo subcomando
  `check-spec`). Regla ausente/false => gate apagado (compat instalaciones previas).
- **AC-4 - Constitution**: nuevo `docs/constitution.md` (docs de la RAIZ) sembrado
  por `setup_harness.sh` y `setup_harness.ps1` desde `templates/docs/constitution.md`
  SOLO si falta (no pisa el del usuario), referenciado por superficies
  (CLAUDE/AGENTS/GEMINI/LLM) y roles, verificado por `harness_check.sh`.
- **AC-5 - Roles con trazabilidad**: leader/implementer/reviewer (en `roles/`,
  `templates/roles/` y los subagentes generados `.claude/agents`/`.codex/agents`/
  `.gemini/agents` de ambos instaladores) exigen spec aprobado antes de implementar
  y trazabilidad AC-n: cada item de Delegacion del plan cita su AC-n; el reviewer
  exige evidencia por AC en el veredicto.
- **AC-6 - Tests**: `cargo test` + `cargo clippy -- -D warnings` limpios con tests
  de firma y gate del spec; `tests/setup_smoke.sh` verifica siembra de spec (via
  `start` e2e) y de constitution (incluida la semantica no-pisa).
- **AC-7 - Docs**: README.md, UPDATING.md (+ `templates/UPDATING.md`), AGENTS.md
  y `docs/architecture.md` actualizados con el flujo SDD.

Contexto de repo: este es el REPO FUENTE del instalador (sin microservicios).
Reglas de mantenedor (UPDATING.md): solo Rust, espejar `templates/` con la raiz,
tests obligatorios, commits sin trailers de IA. NO correr `setup_harness.sh` en
este checkout (footgun conocido): el binario raiz se refresca con cargo + cp.

## Impacto entre microservicios

- Sin microservicios registrados; `graph mapa` / `graph impacto` no aplican aqui.
- Contratos internos afectados (impacto real de este repo):
  - `feature_list.json`: nueva clave por-feature `last_spec_sig` y nueva regla
    `rules.require_spec_approved`. Ambas OPCIONALES al leer (default: sin firma /
    false) para no romper features #1/#2 done ni instalaciones existentes.
  - Exit codes de `check-plan` se conservan (0/1/2); el 2 ahora tambien puede
    significar "spec stale" (stdout distingue). Nuevo subcomando `check-spec`
    con el mismo contrato 0/1/2 para `harness_check.sh` y hooks.
  - `harness_check.sh` suma dos gates (spec aprobado via check-spec + existencia
    de constitution); mismo modo block|warn|off.

## Consulta al grafo (graphify)

- `graphify query "plan_signature update_plan_sig check-plan"` confirma el radio:
  `plan.rs` (plan_signature L38, update_plan_sig L64, is_plan_stale L75,
  plan_staleness_message L84, plan_template L109, write_plan L159) es consumido por
  `commands/start.rs` (L44), `commands/advance.rs` (L44), `commands/autocheck.rs`
  (L73), `commands/check_plan.rs`, `commands/status.rs` (L64-73) y
  `commands/nudge.rs` (L44). `spec.rs` debe integrarse en esos mismos puntos.

## Plantilla del spec (contrato para spec_template() en rust/src/spec.rs)

```
# Spec - Feature #<id>: <name>

Estado: draft
Plan: docs/plan-feature-<id>-<slug>.md
Constitution: docs/constitution.md

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como <rol>, quiero <accion>, para <resultado>.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given <contexto>, When <accion>, Then <resultado observable>.

## No funcionales
- SLOs:
- Seguridad:
- Observabilidad:

## Fuera de alcance
-

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
-
```

Deteccion de `Estado: approved` (decidido por el lider, especificado): se
inspeccionan las PRIMERAS 10 lineas del spec; la primera linea cuyo trim
empiece con `Estado:` define el estado; el valor es el resto tras `Estado:`,
con trim y comparado en minusculas (`approved` => approved, `draft` => draft,
otro/ausente => no aprobado). Solo el USUARIO cambia draft->approved; los
agentes tienen PROHIBIDO auto-aprobar (regla en roles y CHECKPOINTS).

## Delegacion (implementer)

Orden obligatorio U0 -> U5. Cada unidad cita su AC-n. Commits por unidad,
Conventional Commits, SIN trailers de IA.

### U0 - Bootstrap: spec retroactivo de esta feature (AC-1, AC-5)

1. Crear a mano `docs/spec-feature-3-spec-driven-development.md` usando la
   plantilla de arriba, con `Estado: draft`: recorridos P1 (maintainer inicia
   feature y obtiene spec+plan; usuario aprueba spec; implementer bloqueado sin
   aprobacion), AC-1..AC-7 formalizados en Given/When/Then desde el Alcance,
   no funcionales (gate en <1s local, sin red; mensajes accionables; exit codes
   estables para hooks), fuera de alcance (no specs/NNN/, no aprobacion por LLM,
   no migracion retroactiva de features #1/#2).
2. **Pedir a Alan que apruebe el spec** (el edita `Estado: draft` -> `Estado:
   approved`). La regla `require_spec_approved` NO se activa en el
   feature_list.json vivo hasta tener esa aprobacion (evita auto-bloquearse).

### U1 - Nucleo Rust (AC-1, AC-2, AC-3, AC-6)

Archivos en orden:

1. `rust/src/spec.rs` (NUEVO, espejo de plan.rs): `spec_path()` (usa
   `paths.plans` + `plan::slugify`, nombre `spec-feature-<id>-<slug>.md`),
   `spec_template()` (plantilla de arriba, misma tecnica join que
   `plan_template()`), `write_spec()` (crea solo si no existe),
   `update_spec_sig()` / `get_spec_sig()` sobre la clave `last_spec_sig`
   REUSANDO `plan::plan_signature()` (no duplicar el hashing),
   `is_spec_stale()` / `spec_staleness_message()` (mismas tolerancias: hash
   distinto o drift mtime > 1.0s; falso si falta archivo o falta firma previa),
   `spec_state()` -> enum {Missing, Draft, Approved, Other} con la regla de
   deteccion de arriba, `require_spec_approved(data) -> bool` (lee
   `data["rules"]["require_spec_approved"]`, default false) y
   `spec_gate(paths, data, feature) -> Result<(), Exit>` (Exit::msg con ruta del
   spec, estado actual y accion: "pide al usuario aprobar con Estado: approved").
2. `rust/src/main.rs`: registrar `mod spec;`.
3. `rust/src/commands/start.rs`: tras `write_plan` + `update_plan_sig` (L38-44),
   llamar `write_spec` + `update_spec_sig` (siempre, aunque la regla este
   apagada); agregar linea `Spec: <rel_spec>` al current.md generado (L66-74) y
   println final anunciando spec draft + gate.
4. `rust/src/commands/check_plan.rs`: ademas del plan, evaluar
   `is_spec_stale`/`spec_staleness_message`; stale cualquiera => `Exit::code(2)`.
   Reportar informativamente `spec_state` (draft/approved/ausente).
5. `rust/src/commands/check_spec.rs` (NUEVO) + `rust/src/commands/mod.rs` +
   `rust/src/cli.rs` (variante `CheckSpec { feature: Option<String> }`, name
   `check-spec`, mismo patron que CheckPlan): exit 0 = regla apagada (informa)
   o spec aprobado y fresco; exit 1 = sin feature in_progress; exit 2 = spec
   stale O (regla activa y spec ausente/draft/no aprobado).
6. `rust/src/commands/advance.rs`: tras validar in_progress (L23-31), aplicar
   `spec_gate` (regla activa + no aprobado => error con mensaje claro). Tras el
   append al plan, `update_spec_sig` junto a `update_plan_sig` (L42-45).
7. `rust/src/commands/close.rs`: al inicio, si `status == "done"` y regla
   activa, aplicar `spec_gate` ANTES de mutar la feature (L24-42);
   `blocked`/`pending` no gatean (valvula de escape para abortar/aparcar).
8. `rust/src/commands/autocheck.rs`: en el bloque que re-firma (L68-75), sumar
   `update_spec_sig` (ya vigila todos los docs/*.md); sigue best-effort, NUNCA
   bloquea.
9. `rust/src/commands/status.rs`: junto al reporte de frescura del plan
   (L59-74), linea `[spec] #<id> <estado> (fresco|STALE)`.
10. Tests inline (#[cfg(test)]): plantilla contiene `Estado: draft` y secciones;
    spec_path; spec_state (approved linea 3, approved fuera de las 10 primeras
    lineas NO cuenta, case-insensitive del valor, draft, ausente);
    is_spec_stale con drift 1s y edicion; gate: regla false => advance/close
    pasan sin spec (compat #1/#2), regla true + draft => advance y close done
    fallan y close blocked pasa; firma con orden de claves path/mtime/size/hash.
11. `rust/Cargo.toml`: version 0.2.0 -> 0.3.0 (cambio de comportamiento visible).
12. Verificar: `cargo test` + `cargo clippy -- -D warnings` en `rust/`.
13. Refrescar el binario LOCAL de este repo (NUNCA setup_harness.sh aqui):
    `(cd rust && cargo build --release --locked) && cp rust/target/release/harness ./harness`.

### U2 - Gates de shell + regla + constitution (AC-3, AC-4)

1. `templates/docs/constitution.md` (NUEVO asset): principios del proyecto
   (articulos numerados: calidad/tests primero, specs aprobadas antes de
   implementar, trazabilidad AC-n, seguridad/observabilidad minimas, decisiones
   del usuario mandan), nota "documento del USUARIO: el instalador lo siembra
   una vez y nunca lo pisa; specs y planes deben cumplirlo y el reviewer lo
   verifica". Sin placeholders __HREL__ (vive en docs/ de la raiz del proyecto).
2. `docs/constitution.md` (este repo fuente): copiarlo desde el template (aqui
   raiz == repo), ajustado al harness si se quiere.
3. `harness_check.sh` Y `templates/harness_check.sh` (hoy identicos; editar y
   copiar): (a) tras el bloque check-plan (L31-41), bloque equivalente con
   `sh "$HARNESS_DIR/harness_cli" check-spec` (rc 2 => failure con mensaje
   "[!] Spec sin aprobar o modificado..."; rc 1 => no falla); (b) dentro del
   bloque `[ -d "$HARNESS_DIR/roles" ]` (L60-87) o gateado igual:
   `[ ! -f "$REPO_ROOT/docs/constitution.md" ]` => failure pidiendo re-setup.
4. `templates/feature_list.json`: `"require_spec_approved": true` en `rules`.
5. `feature_list.json` VIVO de este repo (no versionado): agregar la regla
   `require_spec_approved: true` SOLO despues de la aprobacion de Alan en U0 y
   con el binario nuevo instalado (U1.13).

### U3 - Instaladores (AC-4, AC-5)

1. `setup_harness.sh`:
   - `required_assets` (L1384-1409, bloque subagents): + `docs/constitution.md`.
   - mkdirs (L1505-1516): con subagents, `do_mkdir "$SURFACE_DIR/docs"` (la
     constitution vive en el docs/ de la RAIZ; en subdir el padre puede no
     tenerlo).
   - Bloque de siembra no destructiva (L1826-1846, junto a feature_list.json):
     `if [ ! -f "$SURFACE_DIR/docs/constitution.md" ]; then install_asset
     "docs/constitution.md" "$SURFACE_DIR/docs/constitution.md"; fi`. NO va en
     `generated` (no se respalda/pisa).
   - Heredoc `write_agent_surface` (L648-856): en "ANTES DE IMPLEMENTAR" sumar
     paso 0.2: spec aprobado obligatorio (`sh "__HREL__harness_cli" check-spec`;
     si draft => detente y pide al usuario aprobar `docs/spec-feature-*.md`);
     mencionar que start genera spec+plan y que check-plan vigila ambos; en
     "Archivos principales" sumar `docs/constitution.md` (principios; los specs
     y planes deben cumplirlo) y `docs/spec-feature-<f>.md`.
   - Heredoc `write_basic_agent_surface` (L572-634): version corta del mismo paso.
   - Descripciones de subagentes (L1803-1805): desc_leader menciona "spec + plan
     con AC-n"; desc_rev menciona "evidencia por AC". (Los cuerpos de
     .claude/.codex/.gemini agents se ensamblan desde roles/*.md: U4 los cubre.)
   - "Comandos utiles" (L2004-2028): sumar linea `harness_cli check-spec`.
2. `setup_harness.ps1`:
   - `Assert-HarnessAssets` (L339-371): + `docs/constitution.md`.
   - Bloque principal (L1131-1183): Ensure-Directory de `SurfaceDir/docs`; siembra
     if-missing con destino explicito:
     `if (-not (Test-Path (Join-Path $script:SurfaceDir "docs/constitution.md")))
     { Install-HarnessAsset -Asset "docs/constitution.md" -Destination ... }`
     (Install-HarnessAssetIfMissing solo apunta a HarnessDir; no sirve tal cual).
   - `Write-AgentSurface` (L545-588): sumar paso check-spec + referencia a
     constitution y specs (paridad conceptual con el surface sh, no literal).
   - `Write-AgentDefinitions` (L598-602): descripciones con spec/AC.
3. Opcional: `HARNESS_VERSION` (sh L49) a `2026.07-harness-process`.

### U4 - Roles, templates espejados y docs (AC-5, AC-7)

1. `templates/roles/leader.md`: nuevo paso: completar el spec generado por start
   (`docs/spec-feature-<id>-<slug>.md`: recorridos P1/P2, AC-n G/W/T, no
   funcionales, fuera de alcance) ANTES del plan; cada item de Delegacion del
   plan CITA su AC-n; el spec debe cumplir `docs/constitution.md`; el lider NO
   aprueba el spec: lo deja en draft y pide la aprobacion del usuario.
2. `templates/roles/implementer.md`: paso 0.2 nuevo: `check-spec` limpio antes
   de implementar; si draft => detente y pide al usuario aprobar; PROHIBIDO
   editar `Estado:` del spec; evidencia en docs/impl-<f>.md indica que AC-n
   cubre cada cambio.
3. `templates/roles/reviewer.md`: verificar spec approved y fresco
   (`check-spec`), evidencia POR AC-n (tabla AC -> evidencia/test), plan
   trazado al spec, cumplimiento de constitution; veredicto lista los AC.
4. `templates/roles/README.md`: diagrama y tabla: LIDER escribe docs/spec-* +
   docs/plan-*; aprobacion del usuario (draft->approved) entre lider e
   implementer.
5. Espejar a la raiz de este repo (regla mantenedor):
   `for f in leader implementer reviewer README; do sed 's|__HREL__|harness_process/|g' templates/roles/$f.md > roles/$f.md; done`.
6. `CHECKPOINTS.md` + `templates/CHECKPOINTS.md`: checkboxes nuevos: spec
   aprobado y fresco (`check-spec` pasa) y evidencia por AC-n en review.
7. `README.md`: seccion del flujo: start genera spec (draft) + plan; gate
   require_spec_approved; check-spec; constitution sembrada.
8. `UPDATING.md` + `templates/UPDATING.md` (divergen: el template trae notas
   extra; editar AMBOS conscientemente): que se actualiza (constitution seed,
   check-spec) y OPT-IN para instalaciones existentes: agregar
   `"require_spec_approved": true` a rules de su feature_list.json (el seed es
   solo-si-falta y no lo agrega solo).
9. `AGENTS.md` (de este repo): orden de trabajo con spec draft -> aprobacion
   usuario -> implementacion; archivos principales + constitution + specs.
10. `docs/architecture.md` (este repo): describir la arquitectura real del
    harness: binario Rust (commands/, plan.rs, spec.rs, gates y exit codes),
    instaladores, superficies, flujo SDD. `templates/docs/architecture.md`:
    solo un bullet apuntando a constitution/specs (sigue siendo esqueleto).

### U5 - Smoke tests (AC-6)

1. `tests/setup_smoke.sh`:
   - ROOT_LAYOUT y SUBDIR: `test -f .../docs/constitution.md` (en SUBDIR es
     `$SUBDIR_ROOT/docs/constitution.md`, la RAIZ, no la subcarpeta).
   - No-pisa: antes del re-run de SUBDIR (bloque CUSTOM_BKP, L199-216), escribir
     sentinel en la constitution y assert de que sobrevive al reinstall.
   - E2E spec con el binario sembrado (fixture tipo POSTGRES_DEFAULT): con
     `DB_HOST=127.0.0.1 DB_PORT=9` (rechazo instantaneo, patron L157-165) y
     `HARNESS_REPO_ROOT` apuntando al fixture: `harness_cli add --name demo` +
     `start --feature 1`; assert `docs/spec-feature-1-demo.md` existe y contiene
     `Estado: draft`; con la regla true (el template ya la trae): `advance`
     falla; `check-spec` rc=2; `sed -i'' 's/Estado: draft/Estado: approved/'`;
     `advance --nota ok --no-graphify` pasa; `check-spec` rc=0.
2. `tests/setup_smoke.ps1`: assert de constitution sembrada y de
   `docs/constitution.md` en required assets (paridad minima; ejecucion real en
   Windows cuando este disponible, como en feature #1).
3. Correr TODO el set final: `cargo test`, `cargo clippy -- -D warnings`,
   `bash tests/setup_smoke.sh`, `bash harness_check.sh`, `sh harness_cli
   check-plan`, `sh harness_cli check-spec`, `sh harness_cli status`.
4. Evidencia en `docs/impl-3.md` (comandos + resultado + AC-n por cambio);
   cierre lo hace el reviewer con el binario NUEVO (el gate debe pasar porque
   el spec de U0 quedo approved).

## Criterios de cierre (reviewer)

- Spec `docs/spec-feature-3-spec-driven-development.md` existe, `Estado:
  approved` puesto por Alan (verificable en la conversacion/advance note), y
  `sh harness_cli check-spec` rc=0 con el binario nuevo (0.3.0).
- `sh harness_cli check-plan` rc=0 (plan y spec frescos, firmas registradas).
- `cargo test` y `cargo clippy -- -D warnings` limpios en `rust/`.
- `bash tests/setup_smoke.sh` verde (incluye siembra spec+constitution y gate).
- `bash harness_check.sh` limpio en este repo (incluye gate de constitution =>
  `docs/constitution.md` debe existir aqui).
- Gate verificado a mano: con regla true y un spec draft de fixture, `advance` y
  `close --status done` fallan con mensaje accionable; `close --status blocked`
  pasa; con regla ausente todo funciona como antes (compat features #1/#2:
  `status` no rompe con features done sin last_spec_sig).
- Trazabilidad: cada unidad U cita AC-n; `docs/impl-3.md` mapea cambios->AC;
  `docs/review-3.md` da veredicto POR AC (AC-1..AC-7) ademas del global.
- Espejado templates/ vs raiz: `diff harness_check.sh templates/harness_check.sh`
  vacio; `roles/*` == `templates/roles/*` modulo `__HREL__`->`harness_process/`;
  CHECKPOINTS identicos; secciones nuevas de UPDATING presentes en ambos.
- `templates/feature_list.json` con la regla true; feature_list.json vivo con la
  regla activada y feature #3 done al cierre.
- Cargo.toml 0.3.0; superficies sh y ps1 con paridad conceptual (check-spec +
  constitution en ambas).
- Commits Conventional SIN trailers de IA (`commit_guard.sh` limpio).

## Riesgos

- **Bootstrapping feature #3**: la feature arranco con el binario viejo (sin
  spec). Mitigado por U0: backfill del spec + aprobacion de Alan ANTES de
  activar la regla en el feature_list.json vivo; si se activara antes, advance
  y close quedarian auto-bloqueados.
- **Instalaciones existentes**: el seed de feature_list.json es solo-si-falta,
  asi que las instalaciones viejas NO reciben la regla => gate apagado por
  defecto para ellas (comportamiento decidido: opt-in documentado en
  UPDATING.md). Features #1/#2 done sin spec no gatean: el gate solo actua en
  advance/close-done de la feature activa y is_spec_stale exige firma previa.
- **Exit code 2 sobrecargado en check-plan**: hooks/mensajes existentes asumen
  "plan stale"; el stdout debe distinguir plan vs spec. Riesgo bajo; no cambiar
  la semantica 0/1/2.
- **advance bloqueado pre-aprobacion**: con gate activo no se pueden registrar
  notas via advance hasta aprobar el spec (decision de Alan: gate duro). El
  flujo es: start -> completar spec -> usuario aprueba -> advance. Documentarlo
  en roles para que nadie lo confunda con un bug.
- **Paridad sh vs ps1**: las superficies ps1 son un resumen en ingles, no copia
  literal; espejar conceptos (check-spec, constitution) y validar con el smoke
  ps1 estatico; ejecucion Windows real queda pendiente de entorno.
- **Smoke lento por DB**: start/advance registran en el hub best-effort; usar
  `DB_HOST=127.0.0.1 DB_PORT=9` (rechazo instantaneo) y `--no-graphify`, nunca
  un hostname DNS inexistente.
- **Binario raiz stale en este repo**: `./harness` es 0.2.0 (jun-11); hasta
  U1.13 los gates nuevos no rigen localmente. NUNCA correr setup_harness.sh en
  este checkout (marker subdir apuntaria a $HOME): rebuild + cp manual.
- **autocheck/nudge**: deben seguir best-effort (tragan errores); el gate duro
  vive SOLO en advance/close/check-spec/harness_check.
- **Marker residual .harness_layout=subdir en este checkout**: hace que el
  binario resuelva repo_root=$HOME (el skeleton del plan quedo en
  ~/docs/plan-feature-3-*.md, 648 bytes). En este repo TODO comando harness_cli
  debe correr con HARNESS_REPO_ROOT=/Users/alan/harness_process (o limpiar los
  markers residuales .harness_layout/.harness_backend y el stray de ~/docs, a
  decision del orquestador). harness_check.sh ya honra HARNESS_REPO_ROOT.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Decision tomada (Alan, 2026-07-22): integracion COMPLETA en el binario Rust
  (start genera spec, last_spec_sig reusa plan_signature, check-plan vigila
  spec+plan).
- Decision tomada (Alan, 2026-07-22): GATE DURO: spec nace draft; sin
  `Estado: approved`, advance/close bloquean y harness_check.sh falla; regla
  `require_spec_approved: true` en feature_list.json.
- Decision tomada (Alan, 2026-07-22): docs/constitution.md sembrado por ambos
  instaladores (si no existe, no pisa), referenciado por superficies y roles,
  verificado por harness_check.sh.
- Decision tomada (Alan, 2026-07-22): layout PLANO: specs junto a los planes en
  docs/ de la raiz (sin carpetas specs/NNN/).
- Decision tomada (lider, 2026-07-23): deteccion de aprobacion = primera linea
  `Estado:` dentro de las primeras 10 lineas del spec; valor trim +
  case-insensitive; solo el usuario aprueba.
- Decision tomada (lider, 2026-07-23): la constitution vive en el docs/ de la
  RAIZ (SURFACE_DIR/docs, junto a planes y specs, versionada con el proyecto),
  no en el docs/ interno del arnes (gitignorado en layout subdir).
- Decision tomada (lider, 2026-07-23): nuevo subcomando `check-spec` (0/1/2)
  para el gate de harness_check.sh/roles; check-plan conserva su contrato y
  suma vigilancia de frescura del spec.
- Decision tomada (lider, 2026-07-23): `close` solo gatea `--status done`;
  blocked/pending no requieren spec aprobado (valvula para abortar/aparcar).
- Decision tomada (lider, 2026-07-23): `start` siembra el spec SIEMPRE (aunque
  la regla este apagada); el gate lo controla solo la regla.
- Decision tomada (lider, 2026-07-23): bootstrapping: el implementer backfillea
  el spec de la feature #3 (U0) y Alan lo aprueba ANTES de activar la regla en
  el feature_list.json vivo de este repo.
- (ninguna observacion pendiente sin decision)

### Avance 2026-07-23T04:13:17Z
Plan del leader persistido (alcance AC-1..AC-7, delegacion U0-U5, gates y riesgos; decisiones registradas como tomadas)

### Avance 2026-07-24T02:47:40Z
Spec aprobado por Alan (instruccion directa); regla require_spec_approved activada

---
Cerrado: 2026-07-24T02:49:29Z - status=done - 
