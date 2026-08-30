# Impl - Feature #64: el_arnes_no_promete_enforcement_que_no_hace

Spec: docs/spec-feature-64-el-arnes-no-promete-enforcement-que-no-hace.md
Plan: docs/plan-feature-64-el-arnes-no-promete-enforcement-que-no-hace.md

## El diagnostico, con la evidencia que lo prueba

No fue una hipotesis: quedo medido sobre este repo antes de escribir una linea.

- **El corte del review es cronologico, no disperso.** Ultimo cierre CON review:
  **#46, 2026-08-22T17:58:57Z**. Primero SIN review: **#57, 2026-08-26T00:54:21Z**.
  **Cero interleaving** (ningun cierre con review despues del primero sin). Las 15
  sin review: #38-43, #53-55, #57, #59-63. Y ninguna linea en `progress/history.md`
  decide saltearlo: la etapa no se relajo, se apago.
- **Parsear prosa no verifica nada.** De los 40 reviews existentes, **7 no
  matchean** ni un regex generoso de veredicto (#17-22, #44). Y hay un falso
  positivo ya en disco: `docs/review-3.md:3` dice
  `Veredicto: approved (implementación) — cierre BLOQUEADO por una acción humana
  pendiente`. Un `contains("approved")` aprueba un review que dice que el cierre
  esta bloqueado.
- **Las tres reglas muertas nunca funcionaron.** `require_tests_to_close`,
  `require_impact_check` y `one_feature_at_a_time`: cero lecturas en `rust/src/`
  y cero en las **7 versiones del Python previo al port** (`bkp/harness.py.bak.*`).
  No es una regresion del port a Rust: nacieron decorativas.
- **La tercera ademas era falsa.** `rust/src/commands/start.rs:63` dice desde la
  #47 que "varias features pueden estar in_progress a la vez", mientras
  `roles/leader.md:97` le seguia afirmando al lider "una sola a la vez".
- **La frescura por mtime no detecta nada.** De los 40 pares review/impl
  existentes, **cero** tienen el review mas viejo que el impl. Se descarto por
  eso y por el deadlock que documenta `documentos.rs:23-26`.

## Que cambio

| Archivo | Cambio |
| --- | --- |
| `rust/src/revision.rs` | `require_review` (default false), `veredicto_estampado` (lee SOLO el sello), `acs_sin_fila` (cobertura derivada del spec), `menciona` (token completo), `fila_responde` (exige cita), `linea_sello`, `gate` |
| `rust/src/commands/revision.rs` | `estampar()`: decide primero (`acs_sin_fila`), escribe despues; idempotente; rastro en `history.md` |
| `rust/src/cli.rs` | `--veredicto` en el subcomando `revision` |
| `rust/src/commands/close.rs:124` | el quinto gate, tras verify y antes de lecciones, en la FASE 0 |
| `templates/feature_list.json` | las tres reglas muertas fuera; `require_review: true` |
| `setup_harness.sh` | `migrate_rules()` + superficie LLM |
| `setup_harness.ps1` | `Migrate-HarnessRules` + superficie LLM |
| `roles/leader.md` (+2 espejos) | deja de afirmar "una sola a la vez" |
| `roles/reviewer.md` (+2 espejos) | ensena a registrar el veredicto |
| `tests/setup_smoke.sh` | flujo de review en el E2E de PRD, asserts de la migracion, y la evidencia que faltaba en el fixture |
| `rust/tests/cli_basics.rs` | 6 tests E2E |
| `CHECKPOINTS.md`, `AGENTS.md`, `README.md`, `UPDATING.md`, `docs/architecture.md` (+ templates) | superficies |

## Evidencia por AC

| AC | Evidencia / test | Estado |
| --- | --- | --- |
| AC-1 | `rust/tests/cli_basics.rs` `gate_review_should_block_close_when_the_verdict_is_missing`; gate en `rust/src/revision.rs:490` | cubierto |
| AC-2 | `rust/tests/cli_basics.rs` `gate_review_should_ignore_a_handwritten_verdict` + unit `rust/src/revision.rs:700` (incluye el caso real de review-3) | cubierto |
| AC-3 | `rust/tests/cli_basics.rs` `veredicto_should_refuse_without_ac_coverage`; `acs_sin_fila` en `rust/src/revision.rs:459` | cubierto |
| AC-4 | `rust/tests/cli_basics.rs` `veredicto_estampa_y_habilita_el_cierre`; `estampar()` en `rust/src/commands/revision.rs:40` | cubierto |
| AC-5 | `rust/tests/cli_basics.rs` `gate_review_should_reject_a_verdict_that_is_not_approved`; `rust/src/revision.rs:513` | cubierto |
| AC-6 | `rust/tests/cli_basics.rs` `close_should_stay_identical_without_the_review_rule` + unit `require_review_default_false`; `rust/src/revision.rs:394` | cubierto |
| AC-7 | `templates/feature_list.json:3` (solo dos reglas, ambas vivas) | cubierto |
| AC-8 | `tests/setup_smoke.sh` (bloque `MIGRATE_RULES` al final); `migrate_rules()` en `setup_harness.sh:570` | cubierto |
| AC-9 | `tests/parity_check.sh`; `Migrate-HarnessRules` en `setup_harness.ps1:573` | cubierto |
| AC-10 | `UPDATING.md:148` (el corte, con los 15 ids y las dos fechas) | cubierto |
| AC-11 | `roles/leader.md:97` + `templates/roles/leader.md:97` + `.claude/agents/leader.md:105`; gate de espejo en `harness_check.sh:222` | cubierto |
| AC-12 | prueba del rojo, abajo | cubierto |
| AC-13 | unit `la_cita_tiene_que_apuntar_a_algo_que_existe`; `cita_resuelve` en `rust/src/revision.rs:494` | cubierto |

`docs/verify-64.md`: **12 verdes, 0 rojos, 0 manuales** (AC-12 es MANUAL y lo cubre la prueba del rojo de abajo).

## La prueba del rojo (AC-12)

El instrumento se probo contra su propio fallo, no solo contra el exito:

1. **El gate.** Desactivada la llamada de `close.rs:124`, los tres tests
   `gate_review_*` pasan a **FAILED**; restaurada, vuelven a **ok**. Sin esa
   corrida, "el gate bloquea" seria una afirmacion.
2. **La migracion.** Quitada la clave del molde, `migrate_rules` no agrega nada:
   la migracion depende del molde y no de codigo hardcodeado.
3. **El `.DS_Store` del instalador** (commit `694f5e9`, aparte): quitado el
   bloque, no se siembra.

## Lo que el propio trabajo encontro

- **Un bug de mi implementacion, detectado por un test propio**:
  `contains("AC-1")` matcheaba `AC-11`. Con un spec de 12 AC —el de esta misma
  feature— el gate habria dado por cubierto el AC-1 con la fila del AC-12: un
  gate que aprueba lo que no reviso. Arreglado con `fn menciona` (match de token
  completo) y el test quedo en la suite.
- **Un nivel de log inexistente en PowerShell**: `Write-HarnessLog SUCCESS` no
  esta en el `ValidateSet` (`INFO/WARN/ERROR/OK`). Habria explotado en Windows,
  donde no hay `pwsh` en esta maquina para detectarlo compilando.
- **Un supuesto falso sobre las rutas**: la migracion usaba
  `templates/feature_list.json` fijo, que no existe en la distribucion aplanada;
  corregido a `$ASSET_DIR` / `$script:AssetDir`, que es lo que ya resuelve
  `install_asset`.
- **Un AC que se pudo fallar**: el AC-4 salio ROJO en la primera corrida de
  `verify` porque el spec nombraba `veredicto_estampa_y_habilita_el_cierre` y el
  test se llamaba distinto. El criterio funciono como criterio; se renombro el
  test, no el spec.
- **Un fallo PREEXISTENTE del smoke**: `tests/setup_smoke.sh` fallaba en `main`
  desde la #60 (el assert de `impl: docs/impl-1.md` esperaba un puntero que la
  #60 decidio no escribir cuando el archivo no existe, y el fixture no lo
  creaba). Comprobado corriendo el smoke sobre el arbol limpio, que muere en el
  mismo `grep`. Se arreglo el fixture: sin eso el AC-8 no era verificable.
- **La regla rechazo algo propio**: al encender `require_review` en el molde, el
  E2E de PRD del smoke dejo de cerrar. Era correcto —cerraba `done` sin review—
  y se pago en esta feature haciendo que el test pase por el flujo completo, que
  ademas ejercita el review de punta a punta sobre una instalacion real.

## La revision adversarial y lo que rompio (segunda vuelta)

El reviewer de esta feature la rechazo: **`changes_requested`, 8 bloqueantes**.
No fueron observaciones de estilo; tres tumbaban promesas del spec. Lo que se
arreglo, con el ataque que lo encontro:

| # | Que rompio | Arreglo | Test |
| --- | --- | --- | --- |
| B1 | El sello COMPLETO tipeado a mano pasaba el gate, con un review sin ninguna fila. El spec afirmaba "imposible de fabricar escribiendo el archivo a mano": era **falso** | El gate re-verifica la cobertura por AC ademas del sello (`revision.rs:520`), y el spec dice ahora que el sello filtra el descuido, no la mala fe | `gate_review_should_reject_a_forged_stamp_without_ac_coverage` |
| B2 | Un sello CITADO dentro de un bloque ``` se leia como veredicto; y `estampar()` borraba esa cita, mutando la prosa del reviewer. Mismo hallazgo que la #23 en `verificacion.rs:157` | `lineas_fuera_de_bloque()` (`revision.rs:407`) y el filtrado con fences en `estampar()` | `gate_review_should_ignore_a_stamp_quoted_inside_a_code_block`, `estampar_should_not_touch_a_stamp_quoted_in_prose` |
| B3 | Con el spec borrado o con 0 AC, `estampar()` sellaba `approved` sobre un review que decia "nada" (`unwrap_or_default()` + 0 AC = 0 faltantes) | Se niega si el spec no se puede leer o no declara AC (`commands/revision.rs:79`) | `veredicto_should_refuse_when_the_spec_declares_no_ac` |
| B4 | `bash tests/setup_smoke.sh 2>&1 \| tail -5`: `verify` corre sin `pipefail`, asi que el rc era el de `tail` y el AC no podia fallar. Dos AC nacieron asi | Comandos sin pipe. **Al corregirlos, AC-11 paso a ROJO al instante**: `harness_check.sh` fallaba y el pipe lo tapaba | el propio `verify` |
| B5 | El `.ps1` solo guardaba contra `$null`: con `"rules": "apagadas"` hacia backup y reescribia en CADA re-run (idempotencia rota), donde el `.sh` era no-op | Guarda alineada (`-isnot [PSCustomObject]`) + aviso | lectura (no hay `pwsh`) |
| B4b | `parity_check.sh` pasaba los ocho modos **con `Migrate-HarnessRules` borrada entera** del `.ps1`: el criterio del AC-9 no podia fallar | Modo `migracion-rules` que ata las dos implementaciones. Prueba del rojo: borrada la funcion, rc=1 | `tests/parity_check.sh` |
| B7 | `feature_list.json` de solo lectura: traceback de Python sin catch + `set -Eeuo pipefail` = **la instalacion entera abortaba** antes de sembrar hooks y constitution | El heredoc atrapa sus errores y avisa; se serializa entero antes de abrir en escritura, para no truncar el archivo del usuario | banco aislado: rc=0, archivo intacto |
| B8 | `rules: null`, `rules: []`, 0 bytes, BOM: skip **en silencio**, contra el comentario de la propia funcion. Y el BOM divergia de Windows (`ConvertFrom-Json` lo acepta) | `log_warn` nombrando las reglas que no se pudieron aplicar; `utf-8-sig` para igualar a Windows | banco aislado: 5 shapes, todos avisan |

**B6: lo declare limpio y no lo estaba.** Mi grep dijo "solo apariciones
historicas" y el reviewer encontro una viva:
`docs/prd/aprendizaje/PRD-aprendizaje.md:187` afirma "`one_feature_at_a_time`
sigue vigente; estos hitos se toman de a uno" — no es historia, es una premisa de
proceso en un PRD vivo, y `:246` la usa como tal. Se me escapo porque grepee
`--include="*.md"` sobre el worktree y **lei el resultado buscando confirmacion**,
no contradicciones: las lineas de `docs/prd/` estaban en la salida. Es ruta
protegida (`docs/prd/**`), asi que la correccion es del USUARIO y se le llevo
como bloque aparte en vez de tocarla. Lo demas de B6 si esta limpio: las otras
apariciones son de la #47, el texto que explica el cambio en `UPDATING.md` y los
asserts deliberados del smoke.

**La observacion que el usuario decidio cerrar aca** (2026-08-28): el gate no
comprobaba que las citas `archivo:linea` apuntaran a algo, asi que
`inventado.rs:99999` valia como evidencia y `3.14:15` matcheaba el patron.
Ahora `cita_resuelve` (`revision.rs:494`) exige que el archivo exista y tenga esa
linea, probando dos raices (la del repo y la del arnes, para el layout subdir), y
rechaza rutas absolutas o con `..`. Es el AC-13. El cambio rompio tres tests E2E
propios que citaban archivos inventados: se corrigieron para citar el spec del
sandbox, que existe — la regla se aplico a sus propias pruebas.

## La segunda revision, y lo que volvio a romper

El reviewer la rechazo otra vez. B1, B2, B4b, B5, B7 y B8 quedaron **resueltos y
re-verificados ejecutando**, pero aparecieron dos bloqueantes, uno de ellos
reabriendo B1 por otra puerta:

| # | Que rompio | Arreglo |
| --- | --- | --- |
| C1 | El **gate** leia el spec con `unwrap_or_default()` y no tenia la guarda que si tenia `estampar`: con el spec borrado —o con los `- AC-n:` metidos dentro de un bloque ``` despues de aprobarlo— `parsear` daba 0 AC, "faltan" quedaba vacio, y `close done` pasaba con el sello tipeado a mano. **B1 reabierto entero por otra puerta** | La misma guarda espejada en `gate()` (`revision.rs:573`): spec ilegible o 0 AC = `[GATE]` |
| C2 | El `feature_list.json` **real** seguia sin `require_review` y con las tres muertas, mientras `UPDATING.md:143` prometia que la regla ya aplicaba. La #64 se iba a cerrar sin su propio gate, igual que los 15 que la motivaron | Regla encendida y muertas borradas en el backlog real (decision del usuario 2026-08-28) |
| C4 | `cargo test gate_review` matchea **dos targets**; si solo caian los 5 E2E del close, el "2 passed" del unit satisfacia el grep y el AC-1 quedaba verde **con el gate desconectado del cierre** — justo el sabotaje de mi propia prueba del rojo | El comando exige ademas que no aparezca `FAILED` |
| C6 | El `catch` del `.ps1` era mudo donde el `.sh` avisa: un `feature_list.json` corrupto en Windows era un skip 100% silencioso | `Write-HarnessLog WARN` con el tipo de excepcion |

Y cuatro observaciones del reviewer que se arreglaron igual, porque eran baratas:
`~~~` no togglea fence en CommonMark (ahora si); `estampar` podia escribir el
sello **dentro** de un bloque, imprimir "[OK] Veredicto registrado" y que el gate
lo negara acto seguido (ahora sella fuera del bloque y **re-lee con el parser del
gate** antes de afirmar nada); `cita_resuelve` se colgaba para siempre con un
FIFO (`mkfifo`) y leia el archivo entero a memoria (ahora exige archivo regular y
cuenta lineas con `BufReader`, con tope); y `parity_check.sh` decia "los ocho
modos verdes" cuando ya corria nueve — un contador que miente, en la feature
sobre no prometer de mas.

## La tercera revision: el arreglo que no arreglaba

El reviewer volvio a rechazarla, y el hallazgo mas util fue contra MI arreglo de
C4, no contra el codigo original.

**D1 — un guard que no podia disparar.** Para que el AC-1 no diera verde con un
target en `FAILED`, escribi:

    cd rust && cargo test gate_review ... && ! (cd rust && cargo test ... | grep -q "FAILED")

Tras el primer `cd rust` la cwd YA es `rust/`, asi que el `cd rust` del subshell
falla (no existe `rust/rust`), el subshell devuelve !=0 y el `!` lo vuelve
verdadero **sin ejecutar cargo**. El guard no corria nunca. Peor: lo declare
arreglado en este mismo archivo — la misma clase de error que C5, dos secciones
mas arriba, cometida por mi mientras la documentaba.

La causa es concreta y vale mas que el sintoma: **mi prueba del rojo verifico el
`grep`, no el comando.** Simule la salida en un archivo y comprobe que el `grep`
la detectaba; nunca corri el comando declarado. El reviewer si: puso un `cargo`
falso en el PATH que emite un target ok y otro FAILED, y corrio el comando
textual. Medido despues del arreglo (mismo metodo): comando nuevo **rc=101**,
comando viejo **rc=0**.

**D2 — el spec seguia prometiendo de mas.** `spec:45` conservaba "Eso no se
fabrica en cinco segundos" despues de que la misma frase se corrigiera en el SDD.
Ahora el spec dice lo que el codigo sostiene, y nombra el limite: **la cita tiene
que resolver, no ser pertinente**; un review falso que cite archivos reales al
azar pasa. Lo que sube es el costo, no la imposibilidad.

Y tres observaciones nuevas, todas introducidas por arreglos anteriores:
`estampar` no conocia `~~~` y borraba la prosa citada ahi (la mitad de B2 que el
arreglo de ``` prometia conservar); tratar `~~~` como toggle simetrico de ```
desincronizaba el parser con fences mezclados (un ``` que contiene un `~~~`), asi
que ahora se recuerda **cual** fence abrio; y el tope de `cita_resuelve` contaba
lineas y no bytes, de modo que un blob de 200 MB en una sola linea costaba 211 MB
de RSS — ahora cuenta saltos por bytes con tope de 8 MB.

## El bug que aparecio al USAR la feature por primera vez

El review aprobado se fue a estampar y el gate lo **rechazo**: "docs/review-64.md
no responde por 6 AC del spec: AC-1, AC-3, AC-4, AC-5, AC-9, AC-13". Las citas
eran correctas. Lo que fallaba era el gate.

`rust/src/revision.rs:602` existe en el worktree (927 lineas) y NO en el checkout
principal (507): son dos versiones del mismo archivo. Las raices candidatas eran
`repo_root` y `root`, las dos apuntando al principal, asi que **una feature que
vive en un worktree citaba archivos que el gate resolvia contra otro arbol**. Es
el mismo defecto de worktree-vs-raiz que arreglaron la #60 (el PRD escrito en la
copia del worktree) y la #63 (el mensaje que nombraba un worktree ya borrado);
esta es la tercera vez que la misma confusion produce un bug.

Arreglado con `raices_de_citas` (`revision.rs:498`): la raiz de la FEATURE —el
padre de `paths.plans`, que es el `docs/` de la feature— va primero, y despues
las otras dos sin duplicar. La parte pura (`raices_desde`) se testea sin armar un
`HarnessPaths`.

Vale la pena decir COMO aparecio: no lo encontro un test ni una revision, sino
**usar la feature contra si misma**. Cuatro rondas de revision adversarial no lo
vieron porque todas corrieron en sandboxes de un solo arbol; el unico escenario
que lo expone es el real, y el arnes trabaja en worktrees desde la #47. La
leccion `probar-contra-datos-reales` ya lo dice ("una fixture prueba la mecanica,
no la calibracion") y aca se cumplio literal.

Y un segundo hallazgo del mismo momento: el binario instalado en la raiz
(`/Users/alan/harness_process/harness`, gitignored) era el viejo y no conocia
`--veredicto`, asi que el comando que el gate exige en su propio mensaje de
remedio no existia hasta reinstalarlo. Cerrar una feature que agrega un
subcomando necesita ese paso; el arnes no lo hace solo.

## Lo que NO se hizo, y por que

- **No se reconstruyeron los 15 reviews faltantes** (decision del usuario
  2026-08-28). Un review escrito despues de que el codigo se integro y funciona
  no intenta romper nada; `roles/reviewer.md:6` define el rol como lo contrario.
  El corte quedo documentado en `UPDATING.md:148` con los ids y las fechas.
- **No se comparo la frescura contra `impl-<id>.md`** (estaba en el acceptance).
  Ver el spec: deadlock del ciclo normal + cero senal en los 40 pares reales.
- **No se agrego un `check-review` a `harness_check.sh`**: correria en el hook de
  fin de turno, cuando el review por definicion todavia no existe.

## Lo que no se pudo verificar en esta maquina

- **La sintaxis de `setup_harness.ps1`**: no hay `pwsh`. La paridad declarativa
  si esta cubierta (`tests/parity_check.sh`, ocho modos verdes) y el nivel de log
  invalido se encontro leyendo el `ValidateSet`, no ejecutando.
