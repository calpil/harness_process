Aplicado: 2026-08-27T19:27:45Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #59: cmd_smoke_real_en_windows

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 59`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:108 (spec `dispara`) y 95 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.DS_Store`, `.github/workflows/windows-cmd-installer.yml`, `docs/estado-feature-59-cmd-smoke-real-en-windows.md`, `docs/verification.md` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica el cuerpo de este PRD sigue en plantilla sin completar y es del USUARIO. La #59 no cambia que se construye: agrega evidencia de runtime en Windows para un instalador que ya existia.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:101 (spec `decision`), docs/prd/SDD-master.md:101 (spec `decisiones`) y 102 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.DS_Store`, `.github/workflows/windows-cmd-installer.yml`, `docs/estado-feature-59-cmd-smoke-real-en-windows.md`, `docs/verification.md` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
- **Entornos**: solo local. El arnes no se despliega; se instala en el repo del
  proyecto con `setup_harness.sh` / `.ps1`, y `tests/parity_check.sh` verifica
  que los dos hagan lo mismo.
Despues:
- **Entornos**: local y un runner Windows en CI (feature #59). El arnes no se
  despliega; se instala en el repo del proyecto con `setup_harness.sh` / `.ps1`,
  y `tests/parity_check.sh` verifica que los dos DECLAREN lo mismo. Lo que la
  paridad no puede probar desde macOS o Linux —que `setup_harness.cmd` de verdad
  arranque el `.ps1` y le traduzca los flags— lo ejecuta
  `.github/workflows/windows-cmd-installer.yml` en `windows-latest`, y
  `tests/cmd_installer_check.ps1` se NIEGA a correr fuera de Windows en vez de
  informar un skip verde.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:105 (spec `devuelve`), docs/architecture.md:105 (spec `fuente`), docs/architecture.md:108 (spec `entrada`) y 241 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.DS_Store`, `.github/workflows/windows-cmd-installer.yml`, `docs/estado-feature-59-cmd-smoke-real-en-windows.md`, `docs/verification.md` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica architecture.md mapea el codigo del arnes (los modulos de rust/src, las superficies y los documentos que instala). El workflow de CI es infraestructura del repositorio fuente, no una pieza del producto que se instala en los proyectos: donde SI corresponde declararlo es en la estrategia de verificacion del SDD, que es lo que este mismo diff actualiza.

