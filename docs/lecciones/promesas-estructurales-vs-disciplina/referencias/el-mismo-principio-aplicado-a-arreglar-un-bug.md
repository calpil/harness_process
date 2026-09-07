# El mismo principio, aplicado a ARREGLAR un bug (feature #44)

Referencia de la leccion `promesas-estructurales-vs-disciplina`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

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
