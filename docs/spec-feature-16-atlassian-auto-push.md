# Spec - Feature #16: atlassian_auto_push

Estado: approved
Aprobado: 2026-08-16T05:03:26Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #16 en el chat (18 AC: worker detached, publish en cada transicion, --kind bug/feature/task, interruptor de tres niveles). Decisiones OBS-1..OBS-9 registradas
Plan: docs/plan-feature-16-atlassian-auto-push.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: la feature #15 dejo el arnes y Atlassian hablando el mismo idioma, pero
la conversacion tiene dos mitades y solo una es automatica. Alan cierra una
feature, el arnes anota fielmente lo que deberia existir del otro lado... y ahi
se queda, en `progress/atlassian/outbox/`. El board sigue mostrando la historia
en In Progress hasta que alguien se acuerda de escribir `atlassian apply`. Con
Confluence es peor: el PRD y el spec no se publican en ninguna transicion, solo
con `atlassian publish` a mano. O sea: volvimos al problema original — el
estado real vive en la terminal y el equipo ve una foto vieja — con un paso
manual mas para olvidarse. Y cuando lo que se carga es un bugfix, entra a Jira
como `Story`, porque el arnes no sabe distinguir un bug de una feature.

DESPUES: no hay segunda mitad que recordar. Alan cierra la feature y el board
se actualiza solo; aprueba un spec y la pagina de Confluence ya esta al dia;
carga un bugfix con `--kind bug` y en Jira aparece como `Bug`, no como historia.
El comando del flujo sigue siendo instantaneo: el envio ocurre en un worker
detached — el mismo patron que ya usa graphify — asi que si Atlassian esta lento
o caido, Alan no espera ni se entera en el momento; lo pendiente queda en la
outbox y el proximo comando lo reintenta. Nada de esto pasa en un repo sin
binding, ni en uno sin token (ahi la outbox espera al agente con MCP, como
hoy), ni cuando se apaga el interruptor.

## Hoy -> Como va a funcionar

```
HOY                                      DESPUES
add     -> intent en la outbox           add     -> intent + worker detached
start   -> intent en la outbox           start   -> intent + worker detached
advance -> intent en la outbox           advance -> intent + worker detached
approve -> intent en la outbox           approve -> intent + worker detached
close   -> intent en la outbox           close   -> intent + worker detached
prd add -> (nada)                        prd add -> intent del epic + worker
                                              |
   ...y despues, a mano:                      |__ harness atlassian-worker (oculto)
   sh harness_cli atlassian apply             |     |__ apply   (intents pendientes)
   sh harness_cli atlassian publish           |     |__ publish (PRD/SDD/specs, por hash)
                                              |
                                              |__ lock: una sola corrida a la vez
                                              |__ log: progress/atlassian/last-push.log
```

El worker es el MISMO `apply` + `publish` de la feature #15, no una segunda
implementacion: se invoca el codigo que ya esta probado. Lo unico nuevo es
quien lo dispara y que no bloquea.

## Recorridos de usuario (priorizados)

- P1: Como Alan cerrando una feature, quiero que el board quede al dia sin que
  yo escriba un comando extra, para que nadie vea una foto vieja del proyecto.
- P1: Como Alan aprobando un spec, quiero que la pagina de Confluence se
  actualice sola, para que el equipo lea la version vigente y no la del lunes.
- P1: Como Alan cargando un bugfix, quiero que entre a Jira como `Bug`, para no
  tener que corregir el tipo a mano despues.
- P1: Como Alan trabajando con la red lenta o Atlassian caido, quiero que mis
  comandos sigan siendo instantaneos y que nada se pierda, para que la
  integracion no sea un impuesto sobre el flujo.
- P2: Como Alan en una tarea sensible, quiero poder apagar el envio automatico
  sin desconfigurar el binding, para trabajar sin publicar nada por un rato.
- P2: Como Alan revisando que paso, quiero ver el resultado del ultimo envio,
  para entender por que algo no llego sin tener que reproducirlo.

## Criterios de aceptacion (Given/When/Then)

### Envio automatico

- AC-1: Given un repo con binding activo y token, When corro `add`, `start`,
  `advance`, `approve-spec` o `close`, Then el comando lanza el worker detached
  y retorna sin esperarlo, y al terminar el worker los intents de esa
  transicion quedan aplicados en Jira (verificable con `atlassian status`).
- AC-2: Given cualquiera de esas transiciones, When el worker esta corriendo o
  falla, Then el comando del flujo conserva su exit code y su salida de siempre
  (la unica diferencia es una linea informativa), y no queda ningun proceso
  colgando de la terminal.
- AC-3: Given un repo con binding y token, When corro `prd add --name <parte>`,
  Then se emite el intent del epic de ese PRD nuevo y el worker lo crea, sin
  esperar a que se le cargue la primera feature.
- AC-4: Given dos comandos del flujo casi simultaneos, When ambos lanzan el
  worker, Then el lock deja correr uno solo y el segundo no duplica nada ni
  falla (mismo patron que el lock de graphify).
- AC-4b: Given que aparecieron intents nuevos MIENTRAS el worker corria, When
  el worker termina su pasada, Then vuelve a mirar la outbox y los aplica antes
  de soltar el lock, de modo que nada quede esperando al proximo comando.
- AC-5: Given que el worker termino, When miro
  `progress/atlassian/last-push.log`, Then encuentro que se aplico, que se
  publico y que fallo (si algo fallo), con timestamp y sin exponer el token.

### Confluence en cada transicion

- AC-6: Given un repo con binding, token y space, When ocurre cualquiera de las
  transiciones del flujo, Then el worker corre tambien la publicacion de PRD,
  SDD y specs, y los documentos que no cambiaron NO generan version nueva (el
  hash de la #15 es el candado).
- AC-7: Given que edito el cuerpo de un spec o de un PRD y despues corro
  cualquier transicion, When el worker publica, Then la pagina correspondiente
  sube exactamente una version con el contenido nuevo.

### Bug, feature o tarea

- AC-8: Given el backlog, When corro `add --name <n> --kind bug`, Then la
  feature queda con `"kind": "bug"` en `feature_list.json` y su intent usa el
  tipo `issue_types.bug` de `atlassian.json` (default `Bug`).
- AC-9: Given `add` sin `--kind`, When se crea la feature, Then se comporta
  exactamente como hoy (`kind` ausente o `feature`, tipo `Story`), sin migrar
  ni tocar las features ya cargadas.
- AC-10: Given `--kind` con un valor que no es `feature`, `bug` ni `task`, When
  corro `add`, Then falla con exit 2 y un mensaje que lista los validos.
- AC-11: Given un proyecto Jira sin el tipo configurado (por ejemplo sin `Bug`),
  When el worker intenta crear el issue, Then el intent queda pendiente con el
  error legible de Jira y el resto de los intents se aplican igual.

### Interruptor y ausencias

- AC-12: Given un repo con binding pero SIN token, When ocurre una transicion,
  Then no se lanza ningun worker, el intent queda en la outbox como hoy y se
  avisa una sola vez como drenarlo con el agente (`atlassian drain`).
- AC-13: Given `atlassian.json` con `"auto": false`, When ocurre una
  transicion, Then se emite el intent pero NO se empuja nada, y `atlassian
  status` muestra el envio automatico como apagado.
- AC-14: Given `HARNESS_ATLASSIAN_AUTO=0` en el entorno, When ocurre una
  transicion, Then el envio automatico se salta para esa corrida sin tocar la
  configuracion del repo.
- AC-15: Given un repo SIN binding, When corro cualquier comando del flujo,
  Then se comporta exactamente como hoy: sin outbox, sin worker, sin cambios de
  exit code (la garantia de la #15 sigue en pie).

### Validacion del binding (a que proyecto y space apunta esto)

- AC-18: Given un token disponible, When corro `atlassian bind` (o el instalador
  con los flags de Atlassian), Then el arnes verifica contra la API que el
  proyecto Jira existe y que el space de Confluence existe, y lo informa; sin
  token, la verificacion se salta con un aviso (no se puede validar sin
  credenciales) y el binding se escribe igual.
- AC-19: Given un proyecto o un space inexistente (o sin permiso), When corro
  `bind` con token, Then el mensaje dice exactamente cual de los dos falta, con
  la respuesta de Atlassian, y ofrece las dos salidas: crearlo en la UI, o
  repetir el comando con `--create-project` / `--create-space`.
- AC-20: Given `atlassian status` con token, When lo corro, Then valida el
  binding contra la API y muestra si el proyecto y el space siguen existiendo;
  si la red falla, lo dice y NO cambia su exit code (status sigue siendo un
  comando informativo).
- AC-21: Given `bind --create-project` con token y permisos, When el proyecto no
  existe, Then el arnes lo crea (`POST /rest/api/3/project`, tipo `software`,
  plantilla scrum team-managed, con el usuario del token como lead) y sigue con
  el binding; sin el flag NUNCA crea nada.
- AC-22: Given `bind --create-space` con token y permisos, When el space no
  existe, Then el arnes lo crea (`POST /wiki/api/v2/spaces`) y sigue con el
  binding; sin el flag NUNCA crea nada.
- AC-23: Given que falta el permiso de administracion, When uso `--create-project`
  o `--create-space`, Then el error de Atlassian (403) se muestra tal cual, con
  la indicacion de crearlo en la UI, y el binding queda escrito igual para no
  perder lo configurado.

### Backfill: el primer push carga lo que ya existe

- AC-24: Given un repo con historia previa (PRDs escritos y features en el
  backlog) que activa el binding, When ocurre la primera transicion, Then el
  arnes emite tambien los intents de lo YA existente: un epic por cada PRD del
  arbol y una historia por cada feature del backlog, con su estado actual
  (`pending` -> To Do, `in_progress` -> In Progress, `done` -> Done).
- AC-25: Given que el backfill ya corrio, When ocurre la siguiente transicion,
  Then no se vuelve a emitir nada de lo cargado (el dedupe de la #15 es el
  candado) y el push sigue siendo incremental.
- AC-26: Given que quiero re-cargar despues de limpiar Jira a mano, When corro
  `atlassian backfill`, Then se emiten los intents faltantes segun el estado
  actual del backlog, sin duplicar lo que sigue mapeado en `state.json`.
- AC-27: Given el backfill de una feature ya cerrada, When se aplica, Then su
  historia queda en el estado que le corresponde (`done` -> Done) y sus
  subtasks AC-n tambien se crean, para que el board sea espejo del repo y no un
  resumen (OBS-14).
- AC-29: Given un proyecto Jira que ya tiene epics creados a mano, When el
  arnes va a crear el epic de un PRD, Then busca primero uno con el MISMO
  titulo en ese proyecto y, si existe, lo adopta (guarda su clave en
  `state.json`) en vez de crear un duplicado.
- AC-30: Given que no hay ningun epic con ese titulo, When se aplica el intent,
  Then se crea uno nuevo como hasta ahora.
- AC-28: Given `atlassian backfill --sin-acs`, When lo corro, Then se cargan
  epics, historias y estado SIN las subtasks de los AC-n (escape para repos
  grandes que no quieren ese volumen).

### Verificacion

- AC-16: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh`, Then los cuatro terminan limpios, con tests nuevos para
  el interruptor, el lock, `--kind` y la ausencia de token.
- AC-17: Given el sitio real `calpil.atlassian.net`, When corro un ciclo
  completo en un repo fixture (incluyendo un `--kind bug`), Then los issues, la
  transicion y las paginas aparecen SIN haber corrido `apply` ni `publish` a
  mano, y queda registrado en `docs/impl-16.md`.

## Los datos que se tocan

- disparador: las mismas transiciones que ya emiten intents (`add`, `start`,
  `advance`, `approve-spec`, `close`) mas `prd add`, que hoy no emite nada.
- interruptor: tres niveles, de mayor a menor prioridad —
  `HARNESS_ATLASSIAN_AUTO=0` (esta corrida), `"auto": false` en
  `atlassian.json` (este repo), y la ausencia de token (sin credenciales no hay
  envio posible: la outbox espera al agente).
- candado: `progress/atlassian/.push.lock` (directorio, igual que el lock de
  graphify) para que dos comandos seguidos no disparen dos workers, mas el
  dedupe por clave que ya trae la #15.
- `feature_list.json` gana UN campo opcional: `kind` (`feature` | `bug` |
  `task`). Su ausencia significa `feature`: las 15 features ya cargadas no se
  migran ni se tocan.
- `atlassian.json` gana `issue_types.bug` (default `Bug`), `issue_types.task`
  (default `Task`) y `auto` (default `true`).
- `progress/atlassian/last-push.log`: que hizo el ultimo worker. Se sobrescribe
  en cada corrida (no es bitacora: la bitacora es `progress/history.md`).

## Pseudo-codigo (el acuerdo)

```
CUANDO una transicion del flujo termina bien

  emitimos el intent como hoy (feature #15)

  ¿hay binding activo?            -> si no, no hacemos nada
  ¿hay token?                     -> si no, avisamos como drenar y listo
  ¿el interruptor esta encendido? -> si no, no hacemos nada
  ¿el lock esta libre?            -> si no, no hacemos nada (ya hay uno corriendo)

  ENTONCES lanzamos el worker detached y volvemos INMEDIATAMENTE,
           con la restriccion de que nada de esto puede cambiar el exit code
           ni la salida del comando del flujo.

CUANDO corre el worker (proceso aparte)

  aplica los intents pendientes  (el `apply` de la #15)
  publica PRD, SDD y specs       (el `publish` de la #15, que salta lo que no cambio)

  escribe que paso en last-push.log y libera el lock SIEMPRE,
  con la restriccion de que un fallo deja el intent pendiente para el proximo
  comando: nunca se pierde, nunca se duplica.

CUANDO se carga una feature con `--kind`

  guardamos el tipo en el backlog y el intent usa el tipo de issue que le
  corresponde, con la restriccion de que sin `--kind` todo sigue igual que hoy.
```

Promesas: el flujo nunca espera a la red · lo pendiente nunca se pierde ·
apagarlo es una linea · sin token o sin binding, cero cambios.

## No funcionales

- SLOs: lanzar el worker no agrega mas de 10 ms al comando del flujo (es un
  `spawn` y un `mkdir` de lock); el worker no tiene limite de tiempo propio
  porque nadie lo espera, pero hereda los timeouts HTTP de la #15 (30 s por
  request).
- Seguridad (Articulo 4): el worker hereda el entorno pero NUNCA escribe el
  token en `last-push.log` ni en la salida; el log se escribe con los mismos
  permisos que el resto de `progress/`.
- Observabilidad: `atlassian status` suma dos lineas — si el envio automatico
  esta encendido y cuando fue el ultimo intento; `last-push.log` guarda el
  detalle; `progress/history.md` no cambia (sigue siendo la bitacora del flujo).

## Fuera de alcance

- Reintentos programados: si el worker falla, el reintento ocurre en la
  proxima transicion del flujo (o a mano con `atlassian apply`). No hay cron ni
  demonio.
- Crear proyectos o spaces sin pedirlo: solo con `--create-project` /
  `--create-space` (OBS-11).
- Sincronizacion desde Jira hacia el arnes: sigue siendo un solo sentido.
- Reutilizar epics existentes en vez de crear uno por PRD: es otra feature (hoy
  cada PRD crea su epic).
- Tipos de issue mas alla de `feature`, `bug` y `task`.

## Observaciones (decisiones pendientes)

- OBS-1 [DECIDIDA por el USUARIO, 2026-08-16]: worker detached con lock, el
  mismo patron que `graphify::refresh_bg`, para que el flujo nunca espere a la
  red.
- OBS-2 [DECIDIDA por el USUARIO, 2026-08-16]: la publicacion en Confluence
  corre en CADA transicion (no solo al cerrar), apoyada en el hash de la #15
  para no generar versiones inutiles.
- OBS-3 [DECIDIDA por el USUARIO, 2026-08-16]: el tipo se declara explicito con
  `add --kind bug|feature|task`; sin heuristica por nombre.
- OBS-4 [DECIDIDA por el USUARIO, 2026-08-16]: el envio automatico viene
  ENCENDIDO cuando hay binding y token, y se apaga con `"auto": false` o
  `HARNESS_ATLASSIAN_AUTO=0`.
- OBS-6 [DECIDIDA por el USUARIO, 2026-08-16]: al activar el binding en un repo
  con historia previa NO se carga nada retroactivo: los intents nacen en las
  transiciones de ahi en adelante. Sin comando de backfill.
- OBS-7 [DECIDIDA por el USUARIO, 2026-08-16]: el publish automatico recorre
  TODO el arbol (todos los PRDs y todos los specs), apoyado en el hash para no
  tocar la red cuando nada cambio; asi un documento editado por fuera del flujo
  igual llega a la wiki.
- OBS-8 [DECIDIDA por el USUARIO, 2026-08-16]: un solo interruptor (`auto`)
  para Jira y Confluence juntos; para algo puntual estan `apply` y `publish` a
  mano.
- OBS-9 [DECIDIDA por el USUARIO, 2026-08-16]: el worker hace una segunda
  pasada antes de soltar el lock si aparecieron intents nuevos mientras corria
  (AC-4b).
- OBS-10 [DECIDIDA por el USUARIO, 2026-08-16]: el binding se valida contra la
  API cuando hay token, tanto al configurarlo (`bind` / instalador) como en cada
  `atlassian status`. Sin token no se valida (no se puede) y no se bloquea nada.
- OBS-11 [DECIDIDA por el USUARIO, 2026-08-16]: si el proyecto o el space no
  existen, el arnes OFRECE crearlos. Como ni el CLI ni el instalador son
  interactivos, "ofrecer" se implementa como en `approve-spec --yes`: el
  mensaje explica la opcion y la creacion ocurre SOLO con el flag explicito
  (`--create-project` / `--create-space`). El arnes jamas crea estructura
  organizacional por su cuenta.
- OBS-12 [DECIDIDA por el USUARIO, 2026-08-16 — REEMPLAZA a OBS-6]: al activar
  el binding en un repo con historia, Jira TAMBIEN se carga completo en el
  primer push (epics de los PRDs existentes + historias de las features del
  backlog con su estado). La decision anterior era "solo lo nuevo"; el usuario
  la cambio para que board y wiki queden consistentes desde el arranque.
- OBS-13 [DECIDIDA por el USUARIO, 2026-08-16]: sin umbral ni confirmacion
  previa para el primer push masivo: empuja todo lo que corresponda desde la
  primera transicion.
- OBS-14 [DECIDIDA por el USUARIO, 2026-08-16]: "la idea es que este
  sincronizado el proyecto local con Jira y Confluence SIEMPRE". El backfill
  carga TODO: epics de los PRDs, historias de las features, sus subtasks AC-n
  (tambien las de features cerradas) y el estado de cada una. Se descarta la
  propuesta del implementer de omitir las subtasks de lo ya cerrado: el
  principio es que el board sea espejo del repo, no un resumen.
  Consecuencia asumida: en un repo grande el primer push crea muchos issues
  (en este mismo arnes serian ~16 historias y ~240 subtasks). El escape para
  quien no lo quiera es `atlassian backfill --sin-acs`, que carga epics,
  historias y estado sin bajar los AC-n.
- OBS-15 [DECIDIDA por el USUARIO, 2026-08-16]: los epics ya existentes en el
  proyecto se REUTILIZAN. Antes de crear el epic de un PRD, el arnes busca por
  titulo exacto en ese proyecto (JQL) y adopta el que encuentre. Motivo: SCRUM
  ya tiene 21 epics escritos a mano y el backfill crearia un juego paralelo que
  habla de lo mismo.
- OBS-5 [REGISTRADA, sin accion]: `close` tambien dispara el worker, asi que el
  ultimo push de una feature ocurre despues de que el comando ya devolvio. Si
  alguna vez hace falta confirmar el envio ANTES de cerrar la sesion, el
  remedio es correr `atlassian apply` a mano, que sigue existiendo y es
  sincrono.
