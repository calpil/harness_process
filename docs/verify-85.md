# Verificacion de AC - Feature #85

Corrida: 2026-09-10T01:41:36Z
Raiz de ejecucion: /Users/alan/harness_process-wt/85-copilot-cli-como-backend-de-primera-clase-superf
Resultado: 8 verde(s), 0 en rojo, 1 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test --locked copilot_instalar` | 0 | 847 |
| AC-2 | verde | `cd rust && cargo test --locked copilot_instrucciones` | 0 | 690 |
| AC-3 | verde | `cd rust && cargo test --locked copilot_quitar` | 0 | 699 |
| AC-4 | verde | `bash tests/copilot_hook_check.sh` | 0 | 21463 |
| AC-5 | verde | `bash tests/parity_check.sh && bash tests/setup_smoke.sh` | 0 | 111379 |
| AC-6 | verde | `cd rust && cargo test --locked doctor_copilot` | 0 | 369 |
| AC-7 | verde | `cd rust && cargo test --locked backend_copilot` | 0 | 110 |
| AC-8 | manual | `(verificacion manual)` | - | 0 |
| AC-9 | verde | `cmp UPDATING.md templates/UPDATING.md && grep -q "copilot" README.md docs/architecture.md roles/README.md AGENTS.md setup_harness.sh setup_harness.ps1` | 0 | 8 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
