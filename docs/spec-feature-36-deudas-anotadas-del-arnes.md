# Spec - Feature #36: deudas_anotadas_del_arnes

Estado: approved
Aprobado: 2026-08-18T02:36:32Z por USUARIO (confirmacion explicita) - Alan aprobo en el chat tras el ritual. OBS-1: exit 2 para los tres gates de close (se mueve solo el de spec). OBS-2: la poda del registro de rutas ocurre en cada consulta de violaciones.
Plan: docs/plan-feature-36-deudas-anotadas-del-arnes.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: cada `docs/impl-*.md` termina con una seccion **"Para el backlog"**. Ahi
el arnes se anota lo que dejo a medias: un exit code inconsistente, un flag que
solo acepta un valor, un chequeo que mira la mitad de los archivos, un registro
que crece sin poda. Seis en total, escritas por honestidad y con el detalle
suficiente para arreglarlas.

Y ahi se quedaron. Vivian **solo en prosa**, dentro de documentos de cierre que
nadie relee. No estaban en el backlog, asi que `next` nunca las ofrecio y
`journey` nunca las conto como hueco. Una nota que no entra al backlog no es una
deuda registrada: es una deuda olvidada con estilo.

DESPUES: las seis estan pagadas. Cada una con su AC, su comando y su test.
Ninguna es grande —van de una linea a veinte— y por eso van juntas: seis specs
para seis correcciones de pocas lineas seria ceremonia, no proceso.

## Hoy -> Como va a funcionar

```
HOY                                    DESPUES
impl-23: "para el backlog: unificar     los tres gates de close salen con el
  el exit code de los tres gates"       mismo codigo, y hay un test que lo fija
  -> nadie lo lee de nuevo
```

## Recorridos de usuario (priorizados)

- P1: Como agente, quiero que `close` falle siempre con el mismo codigo, para no
  tener que aprenderme cual gate dio cual.
- P1: Como implementer, quiero iterar sobre dos AC sin correr los veinte.
- P2: Como mantenedor, quiero que el chequeo de convenciones mire **todos** los
  tests, no la mitad.

## Criterios de aceptacion (Given/When/Then)

<!-- Comportamiento con tests; documentacion con greps. Ningun comando repetido.
     Una deuda por AC, con el impl que la anoto citado. -->

### Deuda de `impl-23`: el exit code de los gates de `close`

- AC-1: Given los tres gates de `close --status done` (spec sin aprobar, leccion
  sin declarar, reporte de verify en rojo), Then los tres salen con el **mismo**
  exit code. Hoy el de spec sale 1 y los otros dos salen 2.
  Comando: `cd rust && cargo test close_gates_should_share_one_exit_code`
- AC-2: Given ese cambio, Then `harness_check.sh` y los hooks siguen
  distinguiendo "gate" de "uso invalido" como antes: no se rompe ningun consumidor.
  Comando: `cd rust && cargo test close_should_keep_usage_errors_separate_from_gates`

### Deuda de `impl-23`: `--solo` con varios AC

- AC-3: Given `verify --solo AC-3,AC-7`, Then corre exactamente esos dos y
  ninguno mas.
  Comando: `cd rust && cargo test verify_solo_should_accept_several_acs`
- AC-4: Given `--solo` con un AC que el spec no declara, Then falla nombrando
  **cual** falta, aunque los otros existan.
  Comando: `cd rust && cargo test verify_solo_should_name_the_missing_ac`

### Deuda de `impl-24`: el chequeo de convenciones mira medio repo

- AC-5: Given un test **unitario** dentro de `rust/src/` que lee un archivo
  fuente, When corre el chequeo de convenciones, Then lo reporta igual que si
  estuviera en `rust/tests/`. La unica violacion historica estaba justamente en
  `src/` y se encontro a mano.
  Comando: `bash tests/conventions_check.sh detecta-en-src`
- AC-6: Given ese alcance nuevo, Then la suite real sigue sin violaciones: el
  chequeo mas amplio no reporta nada que este bien.
  Comando: `bash tests/conventions_check.sh sin-violaciones`

### Deuda de `impl-26`: el registro de rutas crece sin poda

- AC-7: Given una entrada de `progress/.rutas_arnes` cuya ruta ya **no** aparece
  modificada en git (se commiteo, o se revirtio), Then se poda: la exencion ya no
  tiene sentido.
  Comando: `cd rust && cargo test rutas_registro_should_drop_entries_that_are_no_longer_dirty`
- AC-8: Given la poda, Then **nunca** elimina una entrada que todavia protege
  algo: si la ruta sigue modificada y el mtime coincide, se conserva.
  Comando: `cd rust && cargo test rutas_registro_should_keep_live_exemptions`

### Deuda de `impl-25`: doctor no mira el contenido de los hooks

- AC-9: Given un `.claude/settings.json` cuyo hook apunta a una ruta distinta de
  `bin/harness-hook`, When corre `doctor`, Then lo reporta: hoy solo verifica que
  el runtime exista, asi que un hook mal apuntado pasa desapercibido.
  Comando: `cd rust && cargo test doctor_should_detect_a_hook_pointing_to_another_path`
- AC-10: Given hooks correctos, Then no reporta nada: el chequeo mas fino no
  puede volverse ruidoso.
  Comando: `cd rust && cargo test doctor_should_stay_quiet_with_well_wired_hooks`

### Deuda del PRD (#27): `leccion list` con nombres largos

- AC-11: Given un catalogo con un nombre mas largo que 28 caracteres, When corre
  `leccion list`, Then la columna se calcula por el nombre mas largo y ninguna
  fila desborda.
  Comando: `cd rust && cargo test leccion_list_should_size_the_column_to_the_longest_name`
- AC-12: Given ese cambio, Then el orden, los campos, el `--json` y los exit
  codes quedan **exactamente** como estaban: es formato de salida y nada mas.
  Comando: `cd rust && cargo test leccion_list_should_not_change_order_fields_or_json`

### Que el backlog deje de perderse

- AC-13: Given las seis deudas pagadas, Then las entradas #27 y #31-#35 del
  backlog quedan cerradas citando esta feature, para que no queden dos veces.
  Comando: `bash tests/deudas_check.sh backlog-cerrado`
- AC-14: Given el rol del implementer, Then dice que una nota de "Para el
  backlog" tiene que entrar al backlog con `harness_cli add` en el mismo cierre,
  no quedarse solo en el impl.
  Comando: `grep -q "Para el backlog" roles/implementer.md`
- AC-15: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `tests/setup_smoke.sh` y `harness_check.sh` siguen verdes.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: ninguno nuevo; son correcciones sobre caminos que ya existen.
- interruptor: no aplica.
- candado: la poda del registro solo borra entradas que ya no eximen nada.

## Pseudo-codigo (el acuerdo)

```
close falla por un gate        -> siempre el mismo exit code
verify --solo AC-3,AC-7        -> corre esos dos
chequeo de convenciones        -> mira rust/tests/ Y rust/src/
poda de .rutas_arnes           -> saca lo que ya no esta modificado
doctor mira los hooks          -> ademas de que exista, que APUNTE bien
leccion list                   -> ancho = el nombre mas largo
```

## No funcionales

- Sin dependencias nuevas (Articulo 6) y sin comandos nuevos: las seis son
  cambios sobre lo que ya existe (peldano 1 de la escalera).
- Ningun cambio de contrato observable salvo los que los AC declaran.

## Fuera de alcance

- Las deudas que quedaron **declaradas como limite** y no como backlog: que
  `doctor` no valide el handshake de PostgreSQL, que el `PreToolUse` solo exista
  para Claude, que la deteccion de binario viejo sea por mtime. Esas fueron
  decisiones, no olvidos.
- `doctor --fix` y la consolidacion con LLM (#28): features propias.

## Observaciones (decididas por Alan el 2026-08-18)

- OBS-1 **DECIDIDA: exit 2 para los tres.** Se mueve unicamente el gate de spec,
  que hoy sale 1. Dos de los tres ya salen 2 y en el resto del binario el 2
  significa "el arnes te frena" (la barrera de `verify`, el `check-plan` stale,
  el uso invalido). Un solo camino cambia y el significado queda consistente.
  -> AC-1, AC-2.
- OBS-2 **DECIDIDA: se poda en cada consulta de violaciones.** Ese camino ya lee
  el archivo y ya corre `git status`, asi que la poda no agrega trabajo
  perceptible y el registro queda siempre al dia. Podar solo al escribir dejaria
  entradas muertas por mucho tiempo, porque consultar es muchisimo mas frecuente
  que escribir. -> AC-7, AC-8.
