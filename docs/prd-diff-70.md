Aplicado: 2026-09-04T20:01:24Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #70: El gate de citas del review no puede ver un repo hermano: una feature de backend no puede citar su codigo

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 70`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:103 (spec `guarda`) y 114 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/revision.rs`, `rust/src/revision.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica La tabla de hitos llega hasta el #13 y desde el #14 los cierres van a la Bitacora, que la escribe el propio `close`. La #70 no cambia una capacidad del producto ni la resolucion de citas: cambia el TEXTO de un mensaje de error para que se pueda actuar sobre el.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:10 (spec `ningun`) y 207 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/revision.rs`, `rust/src/revision.rs`. Revisa si este documento debe reflejarlo.

Veredicto: cambio

Antes:
- **La cobertura por AC** es la que aguanta: una fila por cada AC-n del spec,
  cada una citando `archivo:linea` **que resuelve** (el archivo existe y tiene
  esa linea), verificada al estampar Y de nuevo en el cierre. Eso sube el costo
  de fabricar un review falso de cinco segundos a leer el codigo. No lo vuelve
  imposible: lo que el arnes NO comprueba es que la cita sea PERTINENTE al AC.

Despues:
- **La cobertura por AC** es la que aguanta: una fila por cada AC-n del spec,
  cada una citando `archivo:linea` **que resuelve** (el archivo existe y tiene
  esa linea), verificada al estampar Y de nuevo en el cierre. Eso sube el costo
  de fabricar un review falso de cinco segundos a leer el codigo. No lo vuelve
  imposible: lo que el arnes NO comprueba es que la cita sea PERTINENTE al AC.
- **Y cuando una cita no resuelve, el gate dice contra QUE resolvio** (feature
  #70). Antes contestaba lo mismo para dos casos distintos —"un archivo que exista
  y una linea que exista en el"— sin nombrar ninguna de sus raices, asi que un
  reviewer que citaba un repo hermano probaba `../repo/archivo:353` y la ruta
  absoluta, las dos se rechazaban por la FORMA, y el mensaje lo mandaba a buscar
  un archivo que estaba donde el creia. Lo que hacia entonces era citar el
  documento que el mismo habia escrito en la columna que el gate comprueba: el
  gate satisfecho con una cita que no era la evidencia. Ahora separa "la forma de
  la ruta no se acepta" de "no se encontro", lista las raices en orden, y ofrece
  la forma de citar un repo hermano **solo en los layouts donde resuelve** — un
  remedio que no funciona es peor que ninguno.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:100 (spec `directorio`), docs/architecture.md:104 (spec `defecto`) y 295 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/revision.rs`, `rust/src/revision.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica architecture.md ya describe `gate()` y el paquete de revision, y la #70 no agrega un modulo ni cambia que raices hay ni como se resuelve una cita: agrega dos funciones de diagnostico en `revision.rs` que arman el texto del rechazo. El mapa de lo que existe no cambia.

