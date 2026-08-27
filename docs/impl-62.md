# Impl - Feature #62: el_cierre_no_declara_hecho_lo_que_no_hizo

Spec: docs/spec-feature-62-el-cierre-no-declara-hecho-lo-que-no-hizo.md
Origen: deuda anotada en el spec de la #61 (Fuera de alcance), observada al
cerrar la #60 el 2026-08-27.

## El diagnostico

`close::run` escribia NUEVE cosas antes de llamar a `integrar`:

| # | Efecto | Se puede deshacer? |
| --- | --- | --- |
| 1 | `feature_list.json` -> `status: done`, `closed_at`, `leccion` | si |
| 2 | intent de transicion en el outbox de Atlassian | **no** |
| 3 | linea `Cerrado:` en el plan | si (pero vive en el worktree) |
| 4 | `docs/estado-feature-<id>.md` archivado | si (idem) |
| 5 | borrado de `progress/current-<id>.md` | solo si se guardo antes |
| 6 | `progress/current.md` reescrito como indice | si |
| 7 | linea en `progress/history.md` | si, pero es append-only por diseno |
| 8 | memoria del cierre en el hub | **no** |
| 9 | `println!("Feature #N cerrada como done")` | **no** |

`integrar` podia fallar por tres motivos: falta `--to`, colision con trabajo sin
commitear (#61) y conflicto real de merge. En los tres casos las nueve ya habian
pasado. Dos de los efectos no se pueden deshacer de ninguna manera, y por eso
un rollback habria quedado parcial — que es peor que no tenerlo, porque promete
consistencia sin darla.

## Que cambio

`close::run` pasa a tener cuatro fases explicitas, e `integrar` se parte en dos:

| Fase | Que | Funcion |
| --- | --- | --- |
| 0 | lo que puede NEGARSE, antes de escribir nada | gates + `planificar_integracion` |
| 1 | lo que tiene que viajar en la rama | `anotar_plan`, `archivar_estado` |
| 2 | integrar | `ejecutar_integracion` |
| 3 | recien ahora, el estado | el resto de `run` |

`planificar_integracion` devuelve un `PlanDeIntegracion` (`Nada` o `Integrar`)
ya validado: valida `--to` y las colisiones sin ejecutar nada.
`ejecutar_integracion` toma ese plan. Lo unico que puede fallar en FASE 2 es un
conflicto de merge REAL, que no se puede saber sin intentarlo — y ahi el backlog
sigue intacto.

| Archivo | Cambio |
| --- | --- |
| `rust/src/commands/close.rs` | `run` en cuatro fases; `PlanDeIntegracion`; `planificar_integracion` / `ejecutar_integracion`; `anotar_plan` (idempotente) y `archivar_estado` extraidas |
| `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `docs/architecture.md` | documentacion |

## Evidencia por AC

- **AC-1**: `cargo test sin_to_el_backlog_no_queda_en_done`. Sin `--to`: exit 2 y
  la feature sigue `in_progress`, sin `closed_at` y sin `leccion`.
- **AC-2**: `cargo test integracion_fallida_no_escribe_nada_del_estado`. Con la
  colision de la #61: `current-1.md` sigue existiendo, `history.md` no tiene la
  linea, y el stdout NO dice "cerrada como".
- **AC-3**: `cargo test conflicto_de_merge_no_deja_el_backlog_en_done`. Con un
  conflicto real: mismo resultado, y el mensaje ahora dice explicitamente "La
  feature NO quedo marcada como cerrada: el backlog dice la verdad".
- **AC-4**: `cargo test reintentar_el_cierre_no_duplica_artefactos` y
  `reintentar_despues_de_un_conflicto_real_no_duplica_la_anotacion`. El
  reintento completa el cierre con UNA sola entrada en `history.md` y UNA sola
  linea `Cerrado:` en el plan.
- **AC-5**: `cargo test cierre_exitoso_hace_todo_lo_de_siempre`. Backlog `done`
  con `closed_at`, `current-1.md` borrado, `history.md` con la linea, bitacora
  en el PRD, worktree borrado, y el estado archivado viajando en el merge
  (`git show main:docs/estado-feature-1-cobranza.md`).
- **AC-6**: `cargo test cierres_que_no_integran_no_cambian`. `pending` escribe
  el estado, conserva rama y worktree e informa lo que conserva.
- **AC-7**: `cargo test anotar_plan_es_idempotente`. Dos corridas con fechas y
  notas distintas dejan UNA sola anotacion, con la del primer intento; cerrar
  con OTRO estado si anota (son dos hechos distintos); un plan inexistente no es
  un error.
- **AC-8**: `rust/src/commands/close.rs` tiene las cuatro fases marcadas con su
  razon, y ninguna escritura de estado ocurre antes de `ejecutar_integracion`.

## Un hallazgo del test, que mejoro el resultado

El AC-4 asumia que un cierre fallido dejaba el plan ya anotado. El test mostro
que **no**: la FASE 0 se niega antes incluso de escribir los artefactos, asi que
un fallo por `--to` o por colision no deja absolutamente nada. Los artefactos
solo quedan escritos cuando falla la FASE 2 (conflicto real), que es el unico
caso donde la idempotencia hace falta de verdad. Se corrigio la asercion —ahora
verifica el comportamiento real, que es mejor— y se agrego un segundo test
(`reintentar_despues_de_un_conflicto_real_no_duplica_la_anotacion`) que ejercita
el escenario donde la idempotencia importa.

## Lo que NO se toco

Sin rollback (decision USUARIO 2026-08-27). Sin cambios en los gates, el merge
ni la vuelta al PRD. Un cierre exitoso hace exactamente lo mismo que antes, con
una sola diferencia visible: `Feature #N cerrada` se imprime DESPUES de la
salida de `[GitFlow]`, que es el orden real de los hechos.

Fuera de alcance declarado: que el cierre sobreviva a una caida del proceso a
mitad de la FASE 3 (`kill -9`, disco lleno). Esto reduce la ventana a lo minimo
razonable; no la elimina.
