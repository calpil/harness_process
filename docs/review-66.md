# Review - Feature #66: el_stop_hook_no_entra_en_bucle

**Veredicto global: changes_requested** (cuarta vuelta) — un solo bloqueante, otra
vez de una linea, otra vez de la MISMA clase: `progress/.stop_streak` con
`<firma-actual>:08` mata el check con rc=1 (`08: value too great for base`) en vez
de decidir. Es la sexta muerte de la clase en esta feature y la primera
ARITMETICA, por eso el barrido de `|| true` del `c463686` —que fue correcto— no la
vio: `08` son puros digitos, pasa el filtro de `racha_de` y bash lo octaliza.

Los dos puntos abiertos de la tercera vuelta (B6 y C1) estan cerrados y
re-verificados ejecutando, no leyendo: los cinco escenarios de symlink/permisos
dan rc=2 con el aviso y el archivo del usuario intacto, y la letra del AC-11 por
fin coincide con el mecanismo, con re-firma trazada. El implementer ademas cerro
por su cuenta dos muertes mas de la clase en `c463686`. Lo que queda es angosto
—hace falta que la firma guardada coincida con el conjunto de fallos actual, cosa
que ninguna escritura del propio check produce— pero rompe el invariante que la
feature acaba de declarar con todas las letras, y cae dentro del Given literal del
AC-7.

## Cobertura de los AC del spec

| AC | Cita | Como se verifico | Estado |
| --- | --- | --- | --- |
| AC-1 | setup_harness.sh:2604 | El `Stop` de POSIX-con-subagentes despacha `bin/harness-hook plain stop`, no `harness_check.sh`; instalacion end-to-end de esta vuelta via `bash tests/setup_smoke.sh`, rc=0. En la tercera vuelta se leyo ademas el `.claude/settings.json` GENERADO en un sandbox real y se ejecuto el comando tal cual quedo escrito | cubierto |
| AC-2 | harness_check.sh:715 | `bash tests/stop_hook_check.sh primera-vuelta` re-corrido por mi a HEAD, verde; y en las ~20 secuencias de esta vuelta la primera corrida SIEMPRE termino en ese `exit 2` con el detalle por stderr. La chance del agente sigue intacta | cubierto |
| AC-3 | harness_check.sh:707 | `segunda-vuelta` verde a HEAD; con `stop_hook_active:true` sale 0, imprime `[Harness] No bloqueo el cierre del turno` con el conteo real de problemas y CERO apariciones de `Harness Check limpio`. Imprime MAS, no menos | cubierto |
| AC-4 | harness_check.sh:651 | `degrada-todos-los-gates` verde a HEAD: la decision se toma sobre `$failures` acumulado de todos los gates (spec en draft, espejo de roles stale), no sobre el commit_guard. El corte es del check entero | cubierto |
| AC-5 | harness_check.sh:700 | `centinela-sin-flag` verde a HEAD, y secuencias 2->0 con payload `false` repetido: a la segunda sale 0 con `pedi lo mismo 2 veces seguidas`. N=2 es el valor elegido, coincide con la semantica de `stop_hook_active`, lo acepto | cubierto |
| AC-6 | harness_check.sh:657 | `centinela-reinicia` y `centinela-problema-nuevo` verdes a HEAD; la firma es del CONJUNTO ordenado de sitios, con el detalle del guard adentro (fix de B2) y las lineas `[i]` de repos ajenos afuera (fix de O7). Probado por mutacion en la vuelta 3 | cubierto |
| AC-7 | harness_check.sh:645 | **NO cubierto, y es el bloqueante.** Lo que B6 pedia esta hecho y lo verifique ejecutando: symlink + `progress/` 555, symlink normal, `progress/` 555 sin symlink, `.stop_streak` como directorio, `chmod 000` — los cinco dan rc=2 con el aviso, el archivo del usuario intacto, y en el caso escribible el symlink se reemplaza por el archivo real. Pero `<firma-actual>:08` mata el check con rc=1. Ver B7 | no cubierto |
| AC-8 | commit_guard.sh:176 | `bash tests/commit_guard_check.sh nombra-archivos` verde; el guard NOMBRA los no exentos y ofrece las tres salidas, con la segunda diciendo textualmente que si no es tuyo NO lo commitees. Verificado end-to-end en el sandbox de la vuelta 3 | cubierto |
| AC-9 | setup_harness.sh:2583 | El `PreToolUse` apunta a `$SURFACE_BASE/bin/harness-hook`, la ruta del runtime de verdad. En la vuelta 3 se ejecuto el comando EXACTO del `settings.json` generado en layout subdir: `deny` con ruta protegida, `{}` con ruta normal, ningun 127. `setup_smoke.sh` rc=0 a HEAD | cubierto |
| AC-10 | tests/parity_check.sh:265 | `bash tests/parity_check.sh` verde a HEAD (diez modos). Los 7 mutantes de la vuelta 2 y el M4b de la vuelta 3 (literales escondidos en comentarios) siguen rojos: el modo filtra comentarios, exige `"command":` y cuenta los dos Stops de Claude | cubierto |
| AC-11 | setup_harness.sh:1409 | C1 cerrado. La letra nueva del Then (lineal + adyacencia clave-valor) coincide con el mecanismo real, y la propiedad esta MEDIDA por `payload-grande`, que extrae el matcher REAL del instalador: matriz de 15 payloads, 200 KB en 0 s, verde re-corrido por mi a HEAD. La re-firma del usuario existe y es posterior a la edicion del spec | cubierto |
| AC-12 | tests/parity_check.sh:255 | El conteo de timeouts es POR Stop (`grep -A1` del despacho), no global; el `.claude/settings.json` generado trae `"timeout": 120` en los dos bloques Stop (setup_harness.sh:2605 y :2637). El mutante que borra las declaraciones pone `cableado-hooks` en rojo | cubierto |
| AC-13 | harness_check.sh:630 | MANUAL. Corrido completo en la segunda vuelta: clone del sandbox con codigo pre-#66 -> rc=2, 2, 2 con `stop_hook_active:true`, el bucle VUELVE; con el codigo nuevo, 2 y despues 0. No re-ejecutado en esta vuelta; ni `512d490` ni `c463686` tocan ese mecanismo, y las secuencias 2->0 de hoy lo re-ejercitan por el lado verde | cubierto |

## El recorrido de las cuatro vueltas

| Vuelta | Que se reporto | Como cerro |
| --- | --- | --- |
| 1 | Cinco bloqueantes: AC-3 no imprimia la linea prometida y stdout decia "limpio" (B1); un problema NUEVO no reiniciaba la racha y el mensaje mentia (B2); tres mutantes con el cableado roto quedaban verdes (B3); el `case` nuevo del AC-11 tenia falsos positivos (B4); el timeout declarado no podia fallar (B5) | `bead02a`, los cinco. B4 tardo hasta la vuelta 3 |
| 2 | Ocho observaciones: symlink clobber, estado ilegible en silencio, candado "a mano" abrible con un `=0` residual, el fix de B2 sin test, `cableado-hooks` evadible por comentario, clave duplicada ganando la primera, la firma reiniciandose por ruido ajeno, el sello del spec sin re-estampar | `cca9274`, las ocho. Pero el remedio del symlink abrio B6 |
| 3 | Un bloqueante (B6: `[ -L ] && rm` mata el check con `progress/` en solo-lectura) y un cambio pedido (C1: la letra del AC-11 decia "no pasa por un pipe" y el mecanismo final es un pipeline) | `512d490` (B6 + C1) y `c463686` (barrido de la clase por iniciativa del implementer) |
| 4 | Un bloqueante (B7: `08` octalizado) y tres observaciones vivas | pendiente |

### El patron que une las cuatro

Tres de los bugs del implementer en esta feature son **la misma clase**: un
endurecimiento que, al agregarse, agrega una forma nueva de morir bajo
`set -Eeuo pipefail`. No son descuidos distintos, es un unico reflejo.

1. **El matcher del AC-11** (`grep -oE ... | tail -1`, setup_harness.sh:1409). El
   endurecimiento era la adyacencia clave-valor y el `tail -1` de O6. La muerte:
   un payload SIN la clave hace salir a `grep` con 1 y `pipefail` mata el hook con
   rc=1 y stderr vacio. Se cerro con el `|| true`, que el propio comentario del
   codigo declara obligatorio y no cosmetico. Este lo pesco el implementer solo.
2. **El remedio del symlink** (`[ -L ] && rm -f`, hoy harness_check.sh:680). El
   endurecimiento era no escribir a traves de un symlink del usuario (O1). La
   muerte: en una lista `&&` el ultimo comando SI dispara `set -e`, asi que un
   `rm` que falla por `progress/` en solo-lectura mataba el check. Fue B6, el
   bloqueante de la tercera vuelta. El remedio actual cierra ademas la secuela:
   con el `rm` fallido ya no se escribe a traves del symlink (harness_check.sh:687).
3. **Las tuberias del centinela** (`| cksum | cut` en `sumar_fallo`,
   `| tr | sort | tr` en `firma=`, harness_check.sh:657). Mismo mecanismo, sin
   reporte de por medio: el implementer barrio la clase entera en `c463686` y
   agrego el modo `herramientas-rotas` (tests/stop_hook_check.sh:234). Lo
   verifique como al resto, sin creerle al verde: mutaciones M-B y M-C sacando
   cada `|| true`, con `cmp` previo, ponen el modo en rojo con `el check murio
   (rc=1) en vez de decidir`; restaurado, verde.

**B7 es la sexta muerte de la clase y la primera que no es un exit status de un
comando externo, sino aritmetica de bash.** Por eso el barrido no la alcanzo: un
`|| true` no la habria evitado. La clase real no es "falta un `|| true`", es "toda
expresion que toca el estado del centinela puede matar el check", y el estado es
texto que el arnes no controla.

### El AC-11: cuatro versiones para volver a donde empezo

1. `783f862` — `printf '%s' "$json" | grep -q ...` bajo `pipefail`. Sospechado de
   devolver el EPIPE de `printf` cuando `grep -q` sale temprano. **No se pudo
   reproducir** (medido hasta 8 MB, rc=0 siempre). El AC se corrigio inline y se
   re-firmo declarando el cambio ROBUSTEZ, no correccion de un bug observado.
2. `bead02a` — `case` con recorte de prefijo. Saco el pipe, y trajo **cuatro
   falsos positivos** (cualquier `true` posterior a la clave: `".../truenorth"`,
   `"verbose":true`, `"True story"`) que se comian la primera vuelta del agente,
   mas una regresion **cuadratica** medida: 200 KB en 19.6 s contra un timeout de
   120 s.
3. `a36098c` — vuelve el `grep`: el codigo original estaba bien.
4. `cca9274` — el `grep` con here-string, adyacencia clave-valor, `tail -1` y el
   `|| true`. Y en `512d490`, la segunda correccion inline del Then, porque la
   redaccion "la deteccion no pasa por un pipe" era falsa contra un mecanismo que
   ES un pipeline.

Saldo: cuatro versiones de codigo y dos correcciones de la letra del AC, para
terminar en el matcher original mas dos arreglos reales (adyacencia y ultima
ocurrencia). Lo que se gano de verdad no fue el matcher: fue que `payload-grande`
(tests/stop_hook_check.sh:157) dejo de inlinear una copia del patron y ahora
**extrae el matcher real del instalador**, probado por mutacion en los dos
sentidos. Un AC que promete una propiedad del mecanismo obliga a medir el
mecanismo, y las dos primeras redacciones prometian mas fuerte que el codigo: la
leccion de la #63, cobrada dos veces en la misma feature.

## El bloqueante

**B7 — AC-7: `progress/.stop_streak` con `<firma-actual>:08` mata el check con
rc=1 en vez de decidir.**

`harness_check.sh:645` es `echo $((n_previo + 1))`. `08` pasa el filtro
`''|*[!0-9]*)` de `racha_de` —son puros digitos— y bash lo interpreta como octal:
`08: value too great for base`. Bajo `set -Eeuo pipefail` el check muere ahi.

Reproducido contra `harness_check.sh` **y** contra `templates/harness_check.sh:645`,
que es identico y se instala en cada proyecto. Cae dentro del Given literal del
AC-7 (`con basura`, docs/spec-feature-66-el-stop-hook-no-entra-en-bucle.md:121-122:
`NUNCA hace fallar el comando`) y el sintoma es el exacto de B6: **un Stop con
rc=1 no bloquea**, el turno cierra sin veredicto, que es peor que cualquiera de
los dos finales legitimos.

Es el mas angosto de los seis: exige que la firma guardada coincida con el
conjunto de fallos actual, y ninguna escritura del propio check produce un `08`
—hace falta edicion manual o corrupcion quirurgica—. Pero rompe el invariante que
la propia feature acaba de declarar (`ninguna parte del centinela puede matar el
check`, el titulo del ultimo commit), y el estado es un dotfile en el arbol de
trabajo del usuario.

Arreglo de una linea, verificado: `n_previo=$((10#$n_previo))` despues del `case`
da 9 con `08` bajo `set -Eeuo pipefail`. Y el caso `<firma>:08` deberia entrar a
`estado-degrada` (tests/stop_hook_check.sh:115), que hoy prueba basura SIN firma
coincidente y por eso nunca llega a la aritmetica.

## Observaciones vivas, con su costo

**V1 — un FIFO como `.stop_streak` no hace fallar el check: lo CUELGA.** El `cat`
de `racha_de` (harness_check.sh:638; el otro esta en :669) bloquea esperando un
escritor que no existe. Medido: el check sigue vivo a los 5 s y hubo que matarlo.
Como hook, el timeout de 120 s lo corta y el turno cierra sin veredicto; a mano,
cuelga para siempre. Costo: bajo, la precondicion es exotica y esta FUERA del
Given enumerado del AC-7 (ausente / vacio / basura / sin permisos), por eso no es
bloqueante. Pero si se toca `racha_de` para el caso `:08`, un guard de
archivo-regular (`[ -f ]` es falso para FIFOs y para directorios) cierra las dos
cosas de paso, y de yapa el caso "directorio" que hoy se cubre por otro camino.

**V2 — la evidencia archivada corrio un `harness_check.sh` viejo.**
`docs/verify-66.md:3` dice `Corrida: 2026-08-30T21:53:32Z`, y quedo commiteada en
`512d490`; el codigo final es `c463686` (19:06:16 -0400, o sea posterior). El
verify verde que esta en el repo NO corrio contra el codigo que se va a mergear.
Costo: documental, no sustantivo — yo re-ejecute los comandos de los 12 AC
automatizables a HEAD (10 modos de `stop_hook_check.sh`, 10 de `parity_check.sh`,
`commit_guard_check.sh` y `setup_smoke.sh`, todo rc=0), asi que la sustancia esta.
Pero el cierre deberia regenerar el verify contra HEAD, o el artefacto afirma algo
que no midio: otra vez la regla de la #63.

**V3 — el candado "a mano" sigue abrible con residuos.** `harness_check.sh:700`
mira el flag sin exigir evento: un `HARNESS_STOP_HOOK_ACTIVE=1` residual degrada
la PRIMERA corrida a mano, y un `HARNESS_HOOK_EVENT=stop` residual degrada la
segunda. Son residuos raros (quedan de debuggear un hook simulando su entorno) y
`=1` significa semanticamente "el CLI declaro reintento", asi que honrarlo es
defendible. Costo: bajo. Lo dejo anotado para que la promesa `correr el check a
mano no degrada NUNCA` no se lea como absoluta.

**V4 — instalacion a medio actualizar: el centinela muere en silencio.** Con el
hook viejo (que no exporta `HARNESS_HOOK_EVENT`) y el check nuevo, el centinela
nunca cuenta y `.stop_streak` no se crea. Costo acotado: falla hacia BLOQUEAR, no
es peor que el estado pre-#66, y `UPDATING.md` manda re-correr el instalador, que
escribe hook y check juntos. Medido y documentado por el implementer en
`docs/impl-66.md`. Detectable barato: `HARNESS_STOP_HOOK_ACTIVE` definida +
`HARNESS_HOOK_EVENT` ausente = hook viejo.

## Lo que NO se probo

Aunque el veredicto de esta vuelta es `changes_requested` y no `approved`, la lista
va igual, porque el dia que esto se apruebe hay que leerlo como **"no se pudo
romper con los casos probados"** y no como "es correcto":

- **PowerShell.** No hay `pwsh` en la maquina. El `.ps1` se verifico solo por la
  paridad declarativa de `tests/parity_check.sh` (diez modos verdes re-corridos
  por mi) y por mutacion de texto. El instalador de Windows no se ejecuto.
- **Los CLIs reales.** Todos los Stop fueron payloads simulados por stdin. NO se
  midio que hace Claude Code de verdad con un `rc=1` ni al vencer el timeout de
  120 s: B7, como B6 antes, asume la semantica documentada.
- **Concurrencia**: dos Stops escribiendo `.stop_streak` a la vez.
- **`check-spec` por binario**: no hay binario compilado en el worktree. La
  aprobacion y las CUATRO firmas se verificaron LEYENDO el `progress/history.md`
  del repo principal (lineas 387, 389, 391 y 392), no ejecutando el comando.
- **AC-13 no re-ejecutado en esta vuelta** (completo en la segunda; ni `512d490`
  ni `c463686` tocan ese mecanismo).
- **Instalacion manual con inspeccion del `settings.json` generado**: la de esta
  vuelta fue via `tests/setup_smoke.sh` (rc=0). La inspeccion a mano fue en la
  tercera.
- **Suficiencia del timeout de 120 s** en el multi-repo real de Alan.

## Registro del veredicto

El veredicto NO quedo estampado con `harness revision --feature 66 --veredicto`:
ese comando escribe en el hub de `/Users/alan/harness_process`, que me esta
vedado. El veredicto a estampar es **changes_requested**, por B7 (arreglo de una
linea mas su caso en `estado-degrada`), sin otras condiciones. V1 se cierra gratis
en el mismo toque; V2 lo cierra el propio cierre regenerando el verify.
