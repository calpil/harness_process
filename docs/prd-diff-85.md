Aplicado: 2026-09-10T01:55:31Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #85: Copilot CLI como backend de primera clase: superficie .github/copilot-instructions.md, hooks en .github/copilot.json, agentes en .github/agents, fila en doctor y backend LLM en la tabla de CLIs

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 85`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:1 (spec `proyecto`) y 264 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `README.md`, `UPDATING.md`, `docs/architecture.md` y 17 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 26 | Partir una leccion sobre el tope es un comando | leccion_partir | <O1> | El tope de la #80 se sostenia con un contrato que solo veia titulos `(feature #N)` y un procedimiento a mano. Medido el 2026-09-08 en realestate: cuatro lecciones sobre el tope (2361, 398, 382 y 275 lineas) con titulos `(#115, fecha)`, `feature #100 (fecha)`, `(fecha, front #131)` y `Patch #100:` que el contrato no nombraba, y un Stop con cuatro parrafos `[i]`. Ahora `leccion partir <clase>` informa que secciones cuentan UNA feature o sesion (titulo con `#N` o fecha, fuera de cuando aplica / procedimiento / pitfalls / verificacion / referencias), cuantas lineas quedarian y cuanto falta; `--aplicar` respalda (`lecciones rollback` lo deshace), mueve cada seccion tal cual a `docs/lecciones/<clase>/referencias/<slug>.md` con su cabecera y deja un puntero de una linea en el indice `## Referencias` (lo crea si falta); `--seccion "<titulo>"` suma una elegida, nunca una canonica. Si sigue sobre el tope, lo movido queda y sale 2 con lo que falta y las secciones mas grandes. El contrato de `close`/`usar` y `lecciones status` nombran el comando; `harness_check.sh` avisa con UNA linea por todas las lecciones sobre el tope. Medido al cerrar sobre una copia de realestate: 2361 -> 1994, 398 -> 359, 382 -> 266, 275 sin candidatas; ninguna baja del tope con lo mecanico, y el comando lo dice con el numero. Disparador: el Stop de realestate del 2026-09-08 y la decision del usuario de hacerlo comando en vez de partirlas a mano | done (2026-09-08) |
Despues:
| 26 | Partir una leccion sobre el tope es un comando | leccion_partir | <O1> | El tope de la #80 se sostenia con un contrato que solo veia titulos `(feature #N)` y un procedimiento a mano. Medido el 2026-09-08 en realestate: cuatro lecciones sobre el tope (2361, 398, 382 y 275 lineas) con titulos `(#115, fecha)`, `feature #100 (fecha)`, `(fecha, front #131)` y `Patch #100:` que el contrato no nombraba, y un Stop con cuatro parrafos `[i]`. Ahora `leccion partir <clase>` informa que secciones cuentan UNA feature o sesion (titulo con `#N` o fecha, fuera de cuando aplica / procedimiento / pitfalls / verificacion / referencias), cuantas lineas quedarian y cuanto falta; `--aplicar` respalda (`lecciones rollback` lo deshace), mueve cada seccion tal cual a `docs/lecciones/<clase>/referencias/<slug>.md` con su cabecera y deja un puntero de una linea en el indice `## Referencias` (lo crea si falta); `--seccion "<titulo>"` suma una elegida, nunca una canonica. Si sigue sobre el tope, lo movido queda y sale 2 con lo que falta y las secciones mas grandes. El contrato de `close`/`usar` y `lecciones status` nombran el comando; `harness_check.sh` avisa con UNA linea por todas las lecciones sobre el tope. Medido al cerrar sobre una copia de realestate: 2361 -> 1994, 398 -> 359, 382 -> 266, 275 sin candidatas; ninguna baja del tope con lo mecanico, y el comando lo dice con el numero. Disparador: el Stop de realestate del 2026-09-08 y la decision del usuario de hacerlo comando en vez de partirlas a mano | done (2026-09-08) |
| 27 | Copilot CLI es un backend del arnes | copilot_backend | <O1> | Copilot no aparecia en ningun punto del arnes (pregunta del usuario del 2026-09-09). Ahora esta en los cuatro lugares donde viven los backends: el instalador (sh y ps1), solo con `copilot` en el PATH o `--copilot`/`-Copilot` (`--no-copilot` lo omite), llama a `harness copilot instalar`, que MEZCLA los hooks `sessionStart`/`agentStop`/`sessionEnd` sobre el `.github/copilot.json` del usuario sin pisar claves ni hooks ajenos y deja un bloque entre marcadores en `.github/copilot-instructions.md` que apunta a `AGENTS.md` (Copilot lo lee solo); `--reset` corre `copilot quitar` y devuelve lo ajeno. El runtime de hooks (`bin/harness-hook` y `harness-hook.ps1`) gana el modo `copilot-json`: `agentStop` es el Stop y responde `{"block":true,"reason":...}` o `{"block":false}` leyendo `stopHookActive` como corte. `doctor` lista `copilot` por su huella y `consolidar` lo detecta como `copilot -s -p` despues de `claude` y `kimi`, con la autenticacion en el skip. Medido contra `copilot --help` de la 1.0.83 y la documentacion; la semantica exacta de `agentStop` queda por medir con sesion (AC-8 manual). Fuera: agentes en `.github/agents/` hasta verificar el formato. Disparador: el usuario pregunto si el arnes estaba configurado para Copilot y no lo estaba | done (2026-09-09) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `ningun`), docs/prd/SDD-master.md:101 (spec `cuando`) y 341 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `README.md`, `UPDATING.md`, `docs/architecture.md` y 17 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D13 | `leccion partir` mueve solo lo mecanico —las secciones cuyo titulo trae `#N` o una fecha, nunca las canonicas— informa antes de escribir, deja lo movido y sale 2 si sigue sobre el tope, y `--seccion` se niega sobre una canonica | (a) solo titulos `(feature #N)` como en la #80; (b) todo o nada: no escribir si no llega al tope; (c) permitir mover canonicas con `--seccion`; (d) partir las lecciones de realestate a mano desde el arnes | Solo `(feature #N)` no ve ninguna de las once secciones reales de realestate. Todo o nada esconde el progreso: mover ocho secciones de un gigante de 2361 lineas es trabajo hecho y reversible con rollback; el exit 2 dice que el objetivo (bajar del tope) no se cumplio. Las canonicas son la clase: si lo que sobra esta ahi, hay que reescribir, y eso no es un comando. Partirlas a mano desde aca cruza la linea de no tocar otros proyectos: se entrega el comando. Decisiones del usuario del 2026-09-08 (OBS-1..3 de la #84) | 2026-09-08 |
Despues:
| D13 | `leccion partir` mueve solo lo mecanico —las secciones cuyo titulo trae `#N` o una fecha, nunca las canonicas— informa antes de escribir, deja lo movido y sale 2 si sigue sobre el tope, y `--seccion` se niega sobre una canonica | (a) solo titulos `(feature #N)` como en la #80; (b) todo o nada: no escribir si no llega al tope; (c) permitir mover canonicas con `--seccion`; (d) partir las lecciones de realestate a mano desde el arnes | Solo `(feature #N)` no ve ninguna de las once secciones reales de realestate. Todo o nada esconde el progreso: mover ocho secciones de un gigante de 2361 lineas es trabajo hecho y reversible con rollback; el exit 2 dice que el objetivo (bajar del tope) no se cumplio. Las canonicas son la clase: si lo que sobra esta ahi, hay que reescribir, y eso no es un comando. Partirlas a mano desde aca cruza la linea de no tocar otros proyectos: se entrega el comando. Decisiones del usuario del 2026-09-08 (OBS-1..3 de la #84) | 2026-09-08 |
| D14 | Copilot entra como backend con lo que se pudo medir: los archivos de `.github/` se mezclan (nunca se pisan), se generan solo con `copilot` detectado o `--copilot`, el formato vive en el binario (`harness copilot instalar\|quitar`), `agentStop` es el Stop bloqueante y los agentes de `.github/agents/` quedan fuera hasta verificar el formato | (a) copiar `AGENTS.md` entero en `copilot-instructions.md`; (b) generar siempre, como Codex y Gemini; (c) agentes a ciegas en el formato documentado para VS Code; (d) solo informar en `sessionEnd` sin bloquear nunca | Copilot carga `AGENTS.md` y `copilot-instructions.md`: copiar uno en el otro duplica el prompt en cada sesion. `.github/` es del usuario y muchos repos ya tienen esos archivos: meterlos en proyectos sin Copilot es ruido, y pisarlos es perder trabajo ajeno (por eso mezcla, marcadores y respaldo). Un formato de agentes que no se pudo verificar es un archivo muerto (leccion remedios-que-la-herramienta-sugiere). `agentStop` trae `stopHookActive` con la semantica del Stop de Claude y hay un `toolCall` aparte: la lectura mas probable es que sea el Stop; sin login no se midio, y el AC-8 decide. Decisiones del usuario del 2026-09-09 (OBS-1..4 de la #85) | 2026-09-09 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:105 (spec `registro`), docs/architecture.md:108 (spec `completa`), docs/architecture.md:108 (spec `master`) y 586 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `README.md`, `UPDATING.md`, `docs/architecture.md` y 17 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:161-170

