# Verificacion de AC - Feature #67

Corrida: 2026-09-01T22:12:57Z
Raiz de ejecucion: /Users/alan/harness_process-wt/67-los-dos-parsers-del-review-no-se-contradicen
Resultado: 11 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && out=$(cargo test verify_no_ejecuta_documentacion 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 1738 |
| AC-2 | verde | `cd rust && out=$(cargo test estampar_no_toca_la_prosa 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 3459 |
| AC-3 | verde | `cd rust && out=$(cargo test estampar_deja_un_solo_sello 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 3564 |
| AC-4 | verde | `cd rust && out=$(cargo test los_parsers_no_discrepan 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 666 |
| AC-5 | verde | `cd rust && out=$(cargo test corpus_real_sin_cambios 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 109 |
| AC-6 | verde | `cd rust && out=$(cargo test cita_grande_no_se_pudo_comprobar 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 143 |
| AC-7 | verde | `cd rust && out=$(cargo test cita_grande_no_cuelga_el_cierre 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 154 |
| AC-8 | verde | `cd rust && out=$(cargo test la_cita_no_acepta_la_linea_siguiente_al_eof 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 107 |
| AC-9 | verde | `cd rust && out=$(cargo test el_sello_se_encuentra_aunque_haya_lineas_peladas 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 101 |
| AC-10 | verde | `bash tests/conventions_check.sh` | 0 | 3514 |
| AC-12 | verde | `cd rust && out=$(cargo test indentad 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 1430 |
