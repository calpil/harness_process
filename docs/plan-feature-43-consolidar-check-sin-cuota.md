# Plan - Feature #43: consolidar_check_sin_cuota

Estado: in_progress
Microservicios:
- harness

## Alcance

Convertir `tests/consolidar_check.sh` en una verificación local con backend
falso por defecto. Mantener el recorrido real únicamente detrás de un flag
explícito y documentado.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- Impacto local (`ADR/harness`; Hub inaccesible por DNS): script de integración
  de consolidación, su salida observable y documentación de cómo pedir real.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa enlaza `consolidar_check.sh` con el comando de consolidación: el
  fixture debe controlar `HARNESS_CONSOLIDAR_CMD`, no alterar producción.

## Delegacion (implementer)

- U1 [AC-1, AC-4, AC-6]: instalar backend falso determinista por defecto e
  impedir que variables/credenciales activen el modo real.
- U2 [AC-2, AC-5]: conservar casos de propuesta, descarte, error y paraguas
  con respuestas falsas controladas y diagnósticos locales.
- U3 [AC-3]: incorporar `--real`, preflight explícito y documentación sin
  activarlo desde la suite cotidiana.

## Criterios de cierre (reviewer)

- La ejecución normal no contacta ningún backend y cubre los contratos
  anteriores; `--real` es la única puerta de integración externa.
- Un falso roto falla localmente, sin fallback ni secreto requerido.

## Riesgos

- Una variable heredada podría reactivar costo sin intención; el script debe
  sobrescribirla en falso y requerir el argumento exacto para real.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: el nombre del interruptor es `--real`; sin él no
  se ejecuta ninguna llamada de integración.

### Avance 2026-08-24T14:30:00Z

Plan #43 completado: backend falso controlado por defecto, `--real` aislado y
cobertura de fallas locales; U1-U3 trazan AC-1..AC-6.

### Avance 2026-08-25T03:06:47Z
Plan #43 completado: falso por defecto, --real explícito y fallas locales por AC-1..AC-6.

---
Cerrado: 2026-08-26T01:01:47Z - status=done - Cierre tras integracion consolidada y validacion verde; sello documental aprobado sin cambios maestros.
