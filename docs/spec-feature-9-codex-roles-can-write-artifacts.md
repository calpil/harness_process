# Spec - Feature #9: codex_roles_can_write_artifacts

Estado: approved
Aprobado: 2026-07-29T01:16:57Z por USUARIO (confirmacion explicita) - Alan aprobo el spec #9 en el chat (2026-07-28): workspace-write para los tres roles de Codex, decidido tras ver la evidencia del sandbox
Plan: docs/plan-feature-9-codex-roles-can-write-artifacts.md
Constitution: docs/constitution.md

## Problema

El leader y el reviewer de Codex se generan con `sandbox_mode = "read-only"`
(`setup_harness.sh:2092` y `:2094`), pero sus roles consisten precisamente en
ESCRIBIR archivos:

- `roles/leader.md:17,35`: persistir `docs/spec-feature-<id>-<slug>.md` y
  `docs/plan-feature-<id>-<slug>.md` en el `docs/` de la raiz. El propio rol
  declara que "una respuesta corta en chat no reemplaza el spec ni el plan
  persistidos" (`roles/leader.md:65`).
- `roles/reviewer.md:33`: escribir el veredicto en
  `docs/review-<feature>.md`.

Bajo el sandbox de Codex esa escritura no ocurre. Verificado empiricamente el
2026-07-28 con el subcomando `codex sandbox` de codex-cli 0.145.0 (prueba del
sandbox puro, sin consumir cuota de modelo):

```
$ codex sandbox -c sandbox_mode='"read-only"' -- sh -c 'echo "# veredicto" > review-9.md'
sh: review-9.md: Operation not permitted     -> el archivo NO se crea

$ codex sandbox -c sandbox_mode='"workspace-write"' -- sh -c 'echo "# veredicto" > review-9.md'
                                             -> el archivo se crea
```

Consecuencia: dos de los tres roles del arnes no pueden entregar su trabajo en
Codex. En sesion interactiva la capa de `approval_policy` de Codex puede
escalar cada escritura como pregunta al usuario (el arnes no fija
`approval_policy` en ningun lado, verificado); en `codex exec` (no
interactivo) la escritura simplemente falla.

## Por que en Claude si funciona

Los mismos dos roles en Claude declaran `tools: Read, Grep, Glob, Bash`
(`setup_harness.sh:2087,2089`). No tienen `Edit` ni `Write`, pero **si tienen
`Bash`**, y con Bash se escribe un archivo sin restriccion alguna. Su "solo
lectura" es nominal: la separacion real de roles la sostiene el PROMPT
(`roles/reviewer.md:44` — "Solo lectura mas ejecucion de validaciones. No
edites codigo fuente"), no el conjunto de herramientas. Evidencia directa: los
reviewers de las features #7 y #8 escribieron `docs/review-7.md` y
`docs/review-8.md` con Bash, operando bajo esa misma allowlist.

Es decir: hoy Codex es MAS restrictivo que Claude para el mismo rol, y esa
asimetria no fue una decision de diseno sino un efecto colateral de que Codex
no ofrezca allowlist de herramientas.

## Terreno verificado (2026-07-28, codex-cli 0.145.0 + doc oficial)

1. El formato `.codex/agents/*.toml` **no admite restringir herramientas**. Los
   campos soportados son `name`, `description`, `developer_instructions` y
   claves de `config.toml` como `model`, `model_reasoning_effort`,
   `sandbox_mode`, `mcp_servers` y `skills.config`. La doc es explicita:
   "Subagents use the tools available to the parent chat".
2. **No se puede acotar la escritura a un subdirectorio**: probado
   `sandbox_workspace_write.writable_roots=["<dir>/docs"]` con
   `sandbox_mode = "workspace-write"` — el sandbox igual permitio escribir en
   `<dir>/src/main.rs`. `writable_roots` AÑADE rutas escribibles fuera del
   workspace; no restringe las de dentro. No hay via intermedia entre "no
   escribe nada" y "escribe todo el workspace".
3. Valores validos de `sandbox_mode`: `read-only`, `workspace-write`,
   `danger-full-access`. `workspace-write` es ademas el default de Codex en
   carpetas versionadas, por lo que declararlo explicitamente aporta
   determinismo, no permisos extra.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario de Codex CLI, quiero que el subagente `leader` persista el
  spec y el plan en `docs/`, para poder correr la fase de planificacion del
  arnes en Codex igual que en Claude.
- P1: Como usuario de Codex CLI, quiero que el subagente `reviewer` escriba su
  veredicto en `docs/review-<feature>.md`, para cerrar features con evidencia
  durable sin cambiar de backend.
- P1: Como responsable del repo, quiero que la separacion de roles siga
  existiendo (el reviewer no modifica codigo fuente) sostenida por el prompt,
  igual que en Claude, para no perder la garantia que ya teniamos.
- P2: Como usuario de Windows, quiero `setup_harness.ps1` en paridad exacta
  con `setup_harness.sh`.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given una instalacion con subagentes, When corre `setup_harness.sh`,
  Then `.codex/agents/leader.toml` y `.codex/agents/reviewer.toml` declaran
  `sandbox_mode = "workspace-write"` (igual que `implementer.toml`), y los tres
  archivos siguen siendo TOML valido parseable.
- AC-2: Given los tres `.toml` generados, When los leo, Then conservan sin
  cambios `name`, `description`, `model_reasoning_effort = "high"` y el
  `developer_instructions` con el cuerpo VERBATIM de `roles/<rol>.md` — esta
  feature solo toca `sandbox_mode`.
- AC-3: Given el generador `build_codex_agent`, When leo su comentario de
  cabecera (`setup_harness.sh:2028-2030`), Then explica por que los tres roles
  usan `workspace-write`: Codex no ofrece allowlist de herramientas, no existe
  via intermedia (verificado con `writable_roots`), y la separacion de roles la
  sostiene el prompt igual que en Claude, donde `Bash` ya permite escribir.
- AC-4: Given `roles/README.md` y su espejo `templates/roles/README.md`
  (linea 88 y alrededores), When describo el mapeo de capacidades por backend,
  Then reflejan el estado nuevo sin mentir sobre Claude: en Codex los tres
  roles son `workspace-write`; en Claude leader y reviewer no tienen
  `Edit`/`Write` pero si `Bash`; en ambos la disciplina de rol es del prompt.
  Los dos archivos siguen equivalentes modulo `__HREL__` (sub-gate de la #7).
- AC-5: Given `tests/setup_smoke.sh`, When corre, Then verifica sobre una
  fixture instalada que los tres `.codex/agents/*.toml` declaran
  `sandbox_mode = "workspace-write"` (assert nuevo, junto al parseo TOML ya
  existente de la linea 159), y `bash tests/setup_smoke.sh` sale 0.
- AC-6: Given `setup_harness.ps1:703`, When se compara con su par `.sh`, Then
  produce el mismo `sandbox_mode` para los tres roles; sin `pwsh`/`powershell`
  en la maquina se verifica estaticamente, como en las features #1, #4, #5,
  #6, #7 y #8.
- AC-7: Given el repo, When corro los comandos oficiales de
  `docs/verification.md` (`cargo test`, `cargo clippy -- -D warnings`,
  `bash tests/setup_smoke.sh`), Then los tres pasan. El binario Rust no se
  toca.

## No funcionales
- SLOs: cambio de configuracion; sin impacto en tiempos de instalacion ni
  dependencias nuevas (Articulo 6).
- Seguridad: es una AMPLIACION deliberada de permisos para dos roles de UN
  backend, hasta igualar lo que Claude ya permite de hecho via `Bash`. El
  limite superior sigue siendo el workspace (`danger-full-access` NO se usa) y
  la disciplina de rol la sostiene el prompt, que ya prohibe editar codigo
  fuente. Queda documentado por que no existe una opcion intermedia.
- Observabilidad: sin cambios de exit codes ni de mensajes del instalador.
- Multi-LLM: no altera Claude, Gemini, Grok, Kimi ni Antigravity.

## Fuera de alcance
- Cambiar la allowlist de Claude o de Kimi (`tools:`): esta feature no toca
  esos backends.
- Fijar `approval_policy` en la configuracion de Codex: es una decision de
  politica del usuario, ortogonal a que el rol pueda escribir.
- `danger-full-access` en cualquier rol.
- Restringir la escritura a `docs/`: verificado como imposible con las
  primitivas actuales de Codex (ver "Terreno verificado", punto 2).
- Cambiar los cuerpos de `roles/*.md`.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- DECIDIDO por el usuario (2026-07-28): aplicar `workspace-write` a los tres
  roles de Codex, en lugar de dejar `read-only` (que mutila leader y reviewer)
  o de intentar acotar la escritura a `docs/` (imposible con las primitivas
  actuales de Codex, verificado). Motivo: replica lo que Claude permite de
  hecho y desbloquea el ciclo completo en Codex.
