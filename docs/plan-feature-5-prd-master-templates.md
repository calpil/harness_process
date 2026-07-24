# Plan - Feature #5: prd_master_templates

Estado: in_progress
Microservicios:
- harness

Spec: docs/spec-feature-5-prd-master-templates.md (Estado: draft — PENDIENTE de
aprobacion del USUARIO)
Constitution: docs/constitution.md

## Alcance

Dar a un proyecto que arranca de cero las dos planillas maestras del que y del
como, sembradas por el instalador en `docs/prd/` de la RAIZ:

```
miproyecto/docs/
|-- constitution.md              principios (feature #3)
|-- architecture.md              mapa de lo que YA existe (feature #4)
|-- conventions.md               (feature #4)
|-- verification.md              (feature #4)
|-- prd/                         <-- ESTA feature
|   |-- PRD-master.md            que se construye y por que
|   `-- SDD-master.md            como se construye, a nivel proyecto
|-- spec-feature-<id>-<slug>.md  detalle por feature (AC-n)
`-- plan-feature-<id>-<slug>.md
```

La feature #4 dejo el patron listo: `HARNESS_DOCS` / `$script:HarnessDocs`,
siembra solo-si-falta contra `SURFACE_DIR/docs` y el criterio "documento del
usuario = no se respalda, no se pisa". Esta feature reusa ese patron para una
subcarpeta, con una diferencia deliberada: `docs/prd/` NO entra en los reset
targets (los tres docs del arnes si entran, porque son plantillas regenerables;
el PRD del proyecto no lo es).

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` ->
"Ningun microservicio registrado depende de 'harness_process/harness'".

Impacto acotado al arnes; transversal a todas las instalaciones que reinstalen,
pero puramente aditivo: solo crea archivos nuevos donde no habia nada.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

La superficie tocada es la misma que en la feature #4 (nodos `setup_harness_*`
del par de instaladores). Sin consumidores fuera del arnes; el binario Rust no
participa (las planillas no son artefactos que `harness_cli` genere ni vigile).

## Delegacion (implementer)

- U1 (AC-7, AC-8): crear `templates/docs/prd/PRD-master.md` y
  `templates/docs/prd/SDD-master.md` con las secciones que exigen los AC. Estilo
  de las plantillas existentes (`templates/docs/constitution.md`): sin acentos en
  el cuerpo, guias entre `<>` para reemplazar, nota de "documento del USUARIO".
  Encadenar con el flujo: tabla "Hitos -> features" citando `harness_cli add`, y
  recorridos P1/P2 alineados con la seccion homonima del spec.
- U2 (AC-1, AC-2, AC-3, AC-5): en `setup_harness.sh`, lista `PRD_DOCS`, crear
  `$SURFACE_DIR/docs/prd` con `do_mkdir` y sembrar cada planilla SOLO si falta
  (patron de `docs/constitution.md`, `setup_harness.sh:1882`). Agregar las dos a
  `required_assets` (validan la plantilla en `$ASSET_DIR`, no el destino).
  NO agregarlas a `generated` (no se respaldan ni se regeneran).
- U3 (AC-4): verificar que `docs/prd/` NO figure en `reset_targets` y dejarlo
  documentado en el comentario de la lista blanca, para que un cambio futuro no
  lo agregue por inercia.
- U4 (AC-6): paridad exacta en `setup_harness.ps1` — `$script:PrdDocs`,
  `Ensure-Directory` de `docs/prd`, siembra solo-si-falta con destino explicito
  bajo `SurfaceDir`, y las dos entradas en `$required`.
- U5 (AC-9): tests. En `tests/setup_smoke.sh`: siembra en el fixture subdir
  (AC-1) y ausencia de `docs/prd/` en la subcarpeta del arnes; sentinel de
  no-pisa en el reinstall (AC-3); supervivencia al `--reset` en el fixture root
  (AC-4). Espejar en `tests/setup_smoke.ps1`.
- U6 (AC-10): documentar en `README.md`, `UPDATING.md`, `templates/UPDATING.md`,
  `AGENTS.md` y `docs/architecture.md`, describiendo el flujo PRD master ->
  `feature_list.json` -> spec por feature -> plan -> implementacion.
- U7: sembrar tambien `docs/prd/` en este repo fuente (es una instalacion del
  arnes sobre si mismo, igual que `docs/constitution.md` y `docs/architecture.md`
  estan versionados aca).

Orden: U1 -> U2 -> U3 -> U5 (verificar sh) -> U4 -> U6 -> U7.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-10 en `docs/impl-5.md`, con salida real de comandos.
- `bash tests/setup_smoke.sh` verde; `pwsh tests/setup_smoke.ps1` verde (ver
  riesgo de entorno mas abajo).
- `cargo test` y `cargo clippy -- -D warnings` verdes. Esta feature NO debe tocar
  `rust/src/`; si lo hace, justificarlo.
- Fixture de reset que demuestre que `docs/prd/` sobrevive (AC-4).
- Ensayos SIEMPRE en directorios temporales, nunca en el checkout fuente.
- Commits sin trailers de IA (regla de `UPDATING.md`).

## Riesgos

- **Que el reset se lleve el PRD.** Es el riesgo central: `docs/prd/` vive dentro
  de `docs/`, que el reset ya toca de forma selectiva. Mitigacion: lista blanca
  explicita (nunca globs sobre `docs/`), AC-4 con fixture dedicado, y comentario
  en el codigo explicando por que no esta ahi.
- **Que un reinstall pise un PRD ya escrito.** Mitigacion: siembra solo-si-falta
  y AC-3 con sentinel; ademas no entra en `generated`, asi que ni siquiera se
  respalda (no hay ruta de codigo que lo sobrescriba salvo `--force`).
- **Paridad sh/ps1 sin poder ejecutar PowerShell.** No hay `pwsh` en el entorno
  de desarrollo (quedo documentado como AC-7 pendiente en la feature #4).
  Mitigacion: portar con revision estatica y dejar el AC-6 marcado como NO
  EJECUTADO en el impl, para que se corra en Windows antes del cierre.
- **Planillas que nadie completa.** Una planilla generica se ignora. Mitigacion:
  cada seccion trae una guia concreta de que escribir y la tabla de hitos indica
  el comando exacto que la conecta con el backlog.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Ubicacion: DECIDIDO (usuario, 2026-07-24) — `docs/prd/` en la RAIZ, dentro del
  `docs/` unificado por la feature #4.
- Que planillas: DECIDIDO (usuario, 2026-07-24) — `PRD-master.md` +
  `SDD-master.md` separadas, sin `README.md` adicional.
- Reset: DECIDIDO (usuario, 2026-07-24) — `docs/prd/` NO entra en reset targets.
- Sin decisiones abiertas. La implementacion avanza por indicacion explicita del
  usuario CON el gate activo (spec en `draft`); falta su aprobacion para poder
  registrar `advance` y cerrar la feature.

---
Cerrado: 2026-07-24T22:22:58Z - status=pending - Aparcada: implementada y commiteada (4c71f30); espera aprobacion del spec con el flujo nuevo de la feature #6

### Avance 2026-07-24T23:05:37Z
Reviewer: veredicto approved en docs/review-5.md (AC-1..AC-10 verificados; AC-6 Windows estatico). Preflight AC-5 re-ejecutado: exit 2

---
Cerrado: 2026-07-24T23:07:29Z - status=done - Planillas maestras PRD/SDD en docs/prd/; AC-6 (pwsh) pendiente de corrida en Windows
