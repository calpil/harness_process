# Evidencia de implementación — Feature #59

Spec: `docs/spec-feature-59-cmd-smoke-real-en-windows.md` (approved)

## Cambios

- `tests/cmd_installer_check.ps1` exige Windows y ejecuta `cmd.exe` real. Usa
  el launcher y delegado reales para `--version`, y sandboxes para hacer
  observables la traduccion de flags, los exit codes y el diagnostico de un
  delegado ausente.
- `.github/workflows/windows-cmd-installer.yml` ejecuta ese smoke en
  `windows-latest` cuando cambia el contrato del launcher o manualmente.
- El check Bash conserva su rol local/estatico y ahora falla si faltan el smoke
  runtime o el workflow Windows. `docs/verification.md` distingue ambos tipos
  de evidencia.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | El workflow versionado usa `windows-latest`, `workflow_dispatch` y paths para el CMD, PS1, smoke y workflow. El modo `ci-windows` del check Bash exige esas cuatro referencias. |
| AC-2 | El smoke PowerShell invoca `cmd.exe /d /c` sobre el `setup_harness.cmd` real con `--version`, exige exit 0 y el texto `harness-process`; se ejecuta en el job Windows. |
| AC-3 | El sandbox conserva el CMD real y un PS1 controlado; exige `--dry-run -> -DryRun`, `--no-subagents -> -NoSubagents` y `-Force` sin cambios. |
| AC-4 | El mismo sandbox exige el exit code 3 del delegado; un directorio sin PS1 exige 127 y la mención de `setup_harness.ps1`. |
| AC-5 | `tests/cmd_installer_check.ps1` exige `Windows_NT` y un `COMSPEC` existente antes de correr; fuera de Windows falla con un diagnóstico, nunca informa skip verde. |
| AC-6 | `docs/verification.md` declara CI Windows como evidencia runtime y limita el check Bash a cobertura estática/local. |

## Verificación ejecutada

- `bash -n tests/cmd_installer_check.sh`
- `bash tests/cmd_installer_check.sh`
- `bash tests/parity_check.sh`
- `git diff --check`

Todas las verificaciones locales verdes. La ejecución runtime de CMD requiere
el runner `windows-latest` recién versionado; debe observarse verde en CI antes
del cierre e integración.
