# Review - Feature #74: add no protege contra la feature duplicada
Revisado: approved · 2026-09-07T02:47:02Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento, y la que cerro esta feature `blocked`
el dia anterior por falta de evidencia. Metodo: medir de nuevo el backlog real
antes de escribir; correr los cinco tests contra HEAD y leer por que cayo cada
uno; dos mutantes con `cmp`; suite, clippy y paridad.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/duplicados.rs:79 · rust/tests/cli_basics.rs:8769 · :8792 | CUBIERTO. Rojo contra HEAD por `code=0`; con el fix exit 2, backlog e history byte-identicos. `blocked` cuenta como abierta y tiene su test propio; el mutante que la saca de `ABIERTAS` tumba el unitario que la prefiere. |
| AC-2 | rust/src/commands/add.rs:70 · rust/tests/cli_basics.rs:8818 | CUBIERTO. Rojo contra HEAD por stderr sin `[i]`; con el fix crea la #2 y cita la #1 `done` con fecha. |
| AC-3 | rust/src/duplicados.rs:109 · rust/tests/cli_basics.rs:8845 | CUBIERTO. Rojo contra HEAD por `unexpected argument '--clave'` (precondicion, pero es el flag que la feature agrega). Con el fix: segunda corrida exit 0, `ya existe`, nada escrito. |
| AC-4 | rust/src/duplicados.rs:36 · :126 | CUBIERTO. Tres unitarios; el mutante sin `to_lowercase()` tumba el de mayusculas/acentos. |
| AC-5 | rust/tests/cli_basics.rs:8869 | CUBIERTO. Pasaba contra HEAD y sigue pasando: es la guarda de que `clave` no se cuela cuando no viene. |
| AC-6 | README.md:1216 · UPDATING.md:48 | CUBIERTO. `cmp` limpio entre las dos copias de UPDATING; `--help` documenta `--clave`. |
| AC-7 | rust/src/commands/add.rs:16 | CUBIERTO. 483 + 273 tests, clippy `-D warnings` limpio, paridad 10/10. `verify`: ver docs/verify-74.md. |

## Lo que el review tiene que decir

1. **La medicion no cambio; la decision si.** El backlog real sigue con cero
   duplicados. Lo que se implementa es una defensa para un escenario, no un
   arreglo para un dato, y el spec lo deja escrito en la primera tabla. Si en
   seis meses el rechazo nunca salto, esta feature costo un modulo de 115
   lineas y un flag; si salto una vez, evito una feature fantasma en el
   backlog y en Jira.
2. **Sin escape por flag.** El rechazo del duplicado abierto no tiene
   `--forzar`. Es la misma decision que el tope de lecciones (#80, OBS-4) y va
   contra la propuesta original del analisis de Hermes ("dedupe por nombre con
   aviso"): un aviso que no bloquea es lo que ya pasaba con `ninguna` antes de
   exigirle motivo. La salida legitima existe y es mejor: un nombre que diga en
   que se diferencia.
3. **Las mutaciones se probaron contra los unitarios.** Cargo se detiene en el
   primer binario de tests que falla; con cada mutante cayo el unitario y los
   de integracion no llegaron a correr. Es evidencia suficiente de que el
   mutante se detecta, pero no de que lo detecte el test de integracion: queda
   dicho, no disimulado.

## Riesgo declarado

- `VACIAS` (21 palabras) es un criterio: dos nombres que solo difieren en una
  de esas palabras cuentan como iguales. La lista es corta y esta documentada
  como decision en el modulo; agregarle palabras cambia que es "el mismo
  nombre" y merece su propio cambio.
- La tabla de acentos es fija (vocales, `ñ`, `ç`); un nombre con otros
  diacriticos se compara con ellos puestos. Para un backlog en castellano
  alcanza.

## Veredicto

Siete AC con cobertura. Cinco tests de integracion (cuatro rojos contra HEAD
por su aserto, uno guarda de regresion), siete unitarios, dos mutantes muertos,
suite y clippy limpios, docs en las dos copias.
