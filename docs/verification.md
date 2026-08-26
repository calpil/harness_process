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

La comprobación cotidiana de consolidación no usa red, cuota ni secretos:

```bash
bash tests/consolidar_check.sh
```

Para pedir la integración real de forma deliberada (requiere un CLI autenticado
`claude` o `kimi`), usa el único interruptor que la habilita:

```bash
bash tests/consolidar_check.sh --real backend-real
```

Dos trampas al elegir el comando: uno que filtra tests por nombre suele salir
**0 cuando no encuentra ninguno**, y cualquier cosa que termine en `|| true` no
puede fallar. Un comando que no puede fallar no verifica: decora.

Paridad de los dos instaladores, **sin PowerShell**:

```bash
bash tests/parity_check.sh
```

Compara lo que `setup_harness.sh` y `setup_harness.ps1` **declaran** (opciones y
superficies) y falla cuando uno se adelanta al otro. Las asimetrias legitimas
estan declaradas en el propio script, cada una con su razon.

Lo que **no** hace, dicho de frente: **no ejecuta el instalador de Windows**. Un
`.ps1` estructuralmente paritario puede fallar igual al correr. Para eso hace
falta una maquina con PowerShell:

```powershell
# si tenes Windows a mano (si no, `parity_check.sh` es lo que hay)
.\tests\setup_smoke.ps1
.\harness_cli.ps1 status
```
