# Spec - Feature #59: cmd_smoke_real_en_windows

Estado: approved
Aprobado: 2026-08-26T01:15:23Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-59-cmd-smoke-real-en-windows.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)
ANTES: Camila cambia `setup_harness.cmd` desde macOS. El check local queda
verde aunque no existe `cmd.exe`, asi que descubre el error recien cuando una
persona instala el arnes en Windows.
DESPUES: cada cambio relevante ejecuta el wrapper con el `cmd.exe` real de un
runner Windows; si deja de traducir una opcion, propagar un exit code o abrir
el PowerShell correcto, el cambio no pasa a integrar.

## Hoy -> Como va a funcionar
```
HOY                                      DESPUES
cambio .cmd -> Bash sin cmd.exe -> skip  cambio .cmd/.ps1/test -> CI Windows
                                                          |__ cmd.exe -> setup_harness.cmd
                                                                  |__ setup_harness.ps1/sandbox
                                                                  |__ aserciones y exit code
```

## Recorridos de usuario (priorizados)
- P1: Como mantenedor, quiero que el runner Windows ejecute el wrapper CMD
  real, para detectar regresiones que un host Unix no puede ejecutar.
- P2: Como autor de un cambio, quiero un fallo que nombre la traduccion,
  delegacion o codigo de salida roto, para corregir el launcher sin revisar
  manualmente una instalacion Windows.

## Criterios de aceptacion (Given/When/Then)
- AC-1: Given un cambio en `setup_harness.cmd`, `setup_harness.ps1` o su
  prueba, When se abre un PR o se actualiza la rama, Then un workflow en
  `windows-latest` ejecuta el smoke CMD nativo.
- AC-2: Given el runner Windows, When el smoke invoca el `setup_harness.cmd`
  real con `--version`, Then arranca el `setup_harness.ps1` real y devuelve su
  version con exit code 0.
- AC-3: Given un sandbox con el wrapper real y un PowerShell controlado, When
  recibe `--dry-run`, `--no-subagents` y una opcion PowerShell, Then los dos
  primeros llegan en PascalCase y la opcion nativa llega sin alterarse.
- AC-4: Given el sandbox, When el PowerShell delegado devuelve un error o falta
  junto al wrapper, Then CMD conserva el exit code del delegado o devuelve 127
  y nombra `setup_harness.ps1`.
- AC-5: Given una maquina que no sea Windows, When se intenta ejecutar el nuevo
  smoke, Then falla de forma explicita en vez de informar un skip verde.
- AC-6: Given el workflow y el smoke, When se revisan las instrucciones de
  verificacion, Then queda claro que la prueba runtime se ejecuta en CI Windows
  y el check Bash local solo conserva cobertura estatica.

## Los datos que se tocan
- disparador: cambios en el launcher, el instalador PowerShell, el smoke CMD o
  el workflow Windows; y ejecucion manual del workflow.
- entrada: argumentos del launcher (`--kebab-case` y `-PascalCase`) y el exit
  code del PowerShell delegado.
- salida: resultado del job Windows y mensajes de asercion del smoke.
- candado: el smoke exige Windows/`cmd.exe`; no transforma la ausencia del
  runtime en un resultado verde.

## Pseudo-codigo (el acuerdo)
```
CUANDO cambia el contrato del launcher CMD o se dispara el workflow

  ¿el runner es Windows y tiene cmd.exe? -> si no, el smoke falla claramente
  ¿el wrapper real abre el instalador real? -> si no, falla
  ¿el sandbox conserva argumentos y exit codes? -> si no, falla

  ENTONCES publicamos el resultado del job,
           sin duplicar el instalador PowerShell en CMD ni instalar el arnes
           dentro del checkout fuente.
```
Promesas: prueba runtime solo en Windows · argumentos y exit codes observables ·
CMD sigue siendo un wrapper del PS1, no un tercer instalador.

## No funcionales
- SLOs: el job es acotado al launcher y no compila ni instala dependencias no
  necesarias para probarlo.
- Seguridad: usa `ExecutionPolicy Bypass` solo dentro del proceso que el
  launcher ya controla; no cambia la policy de la maquina ni usa secretos.
- Observabilidad: cada asercion identifica el contrato fallido y CI conserva su
  salida en el log del job.

## Fuera de alcance
- Convertir CMD en un instalador independiente.
- Ejecutar una instalacion completa o compilar Rust desde el job del launcher.
- Reemplazar los smokes Bash o PowerShell ya existentes.

## Observaciones (decisiones pendientes)
- Decision: GitHub Actions es el entorno Windows de referencia porque el repo
  no tiene un runner CMD disponible en el entorno de desarrollo actual.
