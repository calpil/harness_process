# Plan - Feature #53: check_no_se_cuelga_por_stdin

Estado: in_progress
Microservicios:
- harness

## Alcance

Cerrar stdin exclusivamente en la llamada interna que `harness_check.sh` hace a
`commit_guard.sh`; el guard invocado directamente conserva el payload de hooks.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- Impacto local (`ADR/harness`; Hub inaccesible por DNS): `harness_check.sh`,
  su plantilla instalada y los smoke tests de guard/check.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa sitúa al guard como lector de stdin y al check como consumidor no
  interactivo; la redirección pertenece a esa única frontera.

## Delegacion (implementer)

- U1 [AC-1, AC-2, AC-3]: redirigir stdin de la invocación del check y comprobar
  finalización, limpio y bloqueante.
- U2 [AC-4]: conservar la ruta de invocación directa del guard sin redirección.
- U3 [AC-5, AC-6]: actualizar fuente/plantilla y un fixture acotado que mida
  terminación sin dormir ni usar red.

## Criterios de cierre (reviewer)

- `harness_check.sh` nunca depende de stdin; el guard directo continúa leyendo
  datos que un hook sí provee.
- Fuente y plantilla son idénticas y las rutas prohibidas siguen bloqueando.

## Riesgos

- Cerrar stdin en el guard mismo rompería hooks; se limita al proceso hijo que
  abre el check no interactivo.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: redirección literal `</dev/null` solo en las dos
  copias de `harness_check.sh`.

### Avance 2026-08-25T09:15:00Z

Plan #53 completado: cierre de stdin en la frontera no interactiva y fixture
de terminación/paridad; U1-U3 cubren AC-1..AC-6.

### Avance 2026-08-26T00:13:49Z
Plan #53 completado: cierre de stdin no interactivo, guard directo intacto y pruebas AC-1..AC-6.

---
Cerrado: 2026-08-26T01:01:48Z - status=done - Cierre tras integracion consolidada y validacion verde; sello documental aprobado sin cambios maestros.
