---
nombre: promesas-estructurales-vs-disciplina
descripcion: Si el invariante depende de acordarse, no es invariante: es una intencion.
triggers: [invariante, promesa, no escribe, dry-run, solo lectura, funcion pura, aplicar, trampa, advertencia, arreglar a mano, clase de bug, viaja en el merge, dato compartido, pendiente, best-effort, excepcion, salvo, no se puede, limitacion, orden, rollback, deshacer, transaccional, efecto irreversible, aislamiento, worktree, paralelo, atribuir, declarar, estado prematuro, approve-spec, feature activa, destino implicito, default implicito, sesiones concurrentes, one_feature_at_a_time, autorizacion, sello, aprobar]
relacionadas: [criterios-de-cierre-que-se-pueden-fallar, probar-contra-datos-reales]
origen: [21, 44, 60, 61, 62, 71, 72]
usos: 6
ultimo_uso: 2026-09-06
ultima_actualizacion: 2026-09-06
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

La feature #71 encontro la version dura del mismo caso, y conviene tenerla
aparte porque la pregunta de arriba no alcanza. El `close` escribia el sello de
cierre —`docs/estado-feature-<id>-<slug>.md`, que lleva adentro el cuerpo de
`progress/current-<id>.md` y es su UNICA copia porque `progress/` esta
gitignorado— en el `docs/` de la feature, y despues borraba ese worktree. No
habia conflicto que resolver mal: el archivo simplemente dejaba de existir.

Sobrevivia por una coincidencia: en un repo donde `docs/` es parte del repo
principal, el merge se lo llevaba antes del borrado. En un proyecto donde
`docs/` es un repo APARTE —el de realestate— no viajaba, y se perdio de verdad
(la #124: hubo que reconstruirlo a mano, y el cuerpo literal es irrecuperable).
Los 63 sellos que si sobrevivieron alimentaban la impresion de que el mecanismo
andaba.

Segunda pregunta, entonces, para cuando la primera dice "es de la unidad":
*¿algo del mismo comando destruye el lugar donde lo escribo?* Si la respuesta es
si, el dato no vive ahi por mas que conceptualmente le pertenezca a la unidad.
Y una vez movido a la raiz, la escritura pudo bajar a la fase del estado: estaba
en la fase de "los artefactos que viajan en la rama" por una razon fisica que
dejo de existir. **Cuando arreglas donde se escribe algo, volve a mirar cuando
se escribe: la fase muchas veces era una consecuencia del lugar.**

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

## Referencias (el detalle, caso por caso)

Movidas a `promesas-estructurales-vs-disciplina/referencias/` el 2026-09-06 (feature #80). Cada una es el caso que sostiene una regla de arriba: se leen cuando hace falta el detalle, no para entender la clase.

- [El ORDEN tambien es estructura](promesas-estructurales-vs-disciplina/referencias/el-orden-tambien-es-estructura.md)
- [El mismo principio, aplicado a ARREGLAR un bug (feature #44)](promesas-estructurales-vs-disciplina/referencias/el-mismo-principio-aplicado-a-arreglar-un-bug.md)
- [La variante del DESTINO IMPLICITO: un comando que apunta a estado global mutable](promesas-estructurales-vs-disciplina/referencias/la-variante-del-destino-implicito-un-comando-que-apunta-a-es.md)
