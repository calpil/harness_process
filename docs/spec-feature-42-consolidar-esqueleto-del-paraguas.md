# Spec - Feature #42: consolidar_esqueleto_del_paraguas

Estado: approved
Aprobado: 2026-08-24T12:12:12Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-42-consolidar-esqueleto-del-paraguas.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: prearmar el borrador de una lección paraguas; no decide su prosa ni archiva miembros.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Valentina acepta consolidar varias lecciones y debe recordar a mano todos los triggers y enlaces que `revisar_paraguas` exigirá. Un olvido aparece recién al final, cuando ya escribió la prosa.
DESPUES: Valentina abre un esqueleto completo con la unión de triggers y los punteros a cada miembro; solo escribe la explicación humana que ningún binario puede decidir.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
paraguas aceptado -> archivo vacío  paraguas aceptado -> borrador estructural
                                               |__ unión de triggers
                                               |__ [[miembro]] preinsertados
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como mantenedor que consolida, quiero un esqueleto que ya cumpla la estructura exigida, para concentrarme en la prosa y no perder miembros.
- P2: Como revisor, quiero que el esqueleto sea reproducible desde la selección aceptada, para revisar una fuente clara.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given una consolidación aceptada con dos o más miembros, When se prepara el paraguas, Then se crea un borrador con la unión determinista de todos sus triggers.
- AC-2: Given los miembros seleccionados, When se prepara el borrador, Then contiene un puntero `[[miembro]]` por cada uno, sin omisiones ni duplicados.
- AC-3: Given un trigger repetido con distinto orden o capitalización equivalente, When se forma la unión, Then aparece una sola vez con una regla de orden estable.
- AC-4: Given un borrador que la persona ya empezó a escribir, When vuelve a ejecutar la preparación, Then no pisa su prosa ni los campos ya confirmados.
- AC-5: Given un borrador preparado, When `revisar_paraguas` lo valida, Then la estructura generada satisface sus requisitos y solo puede fallar por la prosa o decisiones aún humanas.
- AC-6: Given fixtures de selección, When corre la suite, Then cubre unión, enlaces, deduplicación, idempotencia y el validador final sin backend real.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: decisión explícita de crear una lección paraguas a partir de una selección válida.
- entrada: miembros y triggers de las lecciones activas candidatas.
- salida: borrador de paraguas con campos estructurales y punteros prellenados.
- candado: si el archivo ya contiene contenido humano, solo se informa y nunca se reemplaza.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO el usuario decide crear un paraguas

  validar miembros y reunir sus triggers
  ordenar y deduplicar triggers y punteros
  ¿el borrador ya contiene prosa humana? -> si, preservarlo

  ENTONCES sembrar la estructura mínima revisable,
           dejando al usuario la descripción y la decisión de aplicar.
```
Promesas: todos los miembros visibles · estructura determinista · no escribe prosa por la persona.

## No funcionales
- SLOs: preparación local, sin llamar a un modelo.
- Seguridad: los punteros proceden de nombres ya validados de lecciones.
- Observabilidad: informa miembros, triggers unidos y si preservó un borrador existente.

## Fuera de alcance
- Archivar miembros, declarar la consolidación terminada o remplazar el texto humano.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: el esqueleto es un borrador asistido, no una mutación del catálogo.
