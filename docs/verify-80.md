# Verificacion de AC - Feature #80

Corrida: 2026-09-07T00:18:53Z
Raiz de ejecucion: /Users/alan/harness_process-wt/80-el-autoaprendizaje-no-tiene-ciclo-de-vida-leccio
Resultado: 8 verde(s), 0 en rojo, 1 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test --locked tope_de_lineas` | 0 | 1326 |
| AC-2 | verde | `cd rust && cargo test --locked usar_rechaza_sobre_el_tope` | 0 | 961 |
| AC-3 | verde | `cd rust && cargo test --locked repeticiones` | 0 | 1455 |
| AC-4 | verde | `cd rust && cargo test --locked aviso_de_perfil` | 0 | 1363 |
| AC-5 | verde | `cd rust && cargo test --locked aviso_de_consolidacion` | 0 | 1331 |
| AC-6 | verde | `bash tests/parity_check.sh` | 0 | 629 |
| AC-6 | verde | `bash tests/leccion_tope_check.sh` | 0 | 3838 |
| AC-7 | manual | `(verificacion manual)` | - | 0 |
| AC-8 | verde | `bash tests/stop_hook_check.sh` | 0 | 4997 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
