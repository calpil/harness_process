# Plan - Feature #23: ac_ejecutables_verify

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 1 del `PRD-master`: que un AC pueda declarar **como se prueba**, que
`harness_cli verify` lo ejecute y registre, y que `close --status done` pueda
exigir el reporte verde y fresco.

Tres limites que definen la feature tanto como lo que hace:

- **Nada se ejecuta sin spec aprobado** (AC-5). Es la barrera contra ejecutar un
  comando que escribio un agente y nadie leyo.
- **Nada se ejecuta al cerrar** (AC-16): `close` LEE el reporte.
- **Nada se rompe**: sin `Comando:` y sin la regla, los 310 AC existentes y las 22
  features cerradas siguen exactamente igual.

Spec aprobado (20 AC, todos con su `Comando:`):
`docs/spec-feature-23-ac-ejecutables-verify.md`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder, como en las seis features anteriores. `verify` no lo toca.

Impacto por inspeccion (un microservicio, `harness`):

- `rust/src/verificacion.rs` (NUEVO) — parseo de los `Comando:`, ejecucion con
  timeout, resultado y reporte.
- `rust/src/commands/verify.rs` (NUEVO) + `cli.rs` — el comando.
- `rust/src/commands/close.rs` — el gate nuevo, en el mismo lugar que los dos que
  ya existen (spec y leccion).
- `rust/src/spec.rs` — la plantilla suma la linea `Comando:` como opcional.
- Docs, roles y superficies.

**Riesgo real, y es distinto al de las features anteriores**: esta es la primera
vez que el binario ejecuta un comando arbitrario. El riesgo no es romper lo
existente (sin la regla y sin `Comando:` nada cambia) sino **abrir una superficie
nueva**. Por eso las tres barreras estan en los AC, no en la prosa.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "gate close require spec aprobado"` (tercera feature
disenada con la #20). Lo que decidio el plan:

- `close.rs` ya tiene **dos** gates en cadena (`spec_gate` de la #3 y
  `lecciones::gate` de la #17), los dos validados **antes** de mutar la feature.
  El de verify entra ahi, con la misma forma: leer, decidir, fallar antes de
  tocar nada.
- `spec.rs` ya calcula frescura comparando firmas (`last_spec_sig`). El reporte
  usa el mismo criterio conceptual pero mas simple: **mtime del reporte contra
  mtime del spec**, porque el reporte no necesita detectar ediciones
  concurrentes, solo saber si es posterior.
- `wait-timeout` **ya esta en `Cargo.toml`** (la usa el worker de graphify). El
  timeout del AC-6 no agrega dependencia: Articulo 6 satisfecho sin ADR.

## Delegacion (implementer)

- **D1 (AC-1, AC-2, AC-3)** — `rust/src/verificacion.rs`: parseo del spec.
  `Verificacion { ac, comando: Option<String> }` por cada `- AC-n:`, tomando la
  linea `Comando:` que le sigue dentro del mismo item (con o sin backticks). Un AC
  sin comando queda con `None` y se reporta como **manual**, nunca como fallo.
- **D2 (AC-4, AC-5, AC-6, OBS-4)** — Ejecucion: `verify` exige `Estado: approved`
  (si no, exit 2), imprime cada comando ANTES de correrlo, lo ejecuta con
  `wait-timeout` (`rules.verify_timeout_segundos`, default 300) y sigue con los
  demas si uno falla o se cuelga. Comando inexistente o no ejecutable => rojo con
  su error.
- **D3 (AC-8, AC-9, AC-10, AC-11)** — Resultado y reporte: `Estado` es un **enum**
  (`Verde` / `Rojo` / `Timeout` / `Manual`); `docs/verify-<id>.md` con AC, comando,
  exit, duracion y la salida **acotada** de los fallos; `--json` con los campos del
  AC-10; `--solo <AC-n>` para iterar sobre uno.
- **D4 (AC-12..AC-16)** — Gate en `close.rs`: con `require_verify_green` y comandos
  declarados, exige reporte existente, **mas nuevo que el spec** y sin rojos,
  nombrando los que fallaron. **Solo lee**: ningun camino del cierre ejecuta.
- **D5 (AC-7)** — Verificar que `verify` no quede enganchado en ningun hook ni
  invocado desde otro comando. Se comprueba con un test que grepea el runtime de
  hooks generado.
- **D6 (AC-17)** — `spec.rs`: la plantilla del spec documenta `Comando:` como
  opcional, con un ejemplo, para que el proximo `start` lo ofrezca solo.
- **D7 (AC-18, AC-19)** — Docs (README, UPDATING + espejo, architecture +
  plantilla, superficies) y los tres roles.
- **D8 (AC-20)** — Tests: unitarios del parseo (con/sin backticks, AC sin comando,
  varios AC), de la clasificacion de estados y del formato del reporte;
  integracion del rechazo en draft, del timeout, de `--solo`, de `--json`, de las
  cuatro ramas del gate y de que el cierre no ejecuta.

## Criterios de cierre (reviewer)

Escritos para poder fallar (leccion `criterios-de-cierre-que-se-pueden-fallar`) y
verificados **contra datos reales** (leccion `probar-contra-datos-reales`):

- Evidencia por AC-1..AC-20 en `docs/impl-23.md`; veredicto en `docs/review-23.md`.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash harness_check.sh`: todo verde.
- **`verify` corrido sobre el spec REAL de esta feature**, que declara sus 20
  comandos: tiene que ejecutarlos y quedar verde. Es la prueba de que la feature
  se verifica a si misma.
- **La compatibilidad, medida**: correr `verify` sobre un spec viejo (de las 21
  features anteriores, 0 comandos declarados) tiene que informar que no hay nada
  que verificar y salir con **0**.
- **La barrera del draft**: con el spec en draft, `verify` se niega y **no ejecuta
  ni un comando** (verificable porque un comando que escriba un archivo no lo
  escribe).
- **El cierre no ejecuta**: cerrar con un reporte verde no dispara ningun comando
  (mismo metodo: un comando que dejaria rastro no lo deja).
- `templates/` y raiz espejados.
- Hito 1 del `PRD-master` marcado por el cierre, con declaracion de leccion.

## Riesgos

- **La superficie nueva de ejecucion.** Es el riesgo central. Mitigado por las
  tres barreras (spec aprobado, invocacion manual, comando impreso) y por que el
  cierre no ejecuta. Vale decir lo que NO mitiga: si el usuario aprueba un spec
  sin leer los comandos, la barrera no sirve. La barrera protege del descuido, no
  de aprobar a ciegas.
- **Que el reporte de una falsa sensacion de cobertura.** Un AC con un comando
  trivial (`true`) pasa igual. El reporte dice el comando de cada AC justamente
  para que el reviewer pueda juzgar si prueba algo.
- **Comandos que dependen del directorio.** Se ejecutan desde la raiz del
  proyecto; los `cd rust && ...` del propio spec lo asumen. Queda documentado.
- **Que nadie corra `verify`.** Sin la regla activa, es opcional y se olvida. Por
  eso la regla existe; encenderla es decision del proyecto.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cinco del spec fueron decididas por Alan el 2026-08-17:

- OBS-1 `Comando:` pegado al AC -> D1.
- OBS-2 el cierre LEE el reporte y exige frescura -> D4.
- OBS-3 `verify` exige spec aprobado -> D2 (la barrera central).
- OBS-4 comando que no corre = rojo -> D2.
- OBS-5 el reporte se versiona -> D3.

## Skills aplicadas

- **`rust-patterns`**: `Estado` como enum (`Verde`/`Rojo`/`Timeout`/`Manual`) con
  matcheo exhaustivo — quinto uso del mismo patron en este repo. Y separar el
  parseo (puro) de la ejecucion (efectos), que es lo que permite testear el
  parseo de 310 AC sin ejecutar nada.
- **`rust-best-practices`**: reusar `wait-timeout`, que ya es dependencia, en vez
  de agregar una (Articulo 6); el gate nuevo entra en el mismo lugar y con la
  misma forma que los dos que ya existen en `close.rs`.
- **`rust-testing`**: el parseo se prueba con specs sinteticos Y con los specs
  reales del repo (310 AC, 0 comandos) para que la compatibilidad sea un test y
  no una promesa.

### Avance 2026-08-17T05:13:19Z
Plan de la #23 escrito: D1-D8 citando cada AC. El riesgo central es distinto al de las features anteriores: es la primera vez que el binario ejecuta comandos arbitrarios, y las tres barreras (spec aprobado, invocacion manual, comando impreso) estan en los AC y no en la prosa. wait-timeout ya es dependencia, asi que el timeout no agrega ninguna (Articulo 6 sin ADR).

### Avance 2026-08-17T05:51:49Z
Feature #23 implementada: verify ejecuta los Comando: que declaran los AC y close los exige con require_verify_green (encendida en este repo por decision de Alan). Dos hallazgos de la corrida real sobre su propio spec: el parser ejecutaba el ejemplo del bloque ``` que ENSENA el formato, y 8 de 20 AC daban verde sin correr nada porque cargo test con un filtro sin coincidencias sale 0. Los dos arreglados y escritos en la leccion.

---
Cerrado: 2026-08-17T05:52:08Z - status=done - AC ejecutables: el spec declara Comando: por criterio, verify los corre (20/20 verde sobre su propio spec) y close exige el reporte verde y fresco. Tres barreras: spec aprobado, invocacion manual, comando impreso; el cierre LEE, nunca ejecuta.
