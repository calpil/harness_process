# Graph Report - harness_process  (2026-06-11)

## Corpus Check
- 85 files · ~36,474 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 582 nodes · 943 edges · 72 communities (49 shown, 23 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 80 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2f265451`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Community 49|Community 49]]
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 51|Community 51]]
- [[_COMMUNITY_Community 52|Community 52]]
- [[_COMMUNITY_Community 53|Community 53]]
- [[_COMMUNITY_Community 54|Community 54]]
- [[_COMMUNITY_Community 55|Community 55]]
- [[_COMMUNITY_Community 56|Community 56]]
- [[_COMMUNITY_Community 57|Community 57]]
- [[_COMMUNITY_Community 58|Community 58]]
- [[_COMMUNITY_Community 59|Community 59]]
- [[_COMMUNITY_Community 61|Community 61]]
- [[_COMMUNITY_Community 62|Community 62]]
- [[_COMMUNITY_Community 63|Community 63]]
- [[_COMMUNITY_Community 64|Community 64]]
- [[_COMMUNITY_Community 65|Community 65]]
- [[_COMMUNITY_Community 66|Community 66]]
- [[_COMMUNITY_Community 67|Community 67]]
- [[_COMMUNITY_Community 68|Community 68]]
- [[_COMMUNITY_Community 69|Community 69]]
- [[_COMMUNITY_Community 70|Community 70]]
- [[_COMMUNITY_Community 71|Community 71]]
- [[_COMMUNITY_Community 74|Community 74]]
- [[_COMMUNITY_Community 75|Community 75]]

## God Nodes (most connected - your core abstractions)
1. `setup_harness.sh script` - 31 edges
2. `Write-HarnessLog()` - 16 edges
3. `run()` - 15 edges
4. `PgGraphStore` - 15 edges
5. `run()` - 14 edges
6. `run()` - 14 edges
7. `plan_path()` - 14 edges
8. `inner()` - 13 edges
9. `load_features()` - 13 edges
10. `is_plan_stale()` - 13 edges

## Surprising Connections (you probably didn't know these)
- `run()` --calls--> `py_str()`  [INFERRED]
  rust/src/commands/add.rs → rust/src/pycompat.rs
- `run()` --calls--> `plan_path()`  [INFERRED]
  rust/src/commands/advance.rs → rust/src/plan.rs
- `run()` --calls--> `update_plan_sig()`  [INFERRED]
  rust/src/commands/advance.rs → rust/src/plan.rs
- `run()` --calls--> `py_str()`  [INFERRED]
  rust/src/commands/advance.rs → rust/src/pycompat.rs
- `inner()` --calls--> `update_plan_sig()`  [INFERRED]
  rust/src/commands/autocheck.rs → rust/src/plan.rs

## Import Cycles
- 1-file cycle: `rust/src/commands/add.rs -> rust/src/commands/add.rs`
- 1-file cycle: `rust/src/commands/advance.rs -> rust/src/commands/advance.rs`
- 1-file cycle: `rust/src/features.rs -> rust/src/features.rs`
- 1-file cycle: `rust/src/commands/autocheck.rs -> rust/src/commands/autocheck.rs`
- 1-file cycle: `rust/src/commands/check_plan.rs -> rust/src/commands/check_plan.rs`
- 1-file cycle: `rust/src/commands/close.rs -> rust/src/commands/close.rs`
- 1-file cycle: `rust/src/commands/next.rs -> rust/src/commands/next.rs`
- 1-file cycle: `rust/src/commands/nudge.rs -> rust/src/commands/nudge.rs`
- 1-file cycle: `rust/src/commands/start.rs -> rust/src/commands/start.rs`
- 1-file cycle: `rust/src/commands/status.rs -> rust/src/commands/status.rs`
- 1-file cycle: `rust/src/graph/mod.rs -> rust/src/graph/mod.rs`
- 1-file cycle: `rust/src/memories.rs -> rust/src/memories.rs`
- 1-file cycle: `rust/src/graph/ids.rs -> rust/src/graph/ids.rs`
- 1-file cycle: `rust/src/graph/store.rs -> rust/src/graph/store.rs`
- 1-file cycle: `rust/src/graph/tls.rs -> rust/src/graph/tls.rs`
- 1-file cycle: `rust/src/graphify.rs -> rust/src/graphify.rs`
- 1-file cycle: `rust/src/main.rs -> rust/src/main.rs`
- 1-file cycle: `rust/src/paths.rs -> rust/src/paths.rs`
- 1-file cycle: `rust/src/plan.rs -> rust/src/plan.rs`
- 1-file cycle: `rust/src/progress.rs -> rust/src/progress.rs`

## Communities (72 total, 23 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.06
Nodes (62): run(), run(), inner(), run(), run(), run(), run(), run() (+54 more)

### Community 1 - "Community 1"
Cohesion: 0.33
Nodes (5): Archivos modificados / eliminados, Cierre, Decisiones, Implementacion - Feature #2: remove_python_full_rust_migration, Verificacion ejecutada

### Community 2 - "Community 2"
Cohesion: 0.40
Nodes (4): Criterios checklist (del plan), Review - Feature #2: remove_python_full_rust_migration, Revision, Riesgo residual

### Community 3 - "Community 3"
Cohesion: 0.17
Nodes (38): acquire_lock(), archive_legacy_file(), archive_local_hub_memory(), backup_file(), backup_path(), build_claude_agent(), build_codex_agent(), build_gemini_agent() (+30 more)

### Community 6 - "Community 6"
Cohesion: 0.13
Nodes (22): inner(), run(), HarnessPaths, Result, Option, Path, PathBuf, Result (+14 more)

### Community 7 - "Community 7"
Cohesion: 0.27
Nodes (23): HarnessPaths, Map, Option, Path, PathBuf, Result, String, Value (+15 more)

### Community 8 - "Community 8"
Cohesion: 0.21
Nodes (11): Client, Config, PgGraphStore, IndexMap, Map, Option, Result, Self (+3 more)

### Community 9 - "Community 9"
Cohesion: 0.17
Nodes (14): CertificateDer, DigitallySignedStruct, make_connector(), NoVerifier, HandshakeSignatureValid, MakeRustlsConnect, Error, Result (+6 more)

### Community 10 - "Community 10"
Cohesion: 0.20
Nodes (15): FnOnce, GraphEnv, GraphMemoryManager, nonempty(), run(), usage_error(), GraphCommand, PgGraphStore (+7 more)

### Community 11 - "Community 11"
Cohesion: 0.29
Nodes (15): Command, Path, PathBuf, TempDir, add_should_create_feature_and_next_should_print_python_style_json(), check_plan_should_exit_one_without_active_feature(), check_plan_should_exit_two_when_plan_edited_by_another_agent(), close_should_archive_current_state_and_reset_it() (+7 more)

### Community 12 - "Community 12"
Cohesion: 0.18
Nodes (9): Display, Formatter, Into, Error, Option, Result, Self, String (+1 more)

### Community 13 - "Community 13"
Cohesion: 0.26
Nodes (6): Path, PathBuf, Result, Self, HarnessPaths, repo_root_from_marker()

### Community 14 - "Community 14"
Cohesion: 0.14
Nodes (11): GraphMemoryManager, GraphMemoryManager, artifact_id(), artifact_id_should_match_python_shape(), is_repo_root(), qualify(), Option, Result (+3 more)

### Community 15 - "Community 15"
Cohesion: 0.20
Nodes (9): Command, Option, Result, String, Cli, Command, GraphCommand, GraphOpts (+1 more)

### Community 17 - "Community 17"
Cohesion: 0.53
Nodes (8): Path, Result, graphify_available(), mark_stale(), refresh_bg(), refresh_sync(), run_graphify_update(), worker()

### Community 19 - "Community 19"
Cohesion: 0.29
Nodes (6): features, project, rules, one_feature_at_a_time, require_impact_check, require_tests_to_close

### Community 20 - "Community 20"
Cohesion: 0.29
Nodes (6): features, project, rules, one_feature_at_a_time, require_impact_check, require_tests_to_close

### Community 21 - "Community 21"
Cohesion: 0.11
Nodes (33): Archive-LegacyHub(), Assert-PostgresConfiguration(), Backup-HarnessPath(), Build-HarnessBinary(), ConvertTo-PowerShellCommandPath(), Ensure-Antigravity(), Ensure-Directory(), Ensure-Graphify() (+25 more)

### Community 22 - "Community 22"
Cohesion: 0.57
Nodes (5): copy_fixture(), copy_flat_fixture(), run_setup(), copy_fixture(), setup_smoke.sh script

### Community 23 - "Community 23"
Cohesion: 0.50
Nodes (3): { chromium }, fs, path

### Community 24 - "Community 24"
Cohesion: 0.83
Nodes (3): ExitCode, main(), restore_sigpipe()

### Community 25 - "Community 25"
Cohesion: 0.50
Nodes (3): { chromium }, fs, path

### Community 41 - "Community 41"
Cohesion: 0.18
Nodes (10): Alcance, Avance 2026-06-11T03:40:46Z, Avance 2026-06-11T04:04:26Z, Avance 2026-06-11T04:04:34Z, Consulta al grafo (graphify), Criterios de cierre (reviewer), Delegacion (implementer), Impacto entre microservicios (+2 more)

### Community 42 - "Community 42"
Cohesion: 0.18
Nodes (10): Actualización del Harness Process, Cuándo actualizar, Cómo actualizar, Cómo obtener este archivo si te falta al actualizar, Mantenimiento dual Rust + Python (obligatorio para maintainers), Mantenimiento Rust only (post feature #2), Para maintainers de este repositorio harness_process, Por qué funciona así (+2 more)

### Community 43 - "Community 43"
Cohesion: 0.18
Nodes (10): Actualización del Harness Process, Cuándo actualizar, Cómo actualizar, Cómo obtener este archivo si te falta al actualizar, Mantenimiento dual Rust + Python (obligatorio para maintainers), Mantenimiento Rust only (post feature #2), Para maintainers de este repositorio harness_process, Por qué funciona así (+2 more)

### Community 44 - "Community 44"
Cohesion: 0.25
Nodes (7): Actualizacion (proceso explicito), harness_cli: binario Rust + fallback Python, Harness Process, Instalacion, Opciones, Requisitos, Verificacion

### Community 45 - "Community 45"
Cohesion: 0.29
Nodes (6): Como se orquesta por herramienta, Flujo, Mapa de Agentes, Modelos, effort y tools por rol (tunable), Regla anti perdida de contexto, Roles

### Community 46 - "Community 46"
Cohesion: 0.29
Nodes (6): Como se orquesta por herramienta, Flujo, Mapa de Agentes, Modelos, effort y tools por rol (tunable), Regla anti perdida de contexto, Roles

### Community 47 - "Community 47"
Cohesion: 0.33
Nodes (5): Archivos modificados, Decisiones, Implementacion - Feature #1: powershell_windows_installer, Riesgos pendientes, Verificacion

### Community 48 - "Community 48"
Cohesion: 0.40
Nodes (4): Archivos principales, Mapa de Agentes, Orden de trabajo, Regla anti perdida de contexto

### Community 49 - "Community 49"
Cohesion: 0.40
Nodes (4): Implementer, Protocolo (OBLIGATORIO), Reglas, Reporte minimo (docs/impl-<feature>.md)

### Community 50 - "Community 50"
Cohesion: 0.40
Nodes (4): Entregable, Lider (planner), Protocolo, Reglas

### Community 51 - "Community 51"
Cohesion: 0.40
Nodes (4): Reglas, Reviewer, Veredicto (docs/review-<feature>.md), Verifica

### Community 52 - "Community 52"
Cohesion: 0.40
Nodes (4): Implementer, Protocolo (OBLIGATORIO), Reglas, Reporte minimo (docs/impl-<feature>.md)

### Community 53 - "Community 53"
Cohesion: 0.40
Nodes (4): Entregable, Lider (planner), Protocolo, Reglas

### Community 54 - "Community 54"
Cohesion: 0.40
Nodes (4): Reglas, Reviewer, Veredicto (docs/review-<feature>.md), Verifica

### Community 55 - "Community 55"
Cohesion: 0.40
Nodes (4): Implementer, Protocolo (OBLIGATORIO), Reglas, Reporte minimo (docs/impl-<feature>.md)

### Community 56 - "Community 56"
Cohesion: 0.40
Nodes (4): Entregable, Lider (planner), Protocolo, Reglas

### Community 57 - "Community 57"
Cohesion: 0.40
Nodes (4): Reglas, Reviewer, Veredicto (docs/review-<feature>.md), Verifica

### Community 58 - "Community 58"
Cohesion: 0.50
Nodes (3): Review - Feature #1: powershell_windows_installer, Revision, Riesgo residual

### Community 59 - "Community 59"
Cohesion: 0.40
Nodes (4): Estado Actual, Evidencia, Estado Actual, Evidencia

### Community 65 - "Community 65"
Cohesion: 0.40
Nodes (4): Feature #1: powershell_windows_installer, Feature #2: remove_python_full_rust_migration, Estado Actual, Evidencia

### Community 75 - "Community 75"
Cohesion: 0.18
Nodes (10): Alcance, Avance 2026-06-11T04:12:47Z, Avance 2026-06-11T04:15:21Z, Avance 2026-06-11T04:22:43Z, Consulta al grafo (graphify), Criterios de cierre (reviewer), Delegacion (implementer), Impacto entre microservicios (+2 more)

## Knowledge Gaps
- **190 isolated node(s):** `allow`, `commit_guard.sh script`, `{ chromium }`, `path`, `fs` (+185 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **23 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `py_str()` connect `Community 6` to `Community 0`, `Community 7`?**
  _High betweenness centrality (0.014) - this node is a cross-community bridge._
- **Why does `env_nonempty()` connect `Community 6` to `Community 10`, `Community 13`?**
  _High betweenness centrality (0.012) - this node is a cross-community bridge._
- **Why does `Exit` connect `Community 0` to `Community 10`?**
  _High betweenness centrality (0.010) - this node is a cross-community bridge._
- **Are the 12 inferred relationships involving `run()` (e.g. with `feature_mut()` and `feature_status()`) actually correct?**
  _`run()` has 12 INFERRED edges - model-reasoned connections that need verification._
- **Are the 10 inferred relationships involving `run()` (e.g. with `active_feature_index()` and `feature_mut()`) actually correct?**
  _`run()` has 10 INFERRED edges - model-reasoned connections that need verification._
- **What connects `allow`, `commit_guard.sh script`, `{ chromium }` to the rest of the system?**
  _190 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.0578386605783866 - nodes in this community are weakly interconnected._