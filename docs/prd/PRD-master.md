# PRD Master - <nombre del proyecto>

Estado: draft
Ultima actualizacion: <YYYY-MM-DD>
Diseno tecnico: docs/prd/SDD-master.md
Constitution: docs/constitution.md

> Documento del USUARIO: el instalador lo siembra una sola vez y nunca lo pisa.
> Es la fuente de la que salen las features del backlog: cada hito de la seccion
> "Hitos" se convierte en una entrada de `feature_list.json`, y de ahi en un
> `docs/spec-feature-<id>-<slug>.md` con sus AC-n.
>
> Para un proyecto que arranca de cero, completa este archivo ANTES de cargar la
> primera feature. Borra los ejemplos entre <> a medida que los reemplazas.

## 1. Problema

<Que problema real existe hoy, para quien, y por que ahora. Escribe el problema,
no la solucion. Si no podes nombrar a quien le duele, todavia no hay PRD.>

## 2. Usuarios y jobs-to-be-done

| Usuario | Que intenta lograr | Como lo resuelve hoy | Por que no alcanza |
| --- | --- | --- | --- |
| <rol> | <job> | <workaround actual> | <limitacion> |

## 3. Metricas de exito

<Como sabras que funciono, en numeros. Cada metrica con su valor de partida y
su objetivo. Sin metrica no hay forma de cerrar el proyecto.>

| Metrica | Hoy | Objetivo | Como se mide |
| --- | --- | --- | --- |
| <ej. tiempo de alta de un cliente> | <45 min> | <5 min> | <log/dashboard> |

## 4. Alcance

### Dentro
- <capacidad 1 que el producto SI resuelve>

### Fuera (no-objetivos)
- <lo que explicitamente NO se hace, y por que>

> Los no-objetivos son tan importantes como el alcance: son lo que evita que el
> proyecto crezca sin control. Si algo aparece despues, se decide de nuevo, no
> se asume.

## 5. Restricciones y supuestos

- Tecnicas: <stack obligado, sistemas con los que hay que integrar>
- Negocio / legales: <plazos, normativa, contratos>
- Supuestos: <lo que damos por cierto y habria que validar; si un supuesto cae,
  el alcance cambia>

## 6. Experiencia esperada

<Recorridos principales en prosa o bullets, priorizados P1/P2. Estos recorridos
son el insumo directo de la seccion "Recorridos de usuario" de cada spec.>

- P1: Como <rol>, quiero <accion>, para <resultado>.
- P2: Como <rol>, quiero <accion>, para <resultado>.

## 7. Hitos -> features

<Cada fila se carga al backlog con:
 sh harness_cli add --name <slug> --service <servicio> --acceptance "<criterio>"
y al arrancarla (`start`) genera su spec con AC-n.>

| # | Hito | Slug de feature | Criterio de aceptacion (resumen) | Estado |
| --- | --- | --- | --- | --- |
| 1 | <hito> | <slug_snake_case> | <que tiene que ser cierto> | pendiente |

## 8. Riesgos

| Riesgo | Impacto | Mitigacion |
| --- | --- | --- |
| <riesgo> | <alto/medio/bajo> | <que se hace al respecto> |

## 9. Decisiones abiertas

<Mismo protocolo que los planes: una decision sin resolver se pregunta al
USUARIO antes de implementar lo que dependa de ella. Registra aqui la respuesta
con su fecha.>

- <pregunta> — DECIDIDO (<usuario>, <fecha>): <respuesta>
- <pregunta> — ABIERTA
