# Implementacion — feature #69: una linea AC ilegible no desaparece en silencio

## De donde sale

La #68 arreglo la mitad peor de este problema —el `Comando:` de una linea
ilegible se le adjudicaba al AC de arriba, y `verify` imprimia un verde contra el
criterio equivocado— y **declaro la otra mitad como limite conocido**: la linea se
seguia tirando sin decir una palabra. Un typo en el encabezado hacia desaparecer
un criterio y nadie se enteraba.

## Lo que se hizo

`parsear` ya evaluaba la condicion, pero moria adentro de la funcion. Se extrae a
`verificacion::lineas_ac_ilegibles`, pura como su hermana, y la usan dos
consumidores:

- **`verify`** imprime un `[!]` con el texto ENTERO de la linea, antes de correr
  nada — donde el autor esta mirando y el error todavia es barato— y sigue
  verificando el resto: cortar ahi le quitaria el resultado de los AC que si
  estan bien.
- **El gate del review** se niega. Un review no puede cubrir un criterio que el
  arnes no leyo.

Medido antes de escribirlo: **cero lineas ilegibles** en los 55 specs reales. El
arreglo no cambia nada de lo que hay; se pone en medio del proximo typo.

## Lo que encontro el test del AC-2

El gate del cierre rechazaba el spec con AC ilegible y **`revision --veredicto` lo
estampaba igual**. Eran dos sitios haciendo la misma pregunta con su propia copia:
`revision::gate` y `commands::revision::estampar` tenian cada uno su
`if parsear(&spec).is_empty()`.

O sea que el sello podia decir `approved` sobre una cobertura que el cierre no iba
a aceptar. Es el patron que cerro la #67 —un formato, un parser— aplicado a una
decision: **una pregunta, una respuesta**. Ahora las dos llaman a
`revision::motivo_spec_inservible`.

Lo encontro el test, no una lectura del codigo: la primera corrida del AC-2 salio
`Unexpected success`.

## Disciplina de test rojo

| mutacion | tests que se ponen rojos |
| --- | --- |
| no detecta ilegibles | `verify_nombra_la_linea_ilegible`, `el_gate_se_niega_con_un_ac_ilegible`, `no_hay_falsos_ilegibles` |
| avisa de todo (`- AC` en vez de `- AC-`) | `no_hay_falsos_ilegibles` |
| el bloque de codigo cuenta | `el_bloque_de_codigo_no_dispara_el_aviso` |
| `estampar` vuelve a su copia | `el_gate_se_niega_con_un_ac_ilegible` |

`el_corpus_real_no_tiene_ilegibles` **no** detecta la mutacion "avisa de todo", y
esta bien que no: ningun spec real tiene una linea `- AC` que no sea `- AC-`, asi
que el filtro mas ancho tambien encuentra cero. Ese frente lo cubre el AC-3. Se
dice para que no se lea como cobertura que no es.

## Lo que NO se toco

- **Adivinar que quiso escribir el autor.** El arnes nombra la linea; corregirla
  es de la persona.
- **Un sexto gate en `close`.** Se llega por el del review, que es obligatorio con
  `require_review`. Con esa regla apagada, un spec con AC ilegible cierra igual —
  declarado como OBS-1 en el spec aprobado.
- **Las lineas ilegibles de un review.** El review no declara AC; los cita.
