# Spec - Feature #24: conventions_escalera_y_tests

Estado: approved
Aprobado: 2026-08-17T18:12:18Z por USUARIO (confirmacion explicita) - Alan aprobo en el chat tras el ritual (spec mostrado + abierto en editor). Decisiones OBS-1..OBS-4 tomadas por el en la misma vuelta.
Plan: docs/plan-feature-24-conventions-escalera-y-tests.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: `docs/conventions.md` tiene **7 lineas** ("usa Conventional Commits",
"prefiere cambios pequenos"). Nada que un reviewer pueda usar para rechazar algo.
Cuando aparece una capacidad nueva, la pregunta "¿esto es un comando nuevo, un
flag, o alcanza con extender lo que hay?" se contesta por instinto: en 23
features el arnes sumo 12 comandos, y ninguno tuvo que justificar por que no era
un flag. Y sobre los tests no hay **ninguna** regla escrita, asi que la unica
defensa contra un test que no prueba nada es que a alguien se le ocurra mirarlo.

Esto ya paso, y ayer: la feature #23 cerro con un test
(`verify_should_not_be_wired_into_any_hook`) que **lee el codigo fuente** y
asserta sobre su texto. Pasa aunque `verify` este mal cableado y falla ante un
refactor correcto. Nadie lo objeto porque no habia una regla que lo prohibiera.

DESPUES: `docs/conventions.md` lleva **la escalera de huella** (elegir siempre el
peldano de menor huella, y justificar por escrito si bajas) y **las tres reglas
de test** (contratos y no snapshots; prohibido leer el fuente en un test;
prohibido el detector-de-cambios), cada una con el ejemplo de este repo que la
motivo. El reviewer las verifica con la misma seriedad con la que verifica los
AC, y `harness_check.sh` avisa cuando un test lee el fuente — porque una regla
que solo vive en la prosa se olvida en la tercera feature.

Y la deuda que la regla descubre **se paga en esta feature**: el test de la #23
se reescribe como contrato de comportamiento.

## Hoy -> Como va a funcionar

```
HOY                                  DESPUES
capacidad nueva -> "hago un comando"  capacidad nueva -> escalera: ¿extender? ¿flag?
                                           |__ si bajas un peldano, se justifica en el plan
test escrito   -> (nadie lo juzga)    test escrito   -> tres reglas + aviso de harness_check
                                           |__ el reviewer rechaza los que no prueban
```

## Recorridos de usuario (priorizados)

- P1: Como implementer, quiero saber **antes** de escribir codigo si esta
  capacidad justifica un comando nuevo, para no engordar la superficie del arnes.
- P1: Como reviewer, quiero una regla concreta que me deje rechazar un test que
  no prueba nada, en vez de discutirlo por gusto.
- P2: Como Alan, quiero que el arnes me avise si un test empieza a leer el
  fuente, sin tener que acordarme de revisarlo.

## Criterios de aceptacion (Given/When/Then)

<!-- Nota de diseno, aplicando la leccion `criterios-de-cierre-que-se-pueden-fallar`:
     los AC que entregan DOCUMENTACION se verifican con comandos de shell, no con
     tests de Rust. Un test que grepea un markdown es exactamente el
     detector-de-cambios que esta feature prohibe. Los tests de Rust quedan para
     COMPORTAMIENTO (AC-7). Y ningun comando se repite entre dos AC: cada uno
     tiene que poder fallar por su propio criterio. -->

### La escalera de huella

- AC-1: Given `docs/conventions.md`, Then contiene la escalera con sus **cinco
  peldanos numerados y ordenados de menor a mayor huella** (extender lo que
  existe > flag en un comando existente > comando nuevo > superficie nueva >
  dependencia nueva) y dice explicitamente que se elige el de **menor** huella
  que resuelva el problema.
  Comando: `test "$(grep -cE '^[1-5]\. \*\*' docs/conventions.md)" = 5 && grep -q "menor huella que resuelva" docs/conventions.md`
- AC-2: Given cada peldano, Then trae **cuando aplica** y **un ejemplo real de
  este repo** citando su feature (no un ejemplo inventado), para que la escalera
  se pueda usar y no solo leer.
  Comando: `test "$(grep -cE '^[1-5]\. \*\*.*\(#[0-9]+' docs/conventions.md)" = 5`
- AC-3: Given que se elige un peldano que **no** es el de menor huella, Then la
  convencion exige justificarlo por escrito en el plan, con la frase exacta que
  el reviewer va a buscar (`Peldano elegido:`), y el rol del lider lo pide.
  Comando: `grep -q "Peldano elegido:" docs/conventions.md && grep -q "Peldano elegido:" roles/leader.md`

### Las tres reglas de test

- AC-4: Given `docs/conventions.md`, Then contiene las tres reglas con nombre
  propio: **contratos de comportamiento y no snapshots**, **prohibido leer el
  codigo fuente en un test** y **prohibido el test detector-de-cambios**.
  Comando: `grep -q "contratos de comportamiento" docs/conventions.md && grep -q "leer el codigo fuente" docs/conventions.md && grep -q "detector-de-cambios" docs/conventions.md`
- AC-5: Given cada una de las tres reglas, Then trae el **contraejemplo** (lo que
  NO se escribe) y la **version correcta**, en Rust y con casos de este repo.
  Comando: `test "$(grep -c '// NO:' docs/conventions.md)" -ge 3 && test "$(grep -c '// SI:' docs/conventions.md)" -ge 3`
- AC-6: Given la regla de no leer el fuente, Then declara su **unica excepcion
  admitida** y por que no es un agujero: leer un archivo que es **dato de
  entrada** del codigo bajo prueba (los specs que parsea `verificacion.rs`, las
  plantillas que el instalador siembra) no es leer el fuente, porque el test
  seguiria valiendo si la implementacion se reescribiera entera.
  Comando: `grep -q "dato de entrada" docs/conventions.md && grep -q "se reescribiera entera" docs/conventions.md`

### La deuda que la regla descubre

- AC-7: Given el test `verify_should_not_be_wired_into_any_hook` de la #23, When
  se aplica la regla nueva, Then queda reescrito como **contrato de
  comportamiento**: se corre cada comando del arnes contra un spec cuyo
  `Comando:` dejaria un rastro observable en el disco, y se asserta que **ningun**
  comando salvo `verify` deja ese rastro.
  Comando: `cd rust && cargo test only_verify_should_execute_declared_commands`
- AC-8: Given la suite entera, When se busca un test que lea un archivo fuente
  (`.rs`, `.sh`, `.ps1`), Then **no queda ninguno**: el de la #23 era el unico y
  ya no lo hace.
  Comando: `bash tests/conventions_check.sh sin-violaciones`
- AC-9: Given el resto de la suite, When se audita contra las tres reglas, Then
  el resultado queda escrito en `docs/impl-24.md` **caso por caso**, incluyendo
  los que se revisaron y se declararon correctos (no solo los violados).
  Comando: `grep -q "Auditoria de la suite" docs/impl-24.md`

### El aviso automatico

- AC-10: Given un test que lee un archivo fuente, When corre el chequeo, Then lo
  reporta con **archivo, linea y nombre del test**, y nombra la regla.
  Comando: `bash tests/conventions_check.sh detecta`
- AC-11: Given ese chequeo dentro de `harness_check.sh`, Then **avisa y no
  bloquea** (`[i]`, no `[!!]`, y el exit code no cambia): la regla admite
  excepciones justificadas y un gate duro obligaria a inventar un `--force`, que
  es peor.
  Comando: `bash tests/conventions_check.sh no-bloquea`
- AC-12: Given un repo **sin** `rust/tests/`, When corre `harness_check.sh`,
  Then el bloque entero se omite sin ruido: un proyecto que no es Rust no ve
  ninguna diferencia.
  Comando: `bash tests/conventions_check.sh sin-rust`

### Integracion, docs y verificacion

- AC-13: Given `templates/docs/conventions.md`, Then es **identico** al de la
  raiz: una instalacion nueva nace con las mismas reglas y los mismos ejemplos.
  Comando: `diff -q docs/conventions.md templates/docs/conventions.md`
- AC-14: Given los tres roles, Then el lider aplica la escalera y escribe
  `Peldano elegido:` en el plan, el implementer conoce las tres reglas antes de
  escribir tests, y el reviewer **rechaza** los que las violan.
  Comando: `grep -q "escalera" roles/leader.md && grep -q "detector-de-cambios" roles/implementer.md && grep -q "rechaza" roles/reviewer.md`
- AC-15: Given `README.md` y `UPDATING.md` (+ espejo), Then explican la escalera
  y las tres reglas, y por que el aviso no bloquea.
  Comando: `grep -q "escalera de huella" README.md UPDATING.md templates/UPDATING.md`
- AC-16: Given esta feature, When se elige su propio peldano, Then queda
  justificado en el plan con la misma frase que la convencion exige: es
  **documentacion + un chequeo dentro de un script que ya existe**, el peldano
  mas alto posible; no agrega comando, ni flag, ni dependencia.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-24-conventions-escalera-y-tests.md`
- AC-17: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test`, `cargo clippy --all-targets -- -D warnings`, `tests/setup_smoke.sh`
  y `harness_check.sh` siguen verdes.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: no hay evento; son convenciones que se leen y un chequeo que corre
  con `harness_check.sh`.
- interruptor: la ausencia de `rust/tests/` apaga el bloque de chequeo entero.
- candado: no aplica (el chequeo no muta nada).

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien va a agregar una capacidad

  ¿la resuelve extender algo que ya existe?  -> ese peldano, listo
  ¿la resuelve un flag en un comando de hoy? -> ese peldano, listo
  si bajas mas -> escribis "Peldano elegido: <n> porque <razon>" en el plan

CUANDO corre harness_check.sh

  ¿existe rust/tests/?               -> si no, no se dice nada
  ¿algun test lee un .rs/.sh/.ps1?   -> se avisa con archivo, linea y test
                                        SIN bloquear y SIN cambiar el exit code
```

## No funcionales

- El chequeo nuevo no agrega dependencias (Articulo 6) ni tarda mas de un
  segundo: es `grep` sobre `rust/tests/`.
- Cero comandos nuevos, cero flags nuevos: es el peldano mas alto de la escalera
  que el propio spec introduce. Si esta feature necesitara un comando, la
  escalera naceria contradicha.

## Fuera de alcance

- **Bloquear** por leer el fuente (OBS-2 lo decide: avisa).
- Auditar los tests de los proyectos que instalan el arnes: la convencion se
  siembra, el cumplimiento es de cada equipo.
- Reescribir tests que no violan ninguna regla: no es una feature de refactor.
- Detectar automaticamente snapshots y detectores-de-cambios. Requiere entender
  que dato "se espera que cambie", y eso no se grepea. Esas dos reglas las
  verifica el reviewer; solo la de leer el fuente tiene chequeo.

## Observaciones (decididas por Alan el 2026-08-17)

- OBS-1 **DECIDIDA: la excepcion se admite, acotada a dato de entrada.** Leer un
  archivo que es entrada del codigo bajo prueba (los specs que parsea
  `verificacion.rs`, la plantilla que siembra el instalador) no es leer el
  fuente: el test seguiria valiendo si la implementacion se reescribiera entera.
  Con ese corte quedan DENTRO de la regla
  `parse_should_stay_compatible_with_the_310_existing_acs` y
  `the_seeded_template_should_match_what_the_binary_writes`, y queda FUERA el de
  la #23. -> AC-6.
- OBS-2 **DECIDIDA: avisa, no bloquea.** `[i]` con archivo, linea y test.
  Coherente con el curador de la #21 (informa; solo mueve con `--aplicar`), y
  porque un gate duro sobre una regla con excepciones empuja a inventar un
  `--force`, que es peor que el aviso. -> AC-10, AC-11.
- OBS-3 **DECIDIDA: se reescribe como contrato de comportamiento.** Una excepcion
  en la primera aplicacion de la regla la vaciaria: el proximo test que grepee
  fuente citaria el precedente. El test nuevo corre los comandos del arnes contra
  un spec cuyo `Comando:` deja rastro, y assertea que solo `verify` lo deja.
  -> AC-7, AC-8.
- OBS-4 **DECIDIDA: la plantilla lleva el mismo texto, con los ejemplos reales.**
  Una escalera sin casos concretos es una lista que nadie sabe aplicar; los
  ejemplos ensenan aunque sean de otro proyecto. Ademas mantiene el espejo
  identico, que es lo que `harness_check.sh` verifica. -> AC-13.
