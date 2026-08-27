Aplicado: 2026-08-27T19:07:57Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #62: el_cierre_no_declara_hecho_lo_que_no_hizo

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 62`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:108 (spec `dispara`) y 191 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica el cuerpo de este PRD sigue en plantilla sin completar y es del USUARIO. Esta feature no cambia que se construye ni por que: cambia el ORDEN en que el cierre escribe lo que ya escribia. Su bitacora la deja el propio cierre.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `ninguna`), docs/prd/SDD-master.md:100 (spec `atlassian`) y 169 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
  archivos: el arnes no elige entre su merge y el trabajo ajeno.
Despues:
  archivos: el arnes no elige entre su merge y el trabajo ajeno.
- **Y nada del ESTADO se escribe hasta que la integracion ocurrio** (feature
  #62). El cierre corre en cuatro fases: lo que puede negarse (gates, `--to`,
  colisiones), los artefactos que tienen que viajar en la rama (la anotacion del
  plan y el estado archivado, idempotentes porque el merge borra el worktree
  donde viven), la integracion, y recien despues el estado (backlog, Atlassian,
  `progress/`, `history.md`, memorias y el mensaje de exito). No hay rollback a
  proposito: quedaria parcial —un intent emitido a Jira y una memoria escrita en
  el hub no se deshacen— y habria que acordarse de mantenerlo cada vez que el
  cierre gane un efecto nuevo. La regla que vale para cualquier comando futuro
  del arnes: **los efectos que no se pueden deshacer van ultimos**.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (spec `indice`), docs/architecture.md:102 (spec `leccion`), docs/architecture.md:103 (spec `transicion`) y 384 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:168-175

