# Spec - Feature #41: consolidar_usa_relacionadas

Estado: approved
Aprobado: 2026-08-24T12:12:07Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-41-consolidar-usa-relacionadas.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: sumar `relacionadas` como señal local para proponer revisiones de consolidación; no archiva ni fusiona lecciones automáticamente.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Bruno conecta dos lecciones que se explican mutuamente, pero sus triggers no comparten palabras. El detector no las presenta como candidatas y él debe descubrir manualmente un solapamiento escrito a propósito.
DESPUES: Bruno ve esas referencias explícitas junto a las demás señales y decide si ameritan una consolidación, sin que una cita sola altere el catálogo.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
consolidar -> solo triggers     consolidar -> triggers + `relacionadas`
                                         |__ pares con vínculo mutuo
                                         |__ propuesta revisable
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como mantenedor de lecciones, quiero que las referencias mutuas sean candidatas de consolidación, para no depender de que los triggers tengan las mismas palabras.
- P2: Como revisor, quiero que la salida explique qué relación escrita originó el candidato, para distinguir una señal barata de una decisión.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given dos lecciones activas que se declaran mutuamente en `relacionadas`, When corre la detección, Then aparecen como candidatas aunque sus triggers no se crucen.
- AC-2: Given una relación de un solo sentido, rota o a una lección inexistente, When se detecta, Then no crea un candidato ficticio y el diagnóstico sigue siendo útil.
- AC-3: Given un par que coincide por triggers y por relación, When se lista, Then aparece una sola vez y conserva ambas razones de evidencia.
- AC-4: Given lecciones archivadas o inválidas, When se calcula la relación, Then respeta las mismas reglas de elegibilidad que la detección existente.
- AC-5: Given un candidato por relación, When se muestra, Then nombra las dos lecciones y la referencia concreta sin enviar cuerpos de lecciones a un modelo.
- AC-6: Given los fixtures, When corre la suite, Then verifica mutua, unilateral/rota, deduplicación y elegibilidad sin backend LLM ni red.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: `leccion consolidar` sobre el catálogo de lecciones.
- entrada: frontmatter `relacionadas`, estado y triggers ya leídos por el catálogo.
- salida: candidato con motivos `triggers` y/o `relacionadas`.
- candado: un par canónico evita duplicar A-B y B-A.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO se comparan lecciones para consolidar

  validar los nombres en `relacionadas` y el estado elegible
  ¿la relación es mutua? -> si no, no elevarla como candidata por relación
  ¿el par ya fue agregado por otra señal? -> si, enriquecer su evidencia

  ENTONCES proponer el par una sola vez,
           sin modificar ninguna lección.
```
Promesas: referencias explícitas cuentan · pares únicos · salida solo informativa.

## No funcionales
- SLOs: análisis local proporcional al catálogo.
- Seguridad: los nombres de frontmatter se validan como nombres de lección, no como rutas arbitrarias.
- Observabilidad: cada candidato conserva las razones que lo originaron.

## Fuera de alcance
- Consolidar, archivar o editar lecciones automáticamente.
- Usar los cuerpos de las lecciones como prompt adicional.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: se exige reciprocidad para que `relacionadas` sea señal de candidato independiente.
