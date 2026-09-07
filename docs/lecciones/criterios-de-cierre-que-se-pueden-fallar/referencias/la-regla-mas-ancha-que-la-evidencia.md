# La regla mas ancha que la evidencia (feature #76)

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

Variante de la hipotesis no reproducida, mas dificil de ver porque la evidencia
SI existia. El incidente de la #72 era real y medido: cuatro features
`--sin-worktree` sobre el mismo arbol. La regla que se escribio fue "una feature
sin aislar bloquea a todas las demas". Sus tests la codificaron con precision,
pasaron, y la regla estaba mal: vetaba a features con su propio worktree, que
no compartian nada con la que no lo tenia. El usuario termino escribiendo
"avisame cuando la #99 libere y arranca".

Lo que paso: de "N features en el mismo arbol se pisan" se salto a "una sin
arbol veta a todas", que es mas facil de implementar y suena mas seguro. El
gate era coherente consigo mismo y con sus tests. Lo detecto el uso real, no la
suite.

**La pregunta que lo detecta**, antes de escribir el gate: *¿que par de cosas se
pisa de verdad?* Nombrar el conflicto concreto —dos escritores en el mismo
directorio— y bloquear exactamente eso. Si la regla que se esta por escribir
bloquea tambien pares que no se pisan, es mas ancha que la evidencia, y va a
costar en paralelismo lo que no compra en seguridad.

Regla corta: **un gate que bloquea de mas no es mas seguro; es una regresion que
todavia nadie reporto.**
