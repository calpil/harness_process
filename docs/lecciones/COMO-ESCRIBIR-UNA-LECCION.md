# Como escribir una leccion

Una **leccion** es la memoria procedural del proyecto: lo que se aprendio
haciendo, guardado por **clase de trabajo** y no por numero de feature, para que
la proxima vez se encuentre por tema.

Los artefactos de una feature (`spec-*`, `plan-*`, `impl-*`, `review-*`) cuentan
**que paso en la feature N**. Una leccion cuenta **como se hace esta clase de
tarea en este proyecto**. Son cosas distintas y por eso viven separadas.

> Documento del arnes: se refresca al reinstalar. Las lecciones que escribas en
> `docs/lecciones/*.md` son tuyas y **sobreviven** a `--reset`.

---

## La regla que ordena todo: primero patchear, crear al final

Cuando aprendiste algo, buscá el lugar donde ponerlo **en este orden** y quedate
en el primero que sirva:

1. **Patchea la leccion que estuvo en juego.** Si en esta tarea consultaste o
   usaste una leccion y lo que aprendiste extiende su territorio, va ahi. Es la
   que estuvo en la cancha: es la que corresponde ampliar.
2. **Patchea el paraguas existente.** Si ninguna estuvo en juego pero hay una
   leccion de clase que cubre el tema, agregale una subseccion, un pitfall, o
   ampliale los `triggers` para que la proxima vez se encuentre.
3. **Agrega un archivo de apoyo** bajo una leccion existente,
   en `docs/lecciones/<clase>/referencias/<tema>.md`: el detalle de una sesion
   concreta (la transcripcion del error, la receta de reproduccion, la rareza de
   una version) o un banco de conocimiento condensado (extractos de
   documentacion, notas de dominio). Dejale a la leccion un puntero de una linea
   para que se sepa que existe.
4. **Recien entonces, crea una leccion nueva** — y solo si ninguna clase
   existente cubre el tema.

La forma que buscamos es **pocas lecciones de clase, ricas**, cada una con sus
referencias. No una lista larga y plana de una-leccion-por-feature: esa lista no
se lee, no se mantiene y termina siendo ruido.

## El nombre tiene que ser de CLASE

El nombre es lo que hace que la leccion se encuentre dentro de seis meses.

**Si el nombre solo tiene sentido para la tarea de hoy, esta mal.** Volve al
punto 1, 2 o 3 de la lista de arriba.

| Mal | Bien |
| --- | --- |
| `fix-espejo-roles-feature-16` | `espejo-de-roles` |
| `error-connection-timed-out` | `hub-postgres-inalcanzable` |
| `debug-instalador-2026-08-16` | `instalador-idempotente` |
| `arreglo-ureq` | `dependencias-y-adrs` |

`harness_cli leccion nueva` **rechaza** el nombre que:

- contenga `feature` o `#`,
- empiece con `fix-`, `debug-`, `audit-` o `hotfix-`,
- contenga una fecha (`2026-08-16`),
- contenga un numero de tres o mas digitos.

No hay forma de saltear la regla: no existe `--force`. El remedio es elegir un
nombre de clase, que es exactamente lo que la regla busca.

## Que NO capturar

Esto es lo que separa una memoria que ayuda de una que se auto-sabotea. Una
leccion equivocada no es neutra: es una restriccion que el proyecto se cita a si
mismo durante meses.

1. **Fallas dependientes del entorno.** Un binario que falta, un error de
   instalacion fresca, un `command not found`, una credencial sin configurar, un
   `PATH` mal armado. Eso se arregla; no es una regla durable. Si una herramienta
   fallo por estado de setup, capturá **el fix** (el comando de instalacion, el
   paso de configuracion, la variable a exportar) dentro de una leccion de setup
   o troubleshooting.
2. **Afirmaciones negativas sobre herramientas.** "El hub no funciona", "esa tool
   esta rota", "no se puede usar X". Se endurecen en negativas que despues alguien
   cita como verdad, mucho despues de que el problema real se arreglo. Escribi lo
   que **si** funciona.
3. **Errores transitorios que ya se resolvieron.** Si el reintento funciono, la
   leccion es el patron de reintento, no la falla original.
4. **Narrativas de una tarea unica.** "Como cerre la feature #14" no es una clase
   de trabajo. "Como se mide una mejora de performance antes de cerrarla" si.
5. **Fracasos no resueltos disfrazados de practica recomendada.** Si probaste
   cinco caminos, ninguno funciono y terminaste pidiendo ayuda: **no** escribas
   esos cinco intentos como "flujo recomendado". Eso presenta una secuencia de
   fracasos no testeada como guia validada, y la proxima sesion la va a creer y
   repetir. O no escribis nada, o escribis unicamente la alternativa que sabes
   que funciona.

## "Ninguna" es una salida valida — pero no el default

Con la regla `require_leccion` activa, cerrar una feature exige declarar que se
aprendio:

```bash
sh harness_cli close --feature <id> --status done --leccion <clase>
sh harness_cli close --feature <id> --status done --leccion ninguna \
                     --leccion-motivo "trabajo mecanico, sin tecnica nueva"
```

Una feature que salio derecho, sin correcciones y sin tecnica nueva, no deja
leccion: eso es honesto y se declara con `ninguna` y su motivo. Pero **no** es la
respuesta por default. Una feature que costo, que tuvo un fork de diseno, que
choco con un pitfall o que te obligo a corregir el rumbo, casi siempre deja algo.

## El formato

```markdown
---
nombre: espejo-de-roles
descripcion: Mantener roles/ como fuente unica y sus espejos por backend.
triggers: [roles, espejo, .claude/agents, harness_check]
relacionadas: [instalador-idempotente]
origen: [7, 9]
usos: 4
ultimo_uso: 2026-08-16
ultima_actualizacion: 2026-08-16
estado: activa
---

## Cuando aplica

<En que situacion alguien deberia leer esto. Concreto: el sintoma o la tarea.>

## Procedimiento

<Los pasos, en orden. Lo que hay que hacer, no la teoria.>

## Pitfalls

<Lo que sale mal. Cada uno con su sintoma, para que se reconozca a tiempo.>

## Verificacion

<Como se sabe que quedo bien: el comando, la salida esperada.>
```

Campos del frontmatter:

| Campo | Que es |
| --- | --- |
| `nombre` | Tiene que coincidir con el nombre del archivo (`harness_check` lo verifica y **bloquea** si no coincide). |
| `descripcion` | Una sola oracion, maximo 80 caracteres, terminada en punto. Es lo que se ve en `leccion list`. |
| `triggers` | Las palabras por las que alguien va a llegar aca. Es el campo que decide si la leccion se encuentra o no. |
| `relacionadas` | Otras lecciones que conviene leer junto a esta. |
| `origen` | Los ids de las features donde se aprendio. |
| `usos` | Lo incrementa `leccion usar`; es la telemetria que decide que esta vivo. |
| `ultimo_uso` | Fecha del ultimo `leccion usar`. |
| `ultima_actualizacion` | Fecha del ultimo cambio de **contenido** (no de uso). |
| `estado` | `activa`, `stale` o `archivada`. |

## Los comandos

```bash
sh harness_cli leccion list             # el catalogo, ordenado por uso
sh harness_cli leccion list --json
sh harness_cli leccion show <clase>     # leerla entera
sh harness_cli leccion nueva <clase>    # crear (el ULTIMO recurso)
sh harness_cli leccion usar <clase>     # dejar rastro de que sirvio
```

`leccion usar` no es burocracia: es lo que despues distingue una leccion viva de
una muerta. Corrélo cuando una leccion te resolvio el problema.

## Sin secretos

Las lecciones son archivos **versionados** del repositorio: entran a los diffs,
a los PRs y al historial de git para siempre. No lleves ahi credenciales, tokens,
hostnames internos ni datos personales. Si el procedimiento necesita un secreto,
nombrá la variable de entorno, nunca su valor.
