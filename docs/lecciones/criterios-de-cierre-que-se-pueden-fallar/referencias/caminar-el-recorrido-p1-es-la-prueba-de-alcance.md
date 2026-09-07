# Caminar el recorrido P1 es la prueba de alcance (feature #75)

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

El spec de la #75 tenia este recorrido P1: *"Alan declara que la #21 depende de
la #17 y `next` deja de ofrecerle la #21"*. Se implemento `add --depends-on`,
los ocho AC quedaron verdes, y recien al escribir un test salio que **ese
recorrido era imposible**: las dos features ya existian y `add` crea una nueva.
Habia que agregar un comando (`depende`) que la ficha no pedia.

No fue un AC mal escrito: cada AC era correcto por separado. Lo que faltaba era
la pregunta de arriba — *¿puedo caminar el recorrido de punta a punta con lo que
construi?*

Y tenia una segunda consecuencia que sola no se veia: la deteccion de ciclos del
AC-5 era **codigo inalcanzable**. Por `add`, una feature nueva solo puede
depender de ids anteriores, asi que el grafo es un DAG por construccion y el
ciclo no puede ocurrir. Ese AC solo se podia satisfacer con tests unitarios sobre
una funcion que la CLI nunca iba a llamar en ese estado.

**Procedimiento, antes de dar por cerrada una feature:** tomar el recorrido P1
del spec literalmente, con los datos que existen de verdad, y ejecutarlo. Si hace
falta un paso que no construiste, el alcance estaba incompleto — no el recorrido.

Regla corta: **un AC verde no prueba que la feature sirva; el recorrido si**.
