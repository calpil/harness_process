Aplicado: 2026-09-01T23:29:56Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #69: una linea AC ilegible no desaparece en silencio

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 69`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:108 (spec `dispara`), docs/prd/PRD-master.md:110 (spec `master`) y 131 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/revision.rs`, `rust/src/commands/verify.rs`, `rust/src/revision.rs`, `rust/src/verificacion.rs` y 1 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica La tabla de hitos llega hasta el #13 y desde el #14 los cierres van a la Bitacora, que la escribe el propio `close`. La #69 no cambia una capacidad del producto: termina de cerrar el limite que la #68 dejo declarado.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `ningun`), docs/prd/SDD-master.md:10 (spec `ninguna`) y 262 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/revision.rs`, `rust/src/commands/verify.rs`, `rust/src/revision.rs`, `rust/src/verificacion.rs` y 1 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio

Antes:
- **Un AC que el parser no ve no existe para nadie**, y eso incluia justo a los
  que pedian una persona (feature #68). `ac_de` exigia los dos puntos PEGADOS a
  los digitos, asi que `- AC-11 (MANUAL):` —la anotacion que marca "esto lo tiene
  que auditar alguien"— hacia desaparecer el AC de `verify` Y del gate del
  review, que saca su lista del mismo parser. Medido: siete AC invisibles en 55
  specs, en dos familias (`(MANUAL)` y el sufijo de letra `AC-4b`). La marca de
  "esto no lo puede comprobar la maquina" era exactamente lo que hacia que nadie
  tuviera que comprobarlo. La gramatica del nombre es ahora
  `AC-<digitos><letras?>` mas una anotacion opcional entre parentesis que NO
  entra en el nombre, y se afloja **lo justo**: un parser que inventa un AC es
  peor que uno que lo pierde, porque hace fallar cierres que estaban bien.

Despues:
- **Un AC que el parser no ve no existe para nadie**, y eso incluia justo a los
  que pedian una persona (feature #68). `ac_de` exigia los dos puntos PEGADOS a
  los digitos, asi que `- AC-11 (MANUAL):` —la anotacion que marca "esto lo tiene
  que auditar alguien"— hacia desaparecer el AC de `verify` Y del gate del
  review, que saca su lista del mismo parser. Medido: siete AC invisibles en 55
  specs, en dos familias (`(MANUAL)` y el sufijo de letra `AC-4b`). La marca de
  "esto no lo puede comprobar la maquina" era exactamente lo que hacia que nadie
  tuviera que comprobarlo. La gramatica del nombre es ahora
  `AC-<digitos><letras?>` mas una anotacion opcional entre parentesis que NO
  entra en el nombre, y se afloja **lo justo**: un parser que inventa un AC es
  peor que uno que lo pierde, porque hace fallar cierres que estaban bien. Y cuando
  una linea DICE ser un AC y no se puede leer, el arnes la **nombra** en vez de
  descartarla: `verify` la imprime con su texto antes de correr nada y el gate del
  review se niega (feature #69). Un criterio que desaparece por un typo es la
  misma clase de perdida que uno que desaparece por una anotacion, y el silencio
  es lo que la vuelve cara: el autor se entera —si se entera— en el review.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (spec `corpus`), docs/architecture.md:104 (spec `ejecuta`), docs/architecture.md:107 (spec `camino`) y 380 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/commands/revision.rs`, `rust/src/commands/verify.rs`, `rust/src/revision.rs`, `rust/src/verificacion.rs` y 1 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica architecture.md ya describe la gramatica del nombre de un AC y su interaccion con `revision::menciona` (lo escribio la #68). La #69 no agrega un modulo ni cambia esa gramatica: agrega una funcion hermana de `parsear` en el mismo archivo y un aviso en `verify`. El mapa no cambia.

