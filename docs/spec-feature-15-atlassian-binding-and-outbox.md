# Spec - Feature #15: atlassian_binding_and_outbox

Estado: approved
Aprobado: 2026-08-16T03:15:43Z por USUARIO (confirmacion explicita) - Alan aprobo el spec ampliado de la feature #15 en el chat (25 AC: binding, outbox, ejecutor MCP, ejecutor REST con token, sprints via Agile API y Confluence PRD/SDD/specs). Decisiones OBS-1..OBS-10 registradas: hibrido, todo junto, Story por default, flag Impediment, ureq+ADR, subconjunto Markdown->storage
Plan: docs/plan-feature-15-atlassian-binding-and-outbox.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan instala el arnes en `realestate` y trabaja como siempre: carga la
feature, arranca, avanza, cierra. Todo ese movimiento queda en
`feature_list.json`, en `progress/history.md` y en `docs/`. Es decir: dentro de
la terminal. Del otro lado, en `calpil.atlassian.net`, el proyecto de Jira
cuenta otra historia — una que alguien tuvo que escribir a mano, en otro
momento, con otras palabras. Cuando pasan tres dias, el board y el arnes ya no
dicen lo mismo, y nadie sabe cual de los dos miente. El PRD y el SDD, que son
los documentos que explican POR QUE existe cada feature, viven en `docs/prd/`
donde solo los ve quien clona el repo. Si Alan no esta frente a la terminal, el
estado real del desarrollo es invisible: el equipo pregunta "como viene esto" y
la respuesta honesta es "tendria que mirar el repo".

DESPUES: el arnes y Atlassian son el mismo relato contado en dos lugares. Al
instalar, el arnes aprende a que proyecto Jira y a que space de Confluence
pertenece ESE repo — no lo adivina: o se lo dicen por flag/config, o pregunta y
no sigue hasta saberlo. Desde ahi, cada movimiento del flujo deja rastro solo:
el PRD nace como epic, la feature como historia bajo ese epic, cada AC-n del
spec como subtask, y el estado de la historia sigue al estado real del
desarrollo (`pending` -> To Do, `start` -> In Progress, `close` -> Done,
`blocked` -> flag Impediment). El `advance` deja su comentario con la
evidencia; la aprobacion del spec, el suyo. Cuando hay sprint abierto, la
historia entra al sprint; y si no hay ninguno, se queda en el backlog sin
romper nada. En Confluence, el PRD maestro, sus PRDs anidados, el SDD y cada
spec quedan publicados como un arbol de paginas que se actualiza — nunca se
duplica — y cada pagina enlaza a su issue y cada issue a su pagina. Alan cierra
la feature en la terminal y el board, el sprint y la wiki ya lo reflejan. Y en
el repo que no configuro nada, no cambia absolutamente nada: el arnes se
comporta exactamente como hoy.

## Hoy -> Como va a funcionar

```
HOY                                  DESPUES
add    -> feature_list.json          add    -> feature_list.json + intent(epic/historia)
start  -> plan + spec draft          start  -> idem + intent(In Progress + subtasks AC-n + sprint)
advance-> progress/history.md        advance-> idem + intent(comentario con la nota)
approve-> sello en el spec           approve-> idem + intent(comentario "spec aprobado")
close  -> estado archivado           close  -> idem + intent(Done + nota de cierre)
                                          |
                                          |__ progress/atlassian/outbox/*.json  (append-only)
                                          |
                                          |__ DOS EJECUTORES, UNA MISMA OUTBOX
                                          |     (a) sin token: `atlassian drain` imprime el plan
                                          |         de llamadas y el agente con MCP lo ejecuta,
                                          |         devolviendo las claves con `atlassian ack`
                                          |     (b) con token: `atlassian apply` ejecuta REST
                                          |         (Jira v3 + Agile 1.0 + Confluence v2) solo
                                          |
                                          |__ progress/atlassian/state.json (mapa local -> remoto)

publicar docs -> (no existia)        `atlassian publish` -> PRD master + PRDs anidados + SDD +
                                      specs como arbol de paginas, actualizando por titulo
sprints       -> (no existia)        `atlassian sprint start|close` -> Agile API (el MCP no puede)
```

El binario no habla MCP: el MCP vive en el agente. Por eso el arnes escribe la
intencion y hay dos maneras de ejecutarla. La outbox es el contrato comun, para
que la ruta con agente y la ruta con token produzcan exactamente lo mismo.

## Recorridos de usuario (priorizados)

- P1: Como Alan instalando el arnes en un repo, quiero decirle a que proyecto
  Jira y a que space de Confluence pertenece ese repo, para que el arnes sepa
  donde publicar sin que yo se lo repita en cada comando.
- P1: Como Alan trabajando una feature, quiero que add/start/advance/close
  dejen su rastro en Jira sin que yo abra el navegador, para que el board diga
  la verdad sin trabajo extra.
- P1: Como Alan en un repo sin binding, quiero que el arnes se comporte igual
  que hoy, para que la integracion no sea un impuesto para todos los proyectos.
- P1: Como Alan con un token configurado, quiero que el arnes publique solo,
  sin agente en el medio, para que el board no dependa de que yo abra una
  sesion de chat.
- P1: Como Alan planificando, quiero abrir un sprint desde la terminal y que
  las features que arranco entren en el, para no administrar el sprint a mano.
- P1: Como cualquiera del equipo, quiero leer el PRD y el SDD en Confluence,
  para entender el porque sin clonar el repo.
- P2: Como Alan retomando despues de dias, quiero ver que quedo sin publicar y
  poder reintentarlo, para que una caida de red no me deje el board a medias.
- P2: Como agente (Claude/Codex/Kimi/Gemini) con MCP de Atlassian, quiero un
  plan de llamadas explicito y sin ambiguedad, para ejecutarlo sin inventar
  campos ni claves.

## Criterios de aceptacion (Given/When/Then)

### Binding (a que proyecto y space pertenece este repo)

- AC-1: Given un repo destino, When corro el instalador con
  `--atlassian-site <host> --jira-project <KEY> --confluence-space <KEY>`,
  Then queda escrito `atlassian.json` en la raiz del proyecto con site,
  proyecto, space y el mapeo de tipos, y el instalador lo reporta en su salida.
- AC-2: Given un repo destino con `.harness.env` que define
  `HARNESS_ATLASSIAN_SITE`, `HARNESS_JIRA_PROJECT` y `HARNESS_CONFLUENCE_SPACE`,
  When corro el instalador sin flags, Then toma esos valores (misma precedencia
  que el resto de la config: flag > config file > nada) y escribe el binding.
- AC-3: Given un repo destino sin flags ni config de Atlassian, When corro el
  instalador, Then NO escribe `atlassian.json`, NO falla, y deja constancia de
  que la integracion queda apagada indicando el comando para activarla
  (`harness_cli atlassian bind`).
- AC-4: Given un repo sin binding, When corro add/start/advance/approve-spec/
  close, Then el arnes se comporta exactamente como hoy: no crea outbox, no
  escribe estado nuevo y ningun comando cambia su exit code.
- AC-5: Given un repo sin binding, When corro `harness_cli atlassian bind` sin
  `--jira-project`, Then el comando se niega con exit 2 y un mensaje que dice
  exactamente que falta preguntarle al USUARIO (a que proyecto Jira y a que
  space pertenece este repo), sin adivinar ninguno de los dos.
- AC-13: Given el instalador de Windows, When corro `setup_harness.ps1` con los
  mismos flags, Then produce el mismo `atlassian.json` que la version Bash
  (paridad verificada por assert de contenido en el smoke ps1).

### Outbox (el contrato entre el flujo y Atlassian)

- AC-6: Given un binding activo, When corro `harness_cli add`, Then la outbox
  gana un intent `feature.create` con el nombre, los acceptance y — si la
  feature cita un PRD — un intent `prd.epic` para ese PRD como padre.
- AC-7: Given un binding activo y una feature con spec que declara AC-1..AC-n,
  When corro `harness_cli start --feature <id>`, Then la outbox gana un intent
  de transicion a In Progress y un intent `ac.subtask` por cada AC-n del spec,
  con el texto del AC como resumen.
- AC-8: Given un binding activo, When corro `advance --nota "<texto>"`,
  `approve-spec --yes` o `close --status done --note "<texto>"`, Then cada uno
  deja su intent (comentario, comentario de aprobacion, transicion Done +
  comentario con la nota) y el historial de Jira queda con la misma bitacora
  que `progress/history.md`.
- AC-11: Given un intent ya aplicado, When vuelvo a correr el mismo comando del
  flujo (o el ejecutor dos veces), Then no se emite ni se propone un duplicado:
  la clave de dedupe (`feature:<id>:create`, `ac:<id>:<n>`, ...) es el candado.
- AC-12: Given un binding activo y una feature cerrada, When corro
  `harness_cli atlassian status`, Then muestra el binding vigente, si hay token
  (solo presente/ausente), el mapeo local -> remoto (feature #15 -> ADR-42) y
  cuantos intents quedan pendientes.

### Ejecutor (a) — agente con MCP, sin token

- AC-9: Given intents pendientes, When corro `harness_cli atlassian drain`,
  Then imprime en JSON el plan de llamadas MCP ordenado por dependencia
  (primero el epic, despues la historia, despues las subtasks) con el nombre
  exacto de la tool y sus argumentos, y no muta nada por su cuenta.
- AC-10: Given que el agente ejecuto el plan, When corro
  `harness_cli atlassian ack --intent <id> --key <ADR-n>`, Then la clave queda
  guardada en `progress/atlassian/state.json`, el intent pasa a aplicado y
  deja de aparecer en `drain`.

### Ejecutor (b) — REST nativo con token

- AC-15: Given `.harness.env` con `HARNESS_ATLASSIAN_EMAIL` y
  `HARNESS_ATLASSIAN_TOKEN`, When corro `harness_cli atlassian apply`, Then el
  arnes ejecuta los intents pendientes contra la API REST (Jira platform v3
  para issues, transiciones y comentarios) y solo marca aplicado lo que la API
  confirmo, guardando la clave devuelta en `state.json`.
- AC-16: Given cualquier salida del arnes (stdout, logs, outbox, state.json,
  commits), When busco el token, Then no aparece en ninguna: `status` informa
  unicamente `token: presente` o `token: ausente`.
- AC-17: Given que la API responde 4xx o 5xx, When corro `apply`, Then el
  intent queda pendiente con el error legible (codigo HTTP + mensaje de
  Atlassian), el comando termina con exit 1 diciendo que hacer, y el siguiente
  `apply` lo reintenta sin duplicar lo ya aplicado.
- AC-18: Given un repo sin token, When corro `apply`, Then se niega con exit 2
  y explica la alternativa (`drain` + agente con MCP), sin dejar estado a medias.

### Sprints (Agile API — el punto ciego del MCP)

- AC-19: Given un binding activo y token, When corro
  `harness_cli atlassian sprint start --name "<nombre>" [--goal <texto>]`,
  Then el arnes resuelve el board del proyecto
  (`GET /rest/agile/1.0/board?projectKeyOrId=<KEY>`), crea el sprint
  (`POST /rest/agile/1.0/sprint` con `originBoardId`), lo activa
  (`PUT /rest/agile/1.0/sprint/{id}` con `state=active`) y guarda su id como
  sprint vigente en `state.json`.
- AC-20: Given un sprint vigente, When arranco una feature con `start`, Then su
  historia entra a ese sprint (`POST /rest/agile/1.0/sprint/{id}/issue`, en
  lotes de hasta 50); y Given que no hay sprint vigente, Then la historia queda
  en el backlog sin error y sin intent colgado.
- AC-21: Given un sprint vigente, When corro `harness_cli atlassian sprint
  close`, Then el sprint pasa a `closed` y la salida lista que historias
  quedaron sin terminar (las que no estan en Done).

### Confluence (PRD, SDD y specs publicados)

- AC-22: Given un binding con space, When corro `harness_cli atlassian publish`,
  Then el PRD maestro se publica como pagina en ese space y cada PRD anidado
  como pagina hija, respetando el mismo arbol que `prd tree`, y el SDD maestro
  como pagina hermana del PRD maestro.
- AC-23: Given que ya publique antes, When corro `publish` de nuevo sin cambiar
  los documentos, Then no se crea ninguna pagina nueva ni version nueva (la
  idempotencia es por titulo dentro del space + hash del contenido en
  `state.json`); y si el documento cambio, se actualiza con
  `PUT /wiki/api/v2/pages/{id}` incrementando `version.number` en uno.
- AC-24: Given una feature con spec, When corro `publish`, Then el spec queda
  publicado como pagina hija del PRD que lo origina (o del PRD maestro si no
  cita ninguno), la pagina enlaza a su issue de Jira y el issue queda con el
  enlace a la pagina.

### Verificacion

- AC-14: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh`, Then los cuatro terminan limpios y los tests nuevos
  cubren binding ausente/presente, dedupe, ack, error de API y idempotencia de
  publish.
- AC-25: Given el sitio real `calpil.atlassian.net`, When ejecuto un ciclo
  completo (bind -> add -> start -> advance -> close -> publish) sobre un repo
  fixture, Then quedan creados el epic, la historia, las subtasks AC-n, los
  comentarios, la transicion a Done y las paginas de PRD/SDD/spec, con sus
  claves e ids registrados en `state.json` como evidencia en `docs/impl-15.md`.

## Los datos que se tocan

- disparador: cada transicion del flujo que hoy ya escribe en `progress/` —
  `add`, `start`, `advance`, `approve-spec`, `close` — mas los comandos
  explicitos `atlassian sprint start|close` y `atlassian publish`.
- interruptor: la existencia de `atlassian.json` con `"enabled": true`. Sin ese
  archivo la integracion no existe; con `"enabled": false` se apaga sin perder
  el binding ni el mapeo ya conseguido.
- candado: la clave de dedupe de cada intent (`feature:<id>:create`,
  `feature:<id>:status:<estado>`, `ac:<id>:<n>`, `prd:<slug>:epic`,
  `page:<ruta-doc>`) contra `progress/atlassian/state.json`. Si la clave ya
  tiene issue o pagina remota, el intent nace aplicado.
- `atlassian.json` (raiz del proyecto, versionable): site, cloud_id, proyecto
  Jira, space de Confluence, mapeo de tipos (`prd -> Epic`,
  `feature -> Story` por default y configurable a `Feature`, `ac -> Subtask`) y
  mapeo de estados (`pending -> To Do`, `in_progress -> In Progress`,
  `done -> Done`, `blocked -> flag Impediment / customfield_10021`).
- `.harness.env` (NO versionado, ya cubierto por `.gitignore`):
  `HARNESS_ATLASSIAN_EMAIL` y `HARNESS_ATLASSIAN_TOKEN` para el ejecutor REST.
- `progress/atlassian/outbox/<ts>-<seq>-<evento>.json` (append-only): un intent
  por archivo, con id, clave de dedupe, tool MCP sugerida, endpoint REST
  equivalente y argumentos.
- `progress/atlassian/state.json`: mapa local -> remoto (features, ACs, PRDs,
  paginas con su version y hash), sprint vigente y cursor de aplicados. Es el
  unico lugar donde se escriben claves e ids remotos.
- NO se toca: `feature_list.json` no gana campos remotos (el mapeo vive
  aparte, para que el backlog siga siendo legible sin Atlassian). El cuerpo de
  los PRD y del SDD tampoco: publicar los lee, nunca los reescribe.

## Pseudo-codigo (el acuerdo)

```
CUANDO una transicion del flujo termina bien (add/start/advance/approve/close)

  ¿existe atlassian.json con enabled=true?  -> si no, no hacemos nada
  ¿el state.json ya tiene esta clave?       -> si si, no hacemos nada

  ENTONCES escribimos UN intent en la outbox describiendo que deberia
           existir del otro lado (issue, subtask, transicion, comentario),
           con la restriccion de que escribir el intent NUNCA puede hacer
           fallar el comando del flujo: si la outbox no se puede escribir,
           se avisa y el comando sigue su curso.

CUANDO alguien pide ejecutar lo pendiente

  ¿hay token?  -> NO: `drain` imprime el plan de llamadas y espera el `ack`
               -> SI: `apply` ejecuta el plan contra la API, en orden de
                      dependencia, y escribe la clave devuelta

  con la restriccion de que solo se marca aplicado lo que el otro lado
  confirmo, y que un fallo deja el intent pendiente con su error legible.

CUANDO se abre un sprint

  resolvemos el board del proyecto, creamos el sprint y lo activamos,
  y lo recordamos como vigente

  ENTONCES cada feature que arranque entra a ese sprint,
           con la restriccion de que sin sprint vigente la historia
           simplemente se queda en el backlog.

CUANDO se publican los documentos

  para cada PRD, SDD y spec:
    ¿existe ya una pagina con ese titulo en el space?
      -> NO: la creamos bajo su padre del arbol
      -> SI: ¿cambio el contenido? -> NO: no tocamos nada
                                   -> SI: actualizamos subiendo la version en uno

  con la restriccion de que jamas reescribimos el documento local:
  el PRD es del USUARIO y Confluence es su reflejo, no su fuente.
```

Promesas: una sola vez por caso (dedupe por clave) · el flujo nunca se rompe
por Atlassian (la emision es best-effort y no bloqueante) · no inventa proyecto
ni space: si no lo sabe, pregunta · el token no se escribe en ningun lado.

## No funcionales

- SLOs: emitir un intent no agrega mas de 5 ms al comando del flujo (escritura
  local de un archivo chico); `drain` con 200 intents pendientes responde en
  menos de 1 s; `apply` procesa los intents en serie con timeout por request y
  reintento acotado.
- Seguridad (Articulo 4): el binding es publico y versionable a proposito —
  solo nombra proyecto y space. Las credenciales van unicamente por entorno o
  `.harness.env` (ignorado por git), nunca en el repo, en la outbox, en el
  state, en los logs ni en los commits (AC-16). Auth Basic email + API token
  sobre HTTPS, sin fallback a HTTP.
- Observabilidad: `atlassian status` es la fuente de verdad (binding, token
  presente/ausente, mapeo, pendientes, sprint vigente); cada intent guarda su
  timestamp y el comando que lo origino; errores accionables con exit codes
  estables (0 ok / 1 fallo de ejecucion / 2 mal uso o falta de binding).

## Fuera de alcance

- Traer cambios DESDE Atlassian hacia el arnes (sincronizacion bidireccional):
  el flujo manda, Jira y Confluence reflejan. Un issue movido a mano en el
  board no reescribe `feature_list.json`.
- Backfill retroactivo de las features ya cerradas (#1..#14) en el repo del
  arnes: ver OBS-3.
- Crear proyectos Jira, boards nuevos o spaces de Confluence: el proyecto, su
  board y el space tienen que existir.
- Estimaciones, asignacion de responsables y worklogs: el arnes no inventa
  story points ni assignees.

## Observaciones (decisiones pendientes)

- OBS-1 [DECIDIDA por el USUARIO, 2026-08-15]: mecanismo hibrido — la outbox es
  el formato canonico, el agente con MCP es un ejecutor y el REST nativo con
  token es el otro; los dos producen el mismo resultado.
- OBS-2 [DECIDIDA por el USUARIO, 2026-08-15]: los sprints los crea el arnes
  via Agile REST API con token, porque el MCP oficial no expone boards ni
  sprints. Verificado en este sitio: asignar un issue a un sprint existente SI
  es posible por MCP (`customfield_10020` acepta `set`); crearlo no.
- OBS-3 [DECIDIDA por el USUARIO, 2026-08-15]: `harness_process` es el repo
  template, asi que el binding vive en el instalador y se resuelve en cada repo
  destino. Este repo no carga sus 14 features en ningun proyecto Jira.
- OBS-4 [DECIDIDA por el USUARIO, 2026-08-15]: space de Confluence por repo,
  publicando PRD + SDD + specs. Para este sitio el default sugerido es `SD`.
- OBS-5 [DECIDIDA por el USUARIO, 2026-08-15]: todo junto en la feature #15 —
  binding, outbox, ejecutor MCP, ejecutor REST, sprints y Confluence. No se
  trocea en tres features.
- OBS-6 [DECIDIDA por el USUARIO, 2026-08-15]: la feature del backlog se
  representa como `Story` por default (unico tipo presente en ADR y en SCRUM),
  configurable a `Feature` en `atlassian.json`.
- OBS-7 [DECIDIDA por el USUARIO, 2026-08-15]: `blocked` se marca con el flag
  `Impediment` (customfield_10021) dejando la historia en su columna.
- OBS-8 [REGISTRADA, sin accion]: los AC-n como subtask replican la convencion
  que el proyecto SCRUM ya usa hoy (Epic -> Story -> Subtask `AC-n · ...`), asi
  que un repo con historia previa en Jira no necesita migrar nada.
- OBS-9 [DECIDIDA por el USUARIO, 2026-08-15]: el ejecutor REST usa `ureq`
  (sincrono, reusa el `rustls` que el binario ya trae) y la feature incluye el
  ADR que justifica la dependencia, como exige el Articulo 6.
- OBS-10 [DECIDIDA por el USUARIO, 2026-08-15]: se convierte un subconjunto
  acotado de Markdown a `storage` (titulos, listas, tablas, bloques de codigo,
  enlaces); lo no soportado se envuelve en bloque de codigo y cada pagina
  enlaza al archivo del repo como fuente de verdad.
