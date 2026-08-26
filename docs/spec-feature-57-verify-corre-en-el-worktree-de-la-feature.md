# Spec - Feature #57: verify_corre_en_el_worktree_de_la_feature

Estado: approved
Aprobado: 2026-08-24T12:12:39Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-57-verify-corre-en-el-worktree-de-la-feature.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: ejecutar los comandos de verificación en la raíz de la feature seleccionada, igual que sus documentos.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Tomás verifica una feature aislada. El reporte se guarda en su worktree, pero el comando corre en `main`; puede dar verde contra código viejo o cero casos y dejar una evidencia equivocada en la rama nueva.
DESPUES: Tomás ejecuta `verify --feature` desde cualquier checkout y cada AC corre donde vive el código de esa feature; el informe y lo medido pertenecen al mismo árbol.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
verify --feature -> CWD principal  verify --feature -> worktree de la feature
reporte -> docs feature                              |__ comandos y reporte coherentes
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como implementador, quiero que los AC se ejecuten contra mi worktree, para que la evidencia pruebe el código que escribí.
- P2: Como revisor, quiero que el informe declare el árbol desde el que se midió, para detectar una resolución incorrecta.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given una feature con worktree y un AC que depende de un archivo presente solo allí, When `verify --feature <id>` corre desde el principal, Then el AC queda verde y el comando se ejecuta en el worktree.
- AC-2: Given contenido diferente entre principal y worktree, When corre un AC, Then observa la versión de la feature y nunca una coincidencia accidental del principal.
- AC-3: Given varios AC de la misma feature, When se genera `docs/verify-<id>.md`, Then todos ejecutan en la misma raíz y el reporte se guarda en ese mismo worktree.
- AC-4: Given una feature sin worktree válido, When se verifica, Then conserva el fallback de raíz actual con un diagnóstico explícito, sin abandonar ni elegir un directorio ajeno.
- AC-5: Given un comando que falla, agota timeout o no ejecuta tests, When se corre desde el worktree, Then conserva los estados rojo, timeout y vacío de las features #44 y #46.
- AC-6: Given fixtures de worktree, When corre la suite, Then prueba ejecución desde principal, aislamiento del código, reporte en feature, fallback y conservación de los estados de verificación.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: `harness verify --feature <id>` o foco de worktree inferido.
- selector: worktree registrado/validado para la feature; coincide con el usado para sus docs.
- entrada: comandos declarados por AC-n en el spec de la feature.
- salida: reporte `verify-<id>.md` con estados y evidencia del árbol correcto.
- candado: una raíz resuelta se pasa a todos los AC, sin reconsultar el CWD por comando.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO se verifica una feature

  resolver la raíz de la feature antes de leer o ejecutar AC-n
  ¿hay un worktree válido? -> si no, usar el fallback documentado
  ejecutar cada comando con esa misma raíz como CWD

  ENTONCES escribir el reporte junto al spec medido,
           preservando timeout, recorte y detección de ejecución vacía.
```
Promesas: código y evidencia en el mismo árbol · un CWD por feature · gates existentes intactos.

## No funcionales
- SLOs: no agrega procesos ni red; solo cambia el directorio de trabajo correcto.
- Seguridad: el CWD sale de la ruta de feature validada, no del texto de AC.
- Observabilidad: el reporte y la salida indican la raíz efectiva cuando ayude a diagnosticar.

## Fuera de alcance
- Rediseñar el parser de AC o cambiar las políticas de timeout, pipes y filtros vacíos.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: la semántica de `--feature` prevalece sobre el CWD del checkout desde el que se invoca.
