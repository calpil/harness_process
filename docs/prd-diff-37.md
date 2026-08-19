Aplicado: 2026-08-18T23:58:54Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #37: estado_superseded

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 37`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'estado_superseded')
Veredicto: no-aplica es una correccion al vocabulario del backlog, no un hito del producto; el PRD maestro no promete nada sobre los estados de feature_list.json

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'estado_superseded')
Veredicto: cambio
Antes:
## 5. Datos
Despues:
## 5. Datos

**Los estados de una feature** (`feature_list.json`). Son cinco y significan
cosas distintas: `pending` (sin empezar, `next` la ofrece), `in_progress` (una
sola a la vez), `done` (hecha, con spec aprobado y su evidencia), `blocked`
(trabada por algo externo) y `superseded` (el trabajo se hizo en OTRA feature,
que se nombra en `superseded_by` y se valida al cerrar). Solo `done` pasa por los
cuatro gates de cierre; `superseded` no cuenta ni en el numerador ni en el
denominador de `prd tree`, porque no es trabajo hecho ni pendiente.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'estado_superseded')
Veredicto: cambio
Antes:
### Exit codes (estables para hooks)
Despues:
### Estados de una feature

`pending` / `in_progress` / `done` / `blocked` / `superseded`. El campo es un
`&str` y **no** un enum: catorce lugares lo comparan por igualdad contra un valor
concreto, lo que hace barato agregar uno nuevo (la #37 agrego `superseded` con un
cambio real de una linea) y a la vez significa que un valor invalido escrito a
mano solo lo detecta clap. `superseded` exige `superseded_by`, que se valida
contra el backlog al cerrar.

### Exit codes (estables para hooks)

