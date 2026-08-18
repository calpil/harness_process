---
nombre: promesas-estructurales-vs-disciplina
descripcion: Si el invariante depende de acordarse, no es invariante: es una intencion.
triggers: [invariante, promesa, no escribe, dry-run, solo lectura, funcion pura, aplicar]
relacionadas: [criterios-de-cierre-que-se-pueden-fallar]
origen: [21]
usos: 1
ultimo_uso: 2026-08-17
ultima_actualizacion: 2026-08-17
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

Fijate el patron: casi todas se sostienen por **lo que el modulo NO importa**.
Un `use` que no esta es una garantia mucho mas fuerte que un comentario.

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
