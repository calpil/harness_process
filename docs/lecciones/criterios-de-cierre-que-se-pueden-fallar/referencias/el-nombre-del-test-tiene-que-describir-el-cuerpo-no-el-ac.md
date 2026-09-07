# El nombre del test tiene que describir el CUERPO, no el AC (feature #75)

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

La seccion de arriba sobre el oraculo copiado (#73) se escribio un dia antes de
esto, y aun asi paso de nuevo, en el trabajo siguiente:
`add_should_refuse_a_dependency_cycle` no rechazaba ningun ciclo — su cuerpo
terminaba comprobando una cadena valida, porque el ciclo era inalcanzable por ese
camino.

El disparador es concreto y vale reconocerlo: **estas implementando el AC-5, y
nombras el test por el AC en vez de por lo que el cuerpo comprueba.** El nombre
sale de la intencion; el cuerpo, de lo que se pudo escribir. Cuando los dos se
separan, el nombre gana la lectura y nadie vuelve a mirar el cuerpo.

Que la leccion ya estuviera escrita no alcanzo — igual que la advertencia de la
#23 no alcanzo para la #44. Lo que si lo detecta es barato: **releer el nombre
del test DESPUES de escribir el cuerpo**, y preguntarse si un desconocido que
solo lee el nombre entenderia lo que ahi se comprueba.
