# Plan - Feature #42: consolidar_esqueleto_del_paraguas

Estado: in_progress
Microservicios:
- harness

## Alcance

Agregar `lecciones consolidar --preparar --en <paraguas> --de <a,b>` para
crear un borrador nuevo y determinista: triggers unidos y punteros a cada
miembro, sin archivar ni escribir la prosa humana.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- Impacto local (`ADR/harness`; Hub inaccesible por DNS): CLI de `lecciones`,
  generador de borrador, validador `revisar_paraguas` y pruebas de fusión.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa conecta `commands/leccion.rs` con `consolidacion.rs`: la preparación
  debe reutilizar los mismos triggers y punteros que el validador exige.

## Delegacion (implementer)

- U1 [AC-1, AC-2, AC-3]: añadir modo explícito de preparación y construir la
  unión canónica de triggers y `[[miembro]]` desde una selección válida.
- U2 [AC-4, AC-5]: preservar cualquier archivo existente y demostrar que el
  borrador nuevo satisface el validador estructural.
- U3 [AC-6]: fixtures sin backend para selección, deduplicación,
  idempotencia y revisión final.

## Criterios de cierre (reviewer)

- El comando crea únicamente un borrador nuevo, no archiva ni reemplaza un
  borrador existente.
- Triggers y punteros son deterministas, completos y suficientes para
  `revisar_paraguas`.

## Riesgos

- Un preparador que reescriba un archivo existente perdería prosa humana; el
  archivo existente es una barrera de no-escritura, incluso si es incompleto.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: `--preparar` es una decisión explícita distinta
  de `--aplicar`; el borrador no implica archivar ni cerrar una consolidación.

### Avance 2026-08-24T14:05:00Z

Plan #42 completado: modo explícito de borrador, unión canónica y preservación
del archivo existente; U1-U3 cubren AC-1..AC-6.

### Avance 2026-08-25T02:33:31Z
Plan #42 completado: preparar borrador explícito, unión canónica y no-sobrescritura por AC-1..AC-6.
