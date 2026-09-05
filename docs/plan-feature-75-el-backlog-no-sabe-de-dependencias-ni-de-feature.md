# Plan - Feature #75: el backlog no sabe de dependencias ni de features que se traban una y otra vez

Estado: in_progress
Microservicios:
- harness_process (rust: features/commands next, close)

## Alcance

Dos preguntas que el backlog no sabia responder: "¿que espera a que?" y "¿que se
traba siempre?". Comparten archivo (`dependencias.rs`) porque comparten el dato
—el backlog— y nada mas.

## Peldano de huella

`Peldano elegido: 3 (comando nuevo: harness depende) porque ningun comando
existente edita una feature: add crea y close cierra.` Se intento el peldano 2
—solo `add --depends-on`— y NO alcanza: una dependencia se descubre casi siempre
DESPUES de que las dos features existen. El recorrido P1 del spec ("Alan declara
que la #21 depende de la #17") es imposible desde `add`, porque las dos ya
estaban creadas. Ademas, sin ese camino la deteccion de ciclos del AC-5 seria
codigo inalcanzable: por `add` el grafo es un DAG por construccion.

## Delegacion (implementer)

- D-1 (AC-1, AC-5): `rust/src/dependencias.rs` NUEVO. Funciones PURAS:
  `motivo_invalido` (id inexistente / auto-referencia / ciclo, en ese orden) y
  `abiertas`. Se prueban contra el backlog real sin tocar disco.
- D-2 (AC-2): `next` saltea las que tienen dependencias abiertas y, si no ofrece
  nada POR ESE MOTIVO, lo dice con nombres.
- D-3 (AC-3): `start` avisa y NO bloquea.
- D-4 (AC-4): el circuit breaker en la FASE 0 de `close` (lo que puede negarse)
  y el contador en la FASE 3 (con el resto del estado): contar antes haria que
  un cierre negado sumara igual.
- D-5 (AC-1, AC-5): `harness depende --feature N --de M [--quitar]`.
- D-6 (AC-7): `status` muestra las dependencias abiertas de las activas.

## Criterios de cierre (reviewer)

- El AC-4 necesita DOS tests, no uno: su condicion no se observo nunca en 84
  cierres reales, asi que sus tests son toda su evidencia. Uno que lo dispare y
  otro que compruebe que no se dispara antes de tiempo.
- Ningun test puede llamarse como algo que no prueba (leccion de la #73).

## Riesgos

- R-1: el AC-4 implementa un gate cuya condicion nunca ocurrio. Declarado en el
  spec y decidido por el usuario con la medicion a la vista.
- R-2: `depends_on` ya existe como nombre en `graph/derive.rs`, pero es una
  relacion del GRAFO del hub y no del backlog. No se reusa ni se mezcla.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por el usuario 2026-09-05): los siete AC de la ficha, incluido
  el circuit breaker.
- OBS-2: la rama de integracion se pregunta antes de `close --status done --to`.
- OBS-3 (decidida al implementar): se agrego `harness depende` porque sin el la
  feature no cumplia su propio recorrido P1. Es alcance nuevo respecto de la
  ficha; queda dicho en el impl y en el review.

---
Cerrado: 2026-09-05T14:07:33Z - status=done - Dependencias entre features con validacion y ciclos, y el contador de bloqueos; harness depende porque add no cumplia el recorrido P1
