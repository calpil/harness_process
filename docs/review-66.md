# Review - Feature #66: el_stop_hook_no_entra_en_bucle

**Veredicto global: changes_requested** (tercera vuelta) — un solo bloqueante y de una linea: el remedio de la observacion O1 (no escribir a traves de un symlink) introdujo una muerte por `set -e` que rompe el AC-7 dentro de su propio Given, y hace que un Stop salga con rc=1, que en Claude no bloquea nada.

Las ocho observaciones de la segunda vuelta estan resueltas y re-verificadas end-to-end contra una instalacion real (sandbox v3, layout subdir, instalador de la rama), no leyendo el diff. El AC-11 —el que se llevo cuatro versiones— quedo cerrado en sustancia y medido lineal. Lo que queda abierto es de la misma clase que el implementer ya pesco solo una vez en esta feature: un endurecimiento que agrega una forma nueva de morir.

## Cobertura de los AC del spec

| AC | Cita | Como se verifico | Estado |
| --- | --- | --- | --- |
| AC-1 | setup_harness.sh:2604 | Instalacion real layout subdir en sandbox del scratchpad; se leyo el `.claude/settings.json` GENERADO (el `Stop` de POSIX-con-subagentes despacha `bash ".../bin/harness-hook" plain stop`) y se ejecuto el comando tal cual quedo escrito: entra al runtime, no llama `harness_check.sh` directo. `bash tests/setup_smoke.sh` re-corrido por mi en esta vuelta: rc=0 | cubierto |
| AC-2 | harness_check.sh:697 | `bash tests/stop_hook_check.sh primera-vuelta` verde, mas el sandbox con un unico gate rojo estable: Stop por `bin/harness-hook` con payload `stop_hook_active:false` -> rc=2 y el stderr nombra el archivo. La primera vuelta sigue siendo la chance del agente; re-verificado en cada una de las 17 corridas de la matriz | cubierto |
| AC-3 | harness_check.sh:689 | Mismo sandbox, mismo unico gate rojo: payload `true` -> rc=0, aparece `[Harness] No bloqueo el cierre del turno` con el conteo correcto de problemas, y CERO apariciones de `[Ok] Harness Check limpio`. La segunda vuelta imprime MAS, no menos | cubierto |
| AC-4 | harness_check.sh:647 | `bash tests/stop_hook_check.sh degrada-todos-los-gates` verde, mas sandbox con el espejo de roles desincronizado (gate que no es el guard): con flag degrada igual. El corte decide sobre el acumulado de todos los gates, no sobre uno | cubierto |
| AC-5 | harness_check.sh:682 | `bash tests/stop_hook_check.sh centinela-sin-flag` verde, mas sandbox con payload `false` dos veces y el mismo conjunto: rc=2 y despues rc=0 con `pedi lo mismo 2 veces seguidas`. N=2 es el valor elegido, coincide con la semantica de `stop_hook_active` y me parece bien | cubierto |
| AC-6 | harness_check.sh:164 | Secuencia real A-commiteado + B-nuevo por `bin/harness-hook`: 2, 0, 2 (nombrando a B), 0. Ahora ademas con test propio (`modo_centinela_problema_nuevo`) y con la prueba del rojo por mutacion: sembrada la regresion EXACTA de B2 (`sumar_fallo "$LINENO"` sin el detalle) el modo nuevo la detecta con rc=1; restaurado, verde | cubierto |
| AC-7 | harness_check.sh:670 | **NO cubierto, y es el bloqueante.** Los demas casos del Given pasan: ausente, vacio, con basura, multilinea, `chmod 000`, `.stop_streak` como directorio y `progress/` en 555 dan rc=2 con el aviso `[i] No se pudo escribir ...`. Pero `.stop_streak` como SYMLINK con `progress/` en solo-lectura mata el check: medido rc=1. La promesa literal del AC (`NUNCA hace fallar el comando`) se rompe. Ver B6 | no cubierto |
| AC-8 | commit_guard.sh:176 | `bash tests/commit_guard_check.sh nombra-archivos` verde, mas el sandbox: el guard nombra `Archivos que no son artefactos del arnes:` y lista las tres salidas, con la segunda diciendo textualmente que si no es tuyo NO lo commitees. Es el remedio que el spec pidio | cubierto |
| AC-9 | setup_harness.sh:2583 | Se ejecuto el comando EXACTO del `PreToolUse` del `settings.json` generado por la instalacion subdir: con `file_path` a un documento protegido devuelve el JSON de `deny`, con ruta normal `{}`. Ningun 127. La capa de prevencion que `rutas-protegidas.md` promete corre de verdad en el layout de Alan | cubierto |
| AC-10 | tests/parity_check.sh:236 | Los 7 mutantes de la vuelta 2 siguen rojos, y el que se escapaba ya no: mutante M4b (Stop despachando `harness_status.sh` + los literales del runtime y del timeout escondidos en COMENTARIOS) da rc=1 con `esperaba-2-hooks-al-runtime-y-hay-0`; restaurado rc=0. El modo ahora filtra comentarios, exige `"command":` y CUENTA los dos Stops de Claude | cubierto |
| AC-11 | setup_harness.sh:1409 | Matriz de 17 payloads end-to-end por stdin contra el `bin/harness-hook` GENERADO: los 4 falsos positivos de la vuelta 1, `stop_hook_activeX`, espacios, tabs, `True`, clave al final, payload real de Claude, clave duplicada en LOS DOS ordenes, ausente, vacio y JSON invalido: 17/17 con el rc esperado. Lineal y medido: 200 KB y 1 MB, clave al principio y al final, 0.44-0.48 s (el intermedio cuadratico tardaba 19.6 s con 200 KB). Ojo: la LETRA del Then quedo falsa, ver C1 | cubierto |
| AC-12 | tests/parity_check.sh:255 | Se leyo el `"timeout": 120` en el `.claude/settings.json` generado (setup_harness.sh:2605), y la prueba del rojo sigue: el mutante que borra las declaraciones de timeout de los tres Stop pone `cableado-hooks` en rc=1. El conteo de timeouts es por Stop, no global | cubierto |
| AC-13 | harness_check.sh:626 | MANUAL, corrido por mi en la segunda vuelta y no re-corrido en esta: clone del sandbox con el codigo de `783f862^` (pre-#66) y el cableado viejo -> rc=2, 2, 2 con `stop_hook_active:true`, el bucle VUELVE; con el codigo nuevo, 2 y despues 0. Nada de `a36098c` ni de `cca9274` toca ese mecanismo, y las secuencias 2->0 de hoy lo re-ejercitan por el lado verde | cubierto |

## El recorrido de las tres vueltas

### Vuelta 1: cinco bloqueantes (B1-B5), todos cerrados en `bead02a`

| B | Que era | Como se cerro |
| --- | --- | --- |
| B1 | AC-3: en el escenario motivador la linea prometida no se imprimia y stdout decia "limpio" | El guard se invoca con `HARNESS_STOP_HOOK_ACTIVE=0` y ya no se auto-degrada adentro del check. Re-verificado en v3: con `true`, rc=0, mensaje presente y `grep -c '[Ok] Harness Check limpio'` = 0 |
| B2 | AC-6: un problema NUEVO en el mismo gate no reiniciaba la racha y el mensaje mentia | El detalle del guard entra en la firma. Re-verificado con la secuencia A/B por el hook y, desde la vuelta 3, con test propio (O4) |
| B3 | AC-10: tres mutantes con el cableado roto quedaban verdes | Greps positivos por evento, conteo de timeouts, clausula del `.ps1`. 7 mutantes rojos con `cmp` previo |
| B4 | AC-11: el `case` nuevo tenia falsos positivos que el `grep` no tenia | Ver la historia del AC-11 abajo: se cerro recien en la vuelta 3 |
| B5 | AC-12: el comando declarado no podia fallar por el timeout | Mutante M5 rojo; el `.claude/settings.json` generado trae `"timeout": 120` |

### Vuelta 2: ocho observaciones (O1-O8), todas cerradas en `cca9274`

| O | Que era | Estado | Como lo verifique |
| --- | --- | --- | --- |
| O1 | symlink clobber en `.stop_streak` | resuelto, con secuela | El archivo apuntado quedo INTACTO y el symlink fue reemplazado por el archivo real. Pero el remedio abrio el bloqueante B6 |
| O2 | estado ilegible mataba el centinela en silencio | resuelto | Tres ataques (`chmod 000`, `.stop_streak` como directorio, `progress/` en 555): rc=2 y el `[i]` presente en las tres. El silencio se acabo (harness_check.sh:675) |
| O3 | un `HARNESS_STOP_HOOK_ACTIVE=0` residual abria el candado "a mano" | resuelto | La condicion pasa por `HARNESS_HOOK_EVENT`, que solo exporta el hook: tres corridas a mano con `=0` dan 2, 2, 2 (antes la segunda degradaba). Quedan otros residuos, ver V2 |
| O4 | el fix de B2 no tenia test | resuelto | `modo_centinela_problema_nuevo`, con la regresion exacta sembrada por mutacion y `cmp` previo |
| O5 | `cableado-hooks` evadible con el literal en un comentario | resuelto | Mutante M4b rojo; el modo cuenta los dos Stops de Claude y filtra comentarios |
| O6 | clave duplicada: ganaba la primera | resuelto | `tail -1` en el matcher: gana la ultima, como cualquier parser JSON. Los dos ordenes medidos end-to-end |
| O7 | la firma se reiniciaba por ruido de repos ajenos | resuelto | Las lineas `[i]` del guard salen de la firma: la racha continua pese a un `[i]` nuevo de otro repo. Sin test que lo proteja, ver V3 |
| O8 | el sello del spec no se re-estampo | aceptado por diseño | `rust/src/spec.rs:275` devuelve `AlreadyApproved` sin tocar el archivo; la re-firma queda en `progress/history.md` del repo principal (20:21:11Z y 20:23:47Z). Es idempotente a proposito; acepto el diseño |

### El AC-11, cuatro versiones para terminar donde empezo

1. `783f862` — `printf '%s' "$json" | grep -q ...` bajo `pipefail`. Sospechado de devolver el EPIPE de `printf` cuando `grep -q` sale temprano. **No se pudo reproducir** (medido hasta 8 MB, rc=0 siempre); el AC se corrigio inline y se re-firmo declarandolo ROBUSTEZ, no bug.
2. `bead02a` — `case` con recorte de prefijo `${var#*...}`. Saco el pipe, pero trajo **cuatro falsos positivos** (cualquier `true` posterior a la clave: `"cwd":".../truenorth"`, `"verbose":true`, `"True story"`) que se comian la primera vuelta del agente, y una regresion **cuadratica** medida: 200 KB en 19.6 s contra un timeout de 120 s.
3. `a36098c` — vuelve el `grep`: el codigo original estaba bien.
4. `cca9274` — `grep -oE '"stop_hook_active"[[:space:]]*:[[:space:]]*[A-Za-z]+' <<<"$stop_input" | tail -1 || true`. Here-string en vez de pipe de entrada, `tail -1` para que gane la ultima clave (O6), y un `|| true` que **no es decorativo**: sin el, un payload SIN la clave hace salir a `grep` con 1 y bajo `pipefail` mata el hook con rc=1 y stderr vacio. Lo probe por mutacion: sacando el `|| true`, el caso NORMAL muere; con el, rc=2 con el gate impreso. El comentario de setup_harness.sh:1406 explica por que esta ahi, que es exactamente lo que hacia falta.

El saldo del AC-11: cuatro versiones, y la que quedo es la primera mas dos correcciones reales (adyacencia clave-valor y ultima ocurrencia). Lo que se gano de verdad no es el matcher: es que `modo_payload_grande` dejo de inlinear una copia del patron y ahora **extrae el matcher real del instalador** (tests/stop_hook_check.sh:130) — probado por mutacion en los dos sentidos, regex laxa y rename de variable, los dos rojos.

## El bloqueante

**B6 — AC-7: el remedio de O1 mata el check cuando el symlink esta en un directorio de solo lectura.**

`harness_check.sh:670` es `[ -L "$streak_file" ] && rm -f "$streak_file" 2>/dev/null`. Bajo `set -Eeuo pipefail`, en una lista `&&` el ultimo comando SI dispara `set -e`. Si `progress/` esta en solo-lectura, el `rm` falla con EACCES: el `2>/dev/null` suprime el mensaje, no el exit status, y el check muere ahi.

Medido, y reproducido de nuevo al escribir este review con un `bash -c` aislado: el `echo` posterior no se ejecuta, rc=1. En el sandbox instalado, el stderr se corta en `Check fallo con 1 problema(s)` y no llega a imprimir la decision de bloquear ni la de degradar. **Un Stop que sale 1 no bloquea en Claude**: el turno cierra sin veredicto, que es peor que cualquiera de los dos finales legitimos. Antes de `cca9274` este caso daba rc=2.

Tres cosas lo agravan:

- Cae **dentro del Given del propio AC-7** (`sin permisos`), o sea no es un caso inventado por mi: es uno de los cuatro que el AC enumera, combinado con el estado que O1 introdujo.
- `templates/harness_check.sh:670` es identico: el bug se instala en cada proyecto.
- La suite entera queda **verde** con esto adentro. `modo_estado_degrada` (tests/stop_hook_check.sh:115) prueba ausente, vacio, basura y multilinea; ningun modo siembra un symlink ni un `chmod`.

Arreglo: una linea. `|| true` en el `rm`, o meter el `rm` en un `if`. Y el caso (symlink + `progress/` en solo-lectura) deberia entrar a `estado-degrada`, que de paso cubre lo de V4.

## El cambio pedido

**C1 — la letra del AC-11 quedo falsa respecto del mecanismo final.**

El Then dice `la deteccion no pasa por un pipe` (docs/spec-feature-66-el-stop-hook-no-entra-en-bucle.md:148) y setup_harness.sh:1409 **es** un pipeline (`grep -oE ... | tail -1`). El here-string saco el pipe de ENTRADA, que era el sospechoso del EPIPE; el de salida quedo, y es inofensivo: `tail` lee todo, no hay early-close posible, y esta verificado lineal (1 MB en 0.48 s).

No pido cambiar el codigo: el mecanismo actual es el correcto. Pido la **CORRECCION inline en el spec y la re-firma**, como se hizo las dos veces anteriores (AC-7 y el propio AC-11, con las re-firmas trazadas en `progress/history.md` del repo principal). Esta tercera vuelta al `grep` no dejo ninguna, y un lector del spec hoy leeria una promesa de mecanismo que el codigo no cumple. Es literalmente la regla de la #63.

## Observaciones vivas, con su costo

**V1 — instalacion a medio actualizar: el centinela muere en silencio.** Probado instalando con `bead02a` y pisando SOLO `harness_check.sh` con el de `cca9274`: el hook viejo no exporta `HARNESS_HOOK_EVENT`, asi que `harness_check.sh:659` nunca ve el evento, con payload `false` repetido da rc=2, 2, 2, 2 y `.stop_streak` nunca se crea. El caso P1 del spec (CLI sin señal) se queda sin proteccion. Costo acotado: falla hacia BLOQUEAR, o sea no es peor que el estado pre-#66, y `UPDATING.md` manda re-correr el instalador, que escribe hook y check juntos. El implementer lo midio y lo documento en `docs/impl-66.md`. Detectable barato: `HARNESS_STOP_HOOK_ACTIVE` definida + `HARNESS_HOOK_EVENT` ausente = hook viejo, un `[i]` alcanzaria.

**V2 — el candado "a mano" sigue abrible con OTROS residuos.** `harness_check.sh:682` mira el flag sin exigir evento: un `HARNESS_STOP_HOOK_ACTIVE=1` residual degrada la PRIMERA corrida a mano (medido rc=0), y un `HARNESS_HOOK_EVENT=stop` residual degrada la segunda (2, 0). Son residuos mas raros que el `=0` que se reporto en la vuelta 2 —quedan de debuggear un hook simulando su entorno— y `=1` significa semanticamente "el CLI declaro reintento", asi que honrarlo es defendible. Costo: bajo. Lo anoto para que la promesa `a mano no degrada NUNCA` no se lea como absoluta. Exigir tambien el evento en el camino del flag cerraria el primero.

**V3 — dos remedios nuevos quedaron sin proteccion de suite.** La misma clase que era O4: (a) revertir el filtro `[i]` de O7 (volver a meter el `$guard_salida` entero en la firma) deja los NUEVE modos de `stop_hook_check.sh` verdes — probado por mutacion con `cmp` previo; (b) ningun modo siembra un symlink ni un `chmod 000`, asi que el fix de O1 y el aviso de O2 tambien se pueden deshacer sin ruido. Costo: O7, O1 y O2 vuelven gratis en la proxima edicion. Si se arregla B6 con un caso nuevo en `estado-degrada`, (b) se cierra de paso.

**V4 — artefactos del arnes sin commitear en el worktree.** `docs/impl-66.md` y `docs/verify-66.md` tienen cambios en el working tree (entre ellos la corrida verde del verify de las 21:53:32Z, posterior a `cca9274`, y el bloque medido de compatibilidad de V1). Los commitea el cierre, pero que quede dicho: hoy la evidencia de la ultima corrida no esta en ningun commit.

## Lo que NO se probo

Si esto termina en `approved`, tiene que leerse como **"no se pudo romper con los casos probados"**, no como "es correcto". Lo que quedo afuera:

- **PowerShell.** No hay `pwsh` en la maquina. El `.ps1` solo se verifico por la paridad declarativa (`tests/parity_check.sh`, diez modos verdes re-corridos por mi) y por mutacion de texto. El instalador de Windows no se ejecuto.
- **Los CLIs reales.** Todos los Stop fueron payloads simulados por stdin. En particular NO se midio que hace Claude Code de verdad con un `rc=1` (el bloqueante B6 asume la semantica documentada de los Stop hooks) ni que hace al vencer el `timeout` de 120 s.
- **Concurrencia**: dos Stops simultaneos escribiendo `.stop_streak`.
- **`harness_check.sh` limpio en el checkout principal**: ejecutar ahi me estaba vedado. En el worktree da rc=2, pero es artefacto de topologia (el basename del worktree rompe la expansion `__HREL__` del espejo de roles; verifique que los espejos SI coinciden bajo `HREL=harness_process/`) mas `progress/current.md` vacio. No imputable a la feature.
- **`check-spec` por binario**: no hay binario compilado en el worktree. La aprobacion y las dos re-firmas se verificaron LEYENDO el `progress/history.md` del repo principal (lineas 387 a 391); el orden contenido-commiteado vs ultima re-firma es trazable por history, no demostrable desde el artefacto.
- **AC-13 no re-ejecutado en esta vuelta** (corrido completo en la segunda, con codigo pre-#66; nada de `a36098c` ni `cca9274` toca ese mecanismo).
- **Suficiencia del timeout de 120 s en el repo real de Alan**, con el multi-repo entero y el gate mas pesado.

## Registro del veredicto

El veredicto NO quedo estampado con `harness revision --feature 66 --veredicto`: ese comando escribe en el hub de `/Users/alan/harness_process`, que me fue vedado en esta vuelta. Estamparlo queda para el lider con el usuario. El veredicto a estampar es **changes_requested**, por B6, con C1 (correccion inline del AC-11 y re-firma) como condicion de la proxima vuelta.
