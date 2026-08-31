# Spec - Feature #65: el_arnes_cierra_lo_resuelto_aguas_arriba

Estado: approved
Aprobado: 2026-08-31T01:27:20Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-65-el-arnes-cierra-lo-resuelto-aguas-arriba.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan trabaja en `realestate`. Ahi, en su backlog, estan los bugs #91 y
#92: el cierre perdia el hito del PRD y la bitacora enlazaba documentos con un
nombre que nunca existio. Los dos son del ARNES, no del negocio, y se arreglaron
aguas arriba —en `harness_process`— como la feature #60, con una cola en la #63.
Alan vuelve a `realestate` a cerrarlos y no puede, porque ningun estado dice lo
que paso:

- `done` exige spec aprobado y evidencia propios, que ahi no existen.
- `superseded` exige `--absorbida-por <id>`, y `close.rs:74-82` valida ese id con
  `find_feature_index`: comprueba que exista **un id igual en ESTE backlog**, que
  no es identidad. Medido: `--absorbida-por harness_process#60` se niega (no hay
  sintaxis para lo ajeno) y `--absorbida-por 200` se niega bien, pero
  **`--absorbida-por 60` sale rc=0** y `status` imprime, con todo aplomo,
  `#91 [superseded por #60]`. La #60 de `realestate` es
  `cerrar-reparaciones-del-acta-de-restitucion`: una feature de negocio que no
  tiene nada que ver. **El unico camino que hoy funciona afirma algo falso con
  exit 0.**
- `blocked` dice "trabada" de algo que esta resuelto, y ademas la deja en el
  DENOMINADOR de `prd tree` para siempre: el proyecto nunca llega a 100%.
- `pending` la deja ofreciendose en `next` como trabajo por hacer.

DESPUES: Alan cierra el #91 diciendo donde se arreglo:

```
close --feature 91 --status resuelto-aguas-arriba --resuelto-en harness_process/feature-60
```

y el backlog, el `status` y el PRD dicen esa misma cosa, sin inventar ninguna
otra. El arnes **no comprueba** que esa referencia exista —no puede, vive en otro
repo— y lo dice con todas las letras en vez de fingir que la valido.

## Hoy -> Como va a funcionar

```
HOY                                        DESPUES

close --status superseded                  close --status resuelto-aguas-arriba
  --absorbida-por 60                         --resuelto-en harness_process/feature-60
  |__ find_feature_index (backlog LOCAL)     |__ solo comprueba la FORMA
  |__ rc=0                                   |__ rc=0, y el mensaje dice
  |__ status: "[superseded por #60]"         |      "sin verificar: vive en otro repo"
        ^^^ la #60 de ESTE repo,             |__ status: "[resuelto aguas arriba en
            que es otra feature                     harness_process/feature-60,
                                                    sin verificar]"
prd tree                                   prd tree
  blocked   -> 0/3 done  (denominador       resuelto-aguas-arriba -> fuera de los dos
               para siempre)                  lados, como superseded
  superseded-> 0/2 done

status (cabecera)                          status (cabecera)
  "4 feature(s) | active=0 pending=0        cada estado con su bucket: los numeros
   blocked=0 done=2"   <- no suma            suman

Jira (emit.rs:277, brazo `_`)              Jira
  cualquier estado desconocido ->            rama explicita: no transiciona,
  statuses.pending (reabre el ticket)        comenta donde se arreglo
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero cerrar un bug del arnes reportado en un repo de trabajo
  diciendo DONDE se arreglo, para no tener que elegir entre una mentira
  (`superseded por #60`) y dejarlo abierto.
- P1: Como Alan, quiero que el arnes no afirme haber verificado una referencia
  que no puede verificar, para poder confiar en las que si verifica.
- P2: Como Alan, quiero que esas features salgan del denominador del PRD, para
  que el porcentaje signifique algo.
- P2: Como quien mira el tablero de Jira, quiero que un cierre asi no reabra el
  ticket ni lo deje mudo.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una feature de un backlog cualquiera, When se corre
  `close --feature <id> --status resuelto-aguas-arriba --resuelto-en <proyecto>/feature-<n>`,
  Then cierra con rc=0, guarda la referencia y el mensaje dice explicitamente que
  NO se comprobo.
  Comando: `cd rust && out=$(cargo test resuelto_aguas_arriba 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-2: Given ese estado, When falta `--resuelto-en`, Then el cierre se niega con
  Exit 2 nombrando el flag, igual que `superseded` sin `--absorbida-por`.
  Comando: `cd rust && out=$(cargo test aguas_arriba_exige_referencia 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-3: Given una referencia con forma invalida (vacia, sin `/`, sin id, con
  espacios), When se cierra, Then se niega nombrando la forma esperada. Se
  comprueba la FORMA, nunca la existencia.
  Comando: `cd rust && out=$(cargo test forma_de_la_referencia_externa 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-4: Given una referencia bien formada que apunta a algo inexistente
  (`no-existe/feature-999`), When se cierra, Then **cierra igual**: el arnes no
  puede comprobarlo y no finge que si.
  Comando: `cd rust && out=$(cargo test referencia_externa_no_se_valida 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-5: Given una feature cerrada asi, When se corre `status`, Then la muestra
  como resuelta aguas arriba, con la referencia y con la marca de que no se
  verifico — nunca como `blocked`.
  Comando: `cd rust && out=$(cargo test status_muestra_aguas_arriba 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-6: Given features en todos los estados, When se corre `status`, Then la
  cabecera SUMA: hoy imprime `4 feature(s) | active=0 pending=0 blocked=0 done=2`
  y se pierden dos.
  Comando: `cd rust && out=$(cargo test cabecera_de_status_suma 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-7: Given un hito del PRD con una feature resuelta aguas arriba, When se corre
  `prd tree`, Then esa feature queda FUERA del numerador y del denominador, como
  `superseded`: el trabajo no se hizo en este producto.
  Comando: `cd rust && out=$(cargo test prd_tree_ignora_aguas_arriba 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-8: Given el binding de Atlassian activo, When se cierra asi, Then NO se
  transiciona el ticket (no cae en el brazo `_` de `emit.rs:277`, que lo mandaria
  a `statuses.pending` y lo reabriria) y se comenta donde se arreglo.
  Comando: `cd rust && out=$(cargo test aguas_arriba_no_reabre_el_ticket 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-9: Given los cinco sitios que comparan el literal del estado (`cli.rs:41`,
  `prd.rs:895`, `status.rs:47`, `emit.rs:276` y `:293`), When se agrega el estado
  nuevo, Then ninguno lo trata por su brazo por defecto. La #37 ya pago este bug
  una vez: un `superseded` que reabria el ticket por caer en el `_`.
  Comando: `cd rust && out=$(cargo test todos_los_estados_tienen_su_rama 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-10: Given el cierre nuevo, When se lo audita, Then `close` NO hace I/O de
  red ni resuelve rutas de otros repos: la comprobacion negativa es que el modulo
  no importa nada de red y que el cierre funciona con el otro repo ausente.
  Comando: `cd rust && out=$(cargo test cierre_sin_io_de_red 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-11 (MANUAL): Given el caso real (#91 y #92 de `realestate`, arreglados en
  `harness_process/feature-60`), When Alan los cierra con el comando nuevo, Then
  el backlog, `status` y `prd tree` dicen lo mismo y nada afirma algo que el
  arnes no comprobo. Lo verifica el reviewer.

## Los datos que se tocan

- disparador: `close --feature <id> --status resuelto-aguas-arriba`.
- el dato nuevo: un campo con la referencia externa, con la sintaxis que el repo
  YA usa para lo cross-proyecto (`<proyecto>/feature-<id>`, `graph/ids.rs:8-14`,
  con test en `ids.rs:59`). No se inventa vocabulario: si el hub aguas arriba y
  el cierre aguas abajo usan el mismo id, el grafo puede unirlos sin traducir.
- lo que NO se valida: la existencia. Se comprueba la forma y se dice que lo
  demas no se comprobo.
- interruptor: ninguno nuevo.
- candado: el estado es terminal como `superseded`; no reabre nada.

## Pseudo-codigo (el acuerdo)

```
CUANDO close --status resuelto-aguas-arriba

  ¿vino --resuelto-en?              -> si no, se niega nombrando el flag
  ¿tiene forma <proyecto>/feature-<id>? -> si no, se niega mostrando la forma

  NO se comprueba que exista: vive en otro repo y el arnes no lo puede abrir.

  ENTONCES se cierra guardando la referencia,
           con la restriccion de que TODO lo que la muestre
           diga tambien que no se verifico.
```

Promesas: no se transiciona el ticket de Jira · la feature sale del numerador y
del denominador del PRD · `close` no hace I/O de red ni toca otros repos · el
mensaje nunca afirma que la referencia existe.

## No funcionales

- SLOs: el cierre no agrega trabajo; no hay lecturas nuevas.
- Seguridad: no se resuelve ninguna ruta ajena ni se abre ningun archivo de otro
  repo. La referencia es un dato, no un puntero que el arnes siga.
- Observabilidad: la referencia queda en el backlog, en `history.md` y en el
  estado archivado, como el resto de los cierres.

## Fuera de alcance

- Validar la referencia contra el otro repo, ahora o nunca por accidente. Si
  algun dia se hace, sera con timeout y produciendo "no se pudo comprobar" como
  TERCERA respuesta, jamas como error: un cierre que depende de que otro repo
  este clonado en esa maquina es el mismo defecto que la leccion de la
  herramienta externa ausente.
- Migrar cierres viejos. Los `blocked` que hoy significan "resuelto aguas arriba"
  los reetiqueta el usuario si quiere.
- Unificar el modelo de estados en un enum de Rust. `docs/review-37.md:70-72`
  argumenta que con un enum el defecto #1 de aquella feature habria fallado en
  `cargo build`, y tiene razon; pero es una refactorizacion con su propio riesgo
  y merece su spec. Aca se cubre lo mismo con el AC-9, que exige que ningun sitio
  trate el estado nuevo por su brazo por defecto.

## Observaciones (decisiones pendientes)

- **Decisiones del usuario ya tomadas (2026-08-31)**: estado nuevo (no solo un
  campo), sintaxis `<proyecto>/feature-<id>`, sin validar y diciendolo.
- **El acceptance original de esta feature era falso y se reescribio**: afirmaba
  que `--note` "no es trazable", y esta medido que la nota queda en cuatro
  lugares. Se conserva el registro de la correccion en el backlog porque es el
  mismo defecto que costo tres vueltas en la #66: un AC construido sobre una
  premisa que nadie verifico.
- **El nombre exacto del estado y del flag** (`resuelto-aguas-arriba` /
  `--resuelto-en`) es lo unico que queda a confirmar al aprobar: entra en el
  vocabulario del arnes y despues cuesta cambiarlo.
- `Peldano elegido:` se agrega un ESTADO y un flag, que es peldaño bajo. La razon
  por la que el peldaño de arriba no alcanza esta medida: con `--note` el rastro
  existe pero `status` sigue diciendo `[blocked]` y `prd tree` sigue contando la
  feature en el denominador, que son las dos cosas que motivan el cambio.
