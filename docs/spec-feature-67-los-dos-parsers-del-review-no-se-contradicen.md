# Spec - Feature #67: los_dos_parsers_del_review_no_se_contradicen

Estado: approved
Aprobado: 2026-08-31T01:27:25Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-67-los-dos-parsers-del-review-no-se-contradicen.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: el reviewer de la #64 dejo anotado que hay DOS parsers de bloques de
codigo que no coinciden. Al investigarlo son **cuatro**, con **tres semanticas
distintas**, sobre los mismos documentos:

| donde | que hace |
| --- | --- |
| `revision.rs:434` (el gate del review) | recuerda CUAL fence abrio; solo cierra con el mismo |
| `commands/revision.rs:117` (el limpiador de `estampar`) | togglea con CUALQUIERA de los dos |
| `verificacion.rs:161` (`parsear`, los AC del spec) | togglea SOLO con ` ``` `; no conoce `~~~` |
| `atlassian/markdown.rs:125` | CONSUME el bloque (caso distinto) |

Y el mas caro no es el que motivo la feature. `verify` **ejecuta comandos
escritos dentro de un bloque `~~~`**, reproducido:

```
~~~markdown
- AC-99: Given ejemplo, When ejemplo, Then ejemplo.
  Comando: `echo LA-DOCUMENTACION-SE-EJECUTO > /tmp/parsear_tildes.txt`
~~~
```

→ `verify --feature 1` dice `AC-99 ... [ok] verde` y el archivo aparece escrito.
Es exactamente el bug que la #23 cerro para backticks —*"un spec que documenta la
sintaxis no puede quedar verificando su documentacion"*, `verificacion.rs:157`— y
esta **abierto para tildes**. Agravante: el test que deberia detectarlo
(`declaraciones_fuera_de_bloques`, `verificacion.rs:973`) togglea tambien solo con
backticks, o sea que **comparte el punto ciego** y su acuerdo sobre 20 specs no
significa lo que su comentario dice.

Lo que el reviewer si vio, tambien reproducido con el binario: `revision
--veredicto` **borra prosa del reviewer** cuando el review cita un bloque ajeno.
(La paridad quedo al reves de como el reviewer la describio: con UNA linea `~~~`
adentro la cita sobrevive, con DOS se borra. El bug es el mismo; el caso
concreto, el opuesto.) Y en la otra direccion **deja dos sellos** en el archivo,
rompiendo la promesa de idempotencia de `commands/revision.rs:108`.

DESPUES: hay **un solo** parser. Los cuatro consumidores le preguntan lo mismo y
no pueden discrepar, porque no hay dos implementaciones que mantener
coincidiendo. Y `verify` deja de ejecutar lo que alguien marco como
documentacion, sea cual sea el fence que uso.

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES

revision.rs        emparejado           markdown::lineas_clasificadas
commands/revision  toggle simetrico       (linea, Fuera | Fence | Dentro)
verificacion.rs    solo backticks               |
atlassian/md       consume el bloque            |__ el gate      -> filtra Fuera
                                                |__ el limpiador -> Fuera + conserva Fence
tres semanticas, cuatro copias                  |__ parsear      -> != Dentro
                                                |__ el test cruzado -> la misma
verify ejecuta lo de adentro de ~~~
  (el bug de la #23, abierto para tildes)  verify no ejecuta documentacion,
                                            con ``` o con ~~~

cita_resuelve: >8MB -> "la linea         cita_resuelve: >8MB -> "no se pudo
  no existe"  (falso: existe)              comprobar" (tercera respuesta)
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero que `verify` no ejecute un comando que yo escribi dentro
  de un bloque de documentacion, sin importar con que fence lo marque.
- P1: Como reviewer, quiero que registrar mi veredicto no me borre la prosa ni me
  deje dos sellos contradictorios en el archivo.
- P2: Como Alan, quiero que cuando el arnes no pueda comprobar una cita me diga
  eso, y no que la linea no existe.
- P2: Como mantenedor, quiero que no puedan volver a existir dos parsers del
  mismo formato divergiendo en silencio.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un spec con un `Comando:` dentro de un bloque `~~~`, When se corre
  `verify`, Then ese comando NO se ejecuta y ese AC no existe para el arnes.
  Comando: `cd rust && out=$(cargo test verify_no_ejecuta_documentacion 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-2: Given un review que cita un bloque ajeno (fences mezclados, en las dos
  paridades), When se corre `revision --veredicto`, Then la prosa del reviewer
  queda intacta.
  Comando: `cd rust && out=$(cargo test estampar_no_toca_la_prosa 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-3: Given un review que ya tiene un sello, en cualquier forma de archivo,
  When se estampa otro, Then queda **exactamente uno**: la promesa de
  idempotencia de `commands/revision.rs:108` se cumple.
  Comando: `cd rust && out=$(cargo test estampar_deja_un_solo_sello 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-4: Given cualquier documento, When lo clasifican los cuatro consumidores,
  Then **coinciden**. Se prueba por enumeracion exhaustiva sobre el alfabeto
  {` ``` `, `~~~`, una linea de sello, texto} hasta n=7: hoy discrepan en 37% de
  los documentos de 7 lineas.
  Comando: `cd rust && out=$(cargo test los_parsers_no_discrepan 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-5: Given los 55 specs y los reviews reales del repo, When se parsean con el
  parser unico, Then dan **los mismos AC** que hoy: 728 AC, cero diferencias.
  Comando: `cd rust && out=$(cargo test corpus_real_sin_cambios 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-6: Given una cita a un archivo mas grande que el tope, When el gate la
  evalua, Then responde **"no se pudo comprobar"** —una tercera respuesta— y NO
  "la linea no existe", que hoy es falso: la linea existe y `sed` la muestra.
  Comando: `cd rust && out=$(cargo test cita_grande_no_se_pudo_comprobar 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-7: Given esa tercera respuesta, When se cierra la feature, Then el cierre
  decide (no cuelga ni muere): el tope se conserva, porque sacarlo cuesta 10,5 s
  por 2 GB dentro de un gate sin timeout.
  Comando: `cd rust && out=$(cargo test cita_grande_no_cuelga_el_cierre 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-8: Given un archivo de N lineas terminado en salto, When se cita la linea
  N+1, Then la cita NO resuelve. Hoy si: reproducido con un archivo de 3 lineas,
  `evidencia.txt:4` da rc=0.
  Comando: `cd rust && out=$(cargo test la_cita_no_acepta_la_linea_siguiente_al_eof 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-9: Given un review con una linea `Revisado:` pelada arriba y el sello real
  mas abajo, When corre el gate, Then encuentra el sello. Hoy el `?` de
  `revision.rs:419` aborta la funcion entera en la primera linea sin valor y el
  gate dice "no lleva el sello" con el sello tres lineas abajo.
  Comando: `cd rust && out=$(cargo test el_sello_se_encuentra_aunque_haya_lineas_peladas 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-10: Given el parser unico, When alguien agrega un quinto consumidor que
  vuelva a parsear fences por su cuenta, Then algo se pone rojo.
  Comando: `bash tests/conventions_check.sh`

- AC-12: Given un spec con un `Comando:` en un bloque de codigo INDENTADO (4
  espacios, la otra forma de bloque que tiene markdown), When se corre `verify`,
  Then ese comando NO se ejecuta; y un sello citado con esa sangria no cuenta
  como veredicto. Los cuatro parsers viejos compartian el hueco, asi que no era
  una divergencia entre ellos y no se veia desde el problema que motivo la
  feature. Confirmado con el binario antes de cerrarlo: `verify` reportaba el
  AC-99 documentado y el archivo aparecia escrito. Costo medido de la regla en el
  corpus real: 0 de 733 AC, 0 de 1346 filas de review y 0 lineas `Comando:`
  tienen sangria de 4, o sea que no cambia lo que hoy se lee de ningun documento.
  Comando: `cd rust && out=$(cargo test indentad 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-11 (MANUAL): Given `estampar`, When se lo audita, Then lo irreversible va
  ultimo: hoy escribe el archivo (`commands/revision.rs:152`) y **despues**
  comprueba si el gate puede leerlo (`:155`). No se pudo reproducir un caso donde
  eso deje el archivo pisado con el comando en error, asi que **no se declara
  como bug**; se invierte el orden como cambio de forma, y el reviewer juzga si
  quedo bien.

## Los datos que se tocan

- disparador: cualquier lectura de un documento markdown del arnes (specs,
  reviews).
- el primitivo nuevo: `markdown::lineas_clasificadas(texto) -> (linea, Clase)`
  con `Clase { Fuera, Fence, Dentro }`. Vive en un modulo propio porque
  `revision` ya depende de `verificacion` (`revision.rs:605`, `:672`) y nunca al
  reves: poner el parser en `revision` invertiria la dependencia.
- semantica: fences **emparejados** (se recuerda cual abrio). Es la del gate, la
  unica que coincide con como se renderiza el markdown de verdad.
- lo que NO se toca: el largo del fence al estilo CommonMark. Los cuatro parsers
  comparten hoy esa divergencia, no produce desacuerdo entre ellos y **no se
  pudo reproducir daño**: endurecerlo seria repetir el AC-11 de la #66.
- el tope de `cita_resuelve`: se conserva y cambia su RESPUESTA, no su valor.

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien necesita saber que lineas de un documento son codigo

  se le pregunta al UNICO parser, que clasifica cada linea en
  Fuera | Fence | Dentro, recordando cual fence abrio el bloque

  el gate del review          -> se queda con Fuera
  el limpiador de estampar    -> saca los sellos de Fuera, conserva Fence
  el parseo de AC del spec    -> se queda con lo que no es Dentro

  ENTONCES los cuatro dicen lo mismo del mismo documento,
           con la restriccion de que no hay una segunda implementacion
           que alguien tenga que acordarse de mantener igual.
```

Promesas: `verify` no ejecuta lo que esta dentro de un bloque, con cualquiera de
los dos fences · estampar no toca prosa ajena y deja un solo sello · el corpus
real no cambia · el gate nunca dice "no existe" de algo que no pudo comprobar.

## No funcionales

- SLOs: una sola pasada por documento, igual que hoy. El tope de lectura se
  conserva, asi que el peor caso sigue acotado.
- Seguridad: **es lo central**. Hoy `verify` ejecuta shell salido de una seccion
  que el autor marco como documentacion. Esa es la razon por la que la feature no
  puede quedarse en los dos parsers del review.
- Observabilidad: la tercera respuesta del tope se ve en el mensaje del gate.

## Fuera de alcance

- `atlassian/markdown.rs`: CONSUME el bloque (lo convierte a storage de
  Confluence) en vez de saltearlo. Puede apoyarse en el mismo primitivo mas
  adelante; meterlo ahora mezcla dos problemas.
- El largo del fence estilo CommonMark (ver "los datos que se tocan").
- `prd.rs:503`, `:552` y `spec.rs:412` cortan secciones con `starts_with("## ")`
  sin conciencia de fences. Es la misma familia —un parser de markdown que no
  sabe donde termina el codigo— pero no se reprodujo daño ahi. Queda anotado, sin
  AC. (`documentos.rs:16-18` ya documenta que por eso ancla por texto literal.)

## Observaciones (decisiones pendientes)

- **Decisiones del usuario ya tomadas (2026-08-31)**: los cuatro parsers (no solo
  los dos del review), el tope con tercera respuesta (no sacarlo), y los dos bugs
  de una linea adentro de esta feature.
- **Una correccion al hallazgo heredado**: el reviewer de la #64 describio la
  paridad del borrado al reves. Verificado con el binario: con UNA linea `~~~`
  dentro del bloque la cita sobrevive; con DOS se borra. El bug es el mismo, el
  caso es el opuesto. Se deja dicho para que nadie escriba un test sobre el caso
  equivocado.
- **Tres hipotesis que NO se reprodujeron** y por eso no son AC: el orden de
  escritura de `estampar` (queda como AC-11 MANUAL, cambio de forma), la
  divergencia de largo de fence, y los parsers de seccion de `prd.rs`/`spec.rs`.
  En la #66 las tres se habrian vuelto AC y habrian arrastrado vueltas.
- `Peldano elegido:` un modulo nuevo (`markdown.rs`) es peldaño bajo, y la razon
  esta medida: los consumidores necesitan cosas distintas de la misma
  clasificacion, asi que una funcion en un modulo existente invertiria la
  dependencia `revision -> verificacion`. No se agrega ningun comando ni
  superficie.
