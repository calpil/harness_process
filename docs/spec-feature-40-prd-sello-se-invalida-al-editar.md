# Spec - Feature #40: prd_sello_se_invalida_al_editar

Estado: approved
Aprobado: 2026-08-24T12:12:02Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-40-prd-sello-se-invalida-al-editar.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: hacer verificable el sello de aplicación de una propuesta compartida; no firma el PRD completo por feature.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Sofía ve `Aplicado:` en una propuesta y asume que el cambio sigue en el PRD. Alguien editó después exactamente el texto que la propuesta había escrito, pero el sello no distingue ese caso.
DESPUES: Sofía sabe si la porción aplicada aún está presente; un cambio posterior invalida el estado de aplicado sin castigar ediciones ajenas del mismo documento.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
prd apply -> sello permanente    prd apply -> sello con huella del texto aplicado
edición posterior -> nada                         |__ comprobación al releer
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario que comparte un PRD entre features, quiero que una aplicación deje de contarse si su texto ya no existe, para no confiar en un sello viejo.
- P2: Como editor, quiero poder cambiar partes no aplicadas sin invalidar aplicaciones que siguen vigentes.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given una propuesta aplicada con `--yes`, When se relee sin edición posterior, Then su sello se reconoce como vigente porque el texto concreto escrito sigue presente.
- AC-2: Given una propuesta aplicada, When se elimina o reemplaza el texto que esa propuesta incorporó, Then la siguiente propuesta o validación la marca no aplicada e impide tratarla como cerrada.
- AC-3: Given un PRD compartido por varias features, When cambia una sección ajena al texto aplicado por una propuesta, Then el sello de esa propuesta permanece vigente.
- AC-4: Given una modificación que conserva literalmente el contenido aplicado aunque cambie el resto del archivo, When se comprueba el sello, Then no se invalida por usar una firma global del documento.
- AC-5: Given una propuesta antigua sin los datos necesarios para comprobarse, When se procesa, Then falla de forma explícita y segura; nunca se declara aplicada por defecto.
- AC-6: Given los fixtures de aplicación, When corre la suite, Then cubre vigencia, invalidación, PRD compartido y compatibilidad segura sin tocar documentos fuera del sandbox.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: `prd apply --yes` y cualquier lectura posterior de su estado.
- registro: por cada escritura aplicada, el texto literal que se esperaba insertar/reemplazar y el documento destino.
- salida: sello `Aplicado:` vigente o invalidado con razón legible.
- candado: la comprobación mira solo el fragmento de la propia propuesta, nunca una firma del archivo completo.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO una propuesta aplicada se vuelve a consultar

  leer el documento y los fragmentos que la propuesta escribió
  ¿cada fragmento sigue presente? -> si, conservar el sello vigente
  ¿falta o cambió alguno? -> si, invalidar la aplicación de esa propuesta

  ENTONCES informar el resultado,
           sin atribuirle al resto del PRD una firma de una sola feature.
```
Promesas: invalida el cambio propio y no el ajeno · falla cerrado · no reescribe sin confirmación.

## No funcionales
- SLOs: comparación local y lineal con los fragmentos registrados.
- Seguridad: no se aceptan rutas fuera del alcance documental ya validado.
- Observabilidad: el diagnóstico nombra propuesta, documento y fragmento que falta.

## Fuera de alcance
- Versionar una firma de archivo por feature o restaurar automáticamente el texto eliminado.
- Alterar la regla de que solo el usuario confirma `prd apply --yes`.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: los datos de integridad pertenecen a la propuesta, no al cuerpo del PRD del usuario.
