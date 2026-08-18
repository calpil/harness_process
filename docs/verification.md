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

Si preferis que cada AC declare su propia prueba, escribila en el spec debajo
del criterio y corre la verificacion desde aca:

```bash
sh harness_cli verify --feature <id>            # corre lo declarado, escribe el reporte
sh harness_cli verify --feature <id> --solo AC-3
```

Dos trampas al elegir el comando: uno que filtra tests por nombre suele salir
**0 cuando no encuentra ninguno**, y cualquier cosa que termine en `|| true` no
puede fallar. Un comando que no puede fallar no verifica: decora.

En Windows PowerShell:

```powershell
.\tests\setup_smoke.ps1
.\harness_cli.ps1 status
```
