# Evidencia de implementacion - Feature #15: atlassian_binding_and_outbox

Spec: `docs/spec-feature-15-atlassian-binding-and-outbox.md` (Estado: approved,
sello 2026-08-16T03:15:43Z)
Plan: `docs/plan-feature-15-atlassian-binding-and-outbox.md`

## Que se construyo

- `rust/src/atlassian/` — modulo nuevo: `binding` (atlassian.json), `state`
  (mapa local -> remoto), `outbox` (intents), `emit` (enganches del flujo),
  `http` (cliente ureq + credenciales + base64 propio), `jira` (plataforma v3 +
  Agile 1.0), `confluence` (v2) y `markdown` (Markdown -> storage).
- `rust/src/commands/atlassian.rs` — `bind`, `status`, `drain`, `ack`, `apply`,
  `sprint start|close`, `publish`.
- Enganches en `commands/{add,start,advance,approve_spec,close}.rs`.
- `setup_harness.sh` / `setup_harness.ps1` — flags `--atlassian-site`,
  `--jira-project`, `--confluence-space`, `--jira-issue-type` (+ variables
  `HARNESS_*` del config file) y escritura de `atlassian.json`.
- `docs/atlassian-integracion.md` (+ `templates/docs/`) y
  `docs/adr/ADR-0001-cliente-http-ureq.md`.

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 binding por flags | OK | `tests/setup_smoke.sh` (fixture `atlassian-flags`): `atlassian.json` con `"project_key": "ADR"` y `"space_key": "SD"`. Tambien por CLI: test `atlassian_bind_should_write_the_binding_and_status_should_show_it` |
| AC-2 binding por config | OK | `tests/setup_smoke.sh` (fixture `atlassian-config`): `.harness.env` con `HARNESS_JIRA_PROJECT=SCRUM` -> `"project_key": "SCRUM"`. La precedencia se resuelve DESPUES de `load_config_file` |
| AC-3 apagado sin config | OK | `tests/setup_smoke.sh` (fixture `atlassian-off`): no se escribe `atlassian.json`, el instalador no falla y loguea "sin binding (integracion apagada)" con el comando para activarla |
| AC-4 sin binding, cero cambios | OK | Test `atlassian_should_stay_invisible_without_binding` (add/start/advance sin crear `progress/atlassian`), unit `hooks_should_do_nothing_without_binding`, unit `emit_best_effort_should_not_panic_on_unwritable_dir` y assert del smoke |
| AC-5 no adivina proyecto/space | OK | Test `atlassian_bind_should_refuse_to_guess_the_project`: exit 2 + "no lo voy a adivinar" + "Preguntale al USUARIO" |
| AC-6 intents en `add` | OK | **Verificacion real**: `add` genero `0001` (epic del PRD) y `0002` (historia). En Jira: **ADR-1** (Epic) y **ADR-2** (Story, parent ADR-1) |
| AC-7 subtasks por AC-n | OK | **Verificacion real**: los tres AC del spec bajaron como **ADR-3**, **ADR-4** y **ADR-5**, subtasks de ADR-2, con el formato `AC-n · <texto>`. Ver nota (1) sobre el momento de emision |
| AC-8 bitacora del otro lado | OK | **Verificacion real** en ADR-2: transicion a **In Progress** (`start`), comentario del `advance` (comment 10000), comentario de la aprobacion del spec (10001), transicion a **Done** y comentario de cierre (10002) tras `close --status done` |
| AC-9 `drain` propone sin mutar | OK | Test `flow_should_emit_intents_and_drain_should_plan_them_in_order` (mismo `pending` al correrlo dos veces; epic antes que historia antes que subtasks) y unit `pending_should_sort_by_dependency_then_id`. En la corrida real, `0002` mostro `needs` hasta que ADR-1 tuvo clave |
| AC-10 `ack` registra y archiva | OK | **Verificacion real**: 10 acks (`0001`..`0010`); `state.json` quedo con `PRD master -> ADR-1`, `feature #1 -> ADR-2 (3 subtasks)`. Test `ack_should_record_the_key_and_dedupe_the_next_run` cubre archivado en `applied/` y ack repetido inofensivo |
| AC-11 dedupe | OK | Units `emit_should_write_one_intent_and_dedupe_the_second`, `emit_should_skip_when_state_already_has_the_key`, `mark_applied_should_be_idempotent`, `comment_keys_should_differ_by_body_and_repeat_by_content` |
| AC-12 `status` | OK | **Verificacion real**: binding, `Token: ausente`, mapeo completo y `Intents pendientes: 0` al terminar el ciclo |
| AC-13 paridad ps1 | PARCIAL (documentado) | `Write-AtlassianBinding` + parametros `-AtlassianSite/-JiraProject/-ConfluenceSpace/-JiraIssueType`; asserts de contenido en `tests/setup_smoke.sh` y bloque nuevo en `tests/setup_smoke.ps1`. **No ejecutado**: no hay PowerShell en esta maquina (mismo limite aceptado en #1, #13 y #14) |
| AC-14 comandos oficiales | OK | `cargo test`: 116 unit + 34 integracion en verde; `cargo clippy --all-targets -- -D warnings` limpio; `bash tests/setup_smoke.sh` exit 0 con el bloque nuevo |
| AC-15 `apply` con token | OK | **Verificacion real**: `apply` ejecuto 5 de 5 intents contra la API y creo **ADR-6** (Story), **ADR-7** y **ADR-8** (subtasks AC-1/AC-2), la transicion a In Progress y el comentario de la aprobacion |
| AC-16 token nunca se imprime | OK | `status` solo dice presente/ausente (verificado en la corrida real); unit `credentials_should_come_from_harness_env_file` comprueba que el token solo viaja en el header |
| AC-17 errores accionables | OK | **Verificacion real** con un binding a un proyecto inexistente: `HTTP 400: The target project doesn't exist or you don't have permission to create issues in it.`, exit 1, los 2 intents quedaron pendientes y el mensaje dice que hacer. Units: `summarize_error_should_prefer_atlassian_messages`, `into_json_should_map_status_to_error` |
| AC-18 `apply` sin token | OK | Test `apply_should_refuse_without_token_and_point_to_the_agent_route`: exit 2 nombrando `atlassian drain` y `HARNESS_ATLASSIAN_TOKEN` |
| AC-19 crear sprint | OK | **Verificacion real**: `sprint start --name "Sprint arnes #15" --days 14` resolvio el board 2 del proyecto ADR, creo el sprint **#14** y lo dejo activo |
| AC-20 historia al sprint | OK | **Verificacion real**: ADR-6 se creo con el sprint vigente y quedo dentro de el (lo confirma el reporte de cierre del sprint, que lo lista) |
| AC-21 cerrar sprint | OK | **Verificacion real**: `sprint close` cerro el #14 y reporto `Quedaron sin terminar 1: ADR-6 [In Progress]` |
| AC-22 arbol de paginas | OK | **Verificacion real** en el space **SD**: PRD-master (721024), SDD-master (557103) y los dos specs (721045, 393286) |
| AC-23 idempotencia | OK | **Verificacion real**: segunda corrida sin cambios -> `= (sin cambios)` en los cuatro documentos, sin version nueva; tras editar el spec-2 -> `~ pagina 393286 (v2)` |
| AC-24 enlaces cruzados | OK | **Verificacion real**: la pagina del spec abre con `> Historia en Jira: .../browse/ADR-6` y ADR-6 tiene el comentario `Documento en Confluence: docs/spec-feature-2-conciliacion-bancaria.md - https://calpil.atlassian.net/wiki/spaces/SD/pages/393286/...` |
| AC-25 ciclo real | OK | **Las dos rutas** verificadas de punta a punta en `calpil.atlassian.net`: la del agente con MCP (feature #1, ADR-1..ADR-5) y la REST con token (feature #2, ADR-6..ADR-8, sprint #14 y las 4 paginas del space SD) |

## Verificacion real (ruta con agente MCP)

Repo fixture con binding a `ADR`, ciclo `add -> start -> approve-spec ->
advance -> close --status done`, drenado con el MCP de Atlassian y confirmado
por JQL. Claves creadas en `calpil.atlassian.net` (borrables si molestan):

| Clave | Tipo | Padre | Estado final |
| --- | --- | --- | --- |
| ADR-1 | Epic (PRD maestro) | - | To Do |
| ADR-2 | Story (`#1 flujo_de_cobranza`) | ADR-1 | **Done** |
| ADR-3 | Subtask `AC-1 · ...` | ADR-2 | To Do |
| ADR-4 | Subtask `AC-2 · ...` | ADR-2 | To Do |
| ADR-5 | Subtask `AC-3 · ...` | ADR-2 | To Do |

Mas 3 comentarios en ADR-2 (avance, aprobacion del spec y cierre) y las dos
transiciones (In Progress, Done). Al terminar: `Intents pendientes: 0`.

## Verificacion real (ruta REST con token)

Segunda feature del mismo fixture (`conciliacion_bancaria`), ejecutada entera
por `atlassian apply` con las credenciales en `.harness.env`:

| Clave / id | Que es |
| --- | --- |
| ADR-6 | Story `#2 conciliacion_bancaria` (In Progress, dentro del sprint #14) |
| ADR-7 / ADR-8 | Subtasks `AC-1 · ...` y `AC-2 · ...` |
| sprint #14 | `Sprint arnes #15` en el board 2: creado, activado y cerrado |
| 721024 / 557103 | Paginas de PRD-master y SDD-master en el space SD |
| 721045 / 393286 | Paginas de los dos specs (la segunda actualizada a v2) |

## Mejoras que salieron de preguntas del usuario (misma feature)

- **Credenciales globales**: `Credentials::discover` ahora mira los MISMOS
  archivos que el instalador y en el mismo orden — `.harness.env` del proyecto y
  del arnes, y despues `~/.config/harness/config` y `~/.harnessrc`. Antes solo
  miraba `.harness.env`, asi que un token global (una sola vez para todos los
  repos) no habria funcionado para `apply`. Unit:
  `config_files_should_match_the_installer_order`.
- **`.harness.env` sembrado y protegido**: el instalador lo deja en la raiz del
  proyecto con las claves comentadas y explicadas (`seed_harness_env` /
  `Initialize-HarnessEnvTemplate`), NUNCA lo pisa si ya existe (puede tener el
  token real) y lo agrega al `.gitignore` del proyecto aunque el archivo ya
  existiera — Articulo 4. Asserts en `tests/setup_smoke.sh`.

## Notas

1. **Momento de emision de las subtasks.** `start` genera el spec como
   plantilla, asi que al arrancar todavia no hay AC-n que bajar. Las subtasks se
   emiten tambien en `approve-spec` — cuando el spec ya declara sus AC-n y el
   USUARIO lo aprobo — y el dedupe por `ac:<fid>:<AC-n>` evita duplicados si el
   spec ya venia escrito al arrancar. El Given del AC-7 ("una feature con spec
   que declara AC-1..AC-n") se cumple igual.
2. **Titulo del epic en el fixture.** Salio como `PRD Master - <nombre del
   proyecto>` (con los signos escapados por la API) porque el fixture usa la
   plantilla del PRD sin completar. En un proyecto real el epic toma el titulo
   real del PRD.
3. **Dependencia nueva.** `ureq` (+ `json`) entra con el ADR-0001 que exige el
   Articulo 6. `base64` NO se sumo: se implemento en `http.rs` con los vectores
   del RFC 4648 como test.
4. **`harness_check.sh`** reporta el `graphify-out/.graphify_stale` heredado del
   cierre de la feature #14 (decision pendiente del usuario sobre el refresh);
   no lo introduce esta feature.
