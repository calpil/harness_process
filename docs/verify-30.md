# Verificacion de AC - Feature #30

Corrida: 2026-08-18T02:13:21Z
Resultado: 11 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `bash tests/parity_check.sh opciones` | 0 | 176 |
| AC-2 | verde | `bash tests/parity_check.sh detecta-opcion` | 0 | 187 |
| AC-3 | verde | `bash tests/parity_check.sh asimetrias-declaradas` | 0 | 8 |
| AC-4 | verde | `bash tests/parity_check.sh superficies` | 0 | 18 |
| AC-5 | verde | `bash tests/parity_check.sh smokes` | 0 | 34 |
| AC-6 | verde | `grep -q "no ejecuta el instalador de Windows" docs/verification.md README.md` | 0 | 4 |
| AC-7 | verde | `bash tests/parity_check.sh promesa-acotada` | 0 | 10 |
| AC-8 | verde | `bash tests/parity_check.sh en-harness-check` | 0 | 10 |
| AC-9 | verde | `bash tests/parity_check.sh sin-ps1` | 0 | 388 |
| AC-10 | verde | `grep -q "Peldano elegido:" docs/plan-feature-30-paridad-ps1-verificable.md` | 0 | 5 |
| AC-11 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 369 |
