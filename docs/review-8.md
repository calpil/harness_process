# Review - Feature #8: kimi_cli_backend

Spec: docs/spec-feature-8-kimi-cli-backend.md (Estado: approved, sellado)
Plan: docs/plan-feature-8-kimi-cli-backend.md
Impl: docs/impl-8.md

## Veredicto global

**approved**, con dos salvedades explicitas y tres precisiones sobre el impl:

1. AC-10 (paridad Windows) queda **parcial**: revision estatica real, sin
   ejecucion, porque no hay `pwsh` ni `powershell` en esta maquina (verificado
   con `command -v`). Misma limitacion aceptada en #1, #4, #5, #6 y #7.
2. El arbol queda **sin commit a proposito** (decision del flujo). La
   coherencia del commit unico esta verificada abajo (Impacto y gates).
3. Precisiones (no bloquean; detalle en Hallazgos): (a) `kimi doctor` real
   tambien falla por issues de SCHEMA del config del usuario, no solo por TOML
   invalido — la afirmacion del impl "rc=1 solo con TOML invalido" es
   imprecisa, aunque el rollback sigue siendo igual de seguro; (b) la primera
   instalacion sobre un config sin newline final normaliza la ultima linea del
   usuario con un `\n` (mitigacion deliberada del plan; las re-instalaciones
   son byte a byte); (c) el conteo de here-strings preexistentes del smoke ps1
   era 3, no 2 (el pareo 6/6 se sostiene igual).

**El home real de Kimi del usuario quedo intacto de punta a punta de esta
revision**: `~/.kimi-code/config.toml` = 181 bytes, mtime `Jul 27 17:12`,
`grep -c '^\[\[hooks\]\]'` = 0, sin `AGENTS.md` ni `agents/` dentro, y sha256
`039a0448...78f5bdf` IDENTICO al baseline tomado al inicio. Toda corrida del
instalador, del smoke y del `kimi doctor` real uso `KIMI_CODE_HOME` de
fixture bajo el scratchpad de la sesion o el tmp del smoke.

Nada de lo verificado contradice las 3 decisiones del usuario (Articulo 5);
se re-ejecuto todo lo re-ejecutable y las pruebas del bloque global, del gate
y de las ramas de no-instalacion se reprodujeron en fixtures propias,
independientes de las del smoke.

## Aprobacion del spec

```
Estado: approved
Aprobado: 2026-07-28T23:19:20Z por USUARIO (confirmacion explicita) - Alan
aprobo el spec #8 en el chat (2026-07-28) tras revisarlo, con las 3 decisiones
de Observaciones ya registradas
```

Rastro en `progress/history.md` linea 49 `approve-spec feature #8
estado=approved nota=...` (misma marca temporal). `sh harness_cli check-spec`
rc=0 (`[OK] Spec aprobado y fresco`) y `check-plan` rc=0, ambos re-corridos en
esta revision. La implementacion arranco DESPUES del sello (23:19:20Z sello;
23:20:44Z primer advance). U0 cerrada con las 3 decisiones registradas en spec
y plan antes de tocar codigo.

## Estado por AC

Verificado en esta sesion contra el arbol real y en fixtures PROPIAS
(scratchpad de la sesion, patron `copy_fixture` del smoke: setup_harness.sh +
templates/ + binario Rust precompilado; `KIMI_CODE_HOME`, `HOME`,
`HARNESS_HUB` y `HARNESS_BKP_DIR` de fixture), no por lectura de `impl-8.md`.

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | Fixture propia: los 3 espejos `.kimi-code/agents/*.md` con `head -1 = ---`, `name: <rol>`, `description:` y `tools:` EXACTOS por rol (leader/reviewer `Read, Grep, Glob, Bash`; implementer `Read, Edit, Write, Bash, Grep, Glob`); cuerpo extraido con el awk del gate == `roles/<rol>.md` **verbatim** (diff limpio) en los 3. Codigo: `build_kimi_agent` solo dentro de `if [ "$WITH_SUBAGENTS" -eq 1 ]` y `do_mkdir` de `.kimi-code/agents` idem; smoke re-ejecutado (rc=0) cubre `--no-subagents` -> `test ! -d .kimi-code` y layouts root+subdir |
| AC-2 | cubierto | `bin/harness-kimi` generado con el MISMO esqueleto `LAUNCHER_EOF` (`AGENT="kimi"`, `command -v` + exit 127), ejecutable; `backup_file` de los 3 espejos + launcher visto en el log del instalador y en `bkp/` de fixture; resumen final lista `superficies/hooks: ... Kimi ...`, la linea condicional del config global (`KIMI_HOOKS_WRITTEN`) y el launcher en ambas listas |
| AC-3 | cubierto | Bloque generado en fixture: EXACTAMENTE 3 `[[hooks]]` (SessionStart 120 / PostToolUse `matcher = "Edit|Write"` 30 / Stop 120), cero `SessionEnd|UserPromptSubmit`; cero rutas absolutas o de proyecto dentro del bloque (grep `/Users|/private|/tmp|harness_process` vacio: el command se ancla en `$PWD`). **Guard ejecutado por mi**: en cwd sin `bin/harness-hook` -> rc=0 y salida vacia; con hook fake -> despacha `plain stop` / `plain post-tool` con `HARNESS_REPO_ROOT=$PWD`. Backup previo verificado: 2 `config.toml.bak.*` bajo `bkp/external/<ruta>` tras 2 installs. TOML valido por `tomllib` y por el **doctor real v0.29.2** (bloque puro -> rc=0 "All checked config files are valid") |
| AC-4 | cubierto | Fixture propia con config de usuario RICO (sentinel + `default_model` + provider dummy local + `[models]` + hook `UserPromptSubmit` propio): install x2 -> contenido del usuario fuera del bloque **byte a byte identico** (cmp del original vs resultado con el bloque removido), 1 solo par de marcadores, 4 `[[hooks]]` (1 usuario + 3 arnes). Caso extra mio: contenido del usuario agregado DESPUES del bloque -> sobrevive byte a byte (reubicado antes del bloque nuevo) y el TOML sigue valido. Matiz verificado: config sin newline final -> la 1a instalacion agrega `\n` a la ultima linea del usuario (mitigacion deliberada del plan contra TOML roto; a partir de ahi, byte a byte) |
| AC-5 | cubierto | Reproducido con kimi falso `exit 1`: setup rc=0 (best-effort, exit inalterado), config del usuario restaurado byte a byte, avisos accionables (`'kimi doctor' reporto config invalido... se restauro...` + ruta del backup), sin residuo `.harness.rollback`; sin config previo -> el archivo recien creado se retira. Control con el doctor REAL: TOML roto -> rc=1 "Invalid TOML"; bloque puro -> rc=0; y el doctor NO escribe en `KIMI_CODE_HOME` (fixtures quedaron con solo su config). Ver Hallazgo 1 sobre validacion de schema |
| AC-6 | cubierto | **Ciclo negativo propio de 7 casos** en fixture instalada: limpio rc=0 (cero falso positivo); stale (append) rc=2 con `Espejo desincronizado: .kimi-code/agents/reviewer.md (leido por Kimi Code) ... propaga el cambio a roles/reviewer.md`; warn rc=0 y reporta; off rc=0 con salida de 0 bytes; restaurado via re-instalador (remedio oficial) rc=0; frontmatter removido rc=2 estructural (`sin frontmatter YAML; Kimi Code no lo registrara`); `.kimi-code/` borrado por completo rc=0 (ausencia no falla). Dogfooding en ESTE checkout (sin `.kimi-code/`): `[Ok] Harness Check limpio.` rc=0. `diff harness_check.sh templates/harness_check.sh` -> **identicos**. El gate reusa `extract_agent_body` en el bucle de roles existente |
| AC-7 | cubierto | `AGENTS.md` generado en fixture: Kimi en el encabezado multi-backend, hooks globales explicados, `bin/harness-kimi` en launchers y `.kimi-code/agents/*.md` en orquestacion (lee `AGENTS.md` nativo); superficie basica de `--no-subagents` lista el launcher (smoke). `roles/README.md` con el bullet Kimi (formato, reemplazo del system prompt, `Agent`/`AgentSwarm`, `--agent` v2 con `KIMI_CODE_EXPERIMENTAL_FLAG=1`, hooks solo-globales) y `templates/roles/README.md` identico modulo `__HREL__` (diff limpio re-corrido) |
| AC-8 | cubierto | `--reset` en fixture: `.kimi-code/agents` y `bin/harness-kimi` removidos con backup previo (`*.kimi-code/agents.bak.*` presente); el config global de fixture **byte-identico** pre/post reset (cmp) con el bloque en pie (1 marcador). Comentario in-code en `reset_targets` con el motivo; `UPDATING.md` documenta que limpia el reset y la remocion manual paso a paso |
| AC-9 | cubierto | `bash tests/setup_smoke.sh` re-ejecutado por mi: **rc=0** con la linea nueva `[Ok] Kimi Code: ...` y **0 skips `[info]`** (kimi no esta en PATH: las ramas doctor-rollback y sin-deteccion corrieron de verdad). El bloque (a)-(e) leido completo y en correspondencia 1:1 con AC-9. Aislamiento verificado: `export KIMI_CODE_HOME="$TMP_ROOT/kimi-home-default"` al inicio del smoke |
| AC-10 | **parcial** | Sin `pwsh`/`powershell` (verificado). Revision estatica REAL y propia: (1) here-strings pareados 13/13 (`setup_harness.ps1`) y 6/6 (`tests/setup_smoke.ps1`: 3 preexistentes en HEAD + 3 nuevos; el impl dijo 2+3, conteo corregido); (2) **tokenizador con estados propio** (here-strings, escapes, comentarios de linea y bloque): balance de llaves = 0 y cero strings sin cerrar en working Y en HEAD de ambos archivos — senal mas fuerte que los checkers crudos, que discrepan entre si (el mio dio -1/-2 donde el del implementer dio -1/-1), confirmando que esos numeros son artefactos y no un desbalance real; (3) lectura linea a linea del delta: bloque TOML **byte-identico** al del `.sh` (diff de ambos literales), `-NoKimi` antes de la deteccion, splice por indices con manejo de CRLF tras el marcador de cierre, doctor con `KIMI_CODE_HOME` seteado/restaurado en `finally` y rollback UTF8-sin-BOM, `-Reset` sin tocar el global, `$tools` calculada POR ROL en `Write-AgentDefinitions` (allowlist decision 3) y reutilizada, dry-run explicito, launcher/backups/dirs en paridad; smoke ps1 con fixture `$env:KIMI_CODE_HOME` restaurada en `finally` y kimi falso por plataforma. SIN ejecucion |
| AC-11 | cubierto | Leido en los archivos: `README.md` seccion "Kimi Code CLI: backend con hooks globales (unica excepcion de `$HOME`)" + flags; `UPDATING.md` + `templates/UPDATING.md` con el POR QUE (hooks solo-globales verificados; decision usuario 2026-07-28), las 5 salvaguardas, los eventos y la razon de omitir `SessionEnd`, la remocion manual en 3 pasos, la guia para instalaciones existentes y la nota de acoplamiento a v0.29.x — el par raiz/template difiere SOLO en el bloque historico "Notas de robustez" (diff: 2 hunks de agregados en template); `docs/architecture.md` bullet `write_kimi_hooks`/`Write-KimiGlobalHooks`; `AGENTS.md` del checkout con Kimi en el gate de espejo |
| AC-12 | cubierto | Re-ejecutado: `cargo test --locked` **44 unit + 22 integracion, 0 fallos**; `cargo clippy --all-targets --all-features --locked -- -D warnings` rc=0; `bash tests/setup_smoke.sh` rc=0. `rust/` con **0 cambios** (git status/diff vacios: cero dependencias nuevas, base identica a la #7) |

## Las 3 decisiones del usuario (Articulo 5)

Verificadas contra el CODIGO y contra EJECUCIONES propias, no contra el impl:

1. **`--reset` NO toca el bloque global**: `reset_targets` solo lleva
   `.kimi-code/agents` + `bin/harness-kimi` del proyecto (comentario in-code
   con el motivo); ejecutado en fixture: config global byte-identico tras el
   reset, bloque en pie; remocion manual documentada en `UPDATING.md`. Paridad
   ps1 leida (`Invoke-HarnessReset` sin el global).
2. **Bloque global solo con Kimi detectado + `--no-kimi`**: rama sin deteccion
   ejecutada (kimi ni en PATH ni en `KIMI_CODE_HOME/bin`) -> fixture VACIA
   (`ls -A` = nada) + aviso `Kimi Code CLI no detectado`; rama `--no-kimi` con
   Kimi detectable y config del usuario -> config **intacto byte a byte con el
   mismo mtime**, sin backup generado (cero actividad sobre el global) + aviso
   `omitido (--no-kimi)`; en ambas, los artefactos de PROYECTO (espejos +
   launcher) se generaron igual. El flag existe en sh (`--no-kimi`) y ps1
   (`-NoKimi`, evaluado ANTES de la deteccion en ambos).
3. **`tools` allowlist por rol**: verificado en los espejos generados
   (grep exacto por rol) y en el codigo de ambos instaladores (ps1 reutiliza la
   variable `$tools` por-rol del agente Claude). El gate no compara frontmatter
   (solo cuerpo), asi que la allowlist no afecta el espejo: confirmado con el
   ciclo limpio rc=0.

## Trazabilidad y constitution

- **Articulo 1**: tests cercanos al cambio (bloque Feature #8 del smoke sh con
  las ramas (a)-(e); paridad ps1) y los tres comandos oficiales de
  `docs/verification.md` en verde, re-ejecutados en esta revision.
- **Articulo 2**: spec approved, sellado por el USUARIO y fresco antes del
  veredicto; rastro en `history.md`; implementacion posterior al sello.
- **Articulo 3**: la Delegacion U0..U7 cita sus AC; `impl-8.md` mapea
  AC-1..AC-12; este veredicto lista los 12.
- **Articulo 4**: sin secretos (providers dummy locales en fixtures, tambien en
  las mias); mensajes accionables verificados en ejecucion (deteccion, skip,
  rollback, gate) y exit codes estables (setup 0 best-effort en el bloque
  global; gate 0/2 con `HARNESS_CHECK_MODE`).
- **Articulo 5**: las 3 decisiones implementadas al pie de la letra (arriba).
  El impl declara "OBSERVACION SIN DECISION: ninguna" y no encontre en el
  codigo ningun fork que las contradiga o que inventara una decision nueva.
  La mejora posible del launcher (Hallazgo 4) quedo correctamente SIN decidir
  y SIN implementar.
- **Articulo 6**: `rust/Cargo.toml` sin cambios (cero dependencias nuevas);
  raiz y `templates/` espejados (`harness_check.sh` identico por diff;
  `roles/README.md` identico modulo `__HREL__`; `UPDATING.md` solo difiere en
  el bloque historico preexistente del template); feature backend-agnostica:
  guard por existencia, condicionalidad identica a los demas backends, y CERO
  regresion multi-LLM (las lineas `[Ok]` del smoke en HEAD estan TODAS en la
  corrida actual; el diff estatico de los `[Ok]` solo AGREGA la linea Kimi).
  Commits: no hay commits nuevos que auditar (arbol sin commitear); el mensaje
  debera ser Conventional y SIN trailers de IA.

## Impacto y gates

- `sh harness_cli graph impacto --microservicio ADR/harness` ejecutado en esta
  revision: "Ningun microservicio registrado depende de 'ADR/harness'".
  Impacto externo nulo; radio interno = exactamente el declarado en el plan
  (13 archivos modificados + 3 untracked de la feature, `rust/` intacto).
- `graphify query` registrado en el plan (111 nodos, dispatcher unico
  `bin/harness-hook` + principio `multi_llm_backend_agnostic`); `graphify-out/`
  presente y sin flag `.graphify_stale`.
- `bash harness_check.sh` limpio (rc=0) en el propio checkout, con `[plan] #8
  fresco` y `[spec] #8 approved (fresco)` — y sin `.kimi-code/` local, lo que
  ademas ejercita la condicionalidad de AC-6.
- `progress/current.md` apunta al plan y lleva la evidencia al dia (3 advances
  registrados); plan archivado en `docs/` de la raiz y fiel a lo implementado.
- Checkpoints de `CHECKPOINTS.md`: cubiertos (backlog consistente, check-plan/
  check-spec ok, sin observaciones pendientes, impacto y graphify consultados,
  tests ejecutados).
- **Coherencia del commit unico pendiente**: 13 modificados (`AGENTS.md`,
  `README.md`, `UPDATING.md`+template, `docs/architecture.md`,
  `harness_check.sh`+template, `roles/README.md`+template, `setup_harness.sh`,
  `setup_harness.ps1`, `tests/setup_smoke.sh`, `tests/setup_smoke.ps1`) + los
  untracked de la feature (`docs/spec-feature-8-*.md`, `docs/plan-feature-8-*.md`,
  `docs/impl-8.md`, este review). Los pares espejo viajan juntos (Articulo 6).

## Hallazgos de esta revision (no bloquean)

1. **`kimi doctor` valida SCHEMA, no solo sintaxis TOML** (medido con el
   doctor real v0.29.2 en fixtures): un config de usuario sintacticamente
   valido pero con issues de schema (p.ej. `[models.X]` sin
   `max_context_size`) da rc=1 ("Invalid configuration ... Validation
   issues"). Consecuencia practica: en una maquina donde el config del usuario
   YA tiene esos issues, el instalador hara rollback SIEMPRE (aviso accionable,
   bloque nunca instalado) aunque el bloque del arnes sea valido. Es el lado
   conservador-seguro del best-effort de AC-5 (el usuario nunca pierde nada) y
   el AC se cumple tal como esta escrito, pero la afirmacion del impl
   "rc=1 solo con TOML invalido, asi que el exit code es senal fiable" es
   imprecisa. Nota: mi primera medicion del rc paso por un pipe (`| tail`) y
   dio falsos rc=0; re-medido sin pipe. El bloque puro del arnes: rc=0.
2. **Normalizacion de newline en la PRIMERA escritura**: si el config del
   usuario no termina en newline, el awk agrega `\n` a su ultima linea (1 byte)
   antes de anexar el bloque. Deliberado (riesgo "archivo terminando sin
   newline" del plan: pegar el marcador a la ultima linea corromperia el TOML)
   y sin efecto en re-instalaciones (byte a byte desde entonces). AC-4, tal
   como esta redactado (bloque ya instalado -> re-correr), se cumple estricto.
3. **`--dry-run` verificado inocuo** (verificacion extra que el impl no
   declara): con Kimi detectable y config de usuario en fixture, `--dry-run`
   NO toca el config global ni genera superficies — early-exit del instalador
   (setup_harness.sh:1788) antes de toda la generacion; el ps1 ademas maneja
   `$DryRun` explicitamente dentro de `Write-KimiGlobalHooks`.
4. **Launcher `bin/harness-kimi` inutilizable TAL CUAL en esta maquina**: usa
   `command -v kimi` (mismo esqueleto que exige AC-2) y el binario real vive
   SOLO en `~/.kimi-code/bin/` (fuera del PATH aqui), asi que el launcher hara
   init + `exit 127` con mensaje claro hasta que el usuario agregue Kimi a su
   PATH (o un alias). Conforme al spec (paridad de esqueleto; el implementer
   lo declaro y NO invento un fallback); los demas artefactos Kimi funcionan
   igual si el usuario lanza `kimi` directo. Un fallback a
   `${KIMI_CODE_HOME}/bin` seria una decision nueva del usuario (Articulo 5).
5. **Hueco de aislamiento de los smoke cerrado y verificado**: el export
   `KIMI_CODE_HOME` de fixture existe al inicio de ambos smoke; en el ps1 era
   un hueco REAL (ese smoke no overridea `HOME`: en una maquina Windows con
   kimi en PATH habria escrito el bloque en el home real). Hallazgo colateral
   legitimo del implementer, confirmado en el codigo.
6. **Precision del conteo estatico del impl**: los here-strings preexistentes
   del smoke ps1 eran 3 (incluye el `Set-Content ... -Value @'` del stub de
   Cargo.toml), no 2. El pareo total 6/6 es correcto. Y la "nota de honestidad"
   del impl sobre el -1 del checker crudo se sostiene en su conclusion
   (artefacto, no desbalance: mi tokenizador de estados da 0 en HEAD y en
   working), aunque los numeros crudos varian segun el checker.
7. **Case patterns de `bin/harness-hook` confirmados en el artefacto generado**:
   `SessionStart`/`PostToolUse`/`Stop` matchean, y `SessionEnd` cae en el brazo
   de `stop` — evidencia directa de que registrar `SessionEnd` duplicaria el
   check por turno, como advierte la nota de diseno del spec.

## Pendientes (decisiones del usuario, sin decidir aqui)

1. **AC-10 sin ejecucion real en Windows**: correr `pwsh tests/setup_smoke.ps1`
   en la primera maquina Windows disponible salda la deuda acumulada de
   #1/#4/#5/#6/#7/#8. Alli tambien se sabra con que shell ejecuta Kimi los
   `command` POSIX del bloque global en Windows (declarado best-effort en
   `UPDATING.md`).
2. **PATH de Kimi en esta maquina** (Hallazgo 4): decidir si se agrega
   `~/.kimi-code/bin` al PATH del usuario, o si se pide un fallback del
   launcher a `${KIMI_CODE_HOME}/bin` como mejora aparte.
3. **Commit unico**: pendiente por decision del usuario; mensaje Conventional
   SIN trailers de IA (politica del repo).
