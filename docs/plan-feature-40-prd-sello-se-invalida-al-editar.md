# Plan - Feature #40: prd_sello_se_invalida_al_editar

Estado: in_progress
Microservicios:
- harness

## Alcance

Un sello `Aplicado:` solo cuenta mientras todo `Despues:` que su propuesta
escribió continúe literalmente en su documento. La comprobación es por
contenido del bloque, no por firma global del PRD ni por feature.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- Impacto local (`ADR/harness`; Hub inaccesible por DNS): `documentos.rs`,
  `commands/prd.rs`, el gate de cierre y tests de propuesta/aplicación.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa muestra que la idempotencia existente ya es por contenido; la misma
  evidencia debe gobernar la vigencia del sello compartido.

## Delegacion (implementer)

- U1 [AC-1, AC-2, AC-4]: modelar sello presente versus aplicación vigente por
  los textos `Despues:` de cada bloque.
- U2 [AC-3, AC-5]: invalidar de forma segura el sello sin firmar el documento
  completo y mantener diagnóstico claro para propuestas antiguas/incompletas.
- U3 [AC-6]: fixtures de aplicación, eliminación del texto propio y edición
  ajena en un PRD compartido.

## Criterios de cierre (reviewer)

- Un cambio posterior al texto aplicado invalida el sello y el gate no cuenta
  la propuesta como cerrada; una edición ajena no lo invalida.
- No se agregan firmas por feature ni se reescribe un documento sin `--yes`.

## Riesgos

- Un `Despues:` que contiene `Antes:` no debe duplicarse; se preserva la
  idempotencia existente por contenido literal.
- Propuestas heredadas pueden carecer de datos comprobables: fallan cerradas.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: la huella de vigencia es la presencia del texto
  literal que esa propuesta escribió.

### Avance 2026-08-24T12:26:51Z
Plan #40 completado: sello vigente por contenido literal aplicado, sin firma global; unidades U1-U3 trazadas a AC-1..AC-6.
