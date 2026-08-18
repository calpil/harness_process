# SDD Master - Harness Process

Estado: en uso
Ultima actualizacion: 2026-08-18
Producto: docs/prd/PRD-master.md
Constitution: docs/constitution.md

> Documento del USUARIO: el instalador lo siembra una sola vez y nunca lo pisa.
> Es el "como" maestro del proyecto: la arquitectura objetivo y las decisiones
> tecnicas que ninguna feature individual deberia re-litigar.
>
> Relacion con los otros documentos:
> - `docs/prd/PRD-master.md`: que se construye y por que.
> - Este archivo: como se construye, a nivel proyecto.
> - `docs/architecture.md`: el mapa de lo que YA existe (se actualiza solo).
> - `docs/spec-feature-*.md`: el detalle por feature, con sus AC-n.

## 1. Arquitectura objetivo

<Diagrama en texto o descripcion de los componentes y como se hablan. Marca que
existe hoy y que hay que construir.>

```
<componente A> --(<protocolo>)--> <componente B>
```

| Componente | Responsabilidad | Estado | Owner |
| --- | --- | --- | --- |
| <servicio> | <que hace, en una linea> | <existe / a construir> | <equipo> |

## 2. Stack y dependencias

- Lenguajes / runtimes: <...>
- Almacenamiento: <base de datos, colas, cache>
- Servicios externos: <APIs de terceros, con su modo de falla>

> Regla sugerida (ajusta en la constitution): ninguna dependencia nueva de
> runtime entra sin una decision registrada abajo.

## 3. Contratos entre componentes

<Interfaces que cruzan un limite de equipo o de servicio: endpoints, eventos,
esquemas. Un cambio aqui impacta a otros; se registra impacto antes de mergear.>

| Contrato | Productor | Consumidores | Versionado |
| --- | --- | --- | --- |
| <endpoint/evento> | <servicio> | <servicios> | <como se versiona> |

## 4. Decisiones tecnicas

<Una fila por decision estructural. Si una decision es grande o polemica, mueve
el detalle a su propio ADR y enlazalo aqui.>

| # | Decision | Alternativas descartadas | Por que | Fecha |
| --- | --- | --- | --- | --- |
| D1 | <lo que se decidio> | <opcion B, opcion C> | <razon> | <YYYY-MM-DD> |

## 5. Datos

- Entidades principales y su dueno: <...>
- Migraciones: <como se aplican y como se revierten>
- Retencion y datos sensibles: <que se guarda, cuanto tiempo, con que proteccion>

## 6. No funcionales

- SLOs: <latencia, disponibilidad, throughput objetivo>
- Seguridad: <autenticacion, autorizacion, manejo de secretos>
- Observabilidad: <logs, metricas, trazas; que se alerta y a quien>
- Costos: <limites o presupuesto que condicionan el diseno>

## 7. Estrategia de verificacion

<Como se prueba el sistema, mas alla de los tests de cada feature. Los comandos
concretos viven en `docs/verification.md`.>

- Tests automaticos: <unitarios, integracion, e2e; que cubre cada nivel>
- Entornos: <local, staging, produccion>
- Criterio de "listo para produccion": <...>

## 8. Riesgos tecnicos

| Riesgo | Probabilidad | Impacto | Mitigacion |
| --- | --- | --- | --- |
| <riesgo> | <alta/media/baja> | <alto/medio/bajo> | <plan> |

## 9. Decisiones abiertas

<Igual que en el PRD: sin decision registrada, se pregunta al USUARIO antes de
implementar lo que dependa de ella.>

- <pregunta> — DECIDIDO (<usuario>, <fecha>): <respuesta>
- <pregunta> — ABIERTA
