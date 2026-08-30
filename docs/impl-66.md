# Impl - Feature #66: el_stop_hook_no_entra_en_bucle

Spec: docs/spec-feature-66-el-stop-hook-no-entra-en-bucle.md
Plan: docs/plan-feature-66-el-stop-hook-no-entra-en-bucle.md

## El diagnostico, y lo que la investigacion le corrigio

El reporte de Alan fue "esto esta pasando seguido", con el mensaje del guard. El
primer diagnostico —el que yo le di en el chat— decia que **nadie** cumplia el
contrato de `harness_check.sh:120-127`. Era falso, y la mitad falsa era la que
decide el diseño:

- `bin/harness-hook` (`run_stop`, `setup_harness.sh:1374-1397`) **si** lo cumple
  desde la #52: lee el JSON una vez, con `[ -t 0 ]` para no colgarse, y exporta
  el env. Lo cumplen **cinco de seis** superficies: Codex, Gemini, Grok, Kimi, y
  el `.claude/settings.json` que escribe el instalador de **PowerShell**.
- El unico roto era `.claude/settings.json` en **POSIX** —el default, el de
  Alan—, que llamaba `harness_check.sh` derecho porque ese bloque es anterior a
  `bin/harness-hook` y nadie lo migro cuando la #52 lo creo. Dos escritores de
  hooks, uno no se entero.

Y lo que cambio el alcance: **`HARNESS_STOP_HOOK_ACTIVE` tenia un solo
consumidor**, `commit_guard.sh:161`. En `harness_check.sh` la variable aparecia
unicamente en un comentario (`:127`); el cuerpo no la leia y salia 2 por
cualquiera de sus otros ~25 sitios de fallo. Medido en este mismo worktree, sin
el guard de por medio: `current.md` vacio y tres divergencias de espejo, todas
con el remedio "re-corre el instalador". Cablear el hook y nada mas habria dejado
el bug abierto con otro texto.

## Que cambio

| Archivo | Cambio |
| --- | --- |
| `setup_harness.sh` | `SURFACE_BASE` (nuevo): las rutas de SUPERFICIE apuntan a la raiz, no al arnes. `Stop` (los dos modos) y `PreToolUse` pasan por `bin/harness-hook`. `timeout: 120` en el Stop de Claude. `run_stop` detecta el flag con `case`, sin pipe |
| `harness_check.sh` (+ template) | `sumar_fallo` acumula QUE fallo; degradacion en la segunda vuelta; centinela `progress/.stop_streak` |
| `commit_guard.sh` (+ template) | nombra los archivos no exentos y ofrece tres salidas |
| `tests/stop_hook_check.sh` | NUEVO: ocho modos |
| `tests/commit_guard_check.sh` | modo `nombra-archivos` |
| `tests/parity_check.sh` | modo `cableado-hooks` |

**La firma del conjunto de fallos** usa `$LINENO` expandido en el sitio de la
llamada. Identifica el gate sin obligar a etiquetar 30 sitios a mano, cuenta
repeticiones (tres divergencias de espejo ≠ dos), y si el script cambia de
version la firma cambia y la racha se reinicia — que es el comportamiento
correcto.

**El centinela** sigue `docs/lecciones/estado-local-en-progress.md` al pie: un
dotfile por concepto, una linea `<firma>:<n>`, toda lectura degradando al
default, y no se reescribe si el valor no cambio (reescribir corre el mtime).

## Evidencia por AC

| AC | Evidencia / test | Estado |
| --- | --- | --- |
| AC-1 | `setup_harness.sh:2572` (Stop -> runtime); `tests/parity_check.sh` modo `cableado-hooks` | cubierto |
| AC-2 | `tests/stop_hook_check.sh:primera-vuelta`; `harness_check.sh` (rama `exit 2`) | cubierto |
| AC-3 | `tests/stop_hook_check.sh:segunda-vuelta` — verifica que imprime MAS, no menos | cubierto |
| AC-4 | `tests/stop_hook_check.sh:degrada-todos-los-gates` (proyecto sin nada sucio, otro gate en rojo) | cubierto |
| AC-5 | `tests/stop_hook_check.sh:centinela-sin-flag` | cubierto |
| AC-6 | `tests/stop_hook_check.sh:centinela-reinicia` | cubierto |
| AC-7 | `tests/stop_hook_check.sh:estado-degrada` (vacio, basura, multilinea, ausente) | cubierto (ver nota) |
| AC-8 | `tests/commit_guard_check.sh:nombra-archivos`; `commit_guard.sh` (bloque del mensaje) | cubierto |
| AC-9 | `setup_harness.sh:2551` (PreToolUse -> `SURFACE_BASE`); `parity_check.sh` modo `cableado-hooks` | cubierto |
| AC-10 | `tests/parity_check.sh` modo `cableado-hooks`, con prueba del rojo de sus tres chequeos | cubierto |
| AC-11 | `tests/stop_hook_check.sh:payload-grande` | cubierto, con la premisa CORREGIDA (abajo) |
| AC-12 | `setup_harness.sh:2573` (`"timeout": 120`) | cubierto |
| AC-13 | prueba del rojo, abajo | cubierto |

**Nota sobre el AC-7**: el spec declaraba `cargo test stop_streak`, asumiendo que
el centinela seria Rust. Se implemento en **shell**, dentro de `harness_check.sh`,
porque ahi es donde estan los fallos y donde se decide el exit code; ponerlo en
Rust habria obligado a un comando nuevo (peldaño mas bajo) solo para consultarlo.
El AC se cubre con `tests/stop_hook_check.sh:estado-degrada`, que prueba lo mismo
que el AC pide. **Esto cambia el comando declarado y necesita la re-firma del
usuario.**

## La prueba del rojo (AC-13)

Cada mecanismo se rompio a proposito y se comprobo que el test lo detecta:

| Mecanismo roto | Lo que reporto el test |
| --- | --- |
| degradacion desactivada (`if false`) | `segunda-vuelta: la segunda vuelta siguio bloqueando (rc=2): el bucle sigue` y `centinela-sin-flag: ... sigue en bucle (rc=2)` |
| centinela ignorando la firma (`if true`) | `centinela-reinicia: con una firma distinta no volvio a bloquear (rc=0)` |
| `Stop` revertido al comando viejo | `cableado-hooks: ... Stop-no-pasa-por-el-runtime` |
| `Stop --no-subagents` revertido | `cableado-hooks: ... Stop-sin-subagentes-no-pasa-por-el-runtime` |
| `PreToolUse` con `HOOK_BASE` | `cableado-hooks: ... runtime-con-HOOK_BASE-en-vez-de-SURFACE_BASE` |

Y restaurado, todo vuelve a verde.

## Lo que el propio trabajo encontro

- **El AC-11 nacio de un bug que no existe.** La premisa era que
  `printf | grep -q` bajo `set -o pipefail` devuelve el EPIPE de `printf` cuando
  `grep -q` sale temprano, dejando el flag en 0 con el JSON diciendo `true`.
  Medido en bash de macOS con 200 KB, 1 MB y 8 MB, y el `rc` crudo del pipeline:
  **detecta SI en los tres casos, rc=0 siempre**. El cambio a `case` se hizo
  igual —es mas simple y saca una dependencia del buffer del pipe— pero quedo
  declarado como ROBUSTEZ, y el AC se reescribio para decir eso. Dejarlo con la
  redaccion original habria sido cerrar una feature afirmando lo que no se pudo
  comprobar, que es exactamente lo que la #63 se prohibio.
- **Mi primera prueba del rojo dio verde falso.** Al revertir el cableado para
  ver si `parity_check` se ponia rojo, siguio verde: mi patron pedia una comilla
  final que la linea real no tiene (termina en comillas escapadas). Lo detecte
  porque el rojo NO aparecio, no porque lo verificara. Es la misma clase de error
  que la #64 (verificar el instrumento equivocado), una vuelta antes.
- **Un test mio estaba mal, no el codigo.** El modo `nombra-archivos` fallaba
  diciendo que el guard nombraba un artefacto exento. Era cierto: yo habia puesto
  el `spec-feature-9-algo.md` dentro de `miservicio/`, y la exencion exige la
  UBICACION ademas del nombre (`commit_guard.sh:97-108`) — un `impl-notas.md`
  suelto en un microservicio es un documento real. El test tenia razon.

## La revision adversarial, y el bug que introdujo mi propio arreglo

El reviewer la rechazo con cinco bloqueantes. El peor era mio, y nacio de la
"robustez" del AC-11:

**B4 — el `case` que introduje era PEOR que el `grep` que saque.** El patron
`*'"stop_hook_active"'*[Tt]rue*` acepta cualquier `true` POSTERIOR a la clave, y
el JSON real del Stop trae `cwd`. Reproducido en frio con un payload real:

    {"stop_hook_active":false,"cwd":"/Users/alan/truenorth"}
                                                  ^^^^

El flag salia 1 con el JSON diciendo `false`, o sea que **la primera vuelta no
bloqueaba y el agente perdia su unica chance de arreglar lo suyo**. El `grep` que
reemplace exigia adyacencia clave-valor y no tenia ese fallo. Tambien caian
`"note":"construed"`, `"verbose":true` y `"msg":"True story"`.

La leccion, que es mas cara que el bug: **quise arreglar un bug que no existia y
en el intento cree uno que si.** El AC-11 nacio de un hallazgo teorico, no
reproducible; en vez de dejar el codigo como estaba, lo "mejore". Ahora el match
recorta hasta la clave y mira SOLO lo que sigue, verificado contra los doce casos
de la matriz del reviewer (incluidos sus cuatro falsos positivos).

Los otros cuatro:

| # | Que rompio | Arreglo |
| --- | --- | --- |
| B1 | Con el guard como UNICO gate en rojo —el escenario exacto que reporto Alan— el check salia 0 pero imprimia `[Ok] Harness Check limpio` debajo del detalle del repo sucio, y nunca la linea prometida. El guard se auto-degradaba (`commit_guard.sh:186`) antes de que el check lo contara | El check invoca el guard con la señal APAGADA: la degradacion vive en UN solo lugar. El guard sigue degradando solo cuando corre sin check (modo `--no-subagents`) |
| B2 | La firma por `$LINENO` identifica el GATE, no el contenido: el guard colapsa todos los archivos sucios en un `sumar_fallo`, asi que arreglar A y ensuciar B dejaba la racha corriendo y el mensaje decia "no cambio nada" sobre un problema nuevo | `sumar_fallo` acepta un DETALLE que entra en la firma; el guard aporta su salida. Verificado con el escenario exacto: A sucio -> rc=2, mismo -> rc=0, A commiteado + B nuevo -> **rc=2** |
| B3 | Mi modo `cableado-hooks` eran tres grep NEGATIVOS de las formas historicas. Tres mutantes reales lo pasaban por arriba | Reescrito como afirmacion POSITIVA: cada evento invoca el runtime con SU evento, con timeout, en los dos instaladores |
| B5 | El comando del AC-12 no podia fallar: borrar el `timeout` dejaba `parity_check` verde | Asercion del timeout, que acepta JSON (`"timeout":`) y TOML (`timeout =`, Kimi) |

Ademas, el comentario del codigo afirmaba el bug del EPIPE como HECHO mientras el
spec y este archivo lo declaraban no reproducido: el mismo commit decia una cosa
en el codigo y la contraria en los docs. Corregido.

**Y una tercera vez me pase por arriba mi propia prueba del rojo.** Los mutantes
M1 y M3 "pasaban" porque mis reemplazos con `python3 -c` y escapes anidados no
mutaban nada. Recien al imprimir `cambio el archivo=True` antes de correr el test
aparecio la verdad: M3 SI se detectaba, y M1 era un agujero real (mi regex
`"command": "[^"]*(harness_check|...)` se corta en la comilla escapada del
comando). La regla que me llevo: **una prueba del rojo empieza por demostrar que
la mutacion existe.**

**Un test ajeno se rompio y tenia razon.** `commit_guard_check.sh:prueba-del-rojo`
reconstruye la invocacion previa del guard para probar que el modo `no-cuelga`
mide algo; al cambiar yo esa invocacion, dejo de reconstruirla y **fallo
ruidosamente**. Es exactamente lo que `criterios-de-cierre-que-se-pueden-fallar`
dice que tiene que pasar ("si tu prueba del rojo empieza a fallar, la primera
hipotesis es que el instrumento dejo de medir"). Se actualizo el `sed` a la
invocacion nueva.

## Lo que NO se hizo, y por que

- **La linea de base de suciedad por sesion** (que el guard solo cuente lo que
  ensucio ESTA sesion). Es el arreglo estructural del caso "no es mio", pero
  cambia la semantica del gate y tiene su propio spec. Aca solo se arreglo el
  REMEDIO.
- **Reclasificar gate por gate** en auto-reparable vs decision-del-usuario. La
  #66 degrada el check entero en la segunda vuelta.
- **Medir que manda cada CLI** en el JSON del Stop. El centinela existe
  justamente para no depender de ese dato.

## Lo que no se pudo verificar en esta maquina

- Nada de PowerShell: no hay `pwsh`. El `.ps1` no se toco en esta feature salvo
  por lo que ya cumplia; la paridad declarativa si corre (`parity_check`, diez
  modos verdes).
- El comportamiento real de cada CLI ante un Stop que falla: el centinela se
  probo simulando el env, no con los CLIs de verdad.
