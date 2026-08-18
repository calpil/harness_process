# Evidencia de implementacion - Feature #24: conventions_escalera_y_tests

Spec: `docs/spec-feature-24-conventions-escalera-y-tests.md` (`Estado: approved`,
17 AC, cada uno con su `Comando:` y ninguno repetido)
Plan: `docs/plan-feature-24-conventions-escalera-y-tests.md` (D1-D8)
PRD: `docs/prd/PRD-master.md` (hito 2)

## Peldano elegido: 1 (extender lo que ya existe)

La feature se aplico su propia escalera antes de escribirse (AC-16, tabla
completa en el plan). Resultado: **cero comandos nuevos, cero flags, cero
dependencias**. `docs/conventions.md` y `harness_check.sh` ya existian; el
chequeo es un bloque mas dentro del script, con la misma forma que el bloque de
lecciones de la #17 (que tambien se omite entero si falta su directorio).

`tests/conventions_check.sh` no es superficie nueva: es un test, hermano de
`tests/setup_smoke.sh`, y existe para que los AC-8/10/11/12 tengan cada uno un
comando que pueda fallar.

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `docs/conventions.md` (+ espejo) | D1, D2, D6 | De 7 lineas a la escalera (5 peldanos con ejemplo real) y las tres reglas de test con `// NO:` / `// SI:` |
| `rust/tests/cli_basics.rs` | D3 | `verify_should_not_be_wired_into_any_hook` -> `only_verify_should_execute_declared_commands` (contrato de comportamiento) |
| `rust/src/verificacion.rs` | D3 | `parse_should_stay_compatible_with_the_310_existing_acs` -> `parse_should_only_report_commands_the_spec_actually_declares` (invariante en vez de snapshot) |
| `harness_check.sh` (+ espejo) | D4 | Bloque de aviso: archivo, linea, nombre del test y la regla. No toca `failures` |
| `tests/conventions_check.sh` | D5 | NUEVO. Cuatro modos, uno por AC, incluida la prueba del rojo |
| `templates/roles/*.md` -> `roles/*.md` -> `.claude/agents/*.md` | D6 | Lider elige peldano, implementer conoce las reglas, reviewer **rechaza** |
| `README.md`, `UPDATING.md` (+ espejo) | D6 | La escalera, las tres reglas y por que el aviso no bloquea |

## Evidencia por AC

`sh harness_cli verify --feature 24` corre los 17 comandos que el spec declara.

| AC | Evidencia |
| --- | --- |
| AC-1 | 5 peldanos `^[1-5]. **` + la frase "menor huella que resuelva" |
| AC-2 | los 5 peldanos citan su feature en la misma linea (#24, #21, #20, #17, #15) |
| AC-3 | `Peldano elegido:` en `conventions.md` y en `roles/leader.md` (paso 5.2) |
| AC-4 | las tres reglas nombradas en el parrafo de apertura, en minuscula y en una linea |
| AC-5 | 3 bloques `// NO:` y 3 `// SI:`, en Rust y con casos de este repo |
| AC-6 | la excepcion "dato de entrada" + el corte "se reescribiera entera" |
| AC-7 | `only_verify_should_execute_declared_commands` verde |
| AC-8 | `bash tests/conventions_check.sh sin-violaciones` |
| AC-9 | esta seccion ("Auditoria de la suite") |
| AC-10 | `bash tests/conventions_check.sh detecta` — la prueba del rojo |
| AC-11 | `bash tests/conventions_check.sh no-bloquea` |
| AC-12 | `bash tests/conventions_check.sh sin-rust` |
| AC-13 | `diff -q docs/conventions.md templates/docs/conventions.md` |
| AC-14 | los tres roles y los tres `.claude/agents/` |
| AC-15 | README + UPDATING + espejo |
| AC-16 | `Peldano elegido:` en el plan, con la tabla de los 5 peldanos |
| AC-17 | 250 + 109 tests, clippy 0, `setup_smoke.sh` verde, `harness_check.sh` limpio |

## Auditoria de la suite

359 tests revisados contra las tres reglas. Se listan **todos** los casos
mirados, no solo los violados: un informe que solo enumera problemas no deja
saber si se reviso todo.

### Regla 2 (prohibido leer el codigo fuente): 3 casos, 1 violacion

Son los tres unicos lugares de la suite que leen un archivo del repo (buscados
por `CARGO_MANIFEST_DIR`, que es la unica forma de salir del sandbox):

| Caso | Que lee | Veredicto |
| --- | --- | --- |
| `seed_guia_real` (`cli_basics.rs:1365`) | `templates/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`, para sembrarla en la fixture | **Correcto**: es dato de entrada. El test seguiria valiendo si el lector de lecciones se reescribiera entero |
| `the_seeded_template_should_match_what_the_binary_writes` (`cli_basics.rs:1542`) | `templates/docs/perfil-usuario.md`, para compararla con lo que escribe el binario | **Correcto**: es un contrato entre dos artefactos (la plantilla sembrada y la generada), justo el tipo de assercion que la regla 1 pide |
| `parse_should_stay_compatible_...` (`verificacion.rs:594`) | `docs/spec-feature-*.md`, el corpus que el parser tiene que leer | **Correcto** como lectura (dato de entrada), pero **violaba la regla 3**: ver abajo |
| `verify_should_not_be_wired_into_any_hook` (#23) | `src/**/*.rs` y `setup_harness.sh`, para grepear `verify::run` | **VIOLACION**. Reescrito |

El resto de los `read_to_string` de la suite (unos 40) leen archivos que el
propio test creo en su sandbox para asserta sobre la salida del binario. Eso no
es leer el fuente: es leer el resultado.

### Regla 3 (prohibido el detector-de-cambios): 1 violacion, encontrada al correr

`parse_should_stay_compatible_with_the_310_existing_acs` asertaba que **ningun
spec salvo el de la #23** declaraba comandos. Era una foto del repo al momento de
escribirlo, y **se rompio en esta misma feature**, en cuanto el spec de la #24
declaro los suyos — sin que nada estuviera mal.

Reescrito como `parse_should_only_report_commands_the_spec_actually_declares`,
que assertea el invariante: por cada spec, la cantidad de comandos que el parser
reporta tiene que ser exactamente la que el spec declara fuera de los bloques
` ``` `. El conteo de control se calcula por un camino distinto al de `parsear`,
asi que el acuerdo sobre 24 specs reales significa algo. Ahora el test crece con
el repo en vez de romperse con el.

Es la mejor prueba de que la regla no es decorativa: la escribi por la manana y a
la tarde condeno a un test que yo mismo habia celebrado en la #23.

### Regla 1 (contratos, no snapshots): 14 candidatos, 0 violaciones

Revise todos los `assert_eq!(...len(), N)` de la suite (`curador.rs`,
`buscar.rs`, `journey.rs`, `progress.rs`, `cli_basics.rs`). En todos, la N
corresponde a **una fixture que el propio test siembra** ("cree 2 lecciones,
espero 2 hallazgos"). Eso no es congelar un dato que se espera que cambie: es el
resultado esperado de una entrada controlada, que es exactamente un contrato.

No hay en la suite ningun assert sobre un numero de version, un conteo de
lecciones reales del repo ni un catalogo que crezca solo.

## La deuda de la #23, pagada

El test viejo grepeaba `src/**/*.rs` buscando `verify::run`:

```rust
assert!(!texto.contains("verify::run"), "...");
```

Fallaba de las dos maneras que la regla describe. Pasaba aunque `verify`
estuviera mal cableado: bastaba invocarlo por otro camino (un `Command::new` a la
CLI, un alias). Y fallaba ante un refactor correcto: renombrar la funcion lo
rompia sin que cambiara nada del comportamiento. Ademas, cuando la #23 documento
`verify` en `setup_harness.sh` —que es obligatorio por su AC-18— el test empezo a
fallar y hubo que ensenarle a distinguir prosa de codigo, ignorando lo que
estuviera entre backticks. Ese parche fue el sintoma: estaba probando la forma
del texto.

La version nueva mira el disco:

```rust
escribir_acs(&spec, "- AC-1: uno.\n  Comando: `touch rastro-de-ejecucion.txt`");
for args in [ /* status, next, check-plan, check-spec, autocheck, nudge,
                 advance, leccion list, journey, buscar, close */ ] {
    let _ = cmd(&bin).args(&args).output();
    assert!(!rastro.exists(), "`{}` ejecuto el Comando: declarado", args.join(" "));
}
cmd(&bin).args(["verify", "--feature", "1"]).assert().success();
assert!(rastro.exists(), "verify no ejecuto el comando: el test no prueba nada");
```

Once comandos del arnes contra un spec que declara `touch rastro.txt`, y el
disco como juez. Sobrevive a cualquier reescritura de la implementacion.

**El control positivo es la mitad del test.** Sin la ultima assercion, el test
pasaria igual si el rastro fuera imposible de crear —por ejemplo si el sandbox no
tuviera permisos, o si el spec quedara en draft— y estaria dando verde sin probar
nada. Es exactamente la trampa que la #23 encontro con `cargo test <nombre>`.

## La prueba del rojo sobre el chequeo nuevo

El criterio de cierre lo exige y la leccion lo describe: un chequeo que nunca se
vio fallar no verifica. `tests/conventions_check.sh detecta` siembra un test que
lee `src/cli.rs`, corre `harness_check.sh` y exige que el aviso nombre las cuatro
cosas:

```
[Ok] detecta: reporta archivo, linea, nombre del test y la regla
```

El fixture se borra siempre, tambien si el assert falla (`trap limpiar EXIT`): un
test no puede dejar el repo sucio.

Y `no-bloquea` compara el exit code de `harness_check.sh` con y sin violacion
presente: **0 en los dos casos**. El aviso avisa y nada mas.

## Limites declarados

- **Solo la regla 2 tiene chequeo automatico.** Las reglas 1 y 3 exigen saber que
  dato "se espera que cambie", y eso no se grepea: las verifica el reviewer. Esta
  dicho en `conventions.md`, en el README y en el rol del reviewer, en vez de
  dejar creer que el script cubre las tres.
- **El chequeo detecta la forma comun, no todas.** Grepea
  `read_to_string(...".rs"/".sh"/".ps1")`. Un test que construya la ruta en dos
  pasos se le escapa. Es un aviso, no un gate: prefiero que no de falsos
  positivos (leccion `probar-contra-datos-reales`) a que atrape todo y se
  vuelva ruido.
- **`tests/setup_smoke.ps1` sigue sin correrse** (novena feature). Ya esta
  declarado como deuda del repo en `review-23.md`.

## Para el backlog

- **El chequeo podria mirar tambien los tests unitarios de `rust/src/`**, no solo
  `rust/tests/`. Hoy solo mira los de integracion; el caso de `verificacion.rs`
  lo encontre a mano.
- **Un aviso simetrico para la regla 3** cuando un `assert_eq!` compara contra un
  literal que aparece tambien en `docs/` o en un catalogo. Heuristico y con
  riesgo de falsos positivos: hay que disenarlo con cuidado o no hacerlo.
