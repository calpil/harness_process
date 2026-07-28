# Plan - Feature #7: harness_check_robustness

Estado: in_progress
Microservicios:
- harness

## Alcance

Una sola feature, dos problemas de la misma superficie (decision del usuario,
2026-07-28; no re-litigar el empaquetado):

- **A. Gate de espejo `roles/` -> agentes** en `harness_check.sh`: comparar el
  cuerpo embebido de `.claude/agents/*.md` (tambien leidos por Grok),
  `.gemini/agents/*.md` y `.codex/agents/*.toml` contra `roles/<rol>.md`
  (fuente unica), y `roles/*.md` contra `templates/roles/*.md` modulo
  `__HREL__`. Origen: hallazgo 1 de `docs/review-6.md` (espejos stale desde la
  feature #3, descubiertos por casualidad).
- **B. Resolucion de `REPO_ROOT` robusta ante el checkout fuente**: hoy el
  marker `.harness_layout` = `subdir` versionado hace que el checkout fuente
  resuelva su raiz a `$HOME` (check con falsos fallos + basura en
  `$HOME/docs` creada por `start`). El fix cubre TODOS los duplicados del
  patron: `harness_check.sh`, `harness_status.sh`, `init.sh`,
  `commit_guard.sh`, sus 4 espejos en `templates/`, y
  `rust/src/paths.rs::repo_root_from_marker` (unico punto Rust; consumido por
  `HarnessPaths::from_root` y `GraphEnv::resolve`). Arreglar uno solo no
  sirve.

Spec: `docs/spec-feature-7-harness-check-robustness.md` (AC-1..AC-13).

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->
- `sh harness_cli graph impacto --microservicio ADR/harness` (2026-07-28):
  "Ningun microservicio registrado depende de 'ADR/harness'". Impacto externo
  nulo; el radio es interno al repo del arnes.
- Radio interno (contratos compartidos del propio arnes):
  - Scripts sh con el patron duplicado: `harness_check.sh`,
    `harness_status.sh`, `init.sh`, `commit_guard.sh` + espejos identicos en
    `templates/` (regla de mantenedor del Articulo 6).
  - Rust: `rust/src/paths.rs` (funcion unica `repo_root_from_marker`; via
    `HarnessPaths` afecta a `start`/`advance`/`close`/`check-*` y via
    `GraphEnv` a `graph *`).
  - Instaladores: `setup_harness.sh` / `setup_harness.ps1` (estado del marker
    y posibles guardas; escriben el marker en cada instalacion).
  - Estado versionado del repo fuente: `.harness_layout`, `.gitignore`
    (segun decision pendiente 3).
  - Tests: `tests/setup_smoke.sh`, `tests/setup_smoke.ps1`,
    `rust/src/paths.rs` (unit), `rust/tests/cli_basics.rs` (integracion).
  - Docs: `README.md`, `UPDATING.md` (+ template), `AGENTS.md`,
    `docs/architecture.md`.
- Instalaciones subdir EXISTENTES en otros proyectos: cero cambio de
  comportamiento (AC-9); si la decision 3 toca el marker versionado, hay una
  ventana benigna pull -> re-setup documentada (AC-10).

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->
- `graphify query "resolucion de REPO_ROOT y .harness_layout en scripts y
  rust"` (20 nodos): confirma que `repo_root_from_marker` es la UNICA funcion
  Rust del marker, con dos consumidores (`HarnessPaths::from_root`,
  `GraphEnv::resolve`) y tests unitarios existentes en `paths.rs:L76-L95`.
  El fix Rust es un solo punto + sus tests.

## Delegacion (implementer)

Orden: U0 -> U1 -> U2 -> U3 -> U4 -> U5 -> U6 -> U7 -> U8. Cada unidad cita
sus AC (Articulo 3).

- U0 (gate previo, Articulo 5; sin AC propio): preguntar al USUARIO las 4
  observaciones PENDIENTE DE DECISION del spec y registrar las respuestas en
  el spec y este plan ANTES de tocar codigo. Ninguna unidad se implementa con
  una decision pendiente que la afecte.
- U1 [AC-6, AC-7, AC-9]: resolucion robusta de `REPO_ROOT` en los 4 scripts:
  `harness_check.sh`, `harness_status.sh`, `init.sh`, `commit_guard.sh` y sus
  espejos `templates/harness_check.sh`, `templates/harness_status.sh`,
  `templates/init.sh`, `templates/commit_guard.sh` (bloques identicos por
  `diff`). Senal de checkout fuente segun la nota de diseno del spec;
  comportamiento ante incoherencia segun decision 4. Precedencia de env vars
  intacta.
- U2 [AC-8, AC-9]: misma regla en `rust/src/paths.rs`
  (`repo_root_from_marker`), con tests unitarios nuevos en `paths.rs` (marker
  subdir sin huella en el padre -> root propio; marker subdir con huella ->
  padre; override env intacto) y test de integracion en
  `rust/tests/cli_basics.rs` que verifique que `start` no escribe fuera del
  checkout simulado.
- U3 [AC-1, AC-2, AC-3, AC-5]: gate de espejo roles -> agentes en
  `harness_check.sh` + `templates/harness_check.sh`: extraccion del cuerpo por
  formato (Claude/Gemini: lo que sigue al segundo `---`, normalizando lineas
  en blanco iniciales; Codex: bloque `developer_instructions` entre comillas
  triples), comparacion contra `roles/<rol>.md`, mensajes accionables con
  archivo y remedio, respeto de `HARNESS_CHECK_MODE` y de la condicionalidad
  por existencia (espejo ausente no falla). Severidad segun decision 1;
  remediacion segun decision 2.
- U4 [AC-4]: sub-gate `roles/*.md` <-> `templates/roles/*.md` modulo
  `__HREL__` (valido si coincide con ALGUNA de las dos expansiones: prefijo
  `<basename del dir del arnes>/` o vacio), condicional a que
  `templates/roles/` exista (distribucion aplanada lo omite). Mismo mensaje
  accionable y modo que U3.
- U5 [AC-10, parte de AC-13]: estado del marker en el repo fuente segun
  decision 3 (`.harness_layout`, `.gitignore` si aplica) + nota de migracion
  pull -> re-setup en `UPDATING.md` y `templates/UPDATING.md`.
- U6 [AC-12]: bloques nuevos en `tests/setup_smoke.sh`: (a) `harness_check.sh`
  ejecutado y limpio en una fixture recien instalada; (b) espejo stale
  inyectado en esa fixture -> el check lo reporta y falla en modo block; (c)
  fixture "checkout fuente" simulada (clon con `templates/harness_cli` +
  `rust/` y padre sin huella, `HOME` de fixture) -> resolucion local, check
  sin el falso `Falta docs/constitution.md`, y `$HOME` de la fixture intacto
  tras `start`.
- U7 [AC-11]: paridad Windows: replicar en `setup_harness.ps1` lo que U5 (y
  U1, si toca al instalador) cambie del lado sh, y en `tests/setup_smoke.ps1`
  los bloques de U6. Sin `pwsh` en esta maquina: verificacion estatica
  documentada (precedente #1/#4/#5/#6).
- U8 [AC-13]: docs: `README.md`, `AGENTS.md`, `docs/architecture.md`
  (describir el gate de espejo dentro de `harness_check.sh` y la resolucion
  de raiz robusta; `UPDATING.md` ya en U5).

## Criterios de cierre (reviewer)

- Evidencia POR AC (AC-1..AC-13) en `docs/impl-7.md`; ningun AC sin evidencia
  (Articulo 3).
- `diff` de los 4 scripts raiz vs `templates/` = identicos, y
  `harness_check.sh` raiz vs `templates/harness_check.sh` incluye el gate
  nuevo en ambos (Articulo 6).
- Comandos oficiales de `docs/verification.md` en verde: `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` (AC-12).
- Dogfooding en ESTE checkout: `env -u HARNESS_REPO_ROOT bash
  harness_check.sh` resuelve local (sin el falso `Falta docs/constitution.md`
  ni rutas fuera del repo) [AC-6] y el gate de espejo pasa con los
  `.claude/agents/*.md` versionados actuales [AC-2].
- Prueba negativa reproducida (via smoke o manual): espejo alterado -> el
  check falla nombrando el archivo [AC-1, AC-3].
- AC-11: sin `pwsh`, revision estatica linea a linea de los pares `.ps1`
  registrada en `docs/review-7.md`.
- Las 4 decisiones de Observaciones registradas con la respuesta del usuario;
  ninguna implementacion las contradice (Articulo 5).
- Commits Conventional sin trailers de IA (`commit_guard.sh`; regla del
  Articulo 6 / UPDATING.md).

## Riesgos

- Falsos positivos del gate en instalaciones generadas por instaladores
  viejos: el frontmatter viejo (descripcion/modelo distintos) NO afecta
  porque solo se compara el cuerpo; un cuerpo realmente viejo SI fallara — es
  el proposito, y el mensaje debe decir que el remedio es re-correr el setup.
- Portabilidad `awk`/`sed`/`diff` BSD vs GNU (macOS / Linux / Git Bash en
  Windows): usar construcciones POSIX ya presentes en el repo; el smoke es la
  red de seguridad.
- Ventana pull -> re-setup si la decision 3 toca el marker versionado:
  instalaciones subdir existentes quedan sin marker (o con `root`) hasta
  re-correr el setup. Efectos acotados y benignos (los checks condicionales no
  fallan; `start` escribiria dentro del clon, no en el proyecto ni en $HOME);
  mitigacion: nota de migracion en `UPDATING.md` (el flujo canonico ya es
  pull -> setup).
- Heuristica de huella: un `CLAUDE.md` casual en el padre haria resolver al
  padre (como hoy); la regla `$HOME` cubre el caso mas probable (checkout
  suelto bajo `$HOME` con `~/CLAUDE.md`), y `HARNESS_REPO_ROOT` siempre esta
  como override. Riesgo residual bajo, documentado en la nota de diseno del
  spec.
- El extractor del bloque Codex asume el envoltorio FIJO que genera
  `build_codex_agent`; si un instalador futuro cambia el formato, el gate debe
  actualizarse en el mismo commit (la regla de espejo raiz/templates del
  Articulo 6 lo fuerza via `templates/harness_check.sh`).
- Firmas de frescura: editar spec/plan de esta feature invalida
  `last_plan_sig`/`last_spec_sig` hasta el `approve-spec` (spec) y el
  `advance` (plan) del flujo normal; no trabajar con `check-plan`/`check-spec`
  en rojo sin re-leer.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
U0 CERRADA: las 4 decisiones se le preguntaron a Alan en el chat el 2026-07-28
y quedaron registradas aqui y en las Observaciones del spec (que ya esta
`approved` y sellado). El implementer NO debe volver a preguntarlas.

- Decision 1 (afecta U3/U4): severidad del gate de espejo — **(a) BLOQUEA como
  los demas checks**. DECIDIDO por el usuario (2026-07-28). Suma `failures` y
  sale 2 bajo `HARNESS_CHECK_MODE=block`; `warn` reporta y sale 0; `off` no
  evalua.
- Decision 2 (afecta U3): remediacion — **(a) el check SOLO reporta**, remedio
  = re-correr el instalador. DECIDIDO por el usuario (2026-07-28).
  `harness_check.sh` permanece read-only: no regenera ni reescribe espejos.
- Decision 3 (afecta U5 y AC-10): marker `.harness_layout` versionado del repo
  fuente — **(a) des-versionar** (`git rm --cached .harness_layout`) y agregar
  a `.gitignore`. DECIDIDO por el usuario (2026-07-28). Ventana benigna pull ->
  re-setup documentada en `UPDATING.md` (AC-10).
- Decision 4 (afecta U1/U2): ante incoherencia marker-vs-entorno — **(a)
  fallback a `HARNESS_DIR` con aviso informativo `[i]`**. DECIDIDO por el
  usuario (2026-07-28). Nada de fallo duro ni de fallback silencioso.

### Avance 2026-07-28T18:11:34Z
Re-sincronizado con plan actualizado por otro agente (U0 cerrada, 4 decisiones registradas); inicio U1

### Avance 2026-07-28T18:25:09Z
F7 U1-U5: guardrail checkout fuente en 4 scripts + espejos, gate de espejo roles/agentes + sub-gate templates en harness_check, regla en paths.rs con tests (44u+22i verdes, clippy limpio), marker des-versionado + gitignore + UPDATING (raiz y template)

### Avance 2026-07-28T18:29:41Z
F7 U6: smoke con check limpio post-install, stale inyectado en Claude/Gemini/Codex + drift templates detectados (block/warn/off), y checkout fuente simulado con resolucion local sin escrituras fuera del clon; setup_smoke.sh rc=0

### Avance 2026-07-28T18:35:17Z
F7 U7: paridad ps1 - bloques del gate de espejo y checkout fuente en tests/setup_smoke.ps1 (extraccion portada + check real via bash si existe) + fix here-strings rotos preexistentes (apertura $fakeCargo perdida en feature #2); revision estatica, sin pwsh en la maquina

### Avance 2026-07-28T18:40:47Z
F7 U8 + evidencia: docs (README/AGENTS/architecture) y docs/impl-7.md con evidencia AC-1..AC-13; verificacion final: cargo test 44u+22i ok, clippy -D warnings ok, setup_smoke.sh rc=0, dogfooding check limpio con [i] y AC-10 via checkout-index

---
Cerrado: 2026-07-28T18:58:03Z - status=done - Gate de espejo roles->agentes (bloquea, read-only) + resolucion robusta de REPO_ROOT en 4 scripts, sus espejos y paths.rs; marker des-versionado. AC-11 (pwsh) verificado estaticamente
