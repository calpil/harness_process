# Plan - Feature #51: revision_adversarial_y_modelos_por_rol

Estado: in_progress
Microservicios:
- harness

## Alcance

Que revisar sea riguroso y barato. Entra: (a) modelo y esfuerzo por rol en una
tabla unica de cada instalador — implementer `claude-opus-5`, leader y reviewer
`claude-fable-5`, los tres `xhigh` —; (b) el rol reviewer reescrito para refutar
en vez de confirmar, con verificacion independiente y disciplina de tokens; (c)
`harness revision --feature <id> [--max-lineas N] [--json]`, el paquete minimo
de revision que reporta su propio tamaño. No entra: correr la revision al
cerrar, doble pasada de revisores, persistir el paquete, ni tocar los modelos de
los otros backends.

## Peldano elegido

Comando nuevo (`revision`), que es un peldano abajo de "solo documentacion". La
razon: el peldano de arriba — dejar la disciplina escrita en el rol y confiar en
que el agente lea poco — es justamente lo que fallo, y el costo medido fue de 10
millones de tokens en una verificacion. Una regla que depende de la buena fe del
que la aplica no acota nada; el paquete si, porque le entrega al reviewer lo que
necesita y hace visible lo que quedo afuera.

## Impacto entre microservicios

Un solo microservicio: `harness`. El cambio de modelos toca los dos instaladores
y sus espejos; `revision` es aditivo y de solo lectura, no toca estado ni gates.

## Consulta al grafo (graphify)

No hace falta: rutas conocidas (`setup_harness.{sh,ps1}`, `roles/`, un modulo
nuevo en `rust/src/` y su comando).

## Delegacion (implementer)

- D1 [AC-1, AC-2, AC-4, AC-5]: tabla de roles en `setup_harness.sh` (modelo y
  esfuerzo por rol) y regeneracion de los espejos de este repo.
- D2 [AC-3]: paridad literal en `setup_harness.ps1`.
- D3 [AC-6, AC-7, AC-8, AC-9]: `roles/reviewer.md` (+ espejo en `templates/`):
  postura adversarial, verificacion independiente, hallazgos con caso concreto y
  el significado explicito del `approved`.
- D4 [AC-10]: reglas de tokens en el mismo rol, concretas y verificables.
- D5 [AC-11, AC-13, AC-14]: modulo `revision` + comando
  `revision --feature <id> [--json]`: AC del spec con su estado en verify,
  evidencia de impl, archivos tocados, diff y rutas protegidas; tolerante a lo
  que falte.
- D6 [AC-12, AC-12b, AC-12c]: presupuesto (`--max-lineas`), recorte declarado y
  reporte del tamaño del propio paquete.
- D7 [AC-15]: tests (presupuesto, ausencias, JSON, rutas protegidas) y asserts
  de los dos instaladores en el smoke.
- D8 [AC-16]: el cierre de esta feature se revisa con `revision --feature 51`.
- D9: `roles/README.md` (+ espejo) documenta donde se cambia modelo y esfuerzo.

## Criterios de cierre (reviewer)

- Evidencia por AC-n en `docs/impl-51.md`.
- `cargo test`, `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh` limpios.
- Reinstalar en este repo NO produce diff en `.claude/agents/*.md` (AC-4).
- El veredicto usa el paquete y declara que intento refutar cada AC (AC-16).

## Riesgos

- R1: el paquete se vuelve otro monstruo de tokens. Mitigacion: presupuesto por
  default, recorte declarado y el propio reporte de tamaño (AC-12b/AC-12c).
- R2: "adversarial" degenera en teatro (decir que se intento refutar sin
  hacerlo). Mitigacion: AC-8 exige el caso concreto y AC-9 obliga a nombrar lo
  que NO se probo.
- R3: cambiar el modelo del implementer a Opus encarece cada implementacion.
  Mitigacion: es decision explicita del usuario (OBS-1); el ahorro esta del lado
  de la revision.
- R4: el diff de una feature grande no entra en ningun presupuesto razonable.
  Mitigacion: se recorta por archivo y se dice que quedo afuera, para que el
  reviewer pida lo que falte a mano.

## Observaciones (decisiones pendientes)

- OBS-1 a OBS-4 [DECIDIDAS 2026-08-22]: modelos y esfuerzo por rol; adversarial
  con verificadores independientes; tokens por los dos lados (rol + paquete);
  todo en una sola feature.
- OBS-5 [DATO]: el disparador fue una verificacion de 10 millones de tokens.

---
Cerrado: 2026-08-22T13:07:28Z - status=done - Modelos por rol (implementer opus-5, lider y reviewer fable-5, xhigh) en la tabla de los dos instaladores, reviewer adversarial con disciplina de tokens, y el comando revision que arma el paquete acotado: 478 lineas / ~6654 tokens contra los 10M que motivaron la feature
