# Review - Feature #13: nested_prds

Spec: docs/spec-feature-13-nested-prds.md (approved 2026-08-12T11:17:50Z,
enmendado el mismo dia con AC-17 por decision del usuario)
Plan: docs/plan-feature-13-nested-prds.md
Implementacion: docs/impl-13.md

**Veredicto: APROBADO para cierre.** Los 17 AC tienen evidencia verificable, la
verificacion oficial esta en verde y la constitution se cumple.

## Verificacion re-ejecutada en esta revision

| Comando | Resultado |
| --- | --- |
| `bash harness_check.sh` | `[Ok] Harness Check limpio.` — plan fresco, spec `approved` fresco, gate de espejo de roles sin novedad, gate nuevo del arbol sin fallos |
| `cargo test --locked` | 64 passed (unit) + 27 passed (cli_basics), 0 failed |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | sin hallazgos |
| `bash tests/setup_smoke.sh` | exit 0, con `[Ok] PRDs anidados: arbol en carpetas, enlace en el padre, cadena --prd -> spec, vuelta del cierre y gate del arbol.` |
| `diff -q harness_check.sh templates/harness_check.sh` | identicos (Articulo 6) |
| `diff -r docs/prd templates/docs/prd` | identicos |
| `git diff --stat -- roles/ templates/roles/ .claude/` | sin salida: roles intactos, el gate de espejo no puede quedar stale |

## Cobertura por AC

| AC | Estado | Como se verifico |
| --- | --- | --- |
| AC-1 | **cubierto y re-ejecutado** | Binario release + `prd add` en fixture: `docs/prd/cobranza/PRD-cobranza.md` y `docs/prd/cobranza/mora/PRD-cobranza-mora.md`. Unit `paths_should_derive_from_the_segment_chain`; smoke `PRD_E2E` |
| AC-2 | cubierto | `prd add` sobre un PRD existente sale != 0 y no duplica la fila (smoke); `resolve` lista los disponibles; `normalize_segment` rechaza vacio/sin alfanumericos y disuelve `../../etc` -> `etc` |
| AC-3 | cubierto | Unit verifica el ORDEN de las 12 secciones, no solo su presencia; smoke re-verifica `Padre:`, seccion 2 y seccion 10 en el archivo generado |
| AC-4 | cubierto | Unit cubre crear-seccion / agregar-fila / no-duplicar / ruta relativa al padre, y que el cuerpo original queda intacto; smoke cuenta la fila una sola vez tras repetir el comando |
| AC-5 | **cubierto y re-ejecutado** | `--prd mora` (cola unica) guarda `cobranza/mora` en `feature_list.json`; `--prd noexiste` sale 1 listando candidatos; ambiguedad cubierta por unit |
| AC-6 | **cubierto y re-ejecutado** | Spec generado con `PRD: docs/prd/cobranza/mora/PRD-cobranza-mora.md` en la linea siguiente a `Plan:`, `Estado: draft` intacto en la linea 3; `spec_template_sections_should_keep_the_prd_order` sigue verde |
| AC-7 | **cubierto y re-ejecutado** | `prd tree` dibuja los dos niveles con `1 hito \| features: 1/1 done`; `--prd cobranza` dibuja el subarbol (verificado a mano con tres hijos); units cubren el conteo y el `[!]` de `Padre:` incoherente |
| AC-8 | **cubierto y re-ejecutado** | Arbol roto a proposito: 5 `[!]` (fuera de lugar, dos carpetas sin PRD, `Padre:` que miente, feature con `prd` inexistente) y exit 2; `HARNESS_CHECK_MODE=warn` -> exit 0; PRD sin hitos solo `[i]` |
| AC-9 | cubierto | Sin `docs/prd/`: check exit 0 y `prd tree` informa "No hay PRDs todavia" (smoke) |
| AC-10 | cubierto | Guia con `prd add`, `prd tree`, `--prd`, layout en carpetas y la aclaracion de que se actualiza solo y que no; asserts en ambos smoke |
| AC-11 | cubierto | `PRD-master.md` con `## PRDs anidados`, `## Bitacora` y `--prd <ruta>` en la tabla de hitos; sentinels ya existentes prueban que sigue siendo del USUARIO (reinstall y `--reset` no lo tocan) |
| AC-12 | cubierto | `setup_harness.sh` (lista "Archivos principales") y `setup_harness.ps1` (parrafo en ingles) + `AGENTS.md`, `README.md`, `UPDATING.md` (raiz y template), `docs/architecture.md`. El smoke verifica los comandos en el `AGENTS.md` **instalado**. `write_basic_agent_surface` y `.grok/GROK.md` intactos |
| AC-13 | cubierto | `diff -q` limpio entre `harness_check.sh` y su template |
| AC-14 | cubierto | 13 tests nuevos en `prd::tests`; suite completa verde |
| AC-15 | cubierto (ps1 estatico) | `tests/setup_smoke.sh` exit 0 con el bloque nuevo; `tests/setup_smoke.ps1` recibio los asserts equivalentes y se reviso estaticamente por falta de `pwsh`, como en #1 y #4-#12 |
| AC-16 | cubierto | Tabla de arriba |
| AC-17 | **cubierto y re-ejecutado** | `close --status done` marca `done (2026-08-12)` en la fila del hito y deja bitacora con spec e impl; re-cierre no duplica ni pisa la fecha; feature sin `--prd` escribe en el maestro; `--status blocked` no toca ningun PRD; PRD ausente -> `[i]` y el cierre sigue |

## Constitution

- **Art. 1 (calidad y tests primero):** cumplido — cobertura en las dos capas
  (13 unit tests sobre el modulo nuevo, E2E de instalador sobre fixture real).
  Los casos negativos estan cubiertos tanto como los felices.
- **Art. 2 (una feature a la vez):** cumplido — backlog con una sola
  `in_progress`.
- **Art. 3 (trazabilidad):** cumplido — cada item de la Delegacion cita su AC-n y
  `docs/impl-13.md` da evidencia AC por AC.
- **Art. 6 (espejos):** cumplido — `harness_check.sh` == template, `docs/prd/*`
  == `templates/docs/prd/*`, roles sin tocar.
- **Protocolo de decisiones:** cumplido y ejercitado dos veces — las cuatro
  decisiones de diseno se preguntaron ANTES de escribir el spec, y la pregunta
  del usuario sobre la actualizacion de los PRDs se convirtio en una enmienda
  explicita (AC-17) registrada en Observaciones, no en un cambio silencioso.

## Observaciones para el futuro (no bloquean el cierre)

1. **Dogfooding en el checkout fuente:** en este repo `docs/prd/PRD-master.md`
   ES la plantilla que se distribuye, asi que la vuelta del cierre (AC-17)
   escribe una linea de bitacora sobre el propio template. Al cerrar la feature
   #13 esa linea se revierte a mano para no shippear la bitacora del arnes
   dentro de la plantilla del usuario. Es la misma clase de rareza que el
   footgun de auto-instalarse en el checkout fuente (feature #7): solo ocurre
   aca, nunca en una instalacion real.
2. **`prd move` / `prd rm` no existen** (fuera de alcance declarado): reacomodar
   el arbol es manual y el gate avisa si quedo incoherente. Candidato natural a
   la proxima feature del area.
3. **Gate de divergencia PRD vs implementacion** (avisar si un hito cerro y el
   PRD no se edito desde entonces) quedo explicitamente descartado en la
   enmienda; sigue siendo la evolucion logica de AC-17.
4. **Idempotencia de `link_child`:** la deteccion de fila duplicada mira todas
   las filas del documento, no solo las de la seccion `## PRDs anidados`. Sin
   riesgo practico (las filas de hitos empiezan con un numero), pero es un
   detalle a acotar si algun dia el formato del PRD cambia.
