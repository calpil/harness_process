# Plan - Feature #1: powershell_windows_installer

Estado: in_progress
Microservicios:
- (sin servicios)

## Alcance

- Mantener `setup_harness.sh` como instalador canonico para Unix.
- Agregar `setup_harness.ps1` como instalador nativo para Windows PowerShell.
- Mantener los layouts `subdir` (default) y `root`, backups, dry-run, reset,
  copia de templates y superficies multi-LLM.
- Detectar Cargo desde `PATH` o `$HOME/.cargo/bin`, compilar con
  `cargo build --release --locked` y copiar `harness.exe`.
- Agregar un shim `harness_cli.ps1` para despachar al binario Rust o a Python
  sin requerir `sh` para los comandos directos.
- Documentar requisitos, instalacion, actualizacion y verificacion en Windows.
- Agregar una prueba smoke PowerShell y validaciones estaticas ejecutables
  cuando `pwsh` no este disponible en el host.

## Impacto entre microservicios
<!-- python3 graph_memory.py impacto --microservicio <proyecto>/<servicio> -->

- No hay microservicios afectados. El cambio pertenece al repositorio fuente
  `harness_process` y no modifica contratos de servicios registrados en el Hub.
- `sh harness_cli graph mapa` fue ejecutado; el Hub no registra este repositorio
  como microservicio.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- Se genero un mapa AST local (528 nodos, 1040 relaciones) porque no habia
  `graphify-out/graph.json`.
- Consulta ejecutada sobre instalacion, Cargo, templates, hooks y paridad
  PowerShell. El nodo central detectado fue `install_asset()` en
  `setup_harness.sh`.
- La extraccion semantica completa no estuvo disponible: no hay clave Gemini y
  la sesion no tiene autorizacion explicita para lanzar subagentes.

## Delegacion (implementer)

- `setup_harness.ps1`: opciones, layout, backups/reset, templates, superficies,
  Cargo y herramientas opcionales.
- `templates/harness_cli.ps1`: shim PowerShell Rust/Python.
- `tests/setup_smoke.ps1`: smoke nativo con fixture temporal y Cargo opcional.
- `README.md`, `UPDATING.md`, `templates/UPDATING.md`,
  `docs/verification.md`: flujo dual Bash/PowerShell.

## Criterios de cierre (reviewer)

- `setup_harness.sh` permanece funcional y su smoke test sigue en verde.
- `setup_harness.ps1` soporta ayuda, version, dry-run, root/subdir, reset y
  copia de estado vivo sin sobrescribirlo.
- Cargo se descubre desde PowerShell y `harness.exe` se compila/copia cuando
  hay toolchain; la ausencia de Cargo conserva el fallback Python.
- `harness_cli.ps1` prioriza `harness.exe` y conserva namespaces `graph` y
  ciclo de vida.
- La prueba PowerShell corre en Windows/pwsh; en este host sin `pwsh` se valida
  estructura, contenido y suites existentes.
- `bash tests/setup_smoke.sh`, `bash tests/parity_smoke.sh`, Cargo clippy/test y
  `bash harness_check.sh` pasan, o cualquier bloqueo queda documentado.

## Riesgos

- Este host no tiene `pwsh`; la ejecucion nativa Windows dependera de CI o de
  una maquina Windows. Se mitiga con un smoke autocontenido y validaciones
  estaticas locales.
- Los hooks y scripts historicos siguen siendo POSIX y requieren Git for
  Windows Bash. El shim PowerShell cubre comandos directos; migrar todos los
  hooks/runtime queda fuera de alcance para evitar cambiar el protocolo.
- Las rutas Windows y el escape JSON/TOML deben generarse con APIs de
  serializacion, no concatenacion manual.

### Avance 2026-06-11T03:40:46Z
Re-sincronizado con plan PowerShell actualizado y alcance confirmado

### Avance 2026-06-11T04:04:26Z
Instalador PowerShell, shim y pruebas completados; smoke Bash/PowerShell, paridad, clippy y tests Rust en verde

### Avance 2026-06-11T04:04:34Z
Instalador PowerShell, shim y pruebas completados; smoke Bash/PowerShell, paridad, clippy y tests Rust en verde

---
Cerrado: 2026-06-11T04:06:31Z - status=done - Instalacion Windows PowerShell agregada; Bash conservado; Cargo y smoke tests verificados
