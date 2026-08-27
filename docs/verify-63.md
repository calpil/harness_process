# Verificacion de AC - Feature #63

Corrida: 2026-08-27T19:39:13Z
Raiz de ejecucion: /Users/alan/harness_process-wt/63-el-arnes-no-afirma-lo-que-no-puede-comprobar
Resultado: 7 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `bash tests/commit_guard_check.sh` | 0 | 25404 |
| AC-2 | verde | `bash tests/commit_guard_check.sh limite` | 0 | 1021 |
| AC-3 | verde | `bash tests/commit_guard_check.sh limite` | 0 | 1025 |
| AC-4 | verde | `bash tests/commit_guard_check.sh prueba-del-rojo` | 0 | 12080 |
| AC-5 | verde | `cd rust && cargo test estado_archivado_apunta_a_donde_quedo_el_archivo` | 0 | 1558 |
| AC-6 | verde | `cd rust && cargo test estado_archivado_sin_integrar_mantiene_la_ruta_real` | 0 | 1018 |
| AC-7 | verde | `cd rust && cargo test ruta_del_estado_archivado_es_pura` | 0 | 101 |
