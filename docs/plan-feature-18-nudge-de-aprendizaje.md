# Plan - Feature #18: nudge_de_aprendizaje

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 2 del PRD `docs/prd/aprendizaje/PRD-aprendizaje.md`: que el arnes **empuje
solo** a capturar lo aprendido, en los dos momentos con senal real (cada N
escrituras y al cerrar sin declaracion), y que el aviso de "sin feature activa"
deje de repetir lo mismo para siempre.

El arnes **emite**, nunca escribe: el contrato sale por stderr, el agente decide
y escribe, el gate de la #17 verifica. Ninguna llamada a un modelo.

Spec aprobado (21 AC): `docs/spec-feature-18-nudge-de-aprendizaje.md`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> el hub
PostgreSQL sigue sin responder en este entorno (`connection timed out`), igual
que en la #17. No bloquea: es best-effort en todo el binario y el AC-17 exige
justamente que el nudge y el contrato no dependan del hub.

Impacto por inspeccion (un solo microservicio, `harness`):

- `rust/src/commands/nudge.rs` — reescrito: dos disparadores nuevos + backoff.
- `rust/src/lecciones.rs` — suma el lector del contrato (la guia es la fuente).
- `rust/src/commands/close.rs` — una llamada al final, despues de cerrar.
- `rust/src/paths.rs` — un campo nuevo (`nudge_lecciones`).
- Docs, roles y superficies. **No** se tocan los instaladores en su logica de
  hooks: el evento `PostToolUse` y su matcher ya existen y quedan igual.

Riesgo para lo existente: el unico camino que cambia de comportamiento observable
es el aviso de "sin feature activa" (ahora escala) y el stderr de `close`. Los
exit codes y el stdout no se tocan (AC-10).

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`graphify query "nudge hook post-tool debounce"` + lectura directa de
`setup_harness.sh`. Lo que decidio el diseno:

- `bin/harness-hook` despacha `post-tool` -> `harness_cli nudge`, y el matcher
  instalado es `Bash|Edit|Write|apply_patch`. **Consecuencia**: el contador no
  necesita infraestructura nueva — cada invocacion del nudge YA es "una escritura
  o comando", y por eso el default de 25 (OBS-7) y no 10.
- `autocheck.rs` es el precedente de "estado local en `progress/` + mtime como
  reloj" (`.last_autocheck`). El contador y el nivel de backoff siguen el mismo
  patron, sin formato nuevo que mantener.
- `paths.rs` ya expone `nudge_stamp`: agregar `nudge_lecciones` al lado mantiene
  toda la resolucion de rutas en un solo lugar.

## Delegacion (implementer)

- **D1 (AC-11, AC-12, AC-13, AC-14)** — Backoff en `nudge.rs`:
  `progress/.last_nudge` pasa a guardar el **nivel** como texto (su mtime sigue
  siendo el reloj). Intervalo = `min(600 * 2^nivel, 3600)`. Emitir sube el nivel;
  encontrar una feature activa lo devuelve a 0 (y solo escribe si no estaba ya en
  0, para no tocar el archivo en cada tool-use). Contenido vacio o ilegible =>
  nivel 0 (compatibilidad con instalaciones previas).
- **D2 (AC-1, AC-2, AC-3, AC-4, AC-5)** — Contador en `progress/.nudge_lecciones`
  con formato `<id-feature>:<contador>`: cambiar de feature resetea (AC-4).
  Intervalo desde `rules.leccion_nudge_interval` (default **25**, `<= 0` apaga).
  **Guarda de entrada**: sin `docs/lecciones/` no se lee ni se crea nada (AC-3).
  Al llegar al intervalo, recordatorio corto (<= 5 lineas) por stderr y reset.
- **D3 (AC-6, AC-21)** — Lector del contrato en `lecciones.rs`: extrae de
  `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md` las secciones
  `## La regla que ordena todo: primero patchear, crear al final` y
  `## Que NO capturar` (desde su encabezado hasta el proximo `## `). Si la guia
  falta, esta vacia o le falta alguna de las dos, **degrada** a un puntero de dos
  lineas. Nunca falla ni propaga error.
- **D4 (AC-6, AC-7, AC-8, AC-9, AC-10)** — Enganche en `close.rs`: al final del
  cierre, si `status == done`, existe `docs/lecciones/` y **no** hubo
  declaracion, emitir el contrato por **stderr**. Todo despues de que el stdout y
  el exit code ya quedaron fijados.
- **D5 (AC-15, AC-16, AC-17)** — Invariantes: todo el camino nuevo va envuelto en
  el `let _ = inner(...)` que ya existe en `nudge`, y en `close` se llama con la
  misma disciplina best-effort. Ningun camino toca lecciones ni artefactos, ni
  abre conexion al hub. Revisar que no quede ningun `?` que pueda escapar.
- **D6 (AC-18)** — `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md`
  (+ plantilla) y las superficies generadas por `setup_harness.sh` y
  `setup_harness.ps1`: los dos disparadores, `leccion_nudge_interval` con su
  default 25, el backoff y el hecho de que el contrato **se lee de la guia**.
- **D7 (AC-19)** — Roles: el implementer explica que hacer cuando ve el
  recordatorio; el reviewer verifica que un cierre sin declaracion haya recibido
  el contrato. Regenerar los espejos y dejar el gate limpio.
- **D8 (AC-20)** — Tests: unitarios del backoff (escalada, techo, reset,
  compatibilidad con archivo vacio), del contador (intervalo, apagado, cambio de
  feature, sin `docs/lecciones/`) y del lector del contrato (extraccion y las
  tres formas de degradar); integracion de las tres ramas del cierre y del
  invariante de exit 0. `tests/setup_smoke.sh` verde.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-21 en `docs/impl-18.md`; veredicto por AC en
  `docs/review-18.md`.
- `cargo test` y `cargo clippy --all-targets -- -D warnings` verdes;
  `bash tests/setup_smoke.sh` verde; `bash harness_check.sh` limpio.
- **Demostrar el invariante**: `close` con y sin contrato tiene el MISMO exit code
  y el MISMO stdout; el contrato solo aparece en stderr.
- **Demostrar que no hay ruido**: un proyecto sin `docs/lecciones/` no ve ninguna
  linea nueva ni gana ningun archivo en `progress/`.
- El contrato emitido coincide con lo que dice la guia (se lee de ahi, no hay
  copia que pueda divergir).
- `templates/` y raiz espejados.
- El hito 2 del PRD `aprendizaje` queda marcado por el cierre.
- **Nuevo en este repo**: `require_leccion` esta ACTIVA desde esta feature
  (decision de Alan, 2026-08-16), asi que el cierre va a exigir la declaracion.
  Es el primer cierre del arnes con su propio gate puesto.

## Riesgos

- **Que el recordatorio se vuelva ruido.** Es el riesgo principal y por eso el
  default subio a 25 (OBS-7), el texto es corto, va a stderr y se apaga con
  `leccion_nudge_interval: 0`. Si igual molesta, la palanca ya existe.
- **Que leer la guia rompa un cierre.** Mitigacion: AC-21 exige degradar a un
  puntero ante cualquier problema, y el lector no propaga errores.
- **Que el contador se escriba en cada tool-use.** Es una escritura chica en
  `progress/` (ya se hace con `.last_autocheck`), y sin `docs/lecciones/` ni
  siquiera se crea el archivo.
- **Regresion silenciosa en el aviso de plan stale.** No se toca, pero comparte
  funcion: los tests existentes de `nudge` tienen que seguir verdes.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las siete observaciones del spec fueron decididas por Alan el
2026-08-16 y estan en la seccion "Observaciones" del spec y en el sello:

- OBS-1 sin `docs/lecciones/` no se emite nada -> D2, D4.
- OBS-2 el intervalo vive en `rules`, no en env -> D2.
- OBS-3 el contrato solo cuando no hubo declaracion -> D4.
- OBS-4 backoff 600 s -> 3600 s (techo de una hora, se descarto el de un dia) -> D1.
- OBS-5 recordatorio y aviso de plan stale son independientes -> D2.
- OBS-6 el contrato se LEE de la guia, no se duplica en el binario -> D3.
- OBS-7 default 25 y no 10, para que no se vuelva ruido de fondo -> D2.

Decisiones de sesion aplicadas fuera del alcance de esta feature, registradas
aca para trazabilidad: `require_leccion` quedo **activa en este repo** y
`docs/prd/PRD-master.md` recibio sus cinco hitos (features #23 a #27).

### Avance 2026-08-16T23:08:41Z
Plan de la #18 escrito: D1-D8 citando cada AC, impacto (hub caido, documentado), consulta al grafo y riesgos. Las 7 observaciones quedaron decididas por Alan en el acto de aprobacion. Fuera de alcance pero aplicado en la sesion: require_leccion activa en este repo y PRD-master con sus 5 hitos (#23-#27).

### Avance 2026-08-16T23:24:47Z
D1-D8 implementados: backoff con nivel en .last_nudge (600->3600, reset al piso), contador por feature en .nudge_lecciones (default 25, apagable), lector del contrato que LEE la guia con degradacion a puntero, enganche en close a stderr sin tocar stdout ni exit code, docs/roles/superficies y 19 tests nuevos (incluido el anti-drift contra la guia real). El pase de reviewer corrigio el mensaje: decia 'escrituras' y contaba tool-calls.

---
Cerrado: 2026-08-16T23:25:03Z - status=done - El arnes empuja solo: recordatorio cada N acciones (default 25, apagable por rules) y CONTRATO completo al cerrar sin declarar, leido de la guia y no duplicado en el binario (con degradacion a puntero). Ademas el aviso de 'sin feature activa' pasa a backoff 600s->3600s con reset al piso. 21 AC cubiertos, sin parciales; todo a stderr y con exit 0 invariante; un proyecto sin docs/lecciones/ no ve nada nuevo.
