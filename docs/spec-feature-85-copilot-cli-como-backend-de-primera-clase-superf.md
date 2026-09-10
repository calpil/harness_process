# Spec - Feature #85: Copilot CLI como backend de primera clase: superficie .github/copilot-instructions.md, hooks en .github/copilot.json, agentes en .github/agents, fila en doctor y backend LLM en la tabla de CLIs

Estado: approved
Aprobado: 2026-09-10T00:41:54Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado' a los nueve AC; OBS-1 bloque corto con marcadores; OBS-2 solo con copilot detectado o --copilot; OBS-3 agentes fuera de alcance; OBS-4 agentStop bloqueante, verificable con login
Plan: docs/plan-feature-85-copilot-cli-como-backend-de-primera-clase-superf.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)
ANTES: Alan pregunta si el arnes esta configurado para Copilot. No lo esta en
ningun punto: Copilot no aparece ni una vez en el repo. Quien abre un proyecto
con arnes desde Copilot CLI recibe el `AGENTS.md` (Copilot lo lee solo), pero
sin Stop hook no hay commit guard, `harness doctor` no lo ve, y `consolidar`
no lo encuentra aunque este en el PATH.
DESPUES: Alan corre el instalador en una maquina con `copilot`. Le quedan
`.github/copilot.json` con los hooks del arnes (sessionStart, agentStop
bloqueante, sessionEnd), respetando lo que ya hubiera en ese archivo, y un
bloque del arnes en `.github/copilot-instructions.md` que apunta al `AGENTS.md`
sin duplicarlo. Al terminar un turno, el commit guard bloquea a Copilot igual
que a Claude, con el motivo. `doctor` lo lista con los demas y `consolidar` lo
usa si no hay `claude` ni `kimi`. Un `--reset` deja los dos archivos como
estaban antes del arnes.

## Hechos medidos (Copilot CLI 1.0.83, `copilot --help` y la documentacion)
- `copilot -p "<prompt>"` es el modo no interactivo (el prompt va por argv,
  como en `claude -p`); `-s`/`--silent` imprime solo la respuesta del agente;
  `--output-format json` es JSONL. Sin sesion de GitHub el CLI sale 1 con
  "No authentication information found" (COPILOT_GITHUB_TOKEN, GH_TOKEN,
  GITHUB_TOKEN o `/login`).
- `--no-custom-instructions` "disable loading of custom instructions from
  AGENTS.md and related files": Copilot lee `AGENTS.md` nativamente. La guia
  de proyecto documentada es `.github/copilot-instructions.md`.
- Config de proyecto: `.github/copilot.json` con `hooks` (un comando por
  evento: `sessionStart`, `userPromptSubmitted`, `agentStop`, `sessionEnd`,
  `toolCall`, `dirChange`; `command` + `shell`, opcional `timeout`).
  `agentStop` recibe por stdin `{event, sessionId, toolName, toolInput,
  toolOutput, stopHookActive, timestamp}` y responde `{block, reason}`; tras 8
  bloqueos seguidos el CLI fuerza la continuacion con `stopHookActive: true`.
- `.github/agents` es configuracion confiable y `--agent <nombre>` la usa; el
  formato del archivo de agente no pudo verificarse en vivo (sin login).
- La documentacion describe `agentStop` como "fires after each agent tool
  call"; existe ademas un evento `toolCall` separado y `stopHookActive` copia
  la semantica del Stop de Claude. Sin login no se pudo medir cual es.

## Hoy -> Como va a funcionar
```
HOY                                        DESPUES
instalador: CLAUDE.md AGENTS.md GEMINI.md  instalador (copilot en PATH, o --copilot):
            LLM.md + hooks claude/codex/     -> harness copilot instalar --raiz <r> --hook "<cmd>"
            gemini/grok/kimi                     .github/copilot.json  (merge: hooks del arnes, lo ajeno queda)
                                                 .github/copilot-instructions.md (bloque con marcadores)
                                            --no-copilot omite; --reset -> harness copilot quitar
bin/harness-hook plain|gemini-json|codex-json   + copilot-json: sessionStart -> session-start
                                                                agentStop -> stop (lee stopHookActive)
                                                                             {"block":true,"reason":...} si falla
                                                                sessionEnd -> check informativo, {}
doctor: claude codex gemini grok            + copilot (.github/copilot.json -> bin/harness-hook)
consolidar: claude -p | kimi -p             + copilot -s -p (ultimo de la tabla)
```

## Recorridos de usuario (priorizados)
- P1: Como usuario que trabaja con Copilot CLI, quiero que el arnes le ponga
  sus hooks y su guia sin pisar lo que ya tengo en `.github/`, para tener el
  mismo commit guard que con Claude.
- P1: Como usuario con Copilot y sin Claude ni Kimi, quiero que `consolidar`
  lo use solo, y que si no esta autenticado el mensaje me diga como.
- P2: Como usuario que corre `harness doctor`, quiero ver a Copilot con los
  demas backends y que me avise si su hook no apunta al runtime.
- P2: Como usuario que desinstala (`--reset`), quiero que `.github/copilot.json`
  y `copilot-instructions.md` queden como antes del arnes.

## Criterios de aceptacion (Given/When/Then)
- AC-1: Given una raiz con o sin `.github/copilot.json`, When corre
  `harness copilot instalar --raiz <raiz> --hook "<comando>"`, Then el archivo
  queda con `hooks.sessionStart`, `hooks.agentStop` y `hooks.sessionEnd`
  apuntando al comando (`<comando> copilot-json <evento>`, `shell: bash`),
  las claves ajenas del JSON y los hooks ajenos de OTROS eventos quedan
  intactos, un hook ajeno en uno de esos tres eventos NO se pisa (se avisa y
  se deja), y correrlo dos veces deja los mismos bytes.
  Comando: `cd rust && cargo test --locked copilot_instalar`
- AC-2: Given una raiz con o sin `.github/copilot-instructions.md`, When
  corre `instalar`, Then el archivo tiene UN bloque del arnes entre marcadores
  (`<!-- harness:copilot:inicio -->` / `fin`) que dice que el flujo, los roles
  y el perfil estan en `AGENTS.md` (Copilot lo carga solo) y que el commit
  guard corre en `agentStop`; el texto del usuario fuera del bloque queda
  intacto y correrlo dos veces no duplica el bloque.
  Comando: `cd rust && cargo test --locked copilot_instrucciones`
- AC-3: Given los dos archivos escritos por `instalar` sobre contenido ajeno,
  When corre `harness copilot quitar --raiz <raiz>`, Then quedan solo las
  partes ajenas (bytes iguales a los previos), y un archivo que solo tenia lo
  del arnes se borra; sin archivos no falla.
  Comando: `cd rust && cargo test --locked copilot_quitar`
- AC-4: Given `bin/harness-hook copilot-json <evento>` con el JSON de Copilot
  por stdin, When el evento es `agentStop`, Then corre el gate de Stop
  (`commit_guard`/`harness_check`) leyendo `stopHookActive` (camelCase) como
  hoy lee `stop_hook_active`, y emite `{"block":true,"reason":"..."}` si el
  gate falla y `{"block":false}` si pasa, siempre con exit 0; `sessionStart`
  corre el inicio de sesion y emite `{}`; `sessionEnd` corre el check de
  forma informativa (stderr) y emite `{}`. El runtime PowerShell hace lo
  mismo con `copilot-json`.
  Comando: `bash tests/copilot_hook_check.sh`
- AC-5: Given el instalador `.sh`, When `copilot` esta en el PATH o se paso
  `--copilot`, Then llama a `harness copilot instalar` con el comando del
  runtime; con `--no-copilot` lo omite y lo dice; sin `copilot` y sin
  `--copilot` no toca `.github/`; `--reset` llama a `quitar`; el texto AGENTS
  nombra a Copilot. El `.ps1` hace lo mismo (`-Copilot`/`-NoCopilot`,
  runtime `harness-hook.ps1` con `shell: powershell`), y la paridad y los dos
  smokes lo cubren con un `copilot` falso en el PATH.
  Comando: `bash tests/parity_check.sh && bash tests/setup_smoke.sh`
- AC-6: Given `.github/copilot.json` en la raiz, When corre `harness doctor`,
  Then lista `copilot` entre los backends instalados y falla si su hook no
  apunta a `bin/harness-hook`; sin el archivo no lo nombra.
  Comando: `cd rust && cargo test --locked doctor_copilot`
- AC-7: Given `copilot` en el PATH y ni `claude` ni `kimi`, When `consolidar`
  resuelve el backend, Then elige `copilot -s -p <prompt>`; con `claude` o
  `kimi` presentes, esos ganan; y el mensaje de "sin backend" nombra la
  autenticacion de Copilot (`COPILOT_GITHUB_TOKEN`/`GH_TOKEN` o `/login`).
  Comando: `cd rust && cargo test --locked backend_copilot`
- AC-8 (MANUAL): Given Copilot CLI autenticado (`/login`), When se corre
  `copilot -s -p 'Responde solo con {"ok":1}'` y una sesion `-p` en un
  proyecto de prueba con los hooks instalados que haga dos llamadas a bash y
  termine, Then la respuesta es el JSON pedido y el registro de eventos dice
  cuantas veces disparo `agentStop` (una al parar, o una por tool call) y que
  el `block`/`reason` del gate se muestra. El resultado se escribe en el
  review y decide si `agentStop` queda bloqueante (OBS-4).
- AC-9: Given README, `UPDATING.md` (dos copias), `docs/architecture.md`,
  `roles/README.md`, el texto AGENTS de los instaladores y el `AGENTS.md` de
  la raiz, When se cierra, Then nombran a Copilot (que lee `AGENTS.md`, sus
  archivos, sus flags, su lugar en `doctor` y `consolidar`, y que los
  agentes de `.github/agents` quedan fuera hasta verificar el formato).
  Comando: `cmp UPDATING.md templates/UPDATING.md && grep -q "copilot" README.md docs/architecture.md roles/README.md AGENTS.md setup_harness.sh setup_harness.ps1`

## Los datos que se tocan
- disparador: el instalador (`copilot` en PATH o `--copilot`), `--reset`,
  `harness doctor`, `consolidar`, y los eventos de Copilot.
- interruptor: `--no-copilot` / `-NoCopilot`; `HARNESS_CONSOLIDAR_CMD` sigue
  ganando a la tabla.
- candado: `instalar` y `quitar` son idempotentes; el JSON se mezcla, no se
  reemplaza; el bloque de instrucciones va entre marcadores.
- archivos del usuario que se tocan: `.github/copilot.json` (claves ajenas
  intactas) y `.github/copilot-instructions.md` (texto ajeno intacto). Se
  respaldan en `bkp/` antes de escribir, como toda superficie.

## Pseudo-codigo (el acuerdo)
```
instalador (sh y ps1):
  ¿--no-copilot?                          -> omitir y decirlo
  ¿copilot en PATH o --copilot?           -> si no, no tocar .github/
  respaldar los dos archivos si existen
  harness copilot instalar --raiz <raiz> --hook "<runtime>"   (el formato vive en el binario)
  --reset: harness copilot quitar --raiz <raiz>

harness copilot instalar:
  json = leer .github/copilot.json o {}
  por evento en [sessionStart, agentStop, sessionEnd]:
     ¿hooks.<evento> ajeno (sin "harness-hook")? -> avisar y dejar
     si no -> hooks.<evento> = {command: "<hook> copilot-json <evento>", shell: "bash"}
  escribir atomico si cambio
  instrucciones = leer .github/copilot-instructions.md o ""
  reemplazar o agregar el bloque entre marcadores; escribir si cambio

bin/harness-hook copilot-json <evento>:
  sessionStart -> run_session_start >&2 ; {}
  agentStop    -> stop_hook_active desde stopHookActive ; run_stop >&2
                  ok -> {"block":false} ; fallo -> {"block":true,"reason":"Harness check fallo; ..."}
  sessionEnd   -> run_stop >&2 (informa) ; {}
```
Promesas: nunca pisa un hook ajeno · nunca borra texto ajeno · exit 0 siempre
en el hook (el CLI lee el JSON) · sin `copilot` no toca `.github/`.

## No funcionales
- SLOs: sin red; el hook corre lo mismo que hoy corren los de Claude.
- Seguridad: no escribe tokens; la autenticacion es del usuario.
- Observabilidad: `doctor` nombra el backend y su hook; el `reason` del block
  es el mismo texto que ven Codex y Gemini.

## Fuera de alcance
- Agentes en `.github/agents/`: el formato no se pudo verificar en vivo;
  queda como feature chica cuando haya login (OBS-3).
- El tramo de API key para Copilot (no hay API publica del CLI).
- `userPromptSubmitted`, `toolCall` y `dirChange`: no tienen equivalente en
  el arnes hoy.

## Observaciones (decisiones pendientes)
- OBS-1 (que va en `.github/copilot-instructions.md`): propuesta: un bloque
  corto entre marcadores que apunta a `AGENTS.md` (Copilot carga los dos
  archivos: copiar `AGENTS.md` entero duplicaria el prompt). Alternativa: copia
  completa de `AGENTS.md`. DECIDIDO (Alan, 2026-09-09): bloque corto con
  marcadores.
- OBS-2 (cuando se genera): propuesta: solo con `copilot` en el PATH o
  `--copilot` explicito (como Kimi: no meter archivos en `.github/` de un
  proyecto que no usa Copilot). Alternativa: siempre, como Codex y Gemini.
  DECIDIDO (Alan, 2026-09-09): solo con `copilot` detectado o `--copilot`.
- OBS-3 (agentes en `.github/agents/`): propuesta: fuera de alcance hasta
  verificar el formato con el CLI autenticado; mientras tanto Copilot usa los
  roles de `AGENTS.md` como fases, igual que Antigravity. Alternativa:
  generarlos a ciegas en el formato documentado para VS Code. DECIDIDO (Alan,
  2026-09-09): fuera de alcance hasta verificar.
- OBS-4 (`agentStop` bloqueante): propuesta: bloqueante, con `stopHookActive`
  como corte, y el AC-8 manual decide si se queda asi (si disparara por cada
  tool call, bloquearia a mitad del trabajo y pasaria a informativo en
  `sessionEnd`). Alternativa: solo informar desde el principio. DECIDIDO (Alan,
  2026-09-09): bloqueante, verificable con login.
