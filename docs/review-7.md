# Review - Feature #7: harness_check_robustness

Spec: docs/spec-feature-7-harness-check-robustness.md (Estado: approved, sellado)
Plan: docs/plan-feature-7-harness-check-robustness.md
Impl: docs/impl-7.md

## Veredicto global

**approved**, con dos salvedades explicitas:

1. AC-11 (paridad Windows) queda **parcial**: revision estatica real, sin
   ejecucion, porque no hay `pwsh` ni `powershell` en esta maquina (verificado
   con `command -v`). Misma limitacion aceptada en las features #1, #4, #5 y #6.
2. El arbol queda **sin commit a proposito** (decision del usuario). La
   coherencia del commit unico esta verificada abajo (Impacto y gates).

Nada de lo verificado contradice las 4 decisiones del usuario (Articulo 5); se
re-ejecuto todo lo re-ejecutable y las pruebas negativas se reprodujeron en
fixtures propias, independientes de las del smoke.

## Aprobacion del spec

```
Estado: approved
Aprobado: 2026-07-28T18:10:02Z por USUARIO (confirmacion explicita) - Alan
aprobo el spec #7 en el chat (2026-07-28) tras revisarlo, con las 4 decisiones
de Observaciones ya registradas
```

Rastro en `progress/history.md` linea `approve-spec feature #7 estado=approved
nota=...` (misma marca temporal). `sh harness_cli check-spec` sale rc=0
(`[OK] Spec aprobado y fresco`). U0 del plan cerrada con las 4 decisiones
registradas en spec y plan antes de implementar.

## Estado por AC

Verificado en esta sesion contra el arbol real, no por lectura de `impl-7.md`.
Fixtures propias construidas con `git checkout-index` en un directorio temporal
(nunca sobre el checkout) mas el script vigente del working tree.

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | **Prueba negativa propia**: espejo `.claude/agents/reviewer.md` roto (append) en fixture -> rc=2, mensaje `Espejo desincronizado: .claude/agents/reviewer.md ... Re-corre el instalador ... propaga el cambio a roles/reviewer.md`. Smoke fixture `check-robust` reproduce lo mismo con `implementer.md` (rc=0 global) |
| AC-2 | cubierto | (a) dogfooding en ESTE repo: `env -u HARNESS_REPO_ROOT -u CLAUDE_PROJECT_DIR bash harness_check.sh` -> `[Ok] Harness Check limpio.` rc=0 con los espejos versionados; (b) **espejos Gemini y Codex fabricados por mi con el formato exacto de `build_gemini_agent`/`build_codex_agent`** -> 0 menciones de `Espejo desincronizado`; (c) smoke: check limpio post-install con guard explicito de no-falso-positivo |
| AC-3 | cubierto | **Prueba negativa propia**: Gemini stale (append) y Codex stale (linea inyectada DENTRO del bloque `developer_instructions`, antes del cierre `'''`) -> ambos nombrados, rc=2. Ausencia no falla: clon fresco sin `.gemini/`/`.codex/` y dogfooding real pasan el gate sin quejas |
| AC-4 | cubierto | **Prueba negativa propia**: drift en `templates/roles/implementer.md` -> `Divergencia roles/implementer.md vs templates/roles/implementer.md` rc=2, citando la regla del Articulo 6. Rama prefijo `__HREL__` ejercitada en el dogfooding (basename `harness_process`); rama vacia en la fixture `--root` del smoke; distribucion aplanada: fixture `flat-layout` del smoke sigue verde |
| AC-5 | cubierto | Mensajes nombran archivo exacto + remedio. Modos verificados por mi sobre la misma fixture rota: `block` rc=2 (suma 4 problemas), `warn` rc=0 y REPORTA, `off` rc=0 con salida vacia (0 bytes, no evalua) |
| AC-6 | cubierto | Rama `$HOME` (dogfooding real): `[i] Checkout fuente del arnes detectado (...): REPO_ROOT=/Users/alan/harness_process`, sin falso `Falta docs/constitution.md`, rc=0. Rama "padre sin huella" (fixture propia con marker `subdir` + senales de fuente, sin huella en el padre): `[i]` + resolucion local, sin falso fallo |
| AC-7 | cubierto | `diff` de los 4 scripts raiz vs `templates/` = **identicos** (re-verificado); mismo bloque de resolucion en los 4. Dogfooding: `commit_guard.sh` rc=0 + `[i]`; `harness_status.sh --brief` rc=0 + `[i]` + `Repos limpios`. Smoke `source-sim` corre los 4 con asserts (`[i]` en check/status/guard/init y `raiz=<clon>`), smoke rc=0 |
| AC-8 | cubierto | `GraphEnv::resolve` y `HarnessPaths::from_root` consumen `paths.rs::repo_root_from_marker` (unico punto, verificado por grep). Tests de integracion re-ejecutados: `start_should_stay_inside_source_checkout_and_not_touch_parent` y `home_parent_should_trigger_source_guardrail_even_with_footprint` ok; smoke `source-sim`: plan/spec dentro del clon, `$HOME` de fixture vacio. El `[i]` del binario aparece tambien en `graph impacto` real |
| AC-9 | cubierto | **Fixture propia de subdir LEGITIMA** (padre con `CLAUDE.md`): sin aviso `[i]` y el check evalua el PADRE (prueba positiva: acusa la constitution faltante del padre, no la del clon). Precedencias verificadas una a una: `HARNESS_REPO_ROOT` y `CLAUDE_PROJECT_DIR` mandan (sin `[i]`, evaluan el destino indicado). Tests Rust de no-regresion (`..._with_footprint`, `..._any_single_footprint_file`, `..._without_source_signals`, `env_override_...`) en verde; fixture subdir preexistente del smoke verde |
| AC-10 | cubierto | Reproducido por mi: `git checkout-index -a --prefix=<tmp>/` -> el clon **NO trae `.harness_layout`** (staged `D`), el check resuelve al propio clon, sin falso `Falta docs/constitution.md`, y su unico fallo es `progress/current.md esta vacio` (exactamente lo declarado; estado vivo no versionado). Padre y `$HOME` de fixture intactos. `.gitignore` con `/.harness_layout` + comentario; migracion pull -> re-setup documentada en `UPDATING.md` y `templates/UPDATING.md` |
| AC-11 | **parcial** | Sin `pwsh`/`powershell` (verificado). Revision estatica REAL y propia: balance mecanico de here-strings 3/3 y llaves = 0; lectura linea a linea del bloque nuevo (`Get-AgentBody`/`Get-CodexBody` en paridad fiel con los extractores awk; stale inyectado y restaurado byte a byte; check real via bash con skip informado). `setup_harness.ps1` sin cambios y sin cambios pendientes de espejo (el `.sh` no fue tocado). SIN ejecucion |
| AC-12 | cubierto | Re-ejecutado: `cargo test --locked` **44 unit + 22 integracion, 0 fallos** (antes 40+19; delta exacto: 4 unit en `paths.rs`, 3 int en `cli_basics.rs`); `cargo clippy --all-targets -- -D warnings` rc=0; `bash tests/setup_smoke.sh` rc=0 con las dos lineas nuevas `[Ok] gate de espejo: ...` y `[Ok] checkout fuente simulado: ...`. Cobertura (a)-(d) presente y leida en el codigo de tests |
| AC-13 | cubierto | `README.md` seccion "harness_check.sh: gates de integridad"; `UPDATING.md` + `templates/UPDATING.md` seccion "Marker `.harness_layout` des-versionado + gate de espejo de roles" con la ventana pull -> re-setup; `AGENTS.md` (gate en el orden de trabajo); `docs/architecture.md` (guardrail en `paths.rs`, layouts y riesgo del checkout fuente actualizado) |

## Las 4 decisiones del usuario (Articulo 5)

Verificadas contra el CODIGO, no contra el impl:

1. **Gate bloquea**: las comparaciones suman `failures`; `block` sale 2, `warn`
   reporta y sale 0, `off` sale 0 al inicio sin evaluar. Probado en los tres
   modos sobre una fixture rota.
2. **Solo reporta / read-only**: leido `harness_check.sh` completo; el gate son
   extractores `awk` puros + comparaciones de strings + `echo >&2`. Cero
   redirecciones de escritura, cero `sed -i`, cero regeneracion. Las llamadas
   (`harness_cli status/check-plan/check-spec`, `commit_guard.sh`) son de
   consulta.
3. **Marker des-versionado**: `D .harness_layout` staged (git rm --cached),
   `.gitignore` con `/.harness_layout` y comentario; `git check-ignore` ya lo
   captura; el archivo local sigue en disco con `subdir` (por eso hoy protege el
   guardrail de la decision 4).
4. **Fallback con `[i]`**: verificado en sh (rama `$HOME` real, rama
   padre-sin-huella en fixture) y en Rust (`eprintln!` visto en tests y en
   `graph impacto` real). Ni fallo duro ni silencioso.

## Trazabilidad y constitution

- **Articulo 1**: tests cercanos al codigo tocado (4 unit en `paths.rs`, 3 de
  integracion en `cli_basics.rs`, dos bloques nuevos en el smoke) y los tres
  comandos oficiales en verde, re-ejecutados en esta revision.
- **Articulo 2**: spec approved, sellado por el USUARIO y fresco antes del
  veredicto; rastro en `history.md`. La implementacion arranco despues de la
  aprobacion (18:10:02Z sello; 18:11:34Z primer advance).
- **Articulo 3**: la Delegacion U0..U8 cita sus AC; `impl-7.md` mapea
  AC-1..AC-13; este veredicto lista los 13.
- **Articulo 4**: sin secretos (las credenciales de las fixtures son dummies
  locales); mensajes accionables (archivo exacto + remedio) y exit codes
  estables 0/2 con `HARNESS_CHECK_MODE` intacto.
- **Articulo 5**: las 4 decisiones registradas en spec y plan, implementadas al
  pie de la letra (verificado arriba). La observacion nueva
  (`.harness_backend`) quedo SIN decidir y SIN tocar: correcto, no se invento
  una decision.
- **Articulo 6**: `rust/Cargo.toml` sin cambios (cero dependencias nuevas);
  raiz y `templates/` espejados (diff identico en los 4 scripts; el sub-gate
  nuevo ademas lo vigila para `roles/`); feature backend-agnostica (el gate
  cubre Claude/Grok, Gemini y Codex por igual y la deteccion no depende de
  ningun backend). Commits: no hay commits nuevos que auditar (arbol sin
  commitear por decision del usuario); el mensaje debera ser Conventional y sin
  trailers de IA.

## Impacto y gates

- `sh harness_cli graph impacto --microservicio ADR/harness` ejecutado en esta
  revision: "Ningun microservicio registrado depende de 'ADR/harness'".
  Impacto externo nulo; el radio interno (4 scripts + espejos, `paths.rs`,
  instaladores, tests, docs) es exactamente el declarado en el plan.
- `graphify query` registrado en el plan (20 nodos, confirmando
  `repo_root_from_marker` como unico punto Rust); `graphify-out/` presente y
  sin flag `.graphify_stale`.
- `bash harness_check.sh` limpio (rc=0) en el propio checkout, con `[plan] #7
  fresco` y `[spec] #7 approved (fresco)`.
- `progress/current.md` apunta al plan y lleva la evidencia al dia; el plan
  esta archivado en `docs/` de la raiz y refleja lo implementado (avances U1-U8).
- **Coherencia del commit unico pendiente**: el par `D .harness_layout`
  (staged) + `.gitignore` (unstaged) es consistente; con `git add` de
  `.gitignore`, los 16 modificados y los untracked de la feature
  (`docs/spec-feature-7-*.md`, `docs/plan-feature-7-*.md`, `docs/impl-7.md`,
  este review) un solo commit deja el repo consistente. El binario raiz
  `harness` y `rust/target` quedan fuera por gitignore, como corresponde.

## Hallazgos de esta revision (no bloquean)

1. **Precision sobre la evidencia AC-10 del impl**: la fixture
   `git checkout-index` materializa el INDICE, y los 4 scripts estan
   modificados SIN stagear, asi que ese clon corre el `harness_check.sh` VIEJO
   (sin gate ni guardrail). Para AC-10 no invalida nada: la propiedad exigida
   (clon fresco sin marker -> fallback root) es preexistente al guardrail y la
   reproduje tambien con el script vigente copiado a la fixture. Pero la
   evidencia del impl no distingue ese matiz. Al commitear, la propiedad queda
   completa en cualquier clon nuevo.
2. **`harness_status.sh` no vigila el PROPIO repo cuando la raiz es el repo**:
   itera `$REPO_ROOT/*` buscando repos git en los hijos; en el checkout fuente
   (raiz = el propio repo, caso que el guardrail hace comun) reporta `Repos
   limpios` con 18 archivos modificados. Comportamiento preexistente del
   script (mismo efecto en una instalacion layout root), no introducido por
   esta feature; anotado como mejora posible.
3. **`docs/review-4.md` untracked** en `docs/` (residuo de la feature #4, ajeno
   a esta feature). Decidir si entra en el commit o se descarta.
4. **El aviso `[i]` se repite por corrida** en el checkout fuente (script +
   cada invocacion del binario via `harness_cli`): consecuencia directa de la
   decision 4 (aviso en cada resolucion), declarado por el implementer.
   Bajarle el volumen seria una decision nueva del usuario.
5. **Hallazgo colateral del implementer VERIFICADO**: `tests/setup_smoke.ps1`
   estaba sintacticamente roto desde la feature #2 (commit 65eab1a): quedaron
   los cierres `'@` y las referencias `$fakeCargo` sin ninguna apertura
   `$fakeCargo = @'` (en 2f26545, feature #1, estaban en las lineas 70 y 96).
   El fix restaura exactamente esas 2 lineas de apertura: mecanico, quirurgico
   y legitimo dentro del alcance de U7 (sin el, AC-11 no tenia objeto porque el
   archivo ni parseaba). Sigue sin ejecucion real por la misma falta de pwsh.

## Pendientes (decisiones del usuario, sin decidir aqui)

1. **`.harness_backend` sigue versionado** (valor `postgres`; confirmado con
   `git ls-files`). Mismo caracter de estado local que `.harness_layout` (el
   instalador lo escribe en cada corrida), pero la decision 3 cubrio SOLO el
   layout. A diferencia del layout NO participa en la resolucion de rutas (no
   causa footgun): des-versionarlo seria por consistencia, no por necesidad.
   Queda pendiente de decision: des-versionarlo igual, o dejarlo como default
   de distribucion.
2. **AC-11 sin ejecucion real en Windows**: correr `pwsh tests/setup_smoke.ps1`
   en la primera maquina Windows disponible cubre de una vez la deuda de las
   features #1, #4, #5, #6 y #7 (incluido el fix de here-strings).
3. **Commit unico**: pendiente por decision del usuario (ver "Coherencia del
   commit unico" arriba). Recordatorio de la politica del repo: mensaje
   Conventional SIN trailers de IA.
