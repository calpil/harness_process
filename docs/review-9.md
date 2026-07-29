# Review - Feature #9: codex_roles_can_write_artifacts

Spec: docs/spec-feature-9-codex-roles-can-write-artifacts.md (Estado: approved, sellado)
Plan: docs/plan-feature-9-codex-roles-can-write-artifacts.md
Impl: docs/impl-9.md

## Veredicto global

**approved**, con una salvedad y cinco hallazgos (ninguno bloquea):

1. AC-6 (paridad Windows) queda **parcial**: revision estatica real, sin
   ejecucion, porque no hay `pwsh` ni `powershell` en esta maquina (verificado
   con `command -v`, rc=1). Misma limitacion aceptada en #1, #4, #5, #6, #7 y #8.
2. **La premisa se sostiene**: la re-ejecute yo con `codex sandbox` sobre
   codex-cli 0.145.0 real y reproduce el spec al pie de la letra (detalle abajo).
   Tambien re-verifique el punto 2 del "Terreno verificado" (`writable_roots`) y
   los valores validos de `sandbox_mode`, que Codex mismo enumera.
3. **La afirmacion sobre Claude es correcta y la estoy demostrando al escribir
   este archivo**: corro con `tools: Read, Grep, Glob, Bash` (sin `Edit` ni
   `Write`) y este veredicto se escribe con `Bash`. Ver "La premisa" abajo.
4. Precisiones sobre el impl (Hallazgos 1 y 2): (a) **la imprecision
   `read-only` que el impl retiro de `roles/README.md` sobrevive en TRES lugares
   mas**, uno de ellos en el mismo `setup_harness.sh` a 12 lineas del cambio, y
   otro dentro del `.codex/agents/reviewer.toml` GENERADO; (b) el techo real de
   `workspace-write` no es "el workspace" como dicen spec y plan — tambien
   escribe `/tmp` y `$TMPDIR` — pero a cambio `$HOME`, las rutas absolutas
   fuera del workspace, la RED y **el propio `.git/` del workspace** estan
   denegados, lo que refuerza la decision mucho mas de lo que el spec argumenta.
5. Deuda menor: la paridad de TESTS quedo rota (`tests/setup_smoke.ps1` sin
   assert equivalente) y el checkpoint de `graphify query` no se cumplio
   habiendo grafo fresco (Hallazgos 3 y 4).

El arbol queda **sin commit a proposito** (5 modificados + 3 untracked de la
feature). Nada de lo verificado contradice la decision del usuario del
2026-07-28 (Articulo 5), y **no acepte ninguna afirmacion de `docs/impl-9.md`
sin re-ejecutarla o comprobarla contra el arbol real**.

Nota de proceso: `docs/impl-9.md` declara que la feature la implemento el LIDER
de la sesion y no un implementer delegado. Lo tuve presente para NO ablandar el
listado: re-hice la prueba negativa del smoke con fixtures propias, mute el
instalador para comprobar que el test falla cuando debe, y reconstrui dos
instalaciones completas (HEAD y working) para diferenciar artefacto por
artefacto.

## Aprobacion del spec

```
Estado: approved
Aprobado: 2026-07-29T01:16:57Z por USUARIO (confirmacion explicita) - Alan
aprobo el spec #9 en el chat (2026-07-28): workspace-write para los tres roles
de Codex, decidido tras ver la evidencia del sandbox
```

Rastro en `progress/history.md`: `2026-07-29T01:16:57Z approve-spec feature #9
estado=approved nota=...` (misma marca temporal que el sello).
`sh harness_cli check-spec` rc=0 (`[OK] Spec aprobado y fresco`) y `check-plan`
rc=0, ambos re-corridos en esta revision.

**Cronologia verificada por mtimes** (TZ local `-0400`; sello 01:16:57Z =
21:16:57 local): spec 21:16:57 -> `setup_harness.sh` 21:17:41 (+44s) ->
`roles/README.md` 21:18:08 -> `templates/roles/README.md` 21:18:28 ->
`tests/setup_smoke.sh` 21:18:41 -> `setup_harness.ps1` 21:18:50 -> advance
21:19:33 -> `docs/impl-9.md` 21:21:28. **Todo archivo de implementacion es
posterior al sello**: Articulo 2 cumplido con evidencia dura, no declarativa.
Medi ademas que `bash tests/setup_smoke.sh` tarda **24s**, asi que la ventana
21:18:50 -> 21:21:28 alcanza de sobra para los tres comandos oficiales que el
impl declara haber corrido: no hay anomalia de plausibilidad.

## La premisa (re-ejecutada por mi, codex-cli 0.145.0)

Todo esto lo corri yo en directorios temporales del scratchpad, nunca sobre
este checkout. `codex sandbox` corre bajo seatbelt sin invocar al LLM.

```
$ codex sandbox -c sandbox_mode='"read-only"' -- sh -c 'echo "# veredicto" > review-9.md'
sh: review-9.md: Operation not permitted            rc=1  -> el archivo NO se crea
$ codex sandbox -c sandbox_mode='"workspace-write"' -- sh -c 'echo "# veredicto" > review-9.md'
                                                    rc=0  -> el archivo se crea (12 bytes)
```

**Punto 2 del "Terreno verificado" (no hay via intermedia), re-verificado:**

```
$ codex sandbox -c sandbox_mode='"workspace-write"' \
    -c sandbox_workspace_write.writable_roots='["<dir>/docs"]' \
    -- sh -c 'echo "// PWNED" >> src/main.rs'      rc=0  -> ESCRIBIO en src/ igual
$ codex sandbox -c sandbox_mode='"read-only"' \
    -c sandbox_workspace_write.writable_roots='["<dir>/docs"]' \
    -- sh -c 'echo x > docs/a.md'                  rc=1  Operation not permitted
```

Confirmado y con un matiz que el spec no registra: `writable_roots` no solo NO
restringe dentro del workspace, ademas es **inerte bajo `read-only`**. No hay
combinacion "solo docs/": la dicotomia es exactamente la que describe el spec.

**Valores validos, confirmados por el propio binario:**

```
$ codex sandbox -c sandbox_mode='"bogus-mode"' -- sh -c 'true'
Error: unknown variant `bogus-mode`, expected one of `read-only`,
`workspace-write`, `danger-full-access` in `sandbox_mode`
```

**La afirmacion sobre Claude, comprobada en la configuracion real y en vivo:**
`setup_harness.sh:2101` y `:2103` generan leader y reviewer con
`"Read, Grep, Glob, Bash"` (el implementer, `"Read, Edit, Write, Bash, Grep,
Glob"`); `.claude/agents/leader.md` y `.claude/agents/reviewer.md` de este
checkout llevan `tools: Read, Grep, Glob, Bash`. `setup_harness.ps1:684-689`
hace lo mismo. Y la evidencia definitiva: **yo soy ese reviewer de Claude, no
tengo herramienta `Write` ni `Edit`, y este archivo lo estoy escribiendo con
`Bash`** — igual que `docs/review-7.md` (13437 bytes) y `docs/review-8.md`
(18951 bytes), que existen en el repo. La asimetria que corrige la feature era
real.

**`approval_policy`**: grep en todo el repo -> no aparece en ningun archivo de
codigo ni de configuracion (solo en los docs de esta feature). La afirmacion
del spec es exacta.

## Que permite realmente `workspace-write` (medido, no citado)

Ni el spec ni el impl caracterizan el limite; lo medi yo. Desde un workspace
git en el scratchpad, con `sandbox_mode = "workspace-write"`:

| Operacion | Resultado |
| --- | --- |
| escribir en el workspace | PERMITIDO |
| escribir en `/tmp` y en `$TMPDIR` | **PERMITIDO** (spec y plan dicen "el limite superior sigue siendo el workspace": impreciso) |
| escribir en `$HOME` (`~/HARNESS-REVIEW9-PROBE.txt`) | DENEGADO, `Operation not permitted` |
| escribir en ruta absoluta fuera (`/Users/Shared/...`) | DENEGADO |
| escribir en `.git/` **del propio workspace** | **DENEGADO** |
| red (`curl https://example.com`) | DENEGADO (`Could not resolve host`) |
| leer cualquier ruta (`$HOME/.zshrc`) | permitido (igual que bajo `read-only`) |

Lectura para el veredicto: la frase "el limite superior sigue siendo el
workspace" del spec es **imprecisa por exceso de optimismo** (`/tmp` y
`$TMPDIR` tambien), pero el balance real es **mas favorable** a la decision de
lo que el propio spec argumenta: un leader o un reviewer descarriado en Codex
no puede tocar el home del usuario, no puede reescribir la historia de git y no
tiene red. Todas las sondas fueron limpiadas al terminar.

## Estado por AC

Verificado en esta sesion contra el arbol real y en fixtures PROPIAS del
scratchpad (patron `copy_fixture` del smoke: `setup_harness.sh` + `templates/`
+ binario Rust precompilado; `HOME`, `HARNESS_HUB` y `KIMI_CODE_HOME` de
fixture). **Nunca corri `setup_harness.sh` sobre este checkout.**

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | Instalacion propia (`--root`, rc=0): los TRES `.codex/agents/*.toml` con `sandbox_mode = "workspace-write"`, parseados con `tomllib` (rc=0, `keys = [description, developer_instructions, model_reasoning_effort, name, sandbox_mode]` en los tres). En el codigo: `setup_harness.sh:2106-2108` -> `build_codex_agent {leader,implementer,reviewer} workspace-write high`. Diff HEAD vs working de `.codex/agents/`: **exactamente 2 lineas**, `sandbox_mode` de `leader.toml` y de `reviewer.toml` |
| AC-2 | cubierto | `developer_instructions` **byte a byte identico** a `roles/<rol>.md` en los tres (`exact=True`; longitudes 3473 / 4158 / 2169 == longitud del fuente). `name`, `description` y `model_reasoning_effort = "high"` sin cambios (mismo set de claves, mismos valores, verificado sobre el TOML parseado). Prueba mas fuerte: **diff recursivo de DOS instalaciones completas** (una desde `git archive HEAD`, otra desde el working tree) -> las unicas diferencias en todo el arbol son las 2 lineas de `sandbox_mode`, `roles/README.md`, `setup_harness.sh` y, modulo el nombre del directorio de fixture, `.codex/hooks.json`/`.gemini/settings.json`/`.gitignore` (verificado normalizando la ruta: identicos). El generador `build_codex_agent` no se toco en su cuerpo. **Gate de espejo corrido DENTRO de la fixture instalada: rc=0, `[Ok] Harness Check limpio.`** — no reporta nada |
| AC-3 | cubierto | Comentario de cabecera `setup_harness.sh:2028-2043`, leido completo. Contiene los cinco puntos: (a) el formato NO admite allowlist, citando la doc ("subagents use the tools available to the parent chat"); (b) por que los TRES usan `workspace-write` (leader y reviewer deben escribir en `docs/`, `read-only` responde `Operation not permitted`), con la version verificada (codex-cli 0.145.0); (c) que no es mas laxo que Claude, "donde Bash ya permite escribir" y la disciplina la pone el prompt de `roles/*.md`; (d) que `writable_roots` solo AÑADE rutas y no restringe las de dentro; (e) que `danger-full-access` no se usa nunca. Los puntos (d) y los valores validos los **re-verifique empiricamente** (arriba). `danger-full-access`: grep en el repo -> solo aparece en ese comentario, jamas como valor |
| AC-4 | cubierto (con residuos: Hallazgo 1) | `roles/README.md:86-91` describe el estado nuevo de Codex, y el parrafo nuevo `:100-106` no miente sobre Claude — contrastado contra `setup_harness.sh:2101/2103`, `setup_harness.ps1:684-689` y los `.claude/agents/*.md` reales. **Equivalencia modulo `__HREL__` demostrada formalmente**: `diff <(sed 's|__HREL__|harness_process/|g' templates/roles/README.md) roles/README.md` -> **rc=0, sin salida**; el sub-gate de la #7 pasa (`harness_check.sh` rc=0). Ademas verifique que el `roles/README.md` INSTALADO en fixture root == `templates/roles/README.md` con `__HREL__` vacio (diff limpio). Barri `README.md`, `AGENTS.md`, `UPDATING.md`, `docs/architecture.md` y `templates/`: sin afirmaciones obsoletas sobre el sandbox de Codex. **Pero la imprecision `read-only` sigue escrita en otros 3 sitios: Hallazgo 1** |
| AC-5 | cubierto | El assert vive en `tests/setup_smoke.sh:160-176`, dentro del bloque `ROOT_LAYOUT` que **si se instala y corre** (`run_setup "$ROOT_LAYOUT" --root`, linea 143), bajo `set -Eeuo pipefail`. Opera sobre el TOML **parseado**, no por grep. **Prueba negativa propia** (6 fixtures sinteticas mias, no las del impl): tres `workspace-write` -> rc=0; `reviewer` en `read-only` -> rc=1 `[FALLO] sandbox_mode != workspace-write en Codex: {'reviewer': 'read-only'}`; falta `leader.toml` -> rc=1 `[FALLO] faltan agentes Codex: ['leader']`; **clave `sandbox_mode` ausente** -> rc=1 `{'leader': None}`; directorio vacio -> rc=1 con los tres roles; **`danger-full-access`** -> rc=1. Y la prueba que el impl no hizo: **mute `setup_harness.sh` (reviewer de vuelta a `read-only`) en una copia completa del repo y corri el smoke ENTERO -> rc=1** con el `[FALLO]` exacto. El test falla cuando debe |
| AC-6 | **parcial** | Sin `pwsh` ni `powershell` (`command -v` rc=1). Revision estatica REAL y propia: (1) leido `setup_harness.ps1:703-718` — el cambio es una asignacion literal `$sandbox = "workspace-write"` dentro del mismo `foreach ($role ...)` preexistente, con el mismo comentario justificatorio que el `.sh`, y el here-string `$codex` lo interpola en `sandbox_mode = "$sandbox"`: produce el mismo valor para los tres roles que el `.sh`; (2) **tokenizador con estados propio** (here-strings `@"`/`@'`, comillas simples y dobles, escapes con backtick, comentarios de linea y de bloque `<# #>`): `setup_harness.ps1` working -> here-strings **13/13 pareados**, balance de llaves **0**, cero strings sin cerrar, cero here-strings abiertos al EOF; **identico en HEAD** (13/13, 0). Idem `tests/setup_smoke.ps1`: 6/6 y 0 en ambos. **Los here-strings que arreglo la #7 no se rompieron**; `tests/setup_smoke.ps1` ni siquiera fue tocado (`git status` vacio). SIN ejecucion |
| AC-7 | cubierto | Re-ejecutados por mi, con los comandos oficiales de `docs/verification.md`: `cargo test --locked` -> **44 unit + 22 integracion, 0 fallos**; `cargo clippy --all-targets --all-features --locked -- -D warnings` -> **rc=0** (y tambien rc=0 con la forma corta `--all-targets -- -D warnings`); `bash tests/setup_smoke.sh` -> **rc=0 en 24s**. `rust/` con **0 cambios** (`git status --porcelain rust/` vacio): cero dependencias nuevas |

## No regresion multi-LLM (verificada por diferencia de artefactos)

No me base en las lineas `[Ok]` del smoke: construi **dos instalaciones
completas** (HEAD y working tree) con identicos flags y difere los arboles.

- `.claude/agents/` -> **identicos** (diff recursivo limpio). Cubre tambien
  **Grok**, que lee esos mismos archivos.
- `.gemini/agents/` -> **identicos**.
- `.kimi-code/agents/` -> **identicos**.
- `.codex/agents/` -> solo las 2 lineas de `sandbox_mode`; `developer_instructions`,
  `name`, `description` y `model_reasoning_effort` intactos.
- `bin/harness-antigravity`, `bin/harness-*`, `AGENTS.md`, hooks, superficies,
  `CHECKPOINTS.md`, `docs/` sembrados -> **identicos** (el diff recursivo del
  arbol completo no los lista).
- `.codex/hooks.json`, `.gemini/settings.json` y `.gitignore` -> identicos tras
  normalizar el nombre del directorio de fixture (unica diferencia: la ruta
  absoluta de la propia fixture).

Y las **10 lineas `[Ok]` de la corrida del smoke** estan todas presentes
(gate de espejo, checkout fuente, Kimi Code, docs en la raiz, planillas PRD,
gate de spec, approve-spec, reset, binario Rust, smoke general). Cero regresion.

## Trazabilidad y constitution

- **Articulo 1**: test cercano al cambio (`tests/setup_smoke.sh:160-176`, sobre
  fixture instalada real) y los tres comandos oficiales en verde,
  re-ejecutados por mi. El test ademas **demostro que falla ante la regresion
  real** (mutacion del instalador + smoke completo rc=1).
- **Articulo 2**: spec `approved`, sellado por el USUARIO, con rastro en
  `history.md` y fresco (`check-spec` rc=0). Implementacion **posterior al
  sello**, probado por mtimes (+44s el primer archivo). Ningun agente aprobo
  nada.
- **Articulo 3**: la Delegacion U1..U4 del plan cita sus AC (`[AC-1, AC-2,
  AC-3]`, `[AC-4]`, `[AC-5, AC-7]`, `[AC-6]`); `docs/impl-9.md` mapea
  AC-1..AC-7 uno a uno; este veredicto lista los 7.
- **Articulo 4**: sin secretos (grep de las fixtures y del diff: solo
  credenciales dummy de fixture, ninguna en el repo). Sin cambios de exit codes
  ni de mensajes del instalador (verificado por diff de los dos logs de
  instalacion). El assert nuevo emite errores accionables (`[FALLO] ...` con el
  rol y el valor encontrado). **Ampliacion de permisos deliberada**: medida y
  acotada arriba; `danger-full-access` no se usa.
- **Articulo 5**: la unica decision (workspace-write para los tres roles) la
  tomo el usuario el 2026-07-28 y esta registrada en Observaciones del spec
  (`:144-148`) y del plan (`:102-105`), con las dos alternativas descartadas y
  su motivo. La implementacion la respeta al pie de la letra y **no invento
  ninguna decision nueva**: revise el diff completo linea por linea. El impl
  declara "OBSERVACION SIN DECISION: ninguna" y lo confirmo.
- **Articulo 6**: `rust/Cargo.toml` sin cambios (cero dependencias nuevas).
  Espejo raiz/`templates/` correcto: de los 4 archivos tocados, **el unico con
  espejo obligatorio es `roles/README.md`** (no existen `templates/setup_harness.sh`,
  `templates/setup_harness.ps1` ni `templates/tests/`), y su par
  `templates/roles/README.md` viaja en el mismo cambio y es **equivalente modulo
  `__HREL__` demostrado formalmente**. Feature backend-agnostica: toca
  exclusivamente la superficie de Codex y no altera la de ningun otro backend
  (probado por diferencia de artefactos). Commits: arbol sin commitear; el
  mensaje debera ser Conventional y SIN trailers de IA (`commit_guard.sh`).

## Impacto y gates

- `sh harness_cli graph impacto --microservicio ADR/harness` ejecutado en esta
  revision: **"Ningun microservicio registrado depende de 'ADR/harness'"**
  (rc=0). Impacto externo nulo, como declara el plan.
- Radio interno = **exactamente** el declarado en el plan, verificado con
  `git diff --stat`: `roles/README.md` (+19/-4), `setup_harness.ps1` (+7/-1),
  `setup_harness.sh` (+20/-3), `templates/roles/README.md` (+19/-4),
  `tests/setup_smoke.sh` (+17/-0). Nada mas. `rust/`, `harness_cli`,
  `harness_check.sh`, hooks y superficies: intactos.
- **`graphify query` NO se ejecuto** habiendo grafo (Hallazgo 4). Lo consulte
  yo como sustituto: `graphify-out/graph.json` (1113 nodos, refrescado el mismo
  dia 21:19, ya con los nodos del plan #9) contiene
  `setup_harness_build_codex_agent`, `harness_check_extract_codex_body` (+ su
  espejo de `templates/`) y `tests_setup_smoke_get_codexbody`. Es decir: la
  consulta habria devuelto el radio del plan MAS los tres consumidores del gate
  de espejo — todos comparadores de CUERPO, no de `sandbox_mode`, y verifique
  que ninguno se ve afectado (gate rc=0 en fixture y en el checkout).
- `bash harness_check.sh` **rc=0**, `[Ok] Harness Check limpio.`, con
  `[plan] #9 fresco` y `[spec] #9 approved (fresco)`.
- Plan archivado en `docs/` de la raiz y fiel a lo implementado (las 4 unidades
  U1..U4 corresponden 1:1 con el diff).
- Checkpoints de `CHECKPOINTS.md`: cubiertos salvo el de `graphify query`
  (Hallazgo 4) y un detalle cosmetico de `progress/current.md` (Hallazgo 5).
- **Coherencia del commit unico pendiente**: 5 modificados + 3 untracked
  (`docs/spec-feature-9-*.md`, `docs/plan-feature-9-*.md`, `docs/impl-9.md`) +
  este review. El par espejo `roles/README.md` / `templates/roles/README.md`
  debe viajar junto (Articulo 6).
- `validate_ui.sh`: no aplica (sin frontend).

## Hallazgos de esta revision (no bloquean)

1. **La imprecision `read-only` que el impl retiro de `roles/README.md`
   sobrevive en TRES lugares.** El propio impl pidio verificarlo ("Punto de
   atencion sugerido"); la busqueda no se hizo o quedo incompleta:

   a. **`setup_harness.sh:2115-2116`** — comentario del bloque Kimi:
      `# --- Kimi Code CLI: .kimi-code/agents/*.md (allowlist de tools por rol:`
      `# leader/reviewer read-only; implementer ademas Edit/Write) ---`.
      Esta a **12 lineas** del cambio de la #9, en el mismo archivo que la
      feature edito, y es exactamente el rotulo que se retiro de la entrada de
      Kimi en `roles/README.md`.
   b. **`tests/setup_smoke.ps1:379`** — mensaje de assert:
      `"Kimi leader allowlist must be read-only (Read, Grep, Glob, Bash)."`.
   c. **`setup_harness.sh:2098` (`desc_rev`)** — el mas visible de todos:
      `...escribe veredicto en docs/ de la raiz. Solo lectura; no implementa.`
      Ese string se emite como `description` en los CUATRO backends. En el
      artefacto generado por esta feature queda asi:

      ```toml
      description = "... escribe veredicto en docs/ de la raiz. Solo lectura; no implementa."
      sandbox_mode = "workspace-write"
      ```

      Dos lineas consecutivas que se contradicen, dentro del archivo que Codex
      lee. Es precisamente el riesgo "falsa sensacion de simetria" que el plan
      queria conjurar, en su version inversa. **No lo considero incumplimiento**:
      AC-2 congela `description` explicitamente, asi que corregirlo exigiria
      enmienda del spec o feature aparte (queda en Pendientes). Pero el impl
      debio detectarlo y no lo hizo, habiendo pedido justamente esa busqueda.
      (El `.ps1` NO arrastra esta contradiccion: sus `descriptions` estan en
      ingles y la del reviewer no lleva la clausula — divergencia preexistente
      de paridad sh/ps1, ajena a la #9.)

2. **El techo de `workspace-write` esta mal caracterizado en spec y plan.**
   "El limite superior sigue siendo el workspace" es impreciso: medido,
   `/tmp` y `$TMPDIR` tambien son escribibles. En sentido contrario, spec e
   impl **omiten tres protecciones reales que fortalecen la decision**:
   `$HOME` denegado, red denegada y **`.git/` del propio workspace denegado**.
   La tabla completa esta arriba. Neto: la decision del usuario es mas
   defendible de lo que el spec argumenta, pero el enunciado del no-funcional
   de Seguridad conviene corregirlo si el spec se reedita.

3. **Paridad de TESTS rota (no de instalador).** `tests/setup_smoke.ps1` no fue
   tocado y **no tiene assert de `sandbox_mode`** (solo compara el cuerpo del
   `.toml` contra `roles/<rol>.md`, lineas 292-293). El instalador `.ps1` si
   esta en paridad. Consecuencia: si alguien revierte solo `setup_harness.ps1` a
   un sandbox por rol, ningun test lo detectara ni siquiera cuando haya `pwsh`.
   La #8 sembro asserts en AMBOS smokes; aqui se sembro en uno. AC-5 solo nombra
   `tests/setup_smoke.sh`, asi que **no incumple el spec**: es deuda.

4. **Checkpoint de `graphify` no cumplido.** `CHECKPOINTS.md` dice "Si existe
   `graphify-out/graph.json`, se consulto `graphify query`". El archivo EXISTE
   (1113 nodos) y ademas estaba **fresco** (refrescado a las 21:19 del mismo
   dia por el `advance`, con los nodos del plan #9 ya dentro, sin flag
   `.graphify_stale`). El plan declara "No se consulto el grafo" con una
   justificacion de metodo ("se leyeron los generadores"), que no es la salida
   sancionada por el checkpoint ("justificacion si no hay grafo"). La #8 si lo
   consulto. **Sin consecuencia material**: lo consulte yo (ver Impacto) y el
   radio que habria devuelto es el que el plan ya declara mas los consumidores
   del gate de espejo, verificados como no afectados.

5. **Detalles cosmeticos de proceso.** (a) `progress/current.md` arrastra un
   bullet de evidencia **vacio** (`- `) heredado del `start` y nunca rellenado;
   la feature registro **un solo `advance`**, posterior a la implementacion
   (el propio impl lo declara con transparencia: por eso `check-plan` estuvo
   temporalmente en rojo). Ambos gates estan verdes ahora. (b)
   `roles/README.md:52-53` conserva "No hay allowlist de tools: la capacidad se
   acota con `sandbox_mode`": sigue siendo cierto como enunciado, pero ya no
   diferencia nada entre roles. (c) El parrafo nuevo "Sobre solo lectura" cubre
   Claude, Kimi y Codex, y omite que en **Gemini** leader y reviewer heredan
   las tools de la sesion (incluidas las de escritura) y que **Grok** lee la
   config de Claude; fuera de la letra de AC-4, pero completaria el cuadro.

## Pendientes (decisiones del usuario, sin decidir aqui)

1. **AC-6 sin ejecucion real en Windows**: correr `pwsh tests/setup_smoke.ps1`
   en la primera maquina Windows disponible salda la deuda acumulada de
   #1/#4/#5/#6/#7/#8/#9.
2. **Residuos de "read-only" (Hallazgo 1)**: decidir si se corrigen los dos
   comentarios/mensajes (a, b) —cambio trivial de texto— y, sobre todo, si se
   enmienda `desc_rev` (c) para que la `description` del reviewer no contradiga
   su propio `sandbox_mode`. Esto ultimo toca `description`, que AC-2 congela:
   requiere spec nuevo o enmienda explicita.
3. **Assert de `sandbox_mode` en `tests/setup_smoke.ps1`** (Hallazgo 3), para
   cerrar la paridad de tests que la #8 si mantenia.
4. **Precision del no-funcional de Seguridad del spec** (Hallazgo 2), si se
   reedita: `/tmp` y `$TMPDIR` tambien son escribibles; `$HOME`, la red y
   `.git/` no lo son.
5. **Commit unico**: pendiente por decision del usuario; mensaje Conventional
   SIN trailers de IA (politica del repo).
6. **Dependencia de la version de Codex** (riesgo ya anotado en el plan): si
   una version futura de codex-cli introduce allowlist de herramientas para
   subagentes, conviene revisar esta decision.
