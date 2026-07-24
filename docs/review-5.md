# Review - Feature #5: prd_master_templates

Spec: docs/spec-feature-5-prd-master-templates.md (Estado: approved, sellado)
Plan: docs/plan-feature-5-prd-master-templates.md
Impl: docs/impl-5.md

## Veredicto global

**approved**, con una salvedad explicita: AC-6 (paridad Windows) queda
verificado por revision estatica, no por ejecucion, porque no hay `pwsh` ni
`powershell` en esta maquina (verificado con `command -v`). Es la misma
limitacion aceptada en las features #1, #4 y #6.

## Aprobacion del spec

Aprobado por el USUARIO con el flujo de la feature #6:

```
Estado: approved
Aprobado: 2026-07-24T22:58:15Z por USUARIO (confirmacion explicita) - Alan aprobo
el spec #5 en el chat (2026-07-24); normalizacion del sello tras aprobacion
manual previa
```

Rastro en `progress/history.md` (dos lineas `approve-spec feature #5`: la
re-firma inicial y la normalizacion con sello). `check-spec --feature 5` sale
rc=0 (`[OK] Spec aprobado y fresco`). La aprobacion original fue manual el
2026-07-24 18:21:38 y arrastraba la falsa alarma multi-LLM que motivo la #6.

## Estado por AC

Verificado en esta sesion sobre el arbol commiteado (no solo por lectura de
`impl-5.md`).

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | `tests/setup_smoke.sh` fixture `subdir-layout`: `<raiz>/docs/prd/{PRD,SDD}-master.md` presentes y `<raiz>/harness_process/docs/prd/` ausente (lineas 195-198) |
| AC-2 | cubierto | fixture layout root: planillas en `docs/prd/`; segunda corrida idempotente (las cuenta skipped) |
| AC-3 | cubierto | sentinel `SENTINEL-PRD-NO-PISA` sobrevive al reinstall (smoke, linea 256) |
| AC-4 | cubierto | `docs/prd/` NO figura en `reset_targets` (`setup_harness.sh:490-527`), con comentario que lo declara deliberado; el smoke verifica supervivencia al `--reset` (lineas 409-410) |
| AC-5 | cubierto | **re-ejecutado en esta revision**: fixture con `templates/docs/prd/PRD-master.md` borrado -> `[!] Falta el recurso requerido: docs/prd/PRD-master.md (buscado en .../templates)` y **exit code 2** exacto. Declarado en ambos instaladores (`setup_harness.sh:1474-1475`, `setup_harness.ps1:425`) |
| AC-6 | **parcial** | `setup_harness.ps1` y `tests/setup_smoke.ps1` portados y revisados estaticamente; SIN ejecutar (no hay PowerShell en la maquina) |
| AC-7 | cubierto | el smoke verifica `## 7. Hitos -> features` y la mencion de `harness_cli add` en el PRD sembrado |
| AC-8 | cubierto | el smoke verifica `## 4. Decisiones tecnicas` y la referencia a `docs/architecture.md` que distingue al SDD master |
| AC-9 | cubierto | `cargo test` 40 unit + 19 integracion, `cargo clippy --all-targets -- -D warnings` 0 issues, `bash tests/setup_smoke.sh` rc=0 con `[Ok] planillas maestras docs/prd/ (PRD + SDD): siembra, no-pisa y supervivencia al reset.` (`pwsh` no disponible, ver AC-6) |
| AC-10 | cubierto | `docs/prd` mencionado en README.md (4), UPDATING.md (6), AGENTS.md (2) y docs/architecture.md (5), describiendo el flujo PRD -> `feature_list.json` -> spec -> plan -> implementacion |

## Trazabilidad y constitution

- Cada unidad de la Delegacion (U1-U7 en `impl-5.md`) cita sus AC y tiene
  evidencia: **Articulo 3 cumplido**.
- **Articulo 1**: tests cercanos al codigo tocado (bloque PRD en el smoke) y los
  tres comandos oficiales de `docs/verification.md` en verde.
- **Articulo 2**: spec `approved` y sellado por el usuario antes del cierre. Se
  deja constancia de que la IMPLEMENTACION avanzo con el spec en `draft`, por
  indicacion explicita del usuario y con el gate activo (documentado por el
  implementer en `docs/impl-5.md`). El cierre si respeta el gate.
- **Articulo 4**: sin secretos; exit codes estables (preflight = 2 verificado).
- **Articulo 5**: las tres decisiones del usuario (ubicacion `docs/prd/`, dos
  planillas separadas, `docs/prd/` fuera del reset) estan registradas en las
  Observaciones del spec.
- **Articulo 6**: sin dependencias nuevas en `rust/Cargo.toml` (la feature no
  toco `rust/src/`); raiz y `templates/` espejados.

## Impacto y gates

- Microservicio unico `harness`; sin contratos compartidos. `graph impacto` no
  se ejecuto contra el hub (PostgreSQL inalcanzable en esta maquina; los
  comandos degradan best-effort). El impacto era de superficie de instalacion y
  se verifico con el smoke sobre instalaciones reales en tres layouts.
- `bash harness_check.sh` limpio antes del cierre.

## Pendientes declarados (no bloquean)

1. **AC-6 sin ejecucion real en Windows.** Recomendacion: correr
   `pwsh tests/setup_smoke.ps1` en la primera maquina Windows disponible; cubre
   de una sola vez la deuda acumulada de las features #1, #4, #5 y #6.
2. Residuos en `$HOME/docs` (`plan-feature-1`, `plan-feature-2`) por el footgun
   del layout del checkout fuente; fuera de alcance, espera decision del usuario.
