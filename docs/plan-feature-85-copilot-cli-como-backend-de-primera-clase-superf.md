# Plan - Feature #85: Copilot CLI como backend de primera clase: superficie .github/copilot-instructions.md, hooks en .github/copilot.json, agentes en .github/agents, fila en doctor y backend LLM en la tabla de CLIs

Estado: in_progress
Microservicios:
- (sin servicios)

## Alcance
Copilot CLI entra al arnes en los cuatro puntos donde viven los backends:
instalador (hooks y guia, con `harness copilot instalar|quitar` en el binario
para que el formato viva en un solo lugar), runtime de hooks (`copilot-json`
en sh y ps1), `doctor` y la tabla de CLIs de `consolidar`. Los agentes de
`.github/agents/` quedan fuera hasta poder verificar el formato en vivo.

## Impacto entre microservicios
- Sin servicios: solo el arnes.

## Consulta al grafo (graphify)
- `harness contexto --feature 85`; lecciones que aplican:
  remedios-que-la-herramienta-sugiere (el reason y los flags tienen que ser
  ciertos), probar-contra-datos-reales (el CLI 1.0.83 real: `--help`),
  promesas-estructurales-vs-disciplina (el merge idempotente, no "acordarse").

## Delegacion (implementer)
- `rust/src/copilot.rs` (nuevo): `mezclar_config(json, hook) -> (Value, avisos)`,
  `bloque_de_instrucciones()`, `con_bloque(texto) -> String`, `sin_bloque`,
  `sin_hooks_del_arnes`, marcadores; puros (AC-1, AC-2, AC-3).
- `rust/src/commands/copilot.rs` + `cli.rs`: `copilot instalar --raiz --hook`,
  `copilot quitar --raiz` (escritura atomica, mensajes).
- `setup_harness.sh`: `--copilot`/`--no-copilot`, deteccion en PATH, respaldo,
  llamada al binario, `--reset`, texto AGENTS; heredoc del runtime con el modo
  `copilot-json` y `stopHookActive`. `setup_harness.ps1`: lo mismo
  (`-Copilot`/`-NoCopilot`, `Write-PowerShellHookRuntime` con `copilot-json`).
- `rust/src/doctor.rs`: fila en `BACKENDS`. `rust/src/consolidacion.rs`:
  `("copilot", &["-s", "-p"])` al final de `CLIS`; mensaje de sin-backend.
- Tests: unitarios en `copilot.rs`; integracion en `rust/tests/cli_basics.rs`
  (`copilot_instalar`, `copilot_instrucciones`, `copilot_quitar`,
  `doctor_copilot`, `backend_copilot`); `tests/copilot_hook_check.sh` (fixture
  por instalador con `copilot` falso; alimenta el JSON de `agentStop` con
  `stopHookActive` true/false); secciones `copilot` en los dos smokes y en
  `TEMAS` de `parity_check.sh`.
- Docs: README, UPDATING x2, architecture, roles/README, AGENTS.md raiz.
  Corpus de `verificacion.rs`: AC-8 (MANUAL).

## Criterios de cierre (reviewer)
- Cada AC con test o comando; rojos por aserto contra HEAD donde sea posible
  (los subcomandos nuevos caen por "unrecognized").
- Nada del usuario pisado: tests con `copilot.json` y `copilot-instructions.md`
  ajenos antes y despues (bytes).
- Suite, clippy `-D warnings`, paridad, smokes; sin `cargo fmt` a secas.

## Riesgos
- `agentStop` podria disparar por cada tool call (doc ambigua): el AC-8 manual
  lo mide con login; si es asi, pasa a informativo y se documenta.
- `.github/` es del usuario: por eso merge y marcadores, y solo con `copilot`
  detectado o pedido.

## Observaciones (decisiones pendientes)
- OBS-1..4: ver el spec; se preguntan con la aprobacion.

---
Cerrado: 2026-09-10T01:55:32Z - status=done - 
