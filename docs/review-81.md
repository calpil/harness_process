# Revision independiente - Feature #81
Revisado: approved · 2026-09-07T23:41:02Z · estampado por `harness revision --veredicto`

Resultado tecnico: no se pudo romper con los casos probados. AC-1 a AC-5
cubiertos; no se encontraron cambios de codigo necesarios.

Estado de cierre: pendiente de aprobacion y aplicacion documental, eleccion de
rama destino y registro final de revision. Este informe NO lleva sello
`Revisado:` ni autoriza escribir documentos protegidos o integrar por cuenta propia.

## Contraste por criterio

| AC | Pregunta adversarial | Resultado y evidencia verificable |
| --- | --- | --- |
| AC-1 | ¿Referencias duplicadas o triggers disjuntos producen certeza o mas de un par sin modelo? | Cubierto. Ejecute `tests/consolidar_check.sh relacionadas`: el caso sin backend informa un par a 0.50 con unica evidencia. Fixture y aserciones: `tests/consolidar_check.sh:131`; peso y procedencia: `rust/src/consolidacion.rs:267`; explicacion derivada: `rust/src/consolidacion.rs:218`. |
| AC-2 | ¿Un JSON vacio, malformado, sin candidatos utilizables, un backend fallido o una propuesta para OTRO par confirma la relacion local? | Cubierto. Los cinco casos de `tests/consolidar_check.sh:133` pasan. En CLI independiente probe `{}`, candidatos null/objeto, miembros con tipo incorrecto, miembro desconocido y modelo proponiendo c-d: a-b conserva 0.50 y unica evidencia. Fallbacks: `rust/src/commands/leccion.rs:690`; lectura tolerante: `rust/src/consolidacion.rs:445`; validacion anterior a union: `rust/src/commands/leccion.rs:726`. |
| AC-3 | ¿Orden inverso, fuentes duplicadas, confianza menor/igual/mayor o texto del modelo que copia la razon local alteran el maximo o la procedencia? | Cubierto. Ejecute `tests/consolidar_check.sh combinadas` y CLI independiente con miembros b-a duplicados y confianza 0.00/0.20/0.50/0.90/1.00: un par, maximo 0.50/0.50/0.50/0.90/1.00, ambas razones una vez y sin exclusividad local. Modelo exclusivo c-d conserva 0.20. Union: `rust/src/consolidacion.rs:297`; permutaciones y motivo impostor: `rust/src/consolidacion.rs:887`, `rust/src/consolidacion.rs:965`; recorrido combinado: `tests/consolidar_check.sh:157`. |
| AC-4 | ¿Una referencia unilateral, ausente, archivada o pinneada se acepta al combinar, o la consulta modifica las lecciones? | Cubierto. CLI independiente descarta el par pinneado tanto local como del modelo, conserva diagnosticos unilateral/inexistente/archivada y registra cero candidatos. En cada caso compare el mapa de rutas y bytes de docs antes/despues y ausencia de bkp: intactos. Validaciones: `rust/src/consolidacion.rs:242`, `rust/src/consolidacion.rs:476`; orden y bitacora: `rust/src/commands/leccion.rs:726`; regresion de inmutabilidad: `tests/consolidar_check.sh:145`. |
| AC-5 | ¿Los tests solo reflejan la implementacion, pasan sin ejecutar casos o fallan al usar el catalogo real? | Cubierto. Los tests observan candidatos, confianza, motivos y filesystem; 0.50 es contrato aprobado, no dato creciente. `docs/verify-81.md:13` registra cargo test, clippy estricto, consolidar_check y setup_smoke con exit 0, posterior a la aprobacion. La regresion anterior falla por confianza 1.00: `progress/test-81-rojo.log:8`, `progress/test-81-cli-rojo.log:1` del repo principal. Los cuatro pares reales mantienen identidad y bajan de 1.00 a 0.50: `progress/repro-81-antes.txt:18`, `progress/repro-81-despues.txt:18` del repo principal. |

## Comprobaciones independientes y alcance

- Empece por `harness_cli revision --feature 81`, inspeccione el diff y lei el
  codigo de produccion antes de la evidencia del implementer.
- `check-spec --feature 81` y `check-plan --feature 81`: exit 0, aprobado y
  fresco. Aprobacion del usuario sellada en
  `docs/spec-feature-81-consolidar-la-declaracion-mutua-en-relacionadas-.md:4`
  y trazada en `progress/history.md:14` del repo principal.
- Ejecute los modos `relacionadas` y `combinadas` con
  `HARNESS_BIN=/Users/alan/harness_process/rust/target/debug/harness`, ambos
  exit 0. Los casos adicionales descritos arriba usaron ese mismo binario y
  sandboxes temporales con respuestas controladas, sin red.
- La confianza se combina por maximo y la procedencia por enum; no se deduce
  autoridad buscando palabras en la prosa del modelo. El caso de motivo que
  copia la razon local se verifico por lectura del test y del codigo, y por
  el resultado de la suite completa; no lo repeti separadamente por CLI.
- Los comandos oficiales completos no se repitieron: revise el reporte fresco
  `docs/verify-81.md:3`, sus cuatro comandos y la evidencia por AC de
  `docs/impl-81.md:24`. El check raiz termina limpio en
  `progress/harness-check-81.log` del repo principal. `git diff --check`: exit 0.
- El diff cumple constitution y las tres reglas de tests; no agrega
  dependencias, flags, comandos ni cambios de formato de entrada. El plan
  cita cada AC y justifica extender la superficie existente.
- El contexto registra Graphify e intento de impacto; el hub fallo por DNS.
  El impacto local contrastado queda limitado a calculo/informe de consolidacion,
  regresiones y documentacion. No se verifico el hub remoto.
- `lecciones status`: ninguna candidata a archivar; la clase modificada tiene
  246/250 lineas. El aprendizaje en
  `docs/lecciones/probar-contra-datos-reales.md:221` es una regla reutilizable
  sobre procedencia y evidencia, no una narrativa de esta feature.
- No se probaron modelos reales ni calibracion estadistica: el spec los excluye.
  Las salidas historicas del catalogo real y el rojo previo se revisaron como
  artefactos; no se repitio la reconstruccion del codigo anterior.

## Pendientes administrativos para registrar el veredicto y cerrar

1. Mostrar `docs/prd-diff-81.md` y obtener el SI del usuario; luego ejecutar
   `prd apply --feature 81 --yes`. La propuesta contiene cambios concretos para
   PRD, SDD y architecture y refleja el contrato implementado; aun no esta
   aplicada ni sellada. No hay escrituras protegidas en el diff revisado.
2. Preguntar al usuario la rama destino de integracion. Revisar el rango
   completo de commits y el estado Git al integrar; los cambios de esta feature
   aun no estan commiteados. Conservar el cambio ajeno de .gitignore en la raiz.
3. Registrar la tarea de revision, completar el sello mediante
   `revision --feature 81 --veredicto approved` una vez satisfechos los gates,
   y cerrar con la leccion `probar-contra-datos-reales` y la rama elegida.

Tareas esperadas: 2 · 2026-09-07T12:55:38Z · registrada por `harness revision --esperar-tareas`
Tarea: impl_81 · AC-1,AC-2,AC-3,AC-4 · ok · 2026-09-07T13:01:04Z · registrada por `harness revision --tarea`

## Revision tecnica

Se intentó una revisión delegada independiente; el primer intento agotó el
límite del agente y la continuación completó las comprobaciones adversariales.
No se encontró un defecto técnico en el diff. El hub no respondió por DNS; no
afecta este camino local.

| AC | Veredicto y caso que podría fallar | Evidencia |
| --- | --- | --- |
| AC-1 | Cubierto. Con dos triggers disjuntos y referencias mutuas, el candidato queda en 0.50 y la salida explica que es la única evidencia. | `rust/src/consolidacion.rs:806`, `rust/src/consolidacion.rs:822`, `rust/tests/cli_basics.rs:4536` |
| AC-2 | Cubierto. Backend apagado, vacío, malformado o fallido no confirma ni elimina la señal local; un par que el modelo no propone permanece en 0.50. | `rust/src/consolidacion.rs:944`, `tests/consolidar_check.sh:131`, `tests/consolidar_check.sh:195` |
| AC-3 | Cubierto. Orden inverso, duplicados, valores 0.20/0.50/0.90 y motivo del modelo que imita texto local conservan una sola candidata, ambas razones y el máximo. | `rust/src/consolidacion.rs:888`, `rust/src/consolidacion.rs:965`, `tests/consolidar_check.sh:157` |
| AC-4 | Cubierto. Validación, diagnósticos, bytes de las lecciones, ausencia de backup y bitácora se conservan. | `rust/src/commands/leccion.rs:726`, `rust/src/commands/leccion.rs:732`, `rust/tests/cli_basics.rs:4539` |
| AC-5 | Cubierto. La regresión anterior falla con 1.00; la versión corregida pasa suite Rust, clippy, consolidación, smoke y comparación del catálogo real 1.00 -> 0.50. | `docs/verify-81.md:5`, `progress/repro-81-antes.txt:1`, `progress/repro-81-despues.txt:1`, `progress/test-81-rojo.log:1` |

La revisión no evaluó precisión de un backend LLM real, ejecución PowerShell
real ni integración remota del hub. La solución no los cambia.
Tarea: review_81 · AC-1,AC-2,AC-3,AC-4,AC-5 · ok · 2026-09-07T23:39:02Z · registrada por `harness revision --tarea`
