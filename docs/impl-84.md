# Impl - Feature #84: leccion partir: parte una leccion sobre el tope a referencias/ con informe y --aplicar, reconoce las secciones por feature en sus formas reales, y el check resume en una linea las lecciones sobre el tope

Spec: docs/spec-feature-84-leccion-partir-parte-una-leccion-sobre-el-tope-a.md
Plan: docs/plan-feature-84-leccion-partir-parte-una-leccion-sobre-el-tope-a.md

## Lo que habia

El tope de la #80 se sostenia con un contrato que solo veia titulos
`(feature #N)` y con un procedimiento a mano (guia, paso 3). En realestate,
cuatro lecciones sobre el tope (2361, 398, 382 y 275 lineas) con titulos
`(#115, 2026-08-30)`, `feature #100 (2026-08-28)`, `(2026-09-07, front
#131)` y `Patch #100:`: el contrato no nombraba ninguna seccion, el Stop
mostraba cuatro parrafos `[i]` y nadie iba a partir 2361 lineas a mano.

## El arreglo

Un modulo `particion.rs` con la parte mecanica del paso 3: `secciones` corta
el cuerpo en bloques `## ` (los `###` y los bloques de codigo viajan adentro),
`cuenta_una_feature` reconoce los titulos con `#N` o fecha fuera de las
canonicas, `planificar` es puro y calcula el saldo REESCRIBIENDO el cuerpo con
la misma funcion que usa `aplicar` (punteros e indice incluidos), `resolver`
resuelve un `--seccion`, y `aplicar` es la unica capa con I/O: respalda con el
respaldo del curador, escribe cada `referencias/<slug>.md`, quita los bloques,
deja los punteros en el indice, fecha `ultima_actualizacion` y conserva el fin
de linea de la leccion. El comando informa primero y con `--aplicar` mueve; el
contrato, `lecciones status` y `harness_check.sh` lo nombran.

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | rust/src/commands/leccion.rs:212 · :311 · rust/src/particion.rs:213 · rust/tests/cli_basics.rs:9347 · :9406 | Sin `--aplicar` imprime lineas/tope, las candidatas con sus lineas, el saldo (lo que la leccion va a medir, indice y punteros incluidos: en la franja del tope el informe dice `253 lineas: faltan 3` y `--aplicar` termina en 253) y —si no alcanza— cuanto falta y las no canonicas mas grandes; el test compara los bytes de la leccion antes y despues y exige que `referencias/` no exista. Bajo el tope: "nada que partir". Sin candidatas y sobre el tope: "Ninguna seccion cuenta una sola feature" + faltan + sugeridas, exit 0. Con `rules.leccion_max_lineas: 0` dice "sin tope" y nunca "bajo el tope" ni "faltan 0". |
| AC-2 | rust/src/particion.rs:71 · :79 · :597 · :604 · rust/src/lecciones.rs:528 | `cuenta_una_feature` = no canonica y (`#<digito>` o `YYYY-MM-DD`). Unitarios con los OCHO titulos reales de realestate y este repo, nueve canonicos (con acento, con prefijo, con `#N` adentro) y tres de clase sin numero. `secciones_por_feature` delega en el mismo criterio, asi el contrato lista lo mismo que el comando. |
| AC-3 | rust/src/particion.rs:485 · :508 · :359 · :409 · :117 · rust/tests/cli_basics.rs:9462 · :9438 | Respaldo `curador::respaldar` con id unico por segundo (`id_de_respaldo`, :452), un archivo por seccion con `cabecera_de_referencia` y el cuerpo verbatim (el test compara el bloque original contra la cola del archivo, `###` incluido), punteros en `## Referencias` (creado al final sin podar los blancos de la ultima canonica), `ultima_actualizacion` = hoy, `usos` intacto, las cuatro canonicas byte-identicas, `lecciones status` sin SOBRE EL TOPE, `lecciones rollback` devuelve la leccion byte-identica y borra la referencia; una leccion CRLF sale CRLF entera (leccion y referencia). |
| AC-4 | rust/src/particion.rs:256 · :736 · rust/src/commands/leccion.rs:272 · rust/tests/cli_basics.rs:9515 | Exacto, despues subcadena unica sin acentos ni mayusculas; ambigua lista y no escribe (bytes iguales); canonica se niega nombrando las cinco; unica se mueve junto con la candidata automatica y las demas quedan. El remedio del informe repite los `--seccion` pasados, asi corrido tal cual mueve lo que el informe lista. |
| AC-5 | rust/src/commands/leccion.rs:212 · rust/tests/cli_basics.rs:9562 | Tras mover, se replanifica sobre la leccion nueva: si `falta() > 0`, `Exit 2` con faltan y las sugeridas; lo movido queda (la referencia existe y el bloque ya no esta). Sin candidatas con `--aplicar`: `Exit 2`, bytes iguales, sin `referencias/`. |
| AC-6 | rust/src/particion.rs:334 · :777 · rust/tests/cli_basics.rs:9597 | Dos titulos identicos -> `caso-repetido-feature-1.md` y `-2.md`; la segunda corrida dice "nada que partir", no cambia la leccion y deja dos referencias y dos punteros. Una referencia que quedo escrita por una corrida fallida se reutiliza (mismo titulo y mismo cuerpo) en vez de duplicarse. |
| AC-7 | harness_check.sh:449 · tests/leccion_tope_check.sh:54 | Una sola linea `[i] N leccion(es) sobre el tope de T lineas: a (398), b (275)...` con el comando; el test agrega una segunda leccion larga y exige exactamente una linea que nombre las dos; `referencias/` no cuenta y `0` o un tope mas alto callan. Las dos copias identicas (`cmp`). |
| AC-8 | rust/src/lecciones.rs:550 · rust/src/commands/leccion.rs:514 · rust/tests/cli_basics.rs:9625 | El contrato de `usar`/`close` nombra `sh harness_cli leccion partir <clase>` como primer paso y lista `Otro caso (#15, 2026-09-01)` y `Patch #100: ...`; `lecciones status` imprime la linea "Sobre el tope: sh harness_cli leccion partir <clase>" cuando alguna lo supera. |
| AC-9 | docs/review-84.md:1 | MANUAL, corrido sobre una copia de las cuatro lecciones de realestate con el binario de esta rama (informe y `--aplicar`): el-verde 2361 -> 1994 (8 movidas, faltan 1744; su indice "Referencias por tema" recibio los ocho punteros detras del que ya tenia), el-control 398 -> 359 (1, faltan 109), un-control 382 -> 266 (2, faltan 16), el-sqlstate 275 (0 candidatas, faltan 25; sale 2 y no escribe). Ninguna baja del tope con lo mecanico, como predijo el spec; las cuatro salen 2 con la lista para `--seccion`. |
| AC-10 | README.md:686 · UPDATING.md:124 · :153 · docs/architecture.md:161 · docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md:107 · setup_harness.sh:1256 · setup_harness.ps1:1012 · AGENTS.md:116 · rust/src/verificacion.rs:1269 | Comando en el bloque de `leccion`, parrafo de como parte, bullet de `leccion_max_lineas` y la frase de la #80 corregida en las dos copias de UPDATING, bullet del modulo en architecture, paso 1 de la guia (dos copias, con las cinco canonicas por prefijo), texto AGENTS de los dos instaladores y del AGENTS.md de la raiz (paridad 10/10), AC-9 en el corpus. |

## El rojo

Los seis tests de integracion se corrieron contra el binario de HEAD antes de
escribir el modulo: cinco cayeron por `unrecognized subcommand 'partir'` (no
hay forma de que un subcomando nuevo caiga por su aserto contra HEAD) y el de
AC-8 cayo por su aserto: `usar` salia 2 pero sin `leccion partir larga` en el
contrato. Los unitarios nacieron con el modulo. Un ajuste del fixture: el
bloque de una seccion incluye su titulo y sus lineas en blanco (44, no 41).

Mutaciones, con `cmp` antes y despues, restauracion y `touch`:
- sin la fecha en `cuenta_una_feature`: cae el unitario de las formas reales
  (`Incidente del 2026-09-06`).
- nada es canonico: caen el unitario de canonicas y el de integracion de AC-4
  (`Pitfalls` se habria movido).
- el indice existente se ignora: cae el unitario de `insertar_punteros`.
- el check vuelve a avisar por leccion: cae `tests/leccion_tope_check.sh`
  ("mas de una linea [i]: 3").

## La revision adversarial (ultracode: tres lentes y un refutador, solo lectura)

Tres agentes leyeron el worktree con lentes distintas —spec, casos borde,
docs— y un cuarto intento refutar cada hallazgo. Veinte hallazgos, cuatro
refutados, dieciseis confirmados que son doce distintos (tres duplicados).
Ninguno bloqueante; diez se corrigieron en el diff, con test cada uno:

1. **El informe mentia en la franja del tope**: `saldo` era `lineas -
   candidatas` y no contaba los punteros ni el indice que `--aplicar` agrega;
   cerca del tope decia "bajo el tope" y `--aplicar` movia y salia 2.
   Arreglado: `planificar` reescribe el cuerpo con la misma `reescribir` que
   usa `aplicar` (rust/src/particion.rs:389); test en la franja (255 -> 253).
2. **Tope 0**: decia "sin tope" y a renglon seguido "bajo el tope" / "faltan
   0". Arreglado el texto; test con `rules.leccion_max_lineas: 0`.
3. **El indice nuevo podaba los blancos finales de la ultima canonica** (AC-3
   promete byte-identico). Arreglado: no se poda; unitario.
4. **CRLF mezclado** en la leccion y la referencia. Arreglado: `lineas_de`
   quita el `\r`, `con_eol` lo devuelve entero (`Frontmatter::eol`,
   rust/src/lecciones.rs:190); test de integracion CRLF.
5. **Fallo entre las referencias y el guardado**: referencias huerfanas, el
   error no nombraba el respaldo y el reintento duplicaba con `-2`.
   Arreglado: el error nombra el respaldo y el `rollback --id`, y una
   referencia identica ya escrita se reutiliza (`slug_para`).
6. **Dos `--aplicar` en el mismo segundo pisaban el respaldo**: `id_de_respaldo`
   agrega `-2`; unitario.
7. **Un ``` sin cerrar arrastraba las canonicas** adentro de la seccion abierta.
   Arreglado: si el bloque no cierra, se leen los titulos sin mirar bloques;
   unitario.
8. **El remedio del informe omitia los `--seccion` pasados**. Arreglado; test.
9. **`lecciones rollback` elegia el respaldo fijo `consolidar`** por orden de
   texto (bug de la #28, pre-existente): el id de consolidar ahora lleva
   timestamp (rust/src/commands/leccion.rs:1128); los tests de rollback y
   consolidar siguen verdes.
10. **Docs**: UPDATING decia "por cada leccion" (frase de la #80), AGENTS.md
    de la raiz sin la frase nueva, la guia decia "cuatro" canonicas cuando
    son cinco y por prefijo, el comentario de setup_smoke. Corregidos.

Los cuatro refutados (y por que): "canonica por prefijo es mas ancho que el
spec" —lo exige AC-2 con los titulos reales—; "dos titulos identicos no se
pueden elegir con `--seccion`" —es la ambiguedad que AC-4 decide—; "`../` en
el nombre" —input deliberado del propio usuario—; "`aplicar` con cero
candidatas toca el disco" —`partir` no la llama; ahora ademas `ensure!`—.
Queda sin cambiar, por diseno: el slug corta a 60 y puede terminar en un
numero suelto (convencion de las referencias existentes).

## Estilo (skills cargados)

`rust-patterns` / `rust-best-practices`: `Seccion` y `Plan` son datos puros;
`resolver` devuelve `Result<_, Exit>` con los tres fallos como mensajes
distintos (misma forma que `perfil replace`); `aplicar` es la unica funcion
con I/O y recibe `hoy`/`ts` de afuera para ser determinista; sin `unwrap`
fuera de tests; `usize::try_from` para el tope. `rust-testing`: unitarios
contra datos reales (los titulos de realestate), un fixture de integracion que
arma la leccion con sus canonicas y sus casos, mutantes por AC.
`find-skills`: sin skill especifica. `rust-async-patterns`: no aplica.
