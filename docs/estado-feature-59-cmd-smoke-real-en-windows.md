# Estado archivado - Feature #59: cmd_smoke_real_en_windows
Cerrada: 2026-08-27T19:28:15Z - status=done - CI Windows versionado (.github/workflows/windows-cmd-installer.yml en windows-latest), smoke CMD nativo en tests/cmd_installer_check.ps1 que se NIEGA a correr fuera de Windows en vez de informar skip verde, y el check Bash exige que ambos existan. LIMITACION EXPLICITA: se cierra con la evidencia estatica y local (cmd_installer_check.sh y parity_check.sh verdes, workflow versionado y bien formado); NO se observo el runner windows-latest en verde porque gh no esta autenticado en esta maquina. Decision del usuario 2026-08-27.

---

# Feature #59: cmd_smoke_real_en_windows

Estado: in_progress
Plan: docs/plan-feature-59-cmd-smoke-real-en-windows.md
Spec: docs/spec-feature-59-cmd-smoke-real-en-windows.md

Microservicios:
- harness

Evidencia:
- 
