# Plan - Feature #10: layout_inferred_from_footprint

Estado: in_progress
Microservicios:
- harness

## Alcance

Cerrar el agujero que abrio la feature #7 al des-versionar `.harness_layout`:
toda instalacion existente que haga `git pull` pierde el marker (el commit
c8392f5 graba `D .harness_layout`) y pasa a tratar `harness_process/` como
raiz, en silencio.

Regla nueva: si el marker NO EXISTE y el padre tiene huella de instalacion, se
infiere layout subdir y la raiz es el padre, avisando con `[i]`. Un marker
explicito (`subdir` o `root`) se sigue respetando tal cual, y el guardrail de
checkout fuente de la #7 queda intacto.

Spec: `docs/spec-feature-10-layout-inferred-from-footprint.md` (AC-1..AC-13),
con la reproduccion del 2026-07-29 sobre el bloque de resolucion real.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->
- Microservicio unico `harness`; impacto externo nulo (ningun microservicio
  registrado depende de `ADR/harness`), igual que en las features #5, #7, #8
  y #9.
- Radio interno: EXACTAMENTE el mismo de la feature #7, porque se toca el mismo
  bloque de resolucion:
  - `harness_check.sh`, `harness_status.sh`, `init.sh`, `commit_guard.sh` +
    sus 4 espejos en `templates/` (identicos por `diff`, Articulo 6).
  - `rust/src/paths.rs::repo_root_from_marker` (punto unico; consumido por
    `HarnessPaths::from_root` y `GraphEnv::resolve`) + tests.
  - Tests: `tests/setup_smoke.sh`, `tests/setup_smoke.ps1`,
    `rust/src/paths.rs` (unit), `rust/tests/cli_basics.rs` (integracion).
  - Docs: `UPDATING.md` (+ template), `docs/architecture.md`.
  - SIN cambios en: `setup_harness.sh`/`.ps1` (el instalador ya escribe el
    marker en cada instalacion), `harness_cli`, `bin/harness-hook`, hooks,
    superficies, roles ni ningun backend.
- Instalaciones existentes (15 en esta maquina): se reparan SOLAS al
  actualizar; no requieren re-correr el instalador.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->
- `graphify-out/` esta presente y fresco. El radio de esta feature es
  identico al de la feature #7 (mismo bloque de resolucion en los mismos 4
  scripts + `paths.rs`), ya mapeado en `docs/plan-feature-7-*.md` con la
  consulta `"resolucion de REPO_ROOT y .harness_layout en scripts y rust"`
  (20 nodos), que confirmo `repo_root_from_marker` como unico punto Rust con
  dos consumidores. El reviewer de la feature #9 dejo anotado que conviene
  consultar el grafo en vez de justificar su omision: aqui se reutiliza esa
  consulta por ser el mismo radio, y el implementer debe re-ejecutarla si toca
  algun archivo fuera de la lista de Impacto.

## Delegacion (implementer)

Orden: U1 -> U2 -> U3 -> U4 -> U5. Cada unidad cita sus AC (Articulo 3). Regla
transversal: raiz y `templates/` espejados en el mismo commit; NUNCA correr
`setup_harness.sh` en este checkout; los scripts siguen siendo READ-ONLY (no
regeneran el marker).

- U1 [AC-1..AC-7, AC-9]: regla nueva en los 4 scripts y sus 4 espejos. Detalle
  clave: hoy la condicion es `[ "$(cat .harness_layout 2>/dev/null)" = "subdir" ]`,
  que confunde "archivo ausente" con "archivo con otro valor". Hay que
  distinguir los tres casos: (a) marker == `subdir` -> comportamiento actual,
  incluido el guardrail de checkout fuente de la #7 SIN cambios; (b) marker
  AUSENTE -> inferir por huella del padre (mismas 4 huellas y misma guarda de
  `$HOME` que el guardrail), con aviso `[i]` y remedio; (c) marker presente con
  cualquier otro valor (`root`) -> directorio del arnes, sin inferencia y sin
  aviso.
- U2 [AC-8]: misma regla en `rust/src/paths.rs::repo_root_from_marker`, con
  tests unitarios nuevos (sin marker + huella -> padre; sin marker + sin huella
  -> propio dir; marker `root` -> propio dir; guarda de `$HOME`) y, si aporta,
  integracion en `rust/tests/cli_basics.rs`.
- U3 [AC-10, AC-13]: bloques nuevos en `tests/setup_smoke.sh` con fixtures
  propias para los cuatro escenarios del AC-10, mas los tres comandos
  oficiales de `docs/verification.md` en verde.
- U4 [AC-11]: paridad `setup_harness.ps1` / `tests/setup_smoke.ps1`. Sin `pwsh`
  en la maquina: verificacion estatica declarada como tal. OJO: no romper los
  here-strings que arreglo la feature #7.
- U5 [AC-12]: `UPDATING.md` (raiz y template) y `docs/architecture.md` —
  corregir la nota de migracion de la #7, que hoy dice que hay que re-correr el
  instalador tras el `git pull`.

## Criterios de cierre (reviewer)

- Evidencia POR AC (AC-1..AC-13) en `docs/impl-10.md`.
- **Reproducir el bug ANTES y comprobar el fix DESPUES**: fixture subdir sin
  marker con huella en el padre; sin el fix resuelve al arnes, con el fix
  resuelve al proyecto y avisa. Sin esa prueba el veredicto no vale.
- AC-3 y AC-4 verificados explicitamente (marker `root` respetado; sin huella
  no infiere): son los que evitan que la inferencia se pase de lista.
- AC-7: el guardrail de la #7 sin regresion, verificado en ESTE checkout.
- `diff` de los 4 scripts raiz vs `templates/` = identicos.
- Comandos oficiales de `docs/verification.md` en verde; sin regresion
  multi-LLM (lineas `[Ok]` previas del smoke intactas).
- AC-11: sin `pwsh`, revision estatica registrada en `docs/review-10.md`.
- Commits Conventional sin trailers de IA.

## Riesgos

- **Inferencia demasiado agresiva**: un `harness_process/` colocado dentro de
  un directorio que casualmente tenga `CLAUDE.md` o `AGENTS.md` resolveria al
  padre. Es el mismo criterio de huella que ya usa el guardrail de la #7 desde
  ayer, `HARNESS_REPO_ROOT` sigue como override, y el aviso `[i]` deja rastro.
  Riesgo aceptado y acotado.
- **Confundir ausencia con valor distinto**: si la implementacion no distingue
  bien los tres casos, un marker `root` podria terminar infiriendo subdir. AC-3
  existe para eso.
- **Ruido de avisos**: el `[i]` se emite en cada invocacion de cada script y
  del binario. Es el mismo patron aceptado en la #7 (decision 4); si molesta,
  bajarle el volumen seria una decision nueva del usuario.
- **Divergencia sh/Rust**: si la regla se implementa distinto en `paths.rs` que
  en los scripts, el binario y los hooks resolverian raices distintas. AC-8 y
  los tests lo cubren.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

U0 CERRADA: las 2 decisiones se le preguntaron a Alan el 2026-07-29 y estan
registradas aqui y en las Observaciones del spec. El implementer NO debe
volver a preguntarlas.

- Decision 1: arreglar por **inferencia por huella del padre**, no
  re-versionando `.harness_layout`. DECIDIDO por el usuario (2026-07-29).
- Decision 2: cuando el layout se infiere, **aviso `[i]` discreto** (no
  silencioso), con el remedio (re-correr el instalador regenera el marker).
  DECIDIDO por el usuario (2026-07-29).

### Avance 2026-07-29T04:21:02Z
Re-sincronizado con plan actualizado por otro agente (feature #10, U1..U5)
