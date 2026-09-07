# Review - Feature #82: el aviso de perfil del cierre cuenta solo las decisiones posteriores a la ultima entrada del perfil
Revisado: approved · 2026-09-07T23:29:21Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento. Metodo: cuatro tests de integracion
contra HEAD leyendo por que cayo cada uno; tres mutantes con `cmp`, `touch` y
`--no-fail-fast`; suite completa, clippy `-D warnings`, `fmt --check`, `verify`;
y el AC manual medido sobre una copia de los datos reales del repo con el
binario de esta rama.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/lecciones.rs:632 · rust/tests/cli_basics.rs:9076 | CUBIERTO. Rojo contra HEAD por el `[i]` que no debia salir (5 > 2); con el fix, 1 nueva <= 2 calla y 4 nuevas > 2 avisa con nuevas, total y corte. El canal sigue siendo stderr y el exit no cambia; el test de la #80 (`0` apaga) sigue verde. |
| AC-2 | rust/src/perfil.rs:416 · :502 · :987 | CUBIERTO. Mutante "plan sin momento" cae en el unitario. Solo unitario: el de integracion no fecha planes; aceptable porque `recolectar` es la misma funcion que usa el binario. |
| AC-3 | rust/src/perfil.rs:611 · :1032 · :1063 · rust/tests/cli_basics.rs:9176 | CUBIERTO. "La mas alta" se prueba contra una feature mas baja que arranco mas tarde; `remove` no cuenta (mutante muerto por unitario e integracion); la bitacora vieja pierde contra el backlog. |
| AC-4 | rust/src/perfil.rs:576 · rust/tests/cli_basics.rs:9257 | CUBIERTO. Sin corte se cuenta todo y el texto lo dice, en el cierre y en `status`; sin archivo de perfil, lo mismo. |
| AC-5 | rust/src/commands/leccion.rs:393 · rust/src/commands/perfil.rs:209 · rust/tests/cli_basics.rs:9135 | CUBIERTO. Las dos cuentas y el corte en texto, `--json` y `sugerir`. `perfil_pendientes` conserva su significado operativo (lo que se compara con el umbral); las tres claves nuevas se agregan. Mutante `>=` muerto por el unitario (rust/src/perfil.rs:1109). |
| AC-6 | docs/impl-82.md:24 | CUBIERTO CON UNA CORRECCION AL SPEC. Medido: corte desde el backlog (inicio de la #80, `2026-09-06T23:25:01Z`), 26 nuevas de 270, no "menos de 25". Las 26 son reales: las OBS decididas hoy en #74, #79, #83 y #82. El aviso saldria una vez mas con el umbral en 25 y es correcto que salga: mide crecimiento desde las entradas de la #80. Al cerrar se quita el `300` y se le ofrece al usuario una entrada del perfil con su si; el spec no se edita para no invalidar la aprobacion, y esta fila lo deja escrito. |
| AC-7 | README.md:727 · UPDATING.md:127 · docs/architecture.md:145 · rust/src/verificacion.rs:1268 | CUBIERTO. `cmp` de las dos copias en el comando del AC; el corpus de `verificacion.rs` registra el AC-6 (MANUAL) de esta feature, como la #80 registro el suyo. |

## Lo que el review tiene que decir

1. **El numero medido contradijo la prediccion del spec por uno.** El AC-6 se
   escribio con "menos de 25 nuevas" antes de medir; la medicion dio 26. La
   diferencia es la propia feature funcionando: las OBS de hoy son
   decisiones nuevas sin entrada que las cite. Se registra la verdad en vez
   de mover el umbral o retocar el AC (la leccion
   `criterios-de-cierre-que-se-pueden-fallar`: un AC manual tambien se puede
   fallar, y cuando falla se dice).
2. **La linea del plan `OBS-1..3: ver el spec` cuenta como decision.** La
   senal `obs-` de la #19 la toma; es ruido de una linea por feature y esta
   fuera de este alcance (el spec lo deja fuera).
3. **Empate al segundo.** `momento > desde` es estricto: una decision en el
   mismo segundo que el `perfil add` no es posterior. Es el caso del propio
   `perfil add` con una nota, que no deberia contarse a si mismo.

## Riesgo declarado

- El corte desde el backlog es una cota inferior (`started_at`): decisiones de
  OTRAS features entre el inicio de la feature citada y el `perfil add` real
  se cuentan como nuevas. Avisa de mas, nunca de menos, y desaparece con la
  siguiente linea `perfil add` de la bitacora.
- `perfil_pendientes` del `--json` cambia de valor (nuevas, no total) para
  quien lo consumia desde la #80; el total sigue disponible con su clave.

## Veredicto

Siete AC con cobertura: cuatro tests de integracion rojos contra HEAD por su
aserto, siete unitarios, tres mutantes muertos, suite 498 + 284 verde, clippy
y fmt limpios, `verify` 6/6 con el manual medido y registrado con su
desviacion.
