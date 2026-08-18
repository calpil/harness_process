# Plan - Feature #21: curador_de_lecciones

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 5 del PRD `docs/prd/aprendizaje/PRD-aprendizaje.md`: el mantenimiento de la
biblioteca. Ciclo de vida **determinista y sin modelo** (`activa` -> `stale` ->
`archivada`), `pin`, comandos manuales, backup + rollback, y reporte por pasada.

Dos limites que definen la feature tanto como lo que hace:

- **Nunca borra.** No existe un subcomando que borre. Archivar es mover.
- **Nada se mueve sin `--aplicar`.** La pasada automatica solo informa (OBS-3).

Spec aprobado (20 AC): `docs/spec-feature-21-curador-de-lecciones.md`.
La consolidacion con LLM salio a la feature **#28** (OBS-1).

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder, como en las cuatro features anteriores. Irrelevante: el curador no lo
toca.

Impacto por inspeccion (un microservicio, `harness`):

- `rust/src/lecciones.rs` — se extiende: `pinneada`, edad, transiciones. **El
  modulo ya existe desde la #17**, asi que esta feature agrega funciones, no un
  modulo nuevo (peldano de menor huella).
- `rust/src/curador.rs` (NUEVO) — la pasada, el backup y el reporte.
- `rust/src/commands/leccion.rs` — subcomandos `lecciones *`.
- `rust/src/buscar.rs` — **AC-18**: una leccion archivada tiene que seguir
  apareciendo pero por debajo de una activa. Es el unico punto donde esta feature
  toca codigo de otra.
- Docs, roles y superficies.

**Riesgo para lo existente**: acotado a `buscar` (una fuente nueva con su peso) y
a `leccion list`, que deja de mostrar las archivadas por default. Todo lo demas
es agregado.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "leccion estado archivada"` — **primera feature que usa la
#20 en su propio diseno** en vez de `graphify query`. Lo que devolvio y que
decidio el plan:

- La #17 ya dejo `estado` en el frontmatter con los tres valores previstos
  (`activa` / `stale` / `archivada`) y `Frontmatter::set` preserva orden y claves
  desconocidas. **Consecuencia**: marcar `pinneada` y mover `estado` no necesita
  formato nuevo ni migracion; se escribe con la funcion que ya existe.
- `lecciones::scan` ya separa validas de rotas y saltea la guia. La pasada del
  curador se monta sobre `scan`, no sobre un recorrido propio.
- `bkp/` ya tiene politica de backups del instalador (`backup_file`, `HARNESS_BKP_DIR`).
  El backup del curador vive ahi para no inventar una segunda convencion.

## Delegacion (implementer)

- **D1 (AC-4, AC-6, AC-7, AC-8)** — En `lecciones.rs`: `pinneada()` sobre el
  frontmatter, `dias_inactiva()` (desde `ultimo_uso`; si nunca se uso, desde
  `ultima_actualizacion`) y `transicion(leccion, hoy, umbrales) -> Transicion`.
  `Transicion` es un **enum** (`Ninguna` / `AStale` / `AArchivada` / `AActiva`),
  no un `Option<&str>`: cada caso tiene su regla y su mensaje. Funcion **pura**,
  testeable sin filesystem ni esperar 90 dias.
- **D2 (AC-5, AC-18, AC-19)** — La carpeta `docs/lecciones/archivo/` (visible,
  OBS-4): `scan` la excluye del catalogo por defecto y la incluye con
  `--archivadas`; `buscar` gana `Fuente::LeccionArchivada` con peso **menor que
  cualquier fuente activa pero mayor que la bitacora**, para que el conocimiento
  archivado siga siendo consultable sin competir con lo vigente.
- **D3 (AC-1, AC-3)** — `lecciones status`: agrupado por estado, con usos, ultimo
  uso, dias de inactividad y **cuantos dias faltan** para la proxima transicion;
  y el resumen de candidatas de hoy. `--json` con los campos del AC-3.
- **D4 (AC-9, AC-10, AC-17)** — `rust/src/curador.rs`: `pasada()` calcula el plan
  de transiciones (puro) y `aplicar()` lo ejecuta. Sin `--aplicar` solo se
  imprime: **ni un archivo tocado, ni un mtime cambiado**. Sin cambios, no se
  crea backup ni reporte.
- **D5 (AC-10, AC-11, AC-12)** — Backup en `bkp/lecciones/<ts>/` (copia del arbol)
  antes de mutar; `lecciones rollback` restaura el mas reciente **tomando antes un
  backup del estado actual**, asi que deshacer tambien se deshace;
  `rollback --list` con fecha y origen de cada uno.
- **D6 (AC-13, AC-14, AC-15)** — `pin` / `unpin` (marcan `pinneada` sin tocar
  cuerpo ni telemetria), `archivar` / `restaurar` manuales con sus errores
  (archivar algo ya archivado, restaurar algo que no lo esta), y clase inexistente
  con sugerencias reusando `lecciones::parecidas`.
- **D7 (AC-16)** — Reporte `progress/lecciones/<ts>/REPORT.md`: que se evaluo, que
  transiciono con sus dias de inactividad, que se salteo por `pin`, y donde quedo
  el backup. Mas la linea en `progress/history.md`.
- **D8 (AC-2)** — Sin `docs/lecciones/`: todos los subcomandos informan y salen 0.
- **D9 (AC-20)** — Tests: unitarios de `transicion` en sus umbrales **exactos**
  (29/30/89/90 dias), del piso de gracia, del pin, de la resurreccion por uso, y
  del calculo de dias; integracion del modo informe (comparando mtimes antes y
  despues), de backup + rollback + rollback del rollback, de los comandos
  manuales y sus errores, del reporte, y de que una archivada sigue apareciendo
  en `buscar` por debajo de una activa.
- **D10 (AC-19 docs)** — Docs (README, UPDATING + espejo, architecture +
  plantilla, superficies) y roles: el reviewer mira `lecciones status` antes de
  cerrar; nadie corre `--aplicar` sin decirselo al usuario.

## Criterios de cierre (reviewer)

Escritos para que se puedan **fallar** (leccion `criterios-de-cierre-que-se-pueden-fallar`):

- Evidencia por AC-1..AC-20 en `docs/impl-21.md`; veredicto por AC en
  `docs/review-21.md`.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash harness_check.sh`: todo verde.
- **El modo informe no toca nada**: capturar `find docs/lecciones -exec stat`
  antes y despues de `lecciones curar`, y que sean **identicos**.
- **El ciclo completo, con fechas falsas**: crear una leccion con `ultimo_uso` de
  hace 95 dias, correr `curar --aplicar`, y verificar que quedo en
  `docs/lecciones/archivo/` con `estado: archivada`; despues `rollback` y
  verificar que volvio a su lugar **con su contenido intacto** (diff vacio).
- **Que archivar no la borra de la busqueda**: `buscar` sobre un termino de la
  leccion archivada la encuentra, y por debajo de una activa que tambien matchea.
- **Que un `pin` sobrevive a una pasada**: leccion pinneada con 200 dias de
  inactividad sigue `activa` tras `--aplicar`.
- `templates/` y raiz espejados; espejos de roles regenerados.
- Hito 5 del PRD marcado por el cierre, con declaracion de leccion.

## Riesgos

- **Perder una leccion.** Es el riesgo grave y por eso hay tres barreras: nunca
  borra (archivar es mover), backup antes de cada pasada mutante, y `--aplicar`
  explicito. El criterio de cierre exige probar el rollback con diff vacio.
- **Que el curador archive algo vivo.** Mitigado por el piso de gracia de las
  nunca usadas, por `pin`, y porque el uso resucita.
- **Interaccion con `buscar`.** Una fuente nueva puede alterar el orden existente;
  el criterio de cierre exige verificar que una archivada quede POR DEBAJO de una
  activa, no que simplemente aparezca.
- **Umbrales no ejercitados en la realidad.** Ninguna leccion de este repo tiene
  30 dias. Los tests usan fechas falsas, que prueban la logica pero no el uso: la
  primera pasada real llega en un mes y vale mirarla.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cinco del spec fueron decididas por Alan el 2026-08-17,
y **tres de ellas corrigen el backlog** (que se escribio por analogia con Hermes):

- OBS-1 la consolidacion con LLM sale de esta feature -> es la **#28**.
- OBS-2 `adoptar` NO se implementa: el arnes no distingue autoria de lecciones.
- OBS-3 la pasada automatica solo informa -> D4.
- OBS-4 `docs/lecciones/archivo/` visible, para que `buscar` la siga viendo -> D2.
- OBS-5 umbrales 30/90 configurables por `rules` -> D1.

## Skills aplicadas

- **`rust-patterns`**: `Transicion` como enum con matcheo exhaustivo (D1) — el
  mismo criterio que `Coincidencia` en la #19 y `Fuente` en la #20; funciones
  puras separadas de la I/O para poder testear el ciclo sin esperar 90 dias.
- **`rust-best-practices`**: extender `lecciones.rs` en vez de crear un modulo
  paralelo (menor huella); `&str` en parametros; sin `unwrap` fuera de tests.
- **`rust-testing`**: umbrales probados en sus bordes **exactos** (29/30, 89/90),
  no "alrededor de"; helpers de fixture con fechas controladas.
- **`rust-async-patterns`**: no aplica; la pasada es I/O sincrono de archivos.

### Avance 2026-08-17T04:09:52Z
Plan de la #21 escrito: D1-D10 citando cada AC, criterios de cierre escritos para poder fallar (modo informe con stat antes/despues, ciclo completo con fechas falsas + rollback con diff vacio, archivada sigue en buscar por debajo de una activa, pin sobrevive a 200 dias). Primera feature que uso 'buscar' (#20) en su propio diseno en vez de graphify query. Las 5 observaciones decididas; 3 corrigen el backlog.

### Avance 2026-08-17T04:22:34Z
D1-D10 implementados: ciclo de vida determinista en lecciones.rs (Transicion como enum, umbrales configurables, piso de gracia, pin), curador.rs con planificar/aplicar separados (lo que hace estructural la promesa de 'no toca nada'), backup + rollback reversible, reporte por pasada, subcomandos lecciones *, integracion con buscar (archivada visible pero por debajo) y 29 tests nuevos. Los 4 criterios de cierre corridos end-to-end con fechas falsas.

---
Cerrado: 2026-08-17T04:22:41Z - status=done - Curador de lecciones: ciclo activa->stale->archivada determinista y sin modelo (30/90 dias configurables), pin que congela, y cuatro garantias probadas pudiendo fallar: nunca borra (archivar es mover), nada se mueve sin --aplicar, toda pasada mutante respalda y el rollback tambien es reversible, y archivar no la saca de buscar. La consolidacion con LLM salio a la #28 por no ser verificable aqui.
