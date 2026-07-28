# Impl - Feature #7: harness_check_robustness

Spec: docs/spec-feature-7-harness-check-robustness.md (Estado: approved)
Plan: docs/plan-feature-7-harness-check-robustness.md

## Que cambio

Dos fallas de robustez sobre la misma superficie, con las 4 decisiones del
usuario (2026-07-28) aplicadas al pie de la letra:

```
A. Gate de espejo de roles en harness_check.sh
   roles/<rol>.md (fuente unica)
      == cuerpo tras el frontmatter de .claude/agents/<rol>.md   (Claude+Grok)
      == cuerpo tras el frontmatter de .gemini/agents/<rol>.md   (si existe)
      == bloque developer_instructions de .codex/agents/<rol>.toml (si existe)
   roles/*.md == templates/roles/*.md modulo __HREL__ (prefijo del arnes o vacio)
   -> desincronizado: BLOQUEA (decision 1), SOLO reporta (decision 2),
      remedio = re-correr el instalador; warn/off degradan como siempre.

B. Resolucion de REPO_ROOT robusta ante el checkout fuente
   overrides (HARNESS_REPO_ROOT, vars de agente)  >  marker .harness_layout
   marker 'subdir' + senales de fuente (templates/harness_cli + rust/) +
   padre sin huella {docs/constitution.md, CLAUDE.md, AGENTS.md,
   .claude/settings.json} (o padre == $HOME sin HARNESS_ALLOW_HOME_SURFACE=1)
   -> REPO_ROOT = el propio checkout, con aviso informativo [i] (decision 4).
   Aplicado en los 4 scripts sh + espejos templates/ + rust/src/paths.rs
   (unico punto Rust: HarnessPaths + GraphEnv).
   .harness_layout DES-VERSIONADO (decision 3): git rm --cached + .gitignore.
```

El incidente que motivo B (reproducido antes del fix): en este checkout,
`env -u HARNESS_REPO_ROOT bash harness_check.sh` resolvia `REPO_ROOT=$HOME`,
reportaba el falso `[!] Falta docs/constitution.md` y `harness_cli start`
creaba planes huerfanos en `$HOME/docs`.

## Unidades

| Unidad | AC | Archivos |
| --- | --- | --- |
| U0 (lider) | - | 4 decisiones del usuario registradas en spec + plan; cerrada antes de esta implementacion |
| U1 resolucion sh | AC-6, AC-7, AC-9 | `harness_check.sh`, `harness_status.sh`, `init.sh`, `commit_guard.sh` + espejos identicos en `templates/` |
| U2 resolucion Rust | AC-8, AC-9 | `rust/src/paths.rs` (`repo_root_from_marker`, `source_checkout_mismatch`, `same_dir` + 4 tests unit nuevos), `rust/tests/cli_basics.rs` (fixture `sandbox_source_checkout` + 3 tests nuevos) |
| U3 gate de espejo | AC-1, AC-2, AC-3, AC-5 | `harness_check.sh` (`extract_agent_body`, `extract_codex_body`, comparaciones por rol) + `templates/harness_check.sh` |
| U4 sub-gate templates | AC-4 | `harness_check.sh` (bloque `templates/roles` modulo `__HREL__`) + `templates/harness_check.sh` |
| U5 marker + migracion | AC-10, parte de AC-13 | `git rm --cached .harness_layout`, `.gitignore`, `UPDATING.md`, `templates/UPDATING.md` |
| U6 smoke sh | AC-12 | `tests/setup_smoke.sh` (fixture `check-robust` + fixture `source-sim`) |
| U7 paridad ps1 | AC-11 | `tests/setup_smoke.ps1` (bloque feature #7 + fix de here-strings rotos preexistentes); `setup_harness.ps1` SIN cambios (ver Decisiones) |
| U8 docs | AC-13 | `README.md`, `AGENTS.md`, `docs/architecture.md` (UPDATING ya en U5) |

## Evidencia por AC

- **AC-1** (espejo Claude stale -> reporta y exit 2 en block): gate en
  `harness_check.sh` dentro del bloque de roles: compara
  `$(extract_agent_body "$agent_md")` contra `$(cat roles/$role.md)` y suma
  `failures` con mensaje que nombra `.claude/agents/<rol>.md`. Prueba negativa
  reproducida en `tests/setup_smoke.sh` (fixture `check-robust`): espejo
  alterado -> rc=2 + `grep 'Espejo desincronizado: .claude/agents/implementer.md'`
  en el log. Smoke completo rc=0.
- **AC-2** (cero falsos positivos con espejos del instalador vigente): la
  extraccion salta las lineas en blanco iniciales del cuerpo (la unica
  diferencia que introduce `build_*_agent`) y la comparacion via command
  substitution normaliza el newline final en ambos lados. Evidencia doble:
  (a) dogfooding en ESTE repo: `env -u HARNESS_REPO_ROOT bash harness_check.sh`
  -> `[Ok] Harness Check limpio.` rc=0 con los `.claude/agents/*.md`
  versionados actuales; (b) smoke: check limpio en fixture recien instalada
  (`grep 'Harness Check limpio'` + guard explicito de que NO aparece
  `Espejo desincronizado`).
- **AC-3** (Gemini/Codex stale -> reportan; ausentes -> no fallan): extractores
  por formato (`extract_agent_body` para el frontmatter Gemini,
  `extract_codex_body` para el bloque `developer_instructions` entre `'''`),
  cada comparacion condicionada a `[ -f ... ]` (se preserva la condicionalidad
  actual). Smoke: stale inyectado en `.gemini/agents/leader.md` (append) y en
  `.codex/agents/reviewer.toml` (linea insertada DENTRO del bloque, antes del
  cierre `'''`) -> ambos nombrados en el log con rc=2. Ausencia cubierta por el
  checkout fuente real (sin `.gemini/`/`.codex/`; check limpio en dogfooding) y
  por la fixture `source-sim` (solo `.claude/agents`).
- **AC-4** (roles vs templates/roles modulo `__HREL__`): sub-gate condicional a
  `[ -d templates/roles ]`; valido si `roles/<f>.md` coincide con la expansion
  `s|__HREL__|<basename del arnes>/|g` O con la expansion vacia (mismo `sed`
  que usa el instalador). Smoke: divergencia inyectada en
  `templates/roles/leader.md` -> rc=2 +
  `grep 'Divergencia roles/leader.md vs templates/roles/leader.md'`. La rama
  "expansion con prefijo" se ejercita en el dogfooding de este repo
  (`harness_process/`) y en la fixture `source-sim`; la rama "expansion vacia"
  en la fixture instalada `--root`. Distribucion aplanada sin `templates/roles`:
  se omite sin fallo (la fixture `flat-layout` del smoke sigue en verde).
- **AC-5** (mensajes accionables + modos): cada fallo nombra el archivo exacto y
  la accion (`Re-corre el instalador (setup_harness.sh / setup_harness.ps1)
  ...; si lo que editaste fue el espejo, propaga el cambio a roles/<rol>.md`;
  el sub-gate cita la regla de espejo del Articulo 6). El gate corre dentro del
  flujo normal del script: `off` sale 0 al inicio sin evaluar, `warn` reporta y
  sale 0, `block` sale 2. Smoke: rc/contenido verificados en los tres modos
  (`check-stale.log`, `check-warn.log`, `check-off.log`).
- **AC-6** (checkout fuente resuelve local): guardrail en el bloque de
  resolucion (decision 4: fallback + aviso `[i]`, ni fallo duro ni silencioso).
  Dogfooding en ESTE checkout sin `HARNESS_REPO_ROOT` ni variables de agente:
  `[i] Checkout fuente del arnes detectado (...): REPO_ROOT=/Users/alan/harness_process`,
  sin `Falta docs/constitution.md`, rc=0; la regla que dispara aqui es la de
  `$HOME` (el padre es `/Users/alan`). La rama "padre sin huella" queda cubierta
  por el smoke (`source-sim`) y por los tests Rust. Rutas evaluadas dentro del
  checkout verificadas en el smoke: el padre de la fixture queda SOLO con el
  clon (`ls -A`) y el `$HOME` de fixture vacio.
- **AC-7** (los 4 scripts + espejos, identicos por diff): mismo bloque de
  resolucion en `harness_check.sh`, `harness_status.sh`, `init.sh` y
  `commit_guard.sh` (POSIX; sin bashismos nuevos). `diff` raiz vs `templates/`
  = identicos para los 4 (verificado tras cada edicion). Ejecucion real:
  dogfooding de `commit_guard.sh` (rc=0 + `[i]`) y `harness_status.sh --brief`
  (rc=0 + `[i]` + `Repos limpios`) en este checkout; smoke `source-sim` corre
  los 4 (check rc=0, status --brief rc=0, commit_guard rc=0, init con `[i]` y
  `raiz=<clon>` en el log, tolerando su fallo por DB inalcanzable).
- **AC-8** (binario Rust local, `$HOME` intacto): misma regla en
  `rust/src/paths.rs::repo_root_from_marker` (cubre `HarnessPaths::from_root` y
  `GraphEnv::resolve`, los dos unicos consumidores). Tests de integracion
  nuevos: `start_should_stay_inside_source_checkout_and_not_touch_parent`
  (artefactos en `<checkout>/docs/`, padre sin `docs/`, stderr con `[i]`) y
  `home_parent_should_trigger_source_guardrail_even_with_footprint` (padre ==
  `$HOME` con huella -> local; con `HARNESS_ALLOW_HOME_SURFACE=1` -> padre).
  Smoke `source-sim`: `harness_cli add + start` reales -> plan y spec dentro
  del clon, `$HOME` de fixture vacio.
- **AC-9** (cero regresion subdir + precedencia de overrides): el guardrail
  exige senales de fuente Y padre sin huella; una instalacion subdir legitima
  (padre con huella) no cambia. Tests unit:
  `repo_root_should_resolve_parent_for_subdir_install_with_footprint`,
  `repo_root_should_accept_any_single_footprint_file`,
  `repo_root_should_resolve_parent_without_source_signals` (fixture historica
  sin senales de fuente -> padre, como siempre) y los 3 tests preexistentes del
  marker intactos. Precedencia: `env_override_should_beat_source_checkout_guardrail`
  (`HARNESS_REPO_ROOT` manda y el `[i]` NO aparece); en sh el guardrail vive
  DENTRO de la rama `[ -z "$REPO_ROOT" ]`, asi que overrides y variables de
  agente ni lo evaluan. La fixture subdir del smoke preexistente sigue en verde.
- **AC-10** (clon fresco sin raiz falsa + migracion documentada): decision 3
  aplicada: `git rm --cached .harness_layout` (queda staged como `D`) +
  `.gitignore` con `/.harness_layout` y comentario. Evidencia del clon fresco:
  `git checkout-index -a --prefix=<tmp>/harness_process/` (materializa el
  indice ya des-versionado) -> el clon NO trae `.harness_layout`, el check
  resuelve al propio clon (sin marker el fallback es root), NO reporta el falso
  `Falta docs/constitution.md`, no escribe en el padre ni en `$HOME`; su unico
  fallo es `progress/current.md esta vacio` (honesto: el estado vivo no se
  versiona; comportamiento preexistente de un clon sin instalar). Migracion
  pull -> re-setup documentada en la seccion nueva de `UPDATING.md` y
  `templates/UPDATING.md`.
- **AC-11** (paridad Windows) - **PARCIAL: revision estatica, sin ejecucion**:
  no hay `pwsh` ni `powershell` en esta maquina (verificado con `command -v`),
  mismo precedente que las features #1, #4, #5 y #6.
  - `setup_harness.ps1`: SIN cambios y sin cambios pendientes de espejo: U1/U5
    no tocaron `setup_harness.sh`; el `.ps1` ya tiene la guarda de `$HOME`
    (`HARNESS_ALLOW_HOME_SURFACE`) y escribe el marker en cada instalacion.
  - `tests/setup_smoke.ps1`: bloque nuevo "Feature #7" con (1) asserts de que el
    `harness_check.sh` sembrado trae gate + guardrail, (2) la extraccion de
    cuerpos portada a PowerShell (`Get-AgentBody`, `Get-CodexBody`) comparando
    los espejos de los 3 formatos que genera el instalador ps1 contra
    `roles/*.md` (paridad AC-2/AC-3), (3) espejo stale inyectado detectado por
    esa misma logica y restaurado byte a byte, (4) `harness_check.sh` REAL
    sobre un checkout fuente simulado cuando `bash` existe (Git Bash en
    Windows), con skip informado si no existe.
  - Verificacion estatica: chequeo mecanico de here-strings pareados y balance
    de llaves (script python, resultado `balance final = 0`) + relectura linea
    a linea del bloque nuevo.
- **AC-12** (comandos oficiales + cobertura nueva):
  - `(cd rust && cargo test --locked)`: **44 unit + 22 integracion, 0 fallos**
    (antes: 40+19; nuevos: 4 unit en `paths.rs`, 3 integracion en
    `cli_basics.rs`).
  - `(cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)`:
    limpio (exit 0).
  - `bash tests/setup_smoke.sh`: **rc=0** con las dos lineas nuevas
    `[Ok] gate de espejo: limpio post-install; stale en Claude/Gemini/Codex y drift de templates detectados; warn/off degradan.`
    y
    `[Ok] checkout fuente simulado: resolucion local con aviso [i] en check/status/guard/init, start sin escrituras fuera del clon.`
  - Cobertura (a) tests Rust de la regla en `paths.rs`: 4 unit + 3 integracion;
    (b) check limpio post-install: fixture `check-robust`; (c) espejo stale
    inyectado detectado: mismo bloque (3 formatos + sub-gate de templates +
    modos warn/off + sanity de restauracion); (d) checkout fuente simulado sin
    escrituras fuera del clon y `$HOME` de fixture intacto: fixture `source-sim`.
- **AC-13** (docs): `README.md` seccion nueva "harness_check.sh: gates de
  integridad" (gate de espejo + resolucion robusta); `UPDATING.md` +
  `templates/UPDATING.md` seccion "Marker `.harness_layout` des-versionado +
  gate de espejo de roles" (con la ventana pull -> re-setup) y bullet en "Que
  se actualiza"; `AGENTS.md` (paso 5 del orden de trabajo); `docs/architecture.md`
  (modulo `paths.rs`, seccion "Instaladores y superficies", seccion "Layouts" y
  riesgo del checkout fuente actualizado).

## Decisiones tomadas (todas del usuario, 2026-07-28; ninguna nueva)

- Decision 1 (severidad): el gate de espejo suma `failures` y bloquea con exit 2
  en `block`; `warn` reporta y sale 0; `off` no evalua. Implementado tal cual.
- Decision 2 (remediacion): `harness_check.sh` sigue siendo read-only; los
  mensajes indican re-correr el instalador (o propagar a `roles/` si lo editado
  fue el espejo). No regenera nada.
- Decision 3 (marker): des-versionado con `git rm --cached` + `.gitignore`. El
  archivo LOCAL de este checkout sigue en disco (dice `subdir`), y por eso el
  guardrail de la decision 4 es el que protege este repo hoy.
- Decision 4 (incoherencia): fallback a `HARNESS_DIR` con aviso `[i]` a stderr,
  en sh y en Rust (`eprintln!`). El aviso aparece en cada resolucion (tambien
  via `harness_cli`, que invoca el binario): es intencional (ni silencioso).
- Detalle de implementacion dentro de la nota de diseno del lider: la senal de
  `$HOME` compara paths resueltos (`pwd -P` en sh, `fs::canonicalize` con
  fallback lexico en Rust) y respeta `HARNESS_ALLOW_HOME_SURFACE=1`, en paridad
  con la guarda del instalador.

## Hallazgos colaterales

- **`tests/setup_smoke.ps1` estaba roto (sintaxis pwsh invalida) desde la
  feature #2** (commit 65eab1a): al remover el bloque del fake python se borro
  tambien la linea de apertura `$fakeCargo = @'` de AMBOS here-strings (quedo el
  cuerpo suelto y el cierre `'@`). Verificado contra 2f26545 (feature #1, donde
  era valido). Restaurada la apertura en las dos ramas; sin ese fix el AC-11 no
  tenia objeto (el archivo ni parseaba). Fix mecanico, no fork de diseno.
- **Cuelgue potencial en corridas interactivas del smoke**: `harness_check.sh`
  invoca `commit_guard.sh`, que hace `cat` de stdin; los bloques nuevos del
  smoke redirigen `< /dev/null` (sh) / pipe de `$null` (ps1) para no heredar el
  stdin del terminal. Comportamiento del guard preexistente, no se cambio.

## OBSERVACION SIN DECISION

- **`.harness_backend` sigue versionado** (valor `postgres`). Es estado local de
  instalacion con el mismo caracter que `.harness_layout` (el instalador lo
  escribe en cada corrida), pero la decision 3 cubrio SOLO `.harness_layout`,
  asi que NO se toco. A diferencia del layout, no participa en la resolucion de
  rutas (no causa el footgun); des-versionarlo seria por consistencia, no por
  necesidad. Pendiente de decision del usuario: des-versionarlo igual que el
  marker de layout, o dejarlo versionado como default de distribucion.

## Riesgos pendientes para el reviewer

- AC-11 parcial: sin ejecucion real de `tests/setup_smoke.ps1` (no hay
  pwsh/powershell); revision estatica documentada arriba. El fix de los
  here-strings tambien queda sin ejecucion real por el mismo motivo.
- Al commitear, incluir `.gitignore` (modificado en working tree, sin stagear)
  junto al `git rm --cached .harness_layout` (ya staged), para que el
  des-versionado y el ignore viajen en el mismo commit. El binario raiz
  `harness` se refresco con `cargo build --release` + `cp` (artefacto
  gitignoreado; flujo documentado en `docs/architecture.md`).
- El aviso `[i]` aparece varias veces por corrida en el checkout fuente (script
  + cada invocacion del binario via `harness_cli`). Es consecuencia directa de
  la decision 4 (aviso en cada resolucion); si molesta, bajarle el volumen
  seria una decision nueva del usuario.
- La ventana pull -> re-setup de la decision 3 (instalaciones subdir existentes
  pierden el marker local hasta re-correr el setup) esta documentada en
  `UPDATING.md`; los efectos son benignos y acotados al clon.
