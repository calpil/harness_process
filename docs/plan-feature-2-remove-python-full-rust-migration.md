# Plan - Feature #2: remove_python_full_rust_migration

Estado: in_progress

Microservicios:

- (sin servicios)

## Alcance

- Eliminar completamente los scripts Python legacy:
  - `harness.py`
  - `graph_memory.py`
  - `templates/harness.py`
  - `templates/graph_memory.py`
  - (dejar __pycache__ se limpia solo; .gitignore se puede limpiar)
- Purgar **toda** referencia a Python, fallback, pip, psycopg2, graphifyy (via pip) y dual Rust+Python en:
  - `harness_cli` (raiz y `templates/harness_cli`)
  - `harness_cli.ps1` (raiz y `templates/harness_cli.ps1`)
  - `setup_harness.sh`
  - `setup_harness.ps1`
  - `init.sh` y `templates/init.sh`
  - `tests/setup_smoke.sh` y `tests/setup_smoke.ps1`
  - `tests/parity_smoke.sh` (eliminar el archivo completo; ya no hay oraculo Python)
  - `README.md`, `UPDATING.md`, `templates/UPDATING.md`
  - `docs/verification.md`, `templates/docs/verification.md`
  - `docs/architecture.md`, `docs/conventions.md` (si aplica)
  - Cualquier otro .sh/.ps1/.md con menciones de fallback Python o "harness.py"
- Simplificar `harness_cli*`:
  - Siempre priorizar y `exec` el binario nativo (`harness` / `harness.exe`).
  - Si no existe binario: error claro "harness binary not found in ...; run setup_harness with rust/cargo available" + exit 127. Sin fallback.
- En `setup_harness.sh` + `.ps1`:
  - Remover `graph_memory.py` / `harness.py` de listas de assets requeridos y generated.
  - Remover llamadas a `install_asset` para los .py y sus `chmod +x`.
  - Remover `python3` de la lista de comandos requeridos (for command_name in ...).
  - Remover toda la seccion de "Mantenimiento dual Rust + Python".
  - Actualizar textos de "binario Rust (build-on-setup)": ahora es obligatorio para que `harness_cli` funcione; warnings de "caera a Python" -> errores o "sin cargo el harness quedara inutilizable".
  - Remover funciones `Get-PythonCommand`, `Ensure-Psycopg`, `Invoke-PostgresMigration` (la parte py), y llamadas a pip install graphifyy / psycopg2-binary.
  - La instalacion de graphify (tool externa via uv/pipx) se puede dejar como opcional o mover nota, pero purgar refs a "Python pip".
  - Cargo build pasa a ser el camino principal; si falla o no hay cargo y no hay bin preexistente -> error (o exit no-zero en setup si !dry).
  - Actualizar secciones de docs internas en el script.
- Actualizar `init.sh` / `templates/init.sh`:
  - Cambiar el chequeo de disponibilidad: solo verificar binario harness/harness.exe ; quitar "ni python3".
  - Mensaje de error actualizado.
- Tests:
  - Eliminar `tests/parity_smoke.sh` (o git rm + actualizar cualquier ref).
  - En `tests/setup_smoke.sh` y `.ps1`: eliminar fixtures de fake-python, PYTHONPATH, asserts sobre graph_memory.py / harness.py , comentarios "caera al fallback Python".
  - Adaptar pruebas que asumian layout sin rust/: ahora los tests que ejercitan harness_cli requieren que el fixture incluya rust/ o el binario (o test solo la parte de copiado de templates sin invocar cli graph/harness cmds que necesitan bin).
  - Mantener `bash tests/setup_smoke.sh` y smoke ps1 en verde.
- Documentacion:
  - Requisitos: quitar "Python 3 (instalador y fallback)", ahora "Rust + cargo (requerido para compilar el binario harness durante setup)".
  - Actualizar todas las secciones que mencionan "sin cargo usa Python", "dual", "parity", "oraculo Python".
  - En UPDATING.md: remover la seccion de mantenimiento dual y la obligacion de parity_smoke.sh ; simplificar a "cambios en rust/src + templates shims + setup; rebuild y smoke".
  - Actualizar ejemplos de uso si listan python.
- `.gitignore`: opcionalmente limpiar `*.py[cod]` (ya no hay .py fuente); dejar `__pycache__/` y `/bkp/` etc.
- Despues de cambios: 
  - `cargo clippy -- -D warnings`, `cargo test`
  - `bash tests/setup_smoke.sh`
  - `sh harness_check.sh` (debe salir limpio)
  - `sh harness_cli check-plan` (debe ser fresco)
  - `sh harness_cli status`
  - Opcional: rebuild del bin local si se quiere test end to end de nuevo cli.

## Impacto entre microservicios

- Cambio puramente interno del harness (sin microservicios registrados).
- `sh harness_cli graph mapa` (o equivalente) no muestra microservicios afectados.
- El Hub / grafo de features se mantiene via el binario Rust (sin cambio de formato de feature_list.json ni progress/).

## Consulta al grafo (graphify)

- (Se generara AST local si hace falta durante impl; se consultara para validar que no queden refs a py en paths criticos.)

## Delegacion (implementer)

- Editar/crear plan + feature state via harness_cli (add/start ya hecho).
- Edits en shims (4 archivos: harness_cli + .ps1 + templates counterparts).
- Edits grandes en setup_harness.sh y setup_harness.ps1 (remover bloques py, simplificar).
- Edits en init.* , tests/* , docs/* (incluyendo este plan, luego impl-2, review-2, estado).
- git rm de los 4 .py + limpiar si queda pycache en templates/.
- Usar search_replace + write para cambios; terminal solo para git ops, rm, tests y harness_cli.
- Escribir evidencia en progress/ (advance) + docs/impl-2.md .
- Al final: harness_cli close + harness_check limpio.

## Criterios de cierre (reviewer)

- feature_list.json refleja feature #2 done.
- Plan fresco (check-plan = 0).
- progress/current.md y history.md actualizados y consistentes con cierre.
- 0 menciones residuales de harness.py / graph_memory.py / "fallback Python" / Get-Python en sh/ps1/md (excepto en bkp/ y comentarios historicos explicitos).
- harness_cli (sh y ps1) solo ejecutan el binario o fallan claramente.
- setup_harness.sh y .ps1 ya no referencian ni instalan python assets ni pip para el harness.
- init.sh ya no menciona python como alternativa.
- tests/setup_smoke.* pasan (sin asumir py).
- `cargo test`, clippy limpio.
- `harness_check.sh` pasa sin bloqueos (o bloqueo documentado explicitamente).
- `docs/review-2.md` con veredicto approved.
- Commit final limpio (sin co-author trailers) siguiendo convenciones.
- (Opcional) push si se desea, pero scope del query prioriza la migracion + estado harness limpio.

## Riesgos

- Algunos usuarios de harness en entornos sin Rust/cargo quedaran sin harness_cli funcional despues de re-setup (era esperado: ahora rust es req duro).
- Tests smoke que usan fixtures minimal sin rust/ necesitaran adaptacion (copiar rust/ o limitar alcance de la prueba a instalacion de assets).
- graphify tool (el bin externo) sigue siendo util via uv/pipx si el usuario lo quiere; no se toca su instalacion mas alla de limpiar notas "python pip".
- El binario "harness" actual (pre-migracion) seguira funcionando; los shims nuevos seran compatibles.

### Avance 2026-06-11T04:12:47Z
Feature iniciada via harness_cli; plan esqueleto generado (y completado manualmente por implementer para detalle).

---

### Avance 2026-06-11T04:15:21Z
Plan completado con detalles completos de la migracion (eliminacion total Python, shims/setup solo Rust, actualizacion docs/tests). Firma del plan resincronizada.

### Avance 2026-06-11T04:22:43Z
Shims (4), setups (2), init (2), tests adaptados, git rm 4py+parity, .gitignore, docs (README UPDATING verification), harness_check limpio, cargo check/clippy/test OK. Listo para close.

---
Cerrado: 2026-06-11T04:24:11Z - status=done - Python eliminado completamente (4 .py + parity + fallbacks en sh/ps1/setup/init/tests/docs); solo Rust. harness_check limpio, cargo verde, evidencia en progress/impl/review.
