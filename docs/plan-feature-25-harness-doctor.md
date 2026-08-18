# Plan - Feature #25: harness_doctor

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 3 del `PRD-master`: `sh harness_cli doctor [--json]` diagnostica la
**instalacion** —binario, hooks, superficies, marker, hub, herramientas,
graphify— e imprime **el comando exacto de remedio** por cada problema. Exit 0
si solo hay avisos, 2 si hay algo que impide trabajar. No arregla nada y no
repite ningun chequeo de `harness_check.sh`.

Spec aprobado (20 AC, cada uno con su `Comando:`):
`docs/spec-feature-25-harness-doctor.md`.

## Peldano elegido: 3 (comando nuevo) para el diagnostico, 1 (extender) para el lanzador

Primera aplicacion real de la escalera de `docs/conventions.md` (#24), y sale
**hibrida**: la feature se parte en dos mitades y cada una toma el peldano mas
alto que la resuelve.

| Peldano | ¿Alcanzaba para el diagnostico? |
| --- | --- |
| 1. extender lo que existe | **NO**. El unico lugar donde cabria es `harness_check.sh`, que **bloquea el proceso** (exit 2 desde los hooks). Una instalacion a medias pasaria a impedir commitear, y hoy no lo hace: seria una regresion de comportamiento disfrazada de mejora |
| 2. flag en un comando existente | **NO**. `harness_check.sh --install` funcionaria aunque el binario este roto —su unica ventaja real, y es grande— pero tendria que **reimplementar en shell la resolucion de rutas del binario**. Esa logica ya se duplico una vez y costo la feature #10 entera. Duplicarla otra vez para diagnosticarla es reabrir el bug que se quiere detectar |
| **3. comando nuevo** | **SI, elegido.** Necesita `--json` (AC-4), exit codes propios (AC-3) y sobre todo consultar la resolucion de rutas **desde adentro**, que es exactamente lo que hay que diagnosticar |
| 4. superficie nueva | no |
| 5. dependencia nueva | no (Articulo 6 satisfecho sin ADR) |

**Peldano elegido: 3 (comando nuevo) porque el peldano 1 convertiria una
instalacion incompleta en un bloqueo del proceso, y el peldano 2 obligaria a
reimplementar en shell la resolucion de rutas del binario — la misma duplicacion
que costo la feature #10.**

Y la mitad que el peldano 3 **no** puede resolver toma el peldano 1: un doctor
que vive en el binario no puede diagnosticar un binario ausente o viejo. Eso se
arregla extendiendo `harness_cli`, el lanzador que ya existe (AC-16). Cero
superficie nueva para esa mitad.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder, como en las ocho features anteriores.

Impacto por inspeccion (un microservicio, `harness`):

- `rust/src/doctor.rs` (NUEVO): las siete areas, puras (leen y devuelven).
- `rust/src/commands/doctor.rs` (NUEVO) + `cli.rs`: el comando y el render.
- `harness_cli` + `templates/harness_cli`: reconocer el binario viejo.
- `tests/doctor_launcher_check.sh` (NUEVO): el AC-16, que es de shell y no de
  Rust porque prueba el lanzador.
- Docs, superficies del instalador y el rol del implementer.

**Riesgo distinto al de las features anteriores**: doctor es un **reporte de
problemas**, y la leccion `probar-contra-datos-reales` es explicita sobre eso —
un falso positivo cuesta mas que un falso negativo, porque el primero hace que se
ignore la herramienta entera. Por eso el AC-13 exige que `doctor` salga **0 en
este repo**, que es un checkout fuente sin superficies ni hooks instalados.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "resolucion de raiz marker layout"` y
`"binario viejo unrecognized subcommand"`. Lo que decidio el plan:

- Las fallas que doctor detecta **no se inventaron**: cada una ya ocurrio aca.
  Binario viejo tras `git pull` (parcheado a mano en `harness_check.sh` en la
  #19, y de nuevo despues), marker perdido (#10), checkout fuente confundido con
  instalacion (#7), hub caido (toda esta sesion). Disenar sobre el historial en
  vez de sobre la imaginacion es la misma leccion que
  `probar-contra-datos-reales` aplica a los tests.
- `HarnessPaths::resolve()` ya tiene toda la logica de resolucion y ya emite el
  `[i] Checkout fuente del arnes detectado`. Doctor **consulta** ese resultado en
  vez de recalcularlo: si algun dia cambia la resolucion, doctor sigue diciendo
  la verdad.
- La deteccion de "checkout fuente" que necesita el AC-12 es la misma del
  guardrail de la #7; se reusa, no se reescribe.

## Delegacion (implementer)

- **D1 (AC-3, AC-4)** — `rust/src/doctor.rs`: `Estado` como enum
  (`Ok`/`Falla`/`Aviso`/`NoAplica`) con `bloquea()` exhaustivo, y `Area` con su
  `detalle` y su `remedio: Option<String>`. `diagnosticar()` es **pura**: lee y
  devuelve `Vec<Hallazgo>`, sin imprimir ni escribir. Esa separacion es lo que
  hace estructural la promesa del AC-15 (leccion
  `promesas-estructurales-vs-disciplina`).
- **D2 (AC-5, AC-8)** — Binario y marker: presencia, permiso de ejecucion, y
  **binario mas viejo que los scripts** por mtime contra `harness_cli` y
  `harness_check.sh`. El marker se contrasta con la raiz que `HarnessPaths`
  resolvio, informando cual eligio y por que.
- **D3 (AC-6, AC-7, AC-12)** — Hooks y superficies, con la deteccion de checkout
  fuente que devuelve `NoAplica`. Solo se exige la superficie de un backend cuya
  huella esta presente: pedir `GEMINI.md` a quien no usa Gemini es ruido.
- **D4 (AC-9, AC-10, AC-11)** — Hub (timeout corto, siempre aviso), herramientas
  requeridas (`git`; `cargo` solo si hay `rust/`) contra opcionales, y graphify.
- **D5 (AC-1, AC-2)** — `rust/src/commands/doctor.rs`: render con `[ok]`/`[!!]`/
  `[i]`/`[--]`, el remedio en su propia linea, `--json`, y el pie que remite a
  `harness_check.sh` para el proceso.
- **D6 (AC-16)** — `harness_cli` (+ espejo): si el binario existe pero rechaza el
  subcomando (`unrecognized subcommand`), imprimir el remedio en vez del error de
  clap. `tests/doctor_launcher_check.sh` lo prueba con un binario falso.
- **D7 (AC-14, AC-15, AC-17, AC-18)** — El no-solapamiento como test, la ausencia
  de escrituras como test, docs, superficies y el rol del implementer.
- **D8 (AC-13, AC-19, AC-20)** — Corrida real en este repo (tiene que salir 0),
  `Peldano elegido:` en este plan, y la verificacion oficial completa.

## Criterios de cierre (reviewer)

Escritos para poder fallar (`criterios-de-cierre-que-se-pueden-fallar`) y
verificados contra datos reales (`probar-contra-datos-reales`):

- Evidencia por AC-1..AC-20 en `docs/impl-25.md`; veredicto en `docs/review-25.md`.
- `sh harness_cli verify --feature 25` **verde**, con sus 20 comandos.
- **`doctor` sale 0 en ESTE repo** y cada linea de su salida se verifica **a
  mano**: que lo que dice `ok` este realmente ok y que lo que dice `no_aplica`
  corresponda. Un falso positivo hunde la herramienta entera.
- **La prueba del rojo, area por area**: por cada una de las siete, romperla a
  proposito en un sandbox y confirmar que doctor la reporta con su remedio.
  Cubierto por los tests, pero se corre ademas una vez a mano sobre el caso que
  mas duele: el binario viejo.
- **Doctor no escribe**: mtimes del arbol antes y despues, identicos.
- **El remedio se puede copiar y pegar**: cada `remedio` es un comando ejecutable
  tal cual, no una frase. Se revisa uno por uno.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash harness_check.sh`: todo verde.
- Hito 3 del `PRD-master` marcado por el cierre, con declaracion de leccion.

## Riesgos

- **El falso positivo.** Riesgo central. Un doctor que reporta cosas que estan
  bien se ignora en dos dias, y con el se ignoran los problemas reales. Mitigado
  por el AC-12/AC-13 (checkout fuente), por exigir solo las superficies del
  backend presente, y por verificar la salida a mano contra este repo.
- **Solaparse con `harness_check.sh`.** Dos herramientas que dicen lo mismo con
  palabras distintas confunden mas que una sola. Mitigado por el AC-14, que lo
  vuelve un test, y porque cada salida remite a la otra.
- **El limite estructural.** Doctor no puede diagnosticar un binario ausente. En
  vez de disimularlo, esta declarado (AC-16) y la mitad que falta se cubre en el
  lanzador.
- **La deteccion de "binario viejo" por mtime.** Es una heuristica: un `touch`
  la enganaria. Se acepta porque el caso real es `git pull`, que actualiza los
  mtimes de los scripts, y porque el costo del falso positivo aca es bajo (el
  remedio es re-correr el instalador, que es idempotente).

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cuatro del spec fueron decididas por Alan el 2026-08-17
antes de aprobar:

- OBS-1 peldano **3 + 1 hibrido** -> este plan y D6.
- OBS-2 **sin solapar** con harness_check -> D7 (AC-14).
- OBS-3 **`no_aplica`** en checkout fuente -> D3.
- OBS-4 **falla solo lo que impide trabajar** -> D1.

## Skills aplicadas

- **`rust-patterns`**: `Estado` como enum con matcheo exhaustivo (sexto uso del
  mismo patron), y `diagnosticar()` puro separado del render — la promesa "no
  escribe nada" se sostiene porque el modulo no tiene con que escribir.
- **`rust-best-practices`**: doctor **consulta** `HarnessPaths::resolve()` en vez
  de recalcular la resolucion de rutas; reusar la fuente de verdad es lo que
  evita que el diagnostico y el comportamiento se separen.
- **`rust-testing`**: cada area se prueba rompiendola en un sandbox (la prueba
  del rojo), y el AC-13 corre el comando contra el repo real, que es el caso que
  las fixtures no pueden representar.

### Avance 2026-08-17T19:10:00Z
Plan de la #25 escrito: D1-D8 citando cada AC. Primera aplicacion real de la escalera de la #24, y sale hibrida: peldano 3 para el diagnostico (con la razon escrita de por que el 1 y el 2 no alcanzaban) y peldano 1 para el arreglo del lanzador. Las siete areas diagnostican fallas que este repo YA sufrio, no fallas imaginadas.

### Avance 2026-08-17T19:04:42Z
Plan de la #25 escrito: D1-D8 citando cada AC. Primera aplicacion real de la escalera de la #24 y sale hibrida: peldano 3 para el diagnostico (con la razon escrita de por que el 1 y el 2 no alcanzaban: el 1 convertiria una instalacion incompleta en bloqueo del proceso, el 2 obligaria a reimplementar en shell la resolucion de rutas que costo la #10) y peldano 1 para el arreglo del lanzador. Las siete areas diagnostican fallas que este repo YA sufrio.

### Avance 2026-08-17T19:28:32Z
Feature #25 implementada: harness_cli doctor diagnostica las siete areas de la instalacion con remedio por linea, y el lanzador cubre la mitad que un doctor dentro del binario no puede (binario ausente o viejo). La escalera de la #24 partio la feature en dos peldanos distintos. Hallazgo: el chequeo del hub estuvo a punto de cerrar con un OK falso (TCP conecta, las operaciones morian con connection reset); la linea ahora dice exactamente que midio.

---
Cerrado: 2026-08-17T19:28:53Z - status=done - doctor diagnostica la instalacion (siete areas) con el comando exacto de remedio por problema; exit 2 solo si algo impide trabajar. La escalera de la #24 partio la feature: peldano 3 para el diagnostico, peldano 1 (el lanzador) para la mitad que un doctor dentro del binario no puede cubrir.
