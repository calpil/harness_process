# Verificacion de AC - Feature #61

Corrida: 2026-08-27T18:33:29Z
Raiz de ejecucion: /Users/alan/harness_process-wt/61-el-merge-del-cierre-no-toca-tu-checkout
Resultado: 7 verde(s), 0 en rojo, 1 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test merge_en_la_rama_abierta_no_usa_el_checkout_principal` | 0 | 3752 |
| AC-2 | verde | `cd rust && cargo test cierre_con_cambios_sin_commitear_que_no_chocan` | 0 | 1569 |
| AC-3 | verde | `cd rust && cargo test colision_se_detecta_antes_de_tocar_nada` | 0 | 1061 |
| AC-4 | verde | `cd rust && cargo test mensaje_de_colision_nombra_archivos_y_remedio` | 0 | 91 |
| AC-5 | verde | `cd rust && cargo test merge_a_rama_no_checkouteada_sigue_funcionando` | 0 | 1145 |
| AC-6 | verde | `cd rust && cargo test conflicto_real_no_deja_nada_a_medias` | 0 | 1214 |
| AC-7 | verde | `cd rust && cargo test colisiones_solo_consulta_y_no_muta` | 0 | 249 |
| AC-8 | manual | `(verificacion manual)` | - | 0 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
