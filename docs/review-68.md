# Review — feature #68: el arnes no pierde los AC que pide revisar a mano
Revisado: approved · 2026-09-01T22:48:46Z · estampado por `harness revision --veredicto`

Revision adversarial sobre el propio trabajo. El mandato fue romperlo.

| AC | evidencia | veredicto |
| --- | --- | --- |
| AC-1 | rust/tests/cli_basics.rs:7519 | cubierto |
| AC-2 | rust/tests/cli_basics.rs:7551 | cubierto |
| AC-3 | rust/src/verificacion.rs:969 | cubierto |
| AC-4 | rust/src/verificacion.rs:988 | cubierto |
| AC-5 | rust/src/verificacion.rs:1013 | cubierto |
| AC-6 | rust/src/verificacion.rs:1047 | cubierto |
| AC-7 | rust/tests/cli_basics.rs:7595 | cubierto |
| AC-8 | docs/verify-68.md:1 | cubierto (manual) |

## Lo que se rompio

**El gate del review empezo a dar por cubierto el AC equivocado, y la culpa es de
esta feature.** `menciona` (rust/src/revision.rs:469) solo se defendia de un
DIGITO despues del nombre: la #64 lo puso para que una fila de `AC-11` no contara
como cobertura de `AC-1`. Esta feature metio **letras** en los nombres (`AC-4b` es
otro criterio que `AC-4`), y con eso una fila que responde por `AC-4b` volvia a
dar por cubierto al `AC-4`.

Es el mismo bug de la #64 con otro alfabeto, reabierto por el cambio que lo
precedia. Confirmado con un test que salio rojo antes de tocar nada
(rust/src/revision.rs:1142) y arreglado extendiendo la guarda a
`is_ascii_alphanumeric`.

Vale la pena nombrar el patron: **ampliar el dominio de un identificador reabre
las defensas escritas para el dominio viejo**. La guarda de la #64 era correcta
para nombres de solo digitos y quedo incompleta en cuanto los nombres pudieron
terminar en letra. No lo detecto ningun test existente porque ninguno tenia un AC
con letra hasta esta feature.

## El AC-8, que se cumplio antes de cerrarse

El AC-8 esta escrito a proposito con la forma que desaparecia. Dos cosas lo
confirman sin que haya que creerle a nadie:

1. Al correr el AC-6 —que asserta que AC trae el arreglo sobre el corpus real—
   aparecio un **octavo**: el AC-8 de este mismo spec. La lista esperada se habia
   escrito antes de que el spec existiera en el corpus.
2. `verify --feature 68` imprime **"7 verde(s), 0 en rojo, 1 manual(es)"**. Antes
   del arreglo ese mismo spec habria dicho "7 en total" y "0 manual(es)": el AC-8
   no habria existido para el arnes.

Si el arreglo se revierte, este AC se vuelve a perder y el AC-6 se pone rojo.

## Lo que aguanto

**El parser no se come prosa.** Se probaron diez formas que NO son AC:
`- AC-12 y AC-13:`, `- AC-:`, `- AC-1 sin dos puntos`, `- ACR-1:`,
`- AC-1 (sin cerrar:`, `- AC 1:`, `-  AC-1:` (doble espacio), `- AC-1b2:`
(letra y numero mezclados), `  - AC-1:` (con sangria) y
`- AC-1 (MANUAL) :` (espacio antes de los dos puntos). Todas rechazadas. La
mutacion que afloja el parser a "cualquier cosa hasta los dos puntos" pone rojo
ese test.

**El corpus real.** El AC-6 asserta la DIFERENCIA nombrada —los siete AC medidos,
mas el AC-8 propio— y no el total, que sube con cada spec nuevo. Ningun AC
inventado.

**Las features cerradas.** `status`, `next`, `journey` y `doctor --json` sobre una
feature cerrada con AC manual no tocan el backlog: se compara el archivo antes y
despues.

**La atribucion de comandos.** Reproducido el daño latente (el comando de un AC
ilegible se le colgaba al anterior) y cerrado. Con una linea ilegible de verdad,
el comando no se le cuelga a nadie.

## Lo que quedo abierto, con nombre

- **Una linea `- AC-` ilegible se descarta sin una palabra.** Es mejor que antes
  —ya no se la adjudica a otro AC— pero sigue siendo silenciosa, y este repo no
  quiere cosas silenciosas. Un typo en el encabezado de un AC hace desaparecer el
  criterio y nadie se entera. Quedo declarado en `impl-68.md`, fuera del alcance
  aprobado.
- **El `return None` del parentesis sin cerrar es una mutacion equivalente.** No
  hay test que lo distinga de dejar caer el caso. El codigo lo declara ahi mismo
  en vez de dejar un `return` que parece proteger algo y no protege nada.
- **Revisar cuesta una fila mas** por cada AC marcado como manual. Es el efecto
  buscado y estaba declarado como OBS-2 en el spec aprobado.
