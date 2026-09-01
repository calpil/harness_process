# Implementacion — feature #67: los dos parsers del review no se contradicen

## Lo que se encontro

El titulo se quedo corto. No eran dos parsers, eran **cuatro**, con **tres**
semanticas distintas sobre los mismos documentos:

| donde | semantica |
| --- | --- |
| `revision::lineas_fuera_de_bloque` (el gate del review) | fences emparejados |
| `commands::revision` (el limpiador de `estampar`) | toggle con cualquier fence |
| `verificacion::parsear` (los AC del spec) | toggle solo con ` ``` ` |
| `atlassian::markdown` | CONSUME el bloque (caso distinto, queda aparte) |

Y el mas caro no era el que motivo la feature.

### El bug que no estaba en el spec original

`verificacion::parsear` no conocia `~~~`, asi que **`verify` ejecutaba los
`Comando:` escritos dentro de un bloque de tildes**. Reproducido antes de tocar
nada: un `- AC-99` dentro de un bloque `~~~` salia `[ok] verde` y su comando
escribia el archivo.

Es literalmente el bug que la #23 cerro para backticks —*"un spec que documenta
la sintaxis no puede quedar verificando su documentacion"*— **abierto para
tildes**. Ejecucion de shell salida de una seccion que el autor marco como
documentacion.

Agravante: el test que deberia haberlo detectado
(`parse_should_only_report_commands_the_spec_actually_declares`) tenia su propio
cross-check con **el mismo punto ciego**. Su acuerdo sobre 20+ specs no
significaba lo que su comentario decia: dos instrumentos mal calibrados de la
misma forma coinciden perfectamente y no miden nada. El cross-check ahora saca la
clasificacion de bloques del parser unico y cuenta los AC por otro camino.

### Lo que el reviewer si habia visto

El limpiador de `estampar` borraba prosa del reviewer o dejaba dos sellos
contradictorios, segun la **paridad** de fences ajenos citados. La paridad quedo
**al reves** de como el reviewer la describio; verificado con el binario real:
con UNA linea `~~~` la cita sobrevive, con DOS se borra.

## Lo que se hizo

`rust/src/markdown.rs` (nuevo) es el unico parser. Devuelve la clasificacion
completa (`Fuera` | `Fence` | `Dentro`) y no una lista ya filtrada, porque cada
consumidor necesita algo distinto de la **misma** respuesta —el gate quiere lo de
afuera, el limpiador necesita todas las lineas para reescribir el archivo
conservando los fences, el parseo de AC quiere todo lo que no sea contenido—. Un
`Vec<&str>` compartido no alcanzaba, y eso fue exactamente lo que hizo que cada
uno se escribiera el suyo.

Semantica: fences **emparejados** (se recuerda cual abrio). Es la unica de las
tres que coincide con como se renderiza el markdown, y la que hace que un review
ajeno citado entero no rompa nada.

Ademas, en las mismas funciones:

- **El tope de lectura deja de mentir.** Devolvia "la linea no existe" sobre
  citas correctas cuya linea caia mas alla del tope; la linea existia y `sed` la
  mostraba. Ahora hay una tercera respuesta, `NoSePudoComprobar`. Es el patron
  127-vs-124 de `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`.
  El tope se conserva: sacarlo cuesta 10,5 s por 2 GB dentro de un gate sin
  timeout.
- **Off-by-one del EOF.** Un archivo de N lineas terminado en salto tiene N
  saltos, no N+1: la cita a la linea N+1 resolvia en cualquier archivo normal.
  Reproducido con `evidencia.txt:4` en un archivo de 3 lineas.
- **El `?` de `veredicto_estampado`** abortaba la funcion entera en la primera
  linea `Revisado:` pelada, y el gate decia "no lleva el sello" con el sello tres
  lineas abajo.
- **`estampar` invierte el orden** (AC-11): comprueba con el parser del gate y
  despues escribe. El chequeo ya operaba sobre la cadena en memoria, asi que
  correrlo primero no pierde nada y saca la escritura de en medio.

`tests/conventions_check.sh` gana el modo `un-solo-parser`, que impide un quinto.
En su primera corrida encontro **dos que yo habia dejado sin unificar**
(`verificacion.rs` cross-check y `commands/revision.rs` `arranca_en_bloque`).

## Lo que encontraron los tests al escribirse

- **`contar_hasta` no podia confirmar ni la linea 1** de un archivo de una sola
  linea larga: exigia ver el salto FINAL de la linea citada. O sea, el mismo
  error que el AC-6 arregla —reportar "no pude" sobre algo que si se puede— un
  paso antes. Lo encontro el test del AC-6 al escribirlo, no una revision.
- **El "37% de los documentos de 7 lineas" del spec no se reproduce.** Medido por
  enumeracion exhaustiva: a n=7 los tres parsers discrepan en **86,6%** de los
  documentos, y en **44,7%** la discrepancia cae sobre una linea de sello (que es
  la que puede cambiar una decision). El 37,8% es la cifra de **n=6**. El numero
  del spec estaba mal etiquetado; el test fija los conteos medidos.
- **Mi propia asercion de "sellos en columna 0" era la equivocada**, no el
  codigo: el sello citado dentro del bloque tambien empieza en columna 0.

## Que verifica el AC-4, ahora que todos usan el mismo parser

Se aviso antes de implementarlo: comparar "los cuatro consumidores" cuando todos
llaman a la misma funcion es una **tautologia**. El test reescrito enumera
exhaustivamente (97.655 documentos, n=1..7 sobre cinco simbolos) y asserta que lo
que cada consumidor **observa** se deriva de la clasificacion del parser unico:

- `verificacion::parsear` encuentra exactamente los AC de las lineas `Fuera`
- `veredicto_estampado` ve sello si y solo si hay una linea de sello `Fuera`
- el limpiador saca exactamente los sellos de `Fuera` y no toca nada mas

Eso **si** se pone rojo si alguien vuelve a escribir un parser local en cualquiera
de los tres, aunque el grep del AC-10 no lo agarre. Verificado: la mutacion que
devuelve `parsear` a solo-backticks lo pone rojo.

Las tres semanticas viejas quedan reimplementadas en el modulo de tests, a
proposito: sin ellas el numero que motivo la feature no se puede reproducir.

## Disciplina de test rojo

Cada test se comprobo ROJO revirtiendo lo que arregla. La primera corrida del
arnes de mutacion dio **falsos verdes**: `cargo test -- --exact <nombre>` sin el
modulo no matchea nada, corre 0 tests y no imprime `FAILED`. Se arreglo el arnes
para exigir que el test haya corrido.

| mutacion | tests que se ponen rojos |
| --- | --- |
| `parsear` vuelve a ignorar `~~~` | `verify_no_ejecuta_documentacion`, `los_parsers_no_discrepan` |
| el limpiador vuelve a togglear con cualquier fence | `estampar_no_toca_la_prosa`, `estampar_deja_un_solo_sello`, `los_parsers_no_discrepan` |
| los backticks dejan de ser fence | `corpus_real_sin_cambios`, `la_divergencia_...` |
| el tope vuelve a decir "no existe" | `cita_grande_no_se_pudo_comprobar` |
| se saca el tope de lectura | `cita_grande_no_cuelga_el_cierre`, `cita_grande_no_se_pudo_comprobar` |
| vuelve el off-by-one del EOF | `la_cita_no_acepta_la_linea_siguiente_al_eof` |
| vuelve el `?` de `veredicto_estampado` | `el_sello_se_encuentra_aunque_haya_lineas_peladas` |

## Lo que encontro la revision adversarial

**El limpiador de `estampar` borraba prosa del reviewer.** Usaba
`starts_with(SELLO_REVIEW)` —o sea, cualquier linea de afuera que empezara con
`Revisado:`— mientras el gate exigia ademas un veredicto valido. Una linea de
prosa como *"Revisado: el parser unico esta bien resuelto, pero el tope miente"*
desaparecia del archivo al estampar, **sin aviso**.

Es la misma falla que el resto de la feature, un nivel mas abajo que los fences:
dos partes de la misma maquinaria que no coinciden en **que es un sello**. El
arreglo es el mismo que el del parser — un solo predicado,
`revision::veredicto_de_sello`, que usan los dos lados.

Los dos tests que quedan cubren las dos mitades: que la prosa sobreviva, y que el
sello de verdad se siga borrando. La segunda importa porque un predicado mas
estricto no puede costar la idempotencia de `estampar`: si el limpiador dejara de
sacar el sello anterior, quedarian DOS sellos contradictorios en el archivo.
Verificado ROJO en las dos direcciones.

Este defecto es **anterior** a la #67 —el limpiador viejo tenia el mismo
criterio— pero entra de lleno en el AC-2 ("la prosa del reviewer queda intacta"),
cuyo test solo cubria prosa dentro de bloques.

## El otro bloque de codigo que tiene markdown (AC-12)

Markdown tiene **dos** formas de bloque y los cuatro parsers viejos conocian solo
la cercada. Confirmado con el binario antes de tocar nada: un `Comando:` escrito
en un bloque **indentado con 4 espacios** se ejecutaba —`verify` reportaba el
AC-99 documentado y el archivo aparecia escrito— y un sello citado con esa
sangria se leia como veredicto.

Es el mismo daño que el bug de `~~~` con la otra sintaxis. No aparecio antes
porque **no era una divergencia entre parsers**: los cuatro compartian el hueco,
asi que mirar en que se contradecian no lo mostraba. Lo encontro sondear el
parser unico contra las formas de bloque que markdown admite, no compararlo con
sus antecesores.

La regla es estrecha a proposito: cuatro espacios (o un tab, que vale cuatro) y
nada mas, sin las reglas de CommonMark sobre interrumpir parrafos. Una linea en
blanco no cuenta aunque tenga espacios. **El costo se midio antes de aplicarla**:
0 de 733 AC reales, 0 de 1346 filas `| AC-n |` de review y 0 lineas `Comando:`
tienen sangria de 4. O sea que no cambia lo que hoy se lee de ningun documento
del repo — igual que el AC-5, es un agujero que se cierra antes de que alguien lo
pise.

Vive en el parser unico, asi que arregla a los tres consumidores de una vez: el
AC deja de parsearse, el sello deja de contarse, y el limpiador de `estampar`
ahora **conserva** el sello indentado en vez de borrarlo, porque es prosa.

El simbolo indentado entra al alfabeto de la enumeracion exhaustiva del AC-4. Se
baja el largo maximo de 7 a 6 para no triplicar el tiempo del test: el caso que
importa —un consumidor que no coincide con la clasificacion— aparece con dos
lineas, no con siete.

## Lo que NO se toco, y por que

- **El largo del fence estilo CommonMark** (un bloque abierto con ```` ```` ````
  no deberia cerrarse con ```` ``` ````). Los cuatro parsers compartian la
  divergencia, no producia desacuerdo entre ellos y **no se pudo reproducir
  daño**. Endurecerlo seria repetir el AC-11 de la #66: cambiar codigo que
  funciona contra un bug que no se pudo reproducir.
- **Los parsers de seccion de `prd.rs` y `spec.rs`.** Misma razon: no parsean
  fences.
- **`atlassian::markdown`.** Consume el bloque para convertirlo a ADF; es otro
  problema, no una cuarta copia del mismo.

## Alcance del AC-5

`corpus_real_sin_cambios` asserta la **diferencia** (cero) y no el total (733 AC
hoy): el total sube con cada spec nuevo y un assert sobre el seria un
detector-de-cambios, que es como murio la primera version del test de al lado.

Vale decir con precision que ese test **no se pone rojo revirtiendo el arreglo
de `~~~`** —si el parser vuelve atras, viejo y nuevo coinciden otra vez—. Lo que
prueba es que el arreglo **no cambia** lo que hoy leen los 98 documentos reales:
es un agujero que se cierra antes de que alguien lo pise, no un cambio de
comportamiento. Se pone rojo, si, ante un cambio futuro del parser que si
alterara lo que significan los specs reales (verificado con la mutacion de los
backticks).
