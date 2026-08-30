# Review de la feature #66 - el stop hook no entra en bucle

**Veredicto global: approved** (quinta vuelta). Approved aca significa una sola
cosa, y conviene decirla sin adorno: **no se pudo romper con los casos probados**.
No significa correcto. Al final de este archivo estan, con nombre y costo, las
observaciones que quedan vivas y las nueve cosas que NO se probaron.

Lo que cierra esta vuelta: el bloqueante B7 de la cuarta (`<firma>:08` matando el
check con `value too great for base`) esta arreglado con base 10 explicita y
verificado por ejecucion Y por mutacion; el cuelgue del FIFO —que no hacia fallar
el `cat`, lo colgaba— esta cerrado por los DOS lados (lectura y escritura) con un
guard unico. Diecinueve topologias del archivo de estado, ninguna muerte, ningun
cuelgue, ningun archivo del usuario tocado, y las trece suites bash mas
`setup_smoke.sh` verdes a HEAD.

## Cobertura de los AC del spec

| AC | Cita | Como se verifico | Estado |
| --- | --- | --- | --- |
| AC-1 | setup_harness.sh:2604 | El `Stop` de Claude (los DOS bloques de settings.json, con y sin subagentes) despacha `bin/harness-hook plain stop` en vez de `harness_check.sh` directo. Verificado por `tests/setup_smoke.sh` contra el settings.json GENERADO (no por grep de la fuente) y por el modo `cableado-hooks` de parity, que exige exactamente 2 ocurrencias y prohibe cualquier `"command":` que corra el check o el guard por su cuenta | cubierto |
| AC-2 | tests/stop_hook_check.sh:50 | Modo `primera-vuelta` en un proyecto de mentira con un repo hermano sucio no resoluble: sin `stop_hook_active`, exit 2 y detalle por stderr. La primera vuelta sigue siendo la chance del agente. Re-corrido en esta vuelta, verde | cubierto |
| AC-3 | tests/stop_hook_check.sh:61 | Modo `segunda-vuelta`: con el flag en 1, imprime TODOS los problemas, agrega la linea de "no los puedo resolver solo" y sale 0. Es el corte del bucle por la señal del CLI | cubierto |
| AC-4 | tests/stop_hook_check.sh:75 | Modo `degrada-todos-los-gates`: con un gate que NO es el commit_guard (spec en draft, espejo de roles stale) tambien degrada. El corte es del check entero, no de un gate — que era el error de diseño que el spec vino a corregir | cubierto |
| AC-5 | tests/stop_hook_check.sh:90 | Modo `centinela-sin-flag`: sin que el CLI mande nunca `stop_hook_active`, el mismo conjunto de fallos repetido llega al tope de racha y el check avisa y sale 0. El corte no depende del CLI | cubierto |
| AC-6 | tests/stop_hook_check.sh:103 | Modo `centinela-reinicia`: cambiado el conjunto de fallos, la racha vuelve a 1 y el Stop bloquea otra vez. La firma es del CONJUNTO (ordenada, con el detalle adentro), no de la cantidad: un problema nuevo siempre merece su vuelta | cubierto |
| AC-7 | harness_check.sh:653 | El AC de esta vuelta. Matriz propia de 15 valores de racha corrida bajo watchdog de 15 s con la firma REAL que el propio check escribio (`08`, `09`, `007`, `0x10`, `00`, veinte digitos, `-1`, vacio, con espacios) mas 5 con firma no coincidente: rc SIEMPRE 0 o 2, nunca 1, nunca cuelgue. Mutacion con `cmp` previo (borrada la linea `10#`): el modo `estado-degrada` se pone ROJO con el mensaje exacto; restaurado, verde. El fix es identico en `templates/harness_check.sh` (diff vacio) | cubierto |
| AC-8 | tests/commit_guard_check.sh:209 | Modo `nombra-archivos`: exit 2, el mensaje NOMBRA el archivo ajeno, NO nombra los artefactos del arnes (que estan exentos) y ofrece las tres salidas, incluida "si no es tuyo, decilo y no lo commitees" y el `off` que apaga el guard entero | cubierto |
| AC-9 | setup_harness.sh:2583 | El `PreToolUse` usa `SURFACE_BASE`, no `HOOK_BASE`: en layout subdir la ruta con `HOOK_BASE` apuntaba a `<raiz>/<subdir>/bin/harness-hook`, que no existe (127) y dejaba la capa de rutas protegidas como promesa vacia. `setup_smoke.sh` instala en subdir y ejecuta el comando tal como quedo escrito; parity ademas falla si `HOOK_BASE/bin/harness-hook` reaparece en la fuente | cubierto |
| AC-10 | tests/parity_check.sh:234 | Modo `cableado-hooks`, reescrito como AFIRMACION POSITIVA tras haber sido atravesado por tres mutantes en una vuelta anterior: cuenta los Stops al runtime, exige el PreToolUse con SU evento, prohibe `"command":` con `harness_check`/`commit_guard`, greppea el instalador SIN comentarios (un literal comentado lo ponia verde con el cableado roto) y verifica el .ps1 por paridad declarativa | cubierto |
| AC-11 | setup_harness.sh:1409 | El AC que se llevo cuatro versiones para volver a donde empezo: `grep -oE` con adyacencia clave-valor, here-string en vez de pipe, `tail -1` para que gane la ultima ocurrencia y `|| true` porque el hook corre con `set -Eeuo pipefail` y el caso NORMAL (sin la clave) sale 1. Modo `payload-grande` verde; medido lineal (1 MB en 0.48 s contra los 19.6 s por 200 KB del intento intermedio) y sin el falso positivo del `cwd` | cubierto |
| AC-12 | tests/parity_check.sh:256 | El chequeo cuenta los Stops y los `timeout` adyacentes y falla si no coinciden, aceptando `"timeout":` de JSON y `timeout =` de TOML. El Stop de Claude, que corre el gate mas pesado, era la unica superficie sin declararlo: ahora declara 120 s | cubierto |
| AC-13 | docs/impl-66.md:77 | MANUAL, y lo corri yo. Prueba del rojo del arreglo entero: revertido el cableado, el bucle vuelve y `cableado-hooks` lo dice con nombre; aplicado, no se reproduce. En esta vuelta se sumaron tres mutaciones mas, cada una con `cmp` antes y despues: borrar el `10#` (AC-7 en rojo), quitar el `[ -f ]` del `cat` y quitar el guard del escritor (las dos revivieron el cuelgue del FIFO). El runner cuenta "0 manual(es)" porque solo cuenta los AC con Comando; la ejecucion esta aca | cubierto |

## Las cinco vueltas, y el patron que las une

Esta feature necesito cinco revisiones. No por ambicion del spec —el spec fue
estable desde `512d490`— sino porque el implementer produjo **siete bugs de la
misma familia**, todos con la misma forma: *una linea que hace que el check
MUERA en vez de DECIDIR*.

- **Cinco por `set -e`**: en un script con `set -Eeuo pipefail`, cualquier
  comando que sale distinto de 0 —un `grep` sin match, un pipeline, un `cat` de
  algo que no existe— mata el proceso. Y un Stop que muere con rc=1 no bloquea:
  cierra el turno sin veredicto, que es exactamente el bug que la feature vino a
  arreglar. Se cerraron de a uno hasta el barrido de `|| true` del `c463686`.
- **Uno por octal** (B7, cuarta vuelta): `08` son puros digitos, pasa el filtro de
  `racha_de`, y bash lo lee como octal en `$(( ))`. Fue la SEXTA muerte de la
  clase y la primera que no era un pipeline: por eso el barrido de `|| true` —que
  fue correcto— no la encontro (harness_check.sh:642 y :653).
- **Uno por cuelgue** (quinta vuelta): el FIFO no hace FALLAR el `cat`, lo
  CUELGA. Peor que morir: como hook, hasta que el timeout de 120 s mate el turno;
  a mano, para siempre. Y se arreglo mal dos veces, de a un lado —primero el
  symlink, despues un `cat`, despues el OTRO `cat`— hasta que se lo cerro con un
  guard unico arriba de todo, porque abrir un FIFO para ESCRITURA tambien bloquea
  (harness_check.sh:696).

El patron es uno solo: **el implementer trataba el estado local como un archivo
que se lee, y el reviewer como una superficie de ataque**. Cada vuelta encontro
un valor mas raro del mismo archivo de una linea.

Y aparte esta el **AC-11, que se llevo cuatro versiones para volver a donde
empezo**: era `printf | grep -q`; una revision teorizo un EPIPE bajo `pipefail`
que NO se reproduce (medido hasta 8 MB); se "endurecio" a un `case` que acepta
cualquier `true` posterior a la clave, y con un `cwd` como `/Users/alan/truenorth`
el flag salia 1 y la primera vuelta dejaba de bloquear; se arreglo con un recorte
de prefijo que en bash es CUADRATICO (1 MB ~8 minutos contra un timeout de 120 s);
y termino en el `grep` de siempre. La leccion, cara y ya escrita en el codigo: no
se endurece codigo que funciona contra un bug que no se pudo reproducir. El texto
del AC tambien tuvo que ceder dos veces —de "no pasa por un pipe" a "lineal y con
adyacencia"— porque la letra prometia mas fuerte que el codigo.

## Lo que NO se probo

Nombrarlo es la mitad del approved:

1. **PowerShell**: no hay `pwsh` en esta maquina. El `.ps1` se verifico solo por
   paridad declarativa (que el Stop despache al runtime), nunca ejecutando.
2. **Los CLIs reales**: todo Stop fue simulado por entorno y stdin. No se midio
   que hacen Claude, Codex, Gemini o Kimi de verdad con un rc=1, ni al vencer el
   timeout de 120 s.
3. **Concurrencia**: dos Stops escribiendo `.stop_streak` a la vez.
4. **Un device de bloque/caracter REAL** como `.stop_streak` (exige root). Solo
   se probo symlink a `/dev/null`, que el guard resuelve por la rama `-L`.
5. **El binario Rust del cierre**: `deny_check` y `verify_vacio_check` corrieron
   con el binario PRESTADO del repo principal, porque el worktree no tiene uno
   compilado. Es valido —la #66 no toca Rust, `git diff --stat` lo confirma— pero
   no es el binario del cierre.
6. **AC-13 no se re-ejecuto entero** en esta vuelta: se completo en la segunda y
   nada posterior toca ese mecanismo. Lo que si se corrio aca son las tres
   mutaciones nuevas.
7. **La suficiencia del timeout de 120 s** en el multi-repo real.
8. **La aprobacion del spec y las re-firmas** no se re-verificaron por binario en
   esta vuelta acotada (se verificaron en la cuarta contra `progress/history.md`
   del repo principal; el spec no cambio desde `512d490`).
9. **El header del verify dice "0 manual(es)"** aunque el spec declara AC-13 como
   MANUAL: es preexistente, el runner solo cuenta los AC con `Comando`. No es un
   bug de la #66, pero el verify miente por omision y conviene saberlo.

## Observaciones vivas, con su costo

Ninguna bloquea. Estan aca para que la proxima feature no confie de mas.

- **`docs/verify-66.md` esta SIN COMMITEAR** (`M` en git status). Se regenero a
  las 23:44:23Z, posterior a `be6a221`, y el diff contra la copia de HEAD es solo
  timestamps salvo AC-7, que pasa de 362 a 1214 ms — consistente con los casos
  octal y FIFO nuevos. **Costo si se ignora**: el HEAD que se mergea a main porta
  un verify de un codigo anterior. Es exactamente la regla que dejo la #63. El
  commit de cierre TIENE que incluirlo (docs/verify-66.md:3).
- **El candado residual sigue abierto por un lado.** Un
  `HARNESS_STOP_HOOK_ACTIVE=1` que quedo exportado en la terminal del usuario
  degrada la primera corrida a mano. El caso `=0` residual —el daño real, y el
  que rompia la promesa "a mano nunca degrada"— esta cubierto por
  `HARNESS_HOOK_EVENT` y su modo `a-mano-no-degrada`. **Costo**: la promesa "a
  mano nunca degrada" tiene una excepcion que no esta escrita en el spec
  (harness_check.sh:719).
- **Instalacion a medias**: hook viejo + check nuevo = el centinela no corre pero
  la degradacion por flag sigue. Falla hacia BLOQUEAR, no es peor que antes de la
  #66, y esta medido y documentado. **Costo**: quien actualice el arnes sin
  re-correr el instalador tiene media feature y ningun aviso
  (docs/impl-66.md:336).
- **El guard mira el archivo, no la ruta de arriba.** Con `progress/` como symlink
  a un directorio del usuario, el check decide bien (rc=2) pero crea
  `.stop_streak` DENTRO del directorio apuntado. No pisa nada, esta fuera del
  Given del AC-7 (que enumera estados del ARCHIVO) y es consistente con que todo
  el estado del arnes —`history.md` incluido— seguiria ese symlink. **Costo**: el
  guard no debe leerse como proteccion de la ruta completa, porque no lo es
  (harness_check.sh:696).
- **El caso octal podria degradar en silencio en el futuro.** `firma_real` sale de
  un `cut` sin assert de no-vacio; si cambiara el formato del estado se
  escribiria `:08`, la firma vacia nunca coincidiria, la aritmetica no se
  alcanzaria y el modo pasaria verde probando nada (rc=2 esta en el conjunto
  permitido). HOY funciona: la mutacion del `10#` puso el modo en rojo, o sea que
  la aritmetica SI se alcanza. **Costo**: un `[ -n "$firma_real" ] || fail` de una
  linea evita que este test se vuelva decorativo sin que nadie se entere
  (tests/stop_hook_check.sh:161).
- **La regresion del FIFO se detecta COLGANDO el test, no fallandolo.** El modo no
  tiene watchdog propio: si el cuelgue vuelve —las mutaciones lo confirman—
  `estado-degrada` espera hasta que algo externo lo mate. El comentario del test
  lo admite con todas las letras. Un timeout portable en bash puro es caro y no lo
  exijo. **Costo**: quien corra la suite sin supervision (CI sin timeout de job)
  ve un cuelgue, no un rojo (tests/stop_hook_check.sh:171).

## Pendiente de proceso

El veredicto queda por estampar con
`harness_cli revision --feature 66 --veredicto approved` desde el hub: desde este
worktree esta vedado, igual que en la cuarta vuelta.
