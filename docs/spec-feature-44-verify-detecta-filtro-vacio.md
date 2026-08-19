# Spec - Feature #44: verify_detecta_filtro_vacio

Estado: approved
Aprobado: 2026-08-19T00:12:40Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-44-verify-detecta-filtro-vacio.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan cierra la feature #28. `verify --feature 28` le dice **27 verde(s),
0 en rojo**, y el gate `require_verify_green` lo deja cerrar. Uno de esos
verdes, el AC-12, dice que `lecciones consolidar` sin `--aplicar` no toca nada
— el invariante que el README promete, que el help de clap promete y que el
comentario del modulo promete. Su `Comando:` es
`cargo test consolidar_without_aplicar_should_not_touch_anything`, y esa funcion
**no existe**. `cargo test` con un filtro que no matchea nada imprime
`running 0 tests`, dice `test result: ok` y sale **0**. El AC quedo verde sin
que corriera una sola linea de codigo, y el reporte lo registro como evidencia.

Es la misma trampa que la feature #23 encontro y que la leccion
`probar-contra-datos-reales` describe. La #23 la arreglo **a mano**, renombrando
los tests: no dejo nada que la detecte la proxima vez. Cinco features despues
volvio a pasar, y esta vez nadie la vio hasta que un pase de refutacion la
busco a proposito.

DESPUES: ese AC sale **`vacio`** en el reporte, con la salida que lo delata
pegada abajo, cuenta como bloqueante y `close --status done` no pasa. El
instrumento deja de decir "verde" cuando no midio nada.

## Hoy -> Como va a funcionar

```
HOY                                    DESPUES
verify corre el comando                verify corre el comando
  |__ exit 0 -> Verde, salida DESCARTADA  |__ exit 0 -> mira la salida
  |__ exit N -> Rojo, salida guardada           |__ ¿libtest y cero casos?
                                                |     -> Vacio (bloquea)
                                                |__ si no -> Verde
close lee docs/verify-<id>.md          close lee docs/verify-<id>.md
  |__ compara la celda contra las         |__ traduce la celda a Estado
      cadenas "rojo" y "timeout"              y pregunta Estado::bloquea()
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero que un AC cuyo comando no ejecuto ningun caso salga
  `vacio` y no `verde`, para que el reporte de verificacion no me mienta.
- P1: Como Alan, quiero que ese AC bloquee el cierre igual que un rojo, para que
  la regla `require_verify_green` signifique lo que dice.
- P2: Como el proximo que agregue un estado a `Estado`, quiero que el lector del
  reporte lo tome del enum y no de una cadena suelta, para que el compilador me
  avise en vez de un usuario.
- P2: Como Alan, quiero que el falso verde que esto destape quede pagado, no
  solo detectado: el invariante de la #28 tiene que tener un test de verdad.

## Criterios de aceptacion (Given/When/Then)

### El detector, puro

- AC-1: Given una salida que no es de libtest (un `grep`, un `bash`, texto
  vacio), When se le pregunta cuantos casos corrio, Then contesta "no opino"
  (`None`) y el estado no cambia.
  Comando: `cd rust && cargo test casos_corridos_should_not_opine_about_non_libtest_output`

- AC-2: Given la salida REAL de
  `cargo test consolidar_without_aplicar_should_not_touch_anything` en este repo
  (dos binarios, `0 passed` y `322`/`161 filtered out`), When se la mide,
  Then dice `Some(0)`.
  Comando: `cd rust && cargo test casos_corridos_should_count_zero_on_the_real_empty_filter`

- AC-3: Given una salida con varios binarios donde UNO corrio casos y los otros
  no (el caso normal de `cargo test <nombre>`), When se la mide, Then suma y
  devuelve el total mayor que cero.
  Comando: `cd rust && cargo test casos_corridos_should_sum_across_test_binaries`

- AC-4: Given una salida de libtest donde todo quedo `ignored`, When se la mide,
  Then cuenta cero: un test ignorado tampoco es evidencia.
  Comando: `cd rust && cargo test casos_corridos_should_count_ignored_tests_as_no_evidence`

### El estado nuevo

- AC-5: Given el estado `Vacio`, When se pregunta si bloquea, Then si, con
  etiqueta y simbolo propios: no se confunde con `Rojo` (el comando anduvo) ni
  con `Verde`.
  Comando: `cd rust && cargo test vacio_should_block_without_pretending_to_be_red`

- AC-6: Given un AC cuyo comando sale 0 pero no corrio ningun caso, When
  `verify` lo ejecuta, Then el resultado es `Vacio` y la salida queda guardada
  como evidencia (hoy se descarta en el camino feliz).
  Comando: `cd rust && cargo test ejecutar_should_mark_an_empty_test_run_as_vacio`

- AC-7: Given un AC cuyo comando sale 0 y SI corrio casos, When `verify` lo
  ejecuta, Then sigue siendo `Verde`.
  Comando: `cd rust && cargo test ejecutar_should_keep_a_real_test_run_green`

- AC-8: Given un AC cuyo comando sale 0 y no es un test (un `grep -q`, un
  `bash tests/*.sh`), When `verify` lo ejecuta, Then sigue siendo `Verde`: el
  detector no opina sobre lo que no entiende.
  Comando: `cd rust && cargo test ejecutar_should_not_mark_a_non_test_command_as_vacio`

### El reporte y el cierre

- AC-9: Given un reporte con un AC `vacio`, When se renderiza, Then el resumen
  lo cuenta aparte ("N sin casos") en vez de esconderlo dentro de "en rojo", y
  su salida aparece en la seccion de los que fallaron.
  Comando: `cd rust && cargo test render_should_count_empty_runs_apart_from_red`

- AC-10: Given un reporte con un AC `vacio`, When `close` lo lee, Then lo
  devuelve como bloqueante y el cierre sale 2.
  Comando: `cd rust && cargo test close_should_block_on_an_empty_verification`

- AC-11: Given cualquier variante de `Estado`, When se escribe su etiqueta en el
  reporte y se la vuelve a leer, Then se recupera la misma variante: el lector
  del reporte deja de comparar contra cadenas sueltas y sale del enum, asi que
  agregar un estado sexto no puede pasar de largo por el cierre.
  Comando: `cd rust && cargo test etiqueta_should_round_trip_for_every_estado`

- AC-12: Given el flujo completo en un sandbox (spec con un AC que declara un
  `cargo test` de un nombre inexistente), When se corre `verify` y despues
  `close --status done`, Then el reporte dice `vacio` y el cierre sale 2
  nombrando ese AC.
  Comando: `bash tests/verify_vacio_check.sh`

### La deuda que esto destapa

- AC-13: Given `lecciones consolidar` SIN `--aplicar` y un backend que SI
  devuelve candidatos, When corre, Then el arbol queda byte a byte igual y no
  hay backup: el invariante que la #28 declaro verde sin test ahora tiene uno,
  y con backend falso, sin gastar cuota.
  Comando: `cd rust && cargo test consolidar_without_aplicar_should_not_touch_anything`

- AC-14: Given el reporte de la #28 regenerado con este binario, When se lo
  mira, Then sus 27 AC estan verdes de verdad: cero `vacio`.
  Comando: `test "$(grep -c "| vacio |" docs/verify-28.md)" -eq 0`

### Los de siempre

- AC-15: Given el plan de esta feature, When se lo lee, Then declara
  `Peldano elegido:` con su razon.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-44-verify-detecta-filtro-vacio.md`

- AC-16: Given la documentacion, When se busca el estado nuevo, Then README,
  UPDATING y el espejo de templates lo explican.
  Comando: `grep -q "vacio" README.md UPDATING.md templates/UPDATING.md`

- AC-17: Given el arbol, When corre clippy con `-D warnings`, Then limpio.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: la salida (stdout+stderr) de cada comando de AC que sale 0.
- interruptor: ninguno. Un AC que no midio nada no es una preferencia: es un
  reporte equivocado. Si hiciera falta apagarlo, se apaga no declarando comando
  (el AC queda `manual`, que ya existe y es honesto).
- candado: el detector solo opina cuando reconoce la forma de libtest
  (`test result:`). Ante cualquier otra salida devuelve `None` y no toca el
  estado.

## Pseudo-codigo (el acuerdo)

```
CUANDO verify termina de correr el comando de un AC

  ¿salio distinto de 0 o se colgo?   -> Rojo / Timeout, como hoy
  ¿la salida tiene lineas "test result:"?  -> si no, Verde, como hoy

  ENTONCES sumar los `N passed` de todas esas lineas,
           y si el total es cero -> Vacio,
           guardando la salida como evidencia de que no corrio nada.
```
Promesas: no opina sobre comandos que no son de libtest · no distingue por el
texto del comando sino por la forma de la salida (un `cargo test` disfrazado
dentro de un script tambien queda cubierto) · no vuelve verde nada que hoy sea
rojo.

## No funcionales

- SLOs: sin costo medible. La deteccion es un `lines().filter()` sobre una
  salida ya recortada a las ultimas lineas.
- Seguridad: la salida de un comando exitoso pasa a guardarse en
  `docs/verify-<id>.md`. Ya se guardaba la de los que fallan, que es la que
  suele traer rutas y entorno; el recorte de lineas es el mismo.
- Observabilidad: el estado nuevo se ve en la tabla del reporte y en el resumen.

## Fuera de alcance

- Otros corredores de test (nextest, pytest, jest). El detector reconoce el
  formato de libtest y calla ante el resto. Si aparece otro, se agrega ahi.
- Detectar ACs cuyo comando es trivialmente verde por otras razones (`true`,
  `grep` sobre un archivo que siempre existe). Es un problema distinto y mas
  dificil; este arregla el caso medido y recurrente.
- Re-verificar las 20 features cerradas. La auditoria de los 99 nombres de test
  declarados ya se hizo a mano y devolvio un solo caso real: el de la #28, que
  esta feature paga. `verificacion` de la #23 es un filtro de modulo y si
  matchea.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por Alan, 2026-08-18): primero #44 y #45, antes de la paraguas
  #38-#43, porque 13 de los 20 AC que la paraguas propone son
  `cargo test <nombre>` y sin esto pueden salir verdes sin correr nada.
