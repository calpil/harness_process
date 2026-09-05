Aplicado: 2026-09-05T12:18:28Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #73: verify corre UN comando por AC y no lo dice: un AC con varias verificaciones se cree verde con una

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 73`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:103 (spec `guarda`), docs/prd/PRD-master.md:11 (spec `instalador`) y 170 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/commands/verify.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 15 | El sello de cierre deja de perderse | el_close_no_pierde_el_sello_de_cierre | <O1> | El `close` escribe `docs/estado-feature-<id>-<slug>.md` —que lleva adentro el cuerpo de `progress/current-<id>.md` y es su UNICA copia, porque `progress/` esta gitignorado— en el `docs/` del repo PRINCIPAL y despues de integrar, no en el `docs/` de la feature y antes. Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra: con un `docs/` que es repo aparte eso lo perdia. El mensaje nombra la ruta real y avisa que queda sin commitear; el caso especial que elegia entre dos rutas (#63) se elimina porque ya no hay dos. Disparador: el cierre de la #124 de realestate, cuyo sello hubo que reconstruir a mano y cuyo cuerpo literal es irrecuperable | done (2026-09-05) |
Despues:
| 15 | El sello de cierre deja de perderse | el_close_no_pierde_el_sello_de_cierre | <O1> | El `close` escribe `docs/estado-feature-<id>-<slug>.md` —que lleva adentro el cuerpo de `progress/current-<id>.md` y es su UNICA copia, porque `progress/` esta gitignorado— en el `docs/` del repo PRINCIPAL y despues de integrar, no en el `docs/` de la feature y antes. Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra: con un `docs/` que es repo aparte eso lo perdia. El mensaje nombra la ruta real y avisa que queda sin commitear; el caso especial que elegia entre dos rutas (#63) se elimina porque ya no hay dos. Disparador: el cierre de la #124 de realestate, cuyo sello hubo que reconstruir a mano y cuyo cuerpo literal es irrecuperable | done (2026-09-05) |
| 16 | Un AC verifica con TODOS los comandos que declara | verify_corre_todos_los_comandos_del_ac | <O1> | `verify` ejecuta cada linea `Comando:` que el AC declara, en orden, y deja UNA FILA POR COMANDO en `docs/verify-<id>.md` con su estado, su exit y su duracion; el AC queda rojo si cualquiera falla y el gate lo nombra una sola vez. Antes el modelo era `comando: Option<String>` y el segundo `Comando:` se descartaba sin marca. Disparador: el AC-8 de la #72 declaro cuatro verificaciones, se corrio una y el reporte dijo "1 verde, 0 en rojo" — con DOS tests en verde al lado, uno que afirmaba el descarte como intencion y otro cuyo oraculo contaba "solo el primero, como en `parsear`" | done (2026-09-05) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:10 (spec `ningun`) y 331 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/commands/verify.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D3 | El sello de cierre se escribe en el `docs/` de la RAIZ y en la fase del estado, no en el `docs/` de la feature ni en la fase de los artefactos que viajan en la rama | (a) escribirlo en los dos lados; (b) dejarlo donde estaba y arreglar solo el mensaje | Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra, y ahi vive la unica copia del estado vivo. Dos copias divergen —la familia de bug mas repetida de este repo— y arreglar solo el mensaje lo dejaba en una rama que nadie mergea. Movido el lugar, la fase pudo bajar: estaba en la fase 1 por una razon fisica que dejo de existir | 2026-09-05 |
Despues:
| D3 | El sello de cierre se escribe en el `docs/` de la RAIZ y en la fase del estado, no en el `docs/` de la feature ni en la fase de los artefactos que viajan en la rama | (a) escribirlo en los dos lados; (b) dejarlo donde estaba y arreglar solo el mensaje | Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra, y ahi vive la unica copia del estado vivo. Dos copias divergen —la familia de bug mas repetida de este repo— y arreglar solo el mensaje lo dejaba en una rama que nadie mergea. Movido el lugar, la fase pudo bajar: estaba en la fase 1 por una razon fisica que dejo de existir | 2026-09-05 |
| D4 | Un AC lleva una LISTA de comandos (`comandos: Vec<String>`) y `verify` emite un resultado por comando | (a) conservar `Option<String>` y solo avisar que hay mas de uno | El modelo de un comando por AC no era una decision: era el motivo del bug. Avisar deja al autor partiendo el AC a mano, que es disciplina y no estructura. La deduplicacion de "AC en rojo" vive en UNA funcion (`sin_repetir`) que usan el mensaje de `verify` y el gate del cierre, para no repetir la divergencia de las features #64, #67 y #69 | 2026-09-05 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:100 (spec `ningun`), docs/architecture.md:101 (spec `propio`) y 464 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/commands/verify.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:139-170

