Aplicado: 2026-09-07T23:32:17Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #82: el aviso de perfil del cierre cuenta solo las decisiones posteriores a la ultima entrada del perfil

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 82`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:108 (spec `interruptor`), docs/prd/PRD-master.md:110 (spec `master`) y 232 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/leccion.rs` y 6 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 24 | `close` refresca el espejo del backlog y de la bitacora | close_refresca_el_espejo | <O1> | `feature_list.json` y `progress/history.md` estan gitignorados y son lo unico que el instalador no regenera (#78). El 2026-09-06 el checkout se borro por error: el backlog volvio del espejo `docs/bkp-backlog/` refrescado a mano en el ultimo cierre; la bitacora no tenia espejo y se perdio. Ahora cada `close` (cualquier `--status`), despues de guardar el estado y de la linea de bitacora de ese cierre, deja `docs/bkp-backlog/feature_list.json` y `docs/bkp-backlog/history.md` byte-identicos en la raiz del repo principal (copia atomica) y lo dice; quedan sin commitear, como el sello. Politica en `rules.espejo_backlog`: ausente = solo si el directorio existe (el directorio es el opt-in), `true` = crea, `false` = no toca. Best-effort no mudo: si la copia falla, el cierre sigue y avisa `[!]`. Modulo puro `espejo.rs` (politica como enum, decision pura); sin refresco en `add`/`start`. Disparador: la perdida de la bitacora del 2026-09-06 y la decision del usuario de espejarla tambien | done (2026-09-07) |
Despues:
| 24 | `close` refresca el espejo del backlog y de la bitacora | close_refresca_el_espejo | <O1> | `feature_list.json` y `progress/history.md` estan gitignorados y son lo unico que el instalador no regenera (#78). El 2026-09-06 el checkout se borro por error: el backlog volvio del espejo `docs/bkp-backlog/` refrescado a mano en el ultimo cierre; la bitacora no tenia espejo y se perdio. Ahora cada `close` (cualquier `--status`), despues de guardar el estado y de la linea de bitacora de ese cierre, deja `docs/bkp-backlog/feature_list.json` y `docs/bkp-backlog/history.md` byte-identicos en la raiz del repo principal (copia atomica) y lo dice; quedan sin commitear, como el sello. Politica en `rules.espejo_backlog`: ausente = solo si el directorio existe (el directorio es el opt-in), `true` = crea, `false` = no toca. Best-effort no mudo: si la copia falla, el cierre sigue y avisa `[!]`. Modulo puro `espejo.rs` (politica como enum, decision pura); sin refresco en `add`/`start`. Disparador: la perdida de la bitacora del 2026-09-06 y la decision del usuario de espejarla tambien | done (2026-09-07) |
| 25 | El aviso de perfil mide crecimiento, no acumulado | aviso_de_perfil_mide_crecimiento | <O1> | El aviso de la #80 (`rules.perfil_pendientes_max`) contaba TODAS las decisiones sin incorporar (268 en este repo, casi todas de agosto y ya destiladas sin cita) y salia en cada cierre para siempre; el remedio habia sido subir el umbral a 300. Ahora cada registro lleva su momento (la bitacora, su timestamp; un plan o spec, el `started_at` de su feature) y el cierre compara con el umbral solo las posteriores a la ultima entrada del perfil: la ultima linea `perfil add|replace` de `history.md` o el `started_at` de la feature mas alta que cita el perfil, la mas reciente; `perfil remove` no cuenta. Sin corte se cuenta todo, como antes. `lecciones status` (texto y `--json`: `perfil_pendientes` sigue siendo lo comparado, mas `perfil_pendientes_total`, `perfil_corte` y `perfil_corte_origen`) y `perfil sugerir` muestran las dos cuentas y el corte. Medido al cerrar sobre este repo: corte desde el backlog (inicio de la #80; la bitacora se perdio el 2026-09-06), 26 nuevas de 270; el umbral vuelve a 25. Disparador: 289 decisiones historicas hacian saltar el aviso en cada cierre (#80) | done (2026-09-07) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:101 (spec `cuando`), docs/prd/SDD-master.md:103 (spec `cuando`) y 218 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/leccion.rs` y 6 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D11 | El espejo del backlog y de la bitacora se refresca en `close`, en la raiz del repo principal, con el directorio `docs/bkp-backlog/` como opt-in (`Auto`), `rules.espejo_backlog` para forzar o apagar, y fallo que avisa sin impedir el cierre | (a) refrescar en cada comando (`add`, `start`); (b) crear el espejo siempre, sin opt-in; (c) que el fallo del espejo haga fallar el cierre; (d) commitearlo desde `close` | Refrescar en cada comando deja un archivo versionable modificado a cada rato en `git status` sin que nadie lo commitee; el cierre es el cambio de estado que importa. Crearlo sin pedirlo mete un documento versionable en un proyecto que no lo pidio (es del usuario, como el PRD). Un respaldo que impide cerrar es peor que ninguno, pero uno que falla en silencio no existe: `[!]` y sigue. Commitear desde `close` es tocar el repo del usuario desde codigo, la misma linea que #60 y #71 no cruzan | 2026-09-07 |
Despues:
| D11 | El espejo del backlog y de la bitacora se refresca en `close`, en la raiz del repo principal, con el directorio `docs/bkp-backlog/` como opt-in (`Auto`), `rules.espejo_backlog` para forzar o apagar, y fallo que avisa sin impedir el cierre | (a) refrescar en cada comando (`add`, `start`); (b) crear el espejo siempre, sin opt-in; (c) que el fallo del espejo haga fallar el cierre; (d) commitearlo desde `close` | Refrescar en cada comando deja un archivo versionable modificado a cada rato en `git status` sin que nadie lo commitee; el cierre es el cambio de estado que importa. Crearlo sin pedirlo mete un documento versionable en un proyecto que no lo pidio (es del usuario, como el PRD). Un respaldo que impide cerrar es peor que ninguno, pero uno que falla en silencio no existe: `[!]` y sigue. Commitear desde `close` es tocar el repo del usuario desde codigo, la misma linea que #60 y #71 no cruzan | 2026-09-07 |
| D12 | El aviso de perfil cuenta las decisiones sin incorporar posteriores a la ultima entrada del perfil; el corte es la mas reciente entre la ultima linea `perfil add|replace` de la bitacora y el `started_at` de la feature mas alta que cita el perfil; un plan o spec se fecha por el inicio de su feature y sin fecha se cuenta; sin corte se cuenta todo | (a) solo la bitacora; (b) una fecha escrita dentro de `docs/perfil-usuario.md` por `perfil add`; (c) fechar planes y specs por el cierre de su feature; (d) que `perfil remove` tambien reinicie la cuenta | Solo la bitacora es exacta pero se pierde con ella: el 2026-09-06 se perdio y el aviso habria seguido en 268 hasta el proximo `perfil add`; el backlog (con espejo desde #79) reconstruye una cota inferior. Una fecha dentro del perfil cambia el formato del documento del usuario y una edicion a mano no la actualiza. `started_at` es cota inferior: avisa de mas, nunca de menos; `closed_at` puede dejar afuera decisiones posteriores al corte en features viejas. Quitar una entrada no incorpora nada. Decisiones del usuario del 2026-09-07 (OBS-1..3 de la #82) | 2026-09-07 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:105 (spec `registro`), docs/architecture.md:107 (spec `contra`), docs/architecture.md:108 (spec `master`) y 468 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/leccion.rs` y 6 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:145-152

