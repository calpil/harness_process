# Spec - Feature #55: check_resuelve_el_spec_de_la_feature

Estado: approved
Aprobado: 2026-08-24T12:12:33Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-55-check-resuelve-el-spec-de-la-feature.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: alinear el resumen de specs de `harness_check.sh` con la resolución por feature que ya usa `check-spec`.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Irene ejecuta el check principal mientras una feature vive en su worktree. El spec está aprobado en esa rama, pero el resumen busca `docs/` del principal y le informa falsamente “spec ausente”.
DESPUES: Irene recibe el estado real de cada feature activa, sin importar desde qué checkout ejecute el resumen, y cualquier ausencia reportada es comprobable.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
harness_check -> docs principal     harness_check -> rutas por feature
                                             |__ mismo resultado que check-spec
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como agente de cierre, quiero un resumen de specs que apunte al worktree correcto, para no perseguir bloqueos inexistentes.
- P2: Como revisor, quiero que el resumen y el gate usen la misma fuente de verdad, para que un mensaje no contradiga el código de salida.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given una feature activa con worktree y spec aprobado fresco en ese worktree, When corre `harness_check.sh` desde el principal, Then el resumen la reporta aprobada y no dice `ausente`.
- AC-2: Given una feature activa con spec realmente ausente, draft o stale en su worktree, When corre el check, Then el resumen nombra el estado real y el gate mantiene su bloqueo.
- AC-3: Given dos o más features activas en worktrees distintos, When corre el resumen, Then resuelve cada una contra su propio árbol sin cruzar sus documentos.
- AC-4: Given una feature sin worktree válido, When corre el check, Then conserva la resolución de raíz existente y no falla por intentar entrar a una ruta inexistente.
- AC-5: Given `check-spec --feature <id>` y el resumen de `harness_check.sh`, When reciben el mismo estado, Then coinciden en aprobado, draft, stale o ausente.
- AC-6: Given fixtures con worktrees simulados, When corre la suite, Then prueba el falso ausente, los estados bloqueantes, múltiples features y fallback sin depender de Git remoto.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: resumen por feature dentro de `harness_check.sh`.
- selector: metadatos de la feature activa y su worktree validado.
- fuente: la misma ruta que determina `check-spec` para esa feature.
- candado: cada vuelta del loop crea o recibe su contexto de ruta aislado.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO el check resume una feature activa

  resolver sus rutas exactamente como `check-spec`
  leer el estado desde el `docs/` de esa feature
  ¿la ruta no es válida? -> si, aplicar el fallback conocido

  ENTONCES imprimir el estado que el gate realmente evaluará,
           sin consultar el `docs/` de otra feature.
```
Promesas: resumen y gate coherentes · múltiples worktrees aislados · ausencias verdaderas visibles.

## No funcionales
- SLOs: solo filesystem local por feature; no agrega llamadas de red.
- Seguridad: las rutas se derivan de metadatos validados, nunca de texto de salida.
- Observabilidad: cada línea conserva el id de feature y el estado exacto.

## Fuera de alcance
- Relajar el gate de spec aprobado o cambiar el ritual de aprobación.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: el resumen debe reutilizar o replicar solo la resolución ya cubierta por `check-spec`, no inventar una segunda regla.
