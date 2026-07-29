# Plan - Feature #8: kimi_cli_backend

Estado: in_progress
Microservicios:
- harness

## Alcance

Kimi Code CLI (v0.29.x) como backend de primera clase del arnes, en paridad
con Claude/Codex/Gemini/Grok:

- Subagentes `leader`/`implementer`/`reviewer` en `.kimi-code/agents/*.md`
  del proyecto, generados desde `roles/` (fuente unica) con el patron de
  `build_claude_agent` (formato verificado empiricamente: Markdown +
  frontmatter `name`/`description`, cuerpo = system prompt).
- Superficie de contexto: `AGENTS.md` ya generado por el arnes (verificado:
  Kimi lo inyecta al system prompt). Solo se actualizan los textos
  multi-backend, no se crea superficie nueva.
- Hooks de ciclo de vida: bloque `[[hooks]]` DELIMITADO en el config GLOBAL
  `${KIMI_CODE_HOME:-$HOME/.kimi-code}/config.toml` (unica via que Kimi
  ofrece; decision usuario 2026-07-28) con backup, idempotencia y guard por
  proyecto (`$PWD/bin/harness-hook`), despachando al runtime existente
  `bin/harness-hook plain <evento>`.
- Gate de espejo de la feature #7 extendido a `.kimi-code/agents/`.
- Launcher `bin/harness-kimi`, reset targets, smoke tests, paridad
  PowerShell y documentacion (incl. justificacion escrita de la excepcion
  `$HOME`).

FUERA: Kimi como proveedor de API LLM-driven (decision usuario), superficie
KIMI.md, modos acp/web/migrate, skills graphify para Kimi.

Spec: `docs/spec-feature-8-kimi-cli-backend.md` (AC-1..AC-12), con la
investigacion empirica del 2026-07-28 que fija el terreno (AGENTS.md
soportado; hooks solo globales; cwd del hook = proyecto; sin env var de
proyecto; Stop exit 2 + stderr bloquea; tools `Edit|Write` verificadas).

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->
- `sh harness_cli graph impacto --microservicio ADR/harness` (2026-07-28):
  "Ningun microservicio registrado depende de 'ADR/harness'". Impacto externo
  nulo; el radio es interno al repo del arnes.
- Radio interno (contratos compartidos del propio arnes):
  - `setup_harness.sh`: `build_kimi_agent` (nueva, junto a `build_*_agent`),
    `write_kimi_hooks` (nueva, junto a `write_*_hooks`), `write_launchers`
    (lista de agentes), `backup_file` de los espejos nuevos, `reset_targets`,
    bloque `--reset`, salida final informativa, flag nuevo si la decision 2
    lo trae (`--no-kimi`).
  - `harness_check.sh` + `templates/harness_check.sh` (espejo identico por
    diff): gate de espejo con el bucle de roles existente
    (`extract_agent_body`) + chequeo estructural.
  - `write_agent_surface` / `write_basic_agent_surface` (texto multi-backend
    de `CLAUDE.md`/`AGENTS.md`/`GEMINI.md`/`LLM.md`): menciones de Kimi.
  - `roles/README.md` + `templates/roles/README.md` (espejo modulo
    `__HREL__`, vigilado por el sub-gate de la #7).
  - `setup_harness.ps1`: `Write-AgentDefinitions`, `Write-AgentHooks` (o
    funcion hermana nueva), `Write-AgentLaunchers`, `Invoke-HarnessReset`,
    `Backup-HarnessPath`.
  - Tests: `tests/setup_smoke.sh`, `tests/setup_smoke.ps1`.
  - Docs: `README.md`, `UPDATING.md` (+ `templates/UPDATING.md`),
    `AGENTS.md` (via superficie), `docs/architecture.md`.
  - SIN cambios en: binario Rust (`rust/`), `harness_cli`, `init.sh`,
    `commit_guard.sh`, `harness_status.sh`, `bin/harness-hook` (los case
    patterns existentes ya aceptan los eventos de Kimi; verificado).
- Estado global de la maquina (unico efecto fuera del proyecto):
  `${KIMI_CODE_HOME:-$HOME/.kimi-code}/config.toml`, con backup y bloque
  delimitado; compartido entre TODOS los proyectos con arnes (relevante para
  la decision 1 de `--reset`).

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->
- `graphify query "donde genera setup_harness los subagentes por backend y
  los hooks nativos"` (111 nodos, 2026-07-28): confirma un unico dispatcher
  multi-LLM (`bin/harness-hook`) llamando a `init.sh`/`harness_status.sh`/
  `harness_check.sh`/`commit_guard.sh`/`harness_cli`, y el principio
  `multi_llm_backend_agnostic` como nodo rationale: Kimi debe engancharse al
  dispatcher existente, no duplicarlo (asi se diseno el spec).

## Delegacion (implementer)

Orden: U0 -> U1 -> U2 -> U3 -> U4 -> U5 -> U6 -> U7. Cada unidad cita sus AC
(Articulo 3). Regla transversal: raiz y `templates/` espejados en el mismo
commit (Articulo 6); NUNCA correr `setup_harness.sh` en este checkout (los
smoke usan fixtures y `KIMI_CODE_HOME`/`HOME` de fixture).

- U0 (gate previo, Articulo 5; sin AC propio): preguntar al USUARIO las 3
  decisiones PENDIENTE DE DECISION del spec (reset del bloque global; cuando
  instalar el bloque + flag; `tools` del frontmatter) y registrar las
  respuestas en spec y plan ANTES de tocar codigo.
- U1 [AC-1, AC-2, AC-8-proyecto]: `setup_harness.sh`: `build_kimi_agent`
  (espejo del patron `build_claude_agent`, cuerpo verbatim de
  `roles/<rol>.md` + `__HREL__`; frontmatter segun decision 3) invocada para
  los tres roles solo con `WITH_SUBAGENTS=1`; `backup_file` de
  `.kimi-code/agents/*.md`; `bin/harness-kimi` en `write_launchers`;
  `"$SURFACE_DIR/.kimi-code/agents"` en `reset_targets`; `do_mkdir` del
  directorio; salida final del instalador actualizada.
- U2 [AC-3, AC-4, AC-5, AC-8-global]: `write_kimi_hooks` en
  `setup_harness.sh`: resolver `${KIMI_CODE_HOME:-$HOME/.kimi-code}`;
  `backup_file` del `config.toml` global ANTES de escribir; bloque delimitado
  por marcadores propios con los tres `[[hooks]]` del spec (SessionStart 120,
  PostToolUse matcher `Edit|Write` 30, Stop 120; command con guard `$PWD` y
  despacho `plain`); insercion/reemplazo idempotente entre marcadores sin
  tocar el resto del archivo; creacion de dir/archivo si faltan; validacion
  best-effort con `kimi doctor` + rollback al backup si invalido (aviso
  accionable, exit del setup inalterado); condicion de instalacion y flag
  segun decision 2; comportamiento de `--reset` segun decision 1.
- U3 [AC-6]: gate de espejo en `harness_check.sh` +
  `templates/harness_check.sh`: dentro del bucle de roles existente, agregar
  `kimi_md="$REPO_ROOT/.kimi-code/agents/$role.md"` con (a) chequeo
  estructural (frontmatter `---`, `name:`, `description:`) y (b) comparacion
  `extract_agent_body` vs `roles/<rol>.md` con mensaje accionable; ausencia
  no falla; `HARNESS_CHECK_MODE` degrada igual; ambos archivos identicos por
  diff.
- U4 [AC-7]: superficies y mapa de agentes: `write_agent_surface` y
  `write_basic_agent_surface` (arranque automatico, orquestacion por
  herramienta, launchers, lista final) mencionan Kimi;
  `roles/README.md` + `templates/roles/README.md` (modulo `__HREL__`)
  agregan la fila/detalle de Kimi Code (formato, precedencia de agentes,
  hooks globales, v2/--agent en modo -p).
- U5 [AC-9]: bloques nuevos en `tests/setup_smoke.sh` con `KIMI_CODE_HOME`
  de fixture: (a) espejos Kimi generados en subdir y root (frontmatter +
  cuerpo == rol); (b) stale inyectado -> check reporta y falla en block; (c)
  bloque global: creacion desde cero, sentinel de hooks del usuario
  sobrevive, re-instalacion no duplica (contar marcadores), backup presente;
  (d) rama de no-instalacion segun decision 2 -> `KIMI_CODE_HOME` de fixture
  intacto; (e) `--reset` segun decision 1. Correr `bash tests/setup_smoke.sh`
  completo.
- U6 [AC-10]: paridad Windows: `setup_harness.ps1` (agentes Kimi en
  `Write-AgentDefinitions`, hooks globales en paridad — ruta
  `$env:KIMI_CODE_HOME`/`$HOME\.kimi-code` —, launcher, reset) y
  `tests/setup_smoke.ps1` (bloques de U5 portados). Sin `pwsh` en esta
  maquina: verificacion estatica documentada (precedente #1/#4/#5/#6/#7).
- U7 [AC-11, AC-12]: docs (`README.md`, `UPDATING.md` raiz + template,
  `docs/architecture.md`; `AGENTS.md` sale de U4) con la justificacion
  escrita de la excepcion `$HOME` y la guia de actualizacion; verificacion
  final: `cargo test`, `cargo clippy -- -D warnings`,
  `bash tests/setup_smoke.sh` y evidencia por AC en `docs/impl-8.md`.

## Criterios de cierre (reviewer)

- Evidencia POR AC (AC-1..AC-12) en `docs/impl-8.md`; ningun AC sin
  evidencia (Articulo 3).
- Las 3 decisiones de Observaciones registradas con la respuesta del usuario
  y la implementacion las respeta (Articulo 5); las 2 decisiones ya tomadas
  (alcance; hooks globales blindados) no contradichas.
- `diff harness_check.sh templates/harness_check.sh` identico;
  `roles/README.md` vs `templates/roles/README.md` equivalentes modulo
  `__HREL__` (el propio gate de la #7 en verde lo demuestra).
- Prueba negativa reproducida (fixture propia, no solo el smoke): espejo
  `.kimi-code/agents/*.md` alterado -> `harness_check.sh` rc=2 nombrando el
  archivo; `warn` rc=0 reportando; `off` silencioso.
- Prueba de blindaje global reproducida: config de fixture con hooks del
  usuario -> re-install x2 -> hooks del usuario intactos, 1 solo bloque,
  backup en `HARNESS_BKP_DIR` de fixture.
- Comandos oficiales de `docs/verification.md` en verde (AC-12).
- Dogfooding en ESTE checkout: `bash harness_check.sh` limpio (la ausencia
  de `.kimi-code/` en el checkout NO debe fallar; confirma la
  condicionalidad de AC-6).
- Cero escrituras fuera de fixtures durante los tests (el `KIMI_CODE_HOME`
  real y `~/.kimi-code/config.toml` del usuario intactos; verificable por
  mtime).
- AC-10 sin `pwsh`: revision estatica linea a linea registrada en
  `docs/review-8.md`.
- Commits Conventional SIN trailers de IA (Articulo 6 / UPDATING.md).

## Riesgos

- **Editar TOML ajeno**: el config global es del usuario; un append/replace
  ingenuo puede romper sintaxis (p.ej. archivo terminando sin newline o
  dentro de una tabla). Mitigacion: bloque SIEMPRE al final del archivo,
  delimitado por marcadores en lineas propias, reemplazo solo entre
  marcadores, newline previo garantizado, validacion `kimi doctor`
  best-effort + rollback (AC-5), y backup previo (AC-3).
- **Hook global corre en TODOS los proyectos de la maquina**: mitigado con
  guard barato por `$PWD/bin/harness-hook` (verificado que el hook corre con
  cwd = proyecto). En cwd que no es la raiz del proyecto, el hook es no-op:
  limite documentado.
- **Acoplamiento a v0.29.x**: nombres de eventos/tools verificados hoy;
  una version futura podria cambiarlos. El fallo es benigno (hook no
  matchea o matcher no aplica) y el gate de espejo no depende de Kimi.
  Nota en `UPDATING.md`.
- **`SessionEnd`/`Stop` solapados en `run_event`**: registrar SOLO `Stop`
  (nota de diseno del spec); si un implementer agrega `SessionEnd`,
  duplicaria el check por turno.
- **Windows**: no esta verificado con que shell ejecuta Kimi el `command`
  del hook en Windows (el guard usa sintaxis POSIX). La paridad ps1 escribe
  el mismo bloque; si Kimi/Windows no ejecuta sh, el hook global queda
  best-effort alli y se documenta (misma deuda de ejecucion real que todo
  AC-Windows: precedente #1/#4/#5/#6/#7).
- **`--agent` en modo `-p` es v2-experimental** (`KIMI_CODE_EXPERIMENTAL_FLAG=1`):
  los smoke NO deben depender de invocar `kimi` (no hay credenciales/CLI en
  CI); toda la verificacion de artefactos es por archivo, como con los demas
  backends.
- **Firmas de frescura**: este plan y el spec fueron editados por el lider:
  `check-plan`/`check-spec` reportaran stale hasta el `approve-spec` del
  usuario (re-firma el spec) y el primer `advance` (re-firma el plan). No
  trabajar con esos gates en rojo sin re-leer (mismo aviso que en la #7).

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
U0 CERRADA: las 3 decisiones se le preguntaron a Alan en el chat el 2026-07-28
y quedaron registradas aqui y en las Observaciones del spec. El implementer NO
debe volver a preguntarlas.

- Decision 1 (afecta U2/U5 y AC-8/AC-9e): `--reset` y el bloque GLOBAL de
  hooks — **(b) NO tocarlo** y documentar la remocion manual en `UPDATING.md`.
  DECIDIDO por el usuario (2026-07-28). El bloque es compartido entre
  proyectos y un reset por-proyecto no debe des-hookear a los demas.
- Decision 2 (afecta U2/U5 y AC-3/AC-5/AC-9d): cuando instalar el bloque
  global — **(b) solo con Kimi detectado** (`command -v kimi` o
  `${KIMI_CODE_HOME:-$HOME/.kimi-code}/bin/kimi`) + flag `--no-kimi`.
  DECIDIDO por el usuario (2026-07-28). Los artefactos de proyecto se generan
  siempre; la escritura en `$HOME` solo si el backend existe.
- Decision 3 (afecta U1 y AC-1): frontmatter `tools` de los subagentes Kimi —
  **(a) allowlist por rol** con nombres verificados (leader/reviewer
  read-only: `Read, Grep, Glob, Bash`; implementer: + `Edit, Write`).
  DECIDIDO por el usuario (2026-07-28).

### Avance 2026-07-28T23:20:44Z
Re-sincronizado con plan actualizado por otro agente (U0 cerrada: 3 decisiones registradas por Alan)

### Avance 2026-07-28T23:42:28Z
Feature #8 U1-U7 implementadas: build_kimi_agent + write_kimi_hooks (bloque global blindado) + gate espejo Kimi + superficies/README + smoke sh en verde + paridad ps1 + docs

### Avance 2026-07-28T23:47:14Z
Evidencia por AC-1..AC-12 escrita en docs/impl-8.md; cargo test 44+22/0, clippy limpio, smoke rc=0 con bloque Kimi, prueba negativa propia del gate, dogfood limpio, home real de Kimi intacto (181 bytes, 0 hooks)

---
Cerrado: 2026-07-29T00:06:56Z - status=done - Kimi Code CLI como backend de primera clase: subagentes en .kimi-code/agents/, superficie AGENTS.md nativa, hook global blindado (backup + marcadores + idempotencia + rollback por doctor), gate de espejo extendido. AC-10 (pwsh) verificado estaticamente
