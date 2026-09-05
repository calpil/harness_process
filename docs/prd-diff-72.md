Aplicado: 2026-09-05T03:47:49Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #72: El paralelo aisla los cambios y acota los workflows

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 72`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:110 (spec `master`) y 188 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `AGENTS.md`, `UPDATING.md`, `commit_guard.sh` y 18 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 8 | Features en paralelo sin pisarse | features_en_paralelo_con_worktrees | <O1> | `start` deja de rechazar la segunda feature activa y le da a cada una su rama GitFlow (`feature/<id>-<slug>`, `bugfix/` si es `kind: bug`) y su worktree hermano; el estado del arnes sigue siendo unico (repo principal) y el vivo se parte en `current-<id>.md` con `current.md` como indice; dentro del worktree los comandos infieren la feature; `close --status done` exige `--to <rama>`, mergea, publica, borra el worktree y conserva la rama, y un conflicto aborta sin dejar nada a medias | done (2026-08-22) |
| 9 | Revisar en serio sin que cueste una fortuna | revision_adversarial_y_modelos_por_rol | <O1> | Un modelo por rol de Claude (implementer `claude-opus-5`, lider y reviewer `claude-fable-5`, los tres `xhigh`) definido en la tabla de roles de los dos instaladores y tuneable por variable; el reviewer intenta REFUTAR cada AC y verifica por su cuenta lo que la evidencia declara verde; y `revision --feature <id>` arma el paquete minimo (AC + estado de verify + evidencia + archivos + diff + rutas protegidas) acotado por presupuesto, que declara lo que recorta y reporta su propio tamaño | done (2026-08-22) |
| 10 | El MCP de Atlassian ya conectado en cada backend | mcp_atlassian_en_los_cuatro_backends | <O1> | Instalar el arnes en un repo con binding de Atlassian deja tambien el MCP por PROYECTO en los backends que lo admiten (`.mcp.json` de Claude, `.kimi-code/mcp.json` de Kimi y `.grok/config.toml` de Grok via `mcp-remote`, porque su cliente HTTP no completa el OAuth), y para Codex —que no admite alcance de proyecto— imprime los dos comandos (servidor + plugin `atlassian-rovo`, imprescindible) en vez de tocar su configuracion global; respeta lo que ya haya, no escribe credenciales y dice por CLI como autorizar | done (2026-08-22) |
| 11 | Empezar con el material en la mano, no explorando | paquete_de_contexto_para_implementar | <O1> | `contexto --feature <id>` (o `--tema`) entrega el mapa —siguiendo el puntero si `architecture.md` apunta a otro archivo—, si ese mapa CUBRE el tema, el impacto del hub con limite, la edad del grafo (vencido a los 7 dias), la historia acotada, las lecciones que aplican y las features del mismo servicio; declara su tamaño y sus huecos, y el resumen sale solo en cada `start`. Disparador: un mapeo de 4 agentes y 693.6k tokens sobre un tema que el mapa no mencionaba | done (2026-08-22) |
| 12 | El arnes no se bloquea a si mismo | el_guard_no_bloquea_por_lo_que_escribe_el_arnes | <O1> | El commit guard deja de contar como sucios los documentos que escribio el propio arnes (specs, planes, impl, review, verify, estados, prd-diff, `docs/prd/**`, `docs/lecciones/**`, architecture y perfil), exigiendo nombre Y ubicacion bajo `docs/`; sigue bloqueando por codigo y por cualquier documento ajeno, y dice en una linea `[i]` cada vez que se saltea un repo. Disparador: en un proyecto donde `docs/` es su propio repo, cada start/advance/prd apply terminaba el turno pidiendo un commit por microservicio de archivos que el `close` iba a commitear | done (2026-08-22) |
| 13 | Verificar lo que de verdad prueba, aunque hable mucho | verify_no_se_cuelga_con_salida_grande | <O1> | `verify` lee los pipes con un hilo por descriptor MIENTRAS el comando corre, en vez de leerlos despues de esperarlo: un comando que imprime mas que el buffer del pipe (~64 KB) ya no cuelga el gate. Retiene la cola con tope de 4 MB declarando el recorte, sigue midiendo el estado sobre la salida completa (leccion #44), sigue cortando por timeout y no se deja pisar por un nieto que hereda el pipe. Disparador: el smoke del instalador dejo a verify once minutos colgado y quedo sin poder declararse como AC | done (2026-08-22) |
Despues:
| 8 | Features en paralelo sin pisarse | features_en_paralelo_con_worktrees | <O1> | `start` deja de rechazar la segunda feature activa y le da a cada una su rama GitFlow (`feature/<id>-<slug>`, `bugfix/` si es `kind: bug`) y su worktree hermano; el estado del arnes sigue siendo unico (repo principal) y el vivo se parte en `current-<id>.md` con `current.md` como indice; dentro del worktree los comandos infieren la feature; `close --status done` exige `--to <rama>`, mergea, borra el worktree y conserva la rama (desde la #72 la publicacion es explicita con --publicar), y un conflicto aborta sin dejar nada a medias | done (2026-08-22) |
| 9 | Revisar en serio sin que cueste una fortuna | revision_adversarial_y_modelos_por_rol | <O1> | Un modelo por rol de Claude (implementer `claude-opus-5`, lider y reviewer `claude-fable-5`, los tres `xhigh`) definido en la tabla de roles de los dos instaladores y tuneable por variable; el reviewer intenta REFUTAR cada AC y verifica por su cuenta lo que la evidencia declara verde; y `revision --feature <id>` arma el paquete minimo (AC + estado de verify + evidencia + archivos + diff + rutas protegidas) acotado por presupuesto, que declara lo que recorta y reporta su propio tamaño | done (2026-08-22) |
| 10 | El MCP de Atlassian ya conectado en cada backend | mcp_atlassian_en_los_cuatro_backends | <O1> | Instalar el arnes en un repo con binding de Atlassian deja tambien el MCP por PROYECTO en los backends que lo admiten (`.mcp.json` de Claude, `.kimi-code/mcp.json` de Kimi y `.grok/config.toml` de Grok via `mcp-remote`, porque su cliente HTTP no completa el OAuth), y para Codex —que no admite alcance de proyecto— imprime los dos comandos (servidor + plugin `atlassian-rovo`, imprescindible) en vez de tocar su configuracion global; respeta lo que ya haya, no escribe credenciales y dice por CLI como autorizar | done (2026-08-22) |
| 11 | Empezar con el material en la mano, no explorando | paquete_de_contexto_para_implementar | <O1> | `contexto --feature <id>` (o `--tema`) entrega el mapa —siguiendo el puntero si `architecture.md` apunta a otro archivo—, si ese mapa CUBRE el tema, el impacto del hub con limite, la edad del grafo (vencido a los 7 dias), la historia acotada, las lecciones que aplican y las features del mismo servicio; declara su tamaño y sus huecos, y el resumen sale solo en cada `start`. Disparador: un mapeo de 4 agentes y 693.6k tokens sobre un tema que el mapa no mencionaba | done (2026-08-22) |
| 12 | El arnes no se bloquea a si mismo | el_guard_no_bloquea_por_lo_que_escribe_el_arnes | <O1> | El commit guard deja de contar como sucios los documentos que escribio el propio arnes (specs, planes, impl, review, verify, estados, prd-diff, `docs/prd/**`, `docs/lecciones/**`, architecture y perfil), exigiendo nombre Y ubicacion bajo `docs/`; sigue bloqueando por codigo y por cualquier documento ajeno, y dice en una linea `[i]` cada vez que se saltea un repo. Disparador: en un proyecto donde `docs/` es su propio repo, cada start/advance/prd apply terminaba el turno pidiendo un commit por microservicio de archivos que el `close` iba a commitear | done (2026-08-22) |
| 13 | Verificar lo que de verdad prueba, aunque hable mucho | verify_no_se_cuelga_con_salida_grande | <O1> | `verify` lee los pipes con un hilo por descriptor MIENTRAS el comando corre, en vez de leerlos despues de esperarlo: un comando que imprime mas que el buffer del pipe (~64 KB) ya no cuelga el gate. Retiene la cola con tope de 4 MB declarando el recorte, sigue midiendo el estado sobre la salida completa (leccion #44), sigue cortando por timeout y no se deja pisar por un nieto que hereda el pipe. Disparador: el smoke del instalador dejo a verify once minutos colgado y quedo sin poder declararse como AC | done (2026-08-22) |
| 14 | El paralelo aisla los cambios y acota los workflows | el_paralelo_aisla_los_cambios | <O1> | `start` resuelve el aislamiento ANTES de marcar `in_progress`: un fallo de git o un `--sin-worktree` con otra feature abierta RECHAZAN el arranque y dejan el backlog intacto, y sin repo git corre una feature a la vez; un `docs/` que es otro repo recibe su propio worktree; el cierre muestra origen, destino y TODO el rango de commits, se niega si arrastra trabajo de otra feature, serializa por destino y ya no publica sin `--publicar`; el Stop revisa el worktree de la sesion en vez de reclamar los repos compartidos; y una tarea delegada fallida se registra y bloquea `approved` hasta cubrirse. Disparador: tres features activas sin rama ni worktree escribiendo en el mismo checkout, y un commit que se habia acordado dejar local publicado por ser el padre de otro | done (2026-09-05) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:103 (spec `bloquea`) y 448 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `AGENTS.md`, `UPDATING.md`, `commit_guard.sh` y 18 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D1 | <lo que se decidio> | <opcion B, opcion C> | <razon> | <YYYY-MM-DD> |
Despues:
| D1 | El aislamiento de una feature se DECIDE en una funcion pura (`aislamiento::decidir`) y se ejecuta despues; un arranque que no lo consigue no arranca | (a) avisar con `[i]` y seguir, como estaba; (b) una regla `require_aislamiento` apagada por defecto | Avisar y seguir dejo tres features `in_progress` sin rama ni worktree escribiendo en el mismo checkout. Una regla opcional habria repetido el problema en toda instalacion que no la active. Separar decidir de ejecutar es lo que impide volver al fallback: la parte que decide no tiene con que continuar | 2026-09-05 |
| D2 | La publicacion del cierre pasa a ser explicita (`close --publicar`); sin el flag el merge queda local y se imprime el comando | (a) seguir publicando siempre; (b) preguntar interactivamente | Un `push` automatico despues del merge publico un commit que se habia acordado dejar local, porque era el padre del que si iba. Preguntar no sirve: el cierre corre en hooks y en sesiones sin nadie mirando | 2026-09-05 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:100 (spec `ambigua`), docs/architecture.md:100 (spec `ambiguo`) y 763 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `AGENTS.md`, `UPDATING.md`, `commit_guard.sh` y 18 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:48-58

