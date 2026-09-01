# Verificacion de AC - Feature #68

Corrida: 2026-09-01T22:44:04Z
Raiz de ejecucion: /Users/alan/harness_process-wt/68-el-arnes-no-pierde-los-ac-que-pide-revisar-a-man
Resultado: 7 verde(s), 0 en rojo, 1 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && out=$(cargo test el_ac_manual_aparece 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 979 |
| AC-2 | verde | `cd rust && out=$(cargo test el_gate_exige_fila_para_el_manual 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 997 |
| AC-3 | verde | `cd rust && out=$(cargo test el_sufijo_de_letra_es_un_ac_propio 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 98 |
| AC-4 | verde | `cd rust && out=$(cargo test el_comando_no_migra_al_ac_anterior 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 92 |
| AC-5 | verde | `cd rust && out=$(cargo test lo_que_no_es_un_ac_sigue_sin_serlo 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 94 |
| AC-6 | verde | `cd rust && out=$(cargo test los_siete_que_faltaban_y_ninguno_mas 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 113 |
| AC-7 | verde | `cd rust && out=$(cargo test las_features_cerradas_no_se_mueven 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 1112 |
| AC-8 | manual | `(verificacion manual)` | - | 0 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
