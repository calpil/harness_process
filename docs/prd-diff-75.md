Aplicado: 2026-09-05T14:07:27Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #75: el backlog no sabe de dependencias ni de features que se traban una y otra vez

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 75`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:101 (spec `evento`), docs/prd/PRD-master.md:103 (spec `guarda`) y 216 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/cli.rs` y 11 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 16 | Un AC verifica con TODOS los comandos que declara | verify_corre_todos_los_comandos_del_ac | <O1> | `verify` ejecuta cada linea `Comando:` que el AC declara, en orden, y deja UNA FILA POR COMANDO en `docs/verify-<id>.md` con su estado, su exit y su duracion; el AC queda rojo si cualquiera falla y el gate lo nombra una sola vez. Antes el modelo era `comando: Option<String>` y el segundo `Comando:` se descartaba sin marca. Disparador: el AC-8 de la #72 declaro cuatro verificaciones, se corrio una y el reporte dijo "1 verde, 0 en rojo" — con DOS tests en verde al lado, uno que afirmaba el descarte como intencion y otro cuyo oraculo contaba "solo el primero, como en `parsear`" | done (2026-09-05) |
Despues:
| 16 | Un AC verifica con TODOS los comandos que declara | verify_corre_todos_los_comandos_del_ac | <O1> | `verify` ejecuta cada linea `Comando:` que el AC declara, en orden, y deja UNA FILA POR COMANDO en `docs/verify-<id>.md` con su estado, su exit y su duracion; el AC queda rojo si cualquiera falla y el gate lo nombra una sola vez. Antes el modelo era `comando: Option<String>` y el segundo `Comando:` se descartaba sin marca. Disparador: el AC-8 de la #72 declaro cuatro verificaciones, se corrio una y el reporte dijo "1 verde, 0 en rojo" — con DOS tests en verde al lado, uno que afirmaba el descarte como intencion y otro cuyo oraculo contaba "solo el primero, como en `parsear`" | done (2026-09-05) |
| 17 | El backlog sabe que feature espera a cual | dependencias_y_circuit_breaker | <O1> | Una feature declara `depends_on` (al crearla o despues, con `harness depende`); `next` no la ofrece hasta que esas cierren y DICE quien espera a que; `start` avisa sin bloquear; los ids inexistentes, la auto-referencia y los ciclos se rechazan sin escribir nada, nombrando el camino del ciclo. `superseded` y `resuelto-aguas-arriba` satisfacen una dependencia; `blocked` y `pending` no. Ademas, a partir del N-esimo cierre `blocked` (`rules.bloqueos_antes_de_decidir`, default 2) el cierre exige decir si la causa es la misma. Medido antes de implementar: 84 cierres reales, CERO features bloqueadas dos veces — el circuit breaker se implemento por decision del usuario con esa medicion a la vista, y sus dos tests son toda su evidencia | done (2026-09-05) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:10 (spec `ningun`) y 416 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/cli.rs` y 11 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D4 | Un AC lleva una LISTA de comandos (`comandos: Vec<String>`) y `verify` emite un resultado por comando | (a) conservar `Option<String>` y solo avisar que hay mas de uno | El modelo de un comando por AC no era una decision: era el motivo del bug. Avisar deja al autor partiendo el AC a mano, que es disciplina y no estructura. La deduplicacion de "AC en rojo" vive en UNA funcion (`sin_repetir`) que usan el mensaje de `verify` y el gate del cierre, para no repetir la divergencia de las features #64, #67 y #69 | 2026-09-05 |
Despues:
| D4 | Un AC lleva una LISTA de comandos (`comandos: Vec<String>`) y `verify` emite un resultado por comando | (a) conservar `Option<String>` y solo avisar que hay mas de uno | El modelo de un comando por AC no era una decision: era el motivo del bug. Avisar deja al autor partiendo el AC a mano, que es disciplina y no estructura. La deduplicacion de "AC en rojo" vive en UNA funcion (`sin_repetir`) que usan el mensaje de `verify` y el gate del cierre, para no repetir la divergencia de las features #64, #67 y #69 | 2026-09-05 |
| D5 | Las dependencias entre features viven en el backlog (`depends_on`) y se declaran con un comando propio (`harness depende`), no solo en `add` | (a) solo `add --depends-on`; (b) reusar el `depends_on` del grafo del hub | Con solo `add` el recorrido P1 del spec es imposible —una dependencia se descubre despues de que las dos features existen— y la deteccion de ciclos queda inalcanzable, porque por `add` el grafo es un DAG por construccion. El `depends_on` de `graph/derive.rs` es otra cosa: relaciona piezas de CODIGO, no features; mismo nombre, dominios distintos | 2026-09-05 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:100 (spec `nombres`), docs/architecture.md:100 (spec `rechaza`) y 638 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/cli.rs` y 11 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:48-65

