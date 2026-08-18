# Spec - Feature #21: curador_de_lecciones

Estado: approved
Aprobado: 2026-08-17T04:08:36Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #21 en el chat (AskUserQuestion: 'Si, lo apruebo'), con el spec mostrado en el chat y abierto en su editor. 20 AC. Decisiones OBS-1..OBS-5, tres de ellas correcciones al backlog: la consolidacion con LLM sale a feature aparte (unica parte que necesita modelo, apagada por default, no verificable aqui), adoptar NO se implementa (el arnes no distingue autoria de lecciones; pin cubre la necesidad), la pasada automatica solo INFORMA y mutar exige --aplicar, el archivo va en docs/lecciones/archivo/ VISIBLE porque buscar saltea los ocultos y el conocimiento archivado desapareceria, y umbrales 30/90 configurables por rules.
Plan: docs/plan-feature-21-curador-de-lecciones.md
PRD: docs/prd/aprendizaje/PRD-aprendizaje.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: el repo tiene 5 lecciones y una sola con uso registrado. Dentro de un ano
va a tener cuarenta, y nadie va a saber cuales siguen vivas. La #17 puso el campo
`estado` en el frontmatter y nunca lo mueve nadie; la telemetria (`usos`,
`ultimo_uso`) se registra y nadie la mira.

El final conocido de una biblioteca sin mantenimiento: se llena de casi
duplicados y de cosas que ya no son ciertas, el catalogo deja de leerse, y una
leccion vieja termina citada como verdad. **Una leccion equivocada es peor que
ninguna** — es la regla que la propia #17 escribio, y sin curador no hay forma de
hacerla cumplir.

DESPUES: `lecciones status` muestra que esta vivo, que se esta enfriando y que
esta por vencer. Las transiciones son **deterministas y sin modelo**: 30 dias sin
uso pasa a `stale`, 90 dias pasa a `archivada`. **Nunca se borra nada**, el
archivo es recuperable, `pin` congela lo que no se toca, y toda pasada que muta
deja backup y reporte.

Y lo mas importante en este arnes: **la pasada automatica solo informa**. Mover
archivos de alguien a sus espaldas, en un hook, no es curar: es perder cosas.

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES

frontmatter: estado: activa             lecciones status
  `__ nadie lo mueve nunca                |__ activas / por enfriarse / stale
                                          `__ "3 candidatas a archivar"
usos / ultimo_uso
  `__ se registran, nadie los mira      lecciones curar          (solo INFORMA)
                                          `__ que pasaria, sin tocar nada
(sin mantenimiento)
  `__ en un ano: 40 lecciones,          lecciones curar --aplicar
      la mitad casi duplicadas            |__ backup en bkp/ ANTES
                                          |__ activa -> stale -> archivada
                                          |__ reporte en progress/lecciones/<ts>/
                                          `__ NUNCA borra

                                        lecciones pin | archivar | restaurar
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero ver de un vistazo que lecciones siguen vivas y cuales se
  estan enfriando, sin abrir cinco archivos.
- P1: Como Alan, quiero que **nada se mueva sin que yo lo pida**: que la pasada
  automatica me avise y que mutar sea siempre explicito.
- P1: Como cualquiera, quiero poder deshacer una pasada completa si archivo algo
  que no correspondia.
- P1: Como Alan, quiero congelar una leccion que se usa poco pero que no quiero
  perder (`pin`).
- P2: Como reviewer, quiero un reporte de que hizo cada pasada, para auditarlo
  sin leer el diff.

## Criterios de aceptacion (Given/When/Then)

### Estado y telemetria

- AC-1: Given lecciones con distinta telemetria, When corre
  `sh harness_cli lecciones status`, Then se listan agrupadas por estado
  (`activa` / `stale` / `archivada` / `pinneada`) con sus usos, su ultimo uso y
  **cuantos dias faltan** para su proxima transicion; y se dice cuantas son
  candidatas a `stale` y a `archivada` **hoy**.
- AC-2: Given un repo sin `docs/lecciones/`, When corre cualquier subcomando de
  `lecciones`, Then se informa que no hay biblioteca todavia y exit 0.
- AC-3: Given `--json`, Then `status` expone por leccion: `nombre`, `estado`,
  `usos`, `ultimo_uso`, `dias_inactiva`, `pinneada`, `proxima_transicion` y
  `dias_para_transicion`.

### El ciclo de vida (determinista, sin modelo)

- AC-4: Given una leccion con `ultimo_uso` de hace **30 dias o mas** (o, si nunca
  se uso, con `ultima_actualizacion` de hace 30 dias o mas), When se aplica una
  pasada, Then pasa a `estado: stale`.
- AC-5: Given una leccion `stale` cuya inactividad llega a **90 dias**, When se
  aplica una pasada, Then se **mueve** a `docs/lecciones/archivo/` con
  `estado: archivada`. **Nunca se borra**: el peor resultado posible de una
  pasada automatica es un archivo movido y recuperable.
- AC-6: Given una leccion **nunca usada** (`usos: 0`), When su antiguedad desde
  `ultima_actualizacion` es menor a 30 dias, Then **no** se archiva ni se marca
  stale: cero usos es ausencia de evidencia, no prueba de que sobra.
- AC-7: Given una leccion **pinneada**, When se aplica cualquier pasada, Then
  **ninguna** transicion automatica la toca, sin importar su antiguedad.
- AC-8: Given una leccion que vuelve a usarse (`leccion usar`), When se aplica la
  siguiente pasada, Then vuelve a `activa`: el uso resucita.

### Nada se mueve sin pedirlo

- AC-9: Given `sh harness_cli lecciones curar` **sin** `--aplicar`, When corre,
  Then **informa** que transiciones ocurririan y **no toca ningun archivo**
  (ni contenido, ni ubicacion, ni mtime).
- AC-10: Given `lecciones curar --aplicar`, When corre, Then **antes** de mutar
  deja un backup del arbol de lecciones bajo `bkp/lecciones/<ts>/`, aplica las
  transiciones y escribe el reporte de la pasada.
- AC-11: Given una pasada aplicada, When se corre
  `sh harness_cli lecciones rollback`, Then el arbol vuelve al estado previo; y
  **antes** de restaurar se toma un backup del estado actual, de modo que el
  rollback tambien sea reversible.
- AC-12: Given `lecciones rollback --list`, Then se listan los backups
  disponibles con su fecha y que pasada los origino.

### Comandos manuales

- AC-13: Given `lecciones pin <clase>` / `unpin <clase>`, Then se marca o
  desmarca `pinneada` en el frontmatter, sin tocar el cuerpo ni la telemetria.
- AC-14: Given `lecciones archivar <clase>` / `restaurar <clase>`, Then la leccion
  se mueve a `docs/lecciones/archivo/` o vuelve de ahi, con su `estado`
  actualizado; `restaurar` de una clase que no esta archivada, o `archivar` de una
  ya archivada, sale con exit 2 y mensaje accionable.
- AC-15: Given un nombre de clase inexistente en cualquier subcomando, Then exit 2
  con las clases de nombre mas parecido (mismo trato que `leccion show`).

### Reporte y auditoria

- AC-16: Given una pasada aplicada, Then queda
  `progress/lecciones/<ts>/REPORT.md` con: que se evaluo, que transiciono y por
  que (dias de inactividad de cada una), que se salteo por `pin`, y donde quedo el
  backup. Y una linea en `progress/history.md`.
- AC-17: Given una pasada **sin** cambios, Then no se crea backup ni reporte
  vacio: no se ensucia el repo por correr un chequeo.

### Integracion con el resto del arnes

- AC-18: Given lecciones archivadas, When se corre `sh harness_cli buscar`, Then
  siguen apareciendo (el conocimiento no desaparece) pero **rankeadas por debajo**
  de las activas, para que una leccion vencida no le gane a una vigente.
- AC-19: Given una leccion archivada, When se corre `leccion list`, Then **no**
  aparece en el catalogo por defecto (se ve con `--archivadas`), y
  `harness_check.sh` sigue validando su formato sin bloquear por estar archivada.
- AC-20: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test` y `cargo clippy --all-targets -- -D warnings` estan verdes con
  tests de: cada transicion y su umbral exacto, el piso de gracia, el pin, la
  resurreccion por uso, el modo informe (no toca nada), backup + rollback +
  rollback del rollback, los comandos manuales y sus errores, y el reporte; y
  `tests/setup_smoke.sh` sigue verde.

## Los datos que se tocan

- **disparador**: `lecciones curar --aplicar` (explicito) y los comandos manuales.
- **interruptor**: `pin` por leccion; y los umbrales en `rules`
  (`leccion_stale_dias`, `leccion_archivo_dias`) para poder ajustarlos o apagar
  el ciclo de vida sin tocar codigo.
- **candado**: el backup previo a cada pasada mutante en `bkp/lecciones/<ts>/`, y
  el reporte por corrida que dice exactamente que se movio.
- **entidades**: el frontmatter de cada leccion (`estado`, `pinneada`), la carpeta
  `docs/lecciones/archivo/` y `progress/lecciones/<ts>/REPORT.md`.
- **lo que NO se toca**: el CUERPO de ninguna leccion (el curador mueve y marca,
  nunca reescribe contenido), el perfil, los specs, los planes y el hub.

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien pide una pasada del curador

  ¿existe docs/lecciones/?     -> si no, lo decimos y salimos con 0

  para cada leccion:
    ¿esta pinneada?            -> si si, la salteamos y lo anotamos
    calculamos su inactividad (desde ultimo_uso, o desde ultima_actualizacion
    si nunca se uso)
    decidimos su transicion: activa <-> stale -> archivada

  ¿vino --aplicar?             -> si NO, informamos que pasaria y salimos
                                  SIN tocar un solo archivo
  ¿hay algo que cambiar?       -> si no, no creamos backup ni reporte

  ENTONCES respaldamos primero, aplicamos, y escribimos el reporte,
           con la restriccion de que NUNCA borramos: archivar es mover.
```

**Promesas:** nada se mueve sin `--aplicar` · nada se borra · toda pasada mutante
tiene backup y es reversible · el cuerpo de una leccion no se reescribe nunca ·
sin modelo y sin hub.

## No funcionales

- **SLOs**: la pasada recorre unas decenas de archivos chicos: milisegundos, sin
  red ni hub.
- **Seguridad**: solo mueve archivos dentro de `docs/lecciones/` y escribe en
  `bkp/` y `progress/`. Nunca borra, asi que el peor error posible es recuperable.
- **Observabilidad**: exit 0 en informe y en pasada limpia; exit 2 solo por uso
  invalido. Toda pasada mutante deja reporte + linea en `history.md`.

## Fuera de alcance

- El mapa `journey` (#22).
- **La consolidacion asistida por LLM**: ver OBS-1.
- Reescribir el contenido de una leccion: el curador mueve y marca, no edita.
- Cualquier borrado. No existe un subcomando que borre.

## Observaciones (decisiones pendientes)

Todas decididas por Alan el 2026-08-17, en el mismo acto de aprobacion del spec.
No queda ninguna observacion abierta: el implementer puede avanzar sin preguntar.

Tres de las cinco son **correcciones al backlog**, que se habia escrito por
analogia con Hermes antes de trabajar la feature.

- OBS-1: ¿Entra la consolidacion asistida por LLM? — **DECIDIDO: no; queda como
  feature aparte del backlog.** Es la unica parte que necesita un modelo, esta
  apagada por default (cero valor el dia uno) y no se podria verificar de punta a
  punta en este entorno: exactamente la deuda que la #20 evito a proposito con el
  hub. La #21 entrega el ciclo de vida completo, determinista y testeable.
- OBS-2: ¿Se implementa `adoptar`? — **DECIDIDO: no.** En Hermes distingue skills
  "del usuario" de las creadas por el agente; **en el arnes esa distincion no
  existe**: todas las lecciones nacen del mismo comando y ninguna lleva marca de
  autoria. `pin` ya cubre la necesidad real. Agregar una marca de procedencia solo
  para tener `adoptar` seria complejidad sin problema que resolver.
- OBS-3: ¿La pasada automatica puede mutar? — **DECIDIDO: no.** Solo informa
  cuando hay candidatas; mutar exige `curar --aplicar` a mano. Mover archivos de
  alguien en un hook, sin que lo pida, no es curar. Vinculante para AC-9.
- OBS-4: ¿Donde va el archivo? — **DECIDIDO: `docs/lecciones/archivo/`, carpeta
  visible.** El motivo es concreto: `buscar` (#20) saltea los directorios ocultos,
  asi que un `.archivo/` como el de Hermes haria **desaparecer** el conocimiento
  archivado de las busquedas — justo lo que la #20 vino a arreglar. Visible, sigue
  siendo consultable pero deja de competir con lo vigente (AC-18).
- OBS-5: ¿Los umbrales? — **DECIDIDO: 30 y 90 dias**, configurables por `rules`
  (`leccion_stale_dias`, `leccion_archivo_dias`) y desactivables con `0`.
