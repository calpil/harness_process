# Impl - Feature #4: harness_docs_to_root_docs

Spec: docs/spec-feature-4-harness-docs-to-root-docs.md (Estado: draft)
Plan: docs/plan-feature-4-harness-docs-to-root-docs.md

> Nota de proceso: la implementacion avanzo por decision explicita del usuario
> CON el gate `require_spec_approved` activo y el spec en `draft`. Por eso NO hay
> `harness_cli advance` registrado (el gate lo bloquea, como debe) ni entradas en
> `progress/current.md`. La feature no puede cerrarse hasta la aprobacion.

## Que cambio

Los tres docs del arnes (`architecture.md`, `conventions.md`, `verification.md`)
pasan del `docs/` de la subcarpeta del arnes al `docs/` de la RAIZ del proyecto,
junto a `constitution.md`, specs y planes. El arnes ya no crea un `docs/` propio.

Ademas, por decision del usuario tomada durante U1, esos tres docs pasan a
sembrarse **solo si faltan** (antes se respaldaban y regeneraban en cada
reinstall). Al compartir carpeta con la documentacion del equipo, un
`docs/conventions.md` propio no puede perderse en un reinstall.

## Unidades

| Unidad | AC | Archivos |
| --- | --- | --- |
| U1 destino raiz (sh) | AC-1, AC-2 | `setup_harness.sh` (`HARNESS_DOCS`, siembra if-missing, `generated`, `do_mkdir`) |
| U2 migracion (sh) | AC-3, AC-4 | `setup_harness.sh` (`migrate_harness_docs`) |
| U3 reset (sh) | AC-6 | `setup_harness.sh` (`reset_targets`) |
| U4 superficies | AC-5 | `setup_harness.sh` (texto de superficie), `AGENTS.md` |
| U5 roles y agentes | AC-8 | `templates/roles/{implementer,reviewer}.md`, `roles/*`, `.claude/agents/*` |
| U6 paridad PowerShell | AC-7 | `setup_harness.ps1` (`$script:HarnessDocs`, `Move-HarnessDocsToRoot`, siembra, reset, directorios) |
| U7 tests | AC-9 | `tests/setup_smoke.sh`, `tests/setup_smoke.ps1` |
| U8 docs | AC-10 | `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `AGENTS.md`, `docs/architecture.md` |

## Evidencia por AC

Fixtures aislados en directorio temporal (nunca el checkout fuente: su
`.harness_layout` dice `subdir` y la raiz resuelta seria `$HOME`).

- **AC-1** (instalacion limpia, layout subdir):
  `miproyecto/docs/` -> `architecture.md`, `constitution.md`, `conventions.md`,
  `verification.md`; `miproyecto/harness_process/docs/` -> no existe.
- **AC-2** (layout root): los cuatro docs quedan en `docs/`; reinstall idempotente.
- **AC-3** (migracion): fixture con `VIEJO-ARCH` / `VIEJO-CONV` / `VIEJO-VERIF` en
  `<harness>/docs/`. Tras reinstalar, el contenido aparece en `<raiz>/docs/`
  (`VIEJO-ARCH`, no la plantilla), la subcarpeta `docs/` desaparece y el log trae
  `[INFO] Migrado al docs/ de la raiz: docs/architecture.md (antes en ...)`.
- **AC-4** (no-pisa): con `CONVENCIONES-DEL-EQUIPO-SENTINEL` en
  `<raiz>/docs/conventions.md` y una copia vieja en la subcarpeta, tras instalar y
  volver a instalar el sentinel sigue intacto, la copia vieja se conserva y el log
  trae `[WARN] Migracion: ... ya existe; se conserva intacto y NO se pisa`.
- **AC-5**: superficies citan `docs/architecture.md` (RAIZ) sin el prefijo
  `__HREL__`.
- **AC-6** (reset): con constitution editada (sentinel) y `spec-feature-1-demo.md`,
  `plan-feature-1-demo.md`, `review-1.md` presentes, `--reset` deja en `docs/`
  solo `constitution.md` + los tres artefactos de feature; los tres docs generados
  desaparecen y el sentinel de la constitution sobrevive.
- **AC-7**: PENDIENTE DE EJECUCION. `pwsh` no esta disponible en esta maquina, asi
  que `setup_harness.ps1` y `tests/setup_smoke.ps1` se portaron y revisaron
  estaticamente (balance de llaves OK, nombres de contadores y switches
  verificados contra el codigo existente) pero NO se ejecutaron. Misma situacion
  que dejo la feature #1. Requiere una corrida en Windows antes de cerrar.
- **AC-8**: `grep` sin hits de `harness_process/docs/` ni `__HREL__docs/` en
  `roles/`, `templates/roles/`, `.claude/agents/` y ambos instaladores (los hits
  restantes son las rutas legacy que usan la migracion y el reset a proposito).
- **AC-9**: `bash tests/setup_smoke.sh` exit 0 con la linea nueva
  `[Ok] docs del arnes en el docs/ de la RAIZ: destino, migracion, no-pisa y reset.`;
  `cargo test` 35 + 14 tests OK; `cargo clippy -- -D warnings` exit 0.
  `tests/setup_smoke.ps1` escrito pero no ejecutado (ver AC-7).
- **AC-10**: `README.md` (seccion nueva con el arbol de `docs/`), `UPDATING.md` y
  `templates/UPDATING.md` (seccion de migracion), `AGENTS.md`, `docs/architecture.md`.

## Notas para el reviewer

- `required_assets` / `$required` NO se tocaron a proposito: validan que la
  plantilla exista en `templates/`, no el destino de instalacion.
- El reset limpia los tres docs en la ubicacion nueva Y en la vieja, para que un
  `--reset` sobre una instalacion sin migrar tampoco deje basura.
- `rust/src/` no se toco: la resolucion de rutas del binario
  (`plans = repo_root/docs`) ya apuntaba a la raiz.
- Falta para cerrar: aprobacion del spec por el usuario, `advance`, corrida real
  de `tests/setup_smoke.ps1` en Windows (AC-7) y veredicto del reviewer.
