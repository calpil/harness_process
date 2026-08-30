# Review de la feature #64 - el arnes no promete enforcement que no hace
Revisado: approved · 2026-08-30T18:04:24Z · estampado por `harness revision --veredicto`

**Veredicto global: approved** (cuarta vuelta). Approved aca significa una sola
cosa y conviene decirla sin adorno: **no se pudo romper con los casos probados**.
No significa correcto. Al final de este archivo estan, con nombre y costo, los
dos agujeros que quedan vivos y las seis cosas que no se probaron.

## Cobertura de los AC del spec

| AC | Cita | Como se verifico | Estado |
| --- | --- | --- | --- |
| AC-1 | rust/src/revision.rs:602 | Sandbox con el binario recompilado de la rama: `close --feature 1 --status done` sin `docs/review-1.md` y con `require_review` activa da exit 2, mensaje `[GATE]` con ruta, regla y comando del remedio, y el status queda `in_progress`. El criterio, que en la tercera vuelta era vacuo, se re-probo con un `cargo` falso en PATH y el comando TEXTUAL del spec (linea 107): mixto (un target `ok. 2 passed`, otro `test result: FAILED`, exit 101) da rc=101; mixto forzando exit 0 igual da rc=1 porque dispara el `grep -q FAILED` por si solo; todo verde da rc=0; `0 passed` da rc=1. Con el cargo real, rc=0 en 5142ms | cubierto |
| AC-2 | rust/src/revision.rs:434 | Sandbox: review con filas validas, sello citado dentro de un bloque de codigo y prosa que dice lo contrario, `close --status done` da exit 2 "no lleva el sello del arnes". `lineas_fuera_de_bloque` ahora recuerda CUAL fence abrio y cierra solo con el mismo. Unit `gate_review_ignora_prosa` y el E2E nuevo de fences mezclados verdes en la suite completa | cubierto |
| AC-3 | rust/src/commands/revision.rs:84 | `estampar` decide antes de escribir: review sin filas, `revision --veredicto approved` se niega nombrando cuales AC quedaron sin fila y el archivo queda con cero sellos. Con el spec borrado o sin `AC-n` parseable, la misma guarda se niega. `cargo test veredicto_exige_cobertura_de_ac` verde | cubierto |
| AC-4 | rust/tests/cli_basics.rs:6845 | E2E `veredicto_estampa_y_habilita_el_cierre` verde en la suite completa re-ejecutada en esta vuelta (396 + 224 passed, 0 failed) y reproducido a mano en sandbox: el binario estampa, deja la linea en el historial y el `close --status done` posterior pasa. Re-lee con el parser del gate antes de afirmar que registro | cubierto |
| AC-5 | rust/src/revision.rs:672 | Sandbox re-corrido en esta vuelta: `changes_requested` estampado, `close --status done` da exit 2 nombrando el veredicto encontrado y ofreciendo `--status blocked`; idem `blocked`. Unit `gate_review_solo_approved` verde | cubierto |
| AC-6 | rust/src/revision.rs:394 | `require_review` con objeto vacio, con `rules` vacio y con `false` devuelve false; solo `true` activa. Unit `require_review_default_false` verde y comprobado en sandbox: sin la clave, el cierre no se bloquea | cubierto |
| AC-7 | templates/feature_list.json:3 | El bloque `rules` del molde tiene `require_spec_approved` y `require_review` y nada mas; el grep negado del AC corrido textual da rc=0 en esta vuelta. Las tres reglas muertas no nacen mas con ningun proyecto nuevo | cubierto |
| AC-8 | setup_harness.sh:633 | `bash tests/setup_smoke.sh` rc=0 en 71s (corrida del verify del 2026-08-29). NO re-ejecutado en esta vuelta; en su lugar se comprobo por mtime que ningun archivo del commit cambio despues de esa corrida, asi que el verde cubre el codigo actual. Banco aislado de vueltas anteriores con la funcion real: `setdefault` no pisa valores del usuario, hay backup previo, target 444 termina en WARN con rc=0, y los 7 shapes degenerados terminan en WARN o migracion limpia | cubierto |
| AC-9 | tests/parity_check.sh:265 | `bash tests/parity_check.sh` rc=0 ejecutado en esta vuelta. El contador dice "los nueve modos verdes" y el dispatch corre exactamente nueve funciones `modo_*`. Prueba del rojo de la migracion hecha en la vuelta 2 por triplicado (funcion borrada, llamada del .ps1 borrada, llamada del .sh borrada: rc=1 en los tres) | cubierto |
| AC-10 | UPDATING.md:143 | `grep -n "2026-08-22" UPDATING.md` rc=0: los 15 ids, las dos fechas del corte y el argumento estan escritos. La frase que los encabeza, "La regla aplica de la #64 en adelante", es VERDADERA desde la vuelta 3: leido con `json.load`, el `feature_list.json` real del checkout principal tiene `require_review: true` y perdio las tres reglas muertas | cubierto |
| AC-11 | roles/leader.md:97 | El rol dice "pueden convivir varias, cada una en su worktree". El comando compuesto del AC corrido textual desde el worktree en esta vuelta: rc=0. En la vuelta 2 se probo el rojo por los dos lados (frase prohibida sembrada, y divergencia rol contra template): rc=1 en ambos | cubierto |
| AC-12 | docs/impl-64.md:69 | Prueba del rojo ejecutada con el binario de la rama, no leida: borrado el sello de un review real, `close --status done` da exit 2 y el status queda intacto; restaurado con `revision --veredicto approved`, el cierre pasa. E2E de sello fraguado sin cobertura por AC verde en la suite | cubierto |
| AC-13 | rust/src/revision.rs:534 | El conteo de lineas paso a bytes con tope de 8MB. Medido con `/usr/bin/time -l` y el binario de la rama: blob de 200MB en UNA linea citado como `:1` cuesta 9.6MB de RSS contra 211MB de antes; blob de 200MB multilinea citado como `:1` resuelve con 9.6MB y el veredicto se estampa. Deteccion normal intacta: `inventado.rs:99999`, linea fuera de rango y `3.14:15` se niegan los tres nombrando el AC. Residuo abajo: el tope tiene un falso negativo | cubierto |

## El recorrido de las cuatro vueltas

### Primera vuelta: B1-B8

| B | Estado | Comando que lo comprobo |
| --- | --- | --- |
| B1 - sello tipeado a mano pasa el gate | resuelto con matiz aceptado | Sandbox: sello a mano sin filas por AC, `close --feature 1 --status done` exit 2 nombrando los AC sin responder. Con filas que citan archivos reales el sello a mano SI pasa; el spec ahora lo declara asi en la linea 48 en vez de desmentirse |
| B2 - sello citado dentro de un fence contaba como veredicto | resuelto en el gate | Sandbox: sello dentro de un bloque, `close --status done` exit 2. Cerrado del todo en la vuelta 4 con el tracking de fence (ver R2) |
| B3 - spec ausente o sin AC: el cierre pasaba igual | resuelto | Ver C1 |
| B4 - el repo del arnes no tenia la regla que UPDATING promete | resuelto | Ver C2 |
| B4b - comandos declarados que no pueden fallar | resuelto en la vuelta 4 | AC-8 y AC-11 se arreglaron y probaron por los dos lados en la vuelta 2; el AC-1 era el ultimo vacuo y se cerro como D1: `cargo` falso en PATH, modo mixto da rc=101 y rc=1, todo verde da rc=0 |
| B5 - parity_check no ataba la migracion (AC-9 vacuo) | resuelto | `bash tests/parity_check.sh` rc=0 mas la prueba del rojo por triplicado de la vuelta 2. Sigue evadible con un stub del mismo nombre: es lo que un chequeo declarativo promete |
| B6 - superficie que afirmaba `one_feature_at_a_time` vigente | resuelto | Ver C5. `grep -rn "una sola a la vez"` sobre roles/, templates/roles/ y .claude/agents/ rc=1 |
| B7 - `feature_list.json` de solo lectura abortaba la instalacion | resuelto | Banco aislado en la vuelta 2 con la funcion real bajo `set -Eeuo pipefail`: target 444, WARN, rc=0, archivo del usuario byte-identico. No re-ejecutado despues; el `.sh` no cambio |
| B8 - shapes degenerados en silencio, BOM divergente | resuelto | Mismo banco, 7 shapes, todos WARN o migracion limpia; BOM migra con `utf-8-sig` como `ConvertFrom-Json` |

### Segunda vuelta: C1-C6

| C | Estado | Comando que lo comprobo |
| --- | --- | --- |
| C1 - el gate del cierre sin la guarda de spec de `estampar` | resuelto | Binario recompilado, las dos variantes del ataque en sandbox: spec borrado mas sello a mano da exit 2 "No se pudo leer el spec"; spec con los `- AC-n:` envueltos en un fence da exit 2 "no declara ningun AC-n". Status `in_progress` en las dos |
| C2 - el repo del arnes sin `require_review`, con UPDATING prometiendolo | resuelto | Solo lectura: `json.load` sobre el `feature_list.json` del checkout principal. `require_review: true` esta; `one_feature_at_a_time`, `require_tests_to_close` y `require_impact_check` ya no |
| C3 - el texto promete lo que el gate no comprueba | resuelto en la vuelta 4 | El SDD se corrigio en la vuelta 3 (docs/prd/SDD-master.md:219 dice "sube el costo"); el spec cerro en la vuelta 4, ver D2 |
| C4 - el comando del AC-1 puede dar verde falso | resuelto en la vuelta 4 | Ver D1 |
| C5 - declarar limpio lo no verificado | resuelto | docs/impl-64.md:126 dice "B6: lo declare limpio y no lo estaba", con el porque; docs/prd/aprendizaje/PRD-aprendizaje.md:188 dice que la regla dejo de bloquear en la #47 y que la #64 la borro del molde, con el si explicito del usuario para la ruta protegida y rastro en el historial |
| C6 - el `catch` mudo del `.ps1` | resuelto por LECTURA | setup_harness.ps1:597 nombra archivo, tipo de excepcion y remedio, en paridad con el `.sh`. No hay `pwsh` en esta maquina: lo unico ejecutado del lado Windows es la paridad declarativa, `bash tests/parity_check.sh` rc=0 |

### Tercera vuelta: D1-D2

| D | Estado | Comando que lo comprobo |
| --- | --- | --- |
| D1 - el guard del AC-1 no podia disparar (doble `cd rust`) | resuelto | Reproducido con un `cargo` falso en PATH y el comando textual de la linea 107 del spec: mixto con exit 101 da rc=101; mixto forzando exit 0 da rc=1 (el `grep -q FAILED` dispara solo); todo verde rc=0; `0 passed` rc=1. Con el cargo real, rc=0 en 5142ms, que coincide con docs/verify-64.md:9 y entierra los 493ms del guard vacuo. docs/impl-64.md:175 ahora cuenta la causa ("mi prueba del rojo verifico el grep, no el comando") en vez de declararlo hecho |
| D2 - el spec prometia mas enforcement del que hay, y estaba sin re-firmar | resuelto | `grep "fabrica en cinco segundos"` sobre el spec: 0 ocurrencias. La linea 48 declara el limite con las palabras exactas. Re-firma: `progress/history.md` del checkout principal tiene `approve-spec feature #64` a las 2026-08-30T16:45:13Z, POSTERIOR al mtime del spec (2026-08-29T01:01:56Z) y 29s antes del commit; `check-spec` rc=0. Que el sello del archivo conserve 22:16:38Z es diseño idempotente de `approve_spec`, no incoherencia |

### Cuarta vuelta: las tres regresiones que abrio el arreglo de la tercera

| R | Estado | Comando que lo comprobo |
| --- | --- | --- |
| R1 - `estampar` borraba prosa citada dentro de fences `~~~` | resuelto para el caso par | Sandbox con el binario de la rama: review con la linea del sello citada dentro de un bloque `~~~`, `revision --veredicto changes_requested` estampa tras el titulo y el diff muestra SOLO la linea nueva; la cita sobrevive intacta. Ojo: queda vivo el caso IMPAR, ver observacion 1 |
| R2 - fences mezclados desincronizaban el gate | resuelto | Sandbox: review sin sello real pero con un bloque de backticks que contiene lineas `~~~` y un sello adentro; `close --status done` da exit 2 con `[GATE]` y el status queda `in_progress`. El parser recuerda cual fence abrio; test nuevo `gate_review_should_not_desync_with_mixed_fences` verde |
| R3 - el tope de `cita_resuelve` contaba lineas, no bytes | resuelto, con residuo | Medido con `/usr/bin/time -l`: 200MB en una linea baja de 211MB a 9.6MB de RSS; el multilinea resuelve y estampa. La direccion del error es segura (niega, no acepta), pero niega de mas: ver observacion 2 |

## Observaciones vivas: lo que este gate NO cubre

Esta es la seccion importante. Ninguna de estas cinco bloquea el merge, y las
cinco son razones para no confiar de mas en la proxima feature.

1. **`estampar` todavia muta prosa del reviewer con fences `~~~` impares.** El
   arreglo de "recordar cual fence abrio" aterrizo en el parser del gate
   (rust/src/revision.rs:434) pero NO en el loop de limpieza
   (rust/src/commands/revision.rs:121), que sigue togleando con cualquiera de
   los dos fences. Repro ejecutada: un bloque de backticks con UNA sola linea
   `~~~` adentro y despues una linea del sello citada; al registrar
   `changes_requested` el sello se estampa bien pero la linea citada
   desaparece del archivo. Con DOS `~~~` adentro los toggles se alinean de
   casualidad y sobrevive. Es la misma asimetria de dos parsers en desacuerdo
   que causo la observacion 1 de la tercera vuelta, movida un metro. El GATE no
   se ve afectado (probado: `close` da rc=2 con el sello adentro del bloque).
   Costo: se destruye contenido del reviewer en un rincon rebuscado. Fix:
   reusar el mismo `abierto_con` en el limpiador.
2. **El tope de 8MB niega citas validas.** Direccion segura, pero es un falso
   negativo real: una cita a un archivo que existe cuya linea cae mas alla de
   los primeros 8MB se rechaza aunque resuelva (medido con el blob de una
   linea). Para archivos fuente reales nunca aplica. Ademas el comentario de
   rust/src/revision.rs:526 sigue diciendo solo "se cuentan lineas sin cargar el
   archivo entero" y no menciona el tope, y no hay test del tope en la suite
   (razonable: un fixture de 8MB pesaria). Costo: una fila honesta puede
   rechazarse sin que el mensaje explique por que.
3. **La cita tiene que RESOLVER, no ser PERTINENTE.** El gate abre el archivo y
   comprueba que tenga esa linea; no juzga si la linea habla del AC. Un review
   falso que cite archivos reales al azar pasa. Ya esta dicho asi en el spec
   (linea 48) y en el SDD (docs/prd/SDD-master.md:219). Costo: lo que sube es el
   costo de mentir, no la imposibilidad. Es el limite declarado del diseño, no
   un bug.
4. **El sello es texto y un agente lo puede tipear.** La barrera que aguanta es
   la cobertura por AC, no el sello. Filtra el descuido, no la mala fe.
5. **`parity_check` es declarativo.** Comprueba que el `.ps1` DECLARE la misma
   migracion, no que la haga: un stub con el mismo nombre lo pasa. Es
   exactamente lo que el AC-9 promete, pero conviene que quien lea sepa que no
   hay ejecucion de PowerShell detras de ningun verde de esta feature.

## Lo que NO se probo

1. **PowerShell: cero ejecucion.** No hay `pwsh` en esta maquina.
   `Migrate-HarnessRules`, su guarda y el `catch` de setup_harness.ps1:597
   estan verificados por lectura y por la paridad declarativa. Nada mas.
2. **`tests/setup_smoke.sh` no se re-ejecuto en esta vuelta.** Su verde es el
   del verify del 2026-08-29 (rc=0, 71s); lo que si se comprobo es que ningun
   archivo del commit cambio por mtime despues de esa corrida.
3. **`harness_check.sh` completo desde el worktree.** Reporta divergencia de
   espejo falsa, que el propio spec documenta en su linea 166. Hay que correrlo
   desde la raiz DESPUES del merge.
4. **La pertinencia de las citas.** No hay como probarla sin juzgar contenido;
   es el limite declarado (observacion 3).
5. **El si del usuario en el chat para la re-firma del 2026-08-30T16:45:13Z.**
   Solo verificable por el rastro en `progress/history.md`, que existe. El chat
   no es un artefacto.
6. **Fuera de alcance:** el symlink que sale del repo (aceptado como
   observacion en la vuelta 2), el grafo de impacto y `graphify` (no aplican a
   un cambio de arnes sin microservicios), y la leccion del cierre, que no se
   puede juzgar antes de cerrar.

Todos los sabotajes vivieron en sandboxes del scratchpad: el worktree y el
checkout principal quedaron sin tocar.

## Paso operativo antes del cierre

Con `require_review: true` en el backlog real, `close --status done` se va a
negar hasta que el reviewer estampe el veredicto de esta cuarta vuelta con
`harness revision --feature 64 --veredicto approved`. Ese es el ultimo paso del
reviewer, no del lider.
