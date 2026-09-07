# El ORDEN tambien es estructura

Referencia de la leccion `promesas-estructurales-vs-disciplina`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

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
