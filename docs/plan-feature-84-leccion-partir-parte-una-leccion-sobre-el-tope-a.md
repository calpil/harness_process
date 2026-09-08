# Plan - Feature #84: leccion partir: parte una leccion sobre el tope a referencias/ con informe y --aplicar, reconoce las secciones por feature en sus formas reales, y el check resume en una linea las lecciones sobre el tope

Estado: in_progress
Microservicios:
- (sin servicios)

## Alcance
Un subcomando `leccion partir <clase> [--aplicar] [--seccion <titulo>]...` que
hace la parte mecanica del paso 3 de la guia (mover a `referencias/` las
secciones que cuentan una sola feature) con informe primero; la deteccion de
esas secciones en las formas reales de los titulos, compartida con el contrato
de particion; y el aviso de `harness_check.sh` colapsado a una linea. Solo el
binario, `harness_check.sh` (dos copias), su test y los docs. Medido sobre las
cuatro lecciones de realestate del 2026-09-08.

## Impacto entre microservicios
- Sin servicios: solo el arnes.

## Consulta al grafo (graphify)
- `harness contexto --feature 84`: lecciones que aplican:
  criterios-de-cierre-que-se-pueden-fallar, promesas-estructurales-vs-disciplina,
  remedios-que-la-herramienta-sugiere.

## Delegacion (implementer)
- `rust/src/lecciones.rs`: `Seccion` (titulo, inicio, fin, lineas, es_canonica,
  cuenta_una_feature), `Leccion::secciones()`, `secciones_por_feature` con el
  criterio de AC-2, `contrato_de_particion` con el comando (AC-2, AC-8).
- `rust/src/particion.rs` (nuevo): `Plan` puro (candidatas, saldo, falta,
  sugeridas), `slug_de_referencia`, `cabecera_de_referencia`, `aplicar`
  (respaldo via `curador::respaldar`, archivos, punteros, frontmatter) (AC-1,
  AC-3, AC-4, AC-5, AC-6).
- `rust/src/commands/leccion.rs::partir` + `cli.rs` (`Partir { nombre, aplicar,
  seccion: Vec<String> }`); `status` sugiere el comando (AC-8).
- `harness_check.sh` y `templates/harness_check.sh`: un `[i]` para todas (AC-7);
  `tests/leccion_tope_check.sh` con el caso de dos lecciones sobre el tope.
- Tests primero, rojos contra HEAD: unitarios en `particion.rs`/`lecciones.rs`
  con los titulos reales de realestate; integracion en `rust/tests/cli_basics.rs`.
- Docs: README, UPDATING x2, architecture, guia x2, AGENTS en sh/ps1 (AC-10).
- Corpus de `verificacion.rs`: el AC-9 (MANUAL).

## Criterios de cierre (reviewer)
- Cada AC con test o comando; rojos por aserto contra HEAD.
- `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`; sin `cargo fmt` a secas.
- AC-9 medido sobre una copia de realestate y escrito en el review con los numeros.

## Riesgos
- Un titulo de clase con `#N` accidental se moveria: por eso las canonicas
  nunca se mueven y el informe muestra todo antes de `--aplicar`.
- El indice `## Referencias` existe con varios nombres ("Referencias por tema",
  "Referencias (el detalle, caso por caso)"): se toma el primer `## Referencias`.

## Observaciones (decisiones pendientes)
- OBS-1..3: ver el spec; se preguntan con la aprobacion.

---
Cerrado: 2026-09-08T23:59:58Z - status=done - 
