# La herramienta externa que no esta convierte el test en un placebo

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

Un criterio puede nacer bien y volverse imposible de fallar **sin que nadie lo
toque**, cuando depende de una herramienta que en otra maquina no existe.

`tests/commit_guard_check.sh` decidia si un script se colgaba con `timeout 10`.
En Linux funciona. En macOS `timeout(1)` **no viene con el sistema**: el
subshell devuelve `127` ("no existe"), y el test solo consideraba colgado el
`124` ("se corto"). Resultado: el modo salia **verde pase lo que pase**, en la
maquina donde mas se corre.

El patron a reconocer: **traducir el codigo de salida de una herramienta sin
comprobar que la herramienta corrio**. `127` y `124` son dos cosas
completamente distintas y el test las metia en la misma bolsa ("no es 124,
entonces termino bien").

Procedimiento cuando una prueba depende de algo externo:

1. **Elegi entre varios**: `timeout`, `gtimeout`, `perl alarm`. Alguno hay.
2. **Si no hay ninguno, FALLA** nombrando cual instalar. Un skip verde es la
   forma mas cara de no enterarse: parece cobertura y no lo es.
3. **Proba el mecanismo elegido** contra un caso que debe cortar y uno que no.
   Sin eso, "ahora si mide" es otra afirmacion sin comprobar — el mismo error
   una capa mas arriba.
4. **No traduzcas codigos que no distinguen**: separa "la herramienta corto" de
   "la herramienta no estaba".

Y una advertencia sobre la prueba del rojo, que es la que salva esto: **tambien
se pudre**. La de este test reconstruia UNA de las dos defensas contra el
cuelgue (habia una por feature, la #52 y la #53), asi que el rojo dejo de
aparecer y su fallo se leyo como ruido de un test viejo durante semanas. Si tu
prueba del rojo empieza a fallar, la primera hipotesis no es "el test esta
viejo": es **"el instrumento dejo de medir"**.
