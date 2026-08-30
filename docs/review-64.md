# Review de la feature #64 - el arnes no promete enforcement que no hace

**Veredicto global: changes_requested** (tercera vuelta).

Cuatro de los seis bloqueantes de la segunda vuelta estan cerrados y verificados
ejecutando, no leyendo: la guarda de spec ya esta espejada en el gate del cierre
(C1), el `feature_list.json` real del checkout principal ya tiene
`require_review` y perdio las tres reglas muertas (C2), la superficie que
afirmaba `one_feature_at_a_time` vigente se corrigio con rastro (C5), y el
`catch` mudo del `.ps1` ahora nombra archivo, excepcion y remedio (C6).

Quedan dos cosas y son de la misma familia que la feature vino a matar: **un
criterio de aceptacion que no puede dar rojo** (C4: el guard del AC-1 nunca
ejecuta el segundo `cargo`, verde falso reproducido) y **un texto que promete
mas enforcement del que hay** (C3: el SDD se corrigio, el spec no). Encima el
spec fue editado despues de la ultima re-firma del usuario y `check-spec` no lo
ve, asi que el criterio en disco no es el que se aprobo.

## Cobertura de los AC del spec

| AC | Cita | Como se verifico | Estado |
| --- | --- | --- | --- |
| AC-1 | rust/src/revision.rs:595 | Sandbox con el binario recompilado de la rama: `close --feature 1 --status done` sin `docs/review-1.md` y con `require_review` activa da exit 2, mensaje `[GATE]` con ruta, regla y comando del remedio, status `in_progress`. El comportamiento esta. Lo que NO esta es el criterio: el comando declarado del AC-1 encadena un segundo `cargo` dentro de un subshell que vuelve a hacer `cd rust` cuando la cwd YA es `rust/`; ese `cd` falla, la negacion lo vuelve verdadero y el segundo `cargo` no corre nunca. Simulado con un `cargo` falso en PATH que emite un target `ok. 2 passed` y otro `test result: FAILED. 0 passed; 5 failed` con exit 101: el comando textual del AC da exit 0 (verde falso); quitando el segundo `cd rust` da exit 1 | no cubierto |
| AC-2 | rust/src/revision.rs:439 | Sandbox: review con filas validas, sello citado dentro de un bloque de codigo y prosa que dice lo contrario, `close --status done` da exit 2 "no lleva el sello del arnes". `lineas_fuera_de_bloque` togglea con las dos formas de fence. Unit `gate_review_ignora_prosa` y E2E en verde en la suite completa | cubierto |
| AC-3 | rust/src/commands/revision.rs:84 | `estampar` decide antes de escribir: review sin filas, `revision --veredicto approved` se niega nombrando cuales AC quedaron sin fila y el archivo queda con cero sellos. Con el spec borrado o sin `AC-n` parseable, la misma guarda se niega. `cargo test veredicto_exige_cobertura_de_ac` verde | cubierto |
| AC-4 | rust/tests/cli_basics.rs:6845 | E2E `veredicto_estampa_y_habilita_el_cierre` verde en la suite completa (396 + 222 passed, 0 failed) y reproducido a mano en sandbox: el binario estampa, deja la linea en el historial y el `close --status done` posterior pasa. Ademas ahora re-lee con el parser del gate antes de afirmar que registro (commands/revision.rs:152) | cubierto |
| AC-5 | rust/src/revision.rs:631 | Sandbox: `changes_requested` y `blocked` estampados, `close --status done` da exit 2 nombrando el veredicto encontrado y ofreciendo `--status blocked`. Unit `gate_review_solo_approved` (revision.rs:819) verde | cubierto |
| AC-6 | rust/src/revision.rs:394 | `require_review` con objeto vacio, con `rules` vacio y con `false` devuelve false; solo `true` activa. Unit `require_review_default_false` verde y comprobado en sandbox: sin la clave, el cierre no se bloquea | cubierto |
| AC-7 | templates/feature_list.json:3 | El bloque `rules` del molde tiene `require_spec_approved` y `require_review` y nada mas; el grep negado del AC da rc=0. Las tres reglas muertas no nacen mas con ningun proyecto nuevo | cubierto |
| AC-8 | setup_harness.sh:633 | `bash tests/setup_smoke.sh` rc=0 (81.5s, corrida del verify de esta vuelta; dos corridas propias en la vuelta anterior, incluida la prueba del rojo con `migrate_rules` vaciada, rc=1). Banco aislado con la funcion real: `setdefault` no pisa valores del usuario, hay backup previo, y los 7 shapes degenerados terminan en WARN o migracion limpia, siempre rc=0. No re-ejecutado en esta vuelta: el `.sh` no cambio salvo lo ya revisado | cubierto |
| AC-9 | tests/parity_check.sh:265 | `bash tests/parity_check.sh` rc=0 ejecutado en esta vuelta. El contador dice "los nueve modos verdes" y el dispatch corre exactamente nueve funciones `modo_*` (contadas una por una). El "ocho" que sobrevive en la narrativa de :191-192 es historia del hallazgo, no el contador. Prueba del rojo de la migracion verificada en la vuelta 2 (funcion borrada, llamada del .ps1 borrada, llamada del .sh borrada: rc=1 en los tres) | cubierto |
| AC-10 | UPDATING.md:143 | `grep -n "2026-08-22" UPDATING.md` rc=0: los 15 ids, las dos fechas del corte y el argumento estan escritos. Y la frase que los encabeza, "La regla aplica de la #64 en adelante", ahora es VERDADERA: leido con `json.load` el `feature_list.json` real del checkout principal, `require_review` esta en `true` y las tres reglas muertas ya no estan. En la vuelta anterior esa misma frase era falsa | cubierto |
| AC-11 | roles/leader.md:97 | El rol dice "pueden convivir varias, cada una en su worktree". El comando compuesto del AC corrido textual desde el worktree: rc=0. En la vuelta anterior se probo el rojo por los dos lados (frase prohibida sembrada, y divergencia rol contra template): rc=1 en ambos. `harness_check.sh` completo no se corre desde el worktree por la divergencia de espejo que el propio spec documenta | cubierto |
| AC-12 | docs/impl-64.md:69 | Prueba del rojo ejecutada con el binario de la rama, no leida: borrado el sello de un review real, `close --status done` da exit 2 y el status queda intacto; restaurado con `revision --veredicto approved`, el cierre pasa. E2E de sello fraguado sin cobertura (cli_basics.rs:6926) verde | cubierto |
| AC-13 | rust/src/revision.rs:506 | `cita_resuelve` exige `metadata().is_file()` y luego cuenta lineas con BufReader: archivo inexistente, linea fuera de rango y `3.14:15` se niegan los tres. Bordes nuevos ejecutados en esta vuelta: un FIFO ya no cuelga (se niega en 0s nombrando el AC) y un archivo multilinea de 200MB citado en la linea 1 cuesta 9.7MB de RSS contra los 300MB de antes. Residuo abajo: el tope cuenta lineas, no bytes | cubierto |

## Estado de los bloqueantes B1-B8 (primera vuelta)

| B | Estado | Como se comprobo |
| --- | --- | --- |
| B1 - sello tipeado a mano pasa el gate | resuelto, con el matiz ya aceptado | Sandbox: sello a mano sin filas por AC, `close --status done` da exit 2 nombrando los AC sin responder. Con filas que citan archivos existentes el sello a mano SI pasa, y eso el spec ya lo admite (spec:41 "filtra el descuido, no la mala fe"). Lo que falta es que el spec deje de desmentirse a si mismo doce lineas mas abajo, ver C3 |
| B2 - sello citado dentro de un fence contaba como veredicto | resuelto en el gate, roto a medias en el limpiador | Ejecutado: sello dentro de fence, `close` exit 2. Pero `revision --veredicto` sobre un review que documenta el formato dentro de un fence `~~~` BORRO la linea citada: el loop de limpieza (commands/revision.rs:119) mira solo el fence de backticks. Observacion viva abajo |
| B3 - spec ausente o sin AC: el cierre pasaba igual | resuelto | Ver C1 |
| B4 - el repo del arnes no tenia la regla que UPDATING promete | resuelto | Ver C2 |
| B4b - comandos declarados que no pueden fallar | NO resuelto | El AC-8 y el AC-11 se arreglaron y se probaron por los dos lados. El AC-1 sigue siendo el caso: su guard es vacuo. Ver C4 |
| B5 - parity_check no ataba la migracion (AC-9 vacuo) | resuelto | Prueba del rojo por triplicado en la vuelta 2; `bash tests/parity_check.sh` rc=0 en esta. Sigue evadible con un stub del mismo nombre, que es lo que un chequeo declarativo promete y nada mas |
| B6 - superficie que afirmaba `one_feature_at_a_time` vigente | resuelto | Ver C5 |
| B7 - `feature_list.json` de solo lectura abortaba la instalacion | resuelto | Banco aislado en la vuelta 2 con la funcion real bajo `set -Eeuo pipefail`: target 444, WARN, rc=0, archivo del usuario byte-identico (setup_harness.sh:624). No re-ejecutado en esta vuelta: el `.sh` no cambio |
| B8 - shapes degenerados en silencio, BOM divergente | resuelto | Mismo banco, 7 shapes, todos WARN o migracion limpia; BOM migra con `utf-8-sig` como `ConvertFrom-Json`. No re-ejecutado en esta vuelta |

## Estado de los bloqueantes C1-C6 (segunda vuelta)

| C | Estado | Como se comprobo |
| --- | --- | --- |
| C1 - el gate del cierre sin la guarda de spec de `estampar` | RESUELTO | Binario recompilado de la rama, las dos variantes del ataque original en sandbox: (a) spec borrado, sello a mano, review "nada" da exit 2 "No se pudo leer el spec de la feature #1"; (b) spec aprobado y despues editado para envolver los `- AC-n:` en un fence da exit 2 "no declara ningun AC-n". Status `in_progress` en las dos. La guarda esta en rust/src/revision.rs:595, identica al bail de commands/revision.rs:84 |
| C2 - el repo del arnes sin `require_review`, con UPDATING prometiendolo | RESUELTO | Solo lectura, como se pidio: `json.load` sobre el `feature_list.json` del checkout principal. `require_review: true` esta, y `one_feature_at_a_time`, `require_tests_to_close` y `require_impact_check` ya no. La promesa de UPDATING.md:143 dejo de ser falsa |
| C3 - el texto promete lo que el gate no comprueba | RESUELTO A MEDIAS | El SDD si: docs/prd/SDD-master.md:219 ahora dice que el gate "sube el costo" y aclara que lo que NO comprueba es la pertinencia de la cita, con rastro de `prd apply` por USUARIO. El spec NO: spec:45 conserva "fabrica en cinco segundos", y ese review de cinco segundos lo volvi a ejecutar con el binario actual (sello a mano mas una fila por AC citando un archivo real cualquiera, `close --status done` exit 0). El item pedia texto honesto en los DOS |
| C4 - el comando del AC-1 puede dar verde falso | NO RESUELTO, y declarado hecho | Simulacion ejecutada con `cargo` falso en PATH: el comando textual del AC-1 da exit 0 con un target en FAILED. El guard nuevo introdujo un doble `cd rust` que lo vuelve inalcanzable; aislado, la parte negada devuelve exit 0 sin ejecutar cargo. Los 493ms del AC-1 en docs/verify-64.md:9 confirman que el segundo cargo nunca corrio. docs/impl-64.md:157 lo declara arreglado: es la misma clase de error que C5 |
| C5 - declarar limpio lo no verificado, y la superficie de B6 | RESUELTO | docs/impl-64.md:126 ahora dice "B6: lo declare limpio y no lo estaba", con el porque. docs/prd/aprendizaje/PRD-aprendizaje.md:188 ahora dice que la regla dejo de bloquear en la #47 y que la #64 la borro del molde. La ruta protegida se toco con el si explicito del usuario y quedo el rastro en el historial |
| C6 - el `catch` mudo del `.ps1` | RESUELTO por lectura | No hay `pwsh` en esta maquina, asi que es lectura: setup_harness.ps1:597 avisa con WARN nombrando el archivo, el tipo de excepcion y el remedio, en paridad con el `.sh`. La paridad declarativa si se ejecuto (`bash tests/parity_check.sh` rc=0) |

## Cambios pedidos

**D1 (C4). Sacar el segundo `cd rust` del comando del AC-1.** Tal como esta, el
subshell arranca con la cwd ya en `rust/`, su propio `cd rust` falla porque no
existe `rust/rust`, el subshell devuelve distinto de cero y la negacion lo
convierte en verdadero sin haber ejecutado cargo. Verificado en los dos sentidos
con un `cargo` falso: con el doble `cd`, exit 0 sobre un target en FAILED; sin
el, exit 1. Corregir tambien docs/impl-64.md:157, que hoy afirma que este
arreglo esta hecho.

**D2 (C3 + firma). Poner el texto honesto en spec:45 y volver a correr el ritual
de aprobacion.** El spec en disco no es el que el usuario firmo: el sello dice
22:16:38Z, el ultimo `approve-spec` del historial es de las 23:50:50Z y el mtime
del archivo es 00:18:46Z, que es cuando entro el guard del AC-1. `check-spec` da
rc=0 porque la firma path/mtime/hash la refresca el flujo del propio agente, que
detecta "otro LLM" y no "edicion sin usuario". Arreglado el comando del AC-1 y
corregida la frase de spec:45 para que diga lo mismo que ya dice el SDD, hay que
re-firmar: si no, se cierra contra un criterio que nadie aprobo.

## Observaciones vivas (lo que el gate NO cubre)

1. **`estampar` muta la prosa del reviewer en fences `~~~`.** El arreglo de los
   fences aterrizo en el parser del gate (rust/src/revision.rs:439) pero no en
   el loop de limpieza (rust/src/commands/revision.rs:119), que sigue mirando
   solo los backticks. Reproducido: un review que documenta el formato del sello
   dentro de `~~~`, al registrar `changes_requested`, pierde la linea citada y
   queda un bloque vacio. Costo: se destruye contenido del reviewer, que es
   justo la mitad de B2 que el arreglo prometia conservar. Fix de una linea.
2. **Fences mezclados desincronizan el toggle.** Un bloque de backticks cuyo
   contenido incluye una linea `~~~` (por ejemplo un review ajeno citado entero)
   cierra el bloque donde CommonMark no lo cierra, y un sello que esta DENTRO
   del bloque conto como veredicto: `close --status done` exit 0, status `done`.
   Es rotura nueva, introducida al tratar `~~~` como toggle simetrico. Costo:
   igual de menor que el borde original (anti-descuido, no anti-mala-fe). Fix:
   recordar cual fence abrio y cerrar solo con el mismo.
3. **El tope de `cita_resuelve` cuenta lineas, no bytes.** Un archivo de 200MB
   en UNA sola linea citado en la linea 1 costo 211MB de RSS medidos, contra
   9.7MB del mismo tamano multilinea: `BufRead::lines` materializa la linea
   entera y el `.take(n)` (rust/src/revision.rs:516) acota cuantas lineas, no
   cuanto pesa cada una. El comentario del codigo promete "sin cargar el archivo
   entero". Costo: una fila del review puede hacer que el gate reserve memoria
   arbitraria. Fix: leer por bytes contando saltos de linea con tope de bytes.
4. **La cita no tiene que ser PERTINENTE.** El gate comprueba que resuelva, no
   que hable del AC. Ya esta dicho asi en el SDD y hay que dejarlo dicho asi en
   el spec (D2). Costo: un agente decidido cierra igual; lo que se filtra es el
   descuido.
5. **`parity_check` es declarativo.** Comprueba que el `.ps1` declare la misma
   migracion, no que la haga. Un stub con el mismo nombre lo pasa. Es lo que el
   AC-9 promete y no mas, pero conviene que quien lea sepa que no hay ejecucion
   de PowerShell detras.

## Lo que NO se probo

- **PowerShell: nada se ejecuto.** No hay `pwsh` en esta maquina. C6,
  `Migrate-HarnessRules` y su guarda son analisis por lectura. La unica
  evidencia ejecutada del lado Windows es la paridad declarativa.
- **No re-ejecutado en esta vuelta** (verde verificado en vueltas anteriores y
  sin cambios en el codigo que cubren): `tests/setup_smoke.sh` completo, los
  bancos de shapes de B7 y B8, la prueba del rojo de la migracion de reglas, y
  el recalculo de los 15 ids y las dos fechas del AC-10 (solo se comprobo que el
  texto no cambio y que su promesa ahora es cierta).
- **Fuera de alcance:** `harness_check.sh` completo desde el worktree (reporta
  las divergencias de espejo falsas que el propio spec documenta); el symlink a
  `/dev/zero` (el `is_file()` nuevo deberia rechazar el char device, razonado
  por lectura); el symlink que sale del repo, aceptado como observacion en la
  vuelta 2; grafo de impacto y `graphify`, que no aplican a un cambio de arnes
  sin microservicios; la leccion del cierre, que no se puede juzgar antes del
  cierre.
- **Verificacion independiente ejecutada:** suite completa 396 + 222 passed 0
  failed, `parity_check` rc=0, `check-spec` rc=0 (con la salvedad de D2). Todos
  los sabotajes vivieron en sandboxes del scratchpad: el worktree y el checkout
  principal quedaron sin tocar.
