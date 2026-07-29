# Impl - Feature #9: codex_roles_can_write_artifacts

Spec: docs/spec-feature-9-codex-roles-can-write-artifacts.md (Estado: approved, sellado)
Plan: docs/plan-feature-9-codex-roles-can-write-artifacts.md

> Nota de proceso: esta feature la implemento el LIDER de la sesion (no un
> implementer delegado), por decision explicita: la investigacion empirica que
> la origino (sandbox de Codex, `writable_roots`, doc de subagentes) ya se
> habia hecho en el chat, y el cambio funcional son 3 lineas. El veredicto SI
> se delego a un reviewer independiente, que es donde la separacion aporta.

## Que cambio

El leader y el reviewer de Codex pasan de `sandbox_mode = "read-only"` a
`"workspace-write"`, igual que el implementer. Sus roles exigen escribir
`docs/spec-*.md`, `docs/plan-*.md` y `docs/review-*.md`, y bajo `read-only` el
sandbox lo impide (`Operation not permitted`).

No es una relajacion frente a Claude: alli esos mismos roles ya escriben esos
archivos con `Bash`, que su allowlist incluye. La asimetria previa no fue una
decision de diseno, sino un efecto de que Codex no ofrezca allowlist de
herramientas.

## Unidades

| Unidad | AC | Archivos |
| --- | --- | --- |
| U1 sandbox + justificacion | AC-1, AC-2, AC-3 | `setup_harness.sh` (invocaciones de `build_codex_agent`, comentario de cabecera del generador) |
| U2 documentacion de roles | AC-4 | `roles/README.md`, `templates/roles/README.md` |
| U3 test | AC-5, AC-7 | `tests/setup_smoke.sh` |
| U4 paridad Windows | AC-6 | `setup_harness.ps1` |

## Evidencia por AC

- **AC-1**: `setup_harness.sh` — las tres invocaciones quedan
  `build_codex_agent {leader,implementer,reviewer} workspace-write high ...`.
  Diff funcional exacto (sin comentarios): dos lineas, `leader` y `reviewer`,
  de `read-only` a `workspace-write`. Validado en instalacion real por el smoke
  (ver AC-5), que ademas parsea los tres `.toml` con `tomllib`.
- **AC-2**: el generador `build_codex_agent` NO se modifico en su cuerpo: sigue
  emitiendo `name`, `description`, `sandbox_mode`, `model_reasoning_effort` y
  `developer_instructions` con `cat "roles/$role.md"` verbatim. Lo unico que
  cambio es el VALOR del segundo argumento en dos llamadas. El gate de espejo
  de la feature #7 sigue limpio (`bash harness_check.sh` rc=0), lo que confirma
  que el cuerpo embebido sigue coincidiendo con `roles/<rol>.md`.
- **AC-3**: comentario de cabecera del generador reescrito. Deja por escrito:
  (a) que el formato no admite allowlist de herramientas, citando la doc
  ("subagents use the tools available to the parent chat"); (b) por que los
  TRES roles usan `workspace-write`; (c) que no es mas laxo que Claude, donde
  `Bash` ya permite escribir; (d) que `writable_roots` no ofrece via intermedia
  (verificado); (e) que `danger-full-access` no se usa nunca.
- **AC-4**: `roles/README.md` — la entrada de Codex explica la ausencia de
  allowlist y el `workspace-write` de los tres roles. Se agrego ademas un
  parrafo **"Sobre solo lectura"** que corrige la imprecision que existia para
  TODOS los backends: leader y reviewer de Claude y Kimi no tienen `Edit` ni
  `Write` pero si `Bash`, y la disciplina de rol la sostiene el prompt, no la
  configuracion. Se quito el rotulo `(read-only)` de la entrada de Kimi por ser
  enganoso por el mismo motivo (cambio de texto, no de configuracion: la
  allowlist de Kimi no se toco, sigue fuera de alcance). `templates/roles/README.md`
  espejado: `diff` modulo `__HREL__` equivalente, y el sub-gate de la feature #7
  pasa.
- **AC-5**: bloque nuevo en `tests/setup_smoke.sh`, junto al parseo TOML
  existente del fixture `root-layout`. Verifica sobre el TOML **parseado** (no
  por `grep`, para que un cambio de formato no lo falsee) que los tres roles
  existen y que los tres declaran `workspace-write`.
  **Prueba negativa del propio assert** (fixture aparte en scratchpad, tres
  `.toml` sinteticos):
  - los tres en `workspace-write` -> rc=0;
  - `reviewer` en `read-only` -> rc=1,
    `[FALLO] sandbox_mode != workspace-write en Codex: {'reviewer': 'read-only'}`;
  - falta `leader.toml` -> rc=1, `[FALLO] faltan agentes Codex: ['leader']`.
- **AC-6**: `setup_harness.ps1` — la expresion condicional por rol se sustituye
  por `$sandbox = "workspace-write"` con el mismo comentario justificatorio que
  el `.sh`. **PARCIAL**: no hay `pwsh` ni `powershell` en esta maquina, asi que
  la verificacion es estatica (la linea es una asignacion literal dentro del
  mismo bucle `foreach ($role ...)` que ya existia; no se tocaron here-strings
  ni llaves). Misma limitacion aceptada en las features #1, #4, #5, #6, #7 y #8.
- **AC-7**: comandos oficiales de `docs/verification.md`, re-ejecutados tras el
  cambio:
  - `cargo test`: **44 unit + 22 integracion, 0 fallos** (identico a antes;
    `rust/` no se toco);
  - `cargo clippy --all-targets -- -D warnings`: rc=0;
  - `bash tests/setup_smoke.sh`: **rc=0**, con TODAS las lineas `[Ok]` previas
    presentes (gate de espejo, checkout fuente, Kimi Code, docs en la raiz,
    planillas PRD, smoke general) — sin regresion multi-LLM.

## Notas para el reviewer

- El cambio funcional son 3 lineas (`setup_harness.sh` x2, `setup_harness.ps1`
  x1). Todo lo demas es comentario, documentacion y test.
- La evidencia empirica que motiva la feature esta en el spec, seccion
  "Problema" y "Terreno verificado", con los comandos `codex sandbox`
  reproducibles sobre codex-cli 0.145.0. Se pueden re-ejecutar sin consumir
  cuota de modelo: `codex sandbox` corre bajo seatbelt sin invocar al LLM.
- El `advance` de la feature se registro DESPUES de implementar (re-firma el
  plan, que se habia editado tras el `start`): por eso `check-plan` estuvo
  temporalmente en rojo y ahora sale limpio.
- Punto de atencion sugerido: verificar que el rotulo `(read-only)` retirado de
  la entrada de Kimi en `roles/README.md` sea el unico lugar del repo donde esa
  imprecision quedaba escrita.

## OBSERVACION SIN DECISION

Ninguna. La unica decision de la feature (workspace-write para los tres roles)
la tomo el usuario en el chat el 2026-07-28 y esta registrada en las
Observaciones del spec y del plan.
