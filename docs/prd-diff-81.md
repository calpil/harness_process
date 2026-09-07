Aplicado: 2026-09-07T23:40:53Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #81

Propuesta preparada; pendiente del SI del usuario antes de prd apply --yes.

## Documento: docs/prd/PRD-master.md

Veredicto: cambio
Antes:
## 9. Restricciones y supuestos
Despues:
**Consolidacion de lecciones (#81).** Al consultar, una declaracion mutua
propone un par con confianza local 0.50 y se identifica como unica evidencia
cuando no hay propuesta del modelo para ese par. Con ambas fuentes, se muestran
las dos razones y la mayor confianza; repetir una fuente no la aumenta. La
consulta conserva las lecciones y registra su corrida. El peso local es
heuristico, no una probabilidad calibrada.

## 9. Restricciones y supuestos

## Documento: docs/prd/SDD-master.md

Veredicto: cambio
Antes:
- **El modelo propone; lo que muta sale de argv.** La mitad que escribe se
  verifica sin backend y de forma determinista.
Despues:
- **El modelo propone; lo que muta sale de argv.** La mitad que escribe se
  verifica sin backend y de forma determinista.
- **La procedencia de una candidata es un dato (#81).** Las referencias mutuas
  aportan confianza local 0.50; el modelo mantiene su valor. La union canonica
  conserva el maximo y ambas razones, sin premiar duplicados. El informe decide
  si la declaracion mutua es la unica evidencia a partir de esa procedencia,
  nunca interpretando palabras del motivo generado por el modelo. 0.50 es un
  peso heuristico acordado, no una probabilidad calibrada.

## Documento: docs/architecture.md

Veredicto: cambio
Antes:
  que el paraguas herede todos los triggers de lo que archiva, porque `buscar`
  puntua una leccion activa 100 y una archivada 30.
Despues:
  que el paraguas herede todos los triggers de lo que archiva, porque `buscar`
  puntua una leccion activa 100 y una archivada 30. Desde la #81,
  `por_relacionadas()` asigna confianza local 0.50 y `Candidato` conserva su
  procedencia (referencias, modelo o ambas). `unir_candidatos()` deduplica por
  miembros, conserva la mayor confianza y ambas razones. El informe solo declara
  unica evidencia local cuando no hay propuesta del modelo para ese grupo.
