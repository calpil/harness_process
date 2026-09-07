# Verificacion de AC - Feature #74

Corrida: 2026-09-07T02:45:57Z
Raiz de ejecucion: /Users/alan/harness_process-wt/74-add-no-protege-contra-la-feature-duplicada-y-no-
Resultado: 7 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test --locked add_duplicado_abierto` | 0 | 3252 |
| AC-2 | verde | `cd rust && cargo test --locked add_mismo_nombre_cerrada` | 0 | 901 |
| AC-3 | verde | `cd rust && cargo test --locked add_clave_idempotente` | 0 | 757 |
| AC-4 | verde | `cd rust && cargo test --locked normalizar_nombre` | 0 | 101 |
| AC-5 | verde | `cd rust && cargo test --locked add_sin_clave_no_cambia_el_backlog` | 0 | 714 |
| AC-6 | verde | `cmp UPDATING.md templates/UPDATING.md` | 0 | 5 |
| AC-7 | verde | `bash tests/parity_check.sh` | 0 | 492 |
