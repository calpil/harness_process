Aplicado: 2026-08-22T13:07:11Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #51: revision_adversarial_y_modelos_por_rol

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 51`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'revision_adversarial_y_modelos_por_rol')
Veredicto: cambio
Antes:
| 8 | Features en paralelo sin pisarse | features_en_paralelo_con_worktrees | <O1> | `start` deja de rechazar la segunda feature activa y le da a cada una su rama GitFlow (`feature/<id>-<slug>`, `bugfix/` si es `kind: bug`) y su worktree hermano; el estado del arnes sigue siendo unico (repo principal) y el vivo se parte en `current-<id>.md` con `current.md` como indice; dentro del worktree los comandos infieren la feature; `close --status done` exige `--to <rama>`, mergea, publica, borra el worktree y conserva la rama, y un conflicto aborta sin dejar nada a medias | done (2026-08-22) |
Despues:
| 8 | Features en paralelo sin pisarse | features_en_paralelo_con_worktrees | <O1> | `start` deja de rechazar la segunda feature activa y le da a cada una su rama GitFlow (`feature/<id>-<slug>`, `bugfix/` si es `kind: bug`) y su worktree hermano; el estado del arnes sigue siendo unico (repo principal) y el vivo se parte en `current-<id>.md` con `current.md` como indice; dentro del worktree los comandos infieren la feature; `close --status done` exige `--to <rama>`, mergea, publica, borra el worktree y conserva la rama, y un conflicto aborta sin dejar nada a medias | done (2026-08-22) |
| 9 | Revisar en serio sin que cueste una fortuna | revision_adversarial_y_modelos_por_rol | <O1> | Un modelo por rol de Claude (implementer `claude-opus-5`, lider y reviewer `claude-fable-5`, los tres `xhigh`) definido en la tabla de roles de los dos instaladores y tuneable por variable; el reviewer intenta REFUTAR cada AC y verifica por su cuenta lo que la evidencia declara verde; y `revision --feature <id>` arma el paquete minimo (AC + estado de verify + evidencia + archivos + diff + rutas protegidas) acotado por presupuesto, que declara lo que recorta y reporta su propio tamaño | pendiente |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'revision_adversarial_y_modelos_por_rol')
Veredicto: cambio
Antes:
**Aislamiento entre features** (feature #47). Dos implementaciones simultaneas
Despues:
**Que revisar no cueste una fortuna** (feature #51). Verificar lo implementado
llego a costar 10 millones de tokens, casi todos gastados explorando el repo.
Dos decisiones que valen para cualquier feature futura que involucre a un
agente revisando:

- **El material se entrega, no se busca.** `revision --feature <id>` arma el
  paquete (AC + estado de verify + evidencia + archivos + diff + rutas
  protegidas) acotado por presupuesto, declara lo que recorta y reporta su
  propio tamaño antes de que alguien lo lea.
- **Un modelo por rol, en la tabla de roles de cada instalador.** El que escribe
  codigo piensa con Opus; el que planifica y el que revisa, con Fable; los tres
  en `xhigh`. `.claude/agents/*.md` es artefacto generado: editarlo a mano no
  sobrevive a la instalacion.

**Aislamiento entre features** (feature #47). Dos implementaciones simultaneas

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'revision_adversarial_y_modelos_por_rol')
Veredicto: cambio
Antes:
## Features en paralelo (feature #47)
Despues:
## Paquete de revision (feature #51)

`rust/src/revision.rs` + `harness revision --feature <id> [--max-lineas N]
[--json]`: junta los AC del spec con su estado en `verify-<id>.md`, las filas de
evidencia de `impl-<id>.md`, los archivos tocados por la feature (incluido lo
sin commitear y lo no indexado, marcado aparte), el diff acotado y las rutas
protegidas tocadas. Es de SOLO LECTURA, declara lo que recorta y reporta su
tamaño en lineas y tokens estimados.

El modelo y el esfuerzo de los subagentes de Claude salen de la tabla de roles
de cada instalador (`CLAUDE_MODEL_*` en `setup_harness.sh`, `$claudeModels` en
`setup_harness.ps1`), no del espejo `.claude/agents/*.md`, que es generado.

## Features en paralelo (feature #47)

