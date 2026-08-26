Aplicado: 2026-08-26T00:54:08Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #41: consolidar_usa_relacionadas

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 41`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:108 (spec `dispara`) y 141 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/leccion.rs`, `rust/src/consolidacion.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica El cambio es interno y la documentacion vigente ya cubre el alcance de este documento.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `ninguna`), docs/prd/SDD-master.md:101 (spec `decision`) y 101 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/leccion.rs`, `rust/src/consolidacion.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica El cambio es interno y la documentacion vigente ya cubre el alcance de este documento.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (módulo `leccion`), docs/architecture.md:100 (spec `leccion`), docs/architecture.md:100 (spec `lecciones`) y 236 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/leccion.rs`, `rust/src/consolidacion.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica El cambio es interno y la documentacion vigente ya cubre el alcance de este documento.

