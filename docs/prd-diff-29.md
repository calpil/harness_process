Aplicado: 2026-08-18T13:09:10Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #29: prd_y_sdd_siempre_al_dia

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 29`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'prd_y_sdd_siempre_al_dia')
Veredicto: cambio
Antes:
| 5 | El catalogo de lecciones se lee bien con nombres largos | leccion_list_alineacion_dinamica | <O1> | `leccion list` calcula el ancho de la columna en vez de usar el 28 fijo; solo formato de salida, sin tocar orden, campos, `--json` ni exit codes | pendiente |
Despues:
| 5 | El catalogo de lecciones se lee bien con nombres largos | leccion_list_alineacion_dinamica | <O1> | `leccion list` calcula el ancho de la columna en vez de usar el 28 fijo; solo formato de salida, sin tocar orden, campos, `--json` ni exit codes | done (2026-08-18) |
| 6 | El PRD, el SDD y architecture.md dejan de poder quedar mintiendo | prd_y_sdd_siempre_al_dia | <O1> | Al cerrar, el arnes calcula el alcance (PRD de origen + padres + SDD + architecture.md), siembra una pregunta por documento en `docs/prd-diff-<id>.md`, y solo con el SI del usuario `prd apply --yes` lo escribe; `require_docs_al_dia` lo exige al cerrar | pendiente |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'prd_y_sdd_siempre_al_dia')
Veredicto: cambio
Antes:
# SDD Master - <nombre del proyecto>

Estado: draft
Ultima actualizacion: <YYYY-MM-DD>
Despues:
# SDD Master - Harness Process

Estado: en uso
Ultima actualizacion: 2026-08-18

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'prd_y_sdd_siempre_al_dia')
Veredicto: cambio
Antes:
- `progress.rs`: `current.md` / `history.md` (estado vivo y bitacora).
Despues:
- `doctor.rs`: diagnostico de la INSTALACION (feature #25). `diagnosticar()` es
  **pura** y devuelve un `Hallazgo` por area (`Estado::{Ok,Falla,Aviso,NoAplica}`);
  solo `Falla` cambia el exit code, asi que un hub caido no puede hacerlo mentir.
  En el checkout fuente del arnes, superficies y hooks dan `NoAplica`: su ausencia
  ahi es lo correcto.
- `rutas.rs`: rutas protegidas (feature #26). `esta_protegida()` es un matcher
  **puro** de globs (`*` un segmento, `**` cualquier profundidad) sobre
  `rules.rutas_protegidas`. Las escrituras del propio binario quedan exentas por
  un registro con mtime que **caduca** en cuanto alguien vuelve a tocar el
  archivo, y por eso `close` y `prd apply` pueden escribir el PRD sin dispararse
  la red de seguridad.
- `documentos.rs`: que el PRD, el SDD y `architecture.md` no queden mintiendo
  (feature #29). `alcance()` deriva los documentos del **arbol real** de PRDs;
  `parsear()` y `planificar()` son puras y devuelven el plan de escritura sin
  tocar disco. El anclaje es por **texto literal** y no por seccion, porque
  cortar en `## ` se tragaria las subsecciones `###`; y la idempotencia sale del
  CONTENIDO, no de una firma, porque un PRD lo comparten N features.
- `progress.rs`: `current.md` / `history.md` (estado vivo y bitacora).

