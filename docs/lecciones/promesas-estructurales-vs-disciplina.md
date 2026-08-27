---
nombre: promesas-estructurales-vs-disciplina
descripcion: Si el invariante depende de acordarse, no es invariante: es una intencion.
triggers: [invariante, promesa, no escribe, dry-run, solo lectura, funcion pura, aplicar, trampa, advertencia, falso verde, arreglar a mano, clase de bug, viaja en el merge, dato compartido, pendiente, best-effort]
relacionadas: [criterios-de-cierre-que-se-pueden-fallar, probar-contra-datos-reales]
origen: [21, 44, 60]
usos: 2
ultimo_uso: 2026-08-24
ultima_actualizacion: 2026-08-27
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
