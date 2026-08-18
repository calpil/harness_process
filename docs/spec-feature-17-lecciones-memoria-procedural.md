# Spec - Feature #17: lecciones_memoria_procedural

Estado: approved
Aprobado: 2026-08-16T20:00:57Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #17 en el chat (AskUserQuestion: 'Si, lo apruebo'), con el spec mostrado en el chat y abierto en su editor. 20 AC. Decisiones registradas: OBS-1 sin --force (hardline en los nombres de clase), OBS-2 close con clase inexistente falla, OBS-3 la guia COMO-ESCRIBIR-UNA-LECCION.md es plantilla del arnes y las lecciones sobreviven al --reset, OBS-4 harness_check BLOQUEA por frontmatter ilegible, OBS-5 el campo leccion es opcional y no migra las 16 features cerradas. Decisiones previas del PRD que este spec hereda: archivos en docs/ (tres almacenes, funciona sin hub), require_leccion apagado por default, perfil versionado.
Plan: docs/plan-feature-17-lecciones-memoria-procedural.md
PRD: docs/prd/aprendizaje/PRD-aprendizaje.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan arranca una feature nueva con Codex. En la #7 se decidio que
`roles/` es la fuente unica y que los espejos por backend se regeneran desde el
instalador; ese conocimiento existe, esta escrito y esta fechado — en
`docs/impl-7.md`, archivado bajo el numero 7. Codex no tiene forma de encontrarlo:
no sabe que existio una feature #7 y, si la buscara, tendria que abrir dieciseis
archivos para descubrir cual habla de espejos. Asi que propone editar
directamente un `.claude/agents/*.md`. Alan lo corrige, la correccion se escribe
en `progress/history.md`, y muere ahi. **El arnes ya sabia; el orden en que lo
guardo hizo que no llegara.**

DESPUES: ese conocimiento vive en `docs/lecciones/espejo-de-roles.md`, ordenado
por **clase de trabajo** y no por numero de feature, con sus `triggers`
(`roles`, `.claude/agents`, `espejo`, `harness_check`) para que se encuentre por
tema. Cuando la feature #23 vuelva a tocar espejos, quien la implemente encuentra
la leccion, la usa (`leccion usar` deja el rastro) y, si aprende algo nuevo,
**patchea esa misma leccion** en vez de crear una nueva. Al cerrar, el arnes le
pide que declare que aprendio: una clase, o `ninguna` con motivo.

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES

close --status done                     close --status done --leccion <clase>
  |__ archiva estado                      |__ valida que la clase exista
  |__ hub: registra evento                |__ archiva estado
  |__ refresca graphify                   |__ hub: registra evento
  (el conocimiento queda ordenado         |__ refresca graphify
   por id de feature: impl-7.md,          |__ registra "leccion" en la feature
   impl-14.md, impl-16.md...)             `__ bitacora en history.md

(nada)                                  docs/lecciones/<clase>.md
                                          |__ frontmatter: triggers, usos, estado
                                          `__ referencias/<tema>.md (detalle)

buscar "como se hace X"                 sh harness_cli leccion list
  -> abrir impl-*.md uno por uno          -> catalogo por clase, ordenado por uso
```

## Recorridos de usuario (priorizados)

- P1: Como implementer, quiero encontrar por TEMA lo que ya se resolvio antes,
  para no reconstruirlo ni repetir un error que el arnes ya pago.
- P1: Como lider, quiero que cerrar una feature me obligue a decidir que se
  aprendio, para que el conocimiento no dependa de que alguien se acuerde.
- P1: Como Alan, quiero que crear una leccion nueva sea el ULTIMO recurso y no el
  primero, para no terminar con cuarenta lecciones de una sola feature cada una.
- P2: Como reviewer, quiero ver cuales lecciones se usan y cuales no, para saber
  cual esta muerta antes de que el curador (#21) la archive.
- P2: Como usuario de cualquier backend (Claude, Gemini, Codex, Kimi, Grok),
  quiero que las reglas de que capturar y que NO capturar esten en mi superficie,
  para que el arnes aprenda igual sin importar con que agente trabaje.

## Criterios de aceptacion (Given/When/Then)

### Formato y siembra

- AC-1: Given un proyecto con el arnes y sin `docs/lecciones/`, When se corre
  `setup_harness.sh` o `setup_harness.ps1`, Then se siembra `docs/lecciones/` con
  `COMO-ESCRIBIR-UNA-LECCION.md` y **ninguna leccion**; y un reinstall posterior
  **no pisa** ninguna leccion existente.
- AC-2: Given una leccion `docs/lecciones/<clase>.md`, Then su frontmatter tiene
  `nombre`, `descripcion` (una sola oracion, <= 80 caracteres, terminada en
  punto), `triggers` (lista), `relacionadas` (lista), `origen` (lista de features),
  `usos` (entero), `ultimo_uso` (fecha o vacio), `ultima_actualizacion` (fecha) y
  `estado` (`activa` | `stale` | `archivada`); y su cuerpo tiene las secciones
  `## Cuando aplica`, `## Procedimiento`, `## Pitfalls` y `## Verificacion`.

### Comandos

- AC-3: Given un repo con el arnes, When se corre
  `sh harness_cli leccion nueva <clase>`, Then se crea
  `docs/lecciones/<clase>.md` desde la plantilla con `estado: activa`, `usos: 0`,
  `ultima_actualizacion` de hoy y `origen` con la feature activa si la hay; exit 0
  e imprime la ruta creada.
- AC-4: Given un nombre que **no** es de clase, When se corre `leccion nueva`,
  Then exit 2, **no** se crea ningun archivo, y el mensaje dice cual regla se
  violo y da dos ejemplos de nombres validos. Se rechaza el nombre que contenga
  `feature` o `#`, empiece con `fix-`, `debug-`, `audit-` o `hotfix-`, contenga
  una fecha `YYYY-MM-DD`, o contenga un numero de tres o mas digitos.
- AC-5: Given que `docs/lecciones/<clase>.md` ya existe, When se corre
  `leccion nueva <clase>`, Then exit 2 sin tocar el archivo, y el mensaje empuja a
  **patchearla** (`leccion show <clase>`) en vez de crear otra.
- AC-6: Given una o mas lecciones, When se corre `leccion list`, Then se listan
  nombre, descripcion, usos, ultimo uso y estado, ordenadas por uso descendente;
  `--json` emite lo mismo en JSON. Sin lecciones, exit 0 con un mensaje que
  explica como crear la primera.
- AC-7: Given una clase existente, When se corre `leccion show <clase>`, Then se
  imprime la leccion completa; si no existe, exit 2 listando las clases de nombre
  mas parecido.
- AC-8: Given una clase existente con `usos: N`, When se corre
  `leccion usar <clase>`, Then queda `usos: N+1` y `ultimo_uso` con la fecha de
  hoy, **sin** modificar el cuerpo ni `ultima_actualizacion`.
- AC-9: Given el hub PostgreSQL caido o no configurado, When se corre cualquier
  subcomando `leccion`, Then el comportamiento y los exit codes son identicos a
  los del hub disponible (las lecciones son archivos y no dependen del hub).

### Gate del cierre

- AC-10: Given `feature_list.json` **sin** la regla `require_leccion` (ausente o
  en `false`), When se corre `close --status done` sin `--leccion`, Then el cierre
  se comporta exactamente como hoy (compatibilidad total con las 16 features ya
  cerradas).
- AC-11: Given `require_leccion: true` y una feature activa, When se corre
  `close --status done` sin `--leccion`, Then exit 2 con mensaje accionable que
  nombra las dos salidas validas (`--leccion <clase>` o
  `--leccion ninguna --leccion-motivo "<texto>"`) y **no** cierra la feature.
- AC-12: Given `require_leccion: true`, When se corre
  `close --status done --leccion <clase>` con una clase que existe, Then la
  feature cierra, queda `"leccion": "<clase>"` en su entrada de
  `feature_list.json` y la linea de `history.md` incluye la clase. Si la clase
  **no** existe, exit 2 sugiriendo `leccion nueva <clase>` y la feature **no**
  cierra.
- AC-13: Given `require_leccion: true`, When se corre
  `close --status done --leccion ninguna --leccion-motivo "<texto>"`, Then la
  feature cierra y el motivo queda registrado en la entrada y en `history.md`;
  con `--leccion ninguna` **sin** motivo, exit 2 (declarar que no se aprendio nada
  es valido, pero hay que decir por que).

### Reglas portadas (el corazon del cambio)

- AC-14: Given `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`, Then contiene
  (a) el **orden de preferencia** — patchear la leccion que estuvo en juego >
  patchear el paraguas existente > agregar `referencias/<tema>.md` > recien
  entonces crear una clase nueva — y (b) la lista completa de **que NO capturar**:
  fallas dependientes del entorno, afirmaciones negativas sobre herramientas,
  errores transitorios ya resueltos, narrativas de tarea unica, y fracasos no
  resueltos presentados como practica recomendada.
- AC-15: Given los tres roles (`roles/leader.md`, `roles/implementer.md`,
  `roles/reviewer.md` y sus espejos en `templates/roles/`), Then cada uno cita las
  reglas de AC-14 en la parte que le toca, y tras el cambio el **gate de espejo de
  roles** de `harness_check.sh` sigue limpio (re-correr el instalador regenera los
  espejos de los cuatro backends).

### Arquitectura, docs e integridad

- AC-16: Given `docs/architecture.md` y su espejo en `templates/docs/`, Then
  documentan el limite de los **tres almacenes**: el hub PostgreSQL guarda
  *eventos*, `docs/lecciones/` guarda *procedimiento* y (a futuro, #19)
  `docs/perfil-usuario.md` guarda *preferencias*; y dejan explicito que las
  lecciones son archivos versionados y **no** agregan tablas ni filas al hub.
- AC-17: Given `README.md`, `UPDATING.md` y las superficies generadas
  (`AGENTS.md` y sus hermanas), Then documentan el comando `leccion`, el formato,
  la regla `require_leccion` y su default apagado.
- AC-18: Given `docs/lecciones/` con contenido, When se corre `harness_check.sh`,
  Then un frontmatter ilegible o un `nombre` que no coincide con el nombre del
  archivo **bloquea** nombrando el archivo; una leccion sin `triggers` solo avisa
  con `[i]`. Sin `docs/lecciones/` el bloque entero se omite y el check pasa igual.
- AC-19: Given lecciones escritas, When se corre el instalador con `--reset`,
  Then las lecciones **sobreviven** (son conocimiento ganado, como el PRD y la
  constitution) y solo se refresca `COMO-ESCRIBIR-UNA-LECCION.md`, que es
  plantilla del arnes.
- AC-20: Given el repo fuente, When se corre la verificacion oficial, Then
  `cargo test` y `cargo clippy --all-targets -- -D warnings` estan verdes con
  tests unitarios de validacion de nombre, parseo de frontmatter, telemetria de
  `usar` y las tres ramas del gate; y `tests/setup_smoke.sh` + `tests/setup_smoke.ps1`
  verifican siembra, idempotencia y supervivencia al `--reset`.

## Los datos que se tocan

- **disparador**: `close --status done` (declara lo aprendido) y `leccion usar`
  (registra que una leccion sirvio). En esta feature el disparo es **manual**; el
  automatico llega con el nudge (#18).
- **interruptor**: la regla `require_leccion` en `rules` de `feature_list.json`,
  **ausente o `false` por default**. Sin ella, el gate es mudo y el flujo es el de
  hoy.
- **candado**: `usos` + `ultimo_uso` en el frontmatter (telemetria que despues
  consume el curador, #21) y el campo opcional `leccion` en la entrada de la
  feature, que deja constancia de que esa feature ya declaro lo suyo.
- **entidad nueva**: `docs/lecciones/<clase>.md` (+ `docs/lecciones/<clase>/referencias/<tema>.md`
  para el detalle de una sesion). Nada de esto toca el hub.

## Pseudo-codigo (el acuerdo)

```
CUANDO se cierra una feature con estado done

  ¿la regla require_leccion esta activa?   -> si no, cerramos como siempre
  ¿el cierre declara una leccion?          -> si no, FALLAMOS con las dos
                                              salidas validas en el mensaje

  SI declara una clase:
     ¿esa clase existe en docs/lecciones/? -> si no, FALLAMOS y sugerimos
                                              'leccion nueva <clase>'
  SI declara 'ninguna':
     ¿trae motivo?                          -> si no, FALLAMOS

  ENTONCES cerramos, guardamos la declaracion en la feature y en la bitacora,
           con la restriccion de que el arnes NUNCA escribe el contenido de la
           leccion: lo escribe el agente, y esto solo verifica que se decidio.


CUANDO alguien pide una leccion nueva

  ¿el nombre es de CLASE?        -> si no, RECHAZAMOS con el motivo y ejemplos
  ¿ya existe esa clase?          -> si si, RECHAZAMOS y empujamos a patchearla

  ENTONCES creamos el esqueleto desde la plantilla,
           con la restriccion de que crear es el ULTIMO recurso del orden de
           preferencia, no el primero.
```

Promesas: una clase por tema, no una por feature · el arnes verifica que se
decidio, nunca decide que se aprendio · nada se borra · funciona sin hub · sin
LLM y sin dependencias nuevas de runtime.

## No funcionales

- **SLOs**: `leccion list` y `show` responden en el orden de milisegundos sobre un
  catalogo tipico (decenas de lecciones); ningun subcomando abre conexion al hub.
- **Seguridad**: las lecciones son archivos **versionados** del repo, asi que no
  llevan secretos: la plantilla lo dice explicitamente y el reviewer lo verifica.
  Ningun comando escribe fuera de `docs/lecciones/` ni fuera del proyecto.
- **Observabilidad**: exit codes estables (0 ok / 1 sin feature o sin catalogo /
  2 rechazo o gate), mensajes accionables que nombran el archivo y el remedio, y
  toda declaracion de cierre queda en `progress/history.md`.

## Fuera de alcance

- El **nudge automatico** que pide la revision cada N escrituras y en cada cierre
  (feature #18): aca el disparo es manual.
- El **perfil de usuario** y su inyeccion en las superficies (feature #19).
- El comando **`buscar`** (feature #20): esta feature ordena el conocimiento, no
  lo indexa.
- Las **transiciones automaticas** `activa -> stale -> archivada`, el pin, los
  backups y el rollback (feature #21). Aca el campo `estado` solo existe en el
  frontmatter para que el curador lo consuma despues.
- El **mapa** `journey` (feature #22).
- Cualquier llamada a un modelo: el arnes no genera el contenido de una leccion.

## Observaciones (decisiones pendientes)

Todas decididas por Alan el 2026-08-16, en el mismo acto de aprobacion del spec.
No queda ninguna observacion abierta: el implementer puede avanzar sin preguntar.

- OBS-1: Nombres rechazados, ¿con escape hatch o sin el? — **DECIDIDO: sin
  `--force`.** Hardline, como Hermes: si el nombre solo tiene sentido para la
  feature de hoy, el remedio es elegir un nombre de clase, no saltear la regla.
  Es la regla que impide que la biblioteca degenere en una lista plana de
  una-leccion-por-feature. Vinculante para AC-4: no existe flag que lo evite.
- OBS-2: `close --leccion <clase>` con una clase inexistente, ¿falla o la crea al
  vuelo? — **DECIDIDO: falla** (AC-12), para que un typo no deje la declaracion
  colgada apuntando a una leccion que no existe.
- OBS-3: La guia `COMO-ESCRIBIR-UNA-LECCION.md`, ¿plantilla del arnes o documento
  del usuario? — **DECIDIDO: plantilla del arnes**, refrescable al reinstalar y
  dentro de los reset targets, igual que `COMO-ESCRIBIR-UN-PRD.md`. Las lecciones
  en si NO son plantilla y sobreviven al reset (AC-19).
- OBS-4: ¿`harness_check.sh` bloquea por frontmatter ilegible o solo avisa? —
  **DECIDIDO: bloquea** (AC-18), con el mismo trato que el gate del arbol de PRDs:
  nombra el archivo y frena, para atrapar la leccion rota temprano. Una leccion
  sin `triggers` sigue siendo solo un aviso `[i]`.
- OBS-5: ¿El campo en `feature_list.json` se llama `leccion` y se escribe solo
  cuando se declara? — **DECIDIDO: si**, opcional e igual que `prd` y `kind`, para
  no migrar ni tocar las 16 features ya cerradas (AC-10).
