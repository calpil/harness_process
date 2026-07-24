# Arquitectura — harness_process

Este repositorio es el **fuente del instalador** de un arnes multi-LLM (Claude,
Codex, Gemini, Grok, Antigravity y otros CLIs). No tiene microservicios propios;
sus componentes son el binario de ciclo de vida, los instaladores, las
superficies de instruccion y la capa de subagentes.

## Vision general

```
setup_harness.sh / .ps1  ->  superficies + hooks + subagentes + binario `harness`
                                     |
        roles/ (leader, implementer, reviewer)  +  docs/constitution.md (principios)
                                     |
   feature_list.json  --start-->  docs/spec-feature-*.md (draft) + docs/plan-feature-*.md
                                     |
                 usuario aprueba spec (draft -> approved)
                                     |
     advance / close / check-spec / harness_check.sh  (gate require_spec_approved)
```

## Binario Rust (`harness`)

`harness_cli` (POSIX) y `harness_cli.ps1` (PowerShell) despachan **exclusivamente**
al binario nativo `harness` / `harness.exe` (un solo ejecutable multi-OS,
compilado desde `rust/` con `cargo build --release --locked`). No hay fallback
Python desde la feature #2. Version actual: `rust/Cargo.toml` = 0.3.0.

### Modulos nucleo (`rust/src/`)

- `main.rs`: declara los modulos y delega en `cli::run`.
- `cli.rs`: definicion `clap` del `enum Command` (subcomandos de ciclo de vida al
  tope y `graph <cmd>` para el hub) y el dispatch a `commands::*`.
- `exit.rs`: `Exit { code, message }`, equivalente al `SystemExit` de Python.
  `Exit::msg(...)` => code 1 con mensaje a stderr; `Exit::code(2)` => code 2
  silencioso; ausencia de error => code 0.
- `paths.rs`: `HarnessPaths::resolve()` localiza raiz del proyecto, `docs/`,
  `progress/`, `feature_list.json` y honra `HARNESS_REPO_ROOT` / markers de layout.
- `features.rs`: carga/guarda `feature_list.json` y selecciona la feature activa.
- `plan.rs`: plantilla y firma del plan (`plan_signature` = dict
  path/mtime/size/hash), `is_plan_stale`, `plan_staleness_message`, `write_plan`,
  `update_plan_sig`.
- `spec.rs`: gemelo de `plan.rs` para el spec (ver seccion SDD).
- `progress.rs`: `current.md` / `history.md` (estado vivo y bitacora).
- `memories.rs`, `graphify.rs`, `graph/` (`commands`, `derive`, `ids`, `store`,
  `tls`): Memory Hub y su integracion con graphify.
- `pycompat.rs`: utilidades de formato de salida (compatibilidad historica).

### Comandos (`rust/src/commands/`)

`add`, `next`, `start`, `status`, `advance`, `close`, `autocheck`, `nudge`,
`check_plan`, `check_spec`. Los gates duros viven en `advance`, `close`
(solo `--status done`), `check_spec` y `harness_check.sh`; `autocheck` y `nudge`
son best-effort y NUNCA bloquean (tragan errores y re-firman en segundo plano).

### Exit codes (estables para hooks)

- `0`: ok (o gate apagado / spec aprobado y fresco).
- `1`: error accionable con mensaje (por ejemplo, sin feature `in_progress`).
- `2`: gate — plan o spec stale (editado por otro LLM sin re-firmar), o spec sin
  aprobar con la regla `require_spec_approved` activa. El stdout distingue el
  caso (plan vs spec).

## Flujo Spec-Driven Development (SDD)

Inspirado en spec-kit, adaptado y con **layout plano** (specs junto a los planes
en el `docs/` de la RAIZ, sin carpetas `specs/NNN/`).

0. (Proyecto nuevo) El USUARIO completa `docs/prd/PRD-master.md` (que se
   construye y por que) y `docs/prd/SDD-master.md` (como, a nivel proyecto). La
   tabla "Hitos -> features" del PRD se carga al backlog con `harness_cli add`.
   Paso opcional: ningun gate lo exige, y las planillas no las genera ni vigila
   el binario, solo las siembra el instalador.
1. `harness_cli start --feature <id>` siembra SIEMPRE (aunque la regla este
   apagada) `docs/spec-feature-<id>-<slug>.md` con `Estado: draft` ademas del
   plan, y firma ambos (`last_spec_sig` reusa `plan::plan_signature`).
2. `spec.rs` expone: `spec_path`, `spec_template`, `write_spec` (solo si falta),
   `get_spec_sig` / `update_spec_sig` (clave `last_spec_sig`), `is_spec_stale` /
   `spec_staleness_message` (hash distinto o drift mtime > 1s; falso sin archivo
   o sin firma previa), el enum `SpecState { Missing, Draft, Approved, Other }`
   con `spec_state` (primera linea `Estado:` dentro de las 10 primeras lineas,
   valor trim + case-insensitive), `require_spec_approved(data)` (lee
   `rules.require_spec_approved`, default `false`), `close_requires_spec` (solo
   `done` gatea) y `spec_gate` (mensaje accionable: ruta, estado y accion).
3. El LIDER completa spec y plan (cada item de la Delegacion cita su AC-n); el
   USUARIO aprueba el spec editando `Estado: draft` -> `Estado: approved`. Ningun
   agente puede auto-aprobar.
4. Con `require_spec_approved: true`, `advance`, `close --status done` y
   `harness_check.sh` (via `check-spec`) bloquean mientras el spec no este
   aprobado. `check-plan` vigila la frescura de spec y plan (exit 2 si cualquiera
   esta stale). El gate resuelve en <1s, solo filesystem, sin red.
5. `docs/constitution.md` (principios del proyecto) lo siembran ambos
   instaladores solo si falta y nunca lo pisan; specs, planes e implementacion
   deben cumplirlo y el reviewer lo verifica.

## Instaladores y superficies

- `setup_harness.sh` (Bash 3.2+) y `setup_harness.ps1` (PowerShell 5.1/7)
  generan las superficies (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `GROK.md`,
  `LLM.md`), hooks, launchers y la capa de subagentes.
- Los subagentes nativos se ensamblan desde `roles/*.md`: `.claude/agents/*.md`,
  `.codex/agents/*.toml` y `.gemini/agents/*.md` (leader, implementer, reviewer).
- Los assets versionados (`harness_cli`, `harness_check.sh`, roles,
  `CHECKPOINTS.md`, `UPDATING.md`, `docs/constitution.md`, ...) se copian desde
  `templates/`. Regla de mantenedor: `templates/` y la raiz se mantienen
  espejados; `roles/*.md` es el espejo de `templates/roles/*.md` con el
  placeholder `__HREL__` sustituido por la ruta relativa del arnes.
- TODA la documentacion del proceso se instala en el `docs/` de la RAIZ
  (`SURFACE_DIR/docs`): `constitution.md` mas los tres docs del arnes
  (`architecture.md`, `conventions.md`, `verification.md`, lista `HARNESS_DOCS` /
  `$script:HarnessDocs`). Ninguno esta en los assets regenerables: se siembran
  solo-si-faltan, no se respaldan y un reinstall no los pisa (solo `--force`).
  El arnes ya no crea un `docs/` propio.
- Planillas maestras del proyecto: `docs/prd/PRD-master.md` y
  `docs/prd/SDD-master.md` (listas `PRD_DOCS` / `$script:PrdDocs`) se siembran en
  `SURFACE_DIR/docs/prd` solo-si-faltan. Son documentos del USUARIO: ni `--force`
  las pisa y NO figuran en los reset targets, a diferencia de `HARNESS_DOCS`, que
  son plantillas regenerables del arnes.
- Migracion: `migrate_harness_docs()` (sh) / `Move-HarnessDocsToRoot` (ps1)
  mueven los docs que quedaron en `<harness>/docs/` de instalaciones previas,
  solo cuando faltan en la raiz; si ya existen, avisan y no pisan nada.
- `--reset` borra los tres docs generados en ambas ubicaciones (nueva y vieja) y
  conserva la constitution y los artefactos de feature.

## Memory Hub

El hub usa exclusivamente PostgreSQL; se accede bajo `harness graph <cmd>`
(`mapa`, `impacto`, `vincular`, ...). La conexion se configura por entorno o
`$HARNESS_HUB/.env` (parseado linea a linea). El gate SDD nunca consulta el hub.

## Layouts

- `subdir` (por defecto): el arnes vive en `harness_process/` dentro de la raiz
  multi-repo y escribe superficies en el directorio padre. Toda la documentacion
  del proceso (constitution, docs del arnes, spec y plan) vive en el `docs/` de
  la RAIZ; el arnes no tiene `docs/` propio.
- `root`: el arnes se instala directamente en la raiz (`SURFACE_DIR == HARNESS_DIR`).

## Riesgos conocidos

- Exit code 2 sobrecargado (plan vs spec stale): el stdout debe distinguir; no se
  cambia la semantica 0/1/2.
- Instalaciones existentes no reciben la regla `require_spec_approved` (seed
  solo-si-falta): gate apagado por defecto, opt-in documentado en `UPDATING.md`.
- Paridad sh vs ps1: las superficies PowerShell son un resumen conceptual, no
  copia literal; la ejecucion Windows real se valida cuando hay entorno.
- No correr `setup_harness.sh` en este checkout fuente (el marker de layout
  apuntaria a `$HOME`): el binario raiz se refresca con `cargo build` + `cp`.
