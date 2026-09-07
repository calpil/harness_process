# El PIPE que se traga el exit code (feature #64)

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

El caso mas barato de "comando que no puede fallar", y el mas facil de escribir
sin darse cuenta:

```
Comando: `bash tests/setup_smoke.sh 2>&1 | tail -5`
```

`verify` ejecuta con `sh -c` y **sin `pipefail`** (`verificacion.rs:228`), asi
que el rc del pipeline es el de `tail`: **siempre 0**. Comprobado:
`sh -c 'false | tail -5'` sale 0. El smoke podia romperse entero y el AC seguia
verde. Dos AC de la #64 nacieron asi (`| tail -5` y `| tail -3`) y los encontro
el reviewer, no el autor.

El agravante es que el pipe se agrega por una razon buena —"que no me llene la
pantalla"— y el costo no se ve: el reporte queda igual de verde.

Regla corta: **en un `Comando:` el ultimo proceso del pipeline es el que decide
si el AC esta verde.** Si lo que te importa es el rc del PRIMERO, no uses pipe:
manda la salida a `/dev/null` (`cmd >/dev/null 2>&1`) o antepone
`set -o pipefail;`. Y `grep` como ultimo eslabon si sirve, porque `grep` falla
cuando no encuentra: `... | grep -E "[1-9][0-9]* passed"` es un buen criterio
justamente porque su rc significa algo.

Al corregir los dos comandos de la #64, uno de ellos **paso a rojo de
inmediato**: `harness_check.sh` fallaba y el `| tail -3` lo venia tapando. Ese
rojo era el valor de la correccion.
