# Plan - Feature #13: nested_prds

Estado: in_progress
Microservicios:
- harness

## Alcance

Hacer real la promesa de PRDs anidados que hoy solo esta escrita en
`docs/prd/COMO-ESCRIBIR-UN-PRD.md`: arbol de carpetas bajo `docs/prd/`, comando
que crea el hijo y lo enlaza en el padre, cadena PRD -> feature -> spec, vista
del arbol y gate de integridad. Spec aprobado:
`docs/spec-feature-13-nested-prds.md` (AC-1 a AC-17).

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

Microservicio unico (`harness`): el arnes es el producto. El hub PostgreSQL no
esta accesible en esta sesion (timeout de conexion, igual que en #10 a #12), asi
que el impacto se calcula por lectura directa del repo:

- `rust/` (binario `harness`): modulo nuevo + dos comandos + dos comandos
  existentes tocados (`add`, `start`/`spec`).
- `harness_check.sh` y su espejo `templates/harness_check.sh` (Articulo 6:
  identicos).
- `docs/prd/*` y `templates/docs/prd/*` (planillas del USUARIO vs guia
  refrescable: regimenes distintos, ver Riesgos).
- Superficies raiz generadas por `setup_harness.sh` / `setup_harness.ps1`
  (paridad sh/ps1).
- `tests/setup_smoke.sh` / `.ps1`.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`graphify-out/` de este repo esta al dia de la feature #12; la superficie tocada
por esta feature (`rust/src/spec.rs`, `commands/add.rs`, `harness_check.sh`,
`docs/prd/`) se leyo directo en la fase de lider.

## Delegacion (implementer)

- D1 [AC-1, AC-2, AC-3]: `rust/src/prd.rs` (modulo nuevo) — identidad de un PRD
  por cadena de segmentos, derivacion carpeta/archivo, escaneo del arbol bajo
  `docs/prd/`, plantilla del PRD hijo con las 12 secciones del metodo, y errores
  de uso (padre inexistente con lista, destino existente, slug vacio).
- D2 [AC-4]: enlace en el padre — seccion `## PRDs anidados`: crearla al final si
  falta, agregar fila si existe, nunca duplicar; sin tocar otra linea.
- D3 [AC-1, AC-7, AC-9]: `rust/src/commands/prd.rs` + `cli.rs` — subcomando
  `prd` con `add` (`--name`, `--parent`) y `tree` (`--prd`), incluyendo el caso
  "sin `docs/prd/`".
- D4 [AC-5]: `add --prd <ref>` — resolucion exacta / por segmento unico /
  ambigua / inexistente, y campo `prd` opcional en `feature_list.json`.
- D5 [AC-6]: `rust/src/spec.rs` — linea `PRD:` en el encabezado del spec
  generado, derivada del campo `prd` (o el maestro por defecto), sin mover
  `Estado: draft` de la linea 3.
- D6 [AC-8, AC-9, AC-13]: gate del arbol en `harness_check.sh` + espejo
  `templates/harness_check.sh`.
- D7 [AC-10, AC-11]: `docs/prd/COMO-ESCRIBIR-UN-PRD.md` y `docs/prd/PRD-master.md`
  (+ copias en `templates/`) — comandos reales, layout en carpetas, seccion
  `## PRDs anidados` y `--prd` en la tabla de hitos.
- D8 [AC-12]: superficies y docs del repo — `write_agent_surface` en
  `setup_harness.sh` y su par ps1, `README.md`, `AGENTS.md`, `UPDATING.md` (raiz
  y `templates/`), `docs/architecture.md`.
- D12 [AC-17]: vuelta del cierre al PRD — `close --status done` marca la fila
  del hito (Estado -> `done (fecha)`) y agrega la linea de `## Bitacora` con
  spec e impl; idempotente, sin tocar el cuerpo, sin fallar si falta el PRD o la
  fila.
- D9 [AC-14]: tests unitarios en Rust (rutas, plantilla, enlace, resolucion,
  encabezado del spec, render del arbol, vuelta del cierre).
- D10 [AC-15]: smoke — `tests/setup_smoke.sh` y `tests/setup_smoke.ps1`.
- D11 [AC-16]: verificacion oficial completa + `docs/impl-13.md`.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-17 en `docs/impl-13.md`.
- `bash harness_check.sh` limpio, `cargo test --locked`,
  `cargo clippy --all-targets --all-features --locked -- -D warnings`,
  `bash tests/setup_smoke.sh`.
- Espejos identicos: `harness_check.sh` == `templates/harness_check.sh`;
  `docs/prd/*` == `templates/docs/prd/*`.
- Paridad sh/ps1 en superficies y smoke.
- Commit sin trailers de IA (UPDATING.md).

## Riesgos

- **Escribir dentro de un documento del USUARIO** (el enlace en el PRD padre):
  mitigado por append acotado a una sola seccion, idempotencia (no duplica) y
  cero reordenamiento del resto.
- **Regimen de archivos**: `PRD-master.md`/`SDD-master.md` son del USUARIO
  (`PRD_DOCS`) y la guia es plantilla refrescable (`HARNESS_DOCS`). El cambio de
  la planilla maestra NO se propaga a instalaciones existentes (ni debe): solo
  aplica a proyectos nuevos. Se documenta en `UPDATING.md`.
- **Compatibilidad de `feature_list.json`**: el campo `prd` es opcional; las 13
  features existentes no lo tienen. El escritor ya preserva claves y orden.
- **Slug hostil** (`../`, separadores): normalizar antes de tocar el filesystem.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

- Las cuatro decisiones de diseno (layout, creacion, cadena, validacion) y las
  tres propuestas del spec quedaron DECIDIDAS por el usuario (2026-08-12) y
  selladas en la aprobacion del spec. Sin decisiones pendientes.

### Avance 2026-08-12T11:39:54Z
Implementadas D1-D12: modulo prd.rs, comandos prd add/tree, add --prd, encabezado PRD del spec, vuelta del cierre al PRD, gate del arbol en harness_check.sh (+espejo), guia/planilla/superficies y tests (unit + smoke)

---
Cerrado: 2026-08-12T11:44:10Z - status=done - PRDs anidados reales: arbol en carpetas, prd add/tree, cadena --prd -> spec, vuelta del cierre al PRD y gate de integridad
