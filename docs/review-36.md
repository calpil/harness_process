# Veredicto de revision - Feature #36: deudas_anotadas_del_arnes

Veredicto global: **aprobado**.

Spec: `docs/spec-feature-36-deudas-anotadas-del-arnes.md` (15 AC)
Plan: `docs/plan-feature-36-deudas-anotadas-del-arnes.md` (`Peldano elegido: 1`)
Evidencia: `docs/impl-36.md`
Reporte: `docs/verify-36.md`

## Estado por AC

Los quince, cubiertos, con un test propio cada uno. La tabla esta en
`impl-36.md`. Lo que un reviewer tiene que mirar:

| AC | Estado | Por que |
| --- | --- | --- |
| AC-1 | cubierto | El test recorre los DOS gates en una sola corrida, no uno solo |
| AC-4 | cubierto | Verifica que nombre el que falta **y que no nombre el que si existe**: sin la segunda mitad, un mensaje que dijera "AC-1, AC-9" pasaria igual |
| AC-5 | cubierto | Prueba del rojo con un test **indentado** dentro de `mod tests`, que es el caso real |
| AC-12 | cubierto | Fija orden, `--json` y campos: lo unico que la feature podia romper sin querer |
| AC-13 | cubierto | Las seis entradas cerradas como `blocked`, no `done` |

## Que hace creible esta revision

Una feature paraguas de seis correcciones chicas es facil de cerrar "a ojo":
cada cambio es de pocas lineas y todos parecen obvios. Tres cosas lo impiden:

1. **Un test por deuda, ninguno compartido.** Si una rompe, el nombre del test
   dice cual.
2. **Dos de las seis rompieron tests existentes**, y eso es informacion: el
   cambio de exit code hizo fallar `spec_gate_should_block_...` y dos unitarios
   de `spec.rs`. Se actualizaron **explicando el porque en el comentario**, no en
   silencio.
3. **La ampliacion del alcance destapo un bug latente** que existia desde la #24.

## El hallazgo mas importante

El chequeo de convenciones **moria en silencio** desde la feature #24 cada vez
que el `fn` mas cercano no estaba al tope del archivo: `set -o pipefail` mas un
`grep` sin coincidencias mataban el script antes de reportar.

Nunca se noto porque en `rust/tests/` los `fn` estan al tope. Aparecio al
ampliar el alcance a `rust/src/`, donde los tests viven indentados.

Es la tercera cara del mismo error que este repo ya encontro dos veces: en la #23
un `cargo test` que salia 0 sin correr nada, en la #25 un `[ok]` del hub que solo
medía TCP, y ahora un chequeo que moria antes de hablar. **Las tres veces el
instrumento decia "todo bien" sin haber mirado.**

## Lo que verifique ademas de los AC

- **Las seis deudas, una por una, contra el impl que las anoto.** La primera ya
  tenia la nota mal: decia "1 / 1 / 2" y era "1 / 2 / 2".
- **El gate de spec rechazando el cierre de las seis entradas.** Correcto: nunca
  tuvieron spec. Se cerraron como `blocked` con la nota. `done` habria sido
  mentira.
- **`leccion list` contra el catalogo real**: nueve lecciones, columnas
  alineadas, el nombre de 39 caracteres ya no desborda.
- **Ningun cambio de contrato accidental**: `--json` de `leccion list` intacto,
  orden por uso intacto.

## Observaciones (no bloquean)

1. **`blocked` no es exactamente "absorbida".** El arnes no tiene ese estado y la
   nota lo compensa, pero en `status` esas seis se ven como bloqueadas, que
   sugiere un problema donde no lo hay. Candidato a feature: un estado
   `superseded`.
2. **El AC-14 es disciplina, no gate.** El rol dice que las notas de backlog
   entren al backlog; nada lo verifica. Automatizarlo exigiria parsear los impl,
   que es fragil. Queda dicho como disciplina, en vez de fingirse garantia.
3. **`apunta_al_runtime` mira texto, no parsea JSON/TOML.** Los cinco backends
   usan formatos distintos y lo unico que importa es si el arnes esta cableado.
   Un settings que mencione el runtime en un comentario pasaria.

## Riesgo que queda vivo

Que la proxima seccion "Para el backlog" vuelva a quedarse en prosa. El rol lo
pide, el reviewer puede exigirlo, y nada lo obliga. Es el mismo tipo de deuda que
esta feature acaba de pagar — con la diferencia de que ahora esta escrito donde
alguien lo lee antes de cerrar, y no solo despues.
