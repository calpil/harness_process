# Implementacion - Feature #2: remove_python_full_rust_migration

## Archivos modificados / eliminados

- `harness_cli`, `templates/harness_cli`: shims ahora solo binario Rust; error claro si falta.
- `harness_cli.ps1`, `templates/harness_cli.ps1`: idem, PowerShell sin Get-Python ni fallbacks.
- `setup_harness.sh`: removidos assets py, json_value py, preflight py, psycopg/pip blocks, dual section, required/generated lists limpias, binario ahora requerido (exit 1 si no se produce), graphify pip-user removido, init shims solo Rust.
- `setup_harness.ps1`: Get-Python, Ensure/Invoke psycopg + heredoc py, asset lists, cargo warns, graphify/psycopg pip removidos; migration calls neutralizados.
- `init.sh`, `templates/init.sh`: chequeo de disponibilidad solo binario, mensaje actualizado.
- `tests/setup_smoke.sh`, `tests/setup_smoke.ps1`: fake-python, PYTHONPATH, asserts .py y comentarios fallback eliminados/adaptados.
- `tests/parity_smoke.sh`: git rm (junto con los .py).
- `harness.py`, `graph_memory.py`, `templates/harness.py`, `templates/graph_memory.py`: git rm.
- `.gitignore`: removido *.py[cod].
- `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `docs/verification.md`, `templates/docs/verification.md`: requisitos, secciones dual/paridad actualizadas o removidas.
- `docs/plan-feature-2-...md`, `progress/*`, `feature_list.json`, `history.md` via harness_cli (add/start/advance).
- `docs/impl-2.md`, `docs/review-2.md` (esta).

## Decisiones

- Cargo ahora hard requirement para que el harness sea funcional (setup falla si no produce bin y no hay preexistente, excepto dry-run).
- pycompat.rs y toda la maquinaria de paridad de output en Rust se deja (asegura que el binario siga produciendo mismos JSON/progress que antes, para compatibilidad de estado con installs previas).
- graphify (tool externa) sigue instalable via uv/pipx; solo se removio el fallback python3-pip en el instalador.
- No se toco rust/src/ mas alla de lo necesario (el port ya estaba hecho).
- harness_check.sh pasa limpio con feature activa (plan fresco, sin sucios, sin graphify_stale).

## Verificacion ejecutada

- `env HARNESS_REPO_ROOT=$PWD sh harness_cli check-plan` -> fresco (0)
- `cargo check && cargo clippy -- -D warnings` -> limpio
- ` (cd rust && cargo test --quiet) ` -> 9 tests OK
- `bash -n tests/setup_smoke.sh` -> OK
- `env HARNESS_REPO_ROOT=$PWD bash harness_check.sh` -> [Ok] limpio
- (smoke completo y ps1 bajo pwsh en CI/Windows; aqui sin pwsh se valida estructura)

## Cierre

Se usara `sh harness_cli close --feature 2 --status done --note "Python eliminado; harness 100% Rust (sh+ps1+setup+tests+docs)"`.

Commit posterior sin co-author.
