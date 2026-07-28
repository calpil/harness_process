# Spec - Feature #7: harness_check_robustness

Estado: approved
Aprobado: 2026-07-28T18:10:02Z por USUARIO (confirmacion explicita) - Alan aprobo el spec #7 en el chat (2026-07-28) tras revisarlo, con las 4 decisiones de Observaciones ya registradas
Plan: docs/plan-feature-7-harness-check-robustness.md
Constitution: docs/constitution.md

## Problema

Dos fallas de robustez descubiertas en la practica, sobre la misma superficie
(`harness_check.sh` y la resolucion de rutas que comparte con el resto del
arnes). El usuario decidio empaquetarlas en una sola feature (2026-07-28).

1. **Espejos de agentes sin gate de sincronia.** `roles/*.md` es la fuente
   unica; el instalador genera espejos con el cuerpo del rol verbatim mas un
   envoltorio fijo (`.claude/agents/*.md` frontmatter de 5 campos,
   `.gemini/agents/*.md` frontmatter de 2 campos, `.codex/agents/*.toml`
   bloque `developer_instructions = '''...'''`; funciones `build_*_agent` de
   `setup_harness.sh`). `harness_check.sh` hoy solo valida ESTRUCTURA
   (frontmatter presente, `name:`/`description:`), nunca contenido: los
   `.claude/agents/*.md` estuvieron stale desde la feature #3 hasta la #6 sin
   que nada lo detectara (hallazgo 1 de `docs/review-6.md`). Grok lee esos
   mismos espejos: un espejo stale desprotege a mas de un backend. Ademas
   `templates/roles/*.md` (fuente de distribucion, con placeholder `__HREL__`)
   puede divergir de `roles/*.md` sin gate, y una divergencia ahi se propaga a
   TODAS las instalaciones nuevas.

2. **`REPO_ROOT` resuelve a `$HOME` en el checkout fuente.** `.harness_layout`
   con valor `subdir` esta VERSIONADO en el repo fuente (desde ade8a34), asi
   que todo clon nace declarando "mi raiz es mi padre". El patron
   marker->padre esta duplicado en `harness_check.sh:4-13`,
   `harness_status.sh`, `init.sh`, `commit_guard.sh`, sus 4 espejos en
   `templates/`, y en `rust/src/paths.rs::repo_root_from_marker` (consumida
   por `HarnessPaths::from_root` y `GraphEnv::resolve`, es decir por TODO el
   binario). En el checkout fuente bajo `$HOME` eso produce: harness_check
   reportando `[!] Falta docs/constitution.md` aunque existe, y
   `harness_cli start` creando planes-plantilla huerfanos en `$HOME/docs`
   (basura real, borrada a mano el 2026-07-28). Una instalacion subdir es un
   clon del repo fuente dentro de un proyecto: el contenido es identico, la
   distincion solo puede venir del ENTORNO (huella de instalacion en el padre)
   y/o de corregir el estado del marker versionado.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como mantenedor del arnes, quiero que `harness_check.sh` falle cuando
  cualquier espejo de roles quedo desincronizado de `roles/`, para no repetir
  el incidente de la feature #3 (agentes operando meses con protocolo viejo
  sin que nadie lo note).
- P1: Como mantenedor trabajando en el checkout fuente, quiero que
  `harness_check.sh`, los scripts hermanos y el binario evaluen contra el
  propio checkout sin exportar variables, para obtener resultados verdaderos
  y cero basura fuera del repo.
- P1: Como usuario de una instalacion subdir real, quiero que nada cambie para
  mi: la raiz sigue siendo el padre del arnes y los overrides
  (`HARNESS_REPO_ROOT`, `CLAUDE_PROJECT_DIR`, ...) siguen mandando.
- P2: Como equipo multi-LLM, quiero que el gate cubra tambien los espejos de
  Gemini y Codex cuando existen, para que ningun backend opere con un rol
  stale.
- P2: Como usuario de Windows, quiero que `setup_harness.ps1` y
  `tests/setup_smoke.ps1` queden en paridad exacta con sus pares `.sh`.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given una instalacion (o el checkout fuente) donde el cuerpo de
  `.claude/agents/<rol>.md` (todo lo que sigue al cierre del frontmatter)
  difiere del contenido de `roles/<rol>.md` para algun rol
  {leader, implementer, reviewer}, When corro `bash harness_check.sh`, Then el
  check reporta el rol y el archivo espejo desincronizado con mensaje
  accionable y sale 2 bajo `HARNESS_CHECK_MODE=block` (default).
- AC-2: Given los espejos tal como los genera el instalador vigente (unica
  diferencia con `roles/<rol>.md`: una linea en blanco entre el frontmatter y
  el cuerpo), When corro `bash harness_check.sh`, Then el gate de espejo NO
  reporta desincronizacion (la comparacion normaliza las lineas en blanco
  iniciales del cuerpo): cero falsos positivos desde el dia uno, verificable
  en este mismo repo.
- AC-3: Given `.gemini/agents/<rol>.md` o `.codex/agents/<rol>.toml` presentes
  con cuerpo embebido distinto de `roles/<rol>.md`, When corro
  `bash harness_check.sh`, Then tambien se reportan como stale (extraccion por
  formato: cuerpo tras el frontmatter en Gemini; bloque
  `developer_instructions` entre comillas triples en Codex); y Given que esos
  archivos NO existen (checkout fuente sin `.codex/`/`.gemini/`, o instalacion
  `--no-subagents`), Then su ausencia NO falla (se preserva la condicionalidad
  actual por existencia).
- AC-4: Given `templates/roles/` presente y un archivo
  `roles/<archivo>.md` ({leader,implementer,reviewer,README}) que NO equivale
  a `templates/roles/<archivo>.md` bajo NINGUNA de las dos expansiones validas
  de `__HREL__` (prefijo `<basename del dir del arnes>/` o prefijo vacio),
  When corro `bash harness_check.sh`, Then reporta la divergencia
  raiz/templates (regla de espejo del Articulo 6); y Given una distribucion
  aplanada sin `templates/roles/`, Then ese sub-gate se omite sin fallo.
- AC-5: Given cualquier fallo del gate de espejo, When leo el mensaje, Then
  indica el archivo exacto y la accion correctiva (re-correr el instalador
  para regenerar espejos, o propagar el cambio a la fuente si lo editado fue
  el espejo), y `HARNESS_CHECK_MODE=warn|off` degrada igual que el resto de
  los checks (warn reporta y sale 0; off sale 0 sin evaluar).
- AC-6: Given un clon del repo fuente (senales de fuente:
  `templates/harness_cli` y `rust/` dentro del propio directorio) cuyo padre
  NO tiene huella de instalacion del arnes (ni `docs/constitution.md`, ni
  `CLAUDE.md`, ni `AGENTS.md`, ni `.claude/settings.json`) o cuyo padre es
  `$HOME` (sin `HARNESS_ALLOW_HOME_SURFACE=1`), When corro
  `bash harness_check.sh` sin `HARNESS_REPO_ROOT` ni variables de agente,
  Then `REPO_ROOT` resuelve al propio checkout, el check NO reporta la falta
  de `docs/constitution.md` (existe en el checkout) y ninguna ruta evaluada
  queda fuera del checkout.
- AC-7: Given el mismo escenario de AC-6, When corro `harness_status.sh`,
  `init.sh` y `commit_guard.sh`, Then aplican exactamente la misma resolucion
  (patron corregido en los 4 scripts y en sus espejos `templates/*`,
  identicos por `diff`).
- AC-8: Given el mismo escenario de AC-6, When corro
  `sh harness_cli start --feature <id>` (binario Rust), Then los artefactos
  (`docs/spec-*`, `docs/plan-*`) se crean bajo `<checkout>/docs/` y
  `$HOME/docs` NO se crea ni se modifica
  (`rust/src/paths.rs::repo_root_from_marker` aplica la misma regla, cubriendo
  `HarnessPaths` y `GraphEnv`).
- AC-9: Given una instalacion subdir legitima (padre con huella de
  instalacion), When corren los scripts y el binario sin overrides, Then
  `REPO_ROOT` sigue siendo el padre (cero regresion); y Given
  `HARNESS_REPO_ROOT` o una variable de agente (`CLAUDE_PROJECT_DIR`, etc.)
  definida, Then esa precedencia sigue mandando sobre cualquier deteccion.
- AC-10: Given un clon FRESCO del repo fuente en una ubicacion nueva, When
  reviso el estado versionado del marker y corro `bash harness_check.sh` sin
  variables, Then el clon no nace declarando una raiz falsa (estado de
  `.harness_layout` corregido segun la decision registrada en Observaciones)
  y `UPDATING.md` (raiz y template) documenta la transicion para
  instalaciones existentes (pull -> re-correr el setup).
- AC-11: Given `setup_harness.ps1` y `tests/setup_smoke.ps1`, When se comparan
  con sus pares `.sh`, Then replican los cambios de esta feature (estado del
  marker, guardas del instalador si las hay, bloques de smoke nuevos); sin
  `pwsh`/`powershell` en la maquina se verifica estaticamente, como en las
  features #1, #4, #5 y #6.
- AC-12: Given el repo, When corro los comandos oficiales de
  `docs/verification.md` (`cargo test`, `cargo clippy -- -D warnings`,
  `bash tests/setup_smoke.sh`), Then los tres pasan, con cobertura nueva: (a)
  tests Rust de la regla nueva en `paths.rs`; (b) bloque de smoke que corre
  `harness_check.sh` en una fixture recien instalada y pasa; (c) bloque que
  inyecta un espejo stale en esa fixture y verifica que el check lo reporta;
  (d) bloque que simula un checkout fuente y verifica resolucion local sin
  escrituras fuera del clon (el `$HOME` de la fixture queda intacto).
- AC-13: Given `README.md`, `UPDATING.md` (raiz y template), `AGENTS.md` y
  `docs/architecture.md`, When busco el gate de espejo y la resolucion de
  raiz, Then estan descritos donde corresponda (el gate como parte de
  `harness_check.sh`; la nota de migracion del marker si la decision de
  Observaciones la requiere).

## No funcionales
- SLOs: el gate agrega solo comparaciones locales (`awk`/`sed`/`diff` POSIX ya
  usados en el repo); sin red, sin dependencias nuevas en `rust/Cargo.toml`
  (Articulo 6); `harness_check.sh` mantiene su orden de magnitud actual.
- Seguridad: `harness_check.sh` sigue siendo de solo lectura (no escribe, no
  borra, no regenera espejos salvo decision contraria del usuario en
  Observaciones); sin secretos en codigo, mensajes ni tests.
- Observabilidad: mensajes accionables (archivo exacto + remedio) y exit codes
  estables (0 limpio / 2 con fallos en modo block), preservando el contrato
  `HARNESS_CHECK_MODE=block|warn|off` (Articulo 4).
- Multi-LLM: la deteccion no depende de ningun backend; cubre espejos de
  Claude (que Grok tambien lee), Gemini y Codex por igual.

## Fuera de alcance
- Auto-regenerar espejos desde `harness_check.sh` (reporta; la regeneracion
  sigue siendo re-correr el instalador), salvo decision contraria en
  Observaciones.
- Gate de contenido para las superficies generadas mayores (`CLAUDE.md`,
  `AGENTS.md`, `GEMINI.md`, `GROK.md`, `LLM.md`): solo roles y sus espejos de
  agentes.
- Espejos de Antigravity (no tiene archivos de definicion; usa `roles/*.md`
  directo) y archivos propios de Grok (lee `.claude/agents/`, ya cubierto).
- Cambiar el formato o las palabras del marker (`subdir`/`root`) para
  instalaciones, o la precedencia de las variables de entorno.
- Un `harness_check.ps1` nativo: `harness_check.sh` sigue siendo bash (en
  Windows corre via Git Bash), como hasta ahora.
- Limpiar residuos historicos fuera del repo (`$HOME/docs` ya fue limpiado a
  mano el 2026-07-28).

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Severidad del gate de espejo: DECIDIDO por el usuario (2026-07-28) — **(a)
  bloquea como los demas checks**: suma `failures` y sale 2 bajo
  `HARNESS_CHECK_MODE=block` (default); `warn` reporta y sale 0; `off` no
  evalua. Motivo: mismo rango que los otros gates, y el MODE global ya ofrece
  la valvula de escape. Un aviso que no bloquea es justamente la severidad que
  permitio que los espejos vivieran stale desde la feature #3.
- Remediacion del espejo stale: DECIDIDO por el usuario (2026-07-28) — **(a) el
  check SOLO reporta**; el remedio es re-correr el instalador.
  `harness_check.sh` permanece read-only. Motivo: regenerar exigiria duplicar
  el ensamblado del instalador (`build_*_agent`) dentro del check, y si lo
  editado fue el espejo (no la fuente) una regeneracion automatica destruiria
  ese cambio sin preguntar.
- Estado del marker `.harness_layout` versionado en el repo fuente: DECIDIDO
  por el usuario (2026-07-28) — **(a) des-versionarlo** (`git rm --cached
  .harness_layout`) y agregarlo a `.gitignore`. Motivo: el marker es estado
  local de instalacion, no codigo; sin marker el fallback ya es root y el
  instalador lo escribe en cada instalacion. Ventana BENIGNA conocida: en
  instalaciones subdir existentes, entre `git pull` y re-correr el setup el
  marker local desaparece; los efectos quedan acotados al clon y no tocan el
  proyecto. Se documenta en `UPDATING.md` (AC-10).
- Comportamiento ante la incoherencia detectada (marker dice `subdir` pero el
  padre no tiene huella de instalacion o es `$HOME`): DECIDIDO por el usuario
  (2026-07-28) — **(a) fallback a `HARNESS_DIR` con aviso informativo `[i]`**.
  Motivo: cumple el acceptance ("resuelve `REPO_ROOT` correctamente") y deja
  rastro visible del desajuste (Articulo 4), sin convertir un caso recuperable
  en un bloqueo.
- Nota de diseno (decidida por el lider, no bloquea): la senal de "checkout
  fuente" es de ENTORNO mas confirmacion de fuente: marker `subdir` Y (padre
  sin ninguna huella de instalacion {`docs/constitution.md`, `CLAUDE.md`,
  `AGENTS.md`, `.claude/settings.json`} O padre == `$HOME` sin
  `HARNESS_ALLOW_HOME_SURFACE=1`) Y senales de fuente en el propio dir
  (`templates/harness_cli` y `rust/`). Justificacion: fuente e instalacion
  subdir son clones identicos y solo el entorno los distingue;
  `templates/harness_cli` es la misma senal que el instalador ya usa para
  `ASSET_DIR`, y la regla de `$HOME` es paridad con la guarda existente del
  instalador. Falso negativo asumido y benigno: un clon subdir ANTES del
  primer setup (padre aun sin huella) opera como root hasta que el instalador
  siembra la huella; en ese estado no hay hooks instalados y nada escribe
  fuera del clon.
