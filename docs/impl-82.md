# Impl - Feature #82: el aviso de perfil del cierre cuenta solo las decisiones posteriores a la ultima entrada del perfil

Spec: docs/spec-feature-82-el-aviso-de-perfil-del-cierre-cuenta-solo-las-de.md
Plan: docs/plan-feature-82-el-aviso-de-perfil-del-cierre-cuenta-solo-las-de.md

## Lo que habia

`perfil_pendientes` (feature #80) contaba TODOS los registros de decision que
ninguna entrada del perfil citaba: 268 en este repo, casi todos de agosto y
ya destilados sin cita. El aviso salia en cada cierre para siempre y el
remedio fue `rules.perfil_pendientes_max: 300`, una regla que ya no media
nada.

## El arreglo

Cada registro lleva su `momento` (timestamp completo), el perfil tiene un
`Corte` —la ultima entrada: la ultima linea `perfil add|replace` de la
bitacora o el `started_at` de la feature mas alta que cita el perfil, la mas
reciente de las dos— y `contar` separa las nuevas del total. El cierre compara
con el umbral solo las nuevas; sin corte cuenta todo, como antes.

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | rust/src/lecciones.rs:632 · rust/src/perfil.rs:642 · rust/tests/cli_basics.rs:9076 | `texto_avisos_de_ciclo` compara `para_el_umbral()` (las nuevas con corte) y el texto dice nuevas, total y corte. Test: 4 decisiones antes del `perfil add`, 1 despues, umbral 2 -> sin aviso (HEAD avisaba, 5 > 2); con 3 mas -> `4 decision(es) nueva(s)`, `8 sin incorporar en total`, `2026-09-05, bitacora`, stdout y exit iguales. |
| AC-2 | rust/src/perfil.rs:416 · :502 · :987 | `recolectar_con` fecha un plan o spec con `inicio_de_feature` (el `started_at` del backlog); sin feature iniciada queda sin momento y `contar` (:644) lo cuenta como nuevo. Unitario: spec de la #2 (iniciada) y de la #3 (no): momentos `2026-09-03` y vacio; con corte `2026-09-04`, nuevas 1 de 2. |
| AC-3 | rust/src/perfil.rs:611 · :585 · :1032 · :1063 · rust/tests/cli_basics.rs:9176 | `ultima_entrada` toma la ultima linea `perfil add|replace` (no `remove`) y la feature citada mas ALTA con `started_at`, y gana la mas reciente. Unitarios: #16 gana a #15 aunque #15 arranco despues; una cita a una feature no iniciada se salta; `replace` mas nuevo gana al backlog, `remove` de 2026-09-09 no cuenta, bitacora vieja pierde. Integracion: sin linea del perfil, `perfil_corte_origen: backlog`, 0 nuevas de 4, sin aviso (HEAD avisaba); con 3 posteriores avisa con `backlog: inicio de la #1`. |
| AC-4 | rust/src/perfil.rs:576 · :1087 · rust/tests/cli_basics.rs:9257 | `Corte::Ninguno` -> `para_el_umbral()` es el total y el texto dice `sin corte`. Unitario: entrada sin cita + `perfil remove` -> `Ninguno`, nuevas == total. Integracion: 4 decisiones, umbral 2 -> aviso `4 decision(es) registradas sin incorporar al perfil` con `sin corte`; sin perfil, `lecciones status` dice `4 decision(es) sin incorporar, sin corte`. |
| AC-5 | rust/src/commands/leccion.rs:282 · :393 · rust/src/commands/perfil.rs:209 · rust/src/perfil.rs:1109 · rust/tests/cli_basics.rs:9135 | `status` texto: `1 decision(es) nueva(s) ... (2026-09-05, bitacora); 5 en total`; `--json`: `perfil_pendientes` 1 (el del umbral), `perfil_pendientes_total` 5, `perfil_corte` con el timestamp completo, `perfil_corte_origen` `bitacora`. `sugerir`: `1 posterior(es) a la ultima entrada del perfil (2026-09-05, bitacora)`. Unitario: el mismo momento que el corte NO es posterior; lo incorporado no entra en ninguna cuenta; `describir` de los tres estados. |
| AC-6 | docs/review-82.md:1 | MANUAL, medido antes de cerrar con el binario de esta rama sobre una copia de los datos reales del repo: corte `2026-09-06T23:25:01Z, backlog: inicio de la #80` (la bitacora se perdio el 2026-09-06), **26 nuevas de 270**. No es "menos de 25": son las OBS decididas hoy en #74, #79, #83 y esta misma feature, decisiones reales sin entrada que las cite. El aviso saldria UNA vez mas al proximo cierre con el umbral en 25, y ese es el diseno: mide crecimiento desde las entradas de la #80. Al cerrar se quita el `300` del backlog y se le ofrece al usuario una entrada con su si. |
| AC-7 | README.md:727 · UPDATING.md:127 · docs/architecture.md:145 · rust/src/verificacion.rs:1268 | Fila de la tabla y parrafo de `lecciones status` en README; bullet y frase en las dos copias de UPDATING (`cmp` limpio); `architecture.md` con `Corte`/`ultima_entrada`/`contar`/`pendientes` y las claves del JSON; el AC-6 (MANUAL) en el corpus de `verificacion.rs`. |

## El rojo

Los cuatro tests de integracion se corrieron contra el binario de HEAD antes
de tocar el codigo: los cuatro cayeron por su aserto (el cierre avisaba con
5 > 2, `status` sin `nueva(s)`, el JSON sin `perfil_corte` ni
`perfil_pendientes_total`). Los siete unitarios nacieron con el modulo.

Mutaciones, con `cmp` antes y despues, restauracion, `touch` y
`--no-fail-fast`:
- `momento > desde` -> `>=`: cae el unitario de AC-5 (el mismo segundo no es
  posterior); el de integracion no lo distingue porque su fixture no empata.
- `perfil remove` contado como entrada: caen el unitario y el de integracion
  de AC-3.
- plan/spec sin momento: cae el unitario de AC-2.

## Estilo (skills cargados)

`rust-patterns` / `rust-best-practices`: `Corte` es un enum de tres estados
(un `Option<String>` no podria decir de donde salio); `ultima_entrada` y
`contar` son puros y se prueban sin filesystem; `pendientes` y
`pendientes_de` son la unica capa con I/O; sin `unwrap` fuera de tests; los
timestamps se comparan como texto porque todos salen de `now_stamp`.
`rust-testing`: tests por escenario con nombre que lo cuenta, un fixture
(`seed_perfil_con_corte`) que escribe el perfil a mano para controlar la
bitacora, y mutantes para que cada AC pueda fallar. `rust-async-patterns` no
aplica: nada asincrono. `find-skills`: sin skill especifica para esto.
