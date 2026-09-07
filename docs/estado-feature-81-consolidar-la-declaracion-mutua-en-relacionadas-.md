# Estado archivado - Feature #81: consolidar: la declaracion mutua en relacionadas no alcanza sola para confianza 1.00
Cerrada: 2026-09-07T23:41:09Z - status=done -

---

# Feature #81: consolidar: la declaracion mutua en relacionadas no alcanza sola para confianza 1.00

Estado: in_progress
Plan: docs/plan-feature-81-consolidar-la-declaracion-mutua-en-relacionadas-.md
Spec: docs/spec-feature-81-consolidar-la-declaracion-mutua-en-relacionadas-.md

Microservicios:

Evidencia:
- 2026-09-07: spec completo en el worktree, Estado: draft; pendiente de aprobacion explicita del usuario. Plan aun es la plantilla de start; no hay implementacion.
- Contexto e impacto: progress/contexto-81.txt; mapa cubre consolidacion. Graphify ubica por_relacionadas/unir_candidatos y tests. Consulta directa al hub ADR/harness fallo por resolucion DNS; contrastar impacto localmente.
- Bug reproducido con copia del catalogo real y backend apagado: cuatro pares con confianza 1.00. Salida completa: progress/repro-81-antes.txt.
- Causa: rust/src/consolidacion.rs:248 asigna confianza 1.0 a toda referencia mutua; unir_candidatos conserva el maximo (linea 287).
- Propuesta pendiente OBS-1: senal local 0.50 (peso heuristico, no probabilidad calibrada), maximo con modelo y explicacion de unica evidencia solo cuando corresponda.
- Lecciones consultadas: probar-contra-datos-reales y criterios-de-cierre-que-se-pueden-fallar; verificar el caso real y la prueba del rojo antes de redactar evidencia.
- Cambio preexistente ajeno en checkout principal: .gitignore modificado; conservarlo.
- Spec abierto en Visual Studio Code con open -a (exit 0); AC-1 a AC-5 y OBS-1 presentados para aprobacion en el chat. No ejecutar approve-spec sin respuesta afirmativa del usuario.
- Alan respondio approved. Aprobacion registrada con approve-spec --yes (sello 2026-09-07T12:53:21Z); check-spec y check-plan verdes tras completar el plan y advance.
- Regresion roja contra produccion anterior: unitario falla con confianza 1.0 (exit 101, progress/test-81-rojo.log); CLI con catalogo aislado falla por confianza 1.00 (exit 1, progress/test-81-cli-rojo.log).
- Delegacion declarada: 2 tareas (implementacion acotada a consolidacion.rs/commands/leccion.rs y revision posterior). El lider prepara tests CLI, catalogo y documentacion; revision empieza cuando terminen escritores.
- Propuesta documental concreta en docs/prd-diff-81.md del worktree (PRD, SDD y mapa), aun SIN aplicar; requiere aprobacion del usuario. Rama destino tambien pendiente para cierre.
- Implementacion delegada terminada y registrada ok: Procedencia explicita, local 0.50, union por maximo, motivos completos y mensaje de unica evidencia local. 29 unitarios de consolidacion verdes.
- Catalogo real comparado con binarios antes/despues (SHA256 en reportes): mismos cuatro pares pasan 1.00 -> 0.50; cada uno informa unica evidencia y documentos byte-identicos. progress/repro-81-antes.txt y progress/repro-81-despues.txt.
- verify en curso (progress/verify-81.log): cargo test verde (159179 ms), clippy verde (2858 ms), suite shell de consolidacion verde (9506 ms); setup_smoke aun ejecutandose.
- harness_check.sh desde repo principal rc=0 (progress/harness-check-81.log); biblioteca dentro de topes, sin candidatas a archivar, leccion usada y ampliada queda 246/250 lineas en worktree.
- 2026-09-07T12:54:32Z Spec aprobado por Alan; plan completo y leido, OBS-1 aceptada: local 0.50 y maximo con modelo. Preparada regresion antes del fix.
