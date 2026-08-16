# Evidencia de implementacion - Feature #16: atlassian_auto_push

Spec: `docs/spec-feature-16-atlassian-auto-push.md` (Estado: approved, 29 AC)
Plan: `docs/plan-feature-16-atlassian-auto-push.md`

## Que se construyo

- `rust/src/atlassian/push.rs` — lanzador detached (`push_bg`) con lock, worker
  (`atlassian-worker`, subcomando oculto) que corre backfill + apply + publish y
  deja `progress/atlassian/last-push.log`, e interruptor de tres niveles
  (`should_push` / `should_push_with`).
- Disparo en las SEIS transiciones: `add`, `start`, `advance`, `approve-spec`,
  `close` y `prd add` (este ultimo tambien emite el epic de su PRD).
- `add --kind feature|bug|task` con validacion, campo `kind` opcional en el
  backlog y mapeo a `issue_types.bug` / `issue_types.task`.
- `atlassian backfill [--sin-acs]` y backfill automatico en el primer envio.
- Verificacion del binding contra la API (`verify_binding`) usada por
  `atlassian bind`, por `atlassian status` y por los dos instaladores, con
  creacion opcional (`--create-project` / `--create-space`, y
  `--create-jira-project` / `--create-confluence-space` en el instalador).
- Adopcion de epics existentes por titulo (`jira::find_epic_by_title`).

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 envio automatico | OK | **Real**: `add` en el fixture disparo el worker, que aplico 2 intents -> **ADR-11** (epic) y **ADR-12** (historia) sin ningun comando manual |
| AC-2 exit codes intactos | OK | Test `transitions_should_keep_their_exit_codes_with_auto_push_on` (add/start/close con credenciales falsas siguen saliendo 0 con su salida de siempre) + unit `push_bg_should_never_panic_without_binding` |
| AC-3 `prd add` crea su epic | OK | Test `prd_add_should_emit_its_epic_without_waiting_for_a_feature` |
| AC-4 lock | OK | Unit `push_bg_should_do_nothing_when_locked` (el lock ajeno queda intacto y no se crea el log) |
| AC-4b segunda pasada | OK | Implementado en `push::worker` (bucle de dos pasadas); en la corrida real la segunda pasada no encontro nada nuevo y no repitio trabajo |
| AC-5 log del ultimo envio | OK | **Real**: `progress/atlassian/last-push.log` con timestamp, backfill, intents aplicados y publicacion; `status` muestra la linea `Ultimo push` |
| AC-6 publish en cada transicion | OK | **Real**: el mismo `add` publico `PRD-master` (pagina 98532, nueva) y actualizo `SDD-master` (v2) |
| AC-7 una version por cambio | OK | **Real** (heredado y re-verificado de la #15): documento sin cambios -> `= (sin cambios)`; documento editado -> `~ (v2)` |
| AC-8 `--kind bug` | OK | **Real**: `add --kind bug` creo **ADR-13** con `issuetype: Bug` bajo el epic; test `add_kind_should_be_optional_and_map_to_the_right_issue_type` |
| AC-9 sin `--kind` no cambia nada | OK | Mismo test: la feature sin `--kind` no gana el campo y va como `Story` |
| AC-10 `--kind` invalido | OK | Test `add_should_reject_an_invalid_kind_before_touching_the_backlog`: exit 2, lista de validos y backlog intacto |
| AC-11 tipo inexistente en el proyecto | OK | Cubierto por el mismo camino de error de la #15 (HTTP 400 legible, intent pendiente, el resto se aplica) |
| AC-12 sin token | OK | Test `auto_push_should_be_reported_and_switchable` (`Auto push: apagado (sin token...)`) + assert en `tests/setup_smoke.sh` |
| AC-13 `"auto": false` | OK | Unit `should_skip_when_binding_disables_auto` |
| AC-14 `HARNESS_ATLASSIAN_AUTO=0` | OK | Test de integracion (status reporta el apagado por entorno) |
| AC-15 sin binding, cero cambios | OK | Test `atlassian_should_stay_invisible_without_binding` (de la #15, sigue verde) + unit `should_skip_without_binding` |
| AC-16 comandos oficiales | OK | `cargo test`: 123 unit + 43 integracion = **166**; `clippy --all-targets -- -D warnings` limpio; `setup_smoke.sh` exit 0 |
| AC-17 ciclo real sin comandos manuales | OK | **Real**: ver la seccion siguiente |
| AC-18 verificacion al configurar | OK | **Real**: `bind` con token -> `verificacion: proyecto Jira ADR OK` / `space de Confluence SD OK`; sin token -> `verificacion: omitida` (test `bind_should_report_that_it_cannot_verify_without_token`) |
| AC-19 mensaje cuando falta | OK | **Real**: con `NOEXISTE`/`FANTASMA` -> `[!] el proyecto Jira 'NOEXISTE' no existe...` + la URL para crearlo y la mencion de `--create-project` / `--create-space` |
| AC-20 `status` valida | OK | `status` corre `verify_binding` cuando hay token y no cambia su exit code |
| AC-21/AC-22 creacion con flag | OK (implementado) | `jira::create_project` (software / scrum team-managed / lead = cuenta del token) y `confluence::create_space`; **no ejecutados en real**: crearia estructura organizacional en el sitio del usuario sin necesidad |
| AC-23 sin permisos | OK | El 403 de Atlassian sube tal cual por `ApiError` y el binding igual queda escrito |
| AC-24 backfill | OK | Test `backfill_should_load_prds_and_backlog_without_touching_the_network` (epic del PRD + historia por feature + transicion de estado de la feature en curso) |
| AC-25 idempotente | OK | Test `backfill_should_be_idempotent_and_respect_sin_acs` (sin claves de dedupe repetidas) |
| AC-26 comando explicito | OK | `atlassian backfill` + test `backfill_should_refuse_without_binding` (exit 2 sin binding) |
| AC-27 estado de lo cerrado | OK | `emit::on_backfill_status` mapea `done` -> Done, `in_progress` -> In Progress y `blocked` -> flag; cubierto por el test de backfill |
| AC-28 `--sin-acs` | OK | Mismo test: con el flag no aparecen las subtasks; sin el flag, si |
| AC-29 adopta epics existentes | OK | **Real**: un segundo repo con el mismo titulo de PRD imprimio `(epic existente adoptado: ADR-11)` y colgo su historia (ADR-14) del epic que ya estaba |
| AC-30 crea si no existe | OK | **Real**: el primer fixture creo ADR-11 porque no habia ninguno con ese titulo |

## Verificacion real (AC-17)

Fixture con binding a `ADR` y token en `~/.config/harness/config`. **No se corrio
ni un solo comando de Atlassian a mano**:

```
$ ./harness add --name cobranza_automatica --acceptance "..."
Feature #1 agregada.

$ cat progress/atlassian/last-push.log
== Atlassian push 2026-08-16T05:20:43Z ==
[backfill] primer envio: cargando PRDs y backlog existentes
[Atlassian] backfill: 0 intent(s) nuevos en la outbox.
[pasada 1] 2 intent(s) pendiente(s)
[Atlassian] epic del PRD master -> ADR-11
[Atlassian] historia de la feature #1 (cobranza_automatica) -> ADR-12
[Atlassian] aplicados 2 de 2 intents.
[Atlassian] publicando en el space SD (calpil.atlassian.net)
  + docs/prd/PRD-master.md -> pagina 98532 (nueva)
  ~ docs/prd/SDD-master.md -> pagina 557103 (v2)
== fin 2026-08-16T05:20:50Z ==
```

Despues: `add --kind bug` -> **ADR-13** con tipo `Bug`; y en OTRO repo con el
mismo PRD, `(epic existente adoptado: ADR-11)` -> **ADR-14** colgada del mismo
epic. Todos los issues de prueba (ADR-1..ADR-17) se borraron al terminar: el
proyecto ADR quedo en 0, como estaba.

## Dos defectos que encontraron los tests (y se corrigieron)

1. **Los tests hablaban con la API real.** Al existir `~/.config/harness/config`
   con el token, los tests de integracion y el `setup_smoke.sh` tomaban las
   credenciales de la maquina: el smoke llego a crear issues de verdad en ADR.
   Corregido: `tests/cli_basics.rs` aisla `HOME` y limpia las variables en su
   helper `cmd()`, y el smoke ejecuta el binario solo a traves de
   `harness_bin()`, con `HOME` de sandbox y `HARNESS_ATLASSIAN_AUTO=0`. Los
   issues creados por accidente se borraron.
2. **Los flags de creacion del instalador nunca hubieran funcionado.** Un
   replace duplico el bloque de defaults DESPUES del parseo de argumentos, asi
   que `CREATE_JIRA_PROJECT` volvia a 0 justo antes de usarse. Lo detecto el
   assert nuevo del smoke; se elimino el bloque duplicado.

## Nota sobre el aislamiento (decision de diseno)

`push::should_push` se partio en dos: la version publica consulta las
credenciales y `should_push_with(paths, has_token)` recibe ese dato. Sin eso,
los tests dependian de si la maquina que corre la suite tiene token configurado
— que es exactamente el bug (1) de arriba.
