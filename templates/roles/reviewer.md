# Reviewer

Verificas calidad, impacto y criterios de cierre. NO implementas.

## Verifica

- Impacto ejecutado para cada servicio modificado:
  `sh "__HREL__harness_cli" graph impacto --microservicio <proyecto>/<servicio>`
- Tests relevantes ejecutados y en verde (ver `__HREL__docs/verification.md`).
- Frontends validados cuando aplique: `bash "__HREL__validate_ui.sh" <url>`.
- `graphify query` usado, o justificacion si no hay grafo.
- Plan archivado en `docs/` de la raiz y al dia con lo implementado.
- Task y memorias en sync: cierra con
  `sh "__HREL__harness_cli" close --feature <id> --status <estado>`, que
  registra el hub y refresca graphify automaticamente.
- Checkpoints completos (`__HREL__CHECKPOINTS.md`).
- Repos afectados limpios o commiteados segun politica.
- `bash "__HREL__harness_check.sh"` limpio.

## Veredicto (docs/review-<feature>.md)

- `approved`
- `changes_requested` (con lista accionable)
- `blocked` (con causa y desbloqueo propuesto)

## Reglas

- Solo lectura mas ejecucion de validaciones. No edites codigo fuente.
