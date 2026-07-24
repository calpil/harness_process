# Impl - Feature #5: prd_master_templates

Spec: docs/spec-feature-5-prd-master-templates.md (Estado: draft)
Plan: docs/plan-feature-5-prd-master-templates.md

> Nota de proceso: la implementacion avanzo por indicacion explicita del usuario
> CON el gate `require_spec_approved` activo y el spec en `draft`. No hay
> `harness_cli advance` registrado (el gate lo bloquea, como debe). La feature no
> puede cerrarse hasta la aprobacion del USUARIO.

## Que cambio

El instalador siembra dos planillas maestras en `docs/prd/` de la RAIZ del
proyecto, para arrancar proyectos desde cero:

```
miproyecto/docs/prd/
|-- PRD-master.md    que se construye y por que
`-- SDD-master.md    como se construye, a nivel proyecto
```

Se tratan como `docs/constitution.md`: documentos del USUARIO. Se siembran solo
si faltan, ningun reinstall las pisa, `--force` tampoco, y `--reset` NO las
borra. Esa ultima garantia es la diferencia deliberada con los tres docs del
arnes de la feature #4, que si se limpian con `--reset` por ser plantillas
regenerables.

## Unidades

| Unidad | AC | Archivos |
| --- | --- | --- |
| U1 plantillas | AC-7, AC-8 | `templates/docs/prd/PRD-master.md`, `templates/docs/prd/SDD-master.md` |
| U2 siembra (sh) | AC-1, AC-2, AC-3, AC-5 | `setup_harness.sh` (`PRD_DOCS`, `do_mkdir`, siembra if-missing, `required_assets`) |
| U3 reset (sh) | AC-4 | `setup_harness.sh` (comentario de la lista blanca; `docs/prd/` deliberadamente ausente) |
| U4 paridad ps1 | AC-6 | `setup_harness.ps1` (`$script:PrdDocs`, `Ensure-Directory`, siembra, `$required`) |
| U5 tests | AC-9 | `tests/setup_smoke.sh`, `tests/setup_smoke.ps1` |
| U6 docs | AC-10 | `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `AGENTS.md`, `setup_harness.sh` (superficie), `docs/architecture.md` |
| U7 seed en el repo fuente | — | `docs/prd/PRD-master.md`, `docs/prd/SDD-master.md` |

## Evidencia por AC

Fixtures en directorio temporal; nunca en el checkout fuente.

- **AC-1** (layout subdir): `<raiz>/docs/prd/PRD-master.md` y
  `<raiz>/docs/prd/SDD-master.md` presentes; `<raiz>/harness_process/docs/prd/`
  no existe. Cubierto por `tests/setup_smoke.sh` (fixture `subdir-layout`).
- **AC-2** (layout root): planillas en `docs/prd/`; segunda corrida idempotente
  (las cuenta como skipped, no las reescribe).
- **AC-3** (no-pisa en reinstall): sentinel `SENTINEL-PRD-NO-PISA-$$` agregado a
  `docs/prd/PRD-master.md` sobrevive al reinstall. Cubierto en el smoke.
- **AC-4** (reset): sentinel `SENTINEL-PRD-RESET-$$` agregado antes de
  `--reset`; tras el reset, `docs/prd/PRD-master.md` conserva el sentinel y
  `docs/prd/SDD-master.md` sigue existiendo, mientras `architecture.md`,
  `conventions.md` y `verification.md` si se borraron. Cubierto en el smoke.
- **AC-5** (preflight): fixture con `templates/docs/prd/PRD-master.md` borrado ->
  el instalador sale con exit 2 y
  `[!] Falta el recurso requerido: docs/prd/PRD-master.md (buscado en .../templates)`.
- **AC-6**: PENDIENTE DE EJECUCION. No hay `pwsh` en esta maquina. `setup_harness.ps1`
  y `tests/setup_smoke.ps1` se portaron y revisaron estaticamente (balance de
  llaves OK; `$script:PrdDocs`, `$script:Counters.skipped` y `Install-HarnessAsset
  -Destination` contrastados contra el codigo existente) pero NO se ejecutaron.
  Igual que el AC-7 de la feature #4: requiere corrida en Windows.
- **AC-7** (contenido PRD): el smoke verifica `## 7. Hitos -> features` y la
  mencion de `harness_cli add` en la planilla sembrada.
- **AC-8** (contenido SDD): el smoke verifica `## 4. Decisiones tecnicas` y la
  referencia explicita a `docs/architecture.md` que las distingue.
- **AC-9**: `bash tests/setup_smoke.sh` exit 0, con la linea nueva
  `[Ok] planillas maestras docs/prd/ (PRD + SDD): siembra, no-pisa y supervivencia al reset.`;
  `cargo test` 35 + 14 OK; `cargo clippy -- -D warnings` exit 0.
  `tests/setup_smoke.ps1` escrito, no ejecutado (ver AC-6).
- **AC-10**: `README.md` (arbol de `docs/` + seccion "Proyectos que arrancan de
  cero" con el diagrama PRD -> backlog -> spec -> plan -> impl/review),
  `UPDATING.md` y `templates/UPDATING.md` (seccion nueva + entrada en "Que se
  actualiza"), `AGENTS.md` y el heredoc de superficie de `setup_harness.sh` (para
  que un reinstall no borre la entrada), `docs/architecture.md` (paso 0 del flujo
  SDD y descripcion de `PRD_DOCS`).

## Notas para el reviewer

- `docs/prd/` NO esta en `reset_targets` a proposito. El comentario de la lista
  blanca en ambos instaladores lo dice explicitamente, para que un cambio futuro
  no lo agregue por inercia. AC-4 lo testea.
- `--force` no pisa las planillas (a diferencia de `HARNESS_DOCS`). Es coherente
  con `docs/constitution.md`, que tampoco tiene rama `FORCE`.
- `required_assets` / `$required` SI incluyen las dos plantillas: validan que
  existan en `templates/`, no el destino.
- `rust/src/` no se toco: las planillas no son artefactos que el binario genere
  ni vigile (fuera de alcance por spec).
- Falta para cerrar: aprobacion del spec por el usuario, `advance`, corrida real
  de `tests/setup_smoke.ps1` en Windows (AC-6) y veredicto del reviewer.
