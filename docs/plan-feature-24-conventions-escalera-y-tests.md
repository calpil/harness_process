# Plan - Feature #24: conventions_escalera_y_tests

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 2 del `PRD-master`: `docs/conventions.md` (+ espejo) deja de ser una lista de
buenos deseos de 7 lineas y pasa a llevar dos cosas que se pueden usar para
**rechazar** algo:

- **La escalera de huella** (5 peldanos, cada uno con un ejemplo real de este
  repo) y la obligacion de escribir `Peldano elegido:` cuando no se toma el mas
  alto.
- **Las tres reglas de test** (contratos y no snapshots; prohibido leer el
  fuente; prohibido el detector-de-cambios), con contraejemplo y version
  correcta en Rust.

Y la deuda que la regla descubre se paga aca mismo: el test de la #23 que lee el
fuente se reescribe como contrato de comportamiento.

Spec aprobado (17 AC, cada uno con su `Comando:`, ninguno repetido):
`docs/spec-feature-24-conventions-escalera-y-tests.md`.

## Peldano elegido: 1 (extender lo que ya existe)

La escalera se aplica a si misma, y es el criterio AC-16.

| Peldano | ¿Alcanzaba? |
| --- | --- |
| 1. extender lo que existe | **SI**. `docs/conventions.md` ya existe (se siembra y se espeja); `harness_check.sh` ya existe y ya tiene bloques opcionales (el de lecciones, que se omite sin `docs/lecciones/`). El chequeo nuevo es un bloque mas, con la misma forma |
| 2. flag en un comando existente | innecesario: no hay nada que parametrizar |
| 3. comando nuevo | innecesario, y ademas contradictorio: un `harness_cli conventions` sumaria superficie permanente para leer un markdown |
| 4. superficie nueva | no |
| 5. dependencia nueva | no (Articulo 6 satisfecho sin ADR) |

Cero comandos, cero flags, cero dependencias. Si esta feature hubiera necesitado
un comando, la escalera naceria contradicha por su propia implementacion.

`tests/conventions_check.sh` **no** es una superficie nueva: es un test, hermano
de `tests/setup_smoke.sh`, y existe para que los AC-8/10/11/12 tengan un comando
que pueda fallar (leccion `criterios-de-cierre-que-se-pueden-fallar`). La logica
que corre en produccion vive en `harness_check.sh`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder, como en las siete features anteriores.

Impacto por inspeccion (un microservicio, `harness`):

- `docs/conventions.md` + `templates/docs/conventions.md` (espejo exacto): el
  cuerpo de la feature.
- `harness_check.sh` + `templates/harness_check.sh`: el bloque de aviso.
- `tests/conventions_check.sh` (NUEVO): los cuatro modos que verifican el bloque.
- `rust/tests/cli_basics.rs`: el test de la #23 reescrito.
- `roles/*.md` (via `templates/roles/*.md`) y `.claude/agents/*.md`.
- `README.md`, `UPDATING.md` (+ espejo).

**Riesgo distinto al de las features anteriores**: esta no agrega capacidad, la
**restringe**. El peligro no es romper algo sino escribir reglas que nadie pueda
aplicar, o que la primera excepcion vacie. Por eso el AC-16 obliga a la feature a
pasar por su propia escalera y el AC-7 paga la deuda en vez de declararla
excepcion.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "escalera menor huella"` y `"detector de cambios"`. Lo que
decidio el plan:

- El origen esta en `docs/analisis-hermes-agent.md:426` (filas 2 y 3 del
  catalogo). El texto de Hermes se leyo del fuente real
  (`~/Downloads/hermes-agent-main/AGENTS.md`, secciones "The Footprint Ladder",
  "Don't write change-detector tests" y "Never read source code in tests") y se
  **adapta**, no se copia: los 6 peldanos de Hermes (extend > CLI+skill >
  service-gated tool > plugin > MCP > core tool) no aplican a un CLI sin modelo,
  y quedan 5 traducidos al vocabulario de este arnes.
- `harness_check.sh` ya tiene el patron exacto que necesita el bloque nuevo: el
  de lecciones (#17) se omite entero si falta `docs/lecciones/`. El de
  conventions se omite si falta `rust/tests/` (AC-12), con la misma forma.
- La leccion `criterios-de-cierre-que-se-pueden-fallar` (extendida ayer en la
  #23) es la que dicta que los AC de documentacion se verifiquen con greps
  estructurales y no con tests que grepeen markdown.

## Delegacion (implementer)

- **D1 (AC-1, AC-2, AC-3)** — `docs/conventions.md`: la escalera. Cinco peldanos
  numerados en formato `N. **<nombre>** ... (#<feature>)`, de menor a mayor
  huella, cada uno con cuando aplica y un ejemplo real de este repo. La regla de
  eleccion ("el de menor huella que resuelva el problema") y la obligacion de
  escribir `Peldano elegido:` en el plan cuando no es el mas alto.
- **D2 (AC-4, AC-5, AC-6)** — `docs/conventions.md`: las tres reglas de test.
  Cada una con su `// NO:` y su `// SI:` en Rust, con casos de este repo. La
  excepcion de **dato de entrada** (OBS-1) escrita con su criterio de corte: "el
  test seguiria valiendo si la implementacion se reescribiera entera".
- **D3 (AC-7, AC-8)** — `rust/tests/cli_basics.rs`: reescribir
  `verify_should_not_be_wired_into_any_hook` como
  `only_verify_should_execute_declared_commands`. Spec con
  `Comando: touch rastro.txt`, aprobado; se corren los comandos del arnes que
  podrian llegar a ejecutarlo (`status`, `next`, `advance`, `check-plan`,
  `check-spec`, `nudge`, `autocheck`, `close`) y se assertea que el rastro **no**
  aparece; despues se corre `verify` y se assertea que **si** aparece. El control
  positivo importa tanto como el negativo: sin el, el test pasaria aunque el
  rastro fuera imposible de crear.
- **D4 (AC-10, AC-11, AC-12)** — `harness_check.sh` (+ espejo): bloque nuevo.
  Grepea `rust/tests/` buscando lecturas de archivos fuente, reporta archivo,
  linea y nombre del test, y **no cambia el exit code**. Sin `rust/tests/`, el
  bloque no imprime nada.
- **D5 (AC-8, AC-10, AC-11, AC-12)** — `tests/conventions_check.sh` (NUEVO) con
  cuatro modos (`sin-violaciones`, `detecta`, `no-bloquea`, `sin-rust`), cada uno
  con su assert, para que cada AC tenga un comando propio que pueda fallar.
- **D6 (AC-13, AC-14, AC-15)** — Espejo exacto de `conventions.md`, los tres
  roles (lider: la escalera y `Peldano elegido:`; implementer: las tres reglas
  antes de escribir tests; reviewer: **rechaza** los que las violan) y las docs.
- **D7 (AC-9)** — La auditoria de la suite entera contra las tres reglas, escrita
  en `docs/impl-24.md` caso por caso, incluyendo los que se revisaron y quedaron
  **correctos**: un informe que solo lista violaciones no deja saber si se miro
  todo.
- **D8 (AC-17)** — Verificacion oficial completa y `verify --feature 24` verde.

## Criterios de cierre (reviewer)

Escritos para poder fallar (leccion `criterios-de-cierre-que-se-pueden-fallar`) y
verificados contra datos reales (leccion `probar-contra-datos-reales`):

- Evidencia por AC-1..AC-17 en `docs/impl-24.md`; veredicto en `docs/review-24.md`.
- `sh harness_cli verify --feature 24` **verde**, con sus 17 comandos.
- **La prueba del rojo sobre el chequeo nuevo**: introducir a mano un test que lea
  un `.rs`, correr `harness_check.sh` y confirmar que lo reporta con archivo,
  linea y nombre; despues sacarlo. Si el chequeo no se ve fallar, no verifica.
  Es literalmente el procedimiento que la leccion extendida ayer describe.
- **El aviso no cambia el exit code**: `harness_check.sh` con una violacion
  presente sigue saliendo 0 (comparado contra la corrida sin violacion).
- **El test reescrito tiene control positivo**: si se rompiera `verify`, el test
  fallaria. Se comprueba invirtiendo la assercion una vez, a mano.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash harness_check.sh`: todo verde.
- `templates/` y raiz espejados (`diff -q` limpio en conventions y harness_check).
- Hito 2 del `PRD-master` marcado por el cierre, con declaracion de leccion.

## Riesgos

- **Escribir reglas que nadie aplica.** Es el riesgo central de una feature de
  convenciones. Mitigado por: la escalera se aplica a si misma (AC-16), la regla
  de tests cobra su primera deuda en la misma feature (AC-7), y el aviso corre
  solo (AC-10).
- **Que la excepcion se vuelva agujero.** "Dato de entrada" podria estirarse
  hasta justificar cualquier grep. Mitigado por el criterio de corte escrito:
  *el test seguiria valiendo si la implementacion se reescribiera entera*. Si no
  sobrevive esa pregunta, no es dato de entrada.
- **Un aviso que se vuelve ruido.** Si el chequeo diera falsos positivos, se
  ignoraria en dos dias y con el se ignorarian los reales (leccion
  `probar-contra-datos-reales`). Por eso se corre contra la suite real y se
  verifica a mano cada linea que reporte.
- **Reglas que envejecen mal en `templates/`.** Los ejemplos citan features de
  este repo (OBS-4). Se acepta: un ejemplo ajeno ensena mas que un placeholder.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cuatro del spec fueron decididas por Alan el 2026-08-17
antes de aprobar:

- OBS-1 excepcion acotada a **dato de entrada** -> D2.
- OBS-2 el chequeo **avisa, no bloquea** -> D4.
- OBS-3 el test de la #23 se **reescribe** como contrato -> D3.
- OBS-4 la plantilla lleva el **mismo texto con ejemplos reales** -> D6.

## Skills aplicadas

- **`rust-testing`**: el corazon de la feature es una politica de tests. El test
  reescrito (D3) es la aplicacion directa: contrato de comportamiento observable
  (¿aparecio el archivo?) en vez de assercion sobre el texto del fuente, y con
  control positivo para que el verde signifique algo.
- **`rust-best-practices`**: peldano 1 de la escalera — reusar `conventions.md` y
  `harness_check.sh` en vez de crear superficie; cero dependencias nuevas.
- **`rust-patterns`**: no hay tipos nuevos en esta feature, y esa es la decision.
  La logica cabe en un bloque de shell dentro de un script que ya existe; meterla
  en el binario habria sido bajar un peldano sin necesidad.

### Avance 2026-08-17T18:20:00Z
Plan de la #24 escrito: D1-D8 citando cada AC. La feature se aplica a si misma en dos frentes: pasa por su propia escalera (peldano 1, documentado arriba con la tabla de por que los otros cuatro no hacian falta) y paga en D3 la deuda que su propia regla descubre en el test de la #23. Los AC de documentacion se verifican con greps estructurales y no con tests de Rust, porque un test que grepea un markdown es el detector-de-cambios que la feature prohibe.

### Avance 2026-08-17T18:13:39Z
Plan de la #24 escrito: D1-D8 citando cada AC. La feature se aplica a si misma: pasa por su propia escalera (peldano 1, con la tabla de por que los otros cuatro no hacian falta) y paga en D3 la deuda que su propia regla descubre en el test de la #23. Los AC de documentacion se verifican con greps estructurales, no con tests de Rust: un test que grepea un markdown es el detector-de-cambios que la feature prohibe.

### Avance 2026-08-17T18:24:47Z
Feature #24 implementada: conventions.md lleva la escalera de huella (5 peldanos con ejemplo real) y las tres reglas de test; harness_check avisa (no bloquea) cuando un test lee el fuente. La feature se aplico su propia escalera (peldano 1) y pago la deuda que su regla descubre: el test de la #23 reescrito como contrato de comportamiento. Hallazgo: la regla del detector-de-cambios condeno tambien el test de compatibilidad que la #23 celebro, reescrito como invariante.

---
Cerrado: 2026-08-17T18:25:04Z - status=done - conventions.md pasa de 7 lineas a la escalera de huella (5 peldanos con ejemplo real de este repo) y las tres reglas de test. La feature se aplico su propia escalera (peldano 1: cero comandos, flags y dependencias) y pago en el acto la deuda que su regla descubre.
