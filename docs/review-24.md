# Veredicto de revision - Feature #24: conventions_escalera_y_tests

Veredicto global: **aprobado con limites declarados**.

Spec: `docs/spec-feature-24-conventions-escalera-y-tests.md` (`Estado: approved`, 17 AC)
Plan: `docs/plan-feature-24-conventions-escalera-y-tests.md` (D1-D8, con `Peldano elegido:`)
Evidencia: `docs/impl-24.md`
Reporte de verificacion: `docs/verify-24.md` (17 verde, 0 rojo, 0 manual)

## Estado por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 | cubierto | 5 peldanos numerados + la regla de eleccion |
| AC-2 | cubierto | los 5 citan una feature real en la linea del peldano |
| AC-3 | cubierto | `Peldano elegido:` en conventions y en el rol del lider |
| AC-4 | cubierto | las tres reglas nombradas |
| AC-5 | cubierto | 3 `// NO:` y 3 `// SI:` en Rust, con casos del repo |
| AC-6 | cubierto | la excepcion acotada + el corte que la limita |
| AC-7 | cubierto | `only_verify_should_execute_declared_commands` verde |
| AC-8 | cubierto | cero tests leyendo fuente en la suite real |
| AC-9 | cubierto | la auditoria, con los correctos y los violados |
| AC-10 | cubierto | la prueba del rojo: detecta una violacion sembrada |
| AC-11 | cubierto | exit code 0 con y sin violacion |
| AC-12 | cubierto | silencio total sin `rust/tests/` |
| AC-13 | cubierto | `diff -q` limpio |
| AC-14 | cubierto | los tres roles y los tres agentes |
| AC-15 | cubierto | README + UPDATING + espejo |
| AC-16 | cubierto | la feature paso por su propia escalera |
| AC-17 | cubierto | 250 + 109 tests, clippy 0, smoke verde, check limpio |

Ningun AC quedo sin evidencia ni marcado `manual`.

## Lo que hace creible esta revision

Una feature de convenciones se puede cerrar escribiendo prosa bonita y marcando
todo como cumplido. Tres cosas lo impiden aca, y las tres son verificables:

1. **La escalera se aplico a si misma** (AC-16). El plan trae la tabla de los
   cinco peldanos con el motivo de descarte de cada uno. Salio peldano 1: cero
   comandos, cero flags, cero dependencias. Si la feature que introduce la
   escalera hubiera necesitado un comando nuevo, habria nacido refutada.
2. **La regla cobro su deuda en el acto** (AC-7). El test de la #23 que grepeaba
   `src/**/*.rs` esta reescrito como contrato de comportamiento, no declarado
   excepcion. Ese era el riesgo real: una excepcion en la primera aplicacion
   habria dado el precedente para todas las siguientes.
3. **El chequeo se vio fallar** (AC-10). `conventions_check.sh detecta` siembra
   una violacion y exige que el aviso nombre archivo, linea, test y regla. Sin
   ese modo, "no reporto nada" seria indistinguible de "no sabe reportar" — que
   es exactamente el verde falso que la #23 encontro.

## El hallazgo del que hay que hablar

La regla 3 (prohibido el detector-de-cambios) **condeno a un test que la #23
habia celebrado**. `parse_should_stay_compatible_with_the_310_existing_acs`
asertaba que ningun spec salvo el de la #23 declaraba comandos: una foto del repo
al momento de escribirlo. Se rompio en esta misma feature, en cuanto el spec de
la #24 declaro los suyos, **sin que nada estuviera mal**.

Quedo reescrito como invariante: por cada spec, los comandos que el parser
reporta tienen que ser exactamente los que el spec declara fuera de los bloques
de codigo, contados por un camino independiente. Ahora crece con el repo en vez
de romperse con el.

Vale decirlo sin adornos: ese test lo escribi ayer y lo presente como "la
compatibilidad es un test y no una promesa". Era verdad y era, al mismo tiempo,
un detector-de-cambios. La regla nueva lo detecto en menos de un dia.

## Observaciones (no bloquean)

1. **Solo la regla 2 tiene chequeo automatico.** Las otras dos dependen del
   reviewer. Esta declarado en `conventions.md`, en el README y en el rol, en vez
   de dejar creer que el script cubre las tres. Correcto, pero conviene recordar
   que dos de cada tres reglas siguen dependiendo de que alguien mire.
2. **El chequeo mira `rust/tests/` y no `rust/src/`.** La violacion de la regla 3
   estaba en un test unitario dentro de `src/verificacion.rs` y la encontre a
   mano. Anotado en el backlog del impl.
3. **La deteccion es por forma comun** (`read_to_string(..."*.rs")`). Una ruta
   construida en dos pasos se le escapa. Decision deliberada: falso negativo
   antes que falso positivo, porque un aviso ruidoso se ignora y con el se
   ignoran los reales.
4. **`setup_smoke.ps1` sin correr, novena feature consecutiva.** Ya lo levante en
   `review-23.md` como deuda del repo y no de una feature. Sigue sin decision.

## Riesgo que queda vivo

El riesgo de una feature de convenciones no es romper algo: es escribir reglas
que nadie aplique. Contra eso hay tres defensas (la escalera aplicada a si misma,
la deuda cobrada, el aviso automatico) y una que no se puede automatizar: que el
reviewer se tome en serio el "rechaza" del rol en vez de anotar la violacion como
observacion y aprobar igual. Eso es disciplina, esta dicho como disciplina, y no
se disfraza de garantia.
