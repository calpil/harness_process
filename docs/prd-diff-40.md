Aplicado: 2026-08-26T00:54:08Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #40: prd_sello_se_invalida_al_editar

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 40`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:108 (spec `dispara`) y 133 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/prd.rs`, `rust/src/documentos.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica El cambio es interno y la documentacion vigente ya cubre el alcance de este documento.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:101 (spec `decision`), docs/prd/SDD-master.md:101 (spec `decisiones`) y 115 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/prd.rs`, `rust/src/documentos.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica El cambio es interno y la documentacion vigente ya cubre el alcance de este documento.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:101 (spec `restaurar`), docs/architecture.md:105 (spec `vuelve`), docs/architecture.md:106 (spec `estructura`) y 223 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/prd.rs`, `rust/src/documentos.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica El cambio es interno y la documentacion vigente ya cubre el alcance de este documento.
