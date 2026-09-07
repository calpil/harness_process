# Plan - Feature #81: confianza de referencias mutuas

Estado: in_progress
Spec: docs/spec-feature-81-consolidar-la-declaracion-mutua-en-relacionadas-.md
Microservicio: harness (consolidacion de lecciones)

## Alcance

Implementar el spec aprobado por Alan el 2026-09-07 con `approved` en el chat.
OBS-1 decidida: confianza local 0.50; combinar por maximo, conservando motivos.

Peldano elegido: 1 (extender lo que ya existe) porque el calculo y el informe
ya viven en consolidacion.rs y commands/leccion.rs; no hace falta otra superficie.

## Impacto y contexto

Paquete: progress/contexto-81.txt del repo principal. El mapa cubre el tema.
Graphify consultado: por_relacionadas, unir_candidatos y sus tests; grafo fresco.
El paquete no trajo impacto; la consulta directa ADR/harness fallo por DNS.
Se contrasta el impacto localmente con los consumidores y el diff.
Lecturas previas: spec/impl de la #41, PRD/SDD y lecciones
probar-contra-datos-reales / criterios-de-cierre-que-se-pueden-fallar.

## Delegacion (implementer)

1. AC-1, AC-2, AC-3, AC-5: fortalecer las regresiones existentes de
   rust/src/consolidacion.rs y rust/tests/cli_basics.rs; agregar casos del backend
   controlado en tests/consolidar_check.sh. Ejecutar el caso local contra el
   codigo anterior y conservar su fallo antes de modificar produccion.
2. AC-1, AC-2, AC-3: corregir confianza y representar la procedencia de evidencia
   en rust/src/consolidacion.rs; mostrar la exclusividad local en el informe de
   rust/src/commands/leccion.rs. Depende de 1 para disponer de la prueba roja.
3. AC-4, AC-5: ejecutar verify y comparar copia del catalogo real antes/despues;
   documentar evidencia por AC en docs/impl-81.md. Depende de 2.
4. AC-1 a AC-5: revision independiente, una vez terminados los escritores,
   sobre spec, diff, casos limite, pruebas y citas en docs/review-81.md.
   Declarar cuenta de tareas antes de delegar y registrar su resultado terminal.

## Documentos al dia

Actualizar el mapa de arquitectura para describir confianza y evidencia local.
Preparar prd propose al terminar: especificar el contrato corregido en el SDD
sin reescribir antecedentes historicos; evaluar PRD maestro por su alcance.
Mostrar la propuesta completa y obtener aprobacion antes de prd apply --yes.

## Criterios de cierre (reviewer)

- AC-1/AC-2: sin modelo, respuesta vacia o error -> confianza local < 1.00,
  una sola candidata, unica evidencia declarada. Caso local observado en rojo
  antes del arreglo y verde despues.
- AC-3: maximo de confianzas, ambas razones, orden inverso, fuente duplicada,
  modelo solo y motivo del modelo que imite etiquetas locales.
- AC-4: validaciones existentes, lecciones byte-identicas, sin backups,
  bitacora de corrida conservada. Suite de consolidacion local sin red/cuota.
- AC-5: cargo test --locked, clippy --all-targets --all-features --locked
  -- -D warnings, tests/consolidar_check.sh y tests/setup_smoke.sh via verify.
  Catalogo real copiado: los cuatro pares antes 1.00 quedan locales 0.50.
- harness_check.sh limpio desde el repo principal con binario construido de la
  feature; revisar tambien espejos, estado Git y frescura de spec/plan.
- Documentos y revision sellados por los comandos del arnes. Rama de integracion
  pendiente de eleccion del usuario antes de close --status done --to.

## Riesgos

0.50 es un peso heuristico aprobado, no una probabilidad calibrada. Preservar
maximo evita premiar duplicados. El informe debe decidir exclusividad por
procedencia de datos, nunca buscando palabras en un motivo generado por LLM.
Los checks deben usar el binario recien construido, no uno instalado previo.
Cambio preexistente ajeno en .gitignore del repo principal: conservar.

## Observaciones (decididas)

OBS-1 aprobada por Alan: local 0.50; maximo con modelo (0.90 -> 0.90,
0.20 -> 0.50). Sin decisiones de implementacion pendientes.

### Avance 2026-09-07T12:54:32Z
Spec aprobado por Alan; plan completo y leido, OBS-1 aceptada: local 0.50 y maximo con modelo. Preparada regresion antes del fix.

---
Cerrado: 2026-09-07T23:41:09Z - status=done - 
