# Review — feature #70: el gate de citas dice contra que resolvio
Revisado: approved · 2026-09-04T20:01:24Z · estampado por `harness revision --veredicto`

Revision adversarial sobre el propio trabajo. El mandato fue romperlo.

| AC | evidencia | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/revision.rs:1308 | cubierto |
| AC-2 | rust/src/revision.rs:1324 | cubierto |
| AC-3 | rust/src/revision.rs:1288 | cubierto |
| AC-4 | rust/src/revision.rs:1347 | cubierto |
| AC-5 | rust/src/revision.rs:1367 | cubierto |

## Lo mas importante: el ticket estaba mal, y se comprobo antes de construir

El ticket afirmaba que una feature de backend **no tiene forma** de citar el
codigo de un repo hermano, y proponia un AC concreto ("`raices_desde` suma las
raices de los microservicios que el plan declara"). Las dos cosas resultaron
falsas al medirlas:

1. **La cita ya resuelve** en layout `subdir`. `repo_root` ES el directorio que
   contiene los repos vecinos y `raices_desde` ya lo ofrece. Comprobado con el
   gate real, no razonado.
2. **El AC propuesto no se puede implementar tal cual**: el campo `microservicios`
   del backlog es prosa libre —dice `harness` o
   `harness_process (rust/src/revision.rs)`—, no rutas.

Construir lo que pedia el ticket habria agregado raices que ya existian, a partir
de un campo que no las contiene, para arreglar algo que no estaba roto. El defecto
real —que la forma que anda no esta nombrada en ningun lado— habria quedado
abierto igual.

Vale decir con precision **que si estaba roto**: el mensaje. Las dos formas que
una persona escribe por instinto se rechazan con el mismo texto que cuando falta
el archivo, y ese texto no nombra ninguna raiz. La consecuencia se ve en la
review-117: cito `docs/impl-117.md` en la columna que el gate comprueba y dejo el
codigo real en la de al lado. El gate se dio por satisfecho con una cita que no
era la evidencia — que es exactamente lo que el gate existe para impedir.

## Lo que se ataco

**El remedio no puede mentir.** Es el riesgo mas caro de esta feature: ofrecer la
forma `<hermano>/<archivo>` en un layout donde no resuelve manda a la persona a
probar algo que va a fallar, y quema la confianza en todos los mensajes del arnes
(`docs/lecciones/remedios-que-la-herramienta-sugiere.md`). El AC-4 lo prueba en
las dos direcciones: que `subdir` la ofrece, que `root` NO, y —lo que hace que el
test signifique algo— que en `root` la cita efectivamente no resuelve. Sin esa
tercera asercion, el test estaria comprobando una politica contra si misma.

**El diagnostico no puede aparecer siempre.** Un gate que explica en cada corrida
se deja de leer. El AC-5 fija que con una cita que resuelve el diagnostico sea la
cadena vacia, y tambien que no hable por un AC que la fila no menciona.

**La resolucion no se aflojo.** El AC-3 comprueba las dos mitades: que el hermano
resuelva y que una linea inexistente en ese mismo archivo **siga** sin resolver.
Las guardas de `..` y de rutas absolutas se conservan enteras: estan para que un
review no cite fuera del arbol, y el arreglo es explicar, no permitir.

**Una sola copia del texto.** Los dos sitios que rechazan —el gate del cierre y
`estampar`— llaman a `porque_no_resolvieron` (rust/src/revision.rs:707). La #69
costo un test rojo justamente por tener dos copias de la misma pregunta; se evito
repetirlo.

## Lo que quedo abierto, con nombre

- **El layout `root` sigue sin poder citar repos hermanos.** Es un caso real y
  esta declarado fuera de alcance: requiere decidir de donde salen esas raices, y
  el campo que el ticket proponia usar no sirve. El AC-4 se asegura de que al
  menos el mensaje no prometa lo que ese layout no puede dar.
- **No se pudo comprobar que le paso a la #117.** Vive en otro repo y aca no se
  toca. Que su review citara `impl-117.md` es compatible con las dos hipotesis
  —layout `root`, o `../` y absoluta rechazadas—. Queda como hipotesis, no como
  dato.
- **El diagnostico no verifica que la forma ofrecida resuelva para ESA cita.**
  Dice "escribila relativa a una de estas raices" sin comprobar que el archivo
  exista bajo alguna. Es correcto —no sabe cual quiso citar la persona— pero
  significa que el remedio orienta, no garantiza.
