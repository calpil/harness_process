# Verificacion de AC - Feature #56

Corrida: 2026-08-22T16:50:04Z
Resultado: 5 verde(s), 0 en rojo, 11 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | manual | `(verificacion manual)` | - | 0 |
| AC-2 | manual | `(verificacion manual)` | - | 0 |
| AC-3 | manual | `(verificacion manual)` | - | 0 |
| AC-4 | manual | `(verificacion manual)` | - | 0 |
| AC-5 | verde | `cd rust && cargo test contexto_puntero` | 0 | 101 |
| AC-6 | manual | `(verificacion manual)` | - | 0 |
| AC-7 | manual | `(verificacion manual)` | - | 0 |
| AC-8 | verde | `cd rust && cargo test contexto_cobertura` | 0 | 94 |
| AC-9 | manual | `(verificacion manual)` | - | 0 |
| AC-10 | manual | `(verificacion manual)` | - | 0 |
| AC-11 | verde | `cd rust && cargo test contexto_presupuesto` | 0 | 87 |
| AC-12 | manual | `(verificacion manual)` | - | 0 |
| AC-13 | manual | `(verificacion manual)` | - | 0 |
| AC-14 | manual | `(verificacion manual)` | - | 0 |
| AC-15 | verde | `cd rust && cargo test contexto_sin_nada` | 0 | 92 |
| AC-16 | verde | `cd rust && cargo clippy --all-targets -- -D warnings` | 0 | 153 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
