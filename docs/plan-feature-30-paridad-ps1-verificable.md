# Plan - Feature #30: paridad_ps1_verificable

Estado: in_progress
Microservicios:
- harness

## Alcance

La deuda mas vieja del repo: once features seguidas declararon el mismo limite
("esta maquina no tiene pwsh") y la promesa de paridad entre `setup_harness.sh` y
`setup_harness.ps1` nunca se verifico.

Se cierra con un chequeo **estructural** que corre sin PowerShell: compara
opciones declaradas y superficies escritas, acepta las asimetrias que esten
**declaradas con su razon**, y falla ante las que no. Y dice lo que no cubre: no
ejecuta el instalador de Windows.

Spec aprobado (11 AC, cada uno con su `Comando:`):
`docs/spec-feature-30-paridad-ps1-verificable.md`.

## Peldano elegido: 1 (extender lo que ya existe)

| Peldano | ¿Alcanzaba? |
| --- | --- |
| **1. extender lo que existe** | **SI, elegido.** El chequeo es un test hermano de `tests/setup_smoke.sh` y `tests/conventions_check.sh`, y el aviso entra como un bloque mas en `harness_check.sh`, que ya tiene tres bloques opcionales con esa forma |
| 2. flag en un comando existente | innecesario: no hay nada que parametrizar |
| 3. comando nuevo | seria superficie permanente para comparar dos archivos |
| 4. superficie nueva | no |
| 5. dependencia nueva | no. Instalar pwsh seria peldano 5 y Alan lo descarto |

**Peldano elegido: 1 (extender lo que ya existe) porque el chequeo cabe como test
al lado de los otros dos y el aviso como un bloque mas del script que ya los
tiene; no hace falta comando, flag ni dependencia.**

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder, como en las diez features anteriores.

- `tests/parity_check.sh` (NUEVO): los seis modos.
- `harness_check.sh` (+ espejo): el bloque de aviso.
- `docs/verification.md` (+ espejo) y `README.md`: la promesa acotada.

**Riesgo especifico**: un chequeo de paridad demasiado literal reporta
diferencias que son correctas y se vuelve ruido — el mismo riesgo que la #25 con
el doctor. Por eso las asimetrias legitimas se declaran con su razon en vez de
silenciarse con una excepcion anonima.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "setup_smoke ps1 paridad"`. Lo que decidio el plan:

- La deuda esta declarada, con esas palabras, en `impl-17` a `impl-26` y
  levantada en `review-23`, `review-24`, `review-25` y `review-26`. Nunca se
  decidio nada: se acepto once veces.
- Las cinco asimetrias de hoy se midieron **antes** de escribir el spec, no se
  supusieron: cuatro opciones solo en el `.sh` y una solo en el `.ps1`.
- La forma del bloque nuevo en `harness_check.sh` la dan los tres que ya existen
  (lecciones #17, conventions #24, rutas #26): se omite entero si falta su
  insumo, y este ademas **no** toca `failures` (OBS-1).

## Delegacion (implementer)

- **D1 (AC-1, AC-2, AC-3)** — `tests/parity_check.sh`, modo `opciones`: extrae
  las opciones del `case` del `.sh` y del bloque `param()` del `.ps1`, traduce
  `--kebab-case` a `-PascalCase`, y compara. La lista de asimetrias declaradas
  vive **en el propio script**, cada una con su razon en una linea.
- **D2 (AC-4, AC-5)** — Modos `superficies` y `smokes`: que los dos instaladores
  escriban las mismas superficies y que los dos smokes cubran los mismos bloques.
- **D3 (AC-2)** — Modo `detecta-opcion`: siembra una opcion en un solo lado (en
  copias temporales, nunca en los archivos reales) y exige que el chequeo la
  reporte nombrando en cual falta. Es la prueba del rojo.
- **D4 (AC-8, AC-9)** — El bloque en `harness_check.sh`: avisa con `[i]`, **no**
  toca `failures`, y se omite entero sin `setup_harness.ps1`.
- **D5 (AC-6, AC-7)** — La promesa acotada: `docs/verification.md` (+ espejo) y
  el README dicen que el chequeo **no ejecuta el instalador de Windows**, y la
  instruccion del smoke `.ps1` queda condicionada a tener la maquina.
- **D6 (AC-10, AC-11)** — `Peldano elegido:` (arriba) y la verificacion oficial.

## Criterios de cierre (reviewer)

Escritos para poder fallar y verificados contra datos reales:

- Evidencia por AC-1..AC-11 en `docs/impl-30.md`; veredicto en `docs/review-30.md`.
- `sh harness_cli verify --feature 30` **verde**, con sus 11 comandos.
- **La prueba del rojo**: sembrar una opcion en un solo instalador (en copias) y
  confirmar que el chequeo la nombra y dice en cual falta.
- **Las cinco asimetrias reales estan declaradas con su razon**, y se verifica a
  mano que cada razon sea cierta: que `--with-subagents` sea de verdad la
  afirmativa de un default encendido, y que `-CargoTargetDir` de verdad no tenga
  sentido en Unix.
- **Cero falsos positivos**: el chequeo pasa sobre los dos instaladores reales
  tal como estan hoy.
- **No necesita PowerShell**: se corre en esta maquina, que no lo tiene.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash harness_check.sh`: todo verde.
- Hito del `PRD-master` marcado por el cierre, con declaracion de leccion.

## Riesgos

- **Ruido por literalidad.** Mitigado por las asimetrias declaradas con razon y
  por correrlo contra los archivos reales antes de cerrar.
- **Falsa sensacion de cobertura.** Un `.ps1` estructuralmente paritario puede
  fallar igual al ejecutarse. Es el riesgo central y no se puede mitigar sin
  pwsh: se **declara** (AC-6), que es lo que la #25 enseño sobre los OK que dicen
  de mas.
- **El parseo por grep se rompe si cambia el formato** del `case` o del
  `param()`. Es aceptable: si el formato cambia, el chequeo falla ruidosamente en
  vez de callar, y eso se ve en el acto.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las dos del spec fueron decididas por Alan el 2026-08-18:

- OBS-1 el chequeo **avisa** en `harness_check.sh`, no bloquea -> D4.
- OBS-2 la instruccion del smoke `.ps1` queda **condicionada** a tener Windows,
  nombrando el chequeo de paridad como sustituto -> D5.

## Skills aplicadas

- **`rust-testing`**: la prueba del rojo es un modo propio (`detecta-opcion`),
  porque un chequeo que nunca se vio fallar no verifica nada.
- **`rust-best-practices`**: peldano 1, cero superficie nueva, cero
  dependencias. Instalar pwsh habria sido peldano 5 para el mismo problema.
- **`rust-patterns`**: no hay codigo Rust en esta feature, y esa es la decision.
  Meter el comparador en el binario habria bajado un peldano sin necesidad.

### Avance 2026-08-18
Plan de la #30 escrito: D1-D6 citando cada AC. Cierra la deuda de once features sin instalar PowerShell, comparando estructura en vez de ejecutar. Las cinco asimetrias reales se midieron antes de escribir el spec.

### Avance 2026-08-18T02:14:33Z
Feature #30 implementada: tests/parity_check.sh compara lo que los dos instaladores DECLARAN (opciones traduciendo kebab a Pascal, superficies, temas de los smokes) sin necesitar PowerShell, con las cinco asimetrias reales declaradas cada una con su razon. Dos razones salieron mal en el primer intento y las encontro la verificacion a mano: --with-postgres es un no-op historico y no la afirmativa de un default, y -CargoTargetDir no tiene que ver con el PATH de rustup.

---
Cerrado: 2026-08-18T02:26:50Z - status=done - Paridad de instaladores verificable sin PowerShell: tests/parity_check.sh compara lo que los dos DECLARAN y falla cuando uno se adelanta. Cierra por trabajo la deuda de once features. El limite (no ejecuta el instalador de Windows) esta escrito en README y verification.md.
