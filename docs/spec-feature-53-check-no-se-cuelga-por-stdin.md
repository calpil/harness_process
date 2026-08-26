# Spec - Feature #53: check_no_se_cuelga_por_stdin

Estado: approved
Aprobado: 2026-08-24T12:12:23Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-53-check-no-se-cuelga-por-stdin.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: garantizar que `harness_check.sh` no espere stdin al invocar el guard; el comportamiento del hook interactivo se conserva.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Camila deja un `harness_check.sh` en segundo plano antes de cerrar una feature. El check abre `commit_guard.sh`, este espera `cat` sobre stdin y la revisión puede quedar bloqueada indefinidamente.
DESPUES: Camila recibe siempre un resultado finito del check no interactivo; el guard conserva su entrada cuando corre desde un hook que sí la provee.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
harness_check -> guard lee stdin  harness_check -> guard con stdin cerrado
                                        hook real -> stdin preservado
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como agente o CI que ejecuta el cierre, quiero que el check termine aunque nadie le entregue stdin, para no bloquear la integración.
- P2: Como hook de commit, quiero que el guard siga evaluando el payload que Git le pasa, para no abrir una vía de bypass.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given `harness_check.sh` sin stdin disponible, When invoca `commit_guard.sh`, Then el check termina dentro de un límite medible y no queda esperando una lectura.
- AC-2: Given stdin cerrado para el guard, When no hay cambios que bloquear, Then mantiene el mismo resultado limpio que antes.
- AC-3: Given cambios de código o documentos no permitidos, When corre el check no interactivo, Then el guard todavía los detecta y bloquea.
- AC-4: Given un hook que entrega su payload por stdin, When ejecuta `commit_guard.sh` por su ruta normal, Then ese payload se conserva y es evaluado.
- AC-5: Given los scripts fuente y sus espejos, When se actualiza el remedio, Then mantienen paridad y no introducen redirecciones que afecten otros llamados.
- AC-6: Given un fixture que antes se colgaba, When corre la prueba, Then prueba la terminación, el caso limpio y el caso bloqueante sin depender de temporizadores largos.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: invocación no interactiva de `harness_check.sh`.
- entrada: estado Git y, solo en hooks, el payload por stdin.
- salida: resultado del guard y del check con código de salida finito.
- candado: el cierre explícito de stdin aplica únicamente a la invocación interna del check.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO `harness_check.sh` llama al commit guard

  cerrar solo el stdin de esa llamada no interactiva
  ejecutar el guard contra el estado del repositorio
  ¿detecta un cambio bloqueante? -> si, propagar el fallo

  ENTONCES terminar siempre,
           sin alterar el camino de hook que recibe stdin real.
```
Promesas: no espera entrada inexistente · conserva los bloqueos · hooks intactos.

## No funcionales
- SLOs: una corrida sin stdin no supera el tiempo de un check normal.
- Seguridad: cerrar stdin no desactiva la inspección de cambios ni acepta repositorios sucios.
- Observabilidad: el test registra la duración del caso que antes se colgaba.

## Fuera de alcance
- Cambiar las reglas de `commit_guard.sh` sobre rutas protegidas o datos de hook.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: el remedio se limita a la frontera no interactiva del check.
