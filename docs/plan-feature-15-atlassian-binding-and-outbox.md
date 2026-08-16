# Plan - Feature #15: atlassian_binding_and_outbox

Estado: in_progress
Microservicios:
- harness

## Alcance

Ensenarle al arnes a que proyecto Jira y a que space de Confluence pertenece
cada repo donde se instala, y hacer que cada transicion del flujo deje su
rastro del otro lado sin romper a quien no lo configure. Por decision del
USUARIO (OBS-5) entra TODO en esta feature: binding en los dos instaladores,
`atlassian.json`, outbox de intents, estado local con el mapeo, subcomandos
`atlassian bind|status|drain|ack|apply|sprint|publish`, enganches en
add/start/advance/approve-spec/close, ejecutor REST con token (Jira platform
v3), sprints via Agile 1.0, publicacion de PRD/SDD/specs en Confluence v2, la
guia de drenaje para agentes con MCP y la documentacion en las superficies.

## Impacto entre microservicios

Un solo microservicio: `harness`. El cambio es aditivo y con interruptor — sin
`atlassian.json` ningun camino nuevo se ejecuta (AC-4), asi que los repos ya
instalados no cambian de comportamiento hasta que corran el instalador con los
flags nuevos. El hub PostgreSQL no se toca: el mapeo remoto vive en
`progress/atlassian/state.json`, no en el grafo.

## Consulta al grafo (graphify)

Pendiente de la decision del USUARIO sobre el refresh de `graphify-out`
(el marcador `.graphify_stale` sigue puesto desde el cierre de la #14). No
bloquea: el alcance esta acotado a rutas conocidas (`rust/src/commands/`,
`rust/src/`, `setup_harness.*`, `templates/`, `tests/`).

## Delegacion (implementer)

- D1 [AC-1, AC-2, AC-3, AC-13]: flags `--atlassian-site`, `--jira-project` y
  `--confluence-space` en `setup_harness.sh` con precedencia flag > config >
  nada, escritura de `atlassian.json` en la raiz del proyecto, mensaje explicito
  cuando queda apagado, y paridad literal en `setup_harness.ps1`.
- D2 [AC-4, AC-5, AC-12]: modulo `atlassian` en el CLI Rust con `bind` (se
  niega con exit 2 si no sabe proyecto o space, nombrando la pregunta para el
  USUARIO), `status` (binding + token presente/ausente + mapeo + pendientes +
  sprint vigente) y lectura tolerante del binding ausente.
- D3 [AC-6, AC-7, AC-8, AC-11]: emision de intents en add/start/advance/
  approve-spec/close, con clave de dedupe por intent y escritura best-effort
  que jamas cambia el exit code del comando del flujo.
- D4 [AC-9, AC-10]: `drain` (plan de llamadas MCP en JSON, ordenado por
  dependencia, sin mutar) y `ack` (guarda la clave remota, marca aplicado,
  idempotente ante repeticion).
- D5 [AC-15, AC-16, AC-17, AC-18]: ejecutor REST con Basic auth (email + API
  token desde `.harness.env`), Jira platform v3 para crear issues, transicionar
  y comentar; errores accionables con el codigo y el mensaje de Atlassian;
  token jamas impreso ni persistido. Depende de OBS-9.
- D6 [AC-19, AC-20, AC-21]: sprints via Agile 1.0 — resolver board del
  proyecto, crear sprint, activarlo, mover historias en lotes de 50, cerrarlo
  reportando lo no terminado.
- D7 [AC-22, AC-23, AC-24]: `publish` a Confluence v2 — arbol PRD maestro ->
  PRDs anidados -> specs, SDD como hermana, idempotencia por titulo + hash y
  actualizacion con `version.number + 1`, enlaces cruzados pagina <-> issue.
  Depende de OBS-10.
- D8 [AC-14]: tests unitarios (binding ausente/presente, dedupe, ack repetido,
  orden de dependencia en drain, mapeo de error HTTP, idempotencia de publish)
  e integracion en `tests/cli_basics.rs`, mas assert de contenido en
  `tests/setup_smoke.sh` y su paridad ps1.
- D9 [AC-25]: verificacion real contra `calpil.atlassian.net` con un repo
  fixture, dejando claves e ids en `docs/impl-15.md`.
- D10 [AC-1..AC-25]: documentacion — README (seccion Atlassian), UPDATING.md y
  su espejo en `templates/UPDATING.md`, AGENTS.md y superficies espejo, guia de
  drenaje MCP en `docs/`, y el ADR de la dependencia si OBS-9 resuelve (a).

## Criterios de cierre (reviewer)

- Evidencia por AC-n en `docs/impl-15.md`, con AC-13 verificado por lectura +
  assert (no hay PowerShell en esta maquina, mismo limite aceptado en #1, #13 y
  #14).
- `cargo test`, `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh` limpios (el `.graphify_stale` preexistente se reporta
  aparte: no lo introduce esta feature).
- Ciclo completo verificado en el sitio real (AC-25), con las claves creadas.
- Constitution: Articulo 2 (spec approved antes de implementar), Articulo 4
  (token solo por entorno, nunca en repo/logs/commits), Articulo 6 (ADR si
  entra dependencia nueva; espejos `templates/` propagados).

## Riesgos

- R1: emitir intents dentro de comandos que hoy no fallan nunca podria
  romperlos. Mitigacion: escritura best-effort, envuelta, sin afectar exit code
  (AC-4) y con test que lo prueba.
- R2: divergencia entre el mapeo local y la realidad de Jira si alguien borra
  un issue a mano. Mitigacion: `status` muestra el mapeo y `ack`/`apply` son
  las unicas puertas de escritura; la reconciliacion bidireccional queda fuera
  de alcance.
- R3: el binding versionado podria filtrar informacion sensible. Mitigacion:
  solo lleva nombres de proyecto y space; el token vive en `.harness.env`.
- R4: la conversion Markdown -> storage de Confluence puede degradar documentos
  largos. Mitigacion: subconjunto acotado + enlace al archivo del repo como
  fuente de verdad (OBS-10), y hash para no reescribir lo que no cambio.
- R5: pruebas contra el sitio real crean issues de verdad. Mitigacion: usar un
  repo fixture y dejar registrado en `impl-15.md` que claves se crearon, para
  poder limpiarlas.
- R6: la ventana de transicion del hub (#14) sigue abierta y `start` ya mostro
  un `statement timeout`. No afecta esta feature (falla legible, no cuelga).

## Observaciones (decisiones pendientes)

- OBS-1 a OBS-8 [DECIDIDAS]: ver el spec (hibrido, sprints con token, binding
  en el instalador, space por repo, todo junto en la #15, `Story` por default,
  `blocked` como flag Impediment, convencion Epic -> Story -> Subtask ya usada
  en SCRUM).
- OBS-9 [DECIDIDA, 2026-08-15]: `ureq` como cliente HTTP + ADR que lo
  justifica (Articulo 6). Habilita D5, D6 y D7.
- OBS-10 [DECIDIDA, 2026-08-15]: conversion de un subconjunto de Markdown a
  `storage` HTML, con enlace al archivo del repo como fuente de verdad.
  Habilita D7.

### Avance 2026-08-16T03:51:28Z
D1-D8 implementados: binding en sh/ps1 (atlassian.json), modulo atlassian (binding/state/outbox/emit/http/jira/confluence/markdown), comando atlassian bind|status|drain|ack|apply|sprint|publish, enganches en add/start/advance/approve-spec/close, ADR-0001 (ureq), docs/atlassian-integracion.md y superficies. Tests: 116 unit + 34 integracion + smoke sh con asserts de binding; clippy limpio

---
Cerrado: 2026-08-16T04:16:21Z - status=done - Integracion Atlassian completa: binding por repo desde el instalador, outbox de intents en cada transicion del flujo, dos ejecutores (agente con MCP y REST con token), sprints via Agile API y publicacion de PRD/SDD/specs en Confluence. Verificado de punta a punta en calpil.atlassian.net (ADR-1..ADR-8, sprint #14, 4 paginas en SD)
