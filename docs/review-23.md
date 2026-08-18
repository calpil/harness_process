# Veredicto de revision - Feature #23: ac_ejecutables_verify

Veredicto global: **aprobado con limites declarados**.

Spec: `docs/spec-feature-23-ac-ejecutables-verify.md` (`Estado: approved`, 20 AC)
Plan: `docs/plan-feature-23-ac-ejecutables-verify.md` (D1-D8, cada uno citando AC)
Evidencia: `docs/impl-23.md`
Reporte de verificacion: `docs/verify-23.md` (20 verde, 0 rojo, 0 manual)

## Estado por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 | cubierto | 7 tests de `parsear()`, incluido el de los specs reales |
| AC-2 | cubierto | test + corrida real sobre `spec-feature-22` y `-20`, exit 0 |
| AC-3 | cubierto | `Estado::Manual` fuera de `bloquea()`, test |
| AC-4 | cubierto | stdout `AC-1  $ true` antes de ejecutar |
| AC-5 | cubierto | test (rastro inexistente) + corrida real con el spec en draft |
| AC-6 | cubierto | timeout desde `rules`, la corrida sigue tras un fallo |
| AC-7 | cubierto | por el test, **no** por el comando que el spec declara (ver abajo) |
| AC-8 | cubierto | `verify_should_write_a_report_per_ac` |
| AC-9 | cubierto | `verify_should_include_output_of_failures` |
| AC-10 | cubierto | `verify_json_should_expose_the_result_per_ac` |
| AC-11 | cubierto | `--solo` corre uno y solo uno; AC inexistente -> exit 2 |
| AC-12 | cubierto | `close_should_stay_identical_without_the_verify_rule` |
| AC-13 | cubierto | exit 2 con mensaje accionable |
| AC-14 | cubierto | nombra los rojos, no nombra los verdes |
| AC-15 | cubierto | reporte mas viejo que el spec -> exit 2 |
| AC-16 | cubierto | el rastro no reaparece al cerrar con reporte verde |
| AC-17 | cubierto | la plantilla de `start` documenta `Comando:` |
| AC-18 | cubierto | README, UPDATING (+ espejo), architecture, verification (+ espejo) |
| AC-19 | cubierto | los tres roles y los tres `.claude/agents/` |
| AC-20 | cubierto | 250 + 109 tests, clippy 0, setup_smoke.sh verde, harness_check limpio |

Ningun AC quedo sin evidencia. Ningun AC quedo `manual`.

## Lo que hace creible esta revision

La feature **se verifica a si misma**: el spec declara sus 20 comandos y
`sh harness_cli verify --feature 23` los corre. Eso mueve la revision de "el
implementer dice que esta cubierto" a "el comando lo ejecuta delante tuyo".

Pero el propio reviewer tiene que resistir la tentacion de leer solo el exit
code, y esta feature lo demuestra en carne propia: en la primera corrida **8 de
los 20 AC dieron verde sin ejecutar nada**, porque `cargo test <nombre>` sale 0
cuando el filtro no matchea ningun test. Se detecto recorriendo comando por
comando y contando cuantos tests corria cada uno (`impl-23.md`, hallazgo 2). Es
la leccion `probar-contra-datos-reales` aplicada al instrumento mismo: la suite
verde no dice que la calibracion sea buena.

## Lo que verifique ademas de los AC

- **El cierre no ejecuta** — y no por promesa: `verificacion::gate()` no llama a
  `ejecutar()`, y el test lo confirma con un comando que dejaria rastro
  (`promesas-estructurales-vs-disciplina`).
- **La barrera del draft sobre datos reales**, no solo en sandbox: el spec real
  de esta feature en `draft` -> exit 2, cero comandos, reporte intacto, spec
  restaurado byte a byte.
- **Compatibilidad medida**: los specs de #1-#22 se parsean en un test (310 AC,
  0 comandos), y dos de ellos se corrieron de verdad.
- **Espejo `templates/` <-> raiz**: `harness_check.sh` limpio.
- **Trazabilidad**: cada D del plan cita sus AC; cada AC tiene su fila arriba.

## Observaciones (no bloquean)

1. **El comando del AC-7 no puede fallar.** Termina en `|| true` y `grep -c`
   devuelve 1 con cero coincidencias: sale 0 siempre. El implementer lo declaro
   en vez de taparlo, y la evidencia real es el test. No bloquea porque el AC
   **esta** cubierto; queda como el ejemplo canonico de la trampa que la feature
   documenta.
2. **Exit codes inconsistentes en `close`**: el gate de spec y el de leccion
   salen con 1, el de verify con 2 porque el spec lo pide asi. Correcto seguir el
   spec aprobado; unificarlos es trabajo de backlog, no de esta feature.
3. **`setup_smoke.ps1` sin correr** por octava feature consecutiva. Ya no es un
   limite de una feature: es una deuda del repo. Merece entrada propia en el
   backlog o una decision explicita de dejar de prometer paridad ps1.

## Riesgo que queda vivo

Esta feature **abrio una superficie de ejecucion** que antes no existia. Las tres
barreras (spec aprobado, invocacion manual, comando impreso) estan implementadas
y testeadas, y el cierre no ejecuta. Lo que ninguna barrera cubre esta dicho sin
adornos en el README: si el usuario aprueba un spec sin leer los comandos, la
proteccion no sirve. Es la unica parte del diseno que depende de una persona, y
esta declarada como tal en vez de disimulada.
