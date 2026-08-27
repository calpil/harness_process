# Verificacion de AC - Feature #60

Corrida: 2026-08-27T16:28:37Z
Raiz de ejecucion: /Users/alan/harness_process-wt/60-la-vuelta-al-prd-no-se-pierde-ni-miente
Resultado: 12 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test close_should_write_the_prd_echo_in_the_root_not_the_worktree` | 0 | 1138 |
| AC-2 | verde | `cd rust && cargo test close_should_not_touch_the_prd_when_integration_fails` | 0 | 1019 |
| AC-3 | verde | `cd rust && cargo test dos_cierres_en_paralelo_conservan_las_dos_bitacoras` | 0 | 1461 |
| AC-4 | verde | `cd rust && cargo test punteros_de_bitacora_son_relativos_a_la_raiz` | 0 | 87 |
| AC-5 | verde | `cd rust && cargo test bitacora_omite_el_puntero_que_no_resuelve` | 0 | 81 |
| AC-6 | verde | `cd rust && cargo test prd_doctor_reporta_y_no_escribe` | 0 | 904 |
| AC-7 | verde | `cd rust && cargo test prd_doctor_reparar_arregla_punteros_y_bitacoras_faltantes` | 0 | 934 |
| AC-8 | verde | `cd rust && cargo test aviso_de_vuelta_al_prd_fallida_no_cambia_el_cierre` | 0 | 1123 |
| AC-9 | verde | `bash tests/prd_doctor_check.sh check` | 0 | 827 |
| AC-10 | verde | `cd rust && cargo test vuelta_al_prd_es_idempotente` | 0 | 88 |
| AC-11 | verde | `cd rust && cargo test decidir_vuelta_es_pura_y_no_escribe` | 0 | 86 |
| AC-12 | verde | `bash tests/prd_doctor_check.sh repo` | 0 | 8 |
