Aplicado: 2026-08-18T22:18:48Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #28: consolidacion_de_lecciones_con_llm

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 28`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/aprendizaje/PRD-aprendizaje.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/aprendizaje/PRD-aprendizaje.md (no menciona 'consolidacion_de_lecciones_con_llm')
Veredicto: cambio
Antes:
| 6 | Mapa de aprendizaje | mapa_de_aprendizaje | O4 | `journey` dibuja la linea de tiempo sobre datos ya existentes y permite podar con `list/delete/edit` | done (2026-08-17) |
Despues:
| 6 | Mapa de aprendizaje | mapa_de_aprendizaje | O4 | `journey` dibuja la linea de tiempo sobre datos ya existentes y permite podar con `list/delete/edit` | done (2026-08-17) |
| 7 | Consolidacion de lecciones asistida por LLM | consolidacion_de_lecciones_con_llm | O4 | `lecciones consolidar` detecta solapamientos viendo solo nombre, descripcion y triggers (NUNCA el cuerpo) e informa; con `--aplicar` fusiona bajo un paraguas tomando la fusion de argv y archiva las miembros con backup y rollback. Apagada por default; cadena override -> CLI -> skip limpio | pendiente |

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'consolidacion_de_lecciones_con_llm')
Veredicto: no-aplica esta feature es el hito 7 del PRD anidado de aprendizaje, no del maestro; el maestro ya remite a ese PRD hijo para todo el programa de aprendizaje

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'consolidacion_de_lecciones_con_llm')
Veredicto: cambio
Antes:
## 4. Decisiones tecnicas
Despues:
## 4. Decisiones tecnicas

**Como habla el arnes con un LLM** (feature #28, la unica parte que usa modelo).
La cadena es `HARNESS_CONSOLIDAR_CMD` -> primer CLI de una tabla corta
(`claude -p`, `kimi -p`) -> **skip limpio**. Apagada por default y de forma
estructural: sin `rules.consolidar_backend` no se resuelve backend ni se mira el
entorno. Tres decisiones que valen para cualquier feature futura con modelo:

- **Al modelo se le manda lo minimo.** Ve nombre, descripcion y triggers; nunca
  el cuerpo de una leccion. Lo peor que puede hacer es equivocarse, no filtrar.
- **El prompt viaja como item de argv, jamas por `sh -c`.** Por eso NO se reusa
  `verificacion::ejecutar`, que si corre con shell.
- **El modelo propone; lo que muta sale de argv.** La mitad que escribe se
  verifica sin backend y de forma determinista.

El tramo HTTP con API key **no esta implementado** y el mensaje de skip lo dice.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'consolidacion_de_lecciones_con_llm')
Veredicto: cambio
Antes:
- `progress.rs`: `current.md` / `history.md` (estado vivo y bitacora).
Despues:
- `consolidacion.rs`: deteccion de lecciones solapadas con un LLM (feature #28),
  la UNICA parte del arnes que usa un modelo. `resolver_backend()` implementa
  override -> CLI -> skip limpio y es pura (el override llega por parametro, no
  del entorno). `detectar` no recibe `&HarnessPaths`: **no puede escribir aunque
  quiera**. Al modelo se le manda solo nombre, descripcion y triggers —nunca el
  cuerpo— y el prompt viaja como item de argv, jamas por `sh -c`, asi que una
  descripcion con backticks no puede inyectar nada. `revisar_paraguas()` exige
  que el paraguas herede todos los triggers de lo que archiva, porque `buscar`
  puntua una leccion activa 100 y una archivada 30.
- `progress.rs`: `current.md` / `history.md` (estado vivo y bitacora).

