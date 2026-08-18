# Spec - Feature #22: mapa_de_aprendizaje

Estado: approved
Aprobado: 2026-08-17T04:38:04Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #22 en el chat (AskUserQuestion: 'Si, lo apruebo'), con el spec mostrado en el chat y abierto en su editor. 18 AC. Decisiones OBS-1..OBS-5, dos de ellas correcciones al backlog que REDUCEN el alcance: solo archivos (no hub ni graphify), journey es SOLO LECTURA sin delete ni edit porque serian una segunda puerta capaz de saltear el 'nunca borra' de la #21 y el gate del --yes de la #19, las features cerradas sin leccion aparecen y cuentan como hueco, y una entrada de perfil se ubica en la fecha de la feature mas reciente que cita.
Plan: docs/plan-feature-22-mapa-de-aprendizaje.md
PRD: docs/prd/aprendizaje/PRD-aprendizaje.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: el arnes ya tiene tres almacenes de memoria y cada uno se mira por
separado. `leccion list` muestra seis lecciones. `perfil show` muestra cuatro
entradas. `status` muestra veintiun features cerradas. **Nadie ve las tres cosas
juntas**, y menos todavia los hilos que las unen: que la leccion
`criterios-de-cierre-que-se-pueden-fallar` nacio en la #20 y ya cambio la #21, o
que la entrada 3 del perfil salio de las features #17 y #19.

Peor: nadie ve los **huecos**. Una leccion cuyo `origen` apunta a una feature que
no existe, una feature que cerro con `leccion: ninguna` en un mes de correcciones
constantes, una entrada del perfil que cita la #14 cuando esa evidencia ya se
reescribio. Los tres almacenes pueden estar internamente sanos y ser incoherentes
entre si, y hoy no hay forma de verlo.

DESPUES: `sh harness_cli journey` dibuja la linea de tiempo de lo que el proyecto
aprendio, con sus enlaces, y **senala los huecos**. Es la vista que responde "¿que
sabe este proyecto, desde cuando, y que quedo suelto?" — util para entender el
arco y, sobre todo, para encontrar lo que hay que corregir.

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES

leccion list      -> 6 lecciones        sh harness_cli journey
perfil show       -> 4 entradas           |__ 2026-08-16
status            -> 21 features          |     #17 lecciones_memoria_procedural
  (tres vistas separadas)                 |       `-- leccion: docs-generados-por-el-instalador
                                          |       `-- leccion: hitos-del-prd (origen)
los enlaces entre ellos:                  |__ 2026-08-17
  `__ no los ve nadie                     |     #20 buscar_en_el_historial
                                          |       `-- leccion: criterios-de-cierre... (usada 1 vez)
los huecos:                               |     perfil: "Ante un gate, prefiere bloquear..." (#17, #19)
  `__ no los ve nadie                     `__ [!] huecos: 1 feature cerro sin leccion
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero ver en una sola vista que aprendio el proyecto y cuando,
  sin cruzar tres comandos a mano.
- P1: Como Alan, quiero que me senale los **huecos**: enlaces rotos, features que
  cerraron sin dejar nada, lecciones huerfanas. Eso es lo que hay que corregir.
- P1: Como cualquiera, quiero que podar siga pasando por los comandos que ya
  tienen sus garantias, y no por una puerta nueva que las saltee.
- P2: Como script, quiero `--json` con los nodos y sus enlaces.
- P2: Como alguien que llega al proyecto, quiero entender el arco de decisiones
  sin leer 21 features.

## Criterios de aceptacion (Given/When/Then)

### La linea de tiempo

- AC-1: Given un repo con lecciones, perfil y features cerradas, When corre
  `sh harness_cli journey`, Then se imprime una linea de tiempo **cronologica**
  agrupada por fecha, con tres tipos de nodo: **feature cerrada**, **leccion** y
  **entrada de perfil**.
- AC-2: Given una feature cerrada que declaro leccion, Then su nodo muestra la
  leccion declarada como hijo; y una leccion cuyo `origen` cita esa feature
  aparece tambien bajo ella, marcada como `origen` para distinguirla de la
  declarada.
- AC-3: Given una leccion con usos, Then su nodo dice cuantas veces se uso y
  cuando fue la ultima: es lo que distingue lo vivo de lo que solo esta escrito.
- AC-4: Given una entrada del perfil que cita features (`(#14, #16)`), Then
  aparece en la fecha de la feature mas reciente que cita, con esas citas
  visibles.
- AC-5: Given lecciones archivadas, Then aparecen marcadas como archivadas y **no**
  se mezclan con las activas.

### Los huecos (lo que hace util al mapa)

- AC-6: Given una leccion cuyo `origen` cita una feature que **no existe** en el
  backlog, Then se reporta como enlace roto, nombrando la leccion y el id.
- AC-7: Given una entrada del perfil que cita una feature inexistente, Then se
  reporta igual.
- AC-8: Given una feature cerrada como `done` **sin** declaracion de leccion (ni
  clase ni `ninguna`), Then se reporta como hueco: cerro sin decidir si dejaba
  algo.
- AC-9: Given una leccion **huerfana** (sin `origen`, o con `origen` vacio), Then
  se reporta: no se sabe de donde salio.
- AC-10: Given que no hay ningun hueco, Then se dice explicitamente que el mapa
  esta coherente, en vez de callar.

### Solo lectura, y una sola puerta para podar

- AC-11: Given cualquier invocacion de `journey`, Then **no se escribe nada**: ni
  en `docs/`, ni en `progress/`, ni en el hub. No existe `journey delete` ni
  `journey edit`.
- AC-12: Given un nodo que el usuario quiera corregir, Then el mapa imprime **el
  comando exacto** que corresponde a ese tipo de nodo (`lecciones archivar
  <clase>`, `perfil remove --old "..." --yes`, `leccion show <clase>`), en vez de
  ofrecer una via propia. Podar sigue pasando por los comandos que ya tienen sus
  garantias.
- AC-13: Given `--json`, Then se exponen `nodos` (con `tipo`, `id`, `fecha`,
  `titulo`, `detalle`) y `enlaces` (con `desde`, `hacia`, `clase`), mas la lista
  de `huecos` con su tipo y su motivo.

### Limites y degradacion

- AC-14: Given el hub PostgreSQL caido o no configurado, When corre `journey`,
  Then el comportamiento y los exit codes son identicos: el mapa se arma **solo
  con archivos** (`feature_list.json`, `docs/lecciones/`, `docs/perfil-usuario.md`).
- AC-15: Given un proyecto sin lecciones ni perfil ni features cerradas, When
  corre `journey`, Then se dice que todavia no hay nada que mapear y exit **0**.
- AC-16: Given cualquier archivo ilegible o con frontmatter roto, When corre
  `journey`, Then se saltea sin abortar y se cuenta entre los huecos.

### Docs y verificacion

- AC-17: Given `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md`
  (+ plantilla) y las superficies, Then documentan `journey` como vista de solo
  lectura, y explican que podar se hace con los comandos de cada almacen.
- AC-18: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test` y `cargo clippy --all-targets -- -D warnings` estan verdes con
  tests de: el orden cronologico, cada tipo de enlace, cada tipo de hueco, el
  caso sin huecos, `--json`, el repo vacio y la independencia del hub; y
  `tests/setup_smoke.sh` sigue verde.

## Los datos que se tocan

- **disparador**: el comando `journey`, invocado a mano o por un agente.
- **interruptor**: ninguno. Es de **solo lectura** y no tiene estado.
- **candado**: no aplica — no hay escritura que repetir.
- **fuentes** (todas archivos, ninguna nueva): `feature_list.json` (features
  cerradas, su fecha y su `leccion` declarada), `docs/lecciones/*.md` (frontmatter:
  `origen`, `usos`, `ultimo_uso`, `relacionadas`, `estado`),
  `docs/lecciones/archivo/*.md` y `docs/perfil-usuario.md` (entradas con sus citas
  `(#n)`).
- **lo que NO se toca**: absolutamente nada. `journey` no escribe ni un byte y no
  puede borrar nada.

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien pide el mapa

  ¿hay algo que mapear?   -> si no, lo decimos y salimos con 0

  leemos las tres fuentes (todas archivos)
  armamos los nodos con su fecha
  tejemos los enlaces: feature -> leccion declarada
                       feature -> leccion que la cita como origen
                       feature -> entrada de perfil que la cita
                       leccion -> leccion relacionada

  buscamos los HUECOS: citas a features que no existen,
                       features cerradas sin declarar nada,
                       lecciones sin origen,
                       archivos ilegibles

  ENTONCES imprimimos la linea de tiempo y los huecos,
           con la restriccion de que para CORREGIR cualquier cosa
           imprimimos el comando del almacen que corresponde,
           porque este comando no escribe nada.
```

**Promesas:** no escribe · no borra · no hay segunda puerta para podar · no
depende del hub · sin huecos lo dice.

## No funcionales

- **SLOs**: lee las mismas decenas de archivos que `lecciones status`:
  milisegundos, sin red ni hub.
- **Seguridad**: solo lectura. No hay entrada del usuario que se interpole en
  ningun comando: los comandos sugeridos se imprimen como texto, no se ejecutan.
- **Observabilidad**: exit 0 con o sin huecos (un hueco es informacion, no un
  error); exit 2 solo por uso invalido.

## Fuera de alcance

- Cualquier escritura, borrado o edicion. Ver OBS-2 y OBS-3.
- Un render grafico o interactivo: la salida es texto para terminal y `--json`.
- La consolidacion con LLM (#28).
- Reemplazar `lecciones status`, que responde otra pregunta (salud de la
  biblioteca, no el arco de lo aprendido).

## Observaciones (decisiones pendientes)

Todas decididas por Alan el 2026-08-17, en el mismo acto de aprobacion del spec.
No queda ninguna observacion abierta: el implementer puede avanzar sin preguntar.

Dos de las cinco vuelven a ser **correcciones al backlog**, y las dos reducen el
alcance de la feature.

- OBS-1: ¿De donde salen los datos? — **DECIDIDO: solo archivos.** El backlog
  decia "Memory Hub y graphify", pero los enlaces que el mapa necesita (origen,
  usos, citas, fechas de cierre) viven en el frontmatter de las lecciones, en el
  perfil y en `feature_list.json`. El hub guarda eventos, no esos enlaces. Misma
  correccion que la #20. Vinculante para AC-14.
- OBS-2: ¿Se implementa `journey delete`? — **DECIDIDO: no.** La #21 establecio
  que el arnes **nunca borra** una leccion (archivar es mover, con backup y
  rollback) y la #19 que **nada sale del perfil sin `--yes`**. Un `journey delete`
  seria una **segunda puerta** capaz de saltear las dos, que es exactamente el
  riesgo descrito en la leccion `promesas-estructurales-vs-disciplina`. En su
  lugar, el mapa imprime el comando del almacen que corresponde. Vinculante para
  AC-11 y AC-12.
- OBS-3: ¿Se implementa `journey edit`? — **DECIDIDO: no.** En una sesion de
  agente no hay editor interactivo, y para el perfil saltearia el escaneo de
  seguridad de la #19. El mapa imprime la ruta y el usuario edita como quiera.
- OBS-4: ¿Aparecen las features cerradas sin leccion? — **DECIDIDO: si**, y
  cuentan como hueco si no declararon ni `ninguna`. Una feature que cerro sin
  decidir nada es justo lo que hay que ver. Vinculante para AC-8.
- OBS-5: ¿Donde se ubica una entrada del perfil? — **DECIDIDO: en la fecha de la
  feature mas reciente que cita**, porque es cuando la preferencia quedo
  confirmada. Vinculante para AC-4.

**Nota de alcance**: con OBS-2 y OBS-3 decididas, esta feature entrega **menos**
de lo que decia el backlog: es una vista de solo lectura, no un editor. Se deja
escrito para que nadie lea el backlog dentro de seis meses y crea que falto algo.
