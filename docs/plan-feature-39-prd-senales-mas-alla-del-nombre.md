# Plan - Feature #39: prd_senales_mas_alla_del_nombre

Estado: in_progress
Microservicios:
- harness

## Alcance

Reemplazar la búsqueda exclusiva del slug por señales locales compuestas desde
nombre normalizado, términos específicos del spec y rutas/módulos del cambio.
Cada coincidencia conserva su fuente y línea; no decide el veredicto.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- `ADR/harness`: Hub no disponible por DNS. Impacto local: siembra de bloques
  en `commands/prd.rs` y tests de propuesta en `cli_basics.rs`.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa sitúa las señales de propuesta junto al parser documental; deben
  permanecer como evidencia previa al veredicto y sin llamadas a modelos.

## Delegacion (implementer)

- U1 [AC-1, AC-2, AC-3]: extraer y normalizar señales desde nombre, spec y
  rutas de feature con filtros de ruido.
- U2 [AC-4, AC-5]: renderizar fuente/línea de forma determinista y verificar
  que la cita corresponde al texto existente.
- U3 [AC-6]: fixtures para falso ausente, ausencia real, normalización y rutas.

## Criterios de cierre (reviewer)

- Un documento que describe un módulo/término del spec sin decir el slug deja
  de salir como ausente.
- Toda línea comunicada puede comprobarse literalmente y las pruebas son locales.

## Riesgos

- Términos demasiado genéricos elevan ruido; se filtran y se conservan fuentes.
- Una señal puede confundirse con evidencia suficiente: sigue siendo sugerencia.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: coincidencias textuales locales, sin LLM.

### Avance 2026-08-24T12:21:28Z
Plan #39 completado: señales compuestas desde slug, spec y módulos, con evidencia de línea y tests locales.
