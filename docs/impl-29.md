# Evidencia de implementacion - Feature #29: prd_y_sdd_siempre_al_dia

Spec: `docs/spec-feature-29-prd-y-sdd-siempre-al-dia.md` (`Estado: approved`, 23 AC)
Plan: `docs/plan-feature-29-prd-y-sdd-siempre-al-dia.md` (D1-D8, `Peldano elegido: 3`)
PRD: `docs/prd/PRD-master.md` (hito 6, agregado por esta misma feature)

## La feature se aplico a si misma, y por eso sirve

El criterio de cierre decia: *si esto no mejora `architecture.md` —que no
mencionaba `doctor.rs` ni `rutas.rs`— la feature no sirve*. Se corrio de verdad:

```
$ sh harness_cli prd propose --feature 29
Propuesta docs/prd-diff-29.md: 3 bloque(s) sembrado(s) de 3 documento(s).
Quedan 3 sin contestar.
```

Los tres bloques se contestaron con `cambio`, se le mostraron a Alan, dijo que
si, y `prd apply --yes` escribio:

| Documento | Que se arreglo |
| --- | --- |
| `docs/prd/PRD-master.md` | hito 5 marcado `done` y **hito 6 agregado** para esta feature |
| `docs/prd/SDD-master.md` | `# SDD Master - <nombre del proyecto>` -> `Harness Process`; `Estado: draft` -> `en uso`; fecha real. **Dejo de publicarse a Confluence con los placeholders puestos** |
| `docs/architecture.md` | suma `doctor.rs` (#25), `rutas.rs` (#26) y `documentos.rs` (#29) |

El drift que la feature existe para evitar estaba ocurriendo en este repo, hoy, y
quedo corregido por el propio mecanismo.

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/documentos.rs` | D1, D3, D6 | NUEVO. `alcance()`, `parsear()`, `planificar()`, `gate()`: todo **puro**; 15 tests |
| `rust/src/commands/prd.rs` | D2, D4, D5 | `propose` y `apply` |
| `rust/src/cli.rs` | D2 | `PrdCommand::{Propose, Apply}` |
| `rust/src/commands/close.rs` | D6 | El cuarto gate |
| `rust/tests/cli_basics.rs` | D7 | 12 tests de integracion |
| `CHECKPOINTS.md`, 3 roles, README, UPDATING (+ espejos) | D8 | El deber, que antes no estaba escrito en ningun lado |

## Evidencia por AC

`sh harness_cli verify --feature 29` corre los 23 comandos.

| AC | Evidencia |
| --- | --- |
| AC-1..AC-3 | `documentos_alcance_*` (cadena de PRDs + SDD + architecture; anidados sin repetir; ausentes omitidos) |
| AC-4..AC-6 | `prd_propose_should_*` (un bloque por documento, no pisa lo contestado, precomputa senales) |
| AC-7 | `prd_apply_should_reject_a_tampered_block_list` (unitario y de integracion) |
| AC-8 | `prd_apply_should_replace_the_literal_anchor_not_the_section` (verifica que el `### Sub` sobreviva) |
| AC-9 | `prd_apply_should_refuse_a_citation_that_does_not_hold` (rango fuera del archivo, archivo inexistente, rango vacio) |
| AC-10, AC-11 | `no-aplica` con razon; bloque sin resolver nombrado |
| AC-12 | `prd_apply_without_yes_should_show_and_refuse_to_write` (compara el archivo byte a byte) |
| AC-13 | `prd_apply_with_yes_should_write_seal_and_log` |
| AC-14 | `prd_apply_should_be_idempotent_by_content` + `idempotence_should_hold_when_despues_contains_antes` |
| AC-15, AC-16 | la propuesta fuera de `docs/prd/**`; `prd apply` registra sus escrituras |
| AC-17 | `close_should_demand_the_docs_proposal_when_the_rule_is_on` + `close_should_demand_the_user_seal_not_just_the_answers` |
| AC-18 | `docs_gate_should_not_depend_on_verify_report_freshness` |
| AC-19 | `no_spec_command_should_invoke_prd_apply_yes`, sobre los specs REALES |
| AC-20..AC-23 | CHECKPOINTS + roles + docs; 298 + 144 tests; clippy 0 |

## El bug que solo aparecio usandolo

Los 15 tests unitarios pasaban. La primera corrida real sobre este repo tambien.
**La segunda no**: `prd apply --yes` volvio a escribir y **duplico** el bloque de
modulos en `docs/architecture.md`.

La causa:

```rust
// mal
if !texto.contains(antes) && texto.contains(despues) { /* ya aplicado */ }
```

El patron mas comun de estos cambios es **"insertar antes de esta linea"**, donde
el `Despues:` **contiene** al `Antes:`. Despues de aplicar, el `antes` sigue
presente —porque quedo adentro del `despues`— asi que el bloque no se reconocia
como aplicado y se reaplicaba.

```rust
// bien
if texto.contains(despues) { /* ya aplicado */ }
```

El test unitario que escribi (`prd_apply_should_be_idempotent_by_content`) usaba
un caso donde el `antes` NO estaba contenido en el `despues`, asi que pasaba. El
caso que rompe quedo encodeado en
`idempotence_should_hold_when_despues_contains_antes`, con el porque adentro.

Es la leccion `probar-contra-datos-reales` otra vez, y en su forma mas pura: la
suite verde no cubria la forma que el uso real produce en el primer intento.

## Tres bloqueos que murieron antes de llegar al codigo

El diseno salio de un workflow de 18 agentes (6 mapearon el codigo con 73
hallazgos citando `archivo:linea`, 3 disenaron en paralelo desde lentes
distintas, 9 refutaron contra el codigo). Tres problemas se mataron ahi:

1. **Deadlock de frescura** (AC-18). Un diseno exigia
   `mtime(propuesta) >= mtime(verify-<id>.md)`. Pero `verify` reescribe su
   reporte en cada corrida y `prd apply` es idempotente: cualquier `verify`
   posterior al `--yes` dejaba la propuesta vieja **para siempre**, sin ningun
   comando capaz de refrescarla. Hay un test que fija que el gate no mire ese
   archivo.
2. **La auto-aplicacion via `verify`** (AC-19). Los AC de este repo declaran
   `Comando:` y `verificacion::ejecutar` los lanza con `sh -c`
   (`verificacion.rs:163`). Un AC que dijera `Comando: prd apply --yes` haria que
   **correr `verify` aplicara la propuesta sin el si del usuario**. El test
   recorre los specs reales y lo prohibe.
3. **El slicing por `## `** (AC-8). El diseno original reusaba la tecnica de
   `prd::echo_close`, que corta con `starts_with("## ")` (`prd.rs:629`).
   `docs/architecture.md` tiene tres `###` que ese predicado se tragaria enteros.
   Por eso el anclaje es por **texto literal**.

Las cuatro afirmaciones de codigo sobre las que se apoya el diseno se
verificaron **a mano** antes de escribir el spec: no se tomaron de los agentes.

## Lo que hace que esto no sea ceremonia

- **El alcance lo calcula el binario**, no el agente. Si lo eligiera el agente,
  "el SDD ya lo refleja" no tendria contraparte.
- **La lista de bloques es cerrada**: el agente no puede agregar, quitar ni
  renombrar. Sin eso podria colapsar cuatro preguntas en una respuesta.
- **`ya-esta` trae una cita que el binario abre y verifica.** Es la unica de las
  tres respuestas refutable por maquina, y es justo la mentira mas probable.
- **`no-aplica` exige razon.** Sigue siendo una afirmacion del agente: por eso el
  rol del reviewer dice que un `no-aplica` en una feature que si cambio el
  producto es `changes_requested`.

## Limites declarados

- **`no-aplica` no se puede verificar por maquina.** Es la puerta por la que se
  escapa una feature perezosa, y esta dicho en el rol del reviewer en vez de
  fingir que el gate lo cubre.
- **El gate pide algo al USUARIO en cada cierre.** Es el riesgo numero uno: si
  molesta, la regla se apaga y la feature muere. Mitigado por el formato (una
  tabla de tres o cuatro renglones) y porque `no-aplica`/`ya-esta` son baratos.
- **El SDD se toco solo en el encabezado.** Llenar sus nueve secciones es del
  usuario: el arnes propone, no escribe el diseno por el.
- **`tests/setup_smoke.ps1` sigue sin correrse**, pero desde la #30 la paridad se
  verifica estructuralmente con `tests/parity_check.sh`.

## Para el backlog

- **`prd propose` no propone el texto**, solo la pregunta. Un paso siguiente
  seria que precompute un `Despues:` candidato a partir del diff de la feature.
- **Las senales `Presente en:` buscan el nombre de la feature**, que es una
  heuristica pobre: una feature puede estar documentada con otras palabras.
- **El sello no se invalida** si alguien edita el documento despues de aplicar.
  Detectarlo exigiria una firma, que es justamente lo que el AC-14 descarta para
  los PRD compartidos.
