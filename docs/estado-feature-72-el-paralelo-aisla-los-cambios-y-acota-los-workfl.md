# Estado archivado - Feature #72: El paralelo aisla los cambios y acota los workflows
Cerrada: 2026-09-05T03:48:04Z - status=done - Aislamiento verificable en start, rango completo en el cierre, Stop acotado a la sesion y cobertura de delegacion que no se puede fingir

---

# Feature #72: El paralelo aisla los cambios y acota los workflows

Estado: in_progress
Plan: docs/plan-feature-72-el-paralelo-aisla-los-cambios-y-acota-los-workfl.md
Spec: docs/spec-feature-72-el-paralelo-aisla-los-cambios-y-acota-los-workfl.md

Microservicios:
- harness

Evidencia:
- Diagnostico: progress/diagnostico-aviso-bug-report.md (trazas de sesiones, journals de workflows, hooks y commits).
- Worktree: /Users/alan/harness_process-wt/72-el-paralelo-aisla-los-cambios-y-acota-los-workfl
- Rama: bugfix/72-el-paralelo-aisla-los-cambios-y-acota-los-workfl
- Configuracion local de Claude aplicada: feedbackDrafts=quiet y workflowSizeGuideline=small. JSON valido; comparacion con respaldo confirma que no cambiaron otras claves.
- Respaldo: /Users/alan/.claude/backups/settings.before-parallel-fix-20260904.json. No se borraron ni enviaron drafts; no se detuvieron sesiones. Falta confirmar visualmente el aviso en la interfaz de Claude.
- La preferencia small es orientativa, no limita los reintentos internos del runtime. El arreglo del arnes no prometera controlar una capacidad que Claude no expone.
- Contexto previo consultado: mapa cubre el tema, grafo reciente, impacto del hub sin datos concretos. Lecciones: promesas-estructurales-vs-disciplina y criterios-de-cierre-que-se-pueden-fallar.
- Spec completo con AC-1 a AC-10, Estado: draft; leido integro y abierto en el editor con open -t (exit 0). Sus criterios se presentan en chat para pedir el SI del usuario antes de planificar/implementar (AGENTS.md paso 2; constitucion articulo 2). No se ejecuto approve-spec.
- git diff --check limpio. Las dos actualizaciones de uso de lecciones viven en la rama #72; checkout principal sin cambios de codigo.
- El launcher del worktree no tiene binario local (exit 127); para la comprobacion de spec se usa el launcher ya instalado del repo principal con --feature 72. No se reinstalo ni compilo antes de la aprobacion.
- check-spec desde el principal devolvio 2 por diferencia con la firma del spec semilla: corresponde a nuestra edicion del borrador, no prueba que otro agente lo haya cambiado. Ya se releyo entero; no se corre advance ni se re-firma/aprueba hasta el SI del usuario.
- No se modificaron el codigo fuente, las sesiones activas de realestate ni la feature pendiente #71. La rama de integracion se preguntara al usuario antes del cierre.
- 2026-09-05T02:37:02Z Spec #72 aprobado por el USUARIO; re-sincronizado tras releerlo completo
