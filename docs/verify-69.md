# Verificacion de AC - Feature #69

Corrida: 2026-09-01T23:28:41Z
Raiz de ejecucion: /Users/alan/harness_process-wt/69-una-linea-ac-ilegible-no-desaparece-en-silencio
Resultado: 5 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && out=$(cargo test verify_nombra_la_linea_ilegible 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 899 |
| AC-2 | verde | `cd rust && out=$(cargo test el_gate_se_niega_con_un_ac_ilegible 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 989 |
| AC-3 | verde | `cd rust && out=$(cargo test no_hay_falsos_ilegibles 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 97 |
| AC-4 | verde | `cd rust && out=$(cargo test el_corpus_real_no_tiene_ilegibles 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 95 |
| AC-5 | verde | `cd rust && out=$(cargo test el_bloque_de_codigo_no_dispara_el_aviso 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 87 |
