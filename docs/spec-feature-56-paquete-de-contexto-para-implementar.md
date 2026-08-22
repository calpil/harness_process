# Spec - Feature #56: paquete_de_contexto_para_implementar

Estado: approved
Aprobado: 2026-08-22T16:31:24Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #56 en el chat (16 AC): paquete de contexto para implementar, simetrico al de revision de la #51, que sigue los punteros de architecture.md, avisa cuando el mapa no cubre el tema, acota por presupuesto y declara sus huecos. Decidio ademas las cuatro observaciones: graphify solo con --con-grafo, grafo vencido a los 7 dias, resumen en start siempre, y arreglar el puntero roto de realestate ya y por separado.
Plan: docs/plan-feature-56-paquete-de-contexto-para-implementar.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan le pide a un agente que implemente algo sobre el motor de reajuste
del SaaS inmobiliario. El agente hace lo que el arnes le pide: busca el mapa.
Y esto es lo que encuentra, verificado el 2026-08-22:

- `GolandProjects/realestate/docs/architecture.md` son **siete lineas y un
  puntero**: dice que la copia canonica vive en
  `.../WebstormProjects/realestate/Real-State/docs/architecture.md`. Esa ruta
  **no existe**: el documento real esta un directorio mas arriba
  (`.../WebstormProjects/realestate/docs/architecture.md`, 656 lineas). El
  puntero quedo viejo despues de una mudanza y nadie lo nota, porque nada lo
  chequea.
- Aun siguiendolo bien, ese mapa **no menciona "reajuste" ni una sola vez**. El
  tema de la task no esta mapeado.
- El grafo de graphify del proyecto es del 7 de agosto: quince dias.
- Y `buscar` con los terminos del tema devuelve **12.521 resultados en 228
  archivos**, porque cuando ningun archivo tiene todos los terminos cae a OR.

O sea: cuatro consultas baratas, cuatro respuestas que no sirven, y ninguna que
diga *"aca no hay nada escrito sobre esto"*. El agente se entera del vacio de la
unica forma que le queda: explorando. En este caso, un mapeo de cuatro agentes,
diez minutos y **693.6k tokens** para descubrir algo que se podia saber en dos
segundos: que el mapa no cubre el tema.

La feature #51 ya resolvio esto del lado del reviewer — `revision --feature`
entrega el paquete acotado y el reviewer dejo de explorar. Del lado de
implementar no existe el equivalente.

DESPUES: antes de leer nada, el agente pide el paquete. En un segundo sabe que
mapa hay, si el puntero apunta a algo que existe, si el mapa cubre el tema o no,
que servicios toca segun el hub, que edad tiene el grafo, que lecciones aplican
y que features anteriores tocaron lo mismo. Y cuando no hay nada — que es el
caso real — el paquete lo **dice**, con esa frase, antes de que se gaste un
token en averiguarlo.

## Hoy -> Como va a funcionar

```
HOY                                      DESPUES
start --feature N                        start --feature N
  |__ crea plan, spec, rama, worktree      |__ crea plan, spec, rama, worktree
  |__ (el agente se las arregla)           |__ RESUMEN del contexto:
                                           |     mapa: 656 lineas (puntero OK)
                                           |     cobertura: NO cubre "reajuste"
                                           |     grafo: 15 dias | impacto: 3 svc
                                           |     lecciones: 2 | relacionadas: 4
                                           |__ "el cuerpo: harness contexto --feature N"

el agente explora el repo               contexto --feature N (o --tema "...")
(4 agentes, 693.6k tokens)                un paquete acotado, con presupuesto
                                          declarado y los huecos dichos en voz alta
```

## Recorridos de usuario (priorizados)

- P1: Como agente que va a implementar, quiero recibir el contexto armado en vez
  de tener que salir a buscarlo, para no pagar una exploracion por cada feature.
- P1: Como Alan, quiero que el arnes me diga **"el mapa no cubre esto"** antes de
  que el agente gaste, para decidir yo si vale la pena mapear primero.
- P1: Como Alan, quiero enterarme de que un puntero de documentacion quedo roto
  el dia que se rompe, no seis features despues.
- P2: Como agente, quiero pedir contexto de un tema **sin** feature creada, para
  el momento en que Alan pregunta "¿que tan grande es esto?" antes de decidir.
- P2: Como Alan, quiero que el paquete diga lo que cuesta y lo que recorto, igual
  que el de revision, para que el presupuesto sea una decision y no una sorpresa.

## Criterios de aceptacion (Given/When/Then)

### El paquete

- AC-1: Given una feature activa, When corro `harness contexto --feature <id>`,
  Then imprime el paquete en este orden — mapa, cobertura del tema, impacto,
  grafo, historia, lecciones, features relacionadas — y no escribe nada (solo
  lectura, como `revision`).
- AC-2: Given un tema sin feature, When corro `harness contexto --tema "<texto>"`,
  Then arma el mismo paquete usando el texto como tema.
- AC-3: Given ni `--feature` ni `--tema`, When lo corro, Then falla con exit 2 y
  un mensaje que dice las dos formas de invocarlo (Articulo 4: error accionable).

### El mapa y sus punteros (el caso real)

- AC-4: Given un `docs/architecture.md` que es un puntero a otra ruta, When armo
  el paquete, Then sigue el puntero y usa el documento de destino, diciendo de
  donde salio.
- AC-5: Given un puntero cuyo destino NO existe, When armo el paquete, Then lo
  declara como hueco con la ruta que falta — nunca lo presenta como "no hay
  mapa", que es un diagnostico distinto.
  Comando: `cd rust && cargo test contexto_puntero`

### La senal que hoy no existe

- AC-6: Given que ninguno de los terminos del tema aparece en el mapa, When armo
  el paquete, Then incluye una linea explicita de que el mapa NO cubre el tema,
  con los terminos buscados y que hacer al respecto.
- AC-7: Given que el mapa SI cubre el tema, Then entrega las secciones del mapa
  donde aparece, recortadas por presupuesto, en vez del documento entero.
- AC-8: Given `graphify-out/graph.json` con mas de **7 dias** (OBS-2), When armo
  el paquete,
  Then declara su edad con la fecha; si no existe, lo dice como hueco con el
  comando que lo genera.
  Comando: `cd rust && cargo test contexto_cobertura`

### Presupuesto (la regla de la #51)

- AC-9: Given `--max-lineas N`, When el material excede el presupuesto, Then
  recorta y declara que recorto y cuanto quedo afuera.
- AC-10: Given cualquier paquete, When termina, Then reporta su tamaño en lineas
  y tokens estimados, como hace `revision`.
- AC-11: Given la busqueda del tema, When ningun archivo tiene todos los
  terminos, Then entrega como maximo los K hits mas curados (lecciones y perfil
  antes que bitacora) en vez del volcado completo — hoy son 12.521.
  Comando: `cd rust && cargo test contexto_presupuesto`

### Que no dependa de que el agente se acuerde

- AC-12: Given `start --feature <id>`, When crea la feature, Then imprime el
  RESUMEN del contexto (que hay, que falta, que cuesta pedir el cuerpo) —
  incluso, y sobre todo, cuando el paquete esta vacio.
- AC-13: Given los roles del lider y del implementer, When los lee un agente,
  Then el primer paso es pedir el paquete, y esta dicho que hacer cuando el
  paquete avisa que el mapa no cubre el tema: proponerselo al USUARIO antes de
  explorar por su cuenta.
- AC-14: Given `templates/`, Then los roles y la guia van espejados (Articulo 6).

### Multi-LLM y verificacion

- AC-15: Given un proyecto sin hub alcanzable, sin graphify y sin
  architecture.md, When armo el paquete, Then sale igual con los huecos
  declarados uno por uno: ninguna pieza es obligatoria y ningun backend de LLM
  es necesario para armarlo.
  Comando: `cd rust && cargo test contexto_sin_nada`
- AC-16: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, `bash tests/setup_smoke.sh` y
  `bash harness_check.sh`, Then los cuatro terminan limpios.
  Comando: `cd rust && cargo clippy --all-targets -- -D warnings`

## Los datos que se tocan

- entrada: `--feature <id>` (usa nombre, servicio, spec y plan de la feature) o
  `--tema "<texto>"`.
- presupuesto: `--max-lineas <N>` (default a definir, en linea con los 400 del
  diff de `revision`).
- fuentes, todas OPCIONALES: `docs/architecture.md` (+ el destino de su puntero),
  `graph impacto` del hub, `graphify-out/graph.json` (edad y comando de query),
  `buscar` acotado, `leccion list` por trigger, specs e impl de features del
  mismo servicio.
- salida: stdout (texto) y `--json`. **No escribe archivos**: el paquete se arma
  cada vez, como `revision`.
- huecos: lista explicita de lo que falto y con que comando se consigue.

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien pide el contexto de <feature|tema>

  el mapa:      architecture.md -> ¿es un puntero? -> seguilo
                                   ¿el destino existe? -> si no, HUECO con la ruta
  la cobertura: ¿el mapa menciona los terminos del tema?
                   si NO -> decilo con esas palabras: el mapa no cubre esto
                   si SI -> entrega esas secciones, no el documento entero
  el impacto:   servicios de la feature -> graph impacto (si el hub responde)
  el grafo:     ¿existe graph.json? -> su edad; si no, HUECO con el comando
  la historia:  buscar acotado -> los mas curados primero, tope K
  lo aprendido: lecciones cuyo trigger pega con el tema
  lo anterior:  specs/impl de features del mismo servicio

  ENTONCES recorta por presupuesto, deci que recortaste,
           reporta tamaño en lineas y tokens,
           y lista los HUECOS uno por uno.
```

Promesas: nunca bloquea · nunca escribe · ninguna fuente es obligatoria · el
vacio se dice, no se disimula.

## No funcionales

- SLOs: el paquete se arma en segundos; el hub y graphify se consultan con
  tiempo limite y su ausencia es un hueco, no un error.
- Seguridad (Articulo 4): solo lectura; ningun secreto en la salida.
- Observabilidad: el paquete declara su tamaño y sus huecos; el resumen de
  `start` es la version corta de lo mismo.

## Fuera de alcance

- Escribir o completar `architecture.md`: el paquete detecta el hueco, no lo
  llena.
- Arreglar el puntero roto de `realestate`: es otro repo (ver OBS-4).
- Regenerar el grafo de graphify o levantar el hub.
- Mejorar la relevancia de `buscar` mas alla del tope K (eso es la #39).
- Reemplazar `revision`: son dos paquetes distintos, uno para revisar y otro
  para implementar.

## Observaciones y decisiones

- OBS-1 [DECIDIDA por el USUARIO, 2026-08-22]: el paquete NO invoca
  `graphify query` por default: informa la edad del grafo y el comando exacto.
  Con `--con-grafo` lo invoca, y solo si el binario existe. El paquete tiene que
  ser barato y predecible.
- OBS-2 [DECIDIDA por el USUARIO, 2026-08-22]: un grafo se declara vencido a los
  **7 dias**, no a los 14 de la propuesta.
- OBS-3 [DECIDIDA por el USUARIO, 2026-08-22]: el resumen de `start` sale
  SIEMPRE. Si depende de acordarse no es invariante
  (`promesas-estructurales-vs-disciplina`), y el caso donde mas importa —el
  paquete vacio— es justo el que nadie pediria.
- OBS-4 [DECIDIDA por el USUARIO, 2026-08-22]: el puntero roto de `realestate`
  se arregla YA y por separado (una linea: sobra el segmento `Real-State/`), sin
  esperar a que esta feature exista. AC-5 se prueba con fixtures propias, no con
  el repo de un tercero.
