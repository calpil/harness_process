Aplicado: 2026-09-10T02:34:12Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #86: Copilot medido en vivo: los hooks van por el .claude/settings.json que el arnes ya genera (sin .github/copilot.json), el Stop de los dos runtimes emite decision:block, y doctor revisa la confianza de la carpeta

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 86`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:101 (spec `evento`) y 401 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `README.md`, `UPDATING.md`, `docs/architecture.md` y 20 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 27 | Copilot CLI es un backend del arnes | copilot_backend | <O1> | Copilot no aparecia en ningun punto del arnes (pregunta del usuario del 2026-09-09). Ahora esta en los cuatro lugares donde viven los backends: el instalador (sh y ps1), solo con `copilot` en el PATH o `--copilot`/`-Copilot` (`--no-copilot` lo omite), llama a `harness copilot instalar`, que MEZCLA los hooks `sessionStart`/`agentStop`/`sessionEnd` sobre el `.github/copilot.json` del usuario sin pisar claves ni hooks ajenos y deja un bloque entre marcadores en `.github/copilot-instructions.md` que apunta a `AGENTS.md` (Copilot lo lee solo); `--reset` corre `copilot quitar` y devuelve lo ajeno. El runtime de hooks (`bin/harness-hook` y `harness-hook.ps1`) gana el modo `copilot-json`: `agentStop` es el Stop y responde `{"block":true,"reason":...}` o `{"block":false}` leyendo `stopHookActive` como corte. `doctor` lista `copilot` por su huella y `consolidar` lo detecta como `copilot -s -p` despues de `claude` y `kimi`, con la autenticacion en el skip. Medido contra `copilot --help` de la 1.0.83 y la documentacion; la semantica exacta de `agentStop` queda por medir con sesion (AC-8 manual). Fuera: agentes en `.github/agents/` hasta verificar el formato. Disparador: el usuario pregunto si el arnes estaba configurado para Copilot y no lo estaba | done (2026-09-09) |
Despues:
| 27 | Copilot CLI es un backend del arnes | copilot_backend | <O1> | Copilot no aparecia en ningun punto del arnes (pregunta del usuario del 2026-09-09). Ahora esta en los cuatro lugares donde viven los backends: el instalador (sh y ps1), solo con `copilot` en el PATH o `--copilot`/`-Copilot` (`--no-copilot` lo omite), llama a `harness copilot instalar`, que MEZCLA los hooks `sessionStart`/`agentStop`/`sessionEnd` sobre el `.github/copilot.json` del usuario sin pisar claves ni hooks ajenos y deja un bloque entre marcadores en `.github/copilot-instructions.md` que apunta a `AGENTS.md` (Copilot lo lee solo); `--reset` corre `copilot quitar` y devuelve lo ajeno. El runtime de hooks (`bin/harness-hook` y `harness-hook.ps1`) gana el modo `copilot-json`: `agentStop` es el Stop y responde `{"block":true,"reason":...}` o `{"block":false}` leyendo `stopHookActive` como corte. `doctor` lista `copilot` por su huella y `consolidar` lo detecta como `copilot -s -p` despues de `claude` y `kimi`, con la autenticacion en el skip. Medido contra `copilot --help` de la 1.0.83 y la documentacion; la semantica exacta de `agentStop` queda por medir con sesion (AC-8 manual). Fuera: agentes en `.github/agents/` hasta verificar el formato. Disparador: el usuario pregunto si el arnes estaba configurado para Copilot y no lo estaba | done (2026-09-09) |
| 28 | Copilot, medido: los hooks van por el `.claude/settings.json` que ya existe | copilot_medido | <O1> | El AC-8 manual de la #85, corrido con Copilot CLI 1.0.83 logueado, dio vuelta el hito 27: el JSON de `.github/` que el instalador escribia no dispara nada (no es un archivo de Copilot; la documentacion del repo del CLI no coincide con el CLI), Copilot lee `AGENTS.md` y los hooks del `.claude/settings.json` del repo (formato Claude, mismos eventos y campos, `Stop` una vez al terminar el turno con `stop_hook_active`), solo en carpetas de `trustedFolders` (`~/.copilot/config.json`), y bloquea SOLO con `{"decision":"block","reason":...}` por stdout: ni `{"block":true}` ni exit 2 + stderr. Ahora no se escribe nada para Copilot: el Stop de `.claude/settings.json` invoca `bin/harness-hook claude-json stop` (y `harness-hook.ps1 claude-json stop`), que con el gate en rojo emite esa linea con exit 0 y el detalle del check en `reason` (Claude Code honra el mismo JSON: su bloqueo no cambia); `plain` queda para Grok, Gemini y Kimi. `harness doctor` gana el area `copilot` (en PATH, carpeta confiada, Stop en `claude-json`; sin `copilot`, no aplica) con el remedio para confiar la carpeta. Se van el JSON de `.github/`, el bloque, `harness copilot instalar\|quitar`, el modo de hook de la #85 y `--copilot`/`--no-copilot`; `consolidar` sigue con `copilot -s -p`. Medido al cerrar sobre un fixture instalado y confiado, con un repo hermano sucio: Copilot recibio el `reason`, volvio con `stop_hook_active: true` y termino. Disparador: la medicion con sesion que la #85 dejo pendiente | done (2026-09-10) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:101 (módulo `cuando`), docs/prd/SDD-master.md:103 (módulo `cuando`) y 424 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `README.md`, `UPDATING.md`, `docs/architecture.md` y 20 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D14 | Copilot entra como backend con lo que se pudo medir: los archivos de `.github/` se mezclan (nunca se pisan), se generan solo con `copilot` detectado o `--copilot`, el formato vive en el binario (`harness copilot instalar\|quitar`), `agentStop` es el Stop bloqueante y los agentes de `.github/agents/` quedan fuera hasta verificar el formato | (a) copiar `AGENTS.md` entero en `copilot-instructions.md`; (b) generar siempre, como Codex y Gemini; (c) agentes a ciegas en el formato documentado para VS Code; (d) solo informar en `sessionEnd` sin bloquear nunca | Copilot carga `AGENTS.md` y `copilot-instructions.md`: copiar uno en el otro duplica el prompt en cada sesion. `.github/` es del usuario y muchos repos ya tienen esos archivos: meterlos en proyectos sin Copilot es ruido, y pisarlos es perder trabajo ajeno (por eso mezcla, marcadores y respaldo). Un formato de agentes que no se pudo verificar es un archivo muerto (leccion remedios-que-la-herramienta-sugiere). `agentStop` trae `stopHookActive` con la semantica del Stop de Claude y hay un `toolCall` aparte: la lectura mas probable es que sea el Stop; sin login no se midio, y el AC-8 decide. Decisiones del usuario del 2026-09-09 (OBS-1..4 de la #85) | 2026-09-09 |
Despues:
| D14 | Copilot entra como backend con lo que se pudo medir: los archivos de `.github/` se mezclan (nunca se pisan), se generan solo con `copilot` detectado o `--copilot`, el formato vive en el binario (`harness copilot instalar\|quitar`), `agentStop` es el Stop bloqueante y los agentes de `.github/agents/` quedan fuera hasta verificar el formato | (a) copiar `AGENTS.md` entero en `copilot-instructions.md`; (b) generar siempre, como Codex y Gemini; (c) agentes a ciegas en el formato documentado para VS Code; (d) solo informar en `sessionEnd` sin bloquear nunca | Copilot carga `AGENTS.md` y `copilot-instructions.md`: copiar uno en el otro duplica el prompt en cada sesion. `.github/` es del usuario y muchos repos ya tienen esos archivos: meterlos en proyectos sin Copilot es ruido, y pisarlos es perder trabajo ajeno (por eso mezcla, marcadores y respaldo). Un formato de agentes que no se pudo verificar es un archivo muerto (leccion remedios-que-la-herramienta-sugiere). `agentStop` trae `stopHookActive` con la semantica del Stop de Claude y hay un `toolCall` aparte: la lectura mas probable es que sea el Stop; sin login no se midio, y el AC-8 decide. Decisiones del usuario del 2026-09-09 (OBS-1..4 de la #85) | 2026-09-09 |
| D15 | Copilot no tiene archivos propios en el arnes: usa los hooks de `.claude/settings.json` (que ya lee), el Stop de Claude pasa a un modo `claude-json` que emite `decision: block` con el detalle, `plain` no cambia, y confiar la carpeta queda del lado del usuario (doctor lo revisa). Reemplaza a D14 | (a) escribir `.github/hooks/*.json` en el formato oficial; (b) cambiar `plain` para todos los backends; (c) solo la frase generica en el `reason`; (d) confiar la carpeta desde el instalador editando `~/.copilot/config.json` | El formato oficial funciona pero Copilot corre TODAS las fuentes: con `.claude/settings.json` presente los hooks correrian dos veces. Cambiar `plain` tocaria a Grok, Gemini y Kimi sin haberlos medido con JSON; un modo nuevo solo para el archivo que Claude y Copilot comparten es el cambio minimo. La frase generica sola le saca al modelo el detalle que hoy ve por stderr con Claude (que repo esta sucio, que gate fallo). `trustedFolders` es config del usuario y de su maquina: el arnes la lee y dice como confiarla, no la escribe. Medido con Copilot CLI 1.0.83 logueado (2026-09-10) | 2026-09-10 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:104 (spec `anidado`), docs/architecture.md:105 (spec `carpeta`), docs/architecture.md:107 (módulo `contra`) y 672 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `README.md`, `UPDATING.md`, `docs/architecture.md` y 20 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:239-242

