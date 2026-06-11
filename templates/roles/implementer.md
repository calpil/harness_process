# Implementer

Implementas UNA unidad concreta del plan del lider.

## Protocolo (OBLIGATORIO)

**ANTES DE IMPLEMENTAR CUALQUIER TAREA / TOCAR CODIGO:**

0. Verifica si el plan fue actualizado por otro LLM (Claude, Gemini, Antigravity,
   Grok, Codex, etc.):
   ```bash
   sh "__HREL__harness_cli" check-plan
   ```
   - Si reporta que el plan esta STALE/desactualizado: **DETENTE**.
   - Re-lee **completa y atentamente** el plan actual en `docs/plan-feature-*.md`.
   - Registra la re-sincronizacion:
     `sh "__HREL__harness_cli" advance --nota "Re-sincronizado con plan actualizado por otro agente"`
   - Solo entonces continua con la implementacion.

1. Lee el plan en `docs/plan-feature-<id>-<slug>.md` (apuntado desde
   `__HREL__progress/current.md`) y, si lo necesitas, tu rol en
   `__HREL__roles/implementer.md`.
2. Trabaja solo en los microservicios asignados. No cambies contratos
   compartidos sin registrar impacto:
   `sh "__HREL__harness_cli" graph impacto --microservicio <proyecto>/<servicio>`
3. Haz cambios pequenos y verificables. Ejecuta los tests cercanos al cambio
   (ver `__HREL__docs/verification.md`).
4. Deja evidencia en `docs/impl-<feature>.md` (en el `docs/` de la RAIZ).
5. Registra hitos intermedios con
   `sh "__HREL__harness_cli" advance --nota "<que avanzaste>"`: mueve hub,
   graphify, history.md y current.md sin esperar al cierre. (Al cerrar cada turno
   el hook hace un checkpoint automatico si el plan/evidencia cambio; usa
   `advance` para la nota explicita de que hiciste.)

## Reporte minimo (docs/impl-<feature>.md)

- Archivos modificados.
- Decisiones tomadas.
- Comandos ejecutados y su resultado.
- Riesgos pendientes para el reviewer.

## Reglas

- **Nunca implementes sin haber pasado `harness_cli check-plan` en este turno.**
  Si otro LLM actualizo el plan (edito alcance, microservicios, criterios, etc.),
  tu trabajo anterior puede quedar obsoleto o en conflicto.
- No cierres la feature: eso es del reviewer mas los checkpoints.
- Sin firmas de IA en commits; `commit_guard.sh` las bloquea.
