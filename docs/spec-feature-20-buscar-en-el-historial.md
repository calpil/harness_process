# Spec - Feature #20: buscar_en_el_historial

Estado: approved
Aprobado: 2026-08-17T03:44:47Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #20 en el chat (AskUserQuestion: 'Si, lo apruebo'), con el spec mostrado en el chat y abierto en su editor. 19 AC. Decisiones OBS-1..OBS-5: el hub queda FUERA (correccion al backlog: guarda eventos y no prosa, y ese camino no se podria verificar con el hub caido), tope de 20 con aviso y --todos, caida a 'algun termino' avisando, fecha del timestamp en history y mtime en el resto, y se recorren los estado-feature-*.
Plan: docs/plan-feature-20-buscar-en-el-historial.md
PRD: docs/prd/aprendizaje/PRD-aprendizaje.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan pregunta "¿donde decidimos usar ureq?". La respuesta existe: esta en
el ADR-0001, citada en el spec de la #15 y repetida en su plan. Pero encontrarla
significa abrir `docs/` —113 archivos, 28.000 lineas— y hacer `grep` a mano
esperando acordarse de la palabra exacta. En la practica no se busca: se vuelve a
decidir, y a veces se decide distinto.

Las tres features anteriores le dieron al arnes memoria (lecciones, nudge,
perfil), pero **la memoria que no se puede consultar no es memoria**. El propio
`perfil sugerir` de la #19 tuvo que reimplementar su propio recorrido de
`history.md` y de los planes porque no habia forma de preguntarle nada al repo.

DESPUES: `sh harness_cli buscar ureq` responde en milisegundos con el archivo, la
linea, la feature y la fecha, ordenado por relevancia y no por orden alfabetico.
Sin LLM, sin indice que mantener y sin dependencias nuevas: el corpus entero de
un proyecto son ~1 MB de texto, y leerlo es mas barato que mantener un indice
que se desactualiza.

## Hoy -> Como va a funcionar

```
HOY                                    DESPUES

"¿donde decidimos X?"                  sh harness_cli buscar "X"
  |__ grep -r manual por docs/           |__ recorre docs/**/*.md + history.md
  |__ adivinar la palabra exacta         |__ terminos en cualquier orden, sin
  |__ leer 113 archivos                  |   importar mayusculas
  `__ o volver a decidir                 |__ rankea: encabezados y conocimiento
                                         |   curado primero, features recientes
                                         |   antes que viejas
                                         `__ archivo:linea + feature + fecha
                                             (y --json para scripts)
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero preguntar "¿donde decidimos X?" y tener la respuesta con
  su archivo y su linea, para no volver a decidir algo que ya decidi.
- P1: Como agente de cualquier backend, quiero encontrar por TEMA lo que el repo
  ya sabe antes de proponer, sin leer 113 archivos ni adivinar el nombre exacto.
- P1: Como cualquiera de los dos, quiero que lo mas relevante este arriba: un
  encabezado o una leccion valen mas que una linea suelta de bitacora.
- P2: Como script o hook, quiero `--json` para encadenar la busqueda con otra
  cosa.
- P2: Como usuario de un proyecto recien instalado, quiero que buscar sin
  resultados me diga que no hay, no que falle.

## Criterios de aceptacion (Given/When/Then)

### Que se busca y como

- AC-1: Given un repo con artefactos, When se corre `sh harness_cli buscar
  "<consulta>"`, Then se recorren `docs/**/*.md` (specs, planes, impl, review,
  estados archivados, lecciones, PRDs, ADRs, perfil y los docs base) y
  `progress/history.md`; y **no** se recorre `bkp/` ni ningun directorio de
  respaldo.
- AC-2: Given una consulta de varios terminos, When se busca, Then una linea
  cuenta como resultado si contiene **todos** los terminos, sin importar el orden
  ni las mayusculas. Si ninguna linea los contiene todos, se cae a las lineas con
  **alguno** y la salida lo **dice explicitamente**, en vez de devolver vacio en
  silencio.
- AC-3: Given una consulta vacia o solo con espacios, When se busca, Then exit 2
  con un mensaje que muestra la forma de uso.

### Ranking

- AC-4: Given resultados en distintas fuentes, Then el orden pone primero el
  conocimiento **curado** (lecciones y perfil), despues las **decisiones** (specs
  y planes, incluidos sus PRDs), despues la **evidencia** (impl, review, estados)
  y por ultimo la **bitacora** (`history.md`); a igualdad, gana la feature mas
  reciente.
- AC-5: Given una linea que es un **encabezado** markdown (`#`) o un campo de
  frontmatter de leccion (`nombre:`, `descripcion:`, `triggers:`), Then pesa mas
  que una linea del cuerpo, porque es donde vive el tema del documento.
- AC-6: Given una consulta cuyos terminos aparecen **contiguos** en la linea
  (frase exacta), Then esa linea pesa mas que una donde aparecen dispersos.
- AC-7: Given `--json`, Then cada resultado expone `archivo`, `linea`, `feature`,
  `fecha`, `fuente`, `texto` y `score`, de modo que el ranking sea **auditable**
  y no una caja negra.

### Salida

- AC-8: Given resultados, When se imprime, Then cada uno muestra
  `<archivo>:<linea>` (ruta relativa a la raiz, clickeable), la feature y la
  fecha cuando se pueden determinar, y el texto de la linea recortado a un ancho
  legible.
- AC-9: Given mas resultados que el tope de salida (20 por defecto), Then se
  imprimen los primeros y una linea final dice **cuantos quedaron fuera** y como
  verlos (`--todos`). Nunca se trunca en silencio.
- AC-10: Given una consulta sin ningun resultado, Then se dice claramente que no
  hubo coincidencias y el exit code es **0** (no encontrar no es un error), con
  una sugerencia de que probar (menos terminos).
- AC-11: Given `--json`, Then la salida es JSON valido tambien cuando no hay
  resultados (lista vacia), para que un script no tenga que manejar dos formatos.

### Limites y garantias

- AC-12: Given el corpus tipico de un proyecto (este repo: 113 archivos, ~28.000
  lineas, 1,1 MB), When se busca, Then la respuesta es del orden de
  **milisegundos** y **no** se crea, lee ni mantiene ningun indice: el escaneo es
  completo en cada corrida.
- AC-13: Given cualquier consulta, Then **no** se invoca ningun modelo y **no** se
  agrega ninguna dependencia de runtime (Articulo 6): se usa lo que ya esta en
  `Cargo.toml`.
- AC-14: Given el hub PostgreSQL caido o no configurado, When se busca, Then el
  comportamiento y los exit codes son identicos: `buscar` no lo consulta.
- AC-15: Given un archivo ilegible o con bytes invalidos, When se busca, Then se
  saltea sin abortar la busqueda ni ensuciar la salida.
- AC-16: Given un `docs/` inexistente (proyecto recien creado), When se busca,
  Then se informa que no hay corpus y exit 0.

### Integracion, docs y verificacion

- AC-17: Given `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md`
  (+ plantilla) y las superficies de ambos instaladores, Then documentan `buscar`
  con su forma de uso, el orden del ranking y la garantia de que no hay indice ni
  LLM.
- AC-18: Given los tres roles, Then el lider y el implementer usan `buscar` antes
  de proponer o reconstruir algo (es mas barato que releer el repo), y el
  reviewer puede verificar con el una decision citada.
- AC-19: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test` y `cargo clippy --all-targets -- -D warnings` estan verdes con
  tests de: AND y su caida a OR, ranking por fuente, por encabezado y por frase,
  consulta vacia, sin resultados, tope de salida con su aviso, `--json` (con y
  sin resultados) y archivo ilegible; y `tests/setup_smoke.sh` sigue verde.

## Los datos que se tocan

- **disparador**: el comando `buscar`, invocado a mano o por un agente.
- **interruptor**: ninguno. `buscar` es de **solo lectura**: no tiene estado, no
  tiene regla que lo apague y no puede romper nada. Si no hay corpus, lo dice.
- **candado**: no aplica — no hay escritura que repetir.
- **lo que NO se toca**: absolutamente nada. `buscar` no escribe ni un byte:
  ni en `progress/`, ni en `docs/`, ni en el hub.

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien pregunta algo

  ¿la consulta tiene terminos?   -> si no, mostramos como se usa y salimos
  ¿existe el corpus?             -> si no, lo decimos y salimos con 0

  recorremos docs/**/*.md y history.md, linea por linea
  nos quedamos con las lineas que contienen TODOS los terminos
  ¿ninguna los tiene todos?      -> nos quedamos con las que tienen ALGUNO,
                                    y lo AVISAMOS

  ordenamos por: que tan curada es la fuente, si es encabezado,
                 si los terminos van juntos, y que tan reciente es

  ENTONCES imprimimos los primeros con su archivo:linea, feature y fecha,
           con la restriccion de que si quedaron mas afuera lo decimos,
           y de que no escribimos nada en ningun lado.
```

**Promesas:** no escribe nada · no llama a ningun modelo · no mantiene indice ·
no depende del hub · no encontrar sale con 0 · nunca trunca en silencio.

## No funcionales

- **SLOs**: milisegundos sobre ~1 MB de texto. Sin indice, sin red, sin hub.
- **Seguridad**: solo lectura. La consulta del usuario **no** se interpola en
  ningun comando ni expresion regular construida dinamicamente (se compara como
  texto), asi que no hay superficie de inyeccion ni de ReDoS.
- **Observabilidad**: exit 0 con o sin resultados, exit 2 solo por uso invalido;
  el `score` visible en `--json` hace auditable el orden.

## Fuera de alcance

- El curador (#21) y el mapa `journey` (#22).
- Cualquier indice persistente (FTS5, tsvector, cache): el corpus es chico y un
  indice desactualizado miente, que es peor que escanear.
- Busqueda semantica o por embeddings: no hay LLM en el camino (NO1 del PRD).
- Reemplazar a `graphify query`, que responde sobre el **grafo del codigo**;
  `buscar` responde sobre los **artefactos del proceso**. Son complementarios.

## Observaciones (decisiones pendientes)

Todas decididas por Alan el 2026-08-17, en el mismo acto de aprobacion del spec.
No queda ninguna observacion abierta: el implementer puede avanzar sin preguntar.

- OBS-1: ¿`buscar` consulta tambien el hub con `to_tsvector`? — **DECIDIDO: no,
  solo archivos.** Es una correccion al texto del backlog, por tres razones: el
  hub guarda filas de evento (accion, estado, artefacto), no la prosa donde estan
  las decisiones; el hub esta caido en este entorno, asi que ese camino se
  entregaria **sin poder verificarse ni una vez**, que es justo la deuda que este
  repo no acepta; y el PRD ya decidio que el aprendizaje funciona sin hub.
  Vinculante para AC-1 y AC-14.
- OBS-2: ¿Tope de salida? — **DECIDIDO: 20 por defecto**, con una linea final que
  dice cuantos quedaron fuera y `--todos` para verlos. Vinculante para AC-9.
- OBS-3: ¿Caida a "algun termino"? — **DECIDIDO: si, avisando.** Ayuda al caso de
  uso real (no recordar la palabra exacta) y el aviso evita que el usuario crea
  que encontro una coincidencia exacta. Vinculante para AC-2.
- OBS-4: ¿Que fecha se muestra? — **DECIDIDO: el timestamp de la linea** en
  `history.md`, y la fecha de modificacion del archivo para el resto.
  Vinculante para AC-8.
- OBS-5: ¿Se recorren los `docs/estado-feature-*.md`? — **DECIDIDO: si**: son
  evidencia real y suelen tener la nota de cierre mas concreta. Vinculante para
  AC-1.
