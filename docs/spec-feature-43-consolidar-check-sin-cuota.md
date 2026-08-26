# Spec - Feature #43: consolidar_check_sin_cuota

Estado: approved
Aprobado: 2026-08-24T12:12:18Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-43-consolidar-check-sin-cuota.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: aislar la suite de consolidación de servicios/modelos reales; no elimina la verificación opcional de integración real.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Nicolás ejecuta la suite para revisar un cambio local y `consolidar_check.sh` consume cuota de un backend real. Una prueba cotidiana es lenta, costosa y frágil por causas externas.
DESPUES: Nicolás obtiene el mismo contrato comprobable con un backend falso por defecto; solo una intención explícita habilita la corrida real.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
suite normal -> backend real     suite normal -> backend falso controlado
                                                |__ flag explícito -> integración real
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como mantenedor, quiero correr toda la suite sin cuota ni credenciales, para que una regresión local sea rápida y reproducible.
- P2: Como responsable de integración, quiero poder solicitar deliberadamente la prueba real, para conservar una verificación de extremo a extremo separada.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given el entorno normal sin credenciales, When corre `tests/consolidar_check.sh`, Then usa un backend falso y termina sin red ni consumo de cuota.
- AC-2: Given el backend falso, When los casos cubren propuesta, descarte, error y paraguas, Then el test controla sus respuestas y verifica el contrato observable igual que antes.
- AC-3: Given una persona que quiere validar el backend real, When pasa el flag explícito documentado, Then el script habilita esa ruta y explica los prerequisitos sin activarla por accidente.
- AC-4: Given una variable de backend real presente pero sin el flag explícito, When corre la suite normal, Then sigue usando el falso.
- AC-5: Given un fallo del backend falso o una respuesta malformada, When corre el check, Then falla con diagnóstico local y no intenta un fallback a un servicio real.
- AC-6: Given CI y una estación limpia, When corre la suite habitual, Then es determinista, no requiere secretos y conserva cobertura de las invariantes de consolidación.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: ejecución de `tests/consolidar_check.sh`.
- modo: falso por defecto; real solo mediante un flag de intención inequívoca.
- entradas: respuestas fijas o script falso controlado por el fixture.
- candado: ninguna variable ambiental basta por sí sola para activar llamadas reales.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO corre el check de consolidación

  ¿se pidió explícitamente integración real? -> si no, configurar backend falso
  ejecutar los casos contra respuestas controladas
  ¿una respuesta viola el contrato? -> si, fallar localmente

  ENTONCES terminar sin red ni cuota por defecto,
           y reservar el backend real para una orden intencional.
```
Promesas: cero llamadas reales por defecto · flag explícito para integración · fallas reproducibles.

## No funcionales
- SLOs: la suite normal no espera servicios externos.
- Seguridad: no imprime ni exige secretos en modo falso.
- Observabilidad: el encabezado informa qué modo se ejecutó y cómo pedir el real.

## Fuera de alcance
- Cambiar el comportamiento productivo de `leccion consolidar` o eliminar por completo la prueba real.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: el modo real queda detrás de un flag documentado y no de la detección de credenciales.
