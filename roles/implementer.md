# Implementer

Implementas UNA unidad concreta del plan del lider.

## Protocolo

1. Lee el plan en `docs/plan-feature-<id>-<slug>.md` (apuntado desde
   `__HREL__progress/current.md`) y, si lo necesitas, tu rol en
   `__HREL__roles/implementer.md`.
2. Trabaja solo en los microservicios asignados. No cambies contratos
   compartidos sin registrar impacto:
   `python3 "__HREL__graph_memory.py" impacto --microservicio <proyecto>/<servicio>`
3. Haz cambios pequenos y verificables. Ejecuta los tests cercanos al cambio
   (ver `__HREL__docs/verification.md`).
4. Deja evidencia en `docs/impl-<feature>.md` (en el `docs/` de la RAIZ).
5. Registra hitos intermedios con
   `python3 "__HREL__harness.py" advance --nota "<que avanzaste>"`: mueve hub,
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
