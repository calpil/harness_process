# Spec - Feature #14: hub_batch_upserts_atomic_install

Estado: approved
Aprobado: 2026-08-14T03:43:37Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature 14 en el chat el 2026-08-14 (AskUserQuestion: 'Si, apruebo'), con el spec mostrado completo en el chat y abierto en su editor. Decisiones registradas: OBS-1 lote con UNNEST (no COPY), OBS-2 mv atomico en los dos instaladores (sh y ps1), OBS-3 sumar timeout de sentencia y candado por proyecto a esta misma feature, y ante el fork del candado por proyecto eligio la opcion segura: escribir solo el delta
Plan: docs/plan-feature-14-hub-batch-upserts-atomic-install.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan cierra un commit en Real-State y el hook post-commit dispara
`harness graph sync_git`. El comando toma el candado global del hub
(`~/.harness-hub/.lock`), lee el grafo entero y lo vuelve a escribir fila por
fila: hoy son 1047 nodos y 1641 aristas, o sea 2688 ida-y-vuelta consecutivos
contra un PostgreSQL de Aiven que esta a 164 ms por consulta. Son mas de siete
minutos de reloj para registrar un commit que toco dos archivos. Y el candado es
uno solo para toda la maquina: mientras ese sync respira, cualquier `start`,
`advance`, `approve-spec` o `autocheck` de CUALQUIER otro proyecto queda en la
cola. Al momento de escribir este spec habia nueve procesos `harness` vivos: uno
de Real-State llevaba 1 h 09 min con la conexion abierta y 0,94 s de CPU (puro
esperar la red), y detras esperaban dos `approve-spec`, dos `advance`, dos
`sync_git` y un `autocheck` de Online-Invoice, ADR y este mismo repo. Si ademas
el hub deja de responder, no hay nada que corte: `connect_timeout` solo cubre el
saludo inicial, asi que el proceso espera para siempre y se lleva el candado
puesto. Alan cree que el arnes se colgo, mata procesos y sigue trabajando sin
memoria en el hub.

Y cuando se cansa de eso y re-instala el arnes para actualizarlo, se lleva el
segundo golpe: el instalador copia el binario recien compilado ENCIMA del
`harness` que ya existe. En macOS eso reescribe el mismo inode y le invalida la
firma al Mach-O que el kernel ya tenia cacheado, asi que la siguiente corrida
muere con `zsh: killed  harness` (SIGKILL). El sintoma vuelve en cada
actualizacion, Alan borra el binario a mano y re-instala, y el arnes queda con
fama de fragil.

DESPUES: el mismo `sync_git` escribe SOLO lo que ese commit toco -- cinco nodos y
tres aristas, dos sentencias -- en lugar de reescribir el grafo entero. El
candado ya no es global: cada proyecto tiene el suyo, asi que Real-State,
Online-Invoice y ADR dejan de hacer fila entre ellos. Si el hub se cuelga, la
sentencia corta sola y el comando falla con un error que se puede leer, en vez de
quedarse tomado del candado hasta que alguien lo mate. Y cuando Alan re-instala,
el instalador compila a un archivo temporal al lado del destino y lo mueve con un
rename atomico: el binario viejo nunca se reescribe en su lugar, el nuevo estrena
inode con su firma intacta y `harness` arranca a la primera. Sin SIGKILL, sin
borrar nada a mano.

## Hoy -> Como va a funcionar

```
HOY                                          DESPUES

git commit                                   git commit
  |__ hook -> harness graph sync_git           |__ hook -> harness graph sync_git
        |__ flock(~/.harness-hub/.lock)              |__ flock(~/.harness-hub/.lock-<proyecto>)
              GLOBAL: toda la maquina                     por proyecto: nadie mas espera
        |__ store.load()      2 queries              |__ store.load()      2 queries
        |__ store.save()                             |__ store.save() (solo lo sucio)
              1047x INSERT nodo   (1 x fila)               1x INSERT ... UNNEST (5 nodos)
              1641x INSERT arista (1 x fila)               1x INSERT ... UNNEST (3 aristas)
              = 2688 round-trips x 164 ms                  = 2 round-trips x 164 ms
              ~ 7,3 min con el candado tomado              ~ 0,3 s con el candado tomado
        |__ el resto de la maquina, en cola          |__ el resto de la maquina, libre

hub que deja de responder                    hub que deja de responder
  -> espera infinita, candado puesto           -> statement_timeout corta y falla claro

./setup_harness.sh                           ./setup_harness.sh
  |__ cargo build --release                    |__ cargo build --release
  |__ cp target/release/harness  harness       |__ cp target/release/harness  .harness.new.$$
        (reescribe el inode vivo)              |__ chmod +x                   .harness.new.$$
  |__ chmod +x harness                         |__ mv -f .harness.new.$$      harness
                                                     (rename atomico: inode nuevo)
  -> siguiente corrida: Killed: 9              -> siguiente corrida: harness responde
```

## Recorridos de usuario (priorizados)

- P1: Como quien commitea en cualquier repo con el arnes instalado, quiero que
  el registro del commit en el hub tarde menos de un segundo y no minutos, para
  que el hook post-commit no me frene el trabajo.
- P1: Como quien trabaja en varios proyectos a la vez en la misma maquina,
  quiero que un sync de un proyecto no deje a los demas esperando un candado
  global, para no tener que matar procesos para poder seguir.
- P1: Como quien actualiza el arnes con `./setup_harness.sh`, quiero que la
  corrida siguiente del binario funcione, para no toparme con `Killed: 9` cada
  vez que actualizo.
- P1: Como quien depende del hub, quiero que un hub que no responde falle con un
  error legible en vez de colgar el comando para siempre, para saber que pasa y
  poder seguir trabajando.
- P2: Como usuario de Windows, quiero la misma escritura no destructiva en
  `setup_harness.ps1`, para que actualizar no dependa de que ningun
  `harness.exe` este corriendo.
- P2: Como mantenedor, quiero que el CONTENIDO del hub no cambie (mismos nodos,
  mismas aristas, mismo merge de props) para que esto sea rendimiento y
  robustez, no una migracion.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un grafo en memoria con N nodos por escribir, When corre
  `PgGraphStore::save()`, Then los nodos se escriben con sentencias
  `INSERT INTO graph_nodes ... SELECT * FROM UNNEST($1::text[], $2::text[],
  $3::jsonb[]) ON CONFLICT (id) DO UPDATE SET label = EXCLUDED.label,
  props = graph_nodes.props || EXCLUDED.props` en lotes de a lo mas 1000 filas
  (`ceil(N/1000)` sentencias), y no una sentencia por fila.

- AC-2: Given un grafo en memoria con M aristas por escribir, When corre
  `save()`, Then las aristas se escriben con `INSERT INTO graph_edges ...
  SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::jsonb[])
  ON CONFLICT (source, target, type) DO UPDATE SET props =
  COALESCE(graph_edges.props, '{}'::jsonb) || EXCLUDED.props` en lotes de a lo
  mas 1000 filas.

- AC-3: Given dos aristas en memoria con la misma clave `(source, target, type)`
  y props distintas (hoy legales: `add_edge` deduplica por igualdad de dict
  completo, no por clave), When corre `save()`, Then van al lote como UNA sola
  fila cuyos props son la fusion de ambas con la ultima ganando clave a clave
  -- identico a lo que produce hoy el par de INSERT secuenciales -- y Postgres
  NO falla con "ON CONFLICT DO UPDATE command cannot affect row a second time".

- AC-4: Given un comando que toca k nodos y j aristas sobre un hub de N nodos y
  M aristas (hoy N=1047, M=1641), When corre `save()`, Then se escriben SOLO
  esas k+j filas (`ceil(k/1000) + ceil(j/1000)` sentencias, cero si el comando
  no toco nada), no las N+M; `load()` reinicia el registro de lo sucio, y una
  mutacion directa sobre `store.nodes` (caso `desmarcar`) marca su nodo
  explicitamente para que lo persistido sea identico a lo de hoy.

- AC-5: Given el estado real del hub del usuario (1047 nodos, 1641 aristas,
  164 ms de round-trip medidos el 2026-08-14), When corre un `sync_git` de un
  commit con dos archivos, Then `save()` emite 2 sentencias en vez de 2688 y el
  candado queda tomado del orden de decimas de segundo, no de minutos.

- AC-6: Given cualquier tamano de grafo, When corre `save()`, Then todo el
  guardado sigue ocurriendo dentro de UNA sola transaccion (igual que hoy): o
  entran todos los lotes o no entra ninguno, y el esquema de las tablas no
  cambia (sin migracion, sin tabla temporal, sin dependencia nueva en
  `Cargo.toml`).

- AC-7: Given la fusion de aristas del AC-3 y el registro de lo sucio del AC-4,
  When corre `cargo test`, Then tests unitarios los cubren SIN base de datos
  (funciones puras: entrada = grafo en memoria + lo tocado, salida = las filas
  que irian al lote).

- AC-8: Given dos proyectos distintos de la misma maquina que comparten
  `~/.harness-hub`, When cada uno corre un comando del hub al mismo tiempo,
  Then cada proceso toma su propio candado `<hub>/.lock-<proyecto>` (nombre del
  proyecto saneado para el filesystem) y ninguno espera al otro; el `.lock`
  global deja de usarse. Escribir solo el delta (AC-4) es lo que hace segura
  esta separacion: ningun proyecto reescribe filas de otro.

- AC-9: Given `DB_STATEMENT_TIMEOUT` en el entorno o en `<hub>/.env` (en
  milisegundos; por defecto 30000; `0` lo desactiva), When el store abre una
  conexion, Then la pasa como `-c statement_timeout=<ms>` en las opciones de
  conexion y habilita keepalives TCP, de modo que una sentencia contra un hub
  que no responde falla con error accionable en vez de bloquear para siempre.
  El spec deja explicito que esto cubre al servidor que no responde, no a una
  conexion cortada en silencio mas alla de lo que detecten los keepalives.

- AC-10: Given una instalacion donde `harness` ya existe y esta corriendo o fue
  ejecutado antes, When corre `./setup_harness.sh` con cargo disponible, Then el
  instalador escribe el binario compilado en un temporal del MISMO directorio,
  le pone permiso de ejecucion y lo mueve al destino con `mv -f` (rename
  atomico): el destino queda con un inode NUEVO, el binario viejo nunca se
  reescribe en su lugar y la corrida siguiente de `sh harness_cli status`
  responde sin SIGKILL.

- AC-11: Given que la copia al temporal o el `mv` fallan, When corre
  `./setup_harness.sh`, Then no queda ningun temporal `.harness.new.*` en el
  directorio, el binario previo sigue intacto y usable, y el instalador reporta
  el error accionable existente y sale con codigo 1 (comportamiento actual
  preservado).

- AC-12: Given Windows, When corre `.\setup_harness.ps1` con cargo disponible,
  Then `harness.exe` se escribe primero a un temporal del mismo directorio y se
  mueve con `Move-Item -Force`; si el destino esta bloqueado por un
  `harness.exe` en ejecucion, el instalador aparta el destino a un
  `.harness.exe.old.<pid>` y completa la instalacion en vez de dejar el binario
  a medio escribir; los temporales/apartados se limpian y el modo `-DryRun` no
  escribe nada.

- AC-13: Given el smoke del instalador, When corre `bash tests/setup_smoke.sh`,
  Then existe un caso que instala DOS veces sobre el mismo directorio y verifica
  que el inode del binario cambio entre corridas y que no quedaron temporales.

- AC-14: Given el repo completo, When corren `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `bash harness_check.sh`, Then los cuatro pasan limpios (Articulo 1), con
  `templates/` espejado donde corresponda (Articulo 6).

## Los datos que se tocan

- disparador: cualquier comando que llame a `GraphMemoryManager` y termine en
  `store.save()` (`sync_git`, `start`, `close`, `advance`, `approve-spec`,
  `autocheck`, `descubrir`, `vincular`, `desmarcar`, `registrar`).
- entidades: tablas `graph_nodes (id PK, label, props jsonb)` y
  `graph_edges (source, target, type PK, props jsonb)`. NO cambian: mismo
  esquema, mismas claves, mismo merge `||` de props.
- lo sucio: conjunto en memoria de ids de nodo y de claves
  `(source, target, type)` tocadas desde el ultimo `load()`; es lo unico que
  `save()` escribe. Vive en el proceso, no en disco.
- interruptor: `DB_STATEMENT_TIMEOUT` (ms; `0` desactiva) para el corte de
  sentencia. El tamano de lote es una constante del codigo
  (`UPSERT_CHUNK = 1000`), no configuracion de usuario.
- candado: `flock` exclusivo sobre `<hub>/.lock-<proyecto>` (antes
  `<hub>/.lock`, uno para toda la maquina).
- archivos: `<harness>/harness` (o `harness.exe`) y su temporal hermano
  `<harness>/.harness.new.<pid>` (`.harness.exe.new.<pid>` en Windows), que solo
  existe durante la instalacion.

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien agrega o cambia un nodo o una arista en memoria
  ademas de guardarlo, anotamos su clave en "lo sucio"
CUANDO recargamos el grafo desde el hub
  "lo sucio" vuelve a cero (lo de la base ya esta en la base)

CUANDO hay que guardar el grafo en el hub

  ¿hay algo sucio? -> si no, no abrimos nada y no emitimos ninguna sentencia

  abrimos UNA transaccion (como hoy)

  para los NODOS SUCIOS:
    los partimos en tandas de a lo mas 1000
    por cada tanda: UNA sentencia que recibe tres arreglos paralelos
                    (ids, labels, props) y los desarma en filas,
                    con el MISMO upsert de hoy

  para las ARISTAS SUCIAS:
    primero las juntamos por (source, target, type):
      la repetida no es un error, es una fusion; la ultima gana clave a clave
      (asi ya se comportan hoy los INSERT uno atras del otro)
    las partimos en tandas de a lo mas 1000
    por cada tanda: UNA sentencia con cuatro arreglos paralelos

  cerramos la transaccion

CUANDO abrimos una conexion al hub
  le pedimos al servidor que corte solo cualquier sentencia que pase del
      limite configurado, y prendemos keepalives para no quedar hablandole
      a una conexion muerta

CUANDO tomamos el candado del hub
  el candado es del PROYECTO, no de la maquina

CUANDO el instalador tiene un binario recien compilado

  ¿existe el binario compilado? -> si no, error accionable y salida 1
  lo copiamos a un temporal HERMANO del destino (mismo directorio = mismo
      filesystem, unica forma de que el rename sea atomico)
  le damos permiso de ejecucion AL TEMPORAL
  lo movemos encima del destino con un rename

  si algo de eso falla -> borramos el temporal, dejamos el binario viejo
                          intacto, error accionable y salida 1
```

Promesas: el hub termina con el mismo contenido que hoy · una transaccion, todo
o nada · ningun proyecto reescribe filas de otro · el binario del destino nunca
se escribe en su lugar · si la instalacion falla, el binario anterior sigue
sirviendo.

## No funcionales

- SLOs: un `sync_git` tipico (5 nodos, 3 aristas) baja de ~7,3 min a ~0,3 s de
  round-trips; el candado del proyecto queda tomado decimas de segundo. Un
  grafo 10x mas grande no cambia el costo del sync, porque ya no se reescribe.
- Seguridad: sin credenciales nuevas ni cambios en TLS/sslmode; los props
  siguen viajando como parametros ligados (`$1..$4`), nunca interpolados en el
  SQL; `DB_STATEMENT_TIMEOUT` se valida como numero antes de armar las opciones
  de conexion. El temporal del instalador nace en el mismo directorio del
  destino, con el nombre marcado por PID, y se borra ante cualquier error.
- Observabilidad: los textos que imprimen los comandos del hub no cambian
  (paridad verbatim); el instalador conserva sus mensajes de exito/error.

## Fuera de alcance

- COPY BINARY a tabla temporal: descartado por decision del usuario a favor de
  UNNEST (menos codigo, misma semantica, sin tabla intermedia).
- Que `load()` traiga solo el subgrafo del proyecto en vez de la tabla entera
  (hoy `map` e `impacto` dependen de ver todo).
- Que `desmarcar` borre de verdad la clave `tipo` en la base: hoy el merge `||`
  nunca borra claves, asi que el comando ya miente; esta feature preserva el
  comportamiento actual sin empeorarlo (ver OBS-4).
- Timeout para la ESPERA del candado (con candado por proyecto y save de decimas
  de segundo, la contencion deja de ser un problema real).
- Hacer atomica la escritura del resto de los assets (`install_asset` copia
  scripts sh/ps1, que no sufren el problema de firma del Mach-O).

## Observaciones (decisiones pendientes)

- OBS-1 [decidida por el usuario, 2026-08-14]: tecnica de lote = UNNEST por
  tandas, no COPY a tabla temporal.
- OBS-2 [decidida por el usuario, 2026-08-14]: el `mv` atomico va en los DOS
  instaladores (sh y ps1), manteniendo la paridad que exige UPDATING.md.
- OBS-3 [decidida por el usuario, 2026-08-14]: los dos sintomas que salieron al
  medir entran en ESTA feature -- timeout de sentencia (AC-9) y candado por
  proyecto (AC-8) -- y, ante el fork que abre el candado por proyecto (dos
  proyectos podrian pisarse props porque hoy `save()` reescribe la tabla
  entera), el usuario eligio la opcion segura: escribir solo el delta (AC-4).
- OBS-4 [registrada, sin accion en esta feature]: `desmarcar` no puede borrar la
  clave `tipo` del hub porque el upsert fusiona props con `||` y nunca elimina.
  Es un bug PREEXISTENTE; esta feature no lo empeora (el nodo se sigue
  escribiendo igual) y queda anotado para una feature propia.
