# Verificacion de AC - Feature #83

Corrida: 2026-09-07T01:49:31Z
Raiz de ejecucion: /Users/alan/harness_process-wt/83-el-stop-hook-bloquea-por-graphify-out-graphify-s
Resultado: 5 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `bash tests/graphify_stale_check.sh` | 0 | 2523 |
| AC-2 | verde | `bash tests/graphify_stale_check.sh` | 0 | 2434 |
| AC-3 | verde | `cmp harness_check.sh templates/harness_check.sh` | 0 | 4 |
| AC-4 | verde | `bash tests/setup_smoke.sh` | 0 | 85809 |
| AC-5 | verde | `cmp UPDATING.md templates/UPDATING.md` | 0 | 5 |
