# Plan - Feature #41: consolidar_usa_relacionadas

Estado: in_progress
Microservicios:
- harness

## Alcance

Extender `leccion consolidar` para que use referencias mutuas de frontmatter
`relacionadas` como una segunda señal local. Conserva la señal de triggers y
solo produce candidatos informativos, sin editar el catálogo.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- Impacto local (`ADR/harness`; Hub inaccesible por DNS): parser/catálogo de
  lecciones, construcción de pares de consolidación y formato de salida CLI.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa relaciona `lecciones.rs` con sus comandos de catálogo y las pruebas
  unitarias; la nueva razón debe conservarse en la misma estructura de pares.

## Delegacion (implementer)

- U1 [AC-1, AC-2, AC-4]: leer `relacionadas`, validar nombres y aceptar solo
  referencias recíprocas entre lecciones elegibles.
- U2 [AC-3, AC-5]: unir la nueva señal con la de triggers mediante un par
  canónico y mostrar ambas razones de forma revisable.
- U3 [AC-6]: fixtures para relación mutua, unilateral/inexistente,
  deduplicación y archivadas, sin red ni LLM.

## Criterios de cierre (reviewer)

- Un par recíproco sin triggers aparece; uno unilateral o roto no.
- Un par por ambas fuentes es único, enumera ambas razones y no cambia archivos
  de lecciones.

## Riesgos

- El frontmatter es entrada de usuario: nombres desconocidos, duplicados o
  estados no elegibles no pueden convertirse en rutas ni candidatos.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: la reciprocidad es obligatoria para que una
  relación cuente como señal por sí sola.

### Avance 2026-08-24T12:50:00Z

Plan #41 completado: se combinan pares canónicos de triggers y referencias
mutuas; U1-U3 trazan AC-1..AC-6.

### Avance 2026-08-24T12:38:38Z
Plan #41 completado: relaciones mutuas elegibles, pares canónicos y evidencia local por AC-1..AC-6.

---
Cerrado: 2026-08-26T01:00:50Z - status=done - Cierre tras integracion consolidada y validacion verde; sello documental aprobado sin cambios maestros.
