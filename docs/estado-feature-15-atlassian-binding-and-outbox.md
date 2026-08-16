# Estado archivado - Feature #15: atlassian_binding_and_outbox
Cerrada: 2026-08-16T04:16:21Z - status=done - Integracion Atlassian completa: binding por repo desde el instalador, outbox de intents en cada transicion del flujo, dos ejecutores (agente con MCP y REST con token), sprints via Agile API y publicacion de PRD/SDD/specs en Confluence. Verificado de punta a punta en calpil.atlassian.net (ADR-1..ADR-8, sprint #14, 4 paginas en SD)

---

# Feature #15: atlassian_binding_and_outbox

Estado: in_progress
Plan: docs/plan-feature-15-atlassian-binding-and-outbox.md
Spec: docs/spec-feature-15-atlassian-binding-and-outbox.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-16T03:51:28Z D1-D8 implementados: binding en sh/ps1 (atlassian.json), modulo atlassian (binding/state/outbox/emit/http/jira/confluence/markdown), comando atlassian bind|status|drain|ack|apply|sprint|publish, enganches en add/start/advance/approve-spec/close, ADR-0001 (ureq), docs/atlassian-integracion.md y superficies. Tests: 116 unit + 34 integracion + smoke sh con asserts de binding; clippy limpio
