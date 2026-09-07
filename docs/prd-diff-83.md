Aplicado: 2026-09-07T01:46:03Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #83: el Stop hook bloquea por graphify-out/.graphify_stale, un marcador de enriquecimiento best-effort que el hook post-commit deja y solo borra si logra el rebuild semantico

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 83`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:11 (spec `instalado`), docs/prd/PRD-master.md:11 (spec `instalador`) y 166 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `harness_check.sh`, `templates/UPDATING.md`, `templates/harness_check.sh` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 21 | Lo aprendido tiene ciclo de vida: tope, racha y dos avisos | el_autoaprendizaje_tiene_ciclo_de_vida | <O1> | Medido el 2026-09-06 sobre este repo: 10 de los ultimos 15 cierres declararon la misma leccion (442 lineas, nueve secciones "(feature #N)"), el paso 3 de la guia (`<clase>/referencias/`) con cero usos, 340 decisiones sin incorporar al perfil y la consolidacion sin correr en 19 dias y sin registro; `require_leccion` media que se declare una leccion, no que se aprenda. Cuatro umbrales en `rules` (`0` apaga): `leccion_max_lineas` (250) —sobre el tope, `close --leccion` y `leccion usar` se niegan con el contrato de particion a `referencias/`, sin escape por flag—; `leccion_repeticiones` (3) —la misma clase K cierres seguidos exige `--leccion-motivo`, `ninguna` no cuenta—; `perfil_pendientes_max` (25) y `consolidar_cada_dias` (30) —el cierre avisa por stderr, y `consolidar`/`curar` registran su corrida en `history.md`—. `lecciones status` y `harness_check.sh` lo muestran. La biblioteca de este repo se puso dentro del tope (442 -> 186, 316 -> 167, doce referencias), sin fusionar las tres lecciones que el consolidador proponia, y el perfil recibio cuatro entradas aprobadas una por una. Disparador: la pregunta del usuario "¿quedo bien implementado el autoaprendizaje de Hermes?" y la medicion que la respondio | done (2026-09-06) |
Despues:
| 21 | Lo aprendido tiene ciclo de vida: tope, racha y dos avisos | el_autoaprendizaje_tiene_ciclo_de_vida | <O1> | Medido el 2026-09-06 sobre este repo: 10 de los ultimos 15 cierres declararon la misma leccion (442 lineas, nueve secciones "(feature #N)"), el paso 3 de la guia (`<clase>/referencias/`) con cero usos, 340 decisiones sin incorporar al perfil y la consolidacion sin correr en 19 dias y sin registro; `require_leccion` media que se declare una leccion, no que se aprenda. Cuatro umbrales en `rules` (`0` apaga): `leccion_max_lineas` (250) —sobre el tope, `close --leccion` y `leccion usar` se niegan con el contrato de particion a `referencias/`, sin escape por flag—; `leccion_repeticiones` (3) —la misma clase K cierres seguidos exige `--leccion-motivo`, `ninguna` no cuenta—; `perfil_pendientes_max` (25) y `consolidar_cada_dias` (30) —el cierre avisa por stderr, y `consolidar`/`curar` registran su corrida en `history.md`—. `lecciones status` y `harness_check.sh` lo muestran. La biblioteca de este repo se puso dentro del tope (442 -> 186, 316 -> 167, doce referencias), sin fusionar las tres lecciones que el consolidador proponia, y el perfil recibio cuatro entradas aprobadas una por una. Disparador: la pregunta del usuario "¿quedo bien implementado el autoaprendizaje de Hermes?" y la medicion que la respondio | done (2026-09-06) |
| 22 | El marcador stale de graphify deja de bloquear el Stop | graphify_stale_no_bloquea | <O1> | `graphify-out/.graphify_stale` lo deja el propio arnes (el hook post-commit tras cada commit que toca un `.md`, o `autocheck` cuando `graphify update` falla) y solo lo limpia el propio arnes cuando logra el rebuild semantico, que esta debounced a 30 minutos y salta sin backend LLM. `harness_check.sh` lo contaba como fallo y el Stop hook bloqueaba al agente hasta un `/graphify --update` a mano. Ahora se avisa con `[i]` —quien lo limpia solo, como forzarlo, y que no bloquea— y el check no falla por el; `templates/harness_check.sh` identico y un test en fixture instalado enganchado al smoke. Tercera vez que un enriquecimiento best-effort se cuela como gate (#18 nudge, #80 avisos): lo que no impide trabajar se avisa. Disparador: captura del usuario del Stop hook de realestate, 2026-09-06 22:40 | done (2026-09-07) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:10 (spec `deberia`), docs/prd/SDD-master.md:10 (spec `ningun`), docs/prd/SDD-master.md:101 (spec `cuando`) y 274 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `harness_check.sh`, `templates/UPDATING.md`, `templates/harness_check.sh` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica no hay decision de diseno nueva: es un cambio de severidad de un aviso del check que aplica el principio ya asentado en las #18 y #80 (lo best-effort avisa, no bloquea); el SDD no lista cada [!] de harness_check.sh

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (spec `aplica`), docs/architecture.md:101 (spec `cierre`), docs/architecture.md:105 (spec `leccion`) y 427 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `harness_check.sh`, `templates/UPDATING.md`, `templates/harness_check.sh` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica architecture.md no describe el marcador .graphify_stale ni la severidad de cada [!] de harness_check.sh; el hook post-commit y su rebuild semantico siguen iguales, y eso ya esta descrito en el bloque de graphify

