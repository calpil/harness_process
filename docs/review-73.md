# Review - Feature #73: verify corre UN comando por AC y no lo dice
Revisado: approved · 2026-09-05T12:15:07Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento. Metodo: medir el corpus real antes de
escribir el spec, y despues mutar produccion para confirmar que cada test cae.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/verificacion.rs:209 | CUBIERTO. `parsear` acumula todos los comandos del AC abierto. Probado en rojo por partida triple: con `&& ultimo.comandos.is_empty()` caen el test unitario, el de comportamiento y el del corpus real. |
| AC-2 | rust/src/commands/verify.rs:97 | CUBIERTO. Un `Resultado` por comando; rust/tests/cli_basics.rs:2964 cuenta las filas del reporte (tres para el AC de tres comandos, una para el de uno). |
| AC-3 | rust/src/verificacion.rs:598 | CUBIERTO. `sin_repetir` deduplica y la usan los DOS lugares que hablan de "AC en rojo". El test afirma `matches("AC-1").count() == 1` sobre stderr y que ninguna fila del AC quede verde. |
| AC-4 | rust/src/verificacion.rs:1270 | CUBIERTO. El test del corpus recorre los 63 specs reales comparando comandos contra comandos. Los 62 de un comando por AC no cambian. |
| AC-5 | rust/src/verificacion.rs:45 | CUBIERTO. `es_manual()` = lista vacia; los tests de AC manual pasan sin tocar su semantica. |
| AC-6 | rust/src/verificacion.rs:209 | CUBIERTO. La guarda `!ac_ilegible` de la #68 esta intacta y su test tambien (`v[0].es_manual()`, "el comando de una linea ilegible se le colgo a AC-1"). |
| AC-7 | rust/src/verificacion.rs:598 | CUBIERTO. `rojos_del_reporte` parsea fila por fila: un reporte viejo de una fila por AC se lee igual, y deduplicar no cambia nada cuando no hay repetidos. |
| AC-8 | rust/tests/cli_basics.rs:2941 | CUBIERTO. Suite, clippy, smoke y paridad. |
| AC-9 | rust/src/verificacion.rs:1270 | CUBIERTO POR EL TEST DEL CORPUS, no a mano. El AC-8 de la #72 declara cuatro `Comando:` y el test compara comandos parseados contra declarados spec por spec; con la mutacion puesta lo delata por nombre (`el parser reporto 1 comando(s) y el spec declara 4`). No se re-corrio `verify --feature 72`: son ~6 minutos de suite, clippy, smoke y paridad, y no agregaria evidencia. |

## El hallazgo del review: dos tests defendian el bug

No es que faltaran tests. Habia dos, y los dos pasaban:

1. `parse_should_only_report_commands_the_spec_actually_declares`, que dice
   **"INVARIANTE: el parser no inventa ni pierde comandos"** y corre sobre los
   310+ AC reales. Su oraculo tenia
   `ac_abierto = false; // solo el primero cuenta, como en parsear`: contaba lo
   mismo que el codigo descartaba, asi que la igualdad se cumplia siempre.
2. `parse_should_keep_only_the_first_command_of_an_ac`, que afirmaba el descarte
   como si fuera la intencion.

El primero es el caso de manual de la leccion
`criterios-de-cierre-que-se-pueden-fallar`: un criterio que no puede fallar no
verifica, tranquiliza. El segundo es mas sutil y mas comun — describia con
exactitud lo que el codigo hacia, y nadie volvio a preguntarse si eso era lo que
tenia que hacer. Los dos se reescribieron; ninguno se borro.

## Lo que NO esta verificado

- **El costo en tiempo.** Un AC con cuatro comandos ahora tarda la suma de los
  cuatro. Esta declarado en el spec como limite conocido; no hay test que lo
  mida, y no deberia haberlo: seria un criterio dependiente de la maquina.
- **Los reportes ya emitidos** (AC-7) se prueban por lectura del parser, no
  contra los 40+ archivos historicos: `rojos_del_reporte` es puro y su
  comportamiento sobre una fila por AC no cambio, pero nadie los volvio a leer
  uno por uno.
- **El AC-9 no se comprobo re-ejecutando** los cuatro comandos del AC-8 de la
  #72. Se comprobo que el parser los VE, que es lo que la feature cambia; que
  esos cuatro comandos pasen es asunto de la #72, no de esta.

## Riesgos que el cambio introduce

- **El reporte gana filas.** Un AC con N comandos ocupa N filas. El gate las
  parsea fila por fila y deduplica, asi que no se rompe; el efecto visible es un
  `docs/verify-<id>.md` mas largo.
- **Un AC puede tardar mas.** Antes se corria uno de sus comandos.

## Veredicto

Los nueve AC tienen cobertura. Las dos mutaciones —volver al primer comando,
sacar la deduplicacion— ponen en rojo al menos un test cada una, y la primera
tambien delata el spec real que motivo la feature.
