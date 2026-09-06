Aplicado: 2026-09-06T02:33:18Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #76: una feature sin worktree veta a todas las demas y mata el paralelismo

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 76`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:110 (spec `master`) y 178 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md` y 5 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 14 | El paralelo aisla los cambios y acota los workflows | el_paralelo_aisla_los_cambios | <O1> | `start` resuelve el aislamiento ANTES de marcar `in_progress`: un fallo de git o un `--sin-worktree` con otra feature abierta RECHAZAN el arranque y dejan el backlog intacto, y sin repo git corre una feature a la vez; un `docs/` que es otro repo recibe su propio worktree; el cierre muestra origen, destino y TODO el rango de commits, se niega si arrastra trabajo de otra feature, serializa por destino y ya no publica sin `--publicar`; el Stop revisa el worktree de la sesion en vez de reclamar los repos compartidos; y una tarea delegada fallida se registra y bloquea `approved` hasta cubrirse. Disparador: tres features activas sin rama ni worktree escribiendo en el mismo checkout, y un commit que se habia acordado dejar local publicado por ser el padre de otro | done (2026-09-05) |
| 15 | El sello de cierre deja de perderse | el_close_no_pierde_el_sello_de_cierre | <O1> | El `close` escribe `docs/estado-feature-<id>-<slug>.md` —que lleva adentro el cuerpo de `progress/current-<id>.md` y es su UNICA copia, porque `progress/` esta gitignorado— en el `docs/` del repo PRINCIPAL y despues de integrar, no en el `docs/` de la feature y antes. Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra: con un `docs/` que es repo aparte eso lo perdia. El mensaje nombra la ruta real y avisa que queda sin commitear; el caso especial que elegia entre dos rutas (#63) se elimina porque ya no hay dos. Disparador: el cierre de la #124 de realestate, cuyo sello hubo que reconstruir a mano y cuyo cuerpo literal es irrecuperable | done (2026-09-05) |
| 16 | Un AC verifica con TODOS los comandos que declara | verify_corre_todos_los_comandos_del_ac | <O1> | `verify` ejecuta cada linea `Comando:` que el AC declara, en orden, y deja UNA FILA POR COMANDO en `docs/verify-<id>.md` con su estado, su exit y su duracion; el AC queda rojo si cualquiera falla y el gate lo nombra una sola vez. Antes el modelo era `comando: Option<String>` y el segundo `Comando:` se descartaba sin marca. Disparador: el AC-8 de la #72 declaro cuatro verificaciones, se corrio una y el reporte dijo "1 verde, 0 en rojo" — con DOS tests en verde al lado, uno que afirmaba el descarte como intencion y otro cuyo oraculo contaba "solo el primero, como en `parsear`" | done (2026-09-05) |
| 17 | El backlog sabe que feature espera a cual | dependencias_y_circuit_breaker | <O1> | Una feature declara `depends_on` (al crearla o despues, con `harness depende`); `next` no la ofrece hasta que esas cierren y DICE quien espera a que; `start` avisa sin bloquear; los ids inexistentes, la auto-referencia y los ciclos se rechazan sin escribir nada, nombrando el camino del ciclo. `superseded` y `resuelto-aguas-arriba` satisfacen una dependencia; `blocked` y `pending` no. Ademas, a partir del N-esimo cierre `blocked` (`rules.bloqueos_antes_de_decidir`, default 2) el cierre exige decir si la causa es la misma. Medido antes de implementar: 84 cierres reales, CERO features bloqueadas dos veces — el circuit breaker se implemento por decision del usuario con esa medicion a la vista, y sus dos tests son toda su evidencia | done (2026-09-05) |
Despues:
| 14 | El paralelo aisla los cambios y acota los workflows | el_paralelo_aisla_los_cambios | <O1> | `start` resuelve el aislamiento ANTES de marcar `in_progress`: un fallo de git, o dos features que escribirian en el checkout compartido, RECHAZAN el arranque y dejan el backlog intacto; una feature con su worktree arranca siempre (corregido en la #76: la regla original vetaba a todas), y sin repo git corre una feature a la vez; un `docs/` que es otro repo recibe su propio worktree; el cierre muestra origen, destino y TODO el rango de commits, se niega si arrastra trabajo de otra feature, serializa por destino y ya no publica sin `--publicar`; el Stop revisa el worktree de la sesion en vez de reclamar los repos compartidos; y una tarea delegada fallida se registra y bloquea `approved` hasta cubrirse. Disparador: tres features activas sin rama ni worktree escribiendo en el mismo checkout, y un commit que se habia acordado dejar local publicado por ser el padre de otro | done (2026-09-05) |
| 15 | El sello de cierre deja de perderse | el_close_no_pierde_el_sello_de_cierre | <O1> | El `close` escribe `docs/estado-feature-<id>-<slug>.md` —que lleva adentro el cuerpo de `progress/current-<id>.md` y es su UNICA copia, porque `progress/` esta gitignorado— en el `docs/` del repo PRINCIPAL y despues de integrar, no en el `docs/` de la feature y antes. Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra: con un `docs/` que es repo aparte eso lo perdia. El mensaje nombra la ruta real y avisa que queda sin commitear; el caso especial que elegia entre dos rutas (#63) se elimina porque ya no hay dos. Disparador: el cierre de la #124 de realestate, cuyo sello hubo que reconstruir a mano y cuyo cuerpo literal es irrecuperable | done (2026-09-05) |
| 16 | Un AC verifica con TODOS los comandos que declara | verify_corre_todos_los_comandos_del_ac | <O1> | `verify` ejecuta cada linea `Comando:` que el AC declara, en orden, y deja UNA FILA POR COMANDO en `docs/verify-<id>.md` con su estado, su exit y su duracion; el AC queda rojo si cualquiera falla y el gate lo nombra una sola vez. Antes el modelo era `comando: Option<String>` y el segundo `Comando:` se descartaba sin marca. Disparador: el AC-8 de la #72 declaro cuatro verificaciones, se corrio una y el reporte dijo "1 verde, 0 en rojo" — con DOS tests en verde al lado, uno que afirmaba el descarte como intencion y otro cuyo oraculo contaba "solo el primero, como en `parsear`" | done (2026-09-05) |
| 17 | El backlog sabe que feature espera a cual | dependencias_y_circuit_breaker | <O1> | Una feature declara `depends_on` (al crearla o despues, con `harness depende`); `next` no la ofrece hasta que esas cierren y DICE quien espera a que; `start` avisa sin bloquear; los ids inexistentes, la auto-referencia y los ciclos se rechazan sin escribir nada, nombrando el camino del ciclo. `superseded` y `resuelto-aguas-arriba` satisfacen una dependencia; `blocked` y `pending` no. Ademas, a partir del N-esimo cierre `blocked` (`rules.bloqueos_antes_de_decidir`, default 2) el cierre exige decir si la causa es la misma. Medido antes de implementar: 84 cierres reales, CERO features bloqueadas dos veces — el circuit breaker se implemento por decision del usuario con esa medicion a la vista, y sus dos tests son toda su evidencia | done (2026-09-05) |
| 18 | Una feature sin worktree ya no veta a las que traen el suyo | el_checkout_compartido_tiene_capacidad_uno | <O1> | Regresion de la #72, mismo dia: una feature abierta sin worktree rechazaba a TODAS las demas, y el usuario termino esperando a que "la #99 libere" para arrancar una feature con su propio arbol. El modelo pasa a ser "el checkout compartido tiene capacidad UNO": se rechaza solo que dos features escriban ahi; una con worktree arranca siempre y se le informa con quien convive. Sin git sigue siendo una a la vez. Disparador: el reporte del usuario con captura; la regla original generalizaba de mas sobre el incidente real (cuatro sin worktree en el MISMO arbol) | done (2026-09-06) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:10 (spec `ninguna`) y 288 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md` y 5 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D1 | El aislamiento de una feature se DECIDE en una funcion pura (`aislamiento::decidir`) y se ejecuta despues; un arranque que no lo consigue no arranca | (a) avisar con `[i]` y seguir, como estaba; (b) una regla `require_aislamiento` apagada por defecto | Avisar y seguir dejo tres features `in_progress` sin rama ni worktree escribiendo en el mismo checkout. Una regla opcional habria repetido el problema en toda instalacion que no la active. Separar decidir de ejecutar es lo que impide volver al fallback: la parte que decide no tiene con que continuar | 2026-09-05 |
Despues:
| D1 | El aislamiento de una feature se DECIDE en una funcion pura (`aislamiento::decidir`) y se ejecuta despues; un arranque que no lo consigue no arranca | (a) avisar con `[i]` y seguir, como estaba; (b) una regla `require_aislamiento` apagada por defecto | Avisar y seguir dejo tres features `in_progress` sin rama ni worktree escribiendo en el mismo checkout. Una regla opcional habria repetido el problema en toda instalacion que no la active. Separar decidir de ejecutar es lo que impide volver al fallback: la parte que decide no tiene con que continuar | 2026-09-05 |
| D6 | El checkout compartido es un recurso de CAPACIDAD UNO: `aislamiento::decidir` rechaza solo que dos features escriban ahi, y una feature con worktree arranca siempre | (a) mantener la regla de la #72, "una sin aislar veta a todas" | Esa regla era mas ancha que la evidencia: el incidente fueron cuatro features sin worktree en el MISMO arbol, y la regla vetaba tambien a las que tenian su propio arbol y no compartian nada. Costaba paralelismo sin comprar seguridad, y el usuario lo detecto usando el arnes, no la suite: el gate era coherente con sus tests y estaba mal igual | 2026-09-06 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:100 (spec `aplica`), docs/architecture.md:101 (módulo `cierre`) y 476 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `AGENTS.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md` y 5 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:68-80

