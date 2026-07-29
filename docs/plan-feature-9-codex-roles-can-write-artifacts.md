# Plan - Feature #9: codex_roles_can_write_artifacts

Estado: in_progress
Microservicios:
- harness

## Alcance

Corregir una asimetria no intencionada entre backends: el leader y el reviewer
de Codex se generan `read-only` pero sus roles exigen escribir spec, plan y
veredicto en `docs/`. Pasan a `workspace-write`, igualando lo que Claude ya
permite de hecho (leader y reviewer no tienen `Edit`/`Write` pero si `Bash`).

Spec: `docs/spec-feature-9-codex-roles-can-write-artifacts.md` (AC-1..AC-7),
con la investigacion empirica del 2026-07-28 sobre codex-cli 0.145.0 que fija
el terreno: sin allowlist de herramientas, sin via intermedia via
`writable_roots`, y `workspace-write` como default de Codex en carpetas
versionadas.

FUERA: allowlist de Claude/Kimi, `approval_policy`, `danger-full-access`,
cuerpos de `roles/*.md`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->
- Microservicio unico `harness`; sin contratos compartidos con otros
  microservicios (mismo resultado que en las features #5, #7 y #8: ningun
  microservicio registrado depende de `ADR/harness`).
- Radio interno:
  - `setup_harness.sh`: dos invocaciones de `build_codex_agent` (lineas 2092 y
    2094) y el comentario de cabecera del generador (2028-2030).
  - `setup_harness.ps1`: la expresion de sandbox por rol (linea 703).
  - `roles/README.md` + `templates/roles/README.md` (espejo modulo `__HREL__`,
    vigilado por el sub-gate de la feature #7).
  - `tests/setup_smoke.sh`: assert nuevo de `sandbox_mode`.
  - SIN cambios en: `rust/`, `harness_cli`, `harness_check.sh`, hooks,
    superficies, ni los otros backends.
- Instalaciones existentes: los `.toml` se regeneran al re-correr el
  instalador (no son documentos del usuario). Sin migracion manual.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->
- No se consulto el grafo: el radio de esta feature se determino leyendo
  directamente los dos generadores (`build_claude_agent` / `build_codex_agent`)
  y verificando el comportamiento del sandbox con `codex sandbox` sobre
  codex-cli 0.145.0 real. La evidencia empirica esta en el spec.

## Delegacion (implementer)

Orden: U1 -> U2 -> U3 -> U4. Cada unidad cita sus AC (Articulo 3). Regla
transversal: raiz y `templates/` espejados (Articulo 6); NUNCA correr
`setup_harness.sh` en este checkout.

- U1 [AC-1, AC-2, AC-3]: `setup_harness.sh` — `build_codex_agent leader` y
  `build_codex_agent reviewer` pasan de `read-only` a `workspace-write`; el
  comentario de cabecera del generador explica el porque (sin allowlist de
  tools, sin via intermedia verificada, disciplina por prompt como en Claude).
  Nada mas del `.toml` cambia.
- U2 [AC-4]: `roles/README.md` y `templates/roles/README.md` — actualizar el
  mapeo de capacidades por backend sin mentir sobre Claude (leader/reviewer sin
  `Edit`/`Write` pero con `Bash`; la disciplina es del prompt). Mantener la
  equivalencia modulo `__HREL__`.
- U3 [AC-5, AC-7]: `tests/setup_smoke.sh` — assert de
  `sandbox_mode = "workspace-write"` en los tres `.codex/agents/*.toml` de una
  fixture instalada, junto al parseo TOML existente (linea 159). Correr el
  smoke completo y los tres comandos oficiales.
- U4 [AC-6]: `setup_harness.ps1` — paridad de la linea 703. Sin `pwsh` en la
  maquina: verificacion estatica declarada como tal.

## Criterios de cierre (reviewer)

- Evidencia POR AC (AC-1..AC-7) en `docs/impl-9.md`; ningun AC sin evidencia.
- Los tres `.codex/agents/*.toml` de una fixture real declaran
  `workspace-write` y parsean como TOML valido; `developer_instructions` sigue
  siendo el cuerpo verbatim de `roles/<rol>.md` (el gate de espejo de la #7 no
  debe reportar nada).
- `roles/README.md` y `templates/roles/README.md` equivalentes modulo
  `__HREL__` (sub-gate de la #7 en verde).
- Comandos oficiales de `docs/verification.md` en verde.
- No regresion multi-LLM: Claude, Gemini, Grok, Kimi y Antigravity generan
  exactamente lo mismo que antes (las lineas `[Ok]` previas del smoke intactas).
- AC-6: sin `pwsh`, revision estatica registrada en `docs/review-9.md`.
- Commits Conventional sin trailers de IA.

## Riesgos

- **Ampliacion de permisos**: leader y reviewer de Codex pasan a poder escribir
  en todo el workspace. Es deliberado y aprobado por el usuario, y equipara a
  Codex con Claude (donde `Bash` ya lo permitia). Mitigacion: el prompt de los
  roles ya prohibe editar codigo fuente (`roles/reviewer.md:44`), el techo
  sigue siendo el workspace y `danger-full-access` no se usa.
- **Falsa sensacion de simetria**: alguien podria leer "workspace-write" y
  creer que Codex es mas laxo que Claude. Por eso AC-3 y AC-4 exigen dejar
  escrito que Claude permite lo mismo via `Bash`.
- **Dependencia de la version de Codex**: la ausencia de allowlist de
  herramientas es de codex-cli 0.145.0. Si una version futura la introduce,
  conviene revisar esta decision (anotado como pendiente, no bloquea).

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- DECIDIDO por el usuario (2026-07-28): `workspace-write` para los tres roles
  de Codex. Alternativas descartadas: dejar `read-only` (mutila leader y
  reviewer) y acotar la escritura a `docs/` (imposible con las primitivas
  actuales; `writable_roots` solo añade rutas, verificado empiricamente).

### Avance 2026-07-29T01:19:33Z
Feature #9 implementada: U1 sandbox_mode workspace-write en los tres roles Codex + comentario justificatorio, U2 roles/README.md y espejo, U3 assert de sandbox en el smoke, U4 paridad ps1

---
Cerrado: 2026-07-29T01:38:32Z - status=done - sandbox_mode workspace-write en los tres roles de Codex: leader y reviewer ya pueden escribir spec, plan y veredicto. AC-6 (pwsh) verificado estaticamente
