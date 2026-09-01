# Implementacion — feature #65: el arnes cierra lo resuelto aguas arriba

## El caso que la motivo

El backlog tenia dos features (`#91`, `#92`) que son del arnes y no del repo que
las anota. Con los estados que habia, la unica salida honesta era `blocked` y
esperar: `done` afirma que se resolvio aca, `superseded` afirma que ya no hace
falta. Las dos son falsas.

## Lo que se hizo

Un tercer estado, `resuelto-aguas-arriba`, que **exige** `--resuelto-en
<proyecto>/feature-<id>` y guarda la referencia en el backlog.

De esa referencia se comprueba **la forma y nada mas**. `status` lo dice literal:
*"resuelto aguas arriba en `<ref>`, sin verificar"*. El arnes no puede abrir el
repo vecino ni leerle el backlog; fingir que valido la existencia seria
exactamente lo que la #64 prohibio —prometer enforcement que no se hace— y lo que
la #63 vino a cerrar.

Por la misma razon **no cuenta como resuelto en el avance del PRD**: sale del
numerador y del denominador, igual que `superseded`. Contarlo como hecho seria
inflar el avance con trabajo que no se hizo aca; dejarlo en el denominador
—lo que hacia `blocked`— condena al PRD a no llegar nunca al 100%.

En Atlassian **no dispara transicion** y deja un comentario con la referencia.
Mover el ticket seria afirmar aguas afuera lo mismo que no se puede comprobar
adentro.

La cabecera de `status` pasa a enumerar los seis estados con `otros=N`. Antes
mostraba tres y los demas desaparecian sin dejar rastro — que es exactamente como
este estado nuevo se habria vuelto invisible.

## Lo que corrigio la medicion, contra lo que yo habia escrito

En el spec escribi que `--note` "no es trazable". **Es falso**: medido, la nota
queda en cuatro lugares. La razon real para un estado propio es otra, y es la que
vale: `status`, el avance del PRD y Atlassian toman decisiones **distintas** segun
el estado, y ninguna de las tres lee las notas.

## Lo que encontro la prueba del rojo

Aca esta lo importante de esta feature, y no es el estado nuevo.

**El test del AC-9 no podia fallar.** `todos_los_estados_tienen_su_rama`
asertaba que cada estado no era la cadena vacia y que `AGUAS_ARRIBA ==
"resuelto-aguas-arriba"` —una constante igual a su propio literal—. El AC-9 dice
"los cinco sitios que comparan el literal del estado tienen su rama"; el test no
comprobaba nada de eso. Se descubrio al mutar: **borrar la rama de produccion de
Atlassian lo dejaba verde**.

**El test del AC-8 medía una copia.** `aguas_arriba_no_reabre_el_ticket` llamaba
a un `transicion_de` definido **dentro de `mod tests`**: una reimplementacion de
la tabla de produccion. Borrar la rama de produccion no lo movia. Es la misma
trampa que el cross-check de `verificacion.rs` en la #67 —dos instrumentos que se
copian coinciden siempre y no miden nada—, encontrada el mismo dia en dos
features distintas.

### El arreglo

Las decisiones dejan de estar inline y pasan a ser produccion consultable:

| consumidor | ahora |
| --- | --- |
| `cli.rs` | `close::ESTADOS_DE_CIERRE`, una sola lista |
| `atlassian/emit.rs` | `efecto_de(status) -> Efecto`, y `on_close` la usa |
| `prd.rs` | `cuenta_en_el_avance(status) -> bool` |
| `commands/status.rs` | `ESTADOS_CON_BUCKET` |
| `commands/close.rs` | el gate de `--resuelto-en` |

Y el test del AC-9 recorre la **tabla completa** —cinco estados x cuatro
decisiones— contra esos consumidores, mas un estado inventado para comprobar que
la tabla mide algo: cae en el brazo por defecto de todos, que es justo el
comportamiento peligroso del que hay que salvarse explicitamente.

Un estado nuevo que no se agregue a la tabla **no compila**; uno que se agregue
sin decidir que hace cada consumidor, **falla**.

## Disciplina de test rojo

Cada AC se comprobo ROJO revirtiendo lo que arregla:

| mutacion | tests que se ponen rojos |
| --- | --- |
| el estado no existe en el CLI | `resuelto_aguas_arriba` (AC-1) |
| se saca el gate de `--resuelto-en` | `aguas_arriba_exige_referencia` (AC-2) |
| la forma se acepta siempre | `forma_de_la_referencia_externa` (AC-3) |
| `close` valida la existencia | `referencia_externa_no_se_valida` (AC-4) |
| `status` no dice donde se resolvio | `status_muestra_aguas_arriba` (AC-5) |
| la cabecera pierde el estado nuevo | `cabecera_de_status_suma` (AC-6) |
| cuenta como resuelto en el PRD | `prd_tree_ignora_aguas_arriba` (AC-7), `todos_los_estados_tienen_su_rama` |
| Atlassian transiciona igual | `aguas_arriba_no_reabre_el_ticket` (AC-8), `todos_los_estados_tienen_su_rama` (AC-9) |
| el estado pierde su bucket en `status` | `todos_los_estados_tienen_su_rama` (AC-9) |
| `close` menciona I/O de red | `cierre_sin_io_de_red` (AC-10) |

Las dos ultimas filas del AC-9 son las que antes salian **verdes**.

## Observacion que queda abierta

`verify` reporta "10 AC con comando declarado (10 en total)" sobre un spec que
tiene **once**. El AC-11 esta escrito `- AC-11 (MANUAL): ...` y `ac_de` exige
`- AC-<digitos>:`, asi que el AC MANUAL es **invisible** para `verify` y tambien
para el gate del review, que saca su lista del mismo parser. O sea: el AC que
explicitamente pide que lo mire una persona es justo el que el arnes no le exige
a nadie. Pasa en los specs #64, #65, #66 y #67.

No se toca aca —es otro defecto, no el de esta feature— y queda para que el
usuario decida si va al backlog.
