# Cuando la herramienta que medis es un modelo

Referencia de la leccion `probar-contra-datos-reales`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-10 por `leccion partir` para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

Un LLM no es un chequeo: es una opinion con formato. Todo lo de arriba sigue
valiendo, y ademas:

1. **Comparalo contra una metrica tonta.** Antes de creerle, medi lo mismo con
   algo deterministico (Jaccard, conteo, diff). No para reemplazarlo: para saber
   **donde discrepan**, que es lo unico interesante.

   En la #28, sobre 9 lecciones: el modelo y el Jaccard coincidieron en el par
   real (0.400 / 0.85) y discreparon en otro (0.048 / 0.60). El segundo era una
   vecindad semantica que la metrica lexica no podia ver — y aun asi se decidio
   NO fusionarlo. Las dos senales juntas dijeron mas que cualquiera sola.

2. **Un umbral que no podes calibrar es un numero inventado.** Con 9 items y un
   solo caso positivo, cualquier corte entre 0.1 y 0.4 da identico resultado: no
   hay nada en la zona gris. Reportá el numero y dejá decidir a quien lee, en vez
   de fingir una precision que el corpus no soporta.

3. **Verificá de punta a punta con el backend de verdad, no con uno falso.** Un
   mock prueba tu parser; no prueba que puedas hablar con un modelo. Y capturá
   la salida REAL como fixture: `claude -p` devuelve JSON pelado y `kimi -p` lo
   envuelve en vinnetas con banner y linea de sesion. Ninguna de las dos formas
   se te habria ocurrido inventarla.

4. **Recortá lo que ve.** Si el modelo no necesita el dato para su tarea, no se
   lo mandes. En la #28 ve nombre, descripcion y triggers, y **nunca** el cuerpo
   de la leccion: asi lo peor que puede hacer es equivocarse, no filtrar.
