# El arnes que prueba el rojo tambien miente, y de dos formas

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

Si la prueba del rojo es lo que salva a los criterios, hace falta decir como se
rompe **ella**. En la #67 se automatizo —mutar el codigo, correr el test, esperar
ROJO, revertir— y la primera corrida dio **seis falsos verdes seguidos**. Dos
causas distintas, las dos silenciosas.

**1. El filtro que no matchea nada corre cero tests y no imprime `FAILED`.**

`cargo test -- --exact los_parsers_no_discrepan` no corre nada: el nombre real es
`markdown::tests::los_parsers_no_discrepan`. Cero tests corridos, salida sin la
palabra `FAILED`, y un arnes que decide por `"FAILED" in salida` lo lee como
verde. Es el 127-vs-124 de mas arriba con otra ropa: **traducir la ausencia de
una señal de fallo en una señal de exito**, sin comprobar que la medicion ocurrio.

El arreglo no es corregir los nombres —eso arregla hoy y no manana— es **exigir
que el test haya corrido**: parsear `running N tests` y tratar `N == 0` como un
estado propio, distinto de verde y de rojo.

**2. Restaurar el archivo mutado deja a `cargo` con el binario mutado.**

`shutil.copy` no preserva la mtime, asi que el `.bak` nace con la hora de la
copia. Al restaurarlo, el archivo bueno queda con una mtime **anterior** a la del
build hecho sobre el codigo mutado: `cargo` compara mtimes, concluye que no hay
nada nuevo y **no recompila**. Las corridas siguientes usan el binario mutado.

Como se vio: un test empezo a fallar en la suite completa y a pasar aislado, y
despues a pasar en las dos en cuanto un edit cualquiera forzo el rebuild. La
tentacion ahi es archivarlo como flaky. No era flaky: era el arnes de mutacion
dejando el arbol de compilacion mintiendo, y el test que fallaba era exactamente
el de la ultima mutacion.

Reglas para un arnes de mutacion:

1. **Comproba que la mutacion cambio el archivo** (`cmp` contra el backup). Un
   ancla que ya no existe muta nada y el verde no significa nada.
2. **Comproba que el test corrio.** Cero tests no es verde.
3. **Toca las fuentes despues de restaurar**, o usa un `target/` aparte. Si no,
   lo que corre despues no es lo que dice el archivo.
4. **Un test que pasa aislado y falla en la suite** —o al reves— es primero una
   sospecha sobre el instrumento, no sobre el test.
