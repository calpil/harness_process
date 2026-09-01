# Review — feature #67: los dos parsers del review no se contradicen
Revisado: approved · 2026-09-01T22:16:55Z · estampado por `harness revision --veredicto`

Revision adversarial. El mandato fue intentar ROMPER, no confirmar.

| AC | evidencia | veredicto |
| --- | --- | --- |
| AC-1 | rust/tests/cli_basics.rs:7138 | cubierto |
| AC-2 | rust/tests/cli_basics.rs:7185 | cubierto |
| AC-3 | rust/tests/cli_basics.rs:7218 | cubierto |
| AC-4 | rust/src/markdown.rs:356 | cubierto, reescrito tras la observacion |
| AC-5 | rust/src/verificacion.rs:1014 | cubierto con alcance declarado |
| AC-6 | rust/src/revision.rs:1021 | cubierto |
| AC-7 | rust/src/revision.rs:1045 | cubierto |
| AC-8 | rust/src/revision.rs:1071 | cubierto |
| AC-9 | rust/src/revision.rs:1103 | cubierto |
| AC-10 | tests/conventions_check.sh:150 | cubierto |
| AC-11 | rust/src/commands/revision.rs:186 | cubierto (manual) |
| AC-12 | rust/tests/cli_basics.rs:7268 | cubierto |

## Lo que se rompio

**1. El limpiador y el gate no coincidian en que es un sello.** `cuerpo_sin_sellos`
borraba cualquier linea de afuera que empezara con `Revisado:`; el gate exigia
ademas un veredicto valido. Una linea de prosa del reviewer —"Revisado: el parser
unico esta bien resuelto, pero el tope miente"— desaparecia del archivo al
estampar, sin aviso. Reproducido con un test que salio rojo antes de tocar nada.

Es la misma falla que la feature vino a cerrar, un nivel mas abajo que los
fences: dos partes de la misma maquinaria que no coinciden en QUE ES un sello.
Arreglado con un solo predicado, `revision::veredicto_de_sello` (rust/src/revision.rs:432),
que usan los dos lados. Dos tests cubren las dos mitades, porque un predicado mas
estricto no puede costar la idempotencia de `estampar`.

**2. El bloque indentado ejecutaba documentacion.** Markdown tiene DOS formas de
bloque de codigo y los cuatro parsers viejos conocian solo la cercada. Confirmado
con el binario, no con una sonda: un `Comando:` escrito con 4 espacios de sangria
se ejecutaba —`verify` reportaba el AC-99 documentado y el archivo aparecia
escrito— y un sello citado asi se leia como veredicto.

Es el mismo daño que el bug de `~~~` con la otra sintaxis. No aparecio antes
porque **no era una divergencia entre parsers**: los cuatro compartian el hueco,
asi que mirar en que se contradecian no lo mostraba. Se cerro con una regla
estrecha en el parser unico (rust/src/markdown.rs:116) y costo medido cero en el
corpus real (0 de 733 AC, 0 de 1346 filas de review, 0 lineas `Comando:`). Es el
AC-12, agregado al spec y re-aprobado por el usuario.

## Lo que aguanto

**Los bordes del parser.** Se sondearon once formas contra el clasificador, cada
una con UN SOLO sello, el que el autor escribio como documentacion, para ver si el
gate lo lee como veredicto o si `verify` lo ejecuta:

| forma | ¿el gate lo lee? |
| --- | --- |
| fence con info-string (```rust) | no |
| 4 backticks abren, 3 cierran | no |
| 3 abren, 4 cierran | no |
| cita completa con `>` en todas las lineas | no |
| cita sin fence | no |
| bloque anidado en una lista | no |
| fence indentado 4 | no |
| bloque sin cerrar | no |
| CRLF | no |
| `~~~` adentro de un bloque ``` | no |
| **indentado 4 sin fence** | **si — era el defecto 2, ya cerrado** |

Las primeras dos corridas de esta sonda estaban MAL CONSTRUIDAS y hay que decirlo:
la primera puso el `>` solo en las lineas de fence y no en el contenido, y las dos
primeras dejaban ademas un sello suelto al final de cada caso, que era el que el
gate estaba leyendo. Los resultados de arriba son de la tercera, aislada.

**Las citas del gate.** Se ataco `evaluar_cita` con symlink a archivo, symlink a
`/dev/zero`, symlink roto, FIFO, directorio, ruta absoluta, `..`, `sub/../real`,
linea 0, linea `usize::MAX` y archivo inexistente. **Todos NoResuelve, todos en
0 ms**: ninguno cuelga, ninguno agota memoria y —lo que importa para el gate—
**ninguno cuenta como cobertura de un AC**. El symlink a un archivo real si
resuelve, que es correcto: es un archivo real.

`Cita::NoSePudoComprobar` NO cuenta como cobertura: `fila_responde` exige
`== Cita::Resuelve`. Verificado en el codigo, no asumido.

## Lo que quedo sin cubrir, y por que

- **El largo del fence estilo CommonMark.** Un bloque abierto con 4 backticks se
  cierra con 3. Los cuatro parsers compartian la divergencia, no producia
  desacuerdo entre ellos y no se pudo construir un caso donde haga que el gate
  acepte un sello ajeno o que `verify` ejecute documentacion. Declarado en
  `markdown.rs` como no tocado a proposito.
- **`sub/../real.txt` se rechaza** aunque resuelva adentro de la raiz. Es
  conservador de mas, no un defecto: rechazar de mas no deja pasar un review vacio.
- **Las reglas de CommonMark sobre interrumpir parrafos** para el bloque
  indentado. La regla puesta es "4 espacios y nada mas". Con el corpus actual da
  el mismo resultado; un documento futuro que indente una continuacion de parrafo
  a 4 espacios la veria como codigo.

## Sobre el AC-5, que no se puede fallar

`impl-67.md` declara que `corpus_real_sin_cambios` NO se pone rojo revirtiendo el
arreglo de `~~~`, porque viejo y nuevo vuelven a coincidir. **La declaracion es
honesta y es completa**: se reviso el resto de los AC y ninguno mas esta en esa
situacion. Los otros once tienen mutacion que los pone rojos, verificada una por
una. El AC-5 igual vale: se pone rojo ante un cambio futuro del parser que si
altere lo que significan los specs reales, comprobado con la mutacion que le saca
el fence a los backticks.
