# Verificacion de AC - Feature #73

Corrida: 2026-09-05T12:14:09Z
Raiz de ejecucion: /Users/alan/harness_process-wt/73-verify-corre-un-comando-por-ac-y-no-lo-dice-un-a
Resultado: 3 verde(s), 0 en rojo, 6 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test --locked verificacion` | 0 | 4310 |
| AC-2 | manual | `(verificacion manual)` | - | 0 |
| AC-3 | manual | `(verificacion manual)` | - | 0 |
| AC-4 | verde | `cd rust && cargo test --locked --test cli_basics verify` | 0 | 14262 |
| AC-5 | manual | `(verificacion manual)` | - | 0 |
| AC-6 | manual | `(verificacion manual)` | - | 0 |
| AC-7 | manual | `(verificacion manual)` | - | 0 |
| AC-8 | verde | `cd rust && cargo test --locked` | 0 | 143810 |
| AC-9 | manual | `(verificacion manual)` | - | 0 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
