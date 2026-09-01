# Implementacion — feature #68: el arnes no pierde los AC que pide revisar a mano

## Lo que se encontro al medir

El backlog decia cuatro AC perdidos. Son **siete**, en dos familias que se
pierden por la MISMA linea de codigo:

| forma | specs | cuantos |
| --- | --- | --- |
| `- AC-n (MANUAL):` | #64, #65, #66, #67 | 4 |
| `- AC-4b:`, `- AC-12b:`, `- AC-12c:` | #16, #51 | 3 |

`ac_de` exigia los dos puntos **pegados** a los digitos. La anotacion `(MANUAL)`
—que existe justo para marcar "esto lo tiene que mirar una persona"— era lo que
hacia que el arnes no se lo pidiera a nadie: ni `verify`, que lo dejaba fuera de
su conteo, ni el gate del review, que saca su lista del mismo parser.

## El segundo daño, que no estaba en el backlog

`parsear` cuelga cada `Comando:` del ultimo AC abierto. Si un AC no reconocido
trae comando, **ese comando se le adjudica al AC anterior**. Reproducido con el
parser real antes de tocar nada:

```
- AC-1: uno
- AC-2 (MANUAL): dos
  Comando: `touch MAL.txt`
```
=> `AC-1 -> comando=Some("touch MAL.txt")`

`verify` habria impreso **"AC-1 verde"** tras correr la prueba de otro criterio:
un verde atribuido al AC equivocado, que es la familia de la #44 y de
`docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`. Y si el AC anterior
ya tenia comando, el del invisible se descartaba en silencio: un criterio que
declara como se prueba y nunca se corre.

**Hay que decir cual es cual.** La desaparicion **esta pasando hoy** (siete AC).
La mala atribucion es **reproducible pero todavia no se disparo** en este repo:
se midieron los siete y ninguno declara `Comando:`. Se arregla igual, y la
diferencia se dice en vez de inflar el problema.

## Lo que se hizo

`ac_de` acepta un sufijo de letras y una anotacion entre parentesis. El sufijo SI
entra en el nombre (`AC-4b` es otro criterio que `AC-4`); la anotacion NO
(`- AC-11 (MANUAL):` es `AC-11`, porque el review lo cita por su numero).

Y `parsear` recuerda si vio una linea que **arranca** como AC pero no se puede
leer: mientras eso pasa, un `Comando:` no se le cuelga a nadie. Vale mas perder
un comando que adjudicarselo al criterio equivocado.

No se toco nada mas. La maquinaria de manuales ya existia entera y funciona
—`Estado::Manual`, el simbolo `[--]`, el conteo del resumen, `bloquea() == false`
para que no trabe el cierre—: lo unico roto era que el parser tiraba el AC antes
de llegar ahi.

## La feature se prueba sobre si misma

El AC-8 de este spec esta escrito **a proposito** con la forma que desaparecia
(`- AC-8 (MANUAL):`). Al correr el AC-6 —que asserta cuales AC trae el arreglo
sobre el corpus real— aparecio un **octavo**: el AC-8 de este mismo spec. El
codigo estaba bien; la lista esperada se habia escrito antes de que el spec
existiera en el corpus.

O sea que el AC-8 ya se cumplio antes de cerrarse: si el arreglo se revierte,
este AC se vuelve a perder y el AC-6 se pone rojo.

## Disciplina de test rojo

| mutacion | tests que se ponen rojos |
| --- | --- |
| vuelve el `ac_de` viejo | `el_ac_manual_aparece`, `el_gate_exige_fila_para_el_manual`, `el_sufijo_de_letra_es_un_ac_propio`, `la_anotacion_no_entra_en_el_nombre`, `los_siete_que_faltaban_y_ninguno_mas` |
| el sufijo de letra se descarta | `el_sufijo_de_letra_es_un_ac_propio`, `los_siete_que_faltaban_y_ninguno_mas` |
| el parser se afloja de mas (acepta cualquier cosa hasta `:`) | `lo_que_no_es_un_ac_sigue_sin_serlo` |
| el comando vuelve a migrar | `el_comando_no_migra_al_ac_anterior` |

**Una mutacion salio VERDE y la culpa era de la mutacion, no del test.** Cambiar
el `return None` del parentesis sin cerrar por dejar caer el caso da el MISMO
resultado, porque lo que queda tampoco empieza con `:`. Es una mutacion
equivalente: no hay test que distinga las dos versiones. El `return` se deja
escrito para que la intencion se lea, y el codigo **declara ahi mismo** que no es
una defensa independiente. Decirlo importa: un `return` que parece proteger algo
y no protege nada es la clase de cosa que despues alguien lee como cobertura.

## Lo que NO se toco, y por que

- **Reabrir features cerradas.** Los cuatro specs con AC manual estan cerrados y
  sus reviews no tienen fila para ellos. El gate corre al cerrar, y ya corrio. El
  AC-7 fija que ningun comando de solo lectura los mueva.
- **Avisar cuando una linea `- AC-` es ilegible.** Hoy se descarta sin una
  palabra. Es mejor que antes —no se la adjudica a otro AC— pero sigue siendo
  silencioso, y el repo no quiere cosas silenciosas. Queda declarado aca, fuera
  del alcance aprobado.
- **Inventar sintaxis de anotacion.** Se acepta lo que los specs YA usan.

## Efecto que conviene tener presente

El gate del review ahora **exige fila para los AC manuales**. Es lo buscado —el
AC-2 lo fija— pero revisar pasa a costar una fila mas por cada AC que el autor
marque como manual. Estaba declarado como OBS-2 en el spec aprobado.
