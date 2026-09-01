# Spec - Feature #68: el arnes no pierde los AC que pide revisar a mano

Estado: approved
Aprobado: 2026-09-01T22:32:03Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-68-el-arnes-no-pierde-los-ac-que-pide-revisar-a-man.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

Alan cierra la #67. `verify` imprime **"10 verde(s), 0 en rojo, 0 manual(es)"** y
el gate lo deja pasar. El spec tiene **once** criterios. El AC-11 dice, con todas
las letras, que hay algo que tiene que auditar una persona: que en `estampar` lo
irreversible vaya ultimo.

Nadie lo audito. Nadie lo salteo tampoco: el arnes nunca lo pidio.

El AC-11 esta escrito `- AC-11 (MANUAL): ...`. `ac_de` (`verificacion.rs:183`)
exige que los dos puntos vengan pegados a los digitos, asi que ese parentesis
—la anotacion que sirve para marcar "esto lo mira un humano"— hace que el AC
**desaparezca**. Y como el gate del review saca su lista del mismo parser,
tampoco le pide al reviewer una fila que responda por el.

O sea: la unica marca que existe para decir "esto no lo puede comprobar la
maquina" es exactamente lo que hace que nadie tenga que comprobarlo.

Despues: el AC aparece. `verify` lo lista como `[--] manual`, lo cuenta en el
resumen, y no bloquea —esa maquinaria ya existe entera y funciona—. El gate del
review le exige su fila. Y un `Comando:` que venga detras de un AC que el parser
no entiende deja de terminar adjudicado al AC de arriba.

## Hoy -> Como va a funcionar

Hoy `ac_de` acepta `- AC-<digitos>:` y nada mas. Cualquier otra cosa entre el
numero y los dos puntos tira el AC entero. Medido sobre los 55 specs del repo,
**siete AC estan invisibles** en dos familias:

| forma | donde | cuantos |
| --- | --- | --- |
| `- AC-n (MANUAL):` | specs #64, #65, #66, #67 | 4 |
| `- AC-4b:`, `- AC-12b:`, `- AC-12c:` | specs #16, #51 | 3 |

Los dos son el mismo defecto: el autor escribio un criterio y el arnes lo tiro.

**Y hay un segundo daño, peor, que hoy esta latente.** `parsear` cuelga cada
`Comando:` del ultimo AC abierto. Si un AC no reconocido trae comando, ese
comando **se le adjudica al AC anterior**. Reproducido con el parser real:

- `- AC-1: uno` (sin comando) + `- AC-2 (MANUAL): dos` + `Comando: touch MAL.txt`
  => **AC-1 queda con `touch MAL.txt`**. `verify` imprimiria "AC-1 verde" habiendo
  corrido la prueba de otro criterio: un verde atribuido al AC equivocado, que es
  la familia de la #44 y de `criterios-de-cierre-que-se-pueden-fallar`.
- Si el AC anterior YA tenia comando, el comando del invisible se **descarta en
  silencio**: un criterio que declara como se prueba y nunca se corre.

Que quede claro cual es cual: la desaparicion **esta pasando hoy** (siete AC). La
mala atribucion es **reproducible pero todavia no se disparo** en este repo,
porque ninguno de los siete declara comando. Se arregla igual, y la diferencia se
dice en vez de inflar el problema.

Despues: `ac_de` acepta un sufijo de letras y una anotacion entre parentesis, y
sigue rechazando lo que no es un AC.

## Recorridos de usuario (priorizados)

- P1: Como usuario que cierra una feature, quiero que un AC que marque `(MANUAL)`
  aparezca en `verify` y se lo exija al reviewer, para que "0 manual(es)" sobre un
  spec que pide auditoria a mano deje de ser una respuesta posible.
- P2: Como autor de un spec, quiero que un `Comando:` nunca termine colgado de un
  AC que no es el suyo, aunque me equivoque escribiendo el encabezado.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un spec con `- AC-2 (MANUAL): ...`, When se corre `verify`, Then el
  AC aparece en el reporte con estado `manual` y el resumen dice `1 manual(es)`.
  Comando: `cd rust && out=$(cargo test el_ac_manual_aparece 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-2: Given ese mismo AC manual, When se corre el gate del review sin una fila
  que responda por el, Then el cierre se niega nombrandolo. La marca "esto lo mira
  una persona" tiene que OBLIGAR a que una persona lo mire, no eximirlo.
  Comando: `cd rust && out=$(cargo test el_gate_exige_fila_para_el_manual 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-3: Given un spec con `- AC-4b:`, When se parsea, Then `AC-4b` es un AC
  propio y distinto de `AC-4`. Son tres AC reales de los specs #16 y #51.
  Comando: `cd rust && out=$(cargo test el_sufijo_de_letra_es_un_ac_propio 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-4: Given un AC seguido de un `Comando:`, When el AC anterior no tiene
  comando, Then el comando NO se le adjudica al anterior. Reproducido: hoy
  `AC-1` se queda con el `touch MAL.txt` que era del `AC-2 (MANUAL)`.
  Comando: `cd rust && out=$(cargo test el_comando_no_migra_al_ac_anterior 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-5: Given lineas que NO son AC (`- AC-12 y AC-13: ...`, `- AC-: x`,
  `- AC-1 sin dos puntos`, `- ACR-1: x`), When se parsean, Then ninguna se
  reconoce. Aflojar el parser no puede empezar a comerse prosa.
  Comando: `cd rust && out=$(cargo test lo_que_no_es_un_ac_sigue_sin_serlo 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-6: Given los 55 specs reales del repo, When se parsean con el `ac_de` nuevo,
  Then aparecen **exactamente los siete AC que hoy faltan** y ninguno mas: 733
  hoy, 740 despues, y la diferencia son esos siete nombrados uno por uno.
  Comando: `cd rust && out=$(cargo test los_siete_que_faltaban_y_ninguno_mas 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-7: Given una feature ya cerrada cuyo spec tiene un AC manual, When se corre
  cualquier comando del arnes sobre ella, Then nada se rompe ni se reabre. El
  arreglo cambia lo que se LEE de cuatro specs cerrados.
  Comando: `cd rust && out=$(cargo test las_features_cerradas_no_se_mueven 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-8 (MANUAL): Given este mismo spec, When se cierra la feature, Then el AC-8
  aparece en `verify` como manual y este spec tiene fila en el review. Es la
  feature probandose sobre si misma: si el arreglo no sirve, este AC se vuelve a
  perder y nadie se entera. (Y si, esta escrito a proposito con la forma que hoy
  desaparece.)

## Los datos que se tocan

- disparador: cualquier lectura de un spec (`verify`, el gate del review, el gate
  de verify verde).
- lee: `docs/spec-feature-<id>.md`.
- escribe: nada nuevo. `docs/verify-<id>.md` gana las filas que faltaban.
- borra: nada.

## Pseudo-codigo (el acuerdo)

```
ac_de(linea):
    resto = linea sin el prefijo "- AC-"        # si no esta, no es AC
    numero = digitos del principio de resto     # vacio => no es AC
    resto = resto sin el numero
    resto = resto sin las letras ascii del principio   # AC-4b, AC-12c
    si resto empieza con " (":
        resto = resto desde el ")" siguiente    # sin ")" => no es AC
    si resto no empieza con ":": no es AC
    devolver "AC-" + numero + letras
```

La anotacion NO entra en el nombre del AC: `- AC-11 (MANUAL):` es `AC-11`. El
sufijo de letra SI, porque `AC-4b` es otro criterio que `AC-4`.

## No funcionales

- `parsear` sigue siendo pura: no toca el filesystem ni ejecuta nada.
- Sin dependencias nuevas y sin regex: es un `strip_prefix` mas largo.

## Fuera de alcance

- **Reabrir features cerradas.** Los cuatro specs con AC manual estan cerrados y
  sus reviews no tienen fila para ellos. No se re-verifica hacia atras: el gate
  corre al cerrar, y ya corrio.
- **Elegir la sintaxis de la anotacion.** Se acepta lo que los specs YA usan
  (`(MANUAL)` y el sufijo de letra), no se inventa una nueva.
- **Que `verify` haga algo con el estado manual mas alla de listarlo.** La
  maquinaria (`Estado::Manual`, `[--]`, el conteo, `bloquea() == false`) ya existe
  y funciona; esta feature solo hace que le llegue el AC.

## Observaciones (decisiones pendientes)

- OBS-1: **el alcance quedo mas ancho que el nombre de la feature.** El backlog
  dice "los AC que pide revisar a mano" y son los cuatro `(MANUAL)`. Medir mostro
  tres mas —`AC-4b`, `AC-12b`, `AC-12c`— que se pierden por la MISMA linea de
  codigo. Arreglar solo una familia deja tres AC invisibles y una segunda pasada
  por el mismo `ac_de`. Se propone cubrir las dos; el usuario decide.
- OBS-2: al arreglarse, el gate del review va a **exigir fila para los AC
  manuales** de las features futuras. Es el efecto buscado —el AC-2 lo fija—
  pero conviene decirlo: revisar pasa a costar una fila mas por cada AC que el
  autor marque como manual.
