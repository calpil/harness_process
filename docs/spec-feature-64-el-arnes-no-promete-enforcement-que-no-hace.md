# Spec - Feature #64: el_arnes_no_promete_enforcement_que_no_hace

Estado: approved
Aprobado: 2026-08-28T22:16:38Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-64-el-arnes-no-promete-enforcement-que-no-hace.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan cierra la #63 el 27 de agosto. El arnes le dice "Feature #63
cerrada". `CHECKPOINTS.md:22` promete una casilla —"`docs/review-<feature>.md`
contiene veredicto del reviewer"— y `AGENTS.md:20` promete un paso 4: "Reviewer
verifica spec aprobado y fresco, evidencia por AC, impacto, tests, checkpoints y
estado Git". Ninguna de las dos se cumplio, y nada se entero. No fue un
descuido aislado: el ultimo cierre CON review es el **#46, del 2026-08-22**; el
primero SIN review es el **#57, del 2026-08-26**; y entre medio no hay ni un
solo caso mezclado. Quince cierres seguidos, cero interleaving, cero lineas en
`progress/history.md` decidiendo saltearlo. La etapa no se relajo: se apago, y
el arnes siguio afirmando que existia.

En el mismo archivo donde vive esa promesa hay otras tres. `feature_list.json`
declara `require_tests_to_close: true`, `require_impact_check: true` y
`one_feature_at_a_time: true`. Las tres estan en `true` y **ninguna la lee
ningun codigo**: cero ocurrencias en `rust/src/`, y cero tambien en las siete
versiones del Python anterior al port (`bkp/harness.py.bak.*`), asi que nunca
funcionaron — nacieron decorativas. La tercera es peor que muerta: es falsa.
`start.rs:63` dice "Feature #47 (AC-1): varias features pueden estar in_progress
a la vez", o sea que el arnes crea worktrees en paralelo mientras su propio
backlog le sigue afirmando al lider que trabaja "una sola a la vez"
(`roles/leader.md:97`).

DESPUES: cuando Alan cierra una feature como `done`, el arnes ya no puede decir
"revisado" sin que alguien haya escrito una revision. Dos barreras, y conviene
ser exacto sobre cuanto aguanta cada una:

- **El sello lo escribe el binario** (`revision --veredicto`). El gate lee esa
  linea y NUNCA la prosa, asi que un `Veredicto: approved` suelto no cuenta.
  Pero seamos honestos: la linea del sello es texto, y un agente decidido la
  puede tipear. **Esta barrera filtra el descuido, no la mala fe.**
- **La cobertura por AC es la que aguanta.** El gate exige una fila por cada
  AC-n **que declara el spec**, y cada fila tiene que citar `archivo:linea`
  **que resuelva**: el archivo tiene que existir y tener esa linea. Se verifica
  al estampar Y de nuevo en el cierre. Una cita inventada (`inventado.rs:99999`)
  o un numero de version (`3.14:15`) no cuentan como evidencia.

Lo que esto NO logra, dicho para que nadie confie de mas: **la cita tiene que
resolver, no ser pertinente.** Un review falso que cite archivos reales al azar
pasa. Lo que sube es el costo —hay que abrir el repo— no la imposibilidad. La
barrera se documenta por lo que filtra, no por lo que uno quisiera que filtrara.

Y el bloque `rules` deja de tener promesas de adorno: cada regla que sigue ahi
hace algo, y las que no, no estan.

## Hoy -> Como va a funcionar

```
HOY                                      DESPUES

close --status done                      close --status done
  |__ spec_gate            (bloquea)       |__ spec_gate            (bloquea)
  |__ verificacion::gate   (bloquea)       |__ verificacion::gate   (bloquea)
  |__ documentos::gate     (bloquea)       |__ documentos::gate     (bloquea)
  |__ lecciones::gate      (bloquea)       |__ revision::gate       (bloquea)  <- nuevo
  |__ (nada mira el review)                |__ lecciones::gate      (bloquea)
                                                     ^
docs/review-<id>.md                                  |
  lo escribe el reviewer a mano            revision --veredicto <v>
  el veredicto es prosa libre                |__ deriva los AC-n del SPEC
  33 de 40 parseables, 7 no                  |__ exige una fila por AC con cita
  review-3.md:3 dice "approved" y            |__ ESTAMPA la linea canonica
     "cierre BLOQUEADO" en la misma linea    |__ deja rastro en history.md

rules: 7 claves, 3 no las lee nadie      rules: cada clave que esta, gatea
  (require_tests_to_close,                 (las tres muertas, resueltas:
   require_impact_check,                    borradas del template y de las
   one_feature_at_a_time)                   superficies que las anunciaban)

instalador: siembra feature_list.json    instalador: mergea las claves de
  SOLO-SI-FALTA -> una regla nueva          `rules` que falten, sin pisar las
  nunca llega a un proyecto ya              que ya estan, con backup previo
  instalado                                 (sh y ps1, con paridad verificada)
```

## Recorridos de usuario (priorizados)

- P1: Como reviewer, quiero registrar mi veredicto con un comando que deje un
  sello verificable, para que "revisado" signifique algo que se pueda comprobar
  despues y no una frase en un markdown.
- P1: Como Alan, quiero que `close --status done` se niegue cuando no hubo
  revision, para que el estado `done` deje de poder mentir.
- P1: Como Alan, quiero que cada regla de `rules` haga algo, para poder leer el
  bloque y saber que exige el arnes sin ir a grepear el codigo.
- P2: Como dueño de un proyecto que ya tiene el arnes instalado, quiero recibir
  las reglas nuevas al actualizar, para no quedarme con un enforcement viejo sin
  enterarme.
- P2: Como lider, quiero que `roles/leader.md` deje de afirmar "una sola feature
  a la vez", para que la guia no contradiga lo que el arnes hace desde la #47.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una feature con spec aprobado y `require_review` activa, When se
  corre `close --feature <id> --status done` sin que exista
  `docs/review-<id>.md`, Then el cierre se niega con Exit code 2 y un mensaje
  `[GATE]` que nombra la ruta faltante, la regla y el comando exacto del remedio.
  Comando: `cd rust && out=$(cargo test gate_review 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-2: Given un `docs/review-<id>.md` escrito a mano que contiene la frase
  `Veredicto: approved` pero SIN el sello del binario, When se corre `close
  --status done`, Then el cierre se niega igual: el gate lee unicamente la linea
  estampada, y jamas la prosa del archivo.
  Comando: `cd rust && cargo test gate_review_ignora_prosa 2>&1 | grep -E "[1-9][0-9]* passed"`

- AC-3: Given un spec que declara N acceptance criteria (AC-1..AC-N), When se
  corre `revision --feature <id> --veredicto approved` y el review no tiene una
  fila por cada AC-n con su cita `archivo:linea`, Then el comando se niega
  nombrando **cuales** AC quedaron sin fila, y no estampa nada.
  Comando: `cd rust && cargo test veredicto_exige_cobertura_de_ac 2>&1 | grep -E "[1-9][0-9]* passed"`

- AC-4: Given un review completo, When se corre `revision --feature <id>
  --veredicto approved`, Then el binario escribe la linea canonica del veredicto
  en `docs/review-<id>.md`, deja la linea correspondiente en
  `progress/history.md`, y a partir de ahi `close --status done` pasa el gate.
  Comando: `cd rust && cargo test veredicto_estampa_y_habilita_el_cierre 2>&1 | grep -E "[1-9][0-9]* passed"`

- AC-5: Given un veredicto `changes_requested` o `blocked` estampado, When se
  corre `close --status done`, Then el cierre se niega nombrando el veredicto
  encontrado: solo `approved` habilita el cierre.
  Comando: `cd rust && cargo test gate_review_solo_approved 2>&1 | grep -E "[1-9][0-9]* passed"`

- AC-6: Given `require_review` ausente del `feature_list.json` (instalacion
  vieja), When se corre `close --status done`, Then el cierre NO se bloquea: la
  regla nace apagada por defecto, como las otras cuatro.
  Comando: `cd rust && cargo test require_review_default_false 2>&1 | grep -E "[1-9][0-9]* passed"`

- AC-7: Given el repo despues de esta feature, When se busca
  `require_tests_to_close`, `require_impact_check` y `one_feature_at_a_time` en
  `templates/feature_list.json`, Then no aparece ninguna: las tres reglas
  muertas se borraron del molde con el que nace todo proyecto nuevo.
  Comando: `! grep -qE "require_tests_to_close|require_impact_check|one_feature_at_a_time" templates/feature_list.json`

- AC-8: Given un proyecto con el arnes YA instalado y un `feature_list.json` sin
  las claves nuevas, When se re-corre el instalador, Then las claves de `rules`
  que falten se agregan sin pisar los valores existentes, con backup previo del
  archivo.
  Comando: `bash tests/setup_smoke.sh >/dev/null 2>&1`

- AC-9: Given los dos instaladores, When se corre el chequeo de paridad, Then el
  `.ps1` declara la misma migracion que el `.sh`.
  Comando: `bash tests/parity_check.sh`

- AC-10: Given la deuda de 15 cierres sin review (#38-43, #53-55, #57, #59-63),
  When alguien pregunta por que la regla nueva no los alcanza, Then el corte
  esta declarado por escrito en `UPDATING.md` con la lista completa de ids, la
  fecha del corte (ultimo con review: #46, 2026-08-22; primero sin: #57,
  2026-08-26) y el argumento de por que no se reconstruyen.
  Comando: `grep -n "2026-08-22" UPDATING.md`

- AC-11: Given `roles/leader.md` y sus espejos, When se comprueba el espejo de
  roles, Then ningun rol sigue afirmando "una sola feature a la vez" y los
  cuerpos embebidos coinciden con `roles/*.md`.
  Comando: `! grep -rqi "una sola a la vez" roles/ templates/roles/ .claude/agents/ && for r in leader implementer reviewer README; do [ "$(cat roles/$r.md)" = "$(sed "s|__HREL__|harness_process/|g" templates/roles/$r.md)" ] || exit 1; done`
  <!-- No se usa `harness_check.sh` como comando: su gate de espejo expande
       `__HREL__` con el basename del directorio, que DENTRO de un worktree es el
       de la feature y no `harness_process/`, asi que reporta divergencia falsa
       en los tres roles (y `progress/` no existe en el worktree, asi que tambien
       ve `current.md` vacio). Ese comando no puede pasar desde donde se
       implementa, que es lo contrario de un criterio util. El check completo lo
       corre el reviewer desde la raiz. -->

- AC-13: Given un review cuya fila cita un archivo que no existe, una linea que
  el archivo no tiene, o un numero que solo parece una cita (`3.14:15`), When se
  corre `revision --veredicto approved`, Then el comando se niega: el gate
  comprueba que la cita RESUELVA, no solo que tenga la forma de una cita.
  Comando: `cd rust && cargo test la_cita_tiene_que_apuntar_a_algo_que_existe 2>&1 | grep -E "[1-9][0-9]* passed"`

- AC-12 (MANUAL): Given el gate ya compilado, When se le hace la prueba del rojo
  —borrar el sello de un `review-<id>.md` real y correr el cierre— Then el
  cierre pasa a rojo; y al restaurarlo, vuelve a verde. Lo verifica el reviewer.

## Los datos que se tocan

- disparador: `close --feature <id> --status done` (y solo `done`: `blocked`,
  `pending` y `superseded` siguen siendo la valvula de escape, como en los otros
  cuatro gates).
- interruptor: `rules.require_review` en `feature_list.json`. Nace `false` en el
  codigo (`.unwrap_or(false)`), asi que ninguna instalacion existente se rompe;
  nace `true` en `templates/feature_list.json`, asi que un proyecto nuevo nace
  con la etapa viva.
- candado: la linea de veredicto la estampa el binario y es idempotente —
  re-estampar el mismo veredicto no duplica la linea; estampar uno distinto la
  reemplaza y deja las dos transiciones en `progress/history.md`.
- el artefacto: `docs/review-<id>.md`, resuelto en el worktree de la feature
  (viaja en el merge, como el spec y la evidencia).
- lo que NO se toca: los 40 `docs/review-*.md` existentes. El gate corre al
  cerrar, y esas features ya estan cerradas.

## Pseudo-codigo (el acuerdo)

```
CUANDO el reviewer corre `revision --feature <id> --veredicto <v>`

  ¿el spec declara AC-1..AC-N?        -> si no, no hay contra que medir: se niega
  ¿el review tiene una fila por cada AC-n, con cita archivo:linea?
                                      -> si falta alguno, se niega NOMBRANDO cuales
                                         (y no estampa nada: la parte que decide
                                          no es la que escribe)

  ENTONCES estampa la linea canonica del veredicto en docs/review-<id>.md
           y deja el rastro en progress/history.md,
           con la restriccion de que la linea la escribe SOLO el binario.


CUANDO close --status done

  ¿la regla require_review esta activa?   -> si no, no hacemos nada
  ¿el status es done?                     -> si no, no hacemos nada

  ¿existe docs/review-<id>.md?            -> si no, [GATE] con la ruta y el remedio
  ¿tiene la linea ESTAMPADA por el binario? -> si no, [GATE]: la prosa no cuenta
  ¿el veredicto estampado es approved?    -> si no, [GATE] nombrando cual es

  ENTONCES sigue, junto a los otros cuatro gates, ANTES de mutar nada.
```

Promesas: el gate corre en la FASE 0 del cierre (antes de escribir el backlog,
emitir a Jira o tocar el hub) · nunca parsea prosa escrita por un agente · no
compara mtime contra ningun artefacto · no toca los reviews ya existentes · la
regla ausente no bloquea.

**Por que NO se compara la frescura contra `docs/impl-<id>.md`** (estaba en el
acceptance y se descarto al diseñar): `documentos.rs:23-26` ya rechazo esa misma
comparacion por deadlock, y aca el deadlock es el ciclo normal — el reviewer
pide cambios, el implementer corrige, el `impl` queda mas nuevo y el gate
bloquea para siempre; la salida barata seria `touch`, o sea que la regla
entrenaria el `touch`. Ademas no detecta nada: de los 40 pares existentes, cero
tienen el review mas viejo que el impl. La cobertura por AC hace el trabajo que
la frescura pretendia hacer, y se puede fallar.

## No funcionales

- SLOs: el gate es lectura de dos archivos (el spec y el review); no agrega
  latencia observable al cierre. No abre conexion al hub.
- Seguridad: el gate no ejecuta nada de lo que lee. `revision --veredicto`
  escribe unicamente en `docs/review-<id>.md` y `progress/history.md`.
- Observabilidad: cada transicion de veredicto queda en `progress/history.md`,
  que es lo que hace auditable "quien aprobo que y cuando" — hoy no existe ese
  rastro.

## Fuera de alcance

- **Reconstruir los 15 reviews faltantes** (#38-43, #53-55, #57, #59-63).
  Decision del usuario 2026-08-28: se declara el corte. Un review escrito
  despues de que el codigo se integro y funciona no intenta romper nada — solo
  rellena el casillero — y `roles/reviewer.md:6` define el rol como lo
  contrario. El corte queda documentado (AC-10) y la leccion
  `reglas-que-se-aplican-a-si-mismas` se patchea con este caso, porque su paso 2
  ("pagala en la misma feature") no contemplaba una deuda que no se puede pagar
  honestamente.
- Un `check-review` en `harness_check.sh`. El check corre en el hook de fin de
  turno y el review, por definicion, todavia no existe durante la
  implementacion: seria ruido garantizado en cada turno.
- Reescribir los 40 reviews existentes al formato estampado.

## Observaciones (decisiones pendientes)

- **El verbo va en `revision`, no en un comando nuevo.** `revision` ya existe
  (`cli.rs:195-204`) y ya es el paquete de revision; agregarle `--veredicto` no
  suma superficie. `Peldano elegido:` extender un subcomando existente, porque
  crear `review` habria agregado un comando nuevo (peldano mas bajo) para el
  mismo resultado. **Confirmar al aprobar**: el preview de la decision decia
  `harness_cli review --veredicto`; si preferis el comando propio, se cambia
  antes de implementar.
- **Migracion de `rules`: contrato nuevo del instalador.** Hoy los dos
  instaladores siembran `feature_list.json` solo-si-falta, por decision escrita
  (`setup_harness.sh:2690-2691`), y el riesgo esta declarado en
  `docs/architecture.md:489-490`. La migracion cambia ese contrato: pasa a
  tocar un archivo del usuario. Se hace con backup previo y sin pisar valores
  existentes, pero es el punto mas delicado de la feature y merece tu ojo en el
  review.
- **`one_feature_at_a_time` arrastra los roles.** Borrarla implica tocar
  `roles/leader.md:97` + `templates/roles/leader.md` + `.claude/agents/leader.md`
  (y los espejos de Gemini/Codex/Kimi los regenera el instalador). El gate de
  espejo de `harness_check.sh:222-238` reporta divergencia falsa dentro de un
  worktree porque usa el basename como `__HREL__`: no obedecer ese mensaje aca.
