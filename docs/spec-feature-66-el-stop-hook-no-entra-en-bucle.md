# Spec - Feature #66: el_stop_hook_no_entra_en_bucle

Estado: approved
Aprobado: 2026-08-30T19:57:33Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-66-el-stop-hook-no-entra-en-bucle.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan termina un turno y el arnes le dice "Cambios sin commitear en: docs /
[Harness] Check fallo con 1 problema(s)". El agente vuelve a arrancar, mira, no
encuentra nada suyo que arreglar —lo sucio es de otra sesion, o de otro repo del
multi-repo— y termina el turno otra vez. El hook vuelve a fallar. **No hay
ninguna accion que lo satisfaga**, y Alan lo reporta con tres palabras: "esto
esta pasando seguido".

La mitigacion existe desde la #52 y esta bien hecha: `bin/harness-hook` lee el
JSON del evento una sola vez, saca `stop_hook_active` —que significa "este turno
YA es consecuencia de un bloqueo mio"— y lo pasa por entorno. La cumplen cinco de
las seis superficies: Codex, Gemini, Grok, Kimi, y hasta el `.claude/settings.json`
que escribe el instalador de **PowerShell**. La unica que no es
`.claude/settings.json` en **POSIX con subagentes** —el default, el de Alan—, que
sigue llamando `harness_check.sh` derecho porque ese bloque es anterior a
`bin/harness-hook` y nadie lo migro cuando la #52 lo creo. Dos escritores de
hooks, uno no se entero.

Pero cablearlo no alcanza, y esto es lo que la investigacion cambio del
diagnostico: **`HARNESS_STOP_HOOK_ACTIVE` tiene un solo consumidor**,
`commit_guard.sh:161`. En `harness_check.sh` la variable aparece unicamente en un
comentario (`:127`); el cuerpo no la lee nunca y sale 2 por cualquiera de sus
otros gates. Medido: con el contrato honrado y un spec en `draft` —el estado
NORMAL justo despues de `start`, cuyo remedio EXIGE el si del usuario— el Stop
sigue saliendo 2 para siempre. El bucle no es del guard: es de clase.

Y el bucle lo corta hoy el CLI, no el arnes. De Claude y Kimi hay evidencia de
que mandan `stop_hook_active`; de Codex, Gemini y Grok no hay una sola linea en
el repo. El arnes esta apostando a un comportamiento que nunca midio, en un
producto que se define como multi-LLM.

DESPUES: el fin de turno deja de poder trabarse. La primera vuelta bloquea igual
—es la unica chance del agente de arreglar lo que SI es suyo— y la segunda
imprime todo lo que encontro, dice con todas las letras **"esto no lo puedo
resolver yo, decidilo vos"** y sale 0. Si el CLI no manda la señal, el arnes se
da cuenta solo: si el MISMO conjunto de fallos se repite, corta igual. Y cuando
lo sucio es de otro, el guard nombra los archivos y ofrece la salida que hoy
falta —"si no es tuyo, decilo y no lo commitees"— en vez de empujar a commitear
trabajo ajeno a ciegas.

## Hoy -> Como va a funcionar

```
HOY                                       DESPUES

Stop (Claude/POSIX)                       Stop (todas las superficies)
  |__ harness_cli autocheck                 |__ bin/harness-hook plain stop
  |__ harness_check.sh   <-- directo             |__ lee el JSON UNA vez
        |__ commit_guard  (</dev/null)           |__ exporta HARNESS_STOP_HOOK_ACTIVE
              |__ stop_hook_active: MUERE        |__ autocheck (repara lo que puede)
        |__ ~10 gates mas: exit 2                |__ harness_check.sh
              (ninguno mira la señal)                  |__ commit_guard
                                                       |__ los ~10 gates
                                                       |__ 2a vuelta -> avisa y sale 0

el CLI corta el bucle (o no)              el arnes tambien:
  Claude: si (documentado)                  progress/.stop_streak
  Kimi:   si (medido en la #8)                <firma del conjunto de fallos>:<n>
  Codex/Gemini/Grok: SIN EVIDENCIA            mismo conjunto N veces -> avisa y sale 0

PreToolUse (layout subdir)                PreToolUse
  $HOOK_BASE/bin/harness-hook               la ruta del runtime de verdad
  -> exit 127, la capa de PREVENCION        -> la capa que rutas-protegidas.md
     no corre NUNCA                            promete, corriendo
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero que el fin de turno nunca quede trabado por algo que el
  agente no puede resolver, para no tener que apagar el guard entero.
- P1: Como agente, quiero que el arnes me diga cuando un problema NO es mio, para
  no commitear trabajo ajeno tratando de satisfacer un gate.
- P1: Como Alan, quiero que el corte no dependa de que cada CLI se acuerde de
  mandar un flag, porque el arnes es multi-LLM.
- P2: Como Alan, quiero que la capa de prevencion de rutas protegidas corra de
  verdad en el layout que uso, o que el arnes deje de prometerla.
- P2: Como mantenedor, quiero que no pueda volver a existir un cableado de hooks
  que se saltee el contrato sin que un test lo note.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una instalacion POSIX con subagentes, When se lee el `Stop` de
  `.claude/settings.json`, Then despacha a `bin/harness-hook` como las otras
  cinco superficies, y no llama a `harness_check.sh` directo.
  Comando: `bash tests/setup_smoke.sh >/dev/null 2>&1`

- AC-2: Given un proyecto con algo sucio que el agente no puede resolver, When el
  Stop corre por PRIMERA vez (sin `stop_hook_active`), Then bloquea con exit 2 y
  el detalle por stderr: la primera vuelta sigue siendo la chance del agente.
  Comando: `bash tests/stop_hook_check.sh primera-vuelta`

- AC-3: Given el mismo proyecto, When el Stop corre con `stop_hook_active: true`,
  Then `harness_check.sh` imprime TODOS los problemas encontrados, agrega una
  linea que dice que no los puede resolver solo y **sale 0**.
  Comando: `bash tests/stop_hook_check.sh segunda-vuelta`

- AC-4: Given un gate que NO es el commit_guard (spec en draft, espejo de roles
  stale), When corre la segunda vuelta, Then tambien degrada: el corte es del
  check entero, no de un gate.
  Comando: `bash tests/stop_hook_check.sh degrada-todos-los-gates`

- AC-5: Given un backend que NUNCA manda `stop_hook_active`, When el mismo
  conjunto de fallos se repite en Stops consecutivos, Then a la N-esima vez el
  check avisa y sale 0 igual: el corte no depende del CLI.
  Comando: `bash tests/stop_hook_check.sh centinela-sin-flag`

- AC-6: Given que el conjunto de fallos CAMBIA entre dos Stops, When corre el
  centinela, Then la racha se reinicia: un problema nuevo vuelve a bloquear.
  Comando: `bash tests/stop_hook_check.sh centinela-reinicia`

- AC-7: Given `progress/.stop_streak` ausente, vacio, con basura o sin permisos,
  When el check lo lee, Then degrada al default y NUNCA hace fallar el comando.
  Comando: `bash tests/stop_hook_check.sh estado-degrada`
  <!-- CORRECCION (2026-08-30, al implementar): este AC declaraba
       `cargo test stop_streak`, asumiendo que el centinela seria Rust. Se
       implemento en SHELL, dentro de `harness_check.sh`, porque ahi estan los
       fallos y ahi se decide el exit code; en Rust habria hecho falta un comando
       nuevo solo para consultarlo, o sea un peldaño mas bajo por nada. El modo
       `estado-degrada` prueba exactamente lo que el AC pide: ausente, vacio, con
       basura y multilinea, ninguno hace fallar el check. -->

- AC-8: Given un repo hermano sucio con archivos que no son artefactos del arnes,
  When el guard bloquea, Then el mensaje NOMBRA los archivos no exentos y ofrece
  tres salidas, incluida "si no es tuyo, decilo y no lo commitees".
  Comando: `bash tests/commit_guard_check.sh nombra-archivos`

- AC-9: Given una instalacion en layout subdir, When se ejecuta el comando del
  hook `PreToolUse` tal como quedo escrito, Then corre el runtime de verdad (no
  sale 127) y la capa de prevencion de rutas protegidas funciona.
  Comando: `bash tests/setup_smoke.sh >/dev/null 2>&1`

- AC-10: Given los dos instaladores, When se comparan los cableados de hooks de
  cada superficie, Then declaran el mismo runtime para el mismo backend, y un
  cableado que no pase por `bin/harness-hook` pone el chequeo en rojo.
  Comando: `bash tests/parity_check.sh`

- AC-11: Given el JSON del Stop con un payload grande, When `run_stop` busca
  `stop_hook_active`, Then lo detecta sin depender del tamaño: la deteccion no
  pasa por un pipe.
  Comando: `bash tests/stop_hook_check.sh payload-grande`
  <!-- CORRECCION (2026-08-30, al implementar): este AC nacio de un hallazgo que
       NO se pudo reproducir. La premisa era que `printf | grep -q` bajo
       `set -o pipefail` devuelve el EPIPE de `printf` cuando `grep -q` sale
       temprano, dejando el flag en 0 con el JSON diciendo `true`. Medido en bash
       de macOS con payloads de 200 KB, 1 MB y 8 MB y el match al principio:
       detecta SI en los tres casos, y el `rc` crudo del pipeline con `pipefail`
       da 0. El cambio a `case` se hace igual —es mas simple y saca una
       dependencia sutil del tamaño del buffer del pipe— pero queda declarado
       como ROBUSTEZ, no como correccion de un bug observado. Dejar el AC con la
       redaccion original habria sido afirmar lo que no se pudo comprobar. -->

- AC-12: Given el `Stop` de Claude, When se lee su declaracion, Then tiene
  `timeout` como las otras cuatro superficies (era la unica sin declararlo, y es
  la que corre el gate mas pesado).
  Comando: `bash tests/parity_check.sh`

- AC-13 (MANUAL): Given el bucle reproducido en un sandbox, When se aplica el
  cambio, Then el bucle no se reproduce; y con el cambio revertido, vuelve. Es la
  prueba del rojo del arreglo entero, y la corre el reviewer.

## Los datos que se tocan

- disparador: el evento `Stop` (y sus equivalentes `AfterAgent` / `SessionEnd` /
  `SessionStop`), en cualquiera de las seis superficies.
- señal externa: `stop_hook_active` del JSON del evento -> `HARNESS_STOP_HOOK_ACTIVE`.
  Sigue siendo el camino normal.
- señal propia: `progress/.stop_streak`, un dotfile por concepto siguiendo
  `docs/lecciones/estado-local-en-progress.md`: **una linea**
  `<firma del conjunto de fallos>:<n>`, declarado en `HarnessPaths`, con toda
  lectura degradando al default. La firma es del CONJUNTO de fallos, no de su
  cantidad: si cambia lo que falla, la racha se reinicia.
- interruptor: los que ya existen (`HARNESS_CHECK_MODE`,
  `HARNESS_COMMIT_GUARD_MODE`). No se agrega ninguno.
- candado: la degradacion solo ocurre en la segunda vuelta o con la racha
  cumplida; una corrida a mano (`bash harness_check.sh`) bloquea como siempre.

## Pseudo-codigo (el acuerdo)

```
CUANDO llega un evento de fin de turno

  el hook lee el JSON UNA vez y exporta si este turno ya es consecuencia
  de un bloqueo suyo                      (una sola puerta: bin/harness-hook)

  autocheck repara lo que se puede reparar solo, y avisa si no pudo

  el check corre TODOS sus gates y junta lo que encontro

  ¿no encontro nada?                 -> sale limpio
  ¿es la primera vuelta?             -> bloquea: es la chance del agente
  ¿es la segunda vuelta,
   o la misma racha por N-esima vez? -> IMPRIME TODO, dice que no lo puede
                                        resolver solo, y DEJA CERRAR el turno

  ENTONCES el fin de turno nunca queda sin salida,
           con la restriccion de que el problema no se oculta: se muestra
           entero y se dice de quien es la decision.
```

Promesas: la primera vuelta bloquea igual que hoy · nada se oculta, la segunda
vuelta imprime MAS, no menos · el corte no depende del CLI · el estado del
centinela nunca hace fallar un comando · correr el check a mano no degrada nunca.

## No funcionales

- SLOs: el hook no agrega trabajo; `autocheck` deja de duplicarse (hoy se corre
  en el comando Y dentro de `run_stop`).
- Seguridad: el centinela es un dotfile local de una linea; no viaja en el merge
  ni sale del proyecto.
- Observabilidad: la salida de `autocheck` deja de ir a `/dev/null` —hoy se traga
  el unico aviso de que fallo— y pasa a stderr, como ya hace `run_stop`.

## Fuera de alcance

- **La linea de base de suciedad por sesion** (que el guard solo cuente lo que se
  ensucio DURANTE la sesion). Es el arreglo estructural del caso "no es mio", con
  precedente en `progress/.rutas_arnes`, pero cambia la semantica del gate y
  merece su propio spec. Aca solo se arregla el REMEDIO (AC-8).
- Reclasificar gate por gate en "auto-reparable" vs "solo el usuario decide". La
  #66 degrada el check entero en la segunda vuelta; el reparto fino es otra
  feature.
- Medir empiricamente que manda cada CLI en el JSON del Stop. El centinela existe
  justamente para no depender de ese dato.

## Observaciones (decisiones pendientes)

- **Decisiones del usuario ya tomadas (2026-08-30)**: (1) degrada el check
  entero, no solo el guard; (2) flag + centinela propio, porque hoy la defensa la
  presta el CLI; (3) el `PreToolUse` roto en layout subdir entra en esta feature,
  porque es el mismo bloque que no migro y cerrar la #66 dejando viva una promesa
  que sabemos falsa seria repetir lo que la #63 se prohibio; (4) el remedio del
  guard nombra archivos y ofrece la tercera salida.
- **El valor de N del centinela** (cuantas rachas iguales antes de cortar) queda
  a elegir al implementar: 2 es el minimo util y coincide con la semantica de
  `stop_hook_active`. Si preferis otro numero, decilo en el review.
- `Peldano elegido:` **no se agrega ningun comando ni superficie nueva**. Se
  reusa `bin/harness-hook` (que ya existe y ya cumple el contrato) y un dotfile
  en `progress/` (patron ya establecido por `.last_nudge` y `.last_autocheck`).
  El unico archivo nuevo es `tests/stop_hook_check.sh`, que es andamiaje de
  pruebas y no superficie de producto.
