---
nombre: promesas-estructurales-vs-disciplina
descripcion: Si el invariante depende de acordarse, no es invariante: es una intencion.
triggers: [invariante, promesa, no escribe, dry-run, solo lectura, funcion pura, aplicar, trampa, advertencia, falso verde, arreglar a mano, clase de bug, viaja en el merge, dato compartido, pendiente, best-effort, excepcion, salvo, no se puede, limitacion, orden, rollback, deshacer, transaccional, efecto irreversible, aislamiento, worktree, paralelo, atribuir, declarar, estado prematuro]
relacionadas: [criterios-de-cierre-que-se-pueden-fallar, probar-contra-datos-reales]
origen: [21, 44, 60, 61, 62, 72]
usos: 4
ultimo_uso: 2026-09-05
ultima_actualizacion: 2026-09-05
estado: activa
---

## Cuando aplica

Cuando un spec promete un invariante del tipo "esto **no** hace X":

- "el modo informe no toca ningun archivo"
- "nunca borra"
- "no escribe fuera de `progress/`"
- "no depende del hub"
- "el contrato va a stderr y no cambia el exit code"

Sintoma de que lo estas resolviendo mal: la promesa se sostiene porque *vos te
acordas* de no llamar a la funcion que muta. Eso funciona hasta el primer refactor
que la llame por comodidad.

## Procedimiento

1. **Parti la operacion en dos**: la parte que decide (lee, calcula) y la parte
   que actua (escribe, mueve, borra). La que decide es una **funcion pura** que
   devuelve un plan; la que actua toma ese plan.
2. Poné la parte que actua **detras de una barrera explicita**: un flag
   (`--aplicar`), un parametro que hay que pasar, un tipo distinto.
3. El camino por defecto usa **solo** la parte que decide. Asi, "no toca nada" no
   es una regla que hay que recordar: es que ahi no hay codigo que toque nada.
4. Recien entonces escribi el test. El test **confirma** la propiedad; no es lo
   que la sostiene.

Ejemplos de este repo:

| Promesa | Que la sostiene |
| --- | --- |
| "el informe no toca nada" (#21) | `planificar()` solo lee; `aplicar()` esta detras de `--aplicar` |
| "el nudge no escribe artefactos" (#18) | el modulo no importa `Leccion`: no tiene con que escribir una |
| "buscar no depende del hub" (#20) | el modulo no importa `graph`: no tiene con que consultarlo |
| "nunca borra" (#21) | no existe ninguna llamada a `remove_*` fuera del `move` de archivar |
| "no escribe un puntero roto" (#60) | `decidir_vuelta` recibe `Candidato { existe }` ya resuelto: no tiene con que mirar el disco, y la que escribe solo ejecuta un plan validado |

Fijate el patron: casi todas se sostienen por **lo que el modulo NO importa**.
Un `use` que no esta es una garantia mucho mas fuerte que un comentario.

## La otra cara: promesas sobre lo que SI va a pasar

El mismo error tiene una variante que cuesta mas ver, porque la promesa es
positiva. Dos formas concretas, las dos de la feature #60:

**1. El dato compartido guardado dentro de la unidad aislada.** La #54 prometio
que los documentos escritos en el worktree "viajan en el merge sin pasos de copia
especiales". Vale para el spec y la evidencia, que son de UNA feature. No vale
para la bitacora del PRD, que es de TODAS: cada cierre en paralelo apendeaba al
final de la misma seccion desde una rama distinta, el merge conflictuaba y la
linea se perdia en la resolucion. **7 de 18 cierres**, sin que nadie se enterara.

La pregunta que lo detecta: *¿este dato es de la unidad de trabajo o de todas?*
Si es de todas, guardarlo adentro de una hace que su supervivencia dependa de
como alguien resuelva un conflicto. Eso es disciplina, no estructura. El arreglo
no fue detectar el conflicto: fue sacar el dato del branch, y entonces no hay
conflicto que resolver mal.

**2. El pendiente que hay que acordarse de anotar.** Cuando algo best-effort
falla, la tentacion es escribir el pendiente en un archivo. Pero eso hereda el
problema: si el paso que falla es el mismo que tiene que anotar, no hay
pendiente. Preguntate si el pendiente se puede **derivar del estado que ya
existe**: una feature `done` que no esta en la bitacora de su PRD ES el
pendiente, lo haya anotado alguien o no — y por eso `prd doctor` encontro los 13
cierres que se perdieron antes de que el mecanismo existiera.

Regla corta: **un `[i]` no es un pendiente**. Un pendiente es algo que se puede
volver a consultar despues de que la salida se fue del scroll.

**3. La excepcion justificada en un limite que nadie volvio a medir.** A veces
la promesa no se cae por olvido: se cae por un `if` que alguien escribio a
proposito, con su comentario y todo. La cabecera de `git.rs` prometia que "el
merge corre en un worktree temporal (no toca tu checkout)"; el codigo tenia:

```rust
// git no permite dos worktrees sobre la misma rama
if rama_actual(principal) == Some(destino) {
    return merge_aqui(principal, ...);   // <-- justo el caso mas comun
}
```

El comentario era **cierto** y aun asi la excepcion era **evitable**: git no
deja dos worktrees sobre la misma rama, pero si deja uno en HEAD detached sobre
su commit. Una linea (`--detach`) borro el caso especial entero.

Como se detecta: buscar en el codigo las palabras que marcan una excepcion
—`salvo`, `si no se puede`, `git no permite`, `la API no deja`— y preguntar
*¿cuando se midio esto?*. Un limite que se acepto sin volver a comprobar suele
tapar el caso mas frecuente, porque las excepciones se escriben cuando algo
falla, y lo que falla primero es lo que mas se usa. Si el limite es real, la
promesa de la cabecera tiene que decirlo; una promesa con una excepcion muda es
peor que no prometer nada.

## El ORDEN tambien es estructura

La forma mas barata de no necesitar un rollback es no haber escrito nada
todavia. Si una operacion tiene varios efectos y alguno puede fallar, el orden
en que los hacer NO es un detalle de implementacion: es lo que decide si el
sistema puede mentir.

`close` escribia nueve cosas —backlog en `done`, transicion a Jira, anotacion
del plan, estado archivado, indice, `history.md`, memoria en el hub, borrado del
estado vivo y "Feature #N cerrada"— y **despues** integraba. Cuando la
integracion fallaba, las nueve ya habian pasado sobre un trabajo que no estaba
integrado.

Procedimiento:

1. **Clasifica los efectos por reversibilidad.** Escribir un JSON se revierte;
   emitir un evento a un sistema externo, escribir en una base compartida o
   imprimir una linea en la terminal, no.
2. **Ordena: lo reversible y lo que puede negarse primero, lo irreversible al
   final.** En `close` quedo asi: (0) lo que puede negarse, (1) lo que tiene que
   viajar en la rama, (2) la operacion que puede fallar, (3) todo el estado.
3. **Lo que no se puede mover, hacelo idempotente.** Dos artefactos del cierre
   tenian que escribirse antes por una razon fisica —viven en el worktree que el
   merge borra— asi que se hicieron re-ejecutables sin duplicar.
4. **No agregues rollback**: seria parcial (los efectos del punto 1 que no se
   deshacen siguen sin deshacerse) y habria que acordarse de mantenerlo cada vez
   que la operacion gane un efecto nuevo. Es disciplina otra vez.

Regla corta: **los efectos que no se pueden deshacer van ultimos**. Y el mensaje
de exito es uno de ellos: una vez que lo leyeron, ya no se puede desdecir.

### La variante que volvio dos veces: DECLARAR antes de conseguir (feature #72)

La #62 ordeno los efectos de `close` por reversibilidad. La #72 encontro la
misma forma otras dos veces, y en las dos el problema no era el rollback: era que
el sistema **afirmaba un hecho antes de asegurarlo**.

| Donde | Que afirmaba | Cuando era cierto |
| --- | --- | --- |
| `start` | `status: in_progress` + `worktree: <ruta>` | recien despues de que `git worktree add` funcionara |
| `close` | "commits que se llevan: (ninguno)" | recien despues de commitear el worktree de la feature |

En `start` el costo fue medible: tres features (`#98`, `#122`, `#126`) quedaron
`in_progress` sin rama ni worktree, escribiendo las tres en el mismo checkout,
porque el estado se escribia primero y el fallo de git se imprimia con un `[i]`.
En `close`, el rango se calculaba antes del commit, asi que el cierre anunciaba
un rango vacio y a la linea siguiente commiteaba y mergeaba.

Los dos son el mismo bug con distinto disfraz, y ninguno es un problema de
reversibilidad: el JSON se podia reescribir, la linea impresa no. La pregunta que
los detecta no es "¿esto se puede deshacer?" sino:

> **¿Lo que estoy por escribir o imprimir ya es cierto en este punto del codigo?**

Procedimiento, ademas del de arriba:

1. Para cada afirmacion que el codigo emite —un campo de estado, una linea de
   consola, un evento— buscá **la linea exacta** donde eso pasa a ser cierto.
2. Si la afirmacion esta antes, moverla despues. No agregues una correccion
   posterior ("en realidad eran 2 commits"): nadie lee la segunda linea.
3. Si no se puede mover porque el dato se necesita antes, **recalcula y volve a
   preguntar** justo antes de actuar. En `close` quedo: commit -> rango
   definitivo -> re-chequeo de ajenos -> merge. El primer chequeo no se saco;
   se le agrego el segundo, sobre el dato ya definitivo.

Y el sintoma que lo delata en una revision: un `println!` con un `[i]` seguido de
codigo que sigue como si nada. **Un `[i]` antes de un `continue` implicito casi
siempre es una promesa que se acaba de romper en silencio** — es la misma familia
que "un `[i]` no es un pendiente", un nivel mas arriba.

## El mismo principio, aplicado a ARREGLAR un bug (feature #44)

No es solo para invariantes. Cuando encontras una trampa —una forma de que la
herramienta mienta— tenes dos maneras de cerrarla:

| Que haces | Que consegus |
| --- | --- |
| arreglas las instancias que ves y escribis una advertencia | documentaste |
| escribis un chequeo que la detecta sola | la cerraste |

El caso medido: la feature #23 descubrio que `cargo test <nombre-inexistente>`
imprime `running 0 tests`, dice `ok` y **sale 0**, asi que un AC quedaba verde
sin ejecutar nada. Lo arreglo **renombrando los tests a mano** y dejo escrita la
advertencia en `UPDATING.md`.

Cinco features despues volvio a pasar: el AC-12 de la #28 declaraba un test que
no existia, y el invariante mas citado de ese comando quedo registrado como
verificado con nada detras. Nadie lo vio hasta que un pase de refutacion lo
busco a proposito, **un dia** despues de cerrar.

La advertencia estaba escrita. La lei y la escribi yo. No sirvio, porque una
advertencia solo actua cuando alguien se acuerda de ella en el momento exacto en
que esta por caer.

La #44 la cerro estructuralmente: `verify` mira la salida ademas del exit code y
marca `vacio` al AC que no ejecuto ningun caso. Ahora la trampa no depende de
que nadie se distraiga.

**La pregunta que hay que hacerse al arreglar cualquier bug**: ¿esto arregla el
caso o la clase? Si la respuesta es "el caso, mas una nota para acordarse", vas a
volver a verlo. Anotalo en el backlog aunque no lo hagas ahora: la nota en el
backlog al menos tiene fecha de vencimiento; la advertencia en un documento, no.

## Pitfalls

- **Confundir el test con la garantia.** Un test que compara mtimes antes y
  despues es evidencia de que hoy no escribe; no impide que manana alguien agregue
  la escritura y ajuste el test. La estructura si lo impide.
- **La barrera que se puede saltear por comodidad.** Si `aplicar()` es publica y
  esta a mano, alguien la va a llamar desde el camino por defecto "para
  simplificar". Manteneka detras del flag y con el comentario que dice por que.
- **Prometer en la prosa lo que el codigo no puede sostener.** Si el invariante no
  se puede hacer estructural (por ejemplo "el mensaje es claro"), no lo escribas
  como promesa: escribilo como criterio de cierre verificable
  ([[criterios-de-cierre-que-se-pueden-fallar]]).
- **Creer que documentar una trampa la desactiva.** Es el pitfall que costo la
  #44: la advertencia de la #23 estaba escrita, era correcta y era clara, y aun
  asi el mismo error volvio cinco features despues. Lo unico que corta la clase
  es algo que corre solo.
- **Olvidar que la barrera final a veces es social.** `--aplicar` protege del
  accidente, no de la decision apurada. Cuando el comando mueve cosas del usuario,
  el rol tiene que decir explicitamente que no se corre sin avisarle.

## Verificacion

```bash
# La garantia mas fuerte es un `use` que no existe:
grep -n "^use crate::" rust/src/<modulo>.rs

# Y despues, el test que lo confirma (mtimes, exit codes, stdout identico):
cargo test <modulo>
```

Si al leer los `use` del modulo se puede decir "esto no tiene con que romper la
promesa", la promesa es estructural. Si hace falta leer el cuerpo entero para
convencerse, todavia depende de disciplina.
