Aplicado: 2026-09-07T03:02:08Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #79: close refresca el espejo docs/bkp-backlog/feature_list.json al cerrar una feature

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 79`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:103 (spec `result`), docs/prd/PRD-master.md:11 (spec `instalador`) y 245 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 23 | `add` no carga dos veces la misma feature | add_no_duplica | <O1> | Reabierta por decision del usuario tras cerrarse `blocked` (medido: 83 features, cero nombres repetidos y cero parecidos; la defensa es para el escenario que no deja huella: dos sesiones con el mismo hallazgo, un `add` re-corrido, un script que se relanza). `add` compara el nombre normalizado (minusculas, sin acentos ni puntuacion, sin palabras vacias) con el backlog ANTES de escribir: igual a una feature abierta (`pending`, `in_progress`, `blocked`) se niega con exit 2 nombrandola, sin flag de escape; igual a una cerrada avisa `[i]` y la crea (una regresion es legitima). `--clave <k>` es el idempotency-key de Hermes: con la misma clave devuelve la feature existente sin escribir nada. Modulo puro `duplicados.rs`, campo opcional `clave`, nada existente se toca; sin aviso por parecidos (medido cero pares). Disparador: idea 7 del catalogo de Hermes, y el usuario pidiendo implementarla con los skills de Rust cargados | done (2026-09-07) |
Despues:
| 23 | `add` no carga dos veces la misma feature | add_no_duplica | <O1> | Reabierta por decision del usuario tras cerrarse `blocked` (medido: 83 features, cero nombres repetidos y cero parecidos; la defensa es para el escenario que no deja huella: dos sesiones con el mismo hallazgo, un `add` re-corrido, un script que se relanza). `add` compara el nombre normalizado (minusculas, sin acentos ni puntuacion, sin palabras vacias) con el backlog ANTES de escribir: igual a una feature abierta (`pending`, `in_progress`, `blocked`) se niega con exit 2 nombrandola, sin flag de escape; igual a una cerrada avisa `[i]` y la crea (una regresion es legitima). `--clave <k>` es el idempotency-key de Hermes: con la misma clave devuelve la feature existente sin escribir nada. Modulo puro `duplicados.rs`, campo opcional `clave`, nada existente se toca; sin aviso por parecidos (medido cero pares). Disparador: idea 7 del catalogo de Hermes, y el usuario pidiendo implementarla con los skills de Rust cargados | done (2026-09-07) |
| 24 | `close` refresca el espejo del backlog y de la bitacora | close_refresca_el_espejo | <O1> | `feature_list.json` y `progress/history.md` estan gitignorados y son lo unico que el instalador no regenera (#78). El 2026-09-06 el checkout se borro por error: el backlog volvio del espejo `docs/bkp-backlog/` refrescado a mano en el ultimo cierre; la bitacora no tenia espejo y se perdio. Ahora cada `close` (cualquier `--status`), despues de guardar el estado y de la linea de bitacora de ese cierre, deja `docs/bkp-backlog/feature_list.json` y `docs/bkp-backlog/history.md` byte-identicos en la raiz del repo principal (copia atomica) y lo dice; quedan sin commitear, como el sello. Politica en `rules.espejo_backlog`: ausente = solo si el directorio existe (el directorio es el opt-in), `true` = crea, `false` = no toca. Best-effort no mudo: si la copia falla, el cierre sigue y avisa `[!]`. Modulo puro `espejo.rs` (politica como enum, decision pura); sin refresco en `add`/`start`. Disparador: la perdida de la bitacora del 2026-09-06 y la decision del usuario de espejarla tambien | done (2026-09-07) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:101 (spec `cuando`), docs/prd/SDD-master.md:103 (spec `cuando`), docs/prd/SDD-master.md:104 (spec `commit`) y 321 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D10 | El duplicado de `add` se decide por nombre NORMALIZADO identico contra las features abiertas (`blocked` incluida), se rechaza sin escape, y la idempotencia para scripts va por una clave opaca (`--clave`) | (a) dedupe con aviso, como proponia el analisis de Hermes; (b) aviso por nombres parecidos con umbral; (c) `--forzar` para saltear el rechazo; (d) `blocked` como cerrada | Un aviso que no bloquea es lo que ya pasaba con `ninguna` antes de exigirle motivo, y un escape por flag se vuelve el default (#80, OBS-4). El umbral de parecido no tiene con que calibrarse: cero pares en el backlog real, y un umbral sin evidencia es un numero inventado (#28). `blocked` sigue siendo la feature: cargar otra igual es esconder el bloqueo. La salida legitima para "es otra cosa" es un nombre que lo diga; la de los scripts, la clave, que ademas deja el reintento en exit 0 sin bitacora ni intent | 2026-09-07 |
Despues:
| D10 | El duplicado de `add` se decide por nombre NORMALIZADO identico contra las features abiertas (`blocked` incluida), se rechaza sin escape, y la idempotencia para scripts va por una clave opaca (`--clave`) | (a) dedupe con aviso, como proponia el analisis de Hermes; (b) aviso por nombres parecidos con umbral; (c) `--forzar` para saltear el rechazo; (d) `blocked` como cerrada | Un aviso que no bloquea es lo que ya pasaba con `ninguna` antes de exigirle motivo, y un escape por flag se vuelve el default (#80, OBS-4). El umbral de parecido no tiene con que calibrarse: cero pares en el backlog real, y un umbral sin evidencia es un numero inventado (#28). `blocked` sigue siendo la feature: cargar otra igual es esconder el bloqueo. La salida legitima para "es otra cosa" es un nombre que lo diga; la de los scripts, la clave, que ademas deja el reintento en exit 0 sin bitacora ni intent | 2026-09-07 |
| D11 | El espejo del backlog y de la bitacora se refresca en `close`, en la raiz del repo principal, con el directorio `docs/bkp-backlog/` como opt-in (`Auto`), `rules.espejo_backlog` para forzar o apagar, y fallo que avisa sin impedir el cierre | (a) refrescar en cada comando (`add`, `start`); (b) crear el espejo siempre, sin opt-in; (c) que el fallo del espejo haga fallar el cierre; (d) commitearlo desde `close` | Refrescar en cada comando deja un archivo versionable modificado a cada rato en `git status` sin que nadie lo commitee; el cierre es el cambio de estado que importa. Crearlo sin pedirlo mete un documento versionable en un proyecto que no lo pidio (es del usuario, como el PRD). Un respaldo que impide cerrar es peor que ninguno, pero uno que falla en silencio no existe: `[!]` y sigue. Commitear desde `close` es tocar el repo del usuario desde codigo, la misma linea que #60 y #71 no cruzan | 2026-09-07 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (spec `incluye`), docs/architecture.md:103 (spec `fuente`), docs/architecture.md:103 (spec `verdad`) y 548 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:55-60

