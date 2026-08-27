# Verificacion de AC - Feature #62

Corrida: 2026-08-27T19:06:05Z
Raiz de ejecucion: /Users/alan/harness_process-wt/62-el-cierre-no-declara-hecho-lo-que-no-hizo
Resultado: 7 verde(s), 0 en rojo, 1 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test sin_to_el_backlog_no_queda_en_done` | 0 | 3573 |
| AC-2 | verde | `cd rust && cargo test integracion_fallida_no_escribe_nada_del_estado` | 0 | 1010 |
| AC-3 | verde | `cd rust && cargo test conflicto_de_merge_no_deja_el_backlog_en_done` | 0 | 1216 |
| AC-4 | verde | `cd rust && cargo test reintentar_el_cierre_no_duplica_artefactos` | 0 | 1430 |
| AC-5 | verde | `cd rust && cargo test cierre_exitoso_hace_todo_lo_de_siempre` | 0 | 1360 |
| AC-6 | verde | `cd rust && cargo test cierres_que_no_integran_no_cambian` | 0 | 1026 |
| AC-7 | verde | `cd rust && cargo test anotar_plan_es_idempotente` | 0 | 108 |
| AC-8 | manual | `(verificacion manual)` | - | 0 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
