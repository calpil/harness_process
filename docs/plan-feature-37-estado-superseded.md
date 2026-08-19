# Plan - Feature #37: estado_superseded

Estado: in_progress
Microservicios:
- harness

## Alcance

Darle al arnes la palabra que le falta: **"esto ya se hizo, pero en otro lado"**.
Hoy esas seis entradas dicen `blocked`, que es falso, y ademas inflan el
denominador de `prd tree`.

Spec aprobado (15 AC, cada uno con su `Comando:`):
`docs/spec-feature-37-estado-superseded.md`.

## Peldano elegido: 1 (extender lo que ya existe)

| Peldano | ¿Alcanzaba? |
| --- | --- |
| **1. extender lo que existe** | **SI, elegido.** `--status` ya existe y ya valida contra una lista (`cli.rs:38`); agregar un valor y un flag acompanante es extender, no crear. Los consumidores del campo (`status`, `next`, `journey`, `prd tree`) tambien existen |
| 2. flag en un comando existente | es lo que se hace: `--absorbida-por` acompana a `--status superseded`. Cae dentro del peldano 1 porque el comando y su validacion ya estan |
| 3. comando nuevo | seria absurdo: no hay verbo nuevo |
| 4-5. superficie / dependencia | no |

**Peldano elegido: 1 (extender lo que ya existe) porque `--status` ya valida
contra una lista cerrada y los cuatro consumidores del campo ya estan escritos;
no hace falta comando, superficie ni dependencia.**

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> el hub
responde con `statement timeout`, como en las catorce features anteriores.

Medido antes de escribir el spec: **14 lugares** comparan contra el campo
`status`. Los que importan:

- `cli.rs:38` — la lista cerrada de valores validos.
- `commands/next.rs:10` — solo ofrece `pending`. **Ya es correcto**: superseded no
  se va a ofrecer sin tocar nada.
- `commands/close.rs:89,174` — los gates solo aplican a `done`. **Ya es
  correcto**: por eso mismo hubo que usar `blocked` en su momento.
- `journey.rs:260` — solo mira `done`. **Ya es correcto**.
- `prd.rs:686` — cuenta `done` sobre el total. **Aca SI hay que tocar** (AC-8).
- `commands/status.rs:47` — imprime el status crudo. Hay que enriquecerlo (AC-7).

**El riesgo es de regresion, no de diseno**: agregar un valor a un enum de facto
puede romper a un consumidor que asume que son cuatro. Por eso el mapeo se hizo
ANTES y el AC-11 exige que las `blocked` de verdad no cambien.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "blocked absorbida por la feature"`. Lo que decidio el
plan:

- Las seis entradas se cerraron como `blocked` el 2026-08-18 con la nota
  "Absorbida por la feature #36": la informacion ya existe, pero **en prosa
  dentro de `note`**. Esta feature la convierte en un campo.
- Tres de los cuatro consumidores del status **ya tratan bien** un valor
  desconocido, porque comparan por igualdad contra `done`/`pending` en vez de
  hacer un `match` exhaustivo. Eso hace que el cambio sea chico y de bajo riesgo,
  y conviene decirlo en vez de descubrirlo.
- El unico que hay que cambiar es `prd::feature_counts`, y el cambio es de una
  linea: excluir superseded del total.

## Delegacion (implementer)

- **D1 (AC-1, AC-2, AC-3, AC-4)** — `cli.rs`: `superseded` en la lista de valores
  y `--absorbida-por <id>`; `close.rs`: exigirlo, validar que la feature exista y
  escribir `superseded_by`.
- **D2 (AC-5, AC-6)** — Tests de que los cuatro gates de `done` NO se disparan y
  de que `next` no la ofrece. Los dos pasan sin tocar codigo: son **tests de
  regresion** que fijan lo que ya es cierto.
- **D3 (AC-7)** — `status.rs`: mostrar `[superseded por #N]`.
- **D4 (AC-8)** — `prd.rs`: `feature_counts` ignora las superseded en los dos
  lados del quebrado.
- **D5 (AC-9)** — `journey.rs`: test de que no aparece como cierre sin leccion.
- **D6 (AC-10, AC-11)** — Migrar las seis reales y `tests/superseded_check.sh`;
  test de que una `blocked` de verdad no se toca.
- **D7 (AC-12, AC-13)** — Docs, espejos y el rol del reviewer.
- **D8 (AC-14, AC-15)** — `Peldano elegido:` y verificacion oficial.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-15 en `docs/impl-37.md`; veredicto en `docs/review-37.md`.
- `sh harness_cli verify --feature 37` **verde**, con sus 15 comandos.
- **Las seis migradas de verdad**, y `prd tree` sobre este repo pasa de mostrar
  `19/21` a `19/19` en el maestro. Si el numero no cambia, la feature no hizo
  nada.
- **Ninguna regresion**: la suite entera verde, y en particular los tests que ya
  existen sobre `close`, `next` y `prd tree`.
- **Una `blocked` de verdad sigue igual**: se comprueba con una feature sembrada.
- `cargo test`, clippy, `setup_smoke.sh`, `parity_check.sh`, `harness_check.sh`.

## Riesgos

- **Regresion en un consumidor del status.** Es el riesgo central. Mitigado por
  el mapeo previo de los 14 lugares y por tests que fijan el comportamiento de
  los que NO cambian.
- **Que `superseded` se use para tapar trabajo sin hacer.** Mitigado porque
  exige nombrar la feature que absorbio y esa referencia se valida: no se puede
  citar una feature inexistente.
- **Perder de vista las absorbidas** al sacarlas del conteo. Mitigado porque
  `status` las sigue listando, ahora con quien las absorbio.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las dos del spec fueron decididas por Alan el 2026-08-18:

- OBS-1 `prd tree` **ignora** las superseded en los dos lados -> D4.
- OBS-2 el flag es **`--absorbida-por`**; el campo, `superseded_by` -> D1.

## Skills aplicadas

- **`rust-patterns`**: el status sigue siendo un `&str` y no se convierte en enum
  en esta feature. Convertirlo tocaria los 14 consumidores por un beneficio que
  esta feature no necesita; queda anotado, no hecho.
- **`rust-best-practices`**: tres de los cuatro consumidores ya tratan bien un
  valor nuevo; el cambio real es de una linea en `prd.rs`.
- **`rust-testing`**: los AC-5, AC-6 y AC-9 son **tests de regresion** que fijan
  lo que ya es cierto. Sin ellos, "no rompe nada" seria una afirmacion; con
  ellos, es un contrato.

### Avance 2026-08-18
Plan de la #37 escrito: D1-D8 citando cada AC. Se mapearon los 14 lugares que comparan contra `status` ANTES de disenar: tres de los cuatro consumidores ya tratan bien un valor nuevo porque comparan por igualdad en vez de hacer match exhaustivo, asi que el cambio real es de una linea en `prd::feature_counts`.

### Avance 2026-08-18T23:01:28Z
Feature #37 implementada: estado superseded con --absorbida-por validado, campo superseded_by, y trato propio en status, next, prd tree y journey. Migradas las seis que absorbio la #36: el denominador del PRD maestro bajo de 36 a 30, que es la verdad. Tres de los cuatro consumidores del status ya trataban bien un valor nuevo, asi que el cambio real fue de una linea en prd::feature_counts mas los tests de regresion que lo fijan.

### Avance 2026-08-18T23:59:04Z
Feature #37: dos defectos encontrados por refutacion adversarial DESPUES de tener los 15 AC verdes. (1) superseded caia en el brazo _ de emit::on_close y emitia transicion a pending, o sea movia la historia de Jira de vuelta a To Do; dano cero aca porque no hay binding, pero seis tickets en cualquier instalacion con Jira. (2) La migracion puso en rojo tests/deudas_check.sh, que es el Comando: del AC-13 de la #36 y estaba verde en verify-36.md. Los dos arreglados y fijados con tests. La causa comun: el mapeo previo de los 14 consumidores fue en Rust y por igualdad, y se salteo el unico match exhaustivo del repo y un test de shell.

---
Cerrado: 2026-08-18T23:59:29Z - status=done - Estado superseded para lo que se hizo en otra feature, con --absorbida-por validado y trato propio en status, next, prd tree y journey. Las seis migradas: el denominador del PRD maestro bajo de 36 a 30. Dos defectos encontrados por refutacion despues de los AC verdes: la transicion espuria de Jira y un AC cerrado de la #36 que la migracion puso en rojo.
