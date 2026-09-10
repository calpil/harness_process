# Impl - Feature #86: Copilot medido en vivo: los hooks van por el .claude/settings.json que el arnes ya genera (sin .github/copilot.json), el Stop de los dos runtimes emite decision:block, y doctor revisa la confianza de la carpeta

Spec: docs/spec-feature-86-copilot-medido-en-vivo-los-hooks-van-por-el-clau.md
Plan: docs/plan-feature-86-copilot-medido-en-vivo-los-hooks-van-por-el-clau.md

## Lo que habia

La #85 dejo un JSON de hooks en `.github/`, un bloque en el `.md` de
instrucciones de Copilot, un subcomando `copilot instalar|quitar`, un modo de
runtime y dos flags, todo construido sobre la documentacion del repo del CLI
(via Context7). El AC-8 manual, corrido con Copilot CLI 1.0.83 logueado, mostro
que nada de eso disparaba: el archivo no es de Copilot. Lo que Copilot hace es
leer `AGENTS.md` y los hooks del `.claude/settings.json` que el arnes ya
genera, solo en carpetas confiadas, y bloquear solo con el JSON
`decision: block` (medido: `{"block":true}` y exit 2 + stderr no bloquean).

## El arreglo

Un modo `claude-json` en los dos runtimes para el Stop de `.claude/settings.json`:
con el gate en rojo emite UNA linea `{"decision":"block","reason":...}` con
exit 0, el `reason` lleva la frase y las ultimas lineas del check (JSON-escapadas
con `sed`/`awk`), lo legible sigue por stderr; con el gate verde o
`stop_hook_active` no imprime nada. Claude Code honra el mismo JSON, asi que su
bloqueo no cambia de fondo. `plain` queda para Grok, Gemini y Kimi. `doctor`
gana el area `copilot`. Todo lo de la #85 que no servia se borra; `consolidar`
queda.

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | setup_harness.sh:1577 · tests/hook_runtime_check.sh:44 | El check instala un fixture, ensucia un repo hermano y alimenta el runtime: una sola linea JSON con `decision: block`, `reason` que empieza con "Harness check fallo" y contiene "sin commitear", stderr con lo legible, exit 0; con `stop_hook_active: true` y con el repo limpio, sin stdout y exit 0; `session-start` imprime el estado por stdout. Rojo contra HEAD por su aserto (el Stop invocaba `plain stop`). |
| AC-2 | setup_harness.sh:2714 · setup_harness.ps1:1609 · tests/parity_check.sh:235 · tests/hook_runtime_check.sh:31 | Los dos bloques de `.claude/settings.json` del sh y el `Get-HookCommand` del ps1 despachan `claude-json stop` con timeout; el modo `cableado-hooks` de la paridad exige ese literal (y ya no se conforma con el `plain stop` de un comando de Gemini, que era lo que lo satisfacia); el check lee el settings generado. Paridad 11/11. |
| AC-3 | setup_harness.ps1:1323 · :1457 · tests/setup_smoke.ps1:144 | `claude-json` en el `ValidateSet`; para los eventos de Stop manda `Invoke-HarnessEvent *>&1` a `[Console]::Error` y en el `catch` emite `decision: block` con el mensaje del gate, exit 0. Por lectura: no hay Windows aca. El smoke ps1 exige el `claude-json stop` en el settings generado. |
| AC-4 | rust/src/doctor.rs:76 · :158 · :186 · :231 · :894 · :910 · :931 · rust/tests/cli_basics.rs:3599 | `revisar_copilot` (PATH, `COPILOT_HOME`, checkout fuente -> no aplica) delega en `revisar_copilot_con`, puro: sin config legible o sin la raiz (o un ancestro) en `trustedFolders` -> falla con el remedio de confiar la carpeta y la ruta del config; sin `claude-json stop` en `.claude/settings.json` -> falla con reinstalar; ok con las dos. `carpeta_confiada` salta las lineas `//` del `config.json`. El test de areas de `doctor --json` incluye `copilot`. |
| AC-5 | setup_harness.sh:1 · setup_harness.ps1:1 | Borrados `rust/src/copilot.rs`, `rust/src/commands/copilot.rs`, `tests/copilot_hook_check.sh`; sin `Copilot` en `cli.rs`, sin `INSTALL_COPILOT`, `write_copilot_hooks`, `Write-CopilotHooks`, `Remove-CopilotParts` ni el modo de la #85 en los instaladores; los mapeos `sessionStart|agentStop|sessionEnd` y el `stopHookActive` vuelven atras (Copilot manda snake_case por este camino). El comando del AC pasa. |
| AC-6 | rust/src/consolidacion.rs:41 · :729 · :744 | Sin cambios respecto de la #85: `copilot -s -p` ultimo de la tabla, pista de autenticacion en el skip y en el error. |
| AC-7 | docs/review-86.md:1 | MANUAL, corrido: fixture instalado con el arnes real (`--root`), carpeta confiada temporalmente en `~/.copilot/config.json` (respaldado y restaurado byte a byte), repo hermano sucio, `copilot -p ... -s --allow-all-tools`. El shim del runtime registro: `claude-json stop` con `stop_hook_active: False` -> salida `{"decision":"block","reason":"Harness check fallo; ... Detalle:\n== Har...`; Copilot leyo el motivo (su respuesta cita "commitear trabajo ajeno a medio hacer", texto del guard) y decidio no commitear lo ajeno; segundo `claude-json stop` con `stop_hook_active: True` -> sin salida; termino con exit 0. |
| AC-8 | README.md:1145 · UPDATING.md:857 · docs/architecture.md:239 · roles/README.md:73 · AGENTS.md:125 · setup_harness.sh:1203 · rust/src/verificacion.rs:1271 | Seccion de Copilot reescrita en README y en las dos copias de UPDATING (con la nota de migracion para quien instalo la #85), bullet de `doctor` en architecture, entrada en roles/README (dos copias), bullet en el AGENTS.md de la raiz y texto AGENTS de los dos instaladores; sin mencion del archivo inexistente; AC-7 en el corpus. |

## El rojo

`tests/hook_runtime_check.sh` se corrio contra HEAD antes de tocar nada y cayo
por su aserto: `.claude/settings.json` invocaba `plain stop` (y el modo
`claude-json` no existia: stdout vacio, exit 2). Los unitarios de `doctor`
nacieron con el area.

Mutaciones, con `cmp` antes y despues, restauracion y `touch`:
- `claude-json stop` no emite el JSON al fallar: cae el check ("stdout no es
  una sola linea").
- el Stop de Claude vuelve a `plain`: caen el check y el modo `cableado-hooks`
  de la paridad ("esperaba 2 hooks al runtime y hay 0").
- `reason` sin el detalle: cae el check ("no es el esperado").

## Lo medido con el CLI (sesion logueada, 2026-09-10)

Con carpetas de prueba confiadas temporalmente (config respaldado y restaurado):
`.github/hooks/*.json` (formato oficial) y `.claude/settings.json` (formato
Claude) disparan los dos; en carpetas no confiadas ninguno, y `--add-dir` no
alcanza. Eventos via `.claude/settings.json`: `SessionStart`, `UserPromptSubmit`,
`PreToolUse`, `PostToolUse`, `Stop` (una vez al terminar el turno; tras un
bloqueo vuelve con `stop_hook_active: True`), `SessionEnd`. Bloqueo: solo
`{"decision":"block","reason"}` en las dos fuentes; `{"block":true}` y exit 2 +
stderr no bloquean. `PreToolUse` niega con el JSON anidado que el arnes ya
emite. `copilot -s -p` devuelve solo la respuesta.

## Estilo (skills cargados)

`rust-patterns` / `rust-best-practices`: `revisar_copilot_con` y
`carpeta_confiada` son puros y se prueban sin PATH ni entorno; el wrapper es
la unica capa que lee `COPILOT_HOME` y el PATH. `posix-shell-pro`: el JSON del
runtime se arma con `printf` y un pipeline `tr`/`sed`/`awk` sin bashismos
raros, bajo `set -Eeuo pipefail` con el `&& rc=0 || rc=$?` para no morir en el
fallo que justamente hay que reportar. `rust-testing`: el check de shell cubre
los cuatro estados del Stop; mutantes por AC. `find-skills`: sin skill nueva.
