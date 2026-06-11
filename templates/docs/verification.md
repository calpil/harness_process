# Verificacion

Registra aqui los comandos oficiales por tipo de proyecto.

Ejemplos:

```bash
go test ./...
npm test
npm run lint
bash validate_ui.sh http://localhost:5173
```

Para cambios del instalador:

```bash
bash tests/setup_smoke.sh
# (parity_smoke.sh removido con los .py; solo Rust) # bash tests/parity_smoke.sh
(cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)
(cd rust && cargo test --locked)
```

En Windows PowerShell:

```powershell
.\tests\setup_smoke.ps1
.\harness_cli.ps1 status
```
