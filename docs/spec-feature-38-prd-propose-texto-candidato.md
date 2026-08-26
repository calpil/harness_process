# Spec - Feature #38: prd_propose_texto_candidato

Estado: approved
Aprobado: 2026-08-24T12:11:52Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-38-prd-propose-texto-candidato.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: solo la propuesta editable de `prd propose`; no aplica ni reescribe documentos del usuario.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Maia termina una feature y abre el diff de documentos. El arnes le hace una pregunta correcta, pero ella debe redactar cada `Despues:` desde cero y puede olvidar lo que cambió.
DESPUES: Maia recibe por documento un candidato breve, trazable al diff de su feature; lo revisa, lo edita si hace falta y sigue conservando el veredicto final.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
feature termina -> bloque      feature termina -> bloque con candidato
sin respuesta sugerida                              |__ alcance + diff acotado
                                                    |__ agente edita/veredicta
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como agente que actualiza documentos, quiero recibir un `Despues:` candidato por documento, para revisar un punto de partida basado en el cambio real.
- P2: Como revisor, quiero distinguir claramente una sugerencia de una decisión del usuario, para que el arnes no convierta una inferencia en una escritura.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given una feature con documentos de alcance y un diff que añade o modifica módulos, When corre `prd propose`, Then cada bloque nuevo lleva `Candidato despues:` derivado de ese diff y conserva `Veredicto: PENDIENTE`.
- AC-2: Given un bloque que ya contiene un veredicto o una edición humana, When `prd propose` se repite, Then no lo pisa ni duplica su candidato.
- AC-3: Given una feature sin diff útil o sin módulos atribuibles, When se propone el bloque, Then declara que no hay candidato en vez de inventar una actualización.
- AC-4: Given un candidato generado, When el agente cambia el veredicto o el texto, Then `prd apply` mantiene el mismo ritual de mostrar, pedir confirmación explícita y escribir solo con `--yes`.
- AC-5: Given cualquier corrida de propuesta, When termina, Then ningún PRD, SDD ni `architecture.md` cambia; únicamente puede cambiar `docs/prd-diff-<id>.md` de la feature.
- AC-6: Given los fixtures de propuesta, When corre la suite, Then prueba candidatos con diff, idempotencia, ausencia de diff y la preservación de documentos protegidos sin invocar un modelo ni red.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: `prd propose --feature <id>` sobre una feature con alcance documental.
- entrada: diff de la feature y rutas/módulos cambiados, siempre acotados a su worktree.
- salida: campo literal `Candidato despues:` dentro de cada bloque de `docs/prd-diff-<id>.md`.
- candado: un bloque ya existente o respondido nunca se regenera ni se sobrescribe.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO se siembra un bloque documental nuevo

  ¿el bloque ya existe o fue respondido? -> si, preservarlo literalmente
  ¿el diff ofrece rutas o términos atribuibles? -> si no, declarar "sin candidato"

  ENTONCES resumir los cambios relevantes como `Candidato despues:`,
           sin convertirlo en veredicto ni escribir el documento objetivo.
```
Promesas: una sugerencia por bloque nuevo · determinista y sin red · nunca aplica por sí sola.

## No funcionales
- SLOs: la propuesta sigue siendo local y termina en el orden de los fixtures habituales.
- Seguridad: el diff y los textos del usuario se tratan como datos; no se ejecutan ni interpolan en shell.
- Observabilidad: la salida nombra cuántos bloques recibieron candidato y cuántos quedaron sin él.

## Fuera de alcance
- Generar redacción libre mediante LLM o decidir el veredicto por el usuario.
- Cambiar el formato y las protecciones existentes de `prd apply`.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: el candidato es siempre editable, no vinculante y se limita al archivo de propuesta de la feature.
