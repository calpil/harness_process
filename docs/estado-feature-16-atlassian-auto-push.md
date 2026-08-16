# Estado archivado - Feature #16: atlassian_auto_push
Cerrada: 2026-08-16T05:36:40Z - status=done - Envio automatico completo: worker detached en las seis transiciones (apply + publish), backfill del repo existente con adopcion de epics, add --kind bug/feature/task, verificacion del binding con creacion opcional e interruptor de tres niveles. Verificado en real: un add creo epic, historia y paginas sin comandos manuales

---

# Feature #16: atlassian_auto_push

Estado: in_progress
Plan: docs/plan-feature-16-atlassian-auto-push.md
Spec: docs/spec-feature-16-atlassian-auto-push.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-16T05:16:22Z Alan aprobo en el chat el spec AMPLIADO de la #16 (29 AC): se sumaron validacion del binding (bind + status), creacion de proyecto/space solo con --create-project/--create-space, backfill completo de Jira en el primer push (OBS-12 reemplaza OBS-6), sincronia total incluyendo subtasks de features cerradas (OBS-14) y reutilizacion de epics existentes por titulo (OBS-15)
