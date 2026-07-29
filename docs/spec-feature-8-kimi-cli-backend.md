# Spec - Feature #8: kimi_cli_backend

Estado: approved
Aprobado: 2026-07-28T23:19:20Z por USUARIO (confirmacion explicita) - Alan aprobo el spec #8 en el chat (2026-07-28) tras revisarlo, con las 3 decisiones de Observaciones ya registradas
Plan: docs/plan-feature-8-kimi-cli-backend.md
Constitution: docs/constitution.md

## Problema

El arnes trata como backends de primera clase a Claude Code, Codex CLI,
Gemini CLI y Grok Build (subagentes desde `roles/`, superficie de contexto,
hooks de ciclo de vida y gate de espejo de la feature #7). Kimi Code CLI
v0.29.2 (sucesor Node/TS de kimi-cli, instalado en `~/.kimi-code/bin/kimi`)
hoy aparece UNA sola vez en el repo (`setup_harness.sh:829`) y solo como
posible proveedor de API: un usuario de Kimi opera sin roles, sin hooks y sin
gates. El objetivo es que Kimi corra el ciclo leader -> implementer ->
reviewer con las mismas protecciones que los demas backends, sin degradar a
ninguno (principio multi-LLM del Articulo 6).

Particularidad que condiciona el diseno: Kimi NO soporta hooks por proyecto.
El unico lugar donde existen es el config GLOBAL del usuario
(`${KIMI_CODE_HOME:-$HOME/.kimi-code}/config.toml`, array `[[hooks]]`).
El usuario ya decidio (2026-07-28) aceptar esa unica excepcion a la regla de
no escribir fuera del proyecto, blindada: backup previo, bloque delimitado,
idempotencia y preservacion absoluta de los hooks preexistentes del usuario.

## Investigacion empirica del lider (2026-07-28, Kimi Code CLI v0.29.2 real)

Metodo: fixture temporal FUERA del repo + `KIMI_CODE_HOME` aislado en un
scratchpad + provider `openai` apuntando a un servidor local de captura
(inspeccion del request que Kimi arma para el modelo). El home real del
usuario no se modifico. Hallazgos que este spec da por hechos:

1. **`AGENTS.md` de la raiz del proyecto SE CARGA al system prompt** (seccion
   "The applicable `AGENTS.md` instructions are:" con marcador
   `<!-- From: <ruta-absoluta>/AGENTS.md -->`; semantica de arbol con
   precedencia por profundidad). La superficie `AGENTS.md` que el arnes ya
   genera sirve para Kimi tal cual: NO hace falta un `KIMI.md`.
2. **Subagentes de proyecto en `.kimi-code/agents/*.md` se descubren**
   (Markdown + frontmatter YAML `name`/`description`, cuerpo = system
   prompt). Formato equivalente a `.claude/agents/*.md`. Al seleccionar un
   perfil, su cuerpo REEMPLAZA el system prompt (el rol debe bastarse solo,
   como ya ocurre con los roles del arnes). En modo `-p`, `--agent` exige
   `KIMI_CODE_EXPERIMENTAL_FLAG=1` (v2 engine); en sesion interactiva la
   delegacion existe via tools `Agent` / `AgentSwarm`.
3. **Hooks SOLO globales**: `[[hooks]]` con `event` (obligatorio), `matcher`
   (regex opcional), `command` (obligatorio), `timeout` (1-600s, default 30)
   en el config global. Test funcional negativo: un `[[hooks]]` en
   `.kimi-code/local.toml` del proyecto NO se ejecuta (y `kimi doctor` ni lo
   valida). No hay via por-proyecto.
4. **El hook corre con cwd = directorio del proyecto** y recibe por stdin un
   JSON estilo Claude (`hook_event_name`, `session_id`, `cwd`, mas extras por
   evento: `source`, `stop_hook_active`, `reason`, `prompt`). **NO existe
   ninguna variable de entorno de proyecto** (nada `KIMI_*`): la raiz se
   resuelve con `$PWD` del propio hook.
5. **`Stop` con exit 2 + stderr BLOQUEA**: Kimi no cierra el turno, reinyecta
   el stderr como mensaje de usuario y relanza al modelo (verificado con 2
   requests capturadas). Semantica identica a Claude Code: el modo `plain` de
   `bin/harness-hook` (exit code de `harness_check.sh` + stderr) sirve sin
   crear un modo JSON nuevo.
6. **Eventos verificados en una corrida**: `SessionStart`, `UserPromptSubmit`,
   `Stop`, `SessionEnd` disparan (nombres exactos que los case patterns de
   `run_event` en `bin/harness-hook` ya matchean).
7. **Tools (case-sensitive)**: `Agent`, `AgentSwarm`, `AskUserQuestion`,
   `Bash`, `Edit`, `Glob`, `Grep`, `Read`, `Write`, entre otras. Los nombres
   relevantes coinciden con los de Claude (`Edit|Write` para el matcher de
   PostToolUse; `Read, Grep, Glob, Bash` para roles read-only).
8. **Esquema de config verificado**: `default_model`, `[providers.<id>]`
   (`type` en {kimi, openai, anthropic, openai_responses, google}, `name`,
   `base_url`, `api_key`) y `[models."<alias>"]` (`provider`, `model`,
   `max_context_size`, ...). `kimi doctor` valida config.toml y sale con
   mensajes claros; `KIMI_CODE_HOME` reubica todo el home (base de los tests).
9. En esta maquina el `~/.kimi-code/config.toml` real esta casi vacio (sin
   hooks del usuario, sin `default_model`); el modo `-p` fallara hasta que el
   usuario complete su login/modelo. El instalador NO depende de eso: solo
   escribe el bloque de hooks.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario de Kimi Code CLI, quiero abrir `kimi` en la raiz de un
  proyecto con arnes y que el contexto (`AGENTS.md`), los tres roles
  (`.kimi-code/agents/`) y los gates de ciclo de vida (init/status/nudge/
  check) operen solos, para correr el ciclo leader -> implementer -> reviewer
  con la misma proteccion que Claude/Codex/Gemini/Grok.
- P1: Como dueno de la maquina, quiero que el instalador toque el config
  global de Kimi de forma reversible (backup previo), delimitada (marcadores
  propios), idempotente (re-instalar no duplica) y que JAMAS destruya mis
  hooks o config preexistentes, para confiar en la unica excepcion a la regla
  de no escribir fuera del proyecto.
- P1: Como usuario de otro backend (o de una maquina sin Kimi), quiero que
  esta feature no cambie NADA de mi flujo: superficies, hooks y agentes de
  Claude/Codex/Gemini/Grok/Antigravity intactos, y cero escrituras globales
  que no me apliquen.
- P2: Como equipo multi-LLM, quiero que el gate de espejo de la feature #7
  cubra tambien `.kimi-code/agents/`, para que Kimi nunca opere con un rol
  stale sin que `harness_check.sh` lo grite.
- P2: Como usuario de Windows, quiero `setup_harness.ps1` y
  `tests/setup_smoke.ps1` en paridad exacta con sus pares `.sh`.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given una instalacion con subagentes (default), When corre
  `setup_harness.sh`, Then genera `.kimi-code/agents/{leader,implementer,
  reviewer}.md` en `SURFACE_DIR` con el mismo patron que `build_claude_agent`:
  frontmatter YAML (`name:` = rol, `description:` compartida `desc_*`, campos
  extra segun la decision 3 de Observaciones), una linea en blanco y el cuerpo
  VERBATIM de `roles/<rol>.md` con `__HREL__` sustituido; y Given
  `--no-subagents`, Then esos archivos NO se generan (misma condicionalidad
  que `.claude/.codex/.gemini`).
- AC-2: Given la instalacion, When termina el setup, Then existe el launcher
  `bin/harness-kimi` (mismo esqueleto que los demas launchers), los espejos
  nuevos tienen su `backup_file` previo como el resto, y el resumen final del
  instalador lista la superficie/hooks/launcher de Kimi.
- AC-3: Given que aplica instalar el bloque global (segun la decision 2 de
  Observaciones), When corre el instalador, Then
  `${KIMI_CODE_HOME:-$HOME/.kimi-code}/config.toml` queda con UN bloque
  delimitado por marcadores propios del arnes que contiene `[[hooks]]` para
  exactamente tres eventos: `SessionStart` (timeout 120), `PostToolUse`
  (matcher `Edit|Write`, timeout 30) y `Stop` (timeout 120) — sin
  `SessionEnd` (duplicaria el check de `Stop` en `run_event`) y sin
  `UserPromptSubmit` (paridad con los demas backends); cada `command` es
  autonomo y generico entre proyectos: guard que sale 0 en silencio si
  `$PWD/bin/harness-hook` no existe/no es ejecutable, y si existe despacha
  `bin/harness-hook plain <session-start|post-tool|stop>` con
  `HARNESS_REPO_ROOT="$PWD"`; y ANTES de tocar el archivo se crea backup via
  el mecanismo existente (`backup_file`/`HARNESS_BKP_DIR`).
- AC-4: Given un `config.toml` global con contenido del usuario (hooks
  propios incluidos) y el bloque del arnes ya instalado, When re-corro el
  instalador, Then el contenido del usuario fuera del bloque queda intacto
  byte a byte, el bloque NO se duplica (se reemplaza entre marcadores) y el
  archivo queda con exactamente un bloque del arnes.
- AC-5: Given una maquina sin `config.toml` global (o sin `~/.kimi-code/`),
  When aplica instalar el bloque, Then el instalador crea directorio y
  archivo con solo el bloque delimitado; y si el binario `kimi` esta
  disponible, corre `kimi doctor` como validacion best-effort del TOML
  resultante: si doctor reporta config invalido, restaura el backup (o retira
  el archivo recien creado), avisa con mensaje accionable y la instalacion
  continua sin cambiar su exit code (el bloque global es best-effort, nunca
  rompe el resto del setup).
- AC-6: Given `.kimi-code/agents/<rol>.md` presente con cuerpo embebido
  distinto de `roles/<rol>.md`, When corro `bash harness_check.sh`, Then el
  gate de espejo (feature #7) lo reporta con mensaje accionable (archivo
  exacto + remedio re-correr el instalador / propagar a `roles/`), suma
  `failures` y sale 2 en `block` (`warn` reporta y sale 0; `off` no evalua);
  la extraccion reusa `extract_agent_body` (mismo formato Markdown que
  Claude/Gemini); el chequeo estructural (frontmatter presente,
  `name:`/`description:`) aplica como en `.claude/agents/`; y Given que
  `.kimi-code/agents/` NO existe (instalacion vieja, `--no-subagents` o
  checkout fuente), Then su ausencia NO falla (condicionalidad por
  existencia, como los demas espejos).
- AC-7: Given las superficies generadas (`AGENTS.md`, `CLAUDE.md`,
  `GEMINI.md`, `LLM.md` comparten plantilla `write_agent_surface`) y
  `roles/README.md`, When las leo tras el setup, Then Kimi Code figura como
  backend de primera clase: lee `AGENTS.md` nativamente (verificado
  empiricamente), subagentes en `.kimi-code/agents/*.md`, hooks globales en
  `${KIMI_CODE_HOME:-$HOME/.kimi-code}/config.toml` y launcher
  `bin/harness-kimi`; y `templates/roles/README.md` queda espejado modulo
  `__HREL__` (el sub-gate de la #7 pasa).
- AC-8: Given `--reset`, When corre, Then `$SURFACE_DIR/.kimi-code/agents`
  (artefacto generado en el proyecto) entra en `reset_targets` con backup
  previo como los demas; y el bloque GLOBAL de hooks se trata segun la
  decision 1 de Observaciones, documentado en `UPDATING.md` en ambos casos.
- AC-9: Given `tests/setup_smoke.sh`, When corre, Then cubre con fixtures
  propias (usando `KIMI_CODE_HOME` de fixture, NUNCA el home real): (a)
  generacion de los tres espejos Kimi en layout subdir y root, con frontmatter
  valido y cuerpo == `roles/<rol>.md`; (b) espejo Kimi stale inyectado ->
  `harness_check.sh` lo reporta y falla en `block`; (c) bloque global: config
  inexistente -> se crea; config con hooks del usuario + sentinel -> sentinel
  sobrevive y el bloque no se duplica tras re-instalar; backup creado; (d) la
  rama de NO instalacion del bloque (segun decision 2) no escribe nada en el
  `KIMI_CODE_HOME` de fixture; (e) `--reset` se comporta segun la decision 1.
  `bash tests/setup_smoke.sh` sale 0.
- AC-10: Given `setup_harness.ps1` y `tests/setup_smoke.ps1`, When se
  comparan con sus pares `.sh`, Then replican esta feature (agentes Kimi en
  `Write-AgentDefinitions`, bloque global con backup/idempotencia en paridad
  con el `.sh`, reset targets, bloques de smoke); sin `pwsh`/`powershell` en
  la maquina se verifica estaticamente, como en las features #1, #4, #5, #6
  y #7.
- AC-11: Given `README.md`, `UPDATING.md` (raiz y template), `AGENTS.md` y
  `docs/architecture.md`, When busco el soporte Kimi, Then esta descrito
  donde corresponde, incluyendo POR ESCRITO la justificacion de la excepcion
  unica de escritura en `$HOME` (hooks solo-globales de Kimi; decision del
  usuario 2026-07-28) y sus salvaguardas (backup, marcadores, idempotencia,
  guard por proyecto), mas la nota de migracion/actualizacion para
  instalaciones existentes.
- AC-12: Given el repo, When corro los comandos oficiales de
  `docs/verification.md` (`cargo test`, `cargo clippy -- -D warnings`,
  `bash tests/setup_smoke.sh`), Then los tres pasan (el binario Rust no se
  toca en esta feature; los tests protegen contra regresiones).

## No funcionales
- SLOs: el gate de espejo Kimi agrega comparaciones locales POSIX (mismo
  costo que #7); los hooks Kimi reusan `bin/harness-hook` (sin runtime
  nuevo); en proyectos SIN arnes el guard del hook global cuesta un stat de
  `$PWD/bin/harness-hook` y sale 0. Sin red en instalador, gates y hooks
  (salvo `kimi doctor` local best-effort). Sin dependencias nuevas en
  `rust/Cargo.toml` (Articulo 6).
- Seguridad: el bloque global NO contiene secretos ni los lee (no se parsea
  el resto del `config.toml` del usuario, solo se localizan los marcadores
  propios); backup antes de toda escritura global; `harness_check.sh` sigue
  read-only. Sin secretos en tests: fixtures con providers dummy locales
  (Articulo 4).
- Observabilidad: el instalador informa la ruta global tocada, el backup
  creado y el resultado de la validacion; mensajes accionables y exit codes
  estables (el bloque global best-effort nunca cambia el exit del setup;
  el gate de espejo mantiene 0/2 con `HARNESS_CHECK_MODE`).
- Multi-LLM: agregar Kimi no altera artefactos ni flujos de Claude, Codex,
  Gemini, Grok ni Antigravity; ninguna pieza queda pinchada a Kimi (guard
  por existencia, condicionalidad identica a los demas backends).

## Fuera de alcance
- Kimi como PROVEEDOR de API para features LLM-driven del arnes (graphify
  semantico, etc.): decision del usuario 2026-07-28; la mencion existente en
  `setup_harness.sh:829` queda como esta.
- Configurar el login/provider/modelo de Kimi del usuario (`default_model`,
  `[providers.*]`, credenciales): el instalador solo gestiona SU bloque de
  hooks delimitado.
- Modos ACP (`kimi acp`), web (`kimi web`) y la migracion desde kimi-cli
  legacy (`kimi migrate`).
- Superficie `KIMI.md` propia (innecesaria: Kimi lee `AGENTS.md`, verificado)
  y el `AGENTS.md` global de `$KIMI_CODE_HOME`.
- Skill/comando `/graphify` nativo para Kimi (graphify no tiene
  `--platform kimi`; como Grok, Kimi usa el CLI `graphify update/query`).
- Hooks de eventos fuera del trio del arnes (`UserPromptSubmit`,
  `PreToolUse`, `PermissionRequest`, `SubagentStart/Stop`, `PreCompact`...).
- Tocar la cadena `AGENT_PROJECT_DIR` de `harness_check.sh:5` (no existe
  variable de proyecto de Kimi que agregar; el hook global pasa
  `HARNESS_REPO_ROOT="$PWD"`, ya honrada por toda la resolucion de la #7).

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- DECIDIDO por el usuario (2026-07-28): alcance = backend CLI completo
  (subagentes generados desde `roles/`, superficie de contexto, gate de
  espejo, documentacion); Kimi como proveedor de API queda FUERA.
- DECIDIDO por el usuario (2026-07-28): los hooks van al config GLOBAL
  `~/.kimi-code/config.toml` (unica via que Kimi ofrece; verificado), con
  backup previo (`backup_file`/`HARNESS_BKP_DIR`), bloque delimitado por
  marcadores propios, idempotente, y sin destruir JAMAS hooks preexistentes
  del usuario. Unica excepcion a la regla de no escribir en `$HOME`,
  justificada por escrito en la documentacion (AC-11).
- DECIDIDO por el usuario (2026-07-28) — `--reset` y el bloque GLOBAL de
  hooks: **(b) NO lo toca**; `UPDATING.md` documenta la remocion manual
  (borrar entre marcadores). Motivo: el bloque es compartido por TODOS los
  proyectos con arnes de la maquina y `--reset` es por-proyecto — removerlo
  desde el proyecto A dejaria sin hooks al proyecto B. Es inofensivo sin
  arnes (el guard sale 0) y queda respaldado desde el primer install. Costo
  aceptado: un residuo benigno en maquinas que abandonan el arnes.
- DECIDIDO por el usuario (2026-07-28) — cuando escribe el instalador el
  bloque global: **(b) solo si detecta Kimi** en la maquina (`command -v
  kimi` o `${KIMI_CODE_HOME:-$HOME/.kimi-code}/bin/kimi` ejecutable), mas un
  flag `--no-kimi` para excluirlo explicitamente. Motivo: una escritura en
  `$HOME` solo se justifica si el backend existe realmente ahi. Los
  artefactos DE PROYECTO (`.kimi-code/agents/`, launcher) se generan
  siempre, como los de los demas backends.
- DECIDIDO por el usuario (2026-07-28) — frontmatter `tools` de los
  subagentes Kimi: **(a) allowlist por rol** con los nombres verificados en
  v0.29.2 (leader/reviewer: `Read, Grep, Glob, Bash`; implementer: +
  `Edit, Write`). Motivo: paridad con la separacion de roles que ya existe
  en Claude (el reviewer no puede escribir); los nombres estan verificados
  empiricamente y el gate de espejo no se ve afectado porque solo compara el
  cuerpo, no el frontmatter.
- Nota de diseno (lider, no bloquea): eleccion de eventos = `SessionStart` /
  `PostToolUse` (matcher `Edit|Write`) / `Stop`, mapeados a
  `session-start`/`post-tool`/`stop` de `bin/harness-hook` (los case patterns
  existentes ya aceptan los nombres de Kimi). `SessionEnd` NO se registra
  (run_event lo trataria como `stop` y el check correria dos veces por
  turno). Modo `plain` (no JSON): verificado que Kimi consume exit 2 + stderr
  del `Stop` reinyectandolo al agente, como Claude.
- Nota de diseno (lider, no bloquea): el `command` global no puede referenciar
  rutas de UN proyecto (es compartido): se ancla en `$PWD` (verificado: el
  hook corre con cwd = proyecto) con guard de existencia de
  `bin/harness-hook`. En cwd fuera de la raiz del proyecto (p.ej. dentro de
  un microservicio) el hook es no-op silencioso: mismo limite que ya tienen
  los hooks por-directorio de los demas backends.
