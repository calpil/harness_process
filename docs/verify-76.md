# Verificacion de AC - Feature #76

Corrida: 2026-09-06T02:28:33Z
Raiz de ejecucion: /Users/alan/harness_process-wt/76-una-feature-sin-worktree-veta-a-todas-las-demas-
Resultado: 4 verde(s), 0 en rojo, 7 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test --locked aislamiento` | 0 | 338 |
| AC-2 | manual | `(verificacion manual)` | - | 0 |
| AC-3 | manual | `(verificacion manual)` | - | 0 |
| AC-4 | manual | `(verificacion manual)` | - | 0 |
| AC-5 | manual | `(verificacion manual)` | - | 0 |
| AC-6 | manual | `(verificacion manual)` | - | 0 |
| AC-7 | manual | `(verificacion manual)` | - | 0 |
| AC-8 | verde | `cd rust && cargo test --locked` | 0 | 146480 |
| AC-8 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 1414 |
| AC-8 | verde | `bash tests/parity_check.sh` | 0 | 503 |
| AC-9 | manual | `(verificacion manual)` | - | 0 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
