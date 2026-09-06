# Verificacion de AC - Feature #78

Corrida: 2026-09-06T22:51:44Z
Raiz de ejecucion: /Users/alan/harness_process-wt/78-el-instalador-respalda-sus-scripts-pero-no-el-ba
Resultado: 2 verde(s), 0 en rojo, 6 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `bash tests/setup_smoke.sh` | 0 | 89116 |
| AC-2 | manual | `(verificacion manual)` | - | 0 |
| AC-3 | manual | `(verificacion manual)` | - | 0 |
| AC-4 | manual | `(verificacion manual)` | - | 0 |
| AC-5 | manual | `(verificacion manual)` | - | 0 |
| AC-6 | verde | `bash tests/parity_check.sh` | 0 | 992 |
| AC-7 | manual | `(verificacion manual)` | - | 0 |
| AC-8 | manual | `(verificacion manual)` | - | 0 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
