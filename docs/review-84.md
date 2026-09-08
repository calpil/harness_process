# Review - Feature #84: leccion partir: parte una leccion sobre el tope a referencias/ con informe y --aplicar, reconoce las secciones por feature en sus formas reales, y el check resume en una linea las lecciones sobre el tope
Revisado: approved · 2026-09-08T23:58:54Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento, mas una revision adversarial de solo
lectura (ultracode: tres lentes —spec, casos borde, docs— y un refutador; 4
agentes, 15 minutos, sin cargo ni binario). Metodo propio: seis tests de
integracion contra HEAD, unitarios con los titulos reales de realestate,
cuatro mutantes, suite completa, clippy `-D warnings`, paridad, `verify`, y
el AC manual corrido sobre una copia de las cuatro lecciones reales.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/commands/leccion.rs:212 · rust/tests/cli_basics.rs:9347 · :9406 | CUBIERTO. El informe no escribe (bytes comparados) y cubre los tres estados: candidatas, bajo el tope, sin candidatas sobre el tope. Tras la revision, el saldo es lo que la leccion va a medir (test en la franja: 255 -> 253) y con tope 0 no habla del tope. |
| AC-2 | rust/src/particion.rs:79 · :597 · :604 | CUBIERTO. Ocho formas reales, nueve canonicos, tres de clase sin numero; mutante "sin fecha" muerto por el unitario. `secciones_por_feature` (rust/src/lecciones.rs:528) usa el mismo criterio. |
| AC-3 | rust/src/particion.rs:485 · :508 · rust/tests/cli_basics.rs:9462 · :9438 | CUBIERTO. Cuerpo verbatim con `###`, canonicas byte-identicas (ya sin podar blancos), `usos` intacto, fecha de hoy, rollback restaura y borra la referencia, CRLF conservado, respaldo con id unico por segundo. |
| AC-4 | rust/src/particion.rs:256 · rust/tests/cli_basics.rs:9515 | CUBIERTO. Mutante "nada es canonico" muerto por unitario e integracion; el remedio del informe repite los `--seccion`. |
| AC-5 | rust/src/commands/leccion.rs:212 · rust/tests/cli_basics.rs:9562 | CUBIERTO. Exit 2 con lo movido en disco (OBS-2) y sin candidatas no escribe. |
| AC-6 | rust/src/particion.rs:334 · rust/tests/cli_basics.rs:9597 | CUBIERTO. Sufijo `-2`, segunda corrida sin cambios, y la referencia de una corrida fallida se reutiliza. |
| AC-7 | harness_check.sh:449 · tests/leccion_tope_check.sh:54 | CUBIERTO. Mutante "una linea por leccion" muerto por el script (cuenta 3). Dos copias identicas. |
| AC-8 | rust/src/lecciones.rs:550 · rust/src/commands/leccion.rs:514 · rust/tests/cli_basics.rs:9625 | CUBIERTO. Rojo contra HEAD por su aserto. |
| AC-9 | docs/impl-84.md:31 | CUBIERTO. Medido: 2361 -> 1994, 398 -> 359, 382 -> 266, 275 sin candidatas; ninguna bajo el tope con lo mecanico, todas salen 2 con la lista para `--seccion`. Es lo que el spec dijo antes de medir. |
| AC-10 | README.md:686 · UPDATING.md:124 · docs/architecture.md:161 · docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md:107 · setup_harness.sh:1256 · setup_harness.ps1:1012 · AGENTS.md:116 | CUBIERTO. `cmp` de las tres parejas de copias en el comando del AC; paridad 10/10; la frase stale de la #80 en UPDATING y el AGENTS.md de la raiz corregidos por la lente de docs. |

## Lo que el review tiene que decir

1. **La revision adversarial pago otra vez.** Veinte hallazgos, doce
   distintos confirmados, diez corregidos con test. El mas serio: el informe
   no contaba lo que `--aplicar` agrega (punteros e indice), asi que en la
   franja de 1..(5+N) lineas bajo el tope decia "bajo el tope" y despues
   movia y salia 2. La causa era tener DOS calculos del cuerpo resultante;
   ahora hay uno (`reescribir`) y el informe es la aplicacion sin escribir.
   Costo: 4 agentes, 489k tokens, 15 minutos.
2. **Un bug de la #28 salio a la luz por la promesa de esta**: "`lecciones
   rollback` lo deshace" era falso en un repo donde `consolidar` aplico
   alguna vez, porque su respaldo se llamaba `consolidar` y ordena despues
   de cualquier timestamp. Se arreglo aca (id con timestamp) porque la
   promesa que se imprime tiene que ser cierta (leccion
   remedios-que-la-herramienta-sugiere).
3. **Lo mecanico no alcanza, y el comando lo dice con el numero.** Las
   cuatro lecciones de realestate quedan sobre el tope despues de
   `--aplicar`; el remedio siguiente (`--seccion` con las secciones mas
   grandes) sale en el mismo mensaje. Lo que queda es editorial.

## Riesgo declarado

- El respaldo es del arbol entero de `docs/lecciones/` (como `curar`): un
  rollback posterior a OTRA operacion del curador tambien deshace esta. Es la
  semantica de `lecciones rollback` desde la #21.
- `--aplicar` que falla a mitad deja las referencias ya escritas y la leccion
  sin tocar (se guarda al final, atomico); el error nombra el respaldo y el
  `rollback --id`, y el reintento reutiliza las referencias identicas.
- El slug corta a 60 y puede terminar en un numero suelto: convencion de las
  referencias existentes, no se cambia.

## Veredicto

Diez AC con cobertura: ocho tests de integracion (cinco rojos por subcomando
inexistente, uno por aserto, dos nacidos de la revision), diecisiete
unitarios contra datos reales, cuatro mutantes muertos, doce hallazgos de la
revision adversarial resueltos o registrados, suite y clippy limpios,
paridad 10/10, `verify` 9/9 con el manual medido.
