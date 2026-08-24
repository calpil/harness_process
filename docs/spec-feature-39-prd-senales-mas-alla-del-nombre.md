# Spec - Feature #39: prd_senales_mas_alla_del_nombre

Estado: approved
Aprobado: 2026-08-24T12:11:57Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-39-prd-senales-mas-alla-del-nombre.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: enriquecer las señales de `prd propose`; no infiere ni aplica veredictos.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Leo revisa una propuesta y lee “Ausente en” aunque el documento ya describe el cambio con otro nombre. La señal buscó solo el slug de la feature y le hace perder tiempo verificando una falsa ausencia.
DESPUES: Leo recibe evidencia basada también en los módulos del diff y términos del spec, con la ubicación que permite comprobarla antes de decidir.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
prd propose -> nombre solo      prd propose -> señales compuestas
                                           |__ slug normalizado
                                           |__ rutas/módulos y términos del spec
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como agente que revisa un PRD, quiero señales que entiendan el vocabulario técnico del cambio, para no marcar como ausente una descripción ya presente.
- P2: Como revisor, quiero saber qué término y qué línea sostienen una señal, para poder refutarla rápidamente.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given un documento que no contiene el nombre de la feature pero sí un módulo nuevo o término relevante de su spec, When corre `prd propose`, Then la señal lo reporta como `Presente en:` con archivo, línea y evidencia encontrada.
- AC-2: Given un documento que no contiene ninguno de los términos o módulos disponibles, When se calcula la señal, Then conserva `Ausente en:` y explica qué evidencia faltó sin afirmar que el producto está actualizado.
- AC-3: Given nombres con guiones, guiones bajos, mayúsculas o rutas, When se derivan tokens, Then la normalización no pierde las coincidencias útiles ni convierte fragmentos triviales en evidencia.
- AC-4: Given varias señales para un mismo documento, When se renderiza el bloque, Then la salida es determinista, acotada y separa coincidencias de nombre, spec y módulos.
- AC-5: Given una señal encontrada, When el usuario revisa la línea indicada, Then esa línea contiene realmente el término reportado; no hay citas inventadas.
- AC-6: Given los fixtures, When corre la suite, Then cubre el falso “ausente”, la ausencia real, normalización y exactitud de las ubicaciones, sin red ni modelo.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: siembra de un bloque por `prd propose --feature <id>`.
- fuentes: nombre de feature, texto aprobado del spec y módulos/rutas atribuibles al diff.
- salida: `Presente en:` o `Ausente en:` con el tipo de coincidencia y localización verificable.
- candado: solo términos normalizados y con longitud/significado suficiente entran al conjunto de búsqueda.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO `prd propose` arma las señales de un documento

  reunir slug, términos específicos del spec y módulos del diff
  normalizar y descartar palabras de ruido
  ¿alguna evidencia aparece literalmente? -> si no, reportar ausencia honesta

  ENTONCES emitir las coincidencias con línea y procedencia,
           sin decidir el veredicto documental.
```
Promesas: evidencia comprobable · mismo resultado para la misma entrada · no usa LLM.

## No funcionales
- SLOs: búsqueda local y acotada por documento.
- Seguridad: los términos del spec se buscan como texto, nunca como comandos o expresiones ejecutables.
- Observabilidad: cada señal declara de qué fuente provino la coincidencia.

## Fuera de alcance
- Reescribir documentos o asumir que una coincidencia equivale a un veredicto `ya-esta`.
- Introducir una dependencia de red o un modelo semántico.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: una coincidencia es ayuda de revisión, no autorización para aplicar cambios.
