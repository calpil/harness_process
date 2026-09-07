# Spec - Feature #81: confianza de las referencias mutuas al consolidar

Estado: approved
Aprobado: 2026-09-07T12:53:21Z por USUARIO (confirmacion explicita) - Alan aprobo explicitamente el spec y OBS-1 (confianza local 0.50) en el chat: approved
Plan: docs/plan-feature-81-consolidar-la-declaracion-mutua-en-relacionadas-.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md

## La historia (antes -> despues)

Alan revisa las propuestas de consolidacion y encuentra clases distintas con
confianza 1.00 por citarse mutuamente. Una referencia util entre procedimientos
se presenta como certeza de que ensenan lo mismo. Reproducido antes del cambio
con una copia del catalogo real y el backend apagado: cuatro pares, todos 1.00
(progress/repro-81-antes.txt, en el repositorio principal).

Despues, los pares siguen visibles, con una confianza local menor y una frase
que identifica la declaracion mutua como unica evidencia cuando corresponda.
Si el modelo tambien propone ese par, el informe conserva ambas razones y la
mayor confianza de las dos fuentes. Alan puede evaluar la propuesta conociendo
en que se apoya.

## Hoy -> Como va a funcionar

Hoy: referencia mutua -> confianza 1.00 -> al combinar, gana siempre ese 1.00.
Despues: referencia mutua -> confianza local 0.50 -> combinar con el modelo
mediante el maximo existente -> explicar si hay solo evidencia local o ambas.

0.50 es un peso heuristico de una referencia local, no una probabilidad
calibrada ni un umbral para filtrar candidatos. Esta eleccion queda sometida
a la aprobacion del usuario junto con este spec.

## Recorridos de usuario (priorizados)

- P1: consultar sin modelo o sin propuesta semantica y reconocer una candidata
  sustentada unicamente en referencias mutuas.
- P2: consultar con una propuesta del modelo para el mismo par y ver una sola
  candidata, con las dos razones y una confianza que refleje la evidencia.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given dos lecciones elegibles con referencias mutuas y triggers
  disjuntos, When se informa sin backend, Then el par aparece una sola vez con
  confianza 0.50 y el informe dice que la declaracion mutua es la unica
  evidencia, sin afirmar solapamiento semantico comprobado.
- AC-2: Given ese mismo par, When el backend devuelve cero candidatos, falla o
  devuelve una respuesta inutilizable, Then el par conserva la confianza local
  y la explicacion de unica evidencia; la ausencia de propuesta semantica no
  se interpreta como confirmacion del modelo.
- AC-3: Given el par local y una propuesta valida del modelo para los mismos
  miembros, incluso en orden inverso, When se combinan, Then aparece una sola
  candidata con ambas razones y confianza igual al maximo de las dos fuentes
  (ejemplo: local 0.50 y modelo 0.90 -> 0.90), sin decir que la referencia es
  la unica evidencia. Un candidato exclusivo del modelo conserva su confianza;
  duplicar una fuente o cambiar el orden no aumenta la confianza.
- AC-4: Given relaciones unilaterales, referencias inexistentes o lecciones no
  elegibles, When se informa, Then se conservan las validaciones y diagnosticos
  existentes. La consulta mantiene las lecciones byte-identicas, no genera
  backups ni fusiones y conserva el registro de corrida en la bitacora.
- AC-5: Given la regresion que motivo la feature y la suite del proyecto,
  When se verifica el cambio, Then los tests de comportamiento distinguen los
  casos local, modelo sin propuesta, error y evidencia combinada; la prueba del
  caso local falla contra la version anterior; y una copia del catalogo real
  muestra sus pares locales con confianza menor a 1.00 y unica evidencia.
  Cargo test, clippy y los checks oficiales aplicables terminan verdes.

Comando: `cargo test --manifest-path rust/Cargo.toml --locked`
Comando: `cargo clippy --manifest-path rust/Cargo.toml --all-targets --all-features --locked -- -D warnings`
Comando: `bash tests/consolidar_check.sh`
Comando: `bash tests/setup_smoke.sh`

AC-1 a AC-4 se verifican con las regresiones ejecutadas por AC-5 y evidencia
por criterio. La comparacion antes/despues del catalogo real se registra aparte
con las salidas y el binario utilizado. Las pruebas usan respuestas controladas
sin red ni cuota; no intentan medir la precision de un modelo real.

## Los datos que se tocan

- Entrada: nombre, descripcion, triggers y relacionadas del catalogo existente.
- Salida: confianza y explicacion de los candidatos del informe existente.
- Identidad: conjunto canonico de miembros, igual que hoy.
- Persistencia: se mantiene el registro de corrida de la #80; no cambia el
  formato de las lecciones, el contrato de respuesta del backend ni el backlog.

## Pseudo-codigo (el acuerdo)

    CUANDO una referencia es mutua y sus miembros son elegibles
      proponer el par con confianza local 0.50 y su evidencia local
    SI el modelo entrega propuestas validas
      agregar su evidencia identificada como proveniente del modelo
    PARA cada conjunto de miembros
      unir las evidencias sin duplicar la candidata
      conservar la mayor confianza existente entre sus fuentes
      si solo hay evidencia local, decir que es la unica evidencia
      si hay ambas fuentes, mostrar ambas sin esa afirmacion de exclusividad
    INFORMAR y registrar la corrida como hoy

## No funcionales

- Mantener el calculo local y la complejidad del recorrido actual, sin nuevas
  dependencias, comandos, flags, llamadas al modelo ni umbrales configurables.
- El modelo sigue recibiendo solo nombre, descripcion y triggers; los cuerpos
  y las referencias locales no se agregan al prompt.
- La procedencia de la evidencia es observable en el informe. Se conservan los
  exit codes y el comportamiento ante backend ausente o fallido.

## Fuera de alcance

Fusionar o reescribir lecciones, recalibrar el modelo, cambiar la politica de
consolidacion o implementar el contador del perfil de la #82.

## Observaciones (decisiones pendientes)

- OBS-1: se propone 0.50 para la senal local y conservar el maximo al combinar.
  Ejemplo: modelo 0.90 -> combinada 0.90; modelo 0.20 -> combinada 0.50.
  Es la opcion de menor huella y elimina el 1.00 automatico sin inventar una
  formula acumulativa. La aprobacion del spec acepta esta eleccion.

## Impacto y limites del contexto

El mapa cubre el tema. Graphify identifica por_relacionadas, unir_candidatos y
sus tests. Impacto previsto: rust/src/consolidacion.rs, el informe en
rust/src/commands/leccion.rs, tests existentes de consolidacion y documentacion
del comportamiento. La consulta directa al hub fallo al resolver su servidor;
el impacto se contrasta localmente. La aprobacion y la implementacion no dependen
del hub, conforme al mapa de arquitectura.
