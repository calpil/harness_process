Aplicado: 2026-09-08T23:59:58Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #84: leccion partir: parte una leccion sobre el tope a referencias/ con informe y --aplicar, reconoce las secciones por feature en sus formas reales, y el check resume en una linea las lecciones sobre el tope

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 84`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:103 (spec `guarda`) y 307 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md` y 14 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 25 | El aviso de perfil mide crecimiento, no acumulado | aviso_de_perfil_mide_crecimiento | <O1> | El aviso de la #80 (`rules.perfil_pendientes_max`) contaba TODAS las decisiones sin incorporar (268 en este repo, casi todas de agosto y ya destiladas sin cita) y salia en cada cierre para siempre; el remedio habia sido subir el umbral a 300. Ahora cada registro lleva su momento (la bitacora, su timestamp; un plan o spec, el `started_at` de su feature) y el cierre compara con el umbral solo las posteriores a la ultima entrada del perfil: la ultima linea `perfil add|replace` de `history.md` o el `started_at` de la feature mas alta que cita el perfil, la mas reciente; `perfil remove` no cuenta. Sin corte se cuenta todo, como antes. `lecciones status` (texto y `--json`: `perfil_pendientes` sigue siendo lo comparado, mas `perfil_pendientes_total`, `perfil_corte` y `perfil_corte_origen`) y `perfil sugerir` muestran las dos cuentas y el corte. Medido al cerrar sobre este repo: corte desde el backlog (inicio de la #80; la bitacora se perdio el 2026-09-06), 26 nuevas de 270; el umbral vuelve a 25. Disparador: 289 decisiones historicas hacian saltar el aviso en cada cierre (#80) | done (2026-09-07) |
Despues:
| 25 | El aviso de perfil mide crecimiento, no acumulado | aviso_de_perfil_mide_crecimiento | <O1> | El aviso de la #80 (`rules.perfil_pendientes_max`) contaba TODAS las decisiones sin incorporar (268 en este repo, casi todas de agosto y ya destiladas sin cita) y salia en cada cierre para siempre; el remedio habia sido subir el umbral a 300. Ahora cada registro lleva su momento (la bitacora, su timestamp; un plan o spec, el `started_at` de su feature) y el cierre compara con el umbral solo las posteriores a la ultima entrada del perfil: la ultima linea `perfil add|replace` de `history.md` o el `started_at` de la feature mas alta que cita el perfil, la mas reciente; `perfil remove` no cuenta. Sin corte se cuenta todo, como antes. `lecciones status` (texto y `--json`: `perfil_pendientes` sigue siendo lo comparado, mas `perfil_pendientes_total`, `perfil_corte` y `perfil_corte_origen`) y `perfil sugerir` muestran las dos cuentas y el corte. Medido al cerrar sobre este repo: corte desde el backlog (inicio de la #80; la bitacora se perdio el 2026-09-06), 26 nuevas de 270; el umbral vuelve a 25. Disparador: 289 decisiones historicas hacian saltar el aviso en cada cierre (#80) | done (2026-09-07) |
| 26 | Partir una leccion sobre el tope es un comando | leccion_partir | <O1> | El tope de la #80 se sostenia con un contrato que solo veia titulos `(feature #N)` y un procedimiento a mano. Medido el 2026-09-08 en realestate: cuatro lecciones sobre el tope (2361, 398, 382 y 275 lineas) con titulos `(#115, fecha)`, `feature #100 (fecha)`, `(fecha, front #131)` y `Patch #100:` que el contrato no nombraba, y un Stop con cuatro parrafos `[i]`. Ahora `leccion partir <clase>` informa que secciones cuentan UNA feature o sesion (titulo con `#N` o fecha, fuera de cuando aplica / procedimiento / pitfalls / verificacion / referencias), cuantas lineas quedarian y cuanto falta; `--aplicar` respalda (`lecciones rollback` lo deshace), mueve cada seccion tal cual a `docs/lecciones/<clase>/referencias/<slug>.md` con su cabecera y deja un puntero de una linea en el indice `## Referencias` (lo crea si falta); `--seccion "<titulo>"` suma una elegida, nunca una canonica. Si sigue sobre el tope, lo movido queda y sale 2 con lo que falta y las secciones mas grandes. El contrato de `close`/`usar` y `lecciones status` nombran el comando; `harness_check.sh` avisa con UNA linea por todas las lecciones sobre el tope. Medido al cerrar sobre una copia de realestate: 2361 -> 1994, 398 -> 359, 382 -> 266, 275 sin candidatas; ninguna baja del tope con lo mecanico, y el comando lo dice con el numero. Disparador: el Stop de realestate del 2026-09-08 y la decision del usuario de hacerlo comando en vez de partirlas a mano | done (2026-09-08) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `ninguna`), docs/prd/SDD-master.md:101 (spec `cuando`) y 322 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md` y 14 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D12 | El aviso de perfil cuenta las decisiones sin incorporar posteriores a la ultima entrada del perfil; el corte es la mas reciente entre la ultima linea `perfil add|replace` de la bitacora y el `started_at` de la feature mas alta que cita el perfil; un plan o spec se fecha por el inicio de su feature y sin fecha se cuenta; sin corte se cuenta todo | (a) solo la bitacora; (b) una fecha escrita dentro de `docs/perfil-usuario.md` por `perfil add`; (c) fechar planes y specs por el cierre de su feature; (d) que `perfil remove` tambien reinicie la cuenta | Solo la bitacora es exacta pero se pierde con ella: el 2026-09-06 se perdio y el aviso habria seguido en 268 hasta el proximo `perfil add`; el backlog (con espejo desde #79) reconstruye una cota inferior. Una fecha dentro del perfil cambia el formato del documento del usuario y una edicion a mano no la actualiza. `started_at` es cota inferior: avisa de mas, nunca de menos; `closed_at` puede dejar afuera decisiones posteriores al corte en features viejas. Quitar una entrada no incorpora nada. Decisiones del usuario del 2026-09-07 (OBS-1..3 de la #82) | 2026-09-07 |
Despues:
| D12 | El aviso de perfil cuenta las decisiones sin incorporar posteriores a la ultima entrada del perfil; el corte es la mas reciente entre la ultima linea `perfil add|replace` de la bitacora y el `started_at` de la feature mas alta que cita el perfil; un plan o spec se fecha por el inicio de su feature y sin fecha se cuenta; sin corte se cuenta todo | (a) solo la bitacora; (b) una fecha escrita dentro de `docs/perfil-usuario.md` por `perfil add`; (c) fechar planes y specs por el cierre de su feature; (d) que `perfil remove` tambien reinicie la cuenta | Solo la bitacora es exacta pero se pierde con ella: el 2026-09-06 se perdio y el aviso habria seguido en 268 hasta el proximo `perfil add`; el backlog (con espejo desde #79) reconstruye una cota inferior. Una fecha dentro del perfil cambia el formato del documento del usuario y una edicion a mano no la actualiza. `started_at` es cota inferior: avisa de mas, nunca de menos; `closed_at` puede dejar afuera decisiones posteriores al corte en features viejas. Quitar una entrada no incorpora nada. Decisiones del usuario del 2026-09-07 (OBS-1..3 de la #82) | 2026-09-07 |
| D13 | `leccion partir` mueve solo lo mecanico —las secciones cuyo titulo trae `#N` o una fecha, nunca las canonicas— informa antes de escribir, deja lo movido y sale 2 si sigue sobre el tope, y `--seccion` se niega sobre una canonica | (a) solo titulos `(feature #N)` como en la #80; (b) todo o nada: no escribir si no llega al tope; (c) permitir mover canonicas con `--seccion`; (d) partir las lecciones de realestate a mano desde el arnes | Solo `(feature #N)` no ve ninguna de las once secciones reales de realestate. Todo o nada esconde el progreso: mover ocho secciones de un gigante de 2361 lineas es trabajo hecho y reversible con rollback; el exit 2 dice que el objetivo (bajar del tope) no se cumplio. Las canonicas son la clase: si lo que sobra esta ahi, hay que reescribir, y eso no es un comando. Partirlas a mano desde aca cruza la linea de no tocar otros proyectos: se entrega el comando. Decisiones del usuario del 2026-09-08 (OBS-1..3 de la #84) | 2026-09-08 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:102 (spec `seccion`), docs/architecture.md:108 (spec `master`), docs/architecture.md:109 (spec `candidato`) y 621 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md` y 14 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:161-171

