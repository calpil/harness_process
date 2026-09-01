Aplicado: 2026-09-01T22:48:35Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #68: el arnes no pierde los AC que pide revisar a mano

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 68`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:110 (spec `master`) y 145 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/revision.rs`, `rust/src/verificacion.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica La tabla de hitos (seccion 10) llega hasta el #13 y desde el #14 los cierres van a la Bitacora, que la escribe el propio `close`. La #68 no agrega ni cambia una capacidad del producto: arregla que el arnes leyera mal los documentos que ya leia. Lo que si cambia va al SDD (la estrategia de verificacion) y a architecture.md (la gramatica del nombre de un AC y su interaccion con el gate del review).

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `ninguna`), docs/prd/SDD-master.md:102 (spec `silencio`) y 239 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/revision.rs`, `rust/src/verificacion.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: cambio

Antes:
- **Los AC se ejecutan**: cada AC-n de un spec puede declarar `Comando:`, y
  `harness_cli verify --feature <id>` los corre y escribe `docs/verify-<id>.md`.
  Con `require_verify_green`, `close --status done` LEE ese reporte —nunca
  ejecuta— y no deja cerrar con alguno bloqueando.

Despues:
- **Los AC se ejecutan**: cada AC-n de un spec puede declarar `Comando:`, y
  `harness_cli verify --feature <id>` los corre y escribe `docs/verify-<id>.md`.
  Con `require_verify_green`, `close --status done` LEE ese reporte —nunca
  ejecuta— y no deja cerrar con alguno bloqueando.
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
- **Un `Comando:` nunca se cuelga del AC equivocado.** `parsear` lo asocia al
  ultimo AC abierto, asi que un encabezado ilegible le regalaba su comando al AC
  de arriba: reproducido, `AC-1` se quedaba con el `touch` que era del `AC-2`, y
  `verify` habria impreso "AC-1 verde" tras correr la prueba de otro criterio.
  Una linea que arranca como AC y no se puede leer cierra el anterior: vale mas
  perder un comando que adjudicarselo al criterio equivocado.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:101 (spec `lectura`), docs/architecture.md:104 (spec `defecto`), docs/architecture.md:104 (spec `efecto`) y 370 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/revision.rs`, `rust/src/verificacion.rs`, `rust/tests/cli_basics.rs`. Revisa si este documento debe reflejarlo.

Veredicto: cambio

Antes:
- `verificacion.rs`: AC ejecutables (feature #23). `parsear()` es **pura** —lee
  el texto del spec y devuelve `Verificacion { ac, comando: Option<String> }`—, y
  esa pureza es lo que permite probar la compatibilidad contra los 310 AC reales
  del repo sin ejecutar un solo comando.

Despues:
- `verificacion.rs`: AC ejecutables (feature #23). `parsear()` es **pura** —lee
  el texto del spec y devuelve `Verificacion { ac, comando: Option<String> }`—, y
  esa pureza es lo que permite probar la compatibilidad contra los cientos de AC
  reales del repo sin ejecutar un solo comando. El nombre de un AC es
  `AC-<digitos><letras?>` y admite una anotacion entre parentesis que no entra en
  el nombre (`- AC-11 (MANUAL):` es `AC-11`, `- AC-4b:` es `AC-4b`): antes se
  exigian los dos puntos pegados a los digitos y eso tiraba siete AC reales,
  entre ellos todos los marcados `(MANUAL)` (feature #68). Cualquier
  ampliacion de esa gramatica hay que mirarla contra `revision::menciona`, que
  compara nombres de AC token a token: cuando la #68 sumo letras, la guarda de la
  #64 —escrita para nombres de solo digitos— quedo incompleta y una fila de
  `AC-4b` volvio a dar por cubierto al `AC-4`.

