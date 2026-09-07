Aplicado: 2026-09-07T02:47:32Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #74: add no protege contra la feature duplicada, y no esta medido que haga falta

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 74`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:103 (spec `guarda`), docs/prd/PRD-master.md:11 (spec `instalador`) y 244 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `rust/src/cli.rs`, `rust/src/commands/add.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 22 | El marcador stale de graphify deja de bloquear el Stop | graphify_stale_no_bloquea | <O1> | `graphify-out/.graphify_stale` lo deja el propio arnes (el hook post-commit tras cada commit que toca un `.md`, o `autocheck` cuando `graphify update` falla) y solo lo limpia el propio arnes cuando logra el rebuild semantico, que esta debounced a 30 minutos y salta sin backend LLM. `harness_check.sh` lo contaba como fallo y el Stop hook bloqueaba al agente hasta un `/graphify --update` a mano. Ahora se avisa con `[i]` —quien lo limpia solo, como forzarlo, y que no bloquea— y el check no falla por el; `templates/harness_check.sh` identico y un test en fixture instalado enganchado al smoke. Tercera vez que un enriquecimiento best-effort se cuela como gate (#18 nudge, #80 avisos): lo que no impide trabajar se avisa. Disparador: captura del usuario del Stop hook de realestate, 2026-09-06 22:40 | done (2026-09-07) |
Despues:
| 22 | El marcador stale de graphify deja de bloquear el Stop | graphify_stale_no_bloquea | <O1> | `graphify-out/.graphify_stale` lo deja el propio arnes (el hook post-commit tras cada commit que toca un `.md`, o `autocheck` cuando `graphify update` falla) y solo lo limpia el propio arnes cuando logra el rebuild semantico, que esta debounced a 30 minutos y salta sin backend LLM. `harness_check.sh` lo contaba como fallo y el Stop hook bloqueaba al agente hasta un `/graphify --update` a mano. Ahora se avisa con `[i]` —quien lo limpia solo, como forzarlo, y que no bloquea— y el check no falla por el; `templates/harness_check.sh` identico y un test en fixture instalado enganchado al smoke. Tercera vez que un enriquecimiento best-effort se cuela como gate (#18 nudge, #80 avisos): lo que no impide trabajar se avisa. Disparador: captura del usuario del Stop hook de realestate, 2026-09-06 22:40 | done (2026-09-07) |
| 23 | `add` no carga dos veces la misma feature | add_no_duplica | <O1> | Reabierta por decision del usuario tras cerrarse `blocked` (medido: 83 features, cero nombres repetidos y cero parecidos; la defensa es para el escenario que no deja huella: dos sesiones con el mismo hallazgo, un `add` re-corrido, un script que se relanza). `add` compara el nombre normalizado (minusculas, sin acentos ni puntuacion, sin palabras vacias) con el backlog ANTES de escribir: igual a una feature abierta (`pending`, `in_progress`, `blocked`) se niega con exit 2 nombrandola, sin flag de escape; igual a una cerrada avisa `[i]` y la crea (una regresion es legitima). `--clave <k>` es el idempotency-key de Hermes: con la misma clave devuelve la feature existente sin escribir nada. Modulo puro `duplicados.rs`, campo opcional `clave`, nada existente se toca; sin aviso por parecidos (medido cero pares). Disparador: idea 7 del catalogo de Hermes, y el usuario pidiendo implementarla con los skills de Rust cargados | done (2026-09-07) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:10 (spec `ningun`), docs/prd/SDD-master.md:10 (spec `ninguna`), docs/prd/SDD-master.md:101 (spec `cuando`) y 332 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `rust/src/cli.rs`, `rust/src/commands/add.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D9 | Las lecciones de clase tienen memoria ACOTADA como el perfil: tope de lineas duro, el detalle por feature en `<clase>/referencias/`, y la misma clase K veces seguidas exige motivo | (a) `--leccion-motivo` como escape del tope; (b) fusionar las lecciones solapadas bajo un paraguas, como proponia `consolidar`; (c) partir en clases nuevas | Un escape por flag se vuelve el default (paso con `ninguna`, que por eso exige motivo), y el limite del perfil es duro y funciona. Fusionar iba contra el tope: el paraguas naceria con ~1000 lineas, y los tres candidatos con confianza 1.00 se explicaban por declararse `relacionadas` entre si, no por ensenar lo mismo. Clases nuevas es lo que la guia desaconseja salvo que ninguna cubra el tema. La guia ya tenia la respuesta (paso 3, `referencias/`) con cero usos: el gate que media la declaracion no empujaba a usarla; un limite fisico si | 2026-09-06 |
Despues:
| D9 | Las lecciones de clase tienen memoria ACOTADA como el perfil: tope de lineas duro, el detalle por feature en `<clase>/referencias/`, y la misma clase K veces seguidas exige motivo | (a) `--leccion-motivo` como escape del tope; (b) fusionar las lecciones solapadas bajo un paraguas, como proponia `consolidar`; (c) partir en clases nuevas | Un escape por flag se vuelve el default (paso con `ninguna`, que por eso exige motivo), y el limite del perfil es duro y funciona. Fusionar iba contra el tope: el paraguas naceria con ~1000 lineas, y los tres candidatos con confianza 1.00 se explicaban por declararse `relacionadas` entre si, no por ensenar lo mismo. Clases nuevas es lo que la guia desaconseja salvo que ninguna cubra el tema. La guia ya tenia la respuesta (paso 3, `referencias/`) con cero usos: el gate que media la declaracion no empujaba a usarla; un limite fisico si | 2026-09-06 |
| D10 | El duplicado de `add` se decide por nombre NORMALIZADO identico contra las features abiertas (`blocked` incluida), se rechaza sin escape, y la idempotencia para scripts va por una clave opaca (`--clave`) | (a) dedupe con aviso, como proponia el analisis de Hermes; (b) aviso por nombres parecidos con umbral; (c) `--forzar` para saltear el rechazo; (d) `blocked` como cerrada | Un aviso que no bloquea es lo que ya pasaba con `ninguna` antes de exigirle motivo, y un escape por flag se vuelve el default (#80, OBS-4). El umbral de parecido no tiene con que calibrarse: cero pares en el backlog real, y un umbral sin evidencia es un numero inventado (#28). `blocked` sigue siendo la feature: cargar otra igual es esconder el bloqueo. La salida legitima para "es otra cosa" es un nombre que lo diga; la de los scripts, la clave, que ademas deja el reintento en exit 0 sin bitacora ni intent | 2026-09-07 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (spec `aplica`), docs/architecture.md:101 (spec `bitacora`), docs/architecture.md:101 (spec `idempotente`) y 556 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `rust/src/cli.rs`, `rust/src/commands/add.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
- `dependencias.rs`: que feature espera a cual, y que feature se traba siempre
  (feature #75). Dos preguntas que comparten archivo porque comparten el dato
Despues:
- `duplicados.rs`: `add` no carga dos veces la misma feature (feature #74). Puro:
  `normalizar` (minusculas, sin acentos, solo `[a-z0-9]`, sin las 21 palabras
  vacias de `VACIAS`), `buscar` -> `Coincidencia` {`Abierta` gana sobre `Cerrada`
  sobre `Ninguna`; `ABIERTAS` = pending/in_progress/blocked} y `por_clave` (la
  clave opaca de `--clave`). `add` decide antes de escribir: abierta rechaza
  (exit 2, sin escape), cerrada avisa `[i]`, clave repetida devuelve la existente
  (exit 0, sin bitacora ni intent). Lo opcional del alta viaja en `AltaOpts`.
- `dependencias.rs`: que feature espera a cual, y que feature se traba siempre
  (feature #75). Dos preguntas que comparten archivo porque comparten el dato

