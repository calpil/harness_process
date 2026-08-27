Aplicado: 2026-08-27T19:40:43Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #63: el_arnes_no_afirma_lo_que_no_puede_comprobar

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 63`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:103 (spec `guarda`) y 182 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `docs/verification.md`, `rust/src/commands/close.rs`, `rust/tests/cli_basics.rs`, `tests/commit_guard_check.sh`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica el cuerpo de este PRD sigue en plantilla sin completar y es del USUARIO. Esta feature no cambia que se construye: arregla un test que no medía y un mensaje que nombraba una ruta borrada.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:10 (spec `ningun`) y 207 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `docs/verification.md`, `rust/src/commands/close.rs`, `rust/tests/cli_basics.rs`, `tests/commit_guard_check.sh`. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
- **Un AC que no midio nada no cuenta como verificado**: `cargo test <nombre>`
  con un filtro que no matchea sale 0, y eso ya produjo un falso verde real. Por
  eso `verify` mira la salida ademas del exit code y marca `vacio` al AC que
  reconocidamente no ejecuto ningun caso. Sobre salidas que no son de libtest no
  opina: el estado no cambia.
Despues:
- **Un AC que no midio nada no cuenta como verificado**: `cargo test <nombre>`
  con un filtro que no matchea sale 0, y eso ya produjo un falso verde real. Por
  eso `verify` mira la salida ademas del exit code y marca `vacio` al AC que
  reconocidamente no ejecuto ningun caso. Sobre salidas que no son de libtest no
  opina: el estado no cambia.
- **Y el andamiaje que no puede medir se pone ROJO, no verde** (feature #63).
  Un test que corta por tiempo con `timeout(1)` no mide nada en macOS —no viene
  con el sistema— y salia verde igual: el codigo 127 de "no existe" no era el
  124 de "se corto". La regla que queda: cuando una prueba depende de una
  herramienta externa, se elige entre varias (`timeout`, `gtimeout`,
  `perl alarm`), se PRUEBA el mecanismo elegido contra un caso que falla y uno
  que no, y si no hay ninguno se falla nombrando cual instalar. Un skip verde es
  la forma mas cara de no enterarse.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:102 (spec `leccion`), docs/architecture.md:104 (spec `ejecuta`) y 349 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `docs/verification.md`, `rust/src/commands/close.rs`, `rust/tests/cli_basics.rs`, `tests/commit_guard_check.sh`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica architecture.md mapea los modulos del arnes y sus responsabilidades; esta feature no agrega ni mueve ninguno. El andamiaje de pruebas lo cuentan docs/verification.md (ya actualizado con los seis modos y el porque) y la estrategia del SDD, que es lo que este mismo diff cambia.

