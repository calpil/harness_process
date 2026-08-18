# Veredicto de revision - Feature #29: prd_y_sdd_siempre_al_dia

Veredicto global: **aprobado con limites declarados**.

Spec: `docs/spec-feature-29-prd-y-sdd-siempre-al-dia.md` (23 AC)
Plan: `docs/plan-feature-29-prd-y-sdd-siempre-al-dia.md` (`Peldano elegido: 3`)
Evidencia: `docs/impl-29.md`
Reporte: `docs/verify-29.md` (23 verde, 0 rojo, 0 manual)

## Estado por AC

Los veintitres, cubiertos. Lo que un reviewer tiene que mirar con atencion:

| AC | Estado | Por que no alcanza con leer el test |
| --- | --- | --- |
| AC-8 | cubierto | El test comprueba que el `### Sub` SOBREVIVA. Anclar por seccion lo habria borrado sin que nadie lo notara hasta leer el documento |
| AC-9 | cubierto | Tres formas de cita falsa: rango fuera del archivo, archivo inexistente, rango vacio. Es la unica respuesta refutable por maquina |
| AC-14 | cubierto | **Dos** tests: el que pasaba desde el principio y el que se agrego DESPUES de que el bug rompiera en produccion |
| AC-18 | cubierto | Test negativo: fija que el gate NO mire un archivo. Es raro y es a proposito |
| AC-19 | cubierto | Corre sobre los specs REALES del repo, no sobre fixtures |

## Lo que hace creible esta revision

**La feature se aplico a si misma y el resultado se puede leer.** No es una
metafora: `docs/architecture.md` no mencionaba `doctor.rs` ni `rutas.rs` —dos
features cerradas el mismo dia con `verify` verde y revision escrita— y ahora si.
El `SDD-master.md` publicaba `# SDD Master - <nombre del proyecto>` a Confluence y
ahora dice `Harness Process`. El drift que la feature existe para evitar estaba
ocurriendo aca y quedo corregido por el propio mecanismo.

Si el resultado hubiera sido "los tres bloques dicen `no-aplica`", la feature
seria ceremonia. No lo fue.

## El bug, y por que importa mas que el codigo

Los 15 tests unitarios pasaban. La primera corrida real tambien. **La segunda
duplico** el bloque de modulos en `architecture.md`.

La idempotencia pedia `!contains(antes) && contains(despues)`. Pero el patron mas
comun —"insertar antes de esta linea"— hace que el `despues` CONTENGA al `antes`,
asi que el `antes` sigue presente tras aplicar y el bloque se reaplica.

Tres cosas sobre eso:

1. **Se encontro usando, no testeando.** El test unitario usaba un caso donde el
   `antes` no estaba contenido en el `despues`. Verde y equivocado.
2. **El dano fue real** (un documento del usuario con texto duplicado) y se
   reparo antes de seguir.
3. **Quedo encodeado** en `idempotence_should_hold_when_despues_contains_antes`,
   con el porque adentro del test.

Es `probar-contra-datos-reales` en su forma mas pura, y ya van cinco features
seguidas con el mismo patron: #25 el `[ok]` del hub, #26 el remedio destructivo,
#30 dos razones escritas sin verificar, #36 el chequeo que moria en silencio, y
ahora la idempotencia que no cubria la forma que el uso real produce.

## Lo que verifique ademas de los AC

- **El ritual no se puede saltear**: `prd apply` sin `--yes` sobre este repo, con
  la propuesta completa, no escribio un byte (`git status` limpio en los tres
  documentos).
- **La idempotencia, sobre el repo real**: tercera corrida -> "ya estaba
  aplicada. Nada que escribir", y `architecture.md` con **una** copia del bloque.
- **El gate no se deadlockea**: el test lo fija poniendo el reporte de `verify`
  mas nuevo que la propuesta y exigiendo que el gate igual pase.
- **La cita se verifica de verdad**: `ya-esta docs/prd/PRD-master.md:900-999`
  sobre un archivo de 200 lineas -> exit 2 nombrando la cita.

## Observaciones (no bloquean)

1. **`no-aplica` no es verificable por maquina.** Es la puerta de escape de una
   feature perezosa. Esta dicho en el rol del reviewer —un `no-aplica` en una
   feature que si cambio el producto es `changes_requested`— y eso es disciplina,
   no garantia. Corresponde decirlo asi.
2. **Las senales `Presente en:` buscan el nombre de la feature.** Heuristica
   pobre: un documento puede tener la feature contada con otras palabras. Es una
   ayuda para el agente, no un veredicto, y el codigo lo dice.
3. **El sello no se invalida** si alguien edita el documento despues de aplicar.
   Detectarlo exigiria una firma, que es justo lo que el AC-14 descarta para
   documentos compartidos por N features.
4. **El gate le pide algo al usuario en CADA cierre.** Riesgo numero uno de la
   feature, declarado en el plan. No se puede mitigar del todo.

## Riesgo que queda vivo

Que la propuesta se conteste en piloto automatico: tres `no-aplica` y a cerrar.
Contra eso hay dos defensas estructurales (el alcance lo calcula el binario, la
lista de bloques es cerrada) y una que no lo es: que el reviewer lea los
veredictos en vez del exit code. Esta escrito en el rol, y es lo mismo que ya
paso con `require_tests_to_close`, que fue declarativa durante veintidos
features. La diferencia es que ahora **una** de las tres respuestas se verifica
sola.
