# Verificacion de AC - Feature #65

Corrida: 2026-08-31T02:27:04Z
Raiz de ejecucion: /Users/alan/harness_process-wt/65-el-arnes-cierra-lo-resuelto-aguas-arriba
Resultado: 10 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && out=$(cargo test resuelto_aguas_arriba 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 937 |
| AC-2 | verde | `cd rust && out=$(cargo test aguas_arriba_exige_referencia 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 796 |
| AC-3 | verde | `cd rust && out=$(cargo test forma_de_la_referencia_externa 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 897 |
| AC-4 | verde | `cd rust && out=$(cargo test referencia_externa_no_se_valida 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 777 |
| AC-5 | verde | `cd rust && out=$(cargo test status_muestra_aguas_arriba 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 807 |
| AC-6 | verde | `cd rust && out=$(cargo test cabecera_de_status_suma 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 947 |
| AC-7 | verde | `cd rust && out=$(cargo test prd_tree_ignora_aguas_arriba 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 95 |
| AC-8 | verde | `cd rust && out=$(cargo test aguas_arriba_no_reabre_el_ticket 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 84 |
| AC-9 | verde | `cd rust && out=$(cargo test todos_los_estados_tienen_su_rama 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 86 |
| AC-10 | verde | `cd rust && out=$(cargo test cierre_sin_io_de_red 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 87 |
