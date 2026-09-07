# La variante del DESTINO IMPLICITO: un comando que apunta a estado global mutable

Referencia de la leccion `promesas-estructurales-vs-disciplina`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

Medido el 2026-09-05 en un proyecto con `one_feature_at_a_time: false` y cinco
sesiones interactivas trabajando a la vez sobre el mismo backlog.

`approve-spec` acepta `--feature` y, cuando no se lo pasan, **apunta a la feature
ACTIVA**. La feature activa no es un dato de quien invoca: es estado global del
backlog, y en paralelo **otra sesion la cambia mientras vos trabajas**. La
secuencia real, con los segundos:

    13:32:09  otra sesion cierra la #100 (que era mia y estaba activa)
    13:32:17  esa sesion arranca la #99
    13:33:04  yo corro `approve-spec --yes` para MI feature

Resultado: el sello *"Estado: approved — aprobado por el USUARIO"* quedo sobre el
spec de la **#99**, que era la plantilla vacia y que el usuario nunca vio, con una
nota que citaba las decisiones de la #100. Cuarenta y siete segundos de
diferencia.

Lo que hace grave a este caso y no solo molesto: **el articulo que ese comando
existe para sostener es "solo el USUARIO aprueba; los agentes tienen PROHIBIDO
auto-aprobar"**. O sea que el unico comando cuya razon de ser es no fabricar una
autorizacion fabrica una autorizacion cuando el destino se resuelve solo. Y el
gate posterior no lo atrapa: `check-spec` de la #99 pasaba a salir **limpio**,
porque para el binario ese spec estaba aprobado y fresco. La unica forma de
detectarlo fue leer el sello y no reconocer la nota.

Por que es esta leccion y no otra: la promesa "no se aprueba lo que el usuario no
vio" se sostenia porque el agente **se acuerda** de pasar `--feature`. Con una
sola sesion eso es invisible; en paralelo es una bomba de tiempo.

El arreglo estructural, en orden de preferencia:

1. **Exigir el destino explicito en los comandos que ESTAMPAN una autorizacion
   del usuario.** Sin `--feature`, que se niegue en vez de adivinar. Es una
   linea, y convierte "acordate" en "no compila".
2. Si el default implicito se conserva por comodidad, **atarlo al contexto de
   quien invoca** (el worktree, el cwd) y no al estado global mutable; y con
   `one_feature_at_a_time: false`, negarse directamente.
3. **Que el sello lleve de que feature habla** dentro de la nota, no solo en el
   nombre del archivo: un sello que se puede leer sin ambiguedad es lo unico que
   permitio descubrir este.

El mismo criterio aplica a todo comando cuyo destino sea implicito: `advance`,
`close --feature` omitido, `verify`. La pregunta que los detecta:

> **Si otra sesion cambia el estado global entre que yo decido y yo ejecuto,
> ¿este comando actua sobre lo que yo creia?**

Y el remedio cuando ya paso, porque tambien hubo que inventarlo: revertir el
`Estado:` a `draft` y **dejar escrito en el cuerpo del spec por que se retiro el
sello**. El sello se saca; la entrada en `progress/history.md` NO, que es
append-only, asi que el rastro queda y hay que explicarlo donde se lea.
