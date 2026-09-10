# Spec - Feature #86: Copilot medido en vivo: los hooks van por el .claude/settings.json que el arnes ya genera (sin .github/copilot.json), el Stop de los dos runtimes emite decision:block, y doctor revisa la confianza de la carpeta

Estado: approved
Aprobado: 2026-09-10T02:15:40Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado' a los ocho AC; OBS-1 modo nuevo claude-json; OBS-2 reason con la frase y las ultimas lineas del check
Plan: docs/plan-feature-86-copilot-medido-en-vivo-los-hooks-van-por-el-clau.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)
ANTES: Alan hace `/login` en Copilot y corremos el AC-8 manual de la #85. El
`.github/copilot.json` que el instalador escribio no dispara nada: no es un
archivo de Copilot. La documentacion del repo del CLI (via Context7) que
sostuvo la #85 no coincide con el CLI real ni con docs.github.com. Copilot
lee, en cambio, el `.claude/settings.json` que el arnes ya genera, pero solo
bloquea con el JSON `{"decision":"block","reason":...}`: el exit 2 + stderr
que hoy emite el Stop de Claude no lo frena. Y solo carga hooks en carpetas
confiadas (`trustedFolders` de `~/.copilot/config.json`).
DESPUES: no se escribe nada nuevo para Copilot. El Stop de `.claude/settings.json`
(bash y PowerShell) emite `{"decision":"block","reason":"..."}` con exit 0, que
Claude Code y Copilot honran igual, y el `reason` trae el detalle del check.
`doctor` dice si la carpeta esta confiada y como confiarla. Lo de la #85 que
no servia (`.github/copilot.json`, el bloque, `harness copilot instalar|quitar`,
el modo `copilot-json`, `--copilot`/`--no-copilot`) se va; queda `copilot -s -p`
en `consolidar`, que si funciono.

## Hechos medidos (Copilot CLI 1.0.83 logueado, 2026-09-10)
- `copilot -s -p 'Responde solo con {"ok":1}'` devuelve exactamente el JSON.
- Hooks: se cargan de `.github/hooks/*.json` (formato oficial de docs.github.com:
  `version: 1`, arrays con `bash`/`powershell`/`timeoutSec`) Y del
  `.claude/settings.json` del repo (formato Claude, tal cual lo genera el arnes);
  con las dos fuentes corren las dos. Solo en carpetas de `trustedFolders`
  (`~/.copilot/config.json`); `--add-dir` no alcanza.
- Eventos via `.claude/settings.json`: `SessionStart`, `UserPromptSubmit`,
  `PreToolUse`, `PostToolUse`, `Stop`, `SessionEnd`, con los nombres y campos
  snake_case de Claude (`hook_event_name`, `tool_input`, `stop_hook_active`).
  `Stop` (= `agentStop` en el formato oficial) dispara UNA vez al terminar el
  turno; tras un bloqueo vuelve con `stop_hook_active: true`.
- Bloqueo: SOLO `{"decision":"block","reason":"..."}` por stdout (en las dos
  fuentes). `{"block":true,"reason"}` (docs del repo del CLI) y exit 2 + stderr
  (protocolo no-JSON de Claude) NO bloquean. `PreToolUse` niega con el JSON
  anidado `hookSpecificOutput.permissionDecision`, el que ya emite el arnes.
- `.github/copilot.json` no existe para Copilot; la config de proyecto oficial
  es `.github/copilot/settings.json` y no hace falta.

## Hoy -> Como va a funcionar
```
HOY (#85)                                        DESPUES
instalador -> .github/copilot.json (no sirve)    instalador -> nada nuevo para Copilot
             .github/copilot-instructions.md     .claude/settings.json: Stop -> bin/harness-hook claude-json stop
.claude/settings.json: Stop -> harness-hook      claude-json stop: gate rojo -> stdout {"decision":"block","reason":<detalle>} exit 0
             plain stop (exit 2: Copilot no lo                    gate verde -> sin stdout, exit 0
             entiende)                                            (Claude Code honra el JSON; Copilot tambien)
doctor: copilot por .github/copilot.json         doctor: copilot = `copilot` en PATH + raiz en trustedFolders + Stop claude-json
consolidar: copilot -s -p                        consolidar: igual
```
`plain` sigue existiendo para Grok, Gemini (comandos `/harness:*`) y Kimi, que
usan el exit code; `codex-json` y `gemini-json` no cambian.

## Recorridos de usuario (priorizados)
- P1: Como usuario que abre un proyecto con arnes desde Copilot CLI (carpeta
  confiada), quiero el mismo commit guard que con Claude: al terminar el
  turno con el repo sucio, Copilot recibe el motivo y sigue trabajando.
- P1: Como usuario de Claude Code, quiero que nada cambie: el Stop sigue
  bloqueando con el detalle del check.
- P2: Como usuario que corre `harness doctor` con Copilot, quiero saber si la
  carpeta esta confiada y como confiarla; sin Copilot, que no diga nada.
- P2: Como usuario que instalo la #85, quiero que no queden archivos muertos
  en `.github/` ni flags que no hacen nada.

## Criterios de aceptacion (Given/When/Then)
- AC-1: Given un proyecto instalado con un repo hermano sucio, When se corre
  `printf '{"stop_hook_active":false}' | bash bin/harness-hook claude-json stop`,
  Then stdout es UNA linea JSON `{"decision":"block","reason":"..."}` cuyo
  `reason` empieza con "Harness check fallo" y contiene el detalle del check
  (por ejemplo "Cambios sin commitear"), lo legible sigue saliendo por stderr
  y el exit es 0; con `stop_hook_active: true` no hay stdout y el exit es 0;
  con el repo limpio no hay stdout y el exit es 0; `claude-json session-start`
  imprime el estado por stdout (contexto) con exit 0.
  Comando: `bash tests/hook_runtime_check.sh`
- AC-2: Given los dos instaladores, When escriben `.claude/settings.json`
  (con y sin subagentes), Then el Stop invoca `bin/harness-hook claude-json stop`
  (`harness-hook.ps1 claude-json stop` en Windows) con su timeout; los demas
  eventos y superficies (Kimi, Gemini, Grok, Codex) siguen igual, y la paridad
  lo verifica en su modo `cableado-hooks`.
  Comando: `bash tests/parity_check.sh && grep -q 'claude-json stop' setup_harness.sh setup_harness.ps1`
- AC-3: Given el runtime PowerShell en modo `claude-json`, When el gate de
  Stop falla, Then emite el mismo JSON `decision: block` con el motivo, exit
  0, y lo legible va a stderr; con el gate verde no imprime nada. (Por
  lectura y por el smoke ps1: no hay Windows aca.)
  Comando: `grep -q '"claude-json"' setup_harness.ps1 && grep -q 'decision = "block"' setup_harness.ps1`
- AC-4: Given `copilot` en el PATH, When corre `harness doctor`, Then el area
  `copilot` falla si la raiz no esta en `trustedFolders` de
  `${COPILOT_HOME:-~/.copilot}/config.json` (remedio: abrir `copilot` en la
  carpeta y aceptar la confianza, o agregar la ruta a `trustedFolders`) o si
  `.claude/settings.json` no invoca `claude-json stop` (remedio: reinstalar);
  ok con las dos; sin `copilot` en el PATH, no aplica y no dice nada.
  Comando: `cd rust && cargo test --locked doctor_copilot`
- AC-5: Given el repo, When se cierra, Then no quedan `rust/src/copilot.rs`,
  `harness copilot instalar|quitar`, el modo `copilot-json`, `--copilot`/
  `--no-copilot`, `write_copilot_hooks`, `Write-CopilotHooks`, ni
  `tests/copilot_hook_check.sh`; `harness copilot` es un subcomando desconocido.
  Comando: `! grep -q 'copilot-json\|INSTALL_COPILOT\|write_copilot_hooks\|Write-CopilotHooks\|Remove-CopilotParts' setup_harness.sh setup_harness.ps1 && ! test -e rust/src/copilot.rs && ! test -e rust/src/commands/copilot.rs && ! test -e tests/copilot_hook_check.sh && ! ./harness copilot --help >/dev/null 2>&1`
- AC-6: Given `copilot` en el PATH y ni `claude` ni `kimi`, When `consolidar`
  resuelve el backend, Then sigue eligiendo `copilot -s -p` y el error sin
  sesion agrega como autenticarse.
  Comando: `cd rust && cargo test --locked backend_copilot`
- AC-7 (MANUAL): Given Copilot logueado y un fixture instalado con el arnes
  en una carpeta confiada, con un repo hermano sucio, When se corre una sesion
  `copilot -p` que edite algo y termine, Then Copilot recibe el `reason` del
  Stop y sigue, y el segundo Stop llega con `stop_hook_active: true`. Se
  registra en el review con lo medido.
- AC-8: Given README, `UPDATING.md` (dos copias), `docs/architecture.md`,
  `roles/README.md` (dos copias), el `AGENTS.md` de la raiz y el texto AGENTS
  de los dos instaladores, When se cierra, Then dicen que Copilot lee
  `AGENTS.md` y los hooks de `.claude/settings.json`, que hace falta confiar
  la carpeta, que bloquea con `decision: block`, y que no se escribe nada en
  `.github/`; las copias son identicas.
  Comando: `cmp UPDATING.md templates/UPDATING.md && cmp roles/README.md templates/roles/README.md && grep -q "trustedFolders" README.md UPDATING.md && ! grep -q "copilot.json" README.md UPDATING.md docs/architecture.md roles/README.md AGENTS.md`

## Los datos que se tocan
- disparador: el Stop de `.claude/settings.json` (Claude Code y Copilot),
  `harness doctor`, `consolidar`.
- interruptor: `HARNESS_COMMIT_GUARD_MODE=warn` sigue apagando el bloqueo;
  `stop_hook_active` sigue cortando el bucle.
- candado: no aplica; nada se escribe en el proyecto del usuario. `trustedFolders`
  es del usuario y `doctor` solo lo lee.
- lo que se borra del arnes: el modulo y comando `copilot`, el modo
  `copilot-json`, los flags, el check de shell y las secciones de docs de la
  #85 que describian archivos que no existen.

## Pseudo-codigo (el acuerdo)
```
bin/harness-hook claude-json <evento>:
  stop   -> salida = run_stop (capturada); mostrarla por stderr
            rc != 0 -> stdout {"decision":"block","reason":"Harness check fallo; ... Detalle:\n<ultimas lineas, JSON-escapadas>"} ; exit 0
            rc == 0 -> sin stdout ; exit 0
  otros  -> como plain (stdout = contexto)

instaladores: .claude/settings.json Stop -> "<runtime> claude-json stop" (los dos bloques; sh y ps1)

doctor revisar_copilot:
  ¿copilot en PATH?              -> no: no_aplica
  ¿raiz en trustedFolders?       -> no: falla + remedio (abrir copilot en la carpeta / editar config)
  ¿.claude/settings.json con claude-json stop? -> no: falla + reinstalar
  ok
```
Promesas: Claude Code bloquea igual que hoy (mismo detalle, ahora en `reason`) ·
Copilot bloquea con la carpeta confiada · nada nuevo en el proyecto · `plain`
no cambia.

## No funcionales
- SLOs: el hook corre lo mismo que hoy; el JSON agrega un `sed` sobre unas
  lineas.
- Seguridad: `doctor` lee `config.json` de Copilot; no lo escribe.
- Observabilidad: el `reason` lleva el detalle; stderr lo repite para el usuario.

## Fuera de alcance
- Escribir `.github/hooks/*.json` (formato oficial): duplicaria los hooks con
  `.claude/settings.json` (Copilot corre las dos fuentes).
- Confiar la carpeta por el arnes: `trustedFolders` es del usuario.
- Agentes en `.github/agents/`.

## Observaciones (decisiones pendientes)
- OBS-1 (modo nuevo `claude-json` vs cambiar `plain`): propuesta: modo nuevo,
  usado solo por `.claude/settings.json`; `plain` sigue con exit code para
  Grok, Gemini (`/harness:check`) y Kimi, que no se pudieron medir con JSON.
  Alternativa: cambiar `plain` para todos. DECIDIDO (Alan, 2026-09-10): modo
  nuevo `claude-json`.
- OBS-2 (que va en `reason`): propuesta: la frase generica + las ultimas
  lineas del check (escapadas), para que el modelo vea el detalle como hoy lo
  ve por stderr. Alternativa: solo la frase, como `codex-json`. DECIDIDO (Alan,
  2026-09-10): frase + ultimas lineas del check.
