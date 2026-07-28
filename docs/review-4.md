# Review - Feature #4: harness_docs_to_root_docs

Spec: docs/spec-feature-4-harness-docs-to-root-docs.md (Estado: approved, SIN sello)
Plan: docs/plan-feature-4-harness-docs-to-root-docs.md
Impl: docs/impl-4.md

> **Veredicto reconstruido.** La feature #4 se cerro el 2026-07-24T20:53:14Z
> (`close ... status=done note=`) sin dejar `docs/review-4.md`, rompiendo la
> cadena de evidencia durable que si tienen las features #1, #2, #3, #5 y #6.
> Este documento lo reconstruye el 2026-07-28. Los AC NO se copian de
> `impl-4.md`: se re-verificaron en esta sesion contra el arbol commiteado en
> `55cd538`. Donde la evidencia solo consta de la epoca y no es reproducible hoy,
> se dice explicitamente.

## Veredicto global

**approved**, con la misma salvedad que arrastran las features #1, #5 y #6:
AC-7 (paridad Windows) queda verificado por revision estatica, no por ejecucion,
porque no hay `pwsh` ni `powershell` en esta maquina (`command -v pwsh
powershell` sin salida, re-comprobado el 2026-07-28).

El cierre retroactivo no cambia el veredicto: el comportamiento que la feature
prometia esta hoy vivo en el arbol y cubierto por el smoke.

## Aprobacion del spec

El spec #4 esta `Estado: approved` pero **sin linea `Aprobado:`**, igual que el
#3. Ambos son anteriores al ritual de la feature #6 (`harness_cli approve-spec`,
que sella con fecha, actor y nota). La aprobacion consta en `progress/history.md`
por via indirecta: el `advance` del 2026-07-24T20:51:09Z solo pudo registrarse
con el gate `require_spec_approved` activo y el spec ya en `approved`.

Esto contradice la nota de proceso de `docs/impl-4.md` ("NO hay `harness_cli
advance` registrado"), que quedo escrita ANTES de que el usuario aprobara el
spec. El historial es la fuente de verdad: el advance existe.

## Estado por AC

Verificado en esta sesion sobre el arbol commiteado.

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | `tests/setup_smoke.sh:186-194` (fixture `subdir-layout`): los tres docs existen en `<raiz>/docs/` y se afirma `test ! -d "$SUBDIR_HARNESS/docs"` — el arnes no deja `docs/` propio |
| AC-2 | cubierto | fixture `root-layout` (`tests/setup_smoke.sh:136-155`) instala con `--root` y el fixture `reset-test` confirma los tres docs en `docs/` antes del reset (lineas 425-430); reinstall idempotente ejercitado en el bloque de reinstall |
| AC-3 | cubierto | fixture `migrate-layout` (`tests/setup_smoke.sh:262-283`): `VIEJO-ARCHITECTURE` / `VIEJO-VERIFICATION` aparecen en `<raiz>/docs/` (contenido movido, no plantilla regenerada), desaparecen de la subcarpeta y el log exige `Migrado al docs/ de la raiz` |
| AC-4 | cubierto | doble cobertura: (a) reinstall — sentinel `SENTINEL-DOCS-ARNES-NO-PISA` en `docs/conventions.md` sobrevive (lineas 229-230, 253-254); (b) migracion — `SENTINEL-CONVENTIONS-DEL-EQUIPO` intacto, copia vieja conservada y log con `ya existe` (lineas 269-282) |
| AC-5 | cubierto | `setup_harness.sh:917-920` cita `docs/architecture.md`, `docs/conventions.md` y `docs/verification.md` como "(RAIZ)", sin prefijo `__HREL__`; las rutas resuelven desde la raiz del proyecto |
| AC-6 | cubierto | `tests/setup_smoke.sh:402-434`: con constitution editada (`SENTINEL-CONSTITUTION-RESET`) y `spec-feature-1-demo.md` / `plan-feature-1-demo.md` / `review-1.md` sembrados, el `--reset` limpia los tres docs generados y falla ruidosamente si toca la constitution o cualquier artefacto de feature |
| AC-7 | **parcial** | `setup_harness.ps1` y `tests/setup_smoke.ps1` portados y revisados estaticamente; SIN ejecutar (no hay PowerShell en la maquina). Deuda compartida con #1, #5 y #6 |
| AC-8 | cubierto | **re-ejecutado**: `grep -rn "harness_process/docs/\|__HREL__docs/" roles/ templates/roles/ .claude/agents/` sin hits. Las unicas referencias vivas son a `docs/verification.md` en `roles/{implementer,reviewer}.md` y sus dos espejos (`templates/roles/`, `.claude/agents/`), todas resolviendo a la raiz |
| AC-9 | cubierto | **re-ejecutado el 2026-07-28**: `bash tests/setup_smoke.sh` rc=0 incluyendo `[Ok] docs del arnes en el docs/ de la RAIZ: destino, migracion, no-pisa y reset.`; `cargo test` 40 unit + 19 integracion; `cargo clippy --all-targets -- -D warnings` sin issues. `tests/setup_smoke.ps1` no ejecutado (ver AC-7) |
| AC-10 | cubierto | `UPDATING.md` y `templates/UPDATING.md` (5 refs cada uno, con seccion de migracion), `AGENTS.md` (4 refs), `README.md` (arbol de `docs/` + migracion) y `docs/architecture.md:112-124`, que describe la ubicacion nueva y la migracion de instalaciones previas |

## Trazabilidad y constitution

- **Articulo 1**: los tres comandos de `docs/verification.md` en verde; el smoke
  gano un bloque propio y asserts embebidos en los fixtures existentes.
- **Articulo 2**: spec `approved` antes del `advance` y del `close`. El sello
  formal no existe porque la feature es anterior al comando `approve-spec`.
- **Articulo 3**: las ocho unidades (U1-U8 en `impl-4.md`) citan sus AC y tienen
  evidencia; la tabla de arriba las re-verifica una por una.
- **Articulo 4**: sin secretos; la migracion solo MUEVE dentro del proyecto y
  nunca sobrescribe un destino existente; `--reset` conserva su lista blanca.
- **Articulo 5**: las tres decisiones del usuario (alcance de la mudanza,
  migracion solo-si-falta, siembra if-missing) estan registradas en las
  Observaciones del spec, incluida la que se levanto durante la implementacion.
- **Articulo 6**: sin dependencias nuevas; `rust/src/` no se toco (la resolucion
  `plans = repo_root/docs` ya apuntaba a la raiz); raiz y `templates/` espejados.

## Impacto y gates

- Microservicio unico `harness`; sin contratos compartidos. `graph impacto` no se
  ejecuto contra el hub (PostgreSQL inalcanzable; los comandos degradan
  best-effort). El impacto era de superficie de instalacion y se verifico con el
  smoke sobre instalaciones reales en tres layouts (root, subdir, migracion).
- `harness_check.sh` limpio con `HARNESS_REPO_ROOT=/Users/alan/harness_process`.
  Sin esa variable falla con `[!] Falta docs/constitution.md`: es el footgun del
  checkout fuente (`.harness_layout=subdir` resuelve la raiz a `$HOME`), no una
  regresion de esta feature.

## Hallazgos (no bloquean el veredicto)

1. **La #4 se cerro sin review.** Causa raiz: `close` no exige `docs/review-N.md`
   como si exige spec aprobado. Candidato a gate futuro — el mismo hueco puede
   repetirse en cualquier feature.
2. **Specs #3 y #4 `approved` sin sello.** `check-spec` los acepta porque solo
   mira `Estado:` + frescura de firma. Si se quiere el sello uniforme, basta
   `harness_cli approve-spec --yes` con la feature activa (idempotente), pero
   fecharia hoy una aprobacion de julio: preferible dejarlos como estan y anotar
   el motivo, que es lo que hace este parrafo.
3. **AC-7 sin ejecucion real en Windows.** Una sola corrida de
   `pwsh tests/setup_smoke.ps1` salda la deuda acumulada de #1, #4, #5 y #6.
4. **Residuos en `$HOME/docs`**: RESUELTO el 2026-07-28. Eran `plan-feature-1` y
   `plan-feature-2`, ambos plantillas vacias (los planes reales viven en
   `docs/` del repo). Borrados por decision del usuario; `$HOME/docs` quedo
   vacio y se elimino.
