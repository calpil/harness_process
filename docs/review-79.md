# Review - Feature #79: close refresca el espejo del backlog y de la bitacora
Revisado: approved · 2026-09-07T03:08:55Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento, mas tres agentes de revision
adversarial (ultracode) con lentes distintas —spec, casos borde, docs—, solo
lectura, sin correr nada. Metodo propio: siete tests de integracion contra HEAD
leyendo por que cayo cada uno; dos mutantes con `cmp` y `--no-fail-fast`;
suite, clippy, paridad y `verify` antes y despues de aplicar los hallazgos.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/espejo.rs:101 · rust/tests/cli_basics.rs:8909 | CUBIERTO. Rojo contra HEAD por stdout sin la frase; con el fix el espejo es byte-identico al backlog cerrado. El refresco esta despues de `save_features` y de `log` (close.rs:345) y relee del disco, no de la memoria. |
| AC-2 | rust/tests/cli_basics.rs:8930 | CUBIERTO. `blocked` refresca. |
| AC-3 | rust/src/espejo.rs:80 · rust/tests/cli_basics.rs:8944 | CUBIERTO. Pasaba contra HEAD (no pasa nada) y ahora ademas exige stderr sin `[!]`: asi el test de integracion mata al mutante `Auto => true`, que antes solo mataba el unitario. |
| AC-4 | rust/src/espejo.rs:112 · rust/tests/cli_basics.rs:8956 | CUBIERTO. Mutante `Siempre => existe_dir` cae en el unitario y en el de integracion. |
| AC-5 | rust/tests/cli_basics.rs:8977 | CUBIERTO. `false` deja los dos archivos viejos byte-identicos. |
| AC-6 | rust/src/espejo.rs:52 · :80 · :146-186 | CUBIERTO. Seis combinaciones en cuatro unitarios. |
| AC-7 | rust/src/espejo.rs:42 · rust/src/commands/close.rs:362 · rust/tests/cli_basics.rs:8997 | CUBIERTO, y mejor que el spec tras la revision: fallo por archivo, las dos copias se intentan siempre, stdout nombra lo escrito. Unitario del fallo parcial con byte no UTF-8 (rust/src/espejo.rs:189). |
| AC-8 | README.md:1225 · docs/architecture.md:55 · UPDATING.md:48 | CUBIERTO tras dos hallazgos de la lente de docs: README sin el momento, architecture sin "sin commitear". Las dos copias de UPDATING identicas (`cmp`). |
| AC-9 | rust/src/features.rs:42 | CUBIERTO. Suite, clippy `-D warnings`, paridad 10/10; `verify` 10/10 (docs/verify-79.md). |
| AC-10 | rust/tests/cli_basics.rs:9011 | CUBIERTO. La bitacora espejada lleva la linea `close feature #1 status=done` de ese mismo cierre. |

## Lo que el review tiene que decir

1. **La revision adversarial pago.** Seis hallazgos, ninguno bloqueante, tres
   de codigo que yo no habia visto: el `?` que perdia la copia buena cuando
   fallaba la segunda, `read_to_string` en una copia que el spec llama "byte a
   byte", y los separadores de Windows en los mensajes que spec, README y
   UPDATING prometen con `/`. Los tres se arreglaron con un unitario cada uno.
   Costo: tres agentes, 330k tokens, seis minutos; sin `cargo` en ellos para
   no pelear el `target/` con la suite.
2. **Un mutante que el test de integracion no mataba.** `Auto => true` solo
   caia en el unitario: el mutante no crea el directorio, falla al escribir y
   lo AVISA, y el test de AC-3 miraba stdout y el directorio, no stderr. El
   aserto de "stderr limpio" es lo que convierte "no paso nada" en algo
   medible. Va a la leccion `criterios-de-cierre-que-se-pueden-fallar`.
3. **`--no-fail-fast` en las mutaciones.** Cargo se detiene en el primer
   binario de tests que falla; en la #74 eso escondio si los tests de
   integracion mataban al mutante. Aca se corrieron los dos binarios.

## Riesgo declarado

- El espejo queda sin commitear en la raiz; con `docs/` como repo aparte,
  queda modificado en ESE repo. Es lo mismo que la bitacora del PRD y el
  sello, y el mensaje lo dice; nadie lo commitea si el usuario no lo hace.
- Pre-existente, fuera de este diff (hallazgo de la lente de docs): el
  instalador (#78) etiqueta `docs/bkp-backlog/history.md` como "(no es un
  backlog JSON)" al nombrar los respaldos, porque cuenta features con
  `json.load` sobre cualquier candidato. Candidato a feature chica.

## Veredicto

Diez AC con cobertura. Siete tests de integracion (cinco rojos contra HEAD
por su aserto, dos guardas de "no paso nada" reforzadas), ocho unitarios, dos
mutantes muertos por unitario e integracion, seis hallazgos de la revision
adversarial verificados y resueltos (cinco en el diff, uno registrado como
pre-existente).
