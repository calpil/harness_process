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

**Como habla el arnes con un LLM** (feature #28, la unica parte que usa modelo).
La cadena es `HARNESS_CONSOLIDAR_CMD` -> primer CLI de una tabla corta
(`claude -p`, `kimi -p`) -> **skip limpio**. Apagada por default y de forma
estructural: sin `rules.consolidar_backend` no se resuelve backend ni se mira el
entorno. Tres decisiones que valen para cualquier feature futura con modelo:

- **Al modelo se le manda lo minimo.** Ve nombre, descripcion y triggers; nunca
  el cuerpo de una leccion. Lo peor que puede hacer es equivocarse, no filtrar.
- **El prompt viaja como item de argv, jamas por `sh -c`.** Por eso NO se reusa
  `verificacion::ejecutar`, que si corre con shell.
- **El modelo propone; lo que muta sale de argv.** La mitad que escribe se
  verifica sin backend y de forma determinista.

El tramo HTTP con API key **no esta implementado** y el mensaje de skip lo dice.

<Una fila por decision estructural. Si una decision es grande o polemica, mueve
el detalle a su propio ADR y enlazalo aqui.>

| # | Decision | Alternativas descartadas | Por que | Fecha |
| --- | --- | --- | --- | --- |
| D1 | <lo que se decidio> | <opcion B, opcion C> | <razon> | <YYYY-MM-DD> |

## 5. Datos

**Los estados de una feature** (`feature_list.json`). Son cinco y significan
cosas distintas: `pending` (sin empezar, `next` la ofrece), `in_progress` (una
sola a la vez), `done` (hecha, con spec aprobado y su evidencia), `blocked`
(trabada por algo externo) y `superseded` (el trabajo se hizo en OTRA feature,
que se nombra en `superseded_by` y se valida al cerrar). Solo `done` pasa por los
cuatro gates de cierre; `superseded` no cuenta ni en el numerador ni en el
denominador de `prd tree`, porque no es trabajo hecho ni pendiente.

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

- **Tests automaticos**: unitarios en `rust/src/**` (modulos `mod tests`, sobre
  todo para las funciones PURAS: parsear, planificar, diagnosticar, decidir),
  integracion en `rust/tests/cli_basics.rs` (el binario de verdad contra un
  sandbox `tempfile`), y chequeos de shell en `tests/*.sh` para lo que vive
  fuera de Rust (los dos instaladores, los espejos, el corpus real del repo).
- **Los AC se ejecutan**: cada AC-n de un spec puede declarar `Comando:`, y
  `harness_cli verify --feature <id>` los corre y escribe `docs/verify-<id>.md`.
  Con `require_verify_green`, `close --status done` LEE ese reporte —nunca
  ejecuta— y no deja cerrar con alguno bloqueando.
- **Un AC que no midio nada no cuenta como verificado**: `cargo test <nombre>`
  con un filtro que no matchea sale 0, y eso ya produjo un falso verde real. Por
  eso `verify` mira la salida ademas del exit code y marca `vacio` al AC que
  reconocidamente no ejecuto ningun caso. Sobre salidas que no son de libtest no
  opina: el estado no cambia.
- **Entornos**: solo local. El arnes no se despliega; se instala en el repo del
  proyecto con `setup_harness.sh` / `.ps1`, y `tests/parity_check.sh` verifica
  que los dos hagan lo mismo.
- **Criterio de "listo"**: los AC del spec en verde en su reporte, la suite
  completa y `cargo clippy -D warnings` limpios, los chequeos de `tests/` en
  verde, y `harness_check.sh` sin problemas.

## 8. Riesgos tecnicos

| Riesgo | Probabilidad | Impacto | Mitigacion |
| --- | --- | --- | --- |
| <riesgo> | <alta/media/baja> | <alto/medio/bajo> | <plan> |

## 9. Decisiones abiertas

<Igual que en el PRD: sin decision registrada, se pregunta al USUARIO antes de
implementar lo que dependa de ella.>

- <pregunta> — DECIDIDO (<usuario>, <fecha>): <respuesta>
- <pregunta> — ABIERTA
