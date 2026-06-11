# Implementacion - Feature #1: powershell_windows_installer

## Archivos modificados

- `setup_harness.ps1`: instalador PowerShell nativo.
- `harness_cli.ps1`, `templates/harness_cli.ps1`: shim Rust/Python.
- `setup_harness.sh`: mantiene Unix e instala tambien el shim PowerShell.
- `tests/setup_smoke.ps1`, `tests/setup_smoke.sh`: cobertura de ambos caminos.
- `README.md`, `UPDATING.md`, `templates/UPDATING.md`.
- `docs/verification.md`, `templates/docs/verification.md`.

## Decisiones

- Se mantienen `setup_harness.sh` y `setup_harness.ps1` en paralelo.
- PowerShell soporta layouts root/subdir, backups, reset, dry-run, JSON,
  config, superficies, agentes, hooks y launchers PowerShell.
- Cargo se resuelve desde `PATH`, `CARGO_HOME/bin` o `~/.cargo/bin`; se agrega
  al PATH del proceso si hace falta.
- El build usa `cargo build --release --locked`, respeta
  `CARGO_TARGET_DIR`/`-CargoTargetDir` y copia `harness.exe`.
- `harness_cli.ps1` prioriza `harness.exe`; sin binario usa Python y conserva
  el namespace `graph`.
- Git for Windows Bash sigue siendo requisito para los scripts POSIX
  historicos. No se migro todo el runtime shell en esta feature.

## Verificacion

- Parser PowerShell 7.6.2: los cuatro `.ps1` sin errores.
- `tests/setup_smoke.ps1`: OK, incluido Cargo simulado y reset.
- `bash tests/setup_smoke.sh`: OK.
- `bash tests/parity_smoke.sh`: OK, 30 pasos Rust/Python identicos.
- `cargo clippy --manifest-path rust/Cargo.toml --all-targets --all-features --locked -- -D warnings`: OK.
- `cargo test --manifest-path rust/Cargo.toml --locked`: OK, 33 tests.
- `bash -n setup_harness.sh tests/setup_smoke.sh tests/parity_smoke.sh`: OK.
- `git diff --check`: OK.

## Riesgos pendientes

- No se ejecuto en un host Windows real. La sintaxis y el flujo se probaron con
  PowerShell oficial portable en macOS; el smoke contiene ramas `.cmd` para CI
  Windows y debe incorporarse a una matriz Windows cuando exista.
- `cargo fmt --check` reporta formato pendiente en archivos Rust preexistentes
  no modificados por esta feature; no se aplico un formateo masivo ajeno.
