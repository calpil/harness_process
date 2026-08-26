# Plan - Feature #54: prd_apply_escribe_en_el_docs_de_la_feature

Estado: in_progress
Microservicios:
- harness

## Alcance

Hacer que `prd propose` y `prd apply` seleccionen una sola vez el `docs/` de
la feature mediante su worktree registrado. Esa selección cubre el alcance,
la propuesta, las citas y los destinos de escritura; si no hay worktree
usable, mantiene el `docs/` efectivo y lo informa.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- Impacto local (`ADR/harness`; Hub inaccesible por DNS): comandos Rust de
  PRD, `HarnessPaths::para_feature`, documentos versionados de cada worktree y
  los fixtures CLI que simulan principal + feature.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa enlaza `prd propose/apply` con `documentos::{alcance,propuesta_path}`;
  ambos reciben `HarnessPaths`, por lo que resolverlo antes de ese borde evita
  dividir lectura y escritura entre el principal y la rama.

## Delegacion (implementer)

- U1 [AC-1, AC-3]: resolver una vez `paths.para_feature(feature)` en
  `propose` y usarlo para alcance, señales y propuesta desde el principal.
- U2 [AC-2, AC-5]: usar la misma resolución en `apply`, incluyendo
  planificación, escritura, sello y registro para que el cambio quede en la
  rama integrable.
- U3 [AC-4, AC-6]: informar el fallback cuando no exista worktree y añadir
  fixtures principal/worktree con documentos distintos, propose y apply
  confirmado sin depender de Git real.

## Criterios de cierre (reviewer)

- Todos los paths documentales de una invocación apuntan al mismo árbol; el
  `docs/` principal no recibe propuesta ni escritura cuando hay worktree.
- El fallback conserva la semántica anterior y es visible; las pruebas cubren
  aislar, aplicar y conservar el principal.

## Riesgos

- Resolver solo el destino y no el alcance permitiría propuestas de un árbol
  que se aplican en otro; la selección se hace antes de ambos recorridos.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: un `worktree` válido manda aunque el proceso se
  ejecute desde el checkout principal; uno ausente o inválido cae en la raíz
  efectiva con aviso.

### Avance 2026-08-25T10:05:00Z

Plan #54 completado: U1-U3 cubren AC-1..AC-6 con una única selección de rutas
por feature y fixtures de aislamiento principal/worktree.

### Avance 2026-08-26T00:20:05Z
Plan completo: selector unico de docs por worktree y fixtures de aislamiento.
