# La pieza mas grande que el problema (feature #77)

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

Hermana de "la regla mas ancha que la evidencia" (#76), del mismo dia y de la
misma feature madre. La #72 encontro un problema real: en el worktree del repo
principal, un `docs/` que es repo aparte queda VACIO, y una sesion lo uso de
excusa para correr `--sin-worktree`. La solucion fue darle al repo docs un
worktree propio por feature. Resolvia el vacio. Y creaba `docs-wt/131-…`,
`docs-wt/132-…`, `docs-wt/142-…`, cada uno con el spec de su feature en una rama
que nadie mergeo. El usuario lo vio en el arbol y pregunto por que sus
documentos no estaban en `docs/` con el PRD y el SDD.

Lo que paso: el problema era "los documentos no llegan a `docs/`" y la solucion
agrego un LUGAR NUEVO donde tampoco llegan. La respuesta chica —escribirlos
directo en `docs/`, como ya hacian el PRD, el SDD y el sello— estaba a la vista,
y el propio repo la habia tomado dos veces antes (#60, #71): los documentos
compartidos van a la raiz.

**La pregunta que lo detecta**: *¿la solucion agrega un lugar, una rama o un
estado que antes no existia?* Si si, preguntar que pasa con eso al cerrar, al
mirar el arbol, a la semana. Una pieza nueva tiene ciclo de vida, y si nadie lo
escribio, el ciclo de vida es "queda ahi".

Regla corta: **antes de agregar una pieza, buscar donde ya va lo parecido.**
