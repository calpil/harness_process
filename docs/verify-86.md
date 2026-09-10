# Verificacion de AC - Feature #86

Corrida: 2026-09-10T02:31:32Z
Raiz de ejecucion: /Users/alan/harness_process-wt/86-copilot-medido-en-vivo-los-hooks-van-por-el-clau
Resultado: 7 verde(s), 0 en rojo, 1 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `bash tests/hook_runtime_check.sh` | 0 | 16990 |
| AC-2 | verde | `bash tests/parity_check.sh && grep -q 'claude-json stop' setup_harness.sh setup_harness.ps1` | 0 | 817 |
| AC-3 | verde | `grep -q '"claude-json"' setup_harness.ps1 && grep -q 'decision = "block"' setup_harness.ps1` | 0 | 14 |
| AC-4 | verde | `cd rust && cargo test --locked doctor_copilot` | 0 | 406 |
| AC-5 | verde | `! grep -q 'copilot-json\\|INSTALL_COPILOT\\|write_copilot_hooks\\|Write-CopilotHooks\\|Remove-CopilotParts' setup_harness.sh setup_harness.ps1 && ! test -e rust/src/copilot.rs && ! test -e rust/src/commands/copilot.rs && ! test -e tests/copilot_hook_check.sh && ! ./harness copilot --help >/dev/null 2>&1` | 0 | 12 |
| AC-6 | verde | `cd rust && cargo test --locked backend_copilot` | 0 | 103 |
| AC-7 | manual | `(verificacion manual)` | - | 0 |
| AC-8 | verde | `cmp UPDATING.md templates/UPDATING.md && cmp roles/README.md templates/roles/README.md && grep -q "trustedFolders" README.md UPDATING.md && ! grep -q "copilot.json" README.md UPDATING.md docs/architecture.md roles/README.md AGENTS.md` | 0 | 13 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
