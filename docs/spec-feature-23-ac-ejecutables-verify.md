# Spec - Feature #23: ac_ejecutables_verify

Estado: approved
Aprobado: 2026-08-17T05:12:11Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #23 en el chat (AskUserQuestion: 'Si, lo apruebo'), con el spec mostrado en el chat y abierto en su editor. 20 AC, y es el PRIMER spec del repo que declara sus propios comandos de verificacion. Decisiones OBS-1..OBS-5: el comando va pegado al AC (unir criterio y prueba es el punto de la feature), el cierre LEE el reporte y exige frescura en vez de ejecutar, verify EXIGE spec aprobado (es la barrera contra ejecutar un comando escrito por un agente que nadie leyo), un comando que no corre es rojo, y el reporte se versiona como evidencia.
Plan: docs/plan-feature-23-ac-ejecutables-verify.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: este repo tiene **310 criterios de aceptacion** repartidos en 21 specs, y
los 310 son **prosa**. "Then se siembra `docs/lecciones/` con su guia y ninguna
leccion" es un criterio perfectamente claro y perfectamente no verificable por
nadie mas que un humano leyendo con atencion.

Al cerrar, el reviewer escribe "AC-1: cubierto" y cita un test. Ese enlace entre
el criterio y la prueba **existe solo en su cabeza y en su prosa**. Si el test se
borra, el AC sigue diciendo "cubierto" para siempre. Si el reviewer se equivoca,
nadie se entera. La regla `require_tests_to_close` esta activa desde la primera
feature y no verifica nada: es una declaracion de intenciones.

DESPUES: un AC puede declarar **como se prueba**, en la linea de abajo:

```
- AC-1: Given un proyecto sin docs/lecciones/, When corre el instalador,
  Then se siembra la guia y ninguna leccion.
  Comando: `bash tests/setup_smoke.sh`
```

`sh harness_cli verify` los corre, registra el exit code de cada uno y escribe
`docs/verify-<id>.md`. Con la regla `require_verify_green`, cerrar exige que ese
reporte exista, este **fresco** y este verde. El enlace entre criterio y prueba
deja de vivir en la prosa.

Los 310 AC existentes siguen siendo validos: un AC sin `Comando:` queda marcado
como **verificacion manual** y sigue siendo trabajo del reviewer, como hoy.

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES

spec: 310 AC en prosa                   spec: AC-n + `Comando: <shell>` (opcional)
  `__ nadie los ejecuta                   |
                                          v
reviewer: "AC-1: cubierto"              sh harness_cli verify
  `__ el enlace vive en su cabeza         |__ MUESTRA cada comando antes de correrlo
                                          |__ lo ejecuta con timeout
require_tests_to_close: true              |__ registra exit code por AC-n
  `__ no verifica nada                    `__ docs/verify-<id>.md

                                        close --status done
                                          `__ con require_verify_green:
                                              exige reporte fresco y verde
                                              (LEE el reporte; no ejecuta nada)
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero que un AC diga como se prueba, para que "cubierto" deje
  de ser una afirmacion y pase a ser un hecho.
- P1: Como reviewer, quiero un reporte que diga que AC paso, cual fallo y con que
  salida, sin correr nada a mano.
- P1: Como cualquiera, quiero que **ningun comando se ejecute por sorpresa**: ni
  desde un hook, ni al cerrar, ni sin verlo antes.
- P1: Como duenno de los 21 specs ya escritos, quiero que **nada se rompa**: sin
  `Comando:` todo sigue igual.
- P2: Como script, quiero `--json` con el resultado por AC.

## Criterios de aceptacion (Given/When/Then)

### Declarar la verificacion

- AC-1: Given un AC-n del spec seguido de una linea `Comando: <shell>` (con el
  comando entre backticks o no), When corre `verify`, Then ese comando queda
  asociado a ese AC-n.
  Comando: `cd rust && cargo test verificacion::tests::parse`
- AC-2: Given un spec **sin ninguna** linea `Comando:`, When corre `verify`, Then
  se informa que no hay verificaciones declaradas, **no se ejecuta nada** y el
  exit code es **0**: los 310 AC existentes siguen siendo validos.
  Comando: `cd rust && cargo test verify_should_do_nothing_without_declared_commands`
- AC-3: Given un AC sin `Comando:`, When se escribe el reporte, Then aparece
  marcado como **verificacion manual**, a cargo del reviewer, y **no** cuenta como
  fallo.
  Comando: `cd rust && cargo test verificacion::tests::manual`

### Ejecutar, con el usuario mirando

- AC-4: Given un spec con comandos declarados, When corre `verify`, Then **antes**
  de ejecutar cada comando se imprime cual es y a que AC pertenece. Nada se corre
  a ciegas.
  Comando: `cd rust && cargo test verify_should_print_each_command_before_running_it`
- AC-5: Given un spec en `Estado: draft`, When corre `verify`, Then **se niega**
  con exit 2: solo se ejecutan comandos de un spec que el USUARIO ya aprobo y por
  lo tanto ya leyo. Es la barrera que impide que un comando escrito por un agente
  se ejecute sin que nadie lo haya visto.
  Comando: `cd rust && cargo test verify_should_refuse_to_run_commands_from_a_draft_spec`
- AC-6: Given un comando que tarda mas que el limite (`rules.verify_timeout_segundos`,
  default 300), When corre `verify`, Then se corta, se registra como fallo por
  timeout y **se sigue con los demas AC**: un comando colgado no cuelga la
  verificacion entera.
  Comando: `cd rust && cargo test verify_should_time_out_a_hung_command`
- AC-7: Given `verify`, Then **nunca** se ejecuta desde un hook ni desde otro
  comando del arnes: solo cuando alguien lo invoca a mano.
  Comando: `grep -rn "verify" bin/harness-hook setup_harness.sh | grep -v "^setup_harness.sh:.*#" | grep -c "harness_cli.*verify" || true`

### El reporte

- AC-8: Given una corrida de `verify`, Then queda `docs/verify-<id>.md` con, por
  cada AC: su numero, su comando, su exit code, cuanto tardo y las ultimas lineas
  de su salida cuando fallo.
  Comando: `cd rust && cargo test verify_should_write_a_report_per_ac`
- AC-9: Given un AC que fallo, Then el reporte muestra su salida (acotada) para
  poder diagnosticar sin re-correr.
  Comando: `cd rust && cargo test verify_should_include_output_of_failures`
- AC-10: Given `--json`, Then se expone por AC: `ac`, `comando`, `exit`, `estado`
  (`verde` / `rojo` / `timeout` / `manual`) y `duracion_ms`.
  Comando: `cd rust && cargo test verify_json_should_expose_the_result_per_ac`
- AC-11: Given `--solo <AC-n>`, When corre `verify`, Then se ejecuta unicamente
  ese AC: en un spec de 20 criterios, iterar sobre uno no cuesta correr los 20.
  Comando: `cd rust && cargo test verify_should_run_a_single_ac_on_demand`

### El gate del cierre

- AC-12: Given `feature_list.json` **sin** `require_verify_green` (ausente o
  `false`), When se cierra como done, Then el cierre se comporta exactamente como
  hoy: compatibilidad total con las 22 features ya cerradas.
  Comando: `cd rust && cargo test close_should_stay_identical_without_the_verify_rule`
- AC-13: Given `require_verify_green: true` y un spec con comandos declarados,
  When se cierra como done **sin** reporte, Then exit 2 con mensaje accionable
  (`corre sh harness_cli verify`) y la feature **no** cierra.
  Comando: `cd rust && cargo test close_should_demand_a_verify_report`
- AC-14: Given un reporte con algun AC en rojo, When se cierra como done, Then
  exit 2 nombrando **cuales** AC fallaron.
  Comando: `cd rust && cargo test close_should_block_on_a_red_report`
- AC-15: Given un reporte **mas viejo que el spec** (el spec se edito despues de
  verificar), When se cierra como done, Then exit 2 por reporte stale: un verde de
  antes de cambiar los criterios no prueba nada.
  Comando: `cd rust && cargo test close_should_block_on_a_stale_report`
- AC-16: Given el cierre, Then **nunca ejecuta los comandos**: solo LEE el
  reporte. Cerrar no puede disparar shell arbitrario ni tardar lo que tarde una
  suite.
  Comando: `cd rust && cargo test close_should_never_execute_verify_commands`

### Integracion, docs y verificacion

- AC-17: Given la plantilla del spec que genera `start`, Then documenta la linea
  `Comando:` como opcional, con un ejemplo.
  Comando: `cd rust && cargo test start_should_document_the_command_line_in_the_spec_template`
- AC-18: Given `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md`
  (+ plantilla) y las superficies, Then documentan `verify`, la regla y la
  barrera del spec aprobado.
  Comando: `grep -q "require_verify_green" README.md UPDATING.md docs/architecture.md`
- AC-19: Given los tres roles, Then el lider declara `Comando:` donde se pueda al
  escribir los AC, el implementer corre `verify` antes de pedir revision, y el
  reviewer exige el reporte verde y fresco.
  Comando: `grep -q "verify" roles/leader.md roles/implementer.md roles/reviewer.md`
- AC-20: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test` y `cargo clippy --all-targets -- -D warnings` estan verdes, y
  `tests/setup_smoke.sh` sigue verde.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- **disparador**: `sh harness_cli verify`, **siempre** a mano.
- **interruptor**: `require_verify_green` en `rules` (ausente/`false` por default);
  y `verify_timeout_segundos` (default 300).
- **candado**: la frescura del reporte contra el mtime del spec — un verde viejo
  no vale — y la exigencia de spec **aprobado** para ejecutar.
- **entidad nueva**: `docs/verify-<id>.md` (el reporte, versionable como el resto
  de la evidencia).
- **lo que NO se toca**: el spec (verify solo lo lee), el plan, el hub, y ningun
  comando se ejecuta fuera de `verify`.

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien corre verify

  ¿el spec esta aprobado?      -> si no, NOS NEGAMOS: no ejecutamos comandos
                                  de un texto que el usuario no leyo
  ¿hay comandos declarados?    -> si no, lo decimos y salimos con 0

  para cada AC con comando:
     MOSTRAMOS el comando
     lo ejecutamos con timeout
     registramos exit code, duracion y salida si fallo

  ENTONCES escribimos docs/verify-<id>.md,
           con la restriccion de que un AC sin comando NO es un fallo:
           es verificacion manual del reviewer.


CUANDO se cierra una feature como done

  ¿require_verify_green activa? -> si no, cerramos como siempre
  ¿el spec declara comandos?    -> si no, cerramos como siempre

  ¿existe el reporte?           -> si no, FALLAMOS pidiendo correr verify
  ¿es mas nuevo que el spec?    -> si no, FALLAMOS: es un verde viejo
  ¿esta todo en verde?          -> si no, FALLAMOS nombrando los AC rojos

  ENTONCES cerramos, sin ejecutar ni un comando.
```

**Promesas:** nada se ejecuta sin que lo veas · nada se ejecuta desde un spec sin
aprobar · nada se ejecuta al cerrar · un AC sin comando sigue siendo valido · sin
la regla, todo sigue igual.

## No funcionales

- **SLOs**: `verify` tarda lo que tarden los comandos declarados; el timeout
  acota cada uno. El cierre no ejecuta nada, asi que sigue siendo instantaneo.
- **Seguridad**: es la parte delicada y la razon del AC-5. El arnes nunca ejecuto
  comandos arbitrarios; ahora puede. Tres barreras: **spec aprobado** (el usuario
  lo leyo), **invocacion manual** (nunca un hook), y **el comando impreso antes de
  correr**. El comando se ejecuta tal cual, sin interpolar nada del entorno.
- **Observabilidad**: exit 0 si todo verde o no hay nada que verificar; exit 2 si
  algun AC fallo, si el spec esta en draft, o por uso invalido.

## Fuera de alcance

- Inferir el comando de un AC escrito en prosa: lo declara quien escribe el spec.
- Correr `verify` en CI o en un hook.
- Reemplazar `harness_check.sh`, que verifica el estado del PROCESO; `verify`
  verifica los CRITERIOS de una feature.
- Backfillear los 310 AC existentes con comandos: los specs cerrados quedan como
  estan.

## Observaciones (decisiones pendientes)

Todas decididas por Alan el 2026-08-17, en el mismo acto de aprobacion del spec.
No queda ninguna observacion abierta: el implementer puede avanzar sin preguntar.

- OBS-1: ¿Como se declara el comando? — **DECIDIDO: una linea `Comando: <shell>`
  inmediatamente debajo del AC**, dentro de su mismo item. Poner los comandos en
  una tabla aparte separaria el criterio de su prueba, que es justo lo que esta
  feature viene a unir. Vinculante para AC-1.
- OBS-2: ¿El cierre ejecuta o lee? — **DECIDIDO: LEE el reporte** y exige
  frescura. Ejecutar al cerrar significaria disparar shell arbitrario en el
  momento menos esperado y hacer que cerrar tarde lo que tarde una suite. El
  costo aceptado es tener que acordarse de correr `verify`, y por eso el mensaje
  de error lo dice. Vinculante para AC-13, AC-15 y AC-16.
- OBS-3: ¿`verify` exige spec aprobado? — **DECIDIDO: si.** En draft se niega con
  exit 2. Es **la** barrera de esta feature: durante el draft el spec lo escribe
  un agente, y ahi es exactamente cuando un comando podria ejecutarse sin que
  nadie lo haya mirado. Aprobar significa que el usuario lo leyo, asi que reusa un
  ritual que ya existe en vez de inventar una confirmacion nueva. Vinculante para
  AC-5.
- OBS-4: ¿Que pasa si el comando no existe o no es ejecutable? — **DECIDIDO: se
  registra como rojo** con su error. Un comando que no corre es un criterio no
  verificado.
- OBS-5: ¿Se versiona el reporte? — **DECIDIDO: si**, como `impl-<id>.md` y
  `review-<id>.md`: es evidencia del cierre.
