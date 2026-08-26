# Plan - Feature #55: check_resuelve_el_spec_de_la_feature

Estado: in_progress
Microservicios:
- harness

## Alcance

Alinear el resumen `status` que invoca `harness_check.sh` con `check-spec`:
por cada feature activa se resuelve su propio `HarnessPaths::para_feature`
antes de medir frescura y estado de plan/spec. El estado global continúa en el
principal; solo los documentos se leen desde cada worktree.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- Impacto local (`ADR/harness`; Hub inaccesible por DNS): `harness_check.sh`
  delega en `harness status`; el comando Rust resume firmas y specs de las
  features activas y debe aislar sus `docs/` igual que `check-spec`.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa conecta `status::{is_spec_stale,spec_state}` con `spec_path`; ambos
  reciben `HarnessPaths`, así que la resolución se hace dentro del loop por
  feature y no se comparte el `docs/` del principal.

## Delegacion (implementer)

- U1 [AC-1, AC-5]: en el resumen, derivar las rutas por feature antes de
  consultar su estado/frescura, con la misma función que `check-spec`.
- U2 [AC-2, AC-3]: probar dos worktrees con specs aprobados/draft/stale y
  contrastar cada línea con `check-spec --feature`.
- U3 [AC-4, AC-6]: cubrir el fallback sin worktree válido, sin Git remoto ni
  llamadas de red, y verificar que los árboles no se cruzan.

## Criterios de cierre (reviewer)

- Cada línea `[spec] #id` corresponde al `docs/` de esa feature y no dice
  ausente si el spec aprobado está en su worktree.
- El resumen conserva ids/estados y coincide con el gate para casos aprobado,
  draft, stale y fallback.

## Riesgos

- Resolver una ruta fuera del loop mezclaría el primer worktree con las demás
  features; el contexto de rutas es una variable local de cada iteración.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: se reutiliza `para_feature`, ya usada por
  `check-spec`; la inexistencia del worktree conserva la raíz efectiva.

### Avance 2026-08-25T10:40:00Z

Plan #55 completado: U1-U3 cubren AC-1..AC-6 con resolución local por feature
y fixtures de worktrees múltiples más el fallback clásico.

### Avance 2026-08-26T00:25:20Z
Plan completo: resumen de specs aislado por worktree y pruebas de paridad.

---
Cerrado: 2026-08-26T01:03:10Z - status=done - Cierre tras integracion consolidada y validacion verde; sello documental aprobado sin cambios maestros.
