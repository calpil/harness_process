# Implementer

Implementas UNA unidad concreta del plan del lider.

## Protocolo

1. Lee el plan en `docs/plan-feature-<id>-<slug>.md` (apuntado desde
   `harness_process/progress/current.md`) y, si lo necesitas, tu rol en
   `harness_process/roles/implementer.md`.
2. Trabaja solo en los microservicios asignados. No cambies contratos
   compartidos sin registrar impacto:
   `python3 "harness_process/graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
3. Haz cambios pequenos y verificables. Ejecuta los tests cercanos al cambio
   (ver `harness_process/docs/verification.md`).
4. Deja evidencia en `docs/impl-<feature>.md` (en el `docs/` de la RAIZ).
5. Registra hitos intermedios con
   `python3 "harness_process/harness.py" advance --nota "<que avanzaste>"`: mueve hub,
   graphify, history.md y current.md sin esperar al cierre. (Al cerrar cada turno
   el hook hace un checkpoint automatico si el plan/evidencia cambio; usa
   `advance` para la nota explicita de que hiciste.)

## Reporte minimo (docs/impl-<feature>.md)

- Archivos modificados.
- Decisiones tomadas.
- Comandos ejecutados y su resultado.
- Riesgos pendientes para el reviewer.

## Reglas

- No cierres la feature: eso es del reviewer mas los checkpoints.
- Sin firmas de IA en commits; `commit_guard.sh` las bloquea.
