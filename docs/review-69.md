# Review — feature #69: una linea AC ilegible no desaparece en silencio

Revision adversarial sobre el propio trabajo. El mandato fue romperlo.

| AC | evidencia | veredicto |
| --- | --- | --- |
| AC-1 | rust/tests/cli_basics.rs:7641 | cubierto |
| AC-2 | rust/tests/cli_basics.rs:7669 | cubierto, encontro un defecto |
| AC-3 | rust/src/verificacion.rs:991 | cubierto |
| AC-4 | rust/src/verificacion.rs:1032 | cubierto con sensibilidad declarada |
| AC-5 | rust/src/verificacion.rs:1021 | cubierto |

## Lo que se rompio

**El gate del cierre y `estampar` no coincidian.** El gate rechazaba un spec con
un AC ilegible; `revision --veredicto` lo estampaba igual. Cada uno tenia su
propia copia de la pregunta ("¿este spec sirve para medir cobertura?"), un
`if parsear(&spec).is_empty()` por lado.

Consecuencia: el sello podia decir `approved` sobre una cobertura que el cierre no
iba a aceptar — el comando afirmando una cosa y el gate la contraria sobre el
mismo archivo, que es lo que la #63 vino a cerrar.

Lo encontro **el test, no una lectura del codigo**: la primera corrida del AC-2
salio `Unexpected success`. Arreglado con `revision::motivo_spec_inservible`
(rust/src/revision.rs:654), que ahora contesta para los dos. Es el patron de la
#67 —un formato, un parser— aplicado a una decision: una pregunta, una respuesta.

## Lo que aguanto

**No hay falsos positivos.** Se probaron las formas que SI son AC (`- AC-1:`,
`- AC-4b:`, `- AC-11 (MANUAL):`, anotacion larga) y prosa que arranca parecido
(`- ACR-1:`, `- Alcance:`, `- AC de la feature:`, filas de tabla `| AC-1 |`).
Ninguna dispara el aviso. Importa: un aviso que salta siempre se deja de leer, y
entonces el dia que importa nadie lo mira.

**Los bloques de codigo no disparan.** Un spec que documenta la forma de un AC
escribe ejemplos rotos a proposito. Sale gratis por usar el parser unico de la
#67, y queda fijado para que no se pierda si alguien lo toca (AC-5).

**El aviso nombra la linea entera.** Sin su texto, obligaria a buscarla.

**`verify` no corta.** Avisa y sigue: el autor conserva el resultado de los AC que
si estan bien.

## Sensibilidad del AC-4, declarada

`el_corpus_real_no_tiene_ilegibles` **no** se pone rojo ante la mutacion que
afloja el filtro de `- AC-` a `- AC`. Es correcto que no: ningun spec real tiene
una linea `- AC` que no sea `- AC-`, asi que el filtro mas ancho tambien encuentra
cero. Ese frente lo cubre el AC-3, que si se pone rojo. Se dice para que el verde
del AC-4 no se lea como una cobertura que no es.

Lo que el AC-4 si prueba: que el arreglo no cambia nada de lo que existe hoy, y
que si manana alguien escribe un AC ilegible, la corrida siguiente lo agarra.

## Lo que quedo abierto, con nombre

- **Con `require_review` apagada, un spec con AC ilegible cierra igual.** No hay
  gate propio en `close`: se llega por el del review. Estaba declarado como OBS-1
  en el spec aprobado y se decidio no agregar un sexto gate.
- **El arnes no adivina el typo.** Nombra la linea; corregirla es de la persona.
