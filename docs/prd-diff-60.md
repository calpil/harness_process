Aplicado: 2026-08-27T18:04:06Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #60: la_vuelta_al_prd_no_se_pierde_ni_miente

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 60`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (módulo `master`), docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`) y 232 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/promesas-estructurales-vs-disciplina.md` y 11 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica el cuerpo de este PRD (historia, objetivos, datos) sigue en plantilla sin completar y es del USUARIO: el arnes no lo escribe. Lo que esta feature toca del PRD es su BITACORA y la fila de su hito, que las escribe el propio cierre; y no hay hito declarado para la #60 porque nacio de dos bugs reportados aguas abajo, no de un hito del producto.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (módulo `master`), docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`) y 205 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/promesas-estructurales-vs-disciplina.md` y 11 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
- **Los docs se resuelven DESDE la feature, no desde el directorio actual.**
  `HarnessPaths::para_feature()` apunta `docs/` al worktree de esa feature, para
  que su spec, su plan y su evidencia viajen con el merge de su rama.
Despues:
- **Los docs se resuelven DESDE la feature, no desde el directorio actual.**
  `HarnessPaths::para_feature()` apunta `docs/` al worktree de esa feature, para
  que su spec, su plan y su evidencia viajen con el merge de su rama.
- **Salvo lo que es de TODAS las features** (feature #60). El PRD es un
  documento raiz y compartido: la vuelta al cierre (marcar el hito, dejar
  bitacora) se escribe en el `docs/prd/` del checkout PRINCIPAL y DESPUES de
  integrar. Guardar un log compartido dentro de una rama por feature hacia que
  dos cierres en paralelo apendearan al final de la misma seccion: el merge
  conflictuaba y la linea se perdia en la resolucion (7 de 18 cierres). La
  pregunta que decide donde va un documento es de quien es el dato, no desde
  donde se escribe.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:102 (módulo `lecciones`), docs/architecture.md:102 (spec `leccion`) y 495 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `docs/lecciones/promesas-estructurales-vs-disciplina.md` y 11 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:62-70

