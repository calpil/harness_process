Aplicado: 2026-09-05T04:20:17Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #71: El close archiva el sello de cierre en el worktree que acaba de borrar, y lo pierde

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 71`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:11 (spec `instala`) y 138 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/promesas-estructurales-vs-disciplina.md`, `rust/src/commands/close.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 14 | El paralelo aisla los cambios y acota los workflows | el_paralelo_aisla_los_cambios | <O1> | `start` resuelve el aislamiento ANTES de marcar `in_progress`: un fallo de git o un `--sin-worktree` con otra feature abierta RECHAZAN el arranque y dejan el backlog intacto, y sin repo git corre una feature a la vez; un `docs/` que es otro repo recibe su propio worktree; el cierre muestra origen, destino y TODO el rango de commits, se niega si arrastra trabajo de otra feature, serializa por destino y ya no publica sin `--publicar`; el Stop revisa el worktree de la sesion en vez de reclamar los repos compartidos; y una tarea delegada fallida se registra y bloquea `approved` hasta cubrirse. Disparador: tres features activas sin rama ni worktree escribiendo en el mismo checkout, y un commit que se habia acordado dejar local publicado por ser el padre de otro | done (2026-09-05) |
Despues:
| 14 | El paralelo aisla los cambios y acota los workflows | el_paralelo_aisla_los_cambios | <O1> | `start` resuelve el aislamiento ANTES de marcar `in_progress`: un fallo de git o un `--sin-worktree` con otra feature abierta RECHAZAN el arranque y dejan el backlog intacto, y sin repo git corre una feature a la vez; un `docs/` que es otro repo recibe su propio worktree; el cierre muestra origen, destino y TODO el rango de commits, se niega si arrastra trabajo de otra feature, serializa por destino y ya no publica sin `--publicar`; el Stop revisa el worktree de la sesion en vez de reclamar los repos compartidos; y una tarea delegada fallida se registra y bloquea `approved` hasta cubrirse. Disparador: tres features activas sin rama ni worktree escribiendo en el mismo checkout, y un commit que se habia acordado dejar local publicado por ser el padre de otro | done (2026-09-05) |
| 15 | El sello de cierre deja de perderse | el_close_no_pierde_el_sello_de_cierre | <O1> | El `close` escribe `docs/estado-feature-<id>-<slug>.md` —que lleva adentro el cuerpo de `progress/current-<id>.md` y es su UNICA copia, porque `progress/` esta gitignorado— en el `docs/` del repo PRINCIPAL y despues de integrar, no en el `docs/` de la feature y antes. Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra: con un `docs/` que es repo aparte eso lo perdia. El mensaje nombra la ruta real y avisa que queda sin commitear; el caso especial que elegia entre dos rutas (#63) se elimina porque ya no hay dos. Disparador: el cierre de la #124 de realestate, cuyo sello hubo que reconstruir a mano y cuyo cuerpo literal es irrecuperable | done (2026-09-05) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:10 (spec `ningun`) y 325 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/promesas-estructurales-vs-disciplina.md`, `rust/src/commands/close.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D2 | La publicacion del cierre pasa a ser explicita (`close --publicar`); sin el flag el merge queda local y se imprime el comando | (a) seguir publicando siempre; (b) preguntar interactivamente | Un `push` automatico despues del merge publico un commit que se habia acordado dejar local, porque era el padre del que si iba. Preguntar no sirve: el cierre corre en hooks y en sesiones sin nadie mirando | 2026-09-05 |
Despues:
| D2 | La publicacion del cierre pasa a ser explicita (`close --publicar`); sin el flag el merge queda local y se imprime el comando | (a) seguir publicando siempre; (b) preguntar interactivamente | Un `push` automatico despues del merge publico un commit que se habia acordado dejar local, porque era el padre del que si iba. Preguntar no sirve: el cierre corre en hooks y en sesiones sin nadie mirando | 2026-09-05 |
| D3 | El sello de cierre se escribe en el `docs/` de la RAIZ y en la fase del estado, no en el `docs/` de la feature ni en la fase de los artefactos que viajan en la rama | (a) escribirlo en los dos lados; (b) dejarlo donde estaba y arreglar solo el mensaje | Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra, y ahi vive la unica copia del estado vivo. Dos copias divergen —la familia de bug mas repetida de este repo— y arreglar solo el mensaje lo dejaba en una rama que nadie mergea. Movido el lugar, la fase pudo bajar: estaba en la fase 1 por una razon fisica que dejo de existir | 2026-09-05 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:100 (spec `ningun`), docs/architecture.md:101 (spec `propio`) y 511 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/promesas-estructurales-vs-disciplina.md`, `rust/src/commands/close.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:203-210

