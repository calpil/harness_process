# Plan - Feature #86: Copilot medido en vivo: los hooks van por el .claude/settings.json que el arnes ya genera (sin .github/copilot.json), el Stop de los dos runtimes emite decision:block, y doctor revisa la confianza de la carpeta

Estado: in_progress
Microservicios:
- (sin servicios)

## Alcance
Corregir la #85 con lo medido: Copilot lee `.claude/settings.json` y bloquea
solo con `decision: block`. Un modo `claude-json` en los dos runtimes para el
Stop de Claude, `doctor` con la confianza de la carpeta, y la remocion de todo
lo que la #85 escribio para un archivo que no existe. `consolidar` no cambia.

## Impacto entre microservicios
- Sin servicios: solo el arnes.

## Consulta al grafo (graphify)
- Lecciones que aplican: probar-contra-datos-reales (el spec anterior se
  escribio desde docs equivocadas), remedios-que-la-herramienta-sugiere,
  promesas-estructurales-vs-disciplina.

## Delegacion (implementer)
- `setup_harness.sh`: modo `claude-json` en el heredoc del runtime (captura de
  `run_stop`, JSON-escape con sed/awk, stderr), `.claude/settings.json` Stop
  -> `claude-json stop` (dos bloques); quitar `INSTALL_COPILOT`, flags,
  `write_copilot_hooks`, el bloque del reset, `copilot-json`, `stopHookActive`,
  los mapeos `sessionStart|agentStop|sessionEnd`; texto AGENTS corregido.
- `setup_harness.ps1`: lo mismo (`claude-json` en el runtime con salida a
  stderr y JSON en el catch; `Get-HookCommand -Mode "claude-json" -Event "stop"`;
  quitar `-Copilot`/`-NoCopilot`, `Write-CopilotHooks`, `Remove-CopilotParts`,
  `$script:HarnessExe`, `copilot-json`).
- Rust: borrar `copilot.rs` y `commands/copilot.rs`, el comando en `cli.rs`,
  `mod copilot`; `doctor.rs`: `Area::Copilot` + `revisar_copilot` (PATH,
  `COPILOT_HOME`, `trustedFolders`, `claude-json stop`) y sacar la fila
  `copilot` de `BACKENDS`; `consolidacion.rs` queda.
- Tests: borrar `tests/copilot_hook_check.sh` y los `copilot_*` de
  `cli_basics.rs`; nuevo `tests/hook_runtime_check.sh` (AC-1); unitarios de
  `doctor_copilot_*`; `parity_check.sh` (`cableado-hooks` con `claude-json
  stop`, sin `copilot|Copilot` en TEMAS ni superficies); smokes sin las
  aserciones de `.github/copilot.json`; corpus de `verificacion.rs` (AC-7).
- Docs: README (seccion Copilot reescrita), UPDATING x2, architecture,
  roles/README x2, AGENTS.md raiz.

## Criterios de cierre (reviewer)
- AC-1 rojo contra HEAD por su aserto (el modo no existe: stdout vacio y exit 2).
- Suite, clippy, paridad 11/11, smoke entero.
- AC-7 medido con Copilot logueado sobre un fixture instalado.

## Riesgos
- Cambiar el Stop de Claude a JSON: Claude Code documenta `decision: block`
  para Stop; el detalle pasa de stderr a `reason`. Se verifica con la sesion
  de Claude Code de este mismo trabajo (el Stop de este repo no usa el
  instalador, asi que se mide en un fixture).
- El runtime PowerShell no se ejecuta aca.

## Observaciones (decisiones pendientes)
- OBS-1..2: ver el spec; se preguntan con la aprobacion.

---
Cerrado: 2026-09-10T02:34:12Z - status=done - 
