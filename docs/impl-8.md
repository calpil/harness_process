# Impl - Feature #8: kimi_cli_backend

Spec: docs/spec-feature-8-kimi-cli-backend.md (Estado: approved, sellado 2026-07-28T23:19:20Z)
Plan: docs/plan-feature-8-kimi-cli-backend.md

## Que cambio

Kimi Code CLI (v0.29.2, verificado empiricamente por el lider y re-verificado
aqui) queda como backend de primera clase del arnes, con las 3 decisiones del
usuario (2026-07-28) aplicadas al pie de la letra:

```
Proyecto (siempre, como los demas backends):
  .kimi-code/agents/{leader,implementer,reviewer}.md   espejos desde roles/
     frontmatter name/description/tools (allowlist por rol, decision 3)
     cuerpo verbatim de roles/<rol>.md con __HREL__ sustituido
  bin/harness-kimi                                     launcher (mismo esqueleto)
  AGENTS.md / superficies                              ya lo lee nativo; textos multi-backend actualizados

Global (UNICA excepcion de $HOME, decision usuario; solo si Kimi detectado,
--no-kimi la excluye — decision 2):
  ${KIMI_CODE_HOME:-$HOME/.kimi-code}/config.toml
     bloque delimitado "# >>> harness-process hooks >>>" ... "# <<< ... <<<"
     [[hooks]] SessionStart(120) / PostToolUse(matcher Edit|Write, 30) / Stop(120)
     command = guard: [ -x "$PWD/bin/harness-hook" ] || exit 0;
               HARNESS_REPO_ROOT="$PWD" exec "$PWD/bin/harness-hook" plain <evento>
     backup previo en bkp/ + reemplazo idempotente SOLO entre marcadores +
     validacion `kimi doctor` con rollback (best-effort, exit del setup intacto)
  --reset NO lo toca (decision 1); remocion manual documentada en UPDATING.md

Gate de espejo (feature #7) extendido: harness_check.sh compara ademas
.kimi-code/agents/<rol>.md (extract_agent_body, mismo formato que Claude) y
chequea su estructura; ausencia no falla. Espejo raiz==templates por diff.
```

Sin cambios en `rust/` (cero dependencias nuevas), `harness_cli`, `init.sh`,
`commit_guard.sh`, `harness_status.sh` ni `bin/harness-hook` (sus case patterns
ya aceptan los eventos de Kimi, verificado por el lider).

## Unidades

| Unidad | AC | Archivos |
| --- | --- | --- |
| U0 (lider) | - | 3 decisiones del usuario registradas en spec + plan; cerrada antes de esta implementacion |
| U1 agentes + launcher | AC-1, AC-2, AC-8(proyecto) | `setup_harness.sh` (`build_kimi_agent`, invocaciones con allowlist, `write_launchers`, `backup_file` x4, `do_mkdir`, `reset_targets`, salida final) |
| U2 hooks globales | AC-3, AC-4, AC-5, AC-8(global) | `setup_harness.sh` (`write_kimi_hooks`, `KIMI_HOOKS_BEGIN/END`, flag `--no-kimi`, usage, `KIMI_HOOKS_WRITTEN` en reporte final) |
| U3 gate de espejo | AC-6 | `harness_check.sh` + `templates/harness_check.sh` (identicos por diff) |
| U4 superficies + mapa | AC-7 | `setup_harness.sh` (heredocs `write_agent_surface` / `write_basic_agent_surface`), `roles/README.md` + `templates/roles/README.md` (modulo `__HREL__`) |
| U5 smoke sh | AC-9 | `tests/setup_smoke.sh` (export `KIMI_CODE_HOME` de aislamiento + assert de backup en CUSTOM_BKP + bloque Feature #8 completo) |
| U6 paridad ps1 | AC-10 (parcial) | `setup_harness.ps1` (`-NoKimi`, espejos Kimi en `Write-AgentDefinitions`, `Write-KimiGlobalHooks`, launcher, reset, backups, dirs), `tests/setup_smoke.ps1` (aislamiento `$env:KIMI_CODE_HOME` + bloque Feature #8) |
| U7 docs + verificacion | AC-11, AC-12 | `README.md`, `UPDATING.md` + `templates/UPDATING.md`, `docs/architecture.md`, `AGENTS.md`; corridas finales |

## Evidencia por AC

- **AC-1** (espejos `.kimi-code/agents/*.md` con patron `build_claude_agent`;
  condicionales a subagentes): `build_kimi_agent` en `setup_harness.sh`
  (frontmatter `---` / `name:` = rol / `description:` = `desc_*` compartida /
  `tools:` segun decision 3 / `---` / linea en blanco / cuerpo verbatim de
  `roles/<rol>.md` via `cat` + `subst_hrel_inplace`). Invocada SOLO dentro de
  `if [ "$WITH_SUBAGENTS" -eq 1 ]`, con allowlist: leader/reviewer
  `Read, Grep, Glob, Bash`; implementer `Read, Edit, Write, Bash, Grep, Glob`
  (= read-only + `Edit, Write`; nombres case-sensitive verificados en v0.29.2).
  Smoke (a): en fixtures root Y subdir, los 3 espejos existen, `head -1 = ---`,
  `name:`/`description:` presentes, `tools:` exactos por rol, y cuerpo
  extraido == `roles/<rol>.md` (comparacion con el mismo awk del gate). Con
  `--no-subagents`: `test ! -d "$NO_SUBAGENTS/.kimi-code"` (ni el dir se crea).
- **AC-2** (launcher + backups + resumen final): `kimi` agregado al bucle de
  `write_launchers` (mismo esqueleto `LAUNCHER_EOF`; smoke: `test -x
  bin/harness-kimi` + `grep 'AGENT="kimi"'`), `backup_file` de los 3 espejos y
  del launcher junto a los demas (smoke: `*.kimi-code/agents/leader.md.bak.*`
  encontrado en `CUSTOM_BKP` tras el reinstall subdir), y la salida final del
  instalador lista superficie/hooks/launcher Kimi (`superficies/hooks: ... Kimi
  ...`, linea condicional del config global segun `KIMI_HOOKS_WRITTEN`, launcher
  en ambas listas, subagentes nativos con `.kimi-code/agents/*.md`).
- **AC-3** (bloque global delimitado, exactamente 3 eventos, guard generico,
  backup previo): `write_kimi_hooks` resuelve `${KIMI_CODE_HOME:-$HOME/.kimi-code}`,
  hace `backup_file "$kimi_cfg"` ANTES de tocar (mas copia local para rollback),
  y escribe el bloque entre `# >>> harness-process hooks >>>` y
  `# <<< harness-process hooks <<<` con `[[hooks]]` para `SessionStart` (120),
  `PostToolUse` (`matcher = "Edit|Write"`, 30) y `Stop` (120) — sin `SessionEnd`
  ni `UserPromptSubmit`. Cada `command`:
  `[ -x "$PWD/bin/harness-hook" ] || exit 0; HARNESS_REPO_ROOT="$PWD" exec
  "$PWD/bin/harness-hook" plain <evento>`. Deteccion segun decision 2
  (`command -v kimi` o `$kimi_home/bin/kimi` ejecutable) + flag `--no-kimi`.
  Smoke (c-1): config inexistente -> archivo creado con 1 marcador de apertura,
  1 de cierre, exactamente 3 `[[hooks]]`, los 3 eventos, el matcher, los 3
  despachos `plain`, `HARNESS_REPO_ROOT` presente, cero `SessionEnd|
  UserPromptSubmit`, y TOML valido por `tomllib`. **Verificacion empirica
  extra**: el bloque generado, tal cual, pasado al `kimi doctor` REAL v0.29.2 ->
  "All checked config files are valid" (rc=0).
- **AC-4** (contenido del usuario intacto byte a byte, sin duplicar): el
  reemplazo remueve con awk SOLO las lineas entre marcadores (inclusive) y
  anexa el bloque fresco al final. Smoke (c-2): config de fixture con sentinel
  + `[[hooks]]` PROPIO del usuario (`UserPromptSubmit` / `echo
  hook-del-usuario`) -> tras instalar 2 veces: sentinel y hook del usuario
  presentes, 1 solo bloque (conteo de marcadores), 4 `[[hooks]]` totales
  (1 usuario + 3 arnes), y `cmp` byte a byte del config original contra el
  resultado con el bloque removido: identicos.
- **AC-5** (creacion desde cero + `kimi doctor` best-effort + rollback sin
  cambiar exit): sin `~/.kimi-code/` la funcion hace `mkdir -p` y crea el
  archivo solo-bloque (smoke c-1). Validacion: `KIMI_CODE_HOME="$kimi_home"
  "$kimi_bin" doctor`; si falla, restaura la copia previa (o `rm` del archivo
  recien creado), avisa con `log_warn` accionable (ruta + backup + re-correr) y
  `return 0`. **Base empirica de esta sesion (v0.29.2 real)**: doctor rc=0 con
  config valido AUNQUE falte login/`default_model` (caso del config casi vacio
  de esta maquina) y rc=1 solo con TOML invalido, asi que el exit code es senal
  fiable; ademas doctor no escribe nada en `KIMI_CODE_HOME` (fixture vacia
  quedo vacia). Smoke: rama con kimi falso `exit 1` -> setup rc=0, config
  restaurado por `cmp`, aviso `doctor' reporto config invalido` en el log; y
  sin config previo -> el archivo recien creado se retira.
- **AC-6** (gate de espejo Kimi en `harness_check.sh`): dentro del bucle de
  roles existente: chequeo estructural (`head -1 = ---`, `name:`,
  `description:`) + comparacion `extract_agent_body` (extractor comun con
  Claude/Gemini) contra `roles/<rol>.md`, mensaje accionable que nombra
  `.kimi-code/agents/<rol>.md` y ambos remedios. Condicionado a `[ -f ... ]`:
  la ausencia no falla. **Prueba negativa propia** (fixture standalone, fuera
  del smoke): espejos construidos con el formato exacto del instalador ->
  limpio rc=0 (cero falsos positivos); stale (append) -> rc=2 nombrando
  `.kimi-code/agents/reviewer.md`; `warn` rc=0 y reporta; `off` rc=0 con salida
  de 0 bytes; frontmatter removido -> rc=2 estructural. Smoke (b): lo mismo
  sobre la fixture instalada `check-robust` (stale rc=2 / warn / off /
  restaurado rc=0). Dogfooding en ESTE checkout (sin `.kimi-code/`):
  `env -u HARNESS_REPO_ROOT -u CLAUDE_PROJECT_DIR bash harness_check.sh` ->
  `[Ok] Harness Check limpio.` rc=0 (condicionalidad por existencia).
  `diff harness_check.sh templates/harness_check.sh` -> identicos.
- **AC-7** (superficies + mapa de agentes): `write_agent_surface` menciona a
  Kimi en el encabezado multi-backend, la lista de hooks nativos (globales en
  `~/.kimi-code/config.toml`), el bloque de launchers y la seccion de
  orquestacion (`.kimi-code/agents/*.md`, lee `AGENTS.md` nativo);
  `write_basic_agent_surface` lista el launcher. Smoke: `grep bin/harness-kimi`
  y `grep .kimi-code/agents` sobre el `AGENTS.md` generado (y el launcher en la
  superficie basica de `--no-subagents`). `roles/README.md` +
  `templates/roles/README.md`: bullet de Kimi en "Como se orquesta" (formato,
  reemplazo del system prompt, `Agent`/`AgentSwarm`, `--agent` v2 con
  `KIMI_CODE_EXPERIMENTAL_FLAG=1`, hooks solo-globales) y en "Modelos, effort y
  tools" (allowlist por rol). Espejo verificado:
  `sed 's|__HREL__|harness_process/|g' templates/roles/README.md | diff -
  roles/README.md` -> identicos (el sub-gate de la #7 en el dogfooding tambien
  paso).
- **AC-8** (`--reset` segun decision 1): `"$SURFACE_DIR/.kimi-code/agents"` y
  `"$SURFACE_DIR/bin/harness-kimi"` agregados a `reset_targets` (con
  `backup_file` previo como los demas), con comentario in-code de por que el
  bloque global NO se toca. Smoke (e): tras `--reset`, `.kimi-code/agents` y el
  launcher fuera del proyecto, backup `*.kimi-code/agents.bak.*` presente, y el
  `config.toml` global de fixture **byte-identico** (`cmp`) al pre-reset.
  `UPDATING.md` (raiz + template) documenta ambos casos: que limpia el reset y
  el procedimiento manual de remocion del bloque global.
- **AC-9** (smoke con fixtures `KIMI_CODE_HOME`): bloque nuevo "Feature #8" en
  `tests/setup_smoke.sh` con TODAS las ramas: (a) espejos en root+subdir
  (frontmatter/tools/cuerpo); (b) stale -> rc=2 + warn/off + restauracion; (c)
  creacion desde cero + blindaje de hooks del usuario + no-duplicacion + backup
  + rollback doctor; (d) `--no-kimi` (home intacto, aviso en log) y rama
  sin-deteccion (`KIMI_CODE_HOME` de fixture queda VACIO, `ls -A` = nada); (e)
  `--reset` conserva lo global y limpia lo local. Ademas, **endurecimiento de
  aislamiento**: `export KIMI_CODE_HOME="$TMP_ROOT/kimi-home-default"` al
  inicio del smoke, para que NINGUNA corrida del instalador (tambien las
  preexistentes) pueda tocar un home real aunque la maquina tenga `kimi` en
  PATH o la variable seteada. `bash tests/setup_smoke.sh` -> **rc=0** con la
  linea nueva `[Ok] Kimi Code: espejos por rol ... --reset conserva lo
  global.`; en esta maquina `kimi` NO esta en PATH, asi que las ramas
  doctor-rollback y sin-deteccion se ejecutaron de verdad (no hubo `[info]` de
  skip). Las 2 ramas sensibles a un `kimi` real en PATH degradan a skip
  informado (mismo patron que el skip de bash en el smoke ps1).
- **AC-10** (paridad Windows) - **PARCIAL: revision estatica, sin ejecucion**
  (no hay `pwsh` ni `powershell`; `command -v` vacio; precedente #1/#4/#5/#6/#7):
  - `setup_harness.ps1`: switch `-NoKimi`; espejos Kimi en
    `Write-AgentDefinitions` (here-string con `name`/`description`/`tools`,
    reutilizando la MISMA variable `$tools` del agente Claude = allowlist de la
    decision 3); `Write-KimiGlobalHooks` en paridad con `write_kimi_hooks`
    (deteccion `Get-Command kimi` o `bin/kimi|.exe|.cmd` bajo
    `$env:KIMI_CODE_HOME`/`$HOME\.kimi-code`, `Backup-HarnessPath` previo,
    splice del bloque por indices sobre el string crudo — preserva los bytes
    del usuario mejor que re-join de lineas —, mismo bloque TOML literal,
    doctor con `KIMI_CODE_HOME` seteado/restaurado y rollback, contadores);
    `kimi` en `Write-AgentLaunchers`; `.kimi-code/agents` +
    `bin/harness-kimi(.ps1)` en `Invoke-HarnessReset` (global intacto,
    comentario in-code); backups de espejos en `$surfaceBackups`; dir en el
    bloque de subagentes; llamada en el flujo principal tras `Write-AgentHooks`;
    superficie conceptual menciona Kimi.
  - `tests/setup_smoke.ps1`: `$env:KIMI_CODE_HOME` de fixture al inicio del try
    (**cierre de un hueco real**: este smoke NO overridea `HOME`, asi que sin
    esto una maquina Windows con kimi en PATH habria escrito el bloque en el
    `~/.kimi-code` REAL) + bloque Feature #8: espejos con `Get-AgentBody` vs
    `roles/`, allowlist por rol, `harness_check.sh` sembrado con el gate Kimi,
    launcher, y el ciclo global completo con kimi falso (`kimi.cmd` en Windows
    / sh en POSIX): sentinel + hook del usuario sobreviven, marcador unico,
    4 `[[hooks]]`, reinstall idempotente, `-Reset` deja el global identico y
    limpia el proyecto, `-NoKimi` no escribe; PATH del fake-cargo y
    `KIMI_CODE_HOME` restaurados en `finally`. Los here-strings nuevos son
    `@'...'@` literales (el bloque TOML con `$PWD` no se interpola).
  - Verificacion estatica mecanica: here-strings pareados 13/13
    (`setup_harness.ps1`) y 5/5 (`tests/setup_smoke.ps1`, 2 preexistentes + 3
    nuevos, ninguno sin cerrar); balance de llaves del DELTA agregado = 0.
    Nota de honestidad: mi checker crudo reporta -1 sobre el smoke ps1
    COMPLETO, pero da exactamente lo mismo sobre la version HEAD previa —
    artefacto del stripper de comillas con apostrofes dentro de strings dobles
    (p.ej. "the user's..."), no un desbalance real (el checker fino de la #7
    dio 0 sobre esa misma base). Ademas, relectura linea a linea del codigo
    nuevo. SIN ejecucion real.
- **AC-11** (docs con la justificacion escrita de la excepcion `$HOME`):
  `UPDATING.md` + `templates/UPDATING.md`: seccion nueva "Kimi Code CLI: hooks
  globales (unica excepcion de escritura en `$HOME`)" con el POR QUE (Kimi no
  soporta hooks por proyecto, verificado; decision usuario 2026-07-28), las 5
  salvaguardas (deteccion previa + flag, backup, marcadores idempotentes,
  doctor+rollback, guard por proyecto), los eventos y la razon de omitir
  `SessionEnd`, el procedimiento de remocion manual (decision 1), la guia de
  migracion para instalaciones existentes y la nota de acoplamiento a v0.29.x;
  mas el bullet en "Que se actualiza". `README.md`: seccion "Kimi Code CLI:
  backend con hooks globales" + flags `--no-kimi`/`-NoKimi` + lista de
  backends. `docs/architecture.md`: subagentes con `.kimi-code/agents/*.md`,
  bullet nuevo de `write_kimi_hooks`/`Write-KimiGlobalHooks` y gate de espejo
  extendido. `AGENTS.md` del checkout: gate de espejo con Kimi en el orden de
  trabajo. (El `AGENTS.md` GENERADO sale de U4/AC-7.) El par UPDATING
  raiz/template sigue difiriendo SOLO en el bloque historico "Notas de
  robustez" (verificado por diff).
- **AC-12** (comandos oficiales de `docs/verification.md`):
  - `(cd rust && cargo test --locked)`: **44 unit + 22 integracion, 0 fallos**
    (identico a la base de la #7: `rust/` no se toco; `Cargo.toml` sin
    dependencias nuevas, verificable por diff vacio).
  - `(cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)`:
    **rc=0** limpio.
  - `bash tests/setup_smoke.sh`: **rc=0** (2 corridas completas: intermedia y
    final con todos los archivos en su estado definitivo), con todas las lineas
    `[Ok]` previas intactas (cero regresion multi-LLM) y la nueva de Kimi.

## Decisiones aplicadas (todas del usuario, 2026-07-28; ninguna nueva)

- **Decision 1** (`--reset` NO toca el bloque global): implementada en ambos
  reset (sh y ps1) — solo `.kimi-code/agents` + launcher del proyecto; probada
  con `cmp` byte a byte del config global antes/despues del reset; remocion
  manual documentada en `UPDATING.md`.
- **Decision 2** (bloque global solo con Kimi detectado + `--no-kimi`):
  deteccion `command -v kimi` || `-x $kimi_home/bin/kimi` (ps1: `Get-Command` ||
  `bin/kimi|.exe|.cmd`); flag en ambos instaladores; artefactos de proyecto
  SIEMPRE. Probadas las 3 ramas: detectado (escribe), no detectado (home de
  fixture queda vacio), `--no-kimi` (aviso + nada escrito).
- **Decision 3** (frontmatter `tools` con allowlist por rol): leader/reviewer
  `Read, Grep, Glob, Bash`; implementer `Read, Edit, Write, Bash, Grep, Glob`
  (mismo conjunto que el agente Claude = read-only + `Edit, Write`; nombres
  verificados en v0.29.2). El gate de espejo no se ve afectado (solo compara el
  cuerpo).

## Verificacion empirica adicional de esta sesion (v0.29.2 real, todo en fixtures)

- `kimi doctor` con config valido (hooks presentes, SIN login/default_model):
  rc=0 "All checked config files are valid" -> la falta de login del usuario no
  provoca falsos rollbacks (relevante para el config casi vacio de esta
  maquina).
- `kimi doctor` con TOML invalido: rc=1 con "Invalid TOML" -> rollback fiable.
- `kimi doctor` NO escribe en `KIMI_CODE_HOME` (fixture vacia quedo vacia).
- El bloque exacto que genera `write_kimi_hooks` fue validado por el doctor
  real (rc=0) y parseado por `tomllib` (3 hooks, timeouts 120/30/120, matcher
  `Edit|Write`, comandos con guard y despacho `plain`).

## Hallazgos colaterales

- **Hueco de aislamiento en los smoke (cerrado aqui)**: ninguno de los dos
  smoke fijaba `KIMI_CODE_HOME`, y `tests/setup_smoke.ps1` ademas NO overridea
  `HOME`: en una maquina con `kimi` en PATH, los install de fixtures habrian
  escrito el bloque global en el home REAL del usuario. Ambos smoke ahora
  exportan una fixture por defecto al inicio (sh: bajo `$TMP_ROOT`; ps1: bajo
  `$tempRoot`, restaurada en `finally`).
- Artefacto del checker estatico ps1 (balance -1 preexistente por apostrofes
  en strings dobles), documentado en AC-10; el delta agregado esta balanceado.

## OBSERVACION SIN DECISION

- Ninguna. Las 3 decisiones registradas cubrieron todos los forks que
  aparecieron durante la implementacion.

## Riesgos pendientes para el reviewer

- **AC-10 parcial**: sin ejecucion real de `setup_harness.ps1` /
  `tests/setup_smoke.ps1` (no hay pwsh/powershell en esta maquina); revision
  estatica documentada arriba. Misma deuda acumulada de #1/#4/#5/#6/#7.
- **Launcher `bin/harness-kimi` usa `command -v kimi`** (mismo esqueleto que
  los demas, como exige AC-2): en maquinas donde el binario vive solo en
  `~/.kimi-code/bin` (fuera del PATH, como esta), el launcher sale 127 con
  mensaje claro hasta que el usuario agregue Kimi a su PATH. No es un fork (el
  spec fija "mismo esqueleto"); un fallback a `KIMI_CODE_HOME/bin` seria una
  mejora a decidir aparte.
- **Ramas del smoke sensibles a `kimi` en PATH**: doctor-rollback y
  no-deteccion se saltan (con `[info]`) si el entorno de test tiene un `kimi`
  real en PATH, porque la deteccion lo preferiria al fake. En esta maquina
  corrieron de verdad. `--no-kimi` y el resto son deterministas siempre.
- **Hook global en Windows**: el `command` del bloque es POSIX; no esta
  verificado con que shell ejecuta Kimi los hooks en Windows. La paridad ps1
  escribe el mismo bloque y `UPDATING.md` lo declara best-effort alli (riesgo
  ya listado en el plan).
- **El home real de Kimi quedo intacto** tras TODA la verificacion:
  `~/.kimi-code/config.toml` = 181 bytes, mtime `Jul 27 17:12`, `grep -c
  '^\[\[hooks\]\]'` = 0 (verificado al inicio, tras el smoke intermedio y al
  final). Toda escritura global de los tests ocurrio en fixtures
  `KIMI_CODE_HOME` bajo el tmp del smoke o el scratchpad de la sesion.
- Arbol SIN commit a proposito (decision del flujo: el commit lo decide el
  usuario). Modificados: 13 archivos + este impl; los pares espejo
  (`harness_check.sh`, `roles/README.md`, `UPDATING.md`) deben viajar en el
  mismo commit (Articulo 6); verificados identicos/equivalentes por diff en
  esta sesion.
