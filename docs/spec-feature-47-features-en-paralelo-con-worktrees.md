# Spec - Feature #47: features_en_paralelo_con_worktrees

Estado: approved
Aprobado: 2026-08-21T21:12:15Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #47 en el chat (25 AC): features en paralelo con rama y worktree por feature (GitFlow), estado vivo por feature con current.md como indice, estado unico resuelto contra el repo principal, foco automatico por worktree y cierre con --to obligatorio que mergea, pushea sin trailers de IA, borra el worktree y conserva la rama. Confirmo los tres puntos senalados: el cierre publica, one_feature_at_a_time deja de bloquear y en este repo todo va a main
Plan: docs/plan-feature-47-features-en-paralelo-con-worktrees.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan tiene dos cosas para hacer y el arnes le contesta siempre lo mismo:
"Ya hay feature in_progress: #45". Una a la vez. Si igual quiere avanzar con la
otra, tiene que cerrar la primera aunque no este terminada, y ahi empieza lo
peor: el estado vivo es UN archivo (`progress/current.md`), asi que cerrar una
feature pisa el estado de la otra — eso ya paso de verdad, y por eso existia la
feature #45. Y si dos agentes trabajan a la vez sobre el mismo checkout, se
pisan los archivos entre ellos: no hay nada que los separe.

DESPUES: cada feature que arranca se lleva su propia rama y su propia carpeta.
Alan puede tener la #47 abierta en una terminal y la #48 en otra, cada una en su
worktree, editando archivos distintos en disco: es imposible que se pisen. El
backlog y la bitacora siguen siendo UNO SOLO — el arnes los lee del repo
principal aunque lo invoques desde un worktree —, pero el estado vivo se parte
en `progress/current-<id>.md`, uno por feature, con `current.md` como indice de
lo que hay abierto. Cuando una feature termina, Alan dice a que rama va (el
arnes no lo adivina: pregunta), y el arnes mergea, publica y limpia la carpeta
de trabajo, dejando la rama por si hay que mirar atras.

## Hoy -> Como va a funcionar

```
HOY                                   DESPUES
start #47 con la #45 abierta          start #47
  -> "Ya hay feature in_progress"       |__ rama feature/47-<slug>  (bugfix/ si kind=bug)
                                        |__ worktree ../<repo>-wt/47-<slug>/
progress/current.md   (UNO)             |__ progress/current-47.md
progress/.last_autocheck (UNO)          |__ progress/.last_autocheck-47
                                        |__ current.md = indice de lo activo

close #47 --status done               close #47 --status done --to <rama>
  -> archiva y resetea current.md         |__ gates de siempre
     (pisando el de la otra feature)      |__ merge de feature/47-<slug> en <rama>
                                          |__ push (sin trailers de IA)
                                          |__ borra el worktree, conserva la rama
                                          |__ conflicto -> abort, nada tocado

trabajar: siempre en el mismo dir     trabajar: dentro del worktree, y los
                                      comandos infieren la feature por la carpeta
```

El estado del arnes NO se bifurca: `feature_list.json` y `progress/` son
siempre los del repo principal (`git rev-parse --git-common-dir`), aunque el
binario se invoque desde un worktree.

## Recorridos de usuario (priorizados)

- P1: Como Alan con dos frentes abiertos, quiero arrancar la segunda feature sin
  cerrar la primera, para no elegir entre terminar algo y atender lo urgente.
- P1: Como Alan con dos agentes trabajando a la vez, quiero que cada uno edite
  su propia copia de los archivos, para que no se pisen las implementaciones.
- P1: Como Alan, quiero que cerrar una feature no toque el estado de las otras,
  para no volver a perder la bitacora de la que sigue viva.
- P1: Como Alan cerrando, quiero decir YO a que rama se integra, para que el
  arnes no publique en develop o en main por su cuenta.
- P2: Como agente trabajando dentro del worktree de la #47, quiero que los
  comandos sepan que estoy en la #47 sin repetir `--feature 47` en cada uno.
- P2: Como Alan en un repo sin git o en una maquina donde no quiero worktrees,
  quiero seguir trabajando como hasta hoy, para que la novedad no sea un
  requisito.

## Criterios de aceptacion (Given/When/Then)

### Arrancar en paralelo

- AC-1: Given una feature ya in_progress, When corro `start --feature <otra>`,
  Then arranca igual (deja de existir el rechazo "Ya hay feature in_progress") y
  ambas quedan in_progress en el backlog.
- AC-2: Given un repo git, When arranco una feature, Then se crea la rama
  `feature/<id>-<slug>` desde la rama base configurada — o `bugfix/<id>-<slug>`
  si la feature es `kind: bug` — sin cambiar la rama del checkout principal.
- AC-3: Given esa rama, When arranco la feature, Then se crea el worktree
  `../<repo>-wt/<id>-<slug>/` y el comando imprime la ruta para trabajar ahi.
- AC-4: Given que la rama o el worktree ya existen (reintento, o venian de
  antes), When arranco de nuevo, Then se reusan sin error y sin perder trabajo.
- AC-5: Given un directorio que no es repo git (o un repo sin commits), When
  arranco una feature, Then el arnes avisa que no puede aislar y sigue con el
  comportamiento de siempre, sin fallar.
- AC-6: Given `start --sin-worktree`, When arranco, Then no se crea ni rama ni
  carpeta: modo clasico explicito.

### Un solo estado, aunque haya varias carpetas

- AC-7: Given que estoy dentro de un worktree, When corro cualquier comando del
  arnes, Then lee y escribe el `feature_list.json` y el `progress/` del REPO
  PRINCIPAL (resuelto por `git rev-parse --git-common-dir`), nunca una copia
  local: el backlog no se bifurca.
- AC-8: Given una feature activa, When se escribe su estado vivo, Then va a
  `progress/current-<id>.md`, y nunca al de otra feature.
- AC-9: Given varias features activas, When miro `progress/current.md`, Then es
  el INDICE de lo que esta abierto (id, nombre, rama, worktree), no el estado de
  una sola.
- AC-10: Given el checkpoint automatico, When corre, Then usa un stamp por
  feature (`.last_autocheck-<id>`), asi que el autocheck de una no borra el de
  la otra.
- AC-11: Given dos features activas, When cierro UNA, Then el estado vivo de la
  otra queda intacto: su `current-<id>.md` no se archiva, no se resetea y su
  stamp no se borra (el bug de la feature #45, ahora imposible por
  construccion).

### Saber en que feature estoy

- AC-12: Given que estoy dentro del worktree de la feature #47, When corro
  `advance`, `approve-spec` o `close` sin `--feature`, Then el arnes infiere la
  #47 por la carpeta en la que estoy.
- AC-13: Given que estoy FUERA de todo worktree y hay varias activas, When corro
  un comando sin `--feature`, Then se niega y lista las activas (comportamiento
  de hoy, que ya existe).

### Cerrar con GitFlow

- AC-14: Given una feature con worktree, When corro `close --status done` SIN
  `--to`, Then se niega con exit 2 y el mensaje ordena preguntarle al USUARIO a
  que rama integrar, listando las ramas candidatas del repo.
- AC-15: Given `close --status done --to <rama>` y los gates en verde, When
  cierro, Then el arnes mergea la rama de la feature en `<rama>`.
- AC-16: Given ese merge, When miro el commit, Then NO lleva trailers de IA
  (`Co-Authored-By`, `Generated with`): la regla de `UPDATING.md` vale tambien
  para los commits que hace el arnes.
- AC-17: Given el merge hecho, When termina el cierre, Then la rama destino
  queda pusheada al remoto.
- AC-18: Given un conflicto de merge, When cierro, Then el arnes ABORTA el
  merge, deja la rama destino y el worktree exactamente como estaban, explica
  que archivos chocan y como resolverlo a mano, y sale con 1 sin marcar la
  feature como cerrada.
- AC-19: Given un cierre exitoso, When termina, Then se borra el worktree y se
  CONSERVA la rama de la feature.
- AC-20: Given un `--to` que no existe en el repo, When cierro, Then falla
  antes de tocar nada, con la lista de ramas validas.
- AC-21: Given `close --status blocked|pending|superseded`, When cierro, Then no
  se mergea ni se exige `--to` (solo el cierre `done` integra), y el worktree se
  conserva para poder retomar.

### Configuracion GitFlow (para los proyectos instalados)

- AC-22: Given un repo con `develop`, When arranco una feature, Then la rama
  sale de `develop`; si no existe `develop`, sale de `main` (o de la rama base
  configurada), y el arnes NUNCA crea `develop` por su cuenta.
- AC-23: Given la configuracion del repo, When cambio los prefijos o la rama
  base, Then el arnes los respeta (`feature/`, `bugfix/`, `release/` son los
  defaults de GitFlow).

### Verificacion

- AC-24: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh`, Then los cuatro terminan limpios, con tests nuevos para el
  arranque en paralelo, el estado por feature, el foco por worktree y el cierre
  con y sin conflicto.
- AC-25: Given dos features arrancadas de verdad en este repo, When trabajo en
  las dos y cierro una, Then la otra sigue intacta (estado, rama y worktree), y
  queda registrado en `docs/impl-47.md`.

## Los datos que se tocan

- disparador: `start` (crea rama + worktree + estado propio) y `close` (integra,
  publica y limpia).
- interruptor: `--sin-worktree` en `start` para el modo clasico, y la ausencia
  de git (sin repo no hay aislamiento posible: se avisa y se sigue).
- candado: el worktree en si — dos features nunca comparten archivos de trabajo.
  Para el estado compartido, el `git-common-dir` garantiza que todos escriben el
  mismo `feature_list.json`.
- `progress/current-<id>.md`: estado vivo por feature (reemplaza al unico).
- `progress/current.md`: pasa a ser el INDICE de features activas (id, nombre,
  rama, worktree). Ya no es el estado de nadie en particular.
- `progress/.last_autocheck-<id>`: stamp por feature.
- `feature_list.json`: cada feature activa gana `branch` y `worktree`
  (opcionales; su ausencia significa modo clasico y no rompe nada de lo ya
  cargado).
- Configuracion GitFlow del repo (rama base y prefijos), con defaults
  `develop`/`main` + `feature/`, `bugfix/`, `release/`.

## Pseudo-codigo (el acuerdo)

```
CUANDO se arranca una feature

  ¿es un repo git con al menos un commit?  -> si no, avisamos y seguimos como siempre
  ¿pidieron --sin-worktree?                -> si si, modo clasico

  ENTONCES creamos (o reusamos) su rama desde la rama base y su worktree
           hermano, y escribimos SU estado vivo,
           con la restriccion de que el checkout principal no cambia de rama
           y el backlog sigue siendo el del repo principal.

CUANDO se corre cualquier comando del arnes

  ¿estoy dentro de un worktree del proyecto? -> la feature se infiere de la carpeta
  ¿hay varias activas y no se cual?          -> me niego y las listo

  y SIEMPRE el estado se lee y se escribe en el repo principal.

CUANDO se cierra una feature como done

  ¿me dijeron a que rama va?  -> si no, me niego: que el AGENTE le pregunte al USUARIO
  ¿esa rama existe?           -> si no, me niego antes de tocar nada
  ¿los gates estan en verde?  -> los de siempre

  ENTONCES mergeamos, publicamos y borramos el worktree,
           con la restriccion de que un conflicto ABORTA y deja todo como estaba,
           y de que ningun commit del arnes lleva trailers de IA.

CUANDO se cierra como blocked, pending o superseded

  no se integra nada y el worktree se conserva: la feature puede retomarse.
```

Promesas: dos features nunca comparten archivos de trabajo · el backlog es uno
solo · cerrar una no toca a las otras · el arnes no elige la rama destino: la
pregunta.

## No funcionales

- SLOs: crear la rama y el worktree agrega menos de 2 s a `start` (dos comandos
  git locales); el resto de los comandos no cambia su costo.
- Seguridad (Articulo 4): el arnes solo opera sobre ramas del repo; nunca hace
  `push --force`, nunca borra ramas, y el merge se aborta ante el primer
  conflicto. Exit codes estables (0 ok / 1 fallo de integracion / 2 mal uso o
  falta de `--to`).
- Observabilidad: `status` lista las features activas con su rama y su worktree;
  `progress/current.md` es el indice legible de lo que hay abierto.

## Fuera de alcance

- Detectar solapamientos entre features (que dos toquen el mismo microservicio):
  decision del USUARIO, no se hace nada por ahora; los conflictos aparecen al
  integrar, que es donde git ya sabe resolverlos.
- Crear `develop` o las `release/*`: el arnes las usa si existen, nunca las crea.
- Pull requests: el cierre mergea y pushea directo (decision del USUARIO).
- Rebase, squash o cualquier reescritura de historia.
- Paralelismo entre MAQUINAS distintas: el estado sigue siendo local al repo.

## Observaciones (decisiones pendientes)

- OBS-1 [DECIDIDA por el USUARIO, 2026-08-21]: metodologia GitFlow — rama por
  feature, integracion a `develop` o `release/*` segun el ambiente, y SIEMPRE
  preguntarle al usuario a que rama pasar los cambios.
- OBS-2 [DECIDIDA por el USUARIO, 2026-08-21]: en ESTE repo (el arnes, que es el
  template) todo va a `main`; el esquema GitFlow completo es para cuando el
  arnes se instale en los proyectos. No se crea `develop` aca.
- OBS-3 [DECIDIDA por el USUARIO, 2026-08-21]: estado vivo por feature
  (`current-<id>.md`) con `current.md` como indice.
- OBS-4 [DECIDIDA por el USUARIO, 2026-08-21]: el foco es automatico por
  worktree; la carpeta dice en que feature estas.
- OBS-5 [DECIDIDA por el USUARIO, 2026-08-21]: el cierre mergea automaticamente
  y pushea la rama destino, y ningun commit que haga el arnes lleva trailers de
  IA.
- OBS-6 [DECIDIDA por el USUARIO, 2026-08-21]: al cerrar se borra el worktree y
  se conserva la rama.
- OBS-7 [DECIDIDA por el USUARIO, 2026-08-21]: los worktrees viven como hermanos
  del repo (`../<repo>-wt/<id>-<slug>/`).
- OBS-8 [DECIDIDA por el USUARIO, 2026-08-21]: no se detectan solapamientos
  entre features activas por ahora.
- OBS-9 [DECIDIDA por el USUARIO, 2026-08-21]: la feature #45 se cierra como
  `superseded --absorbida-por 47`, porque el estado por feature elimina ese bug
  por construccion.
- OBS-10 [REGISTRADA, sin accion]: `one_feature_at_a_time` deja de bloquear,
  pero la clave se conserva en `feature_list.json` para no romper backlogs
  existentes; pasa a leerse como "cuantas puede haber a la vez" y su valor
  historico `true` ya no impide arrancar una segunda.
