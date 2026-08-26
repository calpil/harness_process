# Plan - Feature #59: cmd_smoke_real_en_windows

Estado: in_progress
Microservicios:
- harness

## Alcance
- Ejecutar el contrato runtime de `setup_harness.cmd` en un runner Windows
  real, sin convertirlo en un tercer instalador.
- Mantener el check Bash como cobertura estatica/local y agregar un smoke
  PowerShell que falle fuera de Windows.
- Publicar un workflow acotado a los archivos que definen ese contrato.
## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->
- `harness`: launcher `setup_harness.cmd`, delegado `setup_harness.ps1`,
  pruebas de instalacion y CI de GitHub Actions.
- No hay servicios externos, datos persistidos ni APIs de producto afectados.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->
- Consulta: "CI GitHub Actions Windows cmd.exe setup_harness.cmd tests
  cmd_installer_check". Confirmo que `setup_harness.cmd` delega al PS1 y que
  `tests/cmd_installer_check.sh` omite runtime sin `cmd.exe`; no existe un
  workflow Windows versionado.

## Delegacion (implementer)
- U1 [AC-2, AC-3, AC-4, AC-5]: crear `tests/cmd_installer_check.ps1`, que
  exige Windows, invoca el launcher real y usa un sandbox para observar
  argumentos, exit code y diagnostico de ausencia.
- U2 [AC-1, AC-6]: crear el workflow `windows-latest`, con disparadores
  limitados al launcher, delegado y pruebas; ejecuta el smoke nativo con
  PowerShell sin instalar ni compilar el arnes.
- U3 [AC-1..AC-6]: actualizar `docs/verification.md`, comprobar sintaxis y
  estructura local, y dejar evidencia por AC en `docs/impl-59.md`.

## Criterios de cierre (reviewer)
- El spec debe seguir aprobado y fresco; cada AC tiene evidencia.
- El smoke no puede informar un skip verde fuera de Windows.
- El workflow usa un runner Windows, llama al smoke nativo y sus paths cubren
  los archivos del contrato.
- `bash tests/parity_check.sh`, `bash tests/cmd_installer_check.sh`, checks de
  formato y las pruebas Rust pertinentes pasan; el job Windows queda listo
  para verificar runtime real al publicarse.

## Riesgos
- No hay `cmd.exe` en el entorno actual: no se puede ejecutar el smoke nuevo
  localmente; el script debe fallar con diagnostico fuera de Windows y CI es la
  evidencia runtime.
- Un quoting incorrecto entre PowerShell y CMD puede ocultar el argumento;
  el sandbox comprueba tanto flags traducidos como flags nativos.
- El repo no tiene workflows previos: el YAML debe ser autocontenido y de bajo
  costo.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
 implementer DEBE preguntar al usuario que decision aplicar ANTES de
 implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Decision registrada en el spec: GitHub Actions `windows-latest` es el runner
  de referencia; no quedan decisiones pendientes.

### Avance 2026-08-26T01:16:45Z
Re-sincronizado con el plan de CI Windows y smoke CMD nativo, trazado a AC-1..AC-6.
