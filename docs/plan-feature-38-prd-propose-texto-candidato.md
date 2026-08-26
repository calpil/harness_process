# Plan - Feature #38: prd_propose_texto_candidato

Estado: in_progress
Microservicios:
- harness

## Alcance

Agregar al archivo de propuesta un `Candidato despues:` por bloque nuevo,
derivado solo del diff local de la feature. El campo es una ayuda editable: no
resuelve el veredicto ni permite escribir documentos sin el ritual existente.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- `ADR/harness`: la consulta no pudo llegar al Hub por DNS; impacto local
  confirmado en `rust/src/commands/prd.rs`, el formato de propuesta y los
  fixtures de `cli_basics.rs`.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa conecta `commands/prd.rs`, `documentos.rs` y las pruebas de propuesta;
  el candidato debe quedar fuera de la ruta mutante de `prd apply`.

## Delegacion (implementer)

- U1 [AC-1, AC-3, AC-5]: derivar un resumen determinista y acotado del diff de
  la feature; integrarlo solo al bloque recién sembrado.
- U2 [AC-2, AC-4]: preservar bloques existentes y comprobar que la sugerencia
  no modifica el parser ni el ritual de aplicación.
- U3 [AC-6]: agregar fixtures de diff con candidato, sin diff e idempotencia.

## Criterios de cierre (reviewer)

- Cada bloque nuevo contiene candidato o una ausencia explícita; uno existente
  no cambia tras una segunda propuesta.
- El diff de PRD/SDD/architecture permanece vacío hasta `prd apply --yes`.
- Unitarios/integración relevantes, `cargo test`, clippy y harness check verdes.

## Riesgos

- Un resumen demasiado literal puede filtrar ruido del diff; se limita a rutas
  y fragmentos acotados, nunca al diff completo.
- Confundir sugerencia con veredicto; el campo se etiqueta explícitamente y
  `Veredicto:` queda `PENDIENTE`.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: el candidato se conserva como texto editable en
  `prd-diff`, sin LLM ni red.

### Avance 2026-08-24T12:14:19Z
Plan #38 completado: candidato de propuesta determinista, editable y sin escritura documental; unidades U1-U3 trazadas a AC-1..AC-6.
