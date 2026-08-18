# Verificacion de AC - Feature #24

Corrida: 2026-08-17T18:22:23Z
Resultado: 17 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `test "$(grep -cE '^[1-5]\. \*\*' docs/conventions.md)" = 5 && grep -q "menor huella que resuelva" docs/conventions.md` | 0 | 9 |
| AC-2 | verde | `test "$(grep -cE '^[1-5]\. \*\*.*\(#[0-9]+' docs/conventions.md)" = 5` | 0 | 5 |
| AC-3 | verde | `grep -q "Peldano elegido:" docs/conventions.md && grep -q "Peldano elegido:" roles/leader.md` | 0 | 6 |
| AC-4 | verde | `grep -q "contratos de comportamiento" docs/conventions.md && grep -q "leer el codigo fuente" docs/conventions.md && grep -q "detector-de-cambios" docs/conventions.md` | 0 | 7 |
| AC-5 | verde | `test "$(grep -c '// NO:' docs/conventions.md)" -ge 3 && test "$(grep -c '// SI:' docs/conventions.md)" -ge 3` | 0 | 7 |
| AC-6 | verde | `grep -q "dato de entrada" docs/conventions.md && grep -q "se reescribiera entera" docs/conventions.md` | 0 | 5 |
| AC-7 | verde | `cd rust && cargo test only_verify_should_execute_declared_commands` | 0 | 994 |
| AC-8 | verde | `bash tests/conventions_check.sh sin-violaciones` | 0 | 323 |
| AC-9 | verde | `grep -q "Auditoria de la suite" docs/impl-24.md` | 0 | 4 |
| AC-10 | verde | `bash tests/conventions_check.sh detecta` | 0 | 377 |
| AC-11 | verde | `bash tests/conventions_check.sh no-bloquea` | 0 | 1204 |
| AC-12 | verde | `bash tests/conventions_check.sh sin-rust` | 0 | 40 |
| AC-13 | verde | `diff -q docs/conventions.md templates/docs/conventions.md` | 0 | 5 |
| AC-14 | verde | `grep -q "escalera" roles/leader.md && grep -q "detector-de-cambios" roles/implementer.md && grep -q "rechaza" roles/reviewer.md` | 0 | 8 |
| AC-15 | verde | `grep -q "escalera de huella" README.md UPDATING.md templates/UPDATING.md` | 0 | 5 |
| AC-16 | verde | `grep -q "Peldano elegido:" docs/plan-feature-24-conventions-escalera-y-tests.md` | 0 | 4 |
| AC-17 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 131 |
