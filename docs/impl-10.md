# Impl - Feature #10: layout_inferred_from_footprint

Spec: docs/spec-feature-10-layout-inferred-from-footprint.md (Estado: approved, sellado 2026-07-29T04:19:41Z)
Plan: docs/plan-feature-10-layout-inferred-from-footprint.md

> Nota de proceso: la implementacion quedo commiteada en `c09e1dd`
> (fix(layout): infiere subdir por la huella del padre si falta el marker) en la
> sesion del 2026-07-29, sin la evidencia por AC. Este documento la registra a
> posteriori con la verificacion re-ejecutada el 2026-07-30 sobre el working
> tree actual (mismo contenido que el commit; `git status` sin cambios en los
> archivos de la feature).

## Que cambio

El bloque de resolucion de `REPO_ROOT` (4 scripts + `paths.rs`) distinguia mal
dos estados: `[ "$(cat .harness_layout 2>/dev/null)" = "subdir" ]` confundia
"archivo ausente" con "archivo con otro valor". Ahora son tres casos
excluyentes: (a) marker `subdir` -> padre, con el guardrail de checkout fuente
de la #7 intacto; (b) marker AUSENTE -> inferencia por huella del padre (las
mismas 4 huellas y la misma guarda de `$HOME`), con aviso `[i]` y remedio;
(c) marker presente con otro valor (`root`) -> dir del arnes, sin inferencia y
sin aviso.

## Unidades

| Unidad | AC | Archivos |
| --- | --- | --- |
| U1 regla en 4 scripts + espejos | AC-1..AC-7, AC-9 | `harness_check.sh`, `harness_status.sh`, `init.sh`, `commit_guard.sh` + sus 4 espejos en `templates/` |
| U2 regla en Rust | AC-8 | `rust/src/paths.rs` (`repo_root_from_marker` + helpers `non_empty_parent`, `parent_has_footprint`, `parent_is_home`), integracion en `rust/tests/cli_basics.rs` |
| U3 smoke sh | AC-10, AC-13 | `tests/setup_smoke.sh` (bloque "Feature #10", linea 653) |
| U4 paridad ps1 | AC-11 | `tests/setup_smoke.ps1` (bloque "Feature #10", linea 364) |
| U5 docs | AC-12 | `UPDATING.md`, `templates/UPDATING.md`, `docs/architecture.md`, `README.md` |

## Evidencia por AC

Reproduccion antes/despues con fixture propia (`/tmp/f10_fixture/proyecto/` con
huella `docs/constitution.md` + `CLAUDE.md` + `AGENTS.md`, y
`harness_process/` dentro con el bloque de resolucion extraido del script
real, pre-fix de `git show c09e1dd^:harness_check.sh` y actual del working
tree):

```
A. SIN marker + padre CON huella:
   ANTES   REPO_ROOT=.../proyecto/harness_process            (bug reproducido)
   DESPUES [i] .harness_layout ausente: layout subdir inferido por la huella
           de instalacion del padre: REPO_ROOT=.../proyecto. Re-corre el
           instalador (setup_harness.sh / setup_harness.ps1) para regenerar
           el marker.
           REPO_ROOT=.../proyecto                            (fix)
```

- **AC-1** (sin marker + huella -> padre): escenario A-DESPUES. Ademas, los 4
  escenarios corrieron con el bloque extraido verbatim de `harness_check.sh`;
  los otros 3 scripts son identicos por `diff` (AC-9), asi que la misma regla
  aplica a `harness_status.sh`, `init.sh` y `commit_guard.sh`.
- **AC-2** (aviso `[i]` unico, exit code intacto): escenario A-DESPUES emite el
  aviso a stderr diciendo que el marker falta, que se infirio subdir por la
  huella del padre y el remedio (re-correr el instalador). El script termino
  rc=0 (el `echo` final se ejecuto). El aviso se emite una sola vez por
  corrida, en el punto de resolucion.
- **AC-3** (marker `root` respetado): escenario B — `.harness_layout=root` +
  padre con huella -> `REPO_ROOT=.../proyecto/harness_process`, SIN aviso. La
  inferencia solo dispara en la rama `elif [ ! -f "$harness_marker" ]`
  (archivo AUSENTE). Tambien cubierto en Rust:
  `paths.rs::repo_root_should_not_infer_when_marker_says_root` y
  `cli_basics.rs::explicit_root_marker_should_never_infer_subdir`.
- **AC-4** (sin marker + sin huella -> propio dir): escenario C — fixture
  `/tmp/f10_fixture/pelado/` sin huella en el padre ->
  `REPO_ROOT=.../pelado/harness_process`, SIN aviso. En Rust:
  `repo_root_should_not_infer_without_parent_footprint`.
- **AC-5** (guarda de `$HOME` en la inferencia): `parent_is_home` se calcula
  con la MISMA guarda de la #7 y la inferencia exige
  `harness_parent_is_home -eq 0`. En Rust:
  `cli_basics.rs::home_parent_should_block_marker_inference` (con el escape
  `HARNESS_ALLOW_HOME_SURFACE=1` la huella vuelve a mandar) y
  `repo_root_should_stay_local_for_source_checkout_without_parent_footprint`.
- **AC-6** (precedencia de overrides): escenario E — con
  `HARNESS_REPO_ROOT=/tmp/custom_root`, `REPO_ROOT=/tmp/custom_root` sin
  aviso: la resolucion por inferencia vive dentro del `if [ -z "$REPO_ROOT" ]`,
  asi que `HARNESS_REPO_ROOT` y las variables de agente
  (`CLAUDE_PROJECT_DIR`, `CODEX_PROJECT_DIR`, ...) siguen mandando. En Rust:
  `cli_basics.rs::env_override_should_beat_marker_inference`.
- **AC-7** (guardrail #7 sin regresion): verificado en ESTE checkout —
  `bash harness_check.sh` imprime `[i] Checkout fuente del arnes detectado
  (.harness_layout=subdir sin huella de instalacion en el padre):
  REPO_ROOT=/Users/alan/harness_process` y sale rc=0. Ademas el escenario D
  (marker `subdir` + padre con huella) resuelve al padre SIN aviso, como
  antes de la feature. En Rust:
  `home_parent_should_trigger_source_guardrail_even_with_footprint`.
- **AC-8** (misma regla en Rust, punto unico): `rust/src/paths.rs` —
  `repo_root_from_marker` con helpers `non_empty_parent`,
  `parent_has_footprint` (las mismas 4 huellas), `parent_is_home` (misma
  guarda) y `source_checkout_mismatch` (guardrail #7 intacto). Tests unitarios
  nuevos: `repo_root_should_infer_subdir_without_marker_when_parent_has_footprint`,
  `repo_root_inference_should_accept_any_single_footprint_file`,
  `repo_root_should_not_infer_without_parent_footprint`,
  `repo_root_should_not_infer_when_marker_says_root`,
  `repo_root_should_not_infer_when_marker_is_empty_or_unknown`,
  `repo_root_should_not_infer_without_usable_parent`,
  `repo_root_should_resolve_parent_without_source_signals`. Integracion:
  `start_should_infer_subdir_root_when_marker_is_missing` (verifica el aviso
  `[i]` verbatim). `cargo test --locked`: 50 unit + 27 integracion, 0 fallos
  (2026-07-30).
- **AC-9** (espejos identicos): `diff` de los 4 scripts raiz vs `templates/`
  = identicos (verificado 2026-07-30): `harness_check.sh`,
  `harness_status.sh`, `init.sh`, `commit_guard.sh`.
- **AC-10** (smoke sh): `tests/setup_smoke.sh` bloque "Feature #10" (linea
  653) con fixtures propias para (a) sin marker + huella -> padre + aviso,
  (b) sin marker sin huella -> propio dir sin aviso, (c) marker `root` -> sin
  inferencia, (d) guardrail #7 verde; mas la guarda de `$HOME` y los
  overrides. Corrida 2026-07-30: `bash tests/setup_smoke.sh` rc=0, linea
  `[Ok] marker ausente: layout subdir inferido por huella del padre (scripts +
  binario) con aviso [i]; sin huella, marker 'root', $HOME y overrides no
  infieren; guardrail #7 intacto.` y todas las lineas `[Ok]` previas intactas
  (Kimi backend, docs raiz, PRD/SDD, Rust-only, layouts, reset).
- **AC-11** (paridad ps1): `tests/setup_smoke.ps1` bloque "Feature #10"
  (linea 364) en paridad estatica con el `.sh`: (1) verifica que los CUATRO
  scripts sembrados traen la regla (`\.harness_layout ausente`,
  `harness_parent_footprint`, el gate `elif [ ! -f "$harness_marker" ]`) y que
  conservan el guardrail de la #7; (2) con bash disponible ejecuta fixtures
  `New-LostMarkerCase` (marker ausente -> aviso de inferencia; `marker-root`
  -> nunca infiere). Sin `pwsh`/`powershell` en esta maquina (verificado
  2026-07-30, `which` falla para ambos): verificacion ESTATICa declarada como
  tal, mismo criterio que las features #1, #4, #5, #6, #7, #8 y #9.
  `setup_harness.ps1` no requirio cambios: el instalador ya escribe el marker
  en cada instalacion (decision del plan, "SIN cambios en setup_harness").
- **AC-12** (docs corregidas): `UPDATING.md` (raiz, lineas 175-239) y
  `templates/UPDATING.md` (lineas 215-279) — la nota de migracion de la #7
  ahora dice que tras `git pull` el marker desaparece pero "Ya no hace falta
  re-correr el instalador para que la raiz vuelva a ser tu proyecto", y la
  seccion nueva "Layout `subdir` inferido cuando falta `.harness_layout`"
  explica la regla, el aviso `[i]` y cuando SI conviene regenerar el marker
  (para que el aviso desaparezca). `docs/architecture.md` (lineas 44, 173-183)
  documenta los tres casos. La diferencia raiz vs template es solo el
  historial propio del proyecto (notas 2026-06), esperado.
- **AC-13** (comandos oficiales de `docs/verification.md`), corrida
  2026-07-30: `bash tests/setup_smoke.sh` rc=0;
  `(cd rust && cargo clippy --all-targets --all-features --locked --
  -D warnings)` rc=0; `(cd rust && cargo test --locked)` rc=0 (50+27).
