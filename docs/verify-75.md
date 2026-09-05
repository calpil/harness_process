# Verificacion de AC - Feature #75

Corrida: 2026-09-05T14:05:47Z
Raiz de ejecucion: /Users/alan/harness_process-wt/75-el-backlog-no-sabe-de-dependencias-ni-de-feature
Resultado: 5 verde(s), 0 en rojo, 5 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test --locked depend` | 0 | 3965 |
| AC-2 | manual | `(verificacion manual)` | - | 0 |
| AC-3 | manual | `(verificacion manual)` | - | 0 |
| AC-4 | verde | `cd rust && cargo test --locked --test cli_basics blocked` | 0 | 3072 |
| AC-5 | manual | `(verificacion manual)` | - | 0 |
| AC-6 | manual | `(verificacion manual)` | - | 0 |
| AC-7 | manual | `(verificacion manual)` | - | 0 |
| AC-8 | verde | `cd rust && cargo test --locked` | 0 | 148076 |
| AC-8 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 329 |
| AC-8 | verde | `bash tests/parity_check.sh` | 0 | 579 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
