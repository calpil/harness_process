# Spec - Feature #4: harness_docs_to_root_docs

Estado: draft
Plan: docs/plan-feature-4-harness-docs-to-root-docs.md
Constitution: docs/constitution.md

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario que instala el arnes en un proyecto (layout subdir), quiero que
  `architecture.md`, `conventions.md` y `verification.md` queden en el `docs/` de
  la RAIZ del proyecto junto a `constitution.md`, los specs y los planes, para
  tener toda la documentacion del proceso en un solo lugar y no repartida entre
  la raiz y la subcarpeta del arnes.
- P1: Como usuario con una instalacion previa, quiero que al reinstalar los tres
  docs se MUEVAN desde `<harness>/docs/` al `docs/` de la raiz cuando alli no
  existen, para no quedarme con copias huerfanas duplicadas.
- P1: Como usuario que ya edito esos docs en la raiz, quiero que el instalador
  NUNCA los pise (ni con el reinstall ni con la migracion), para no perder mi
  trabajo — misma regla que ya protege `docs/constitution.md`.
- P1: Como agente (Claude/Codex/Gemini/Grok/Antigravity), quiero que las
  superficies y los roles me apunten a la ruta real de esos docs, para no leer
  una ruta que ya no existe.
- P2: Como usuario de Windows, quiero que `setup_harness.ps1` haga exactamente lo
  mismo que `setup_harness.sh`, para que el arnes se comporte igual en ambos SO.
- P2: Como maintainer, quiero que `--reset` siga borrando solo lo GENERADO,
  ahora en su nueva ubicacion, para que limpiar el arnes no toque mi
  constitution ni los artefactos de feature.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given un proyecto limpio con el arnes en `<raiz>/harness_process/` (layout
  subdir), When corro `setup_harness.sh`, Then `<raiz>/docs/architecture.md`,
  `<raiz>/docs/conventions.md` y `<raiz>/docs/verification.md` existen y
  `<raiz>/harness_process/docs/` NO contiene ninguno de los tres.
- AC-2: Given un proyecto con el arnes en la raiz (layout root, `--root`), When
  corro `setup_harness.sh`, Then los tres docs siguen quedando en `docs/` (no hay
  regresion: `HARNESS_DIR == SURFACE_DIR`) y la instalacion reporta idempotencia
  al re-correrla.
- AC-3: Given una instalacion previa con `<raiz>/harness_process/docs/architecture.md`
  presente y `<raiz>/docs/architecture.md` ausente, When reinstalo, Then el archivo
  queda en `<raiz>/docs/architecture.md`, deja de existir en la subcarpeta y el
  instalador emite un aviso de migracion.
- AC-4: Given `<raiz>/docs/conventions.md` ya existe con contenido editado por el
  usuario y ademas hay una copia vieja en `<raiz>/harness_process/docs/`, When
  reinstalo, Then el contenido de la raiz queda intacto (byte a byte): ni la
  migracion ni la siembra lo pisan, y la copia vieja se conserva donde esta.
- AC-5: Given la instalacion en layout subdir, When leo las superficies generadas
  (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`, `.grok/GROK.md`), Then los tres
  docs se citan como `docs/<archivo>.md` (sin el prefijo relativo al arnes) y esa
  ruta resuelve a un archivo existente desde la raiz del proyecto.
- AC-6: Given una instalacion con los tres docs en la raiz, un `docs/constitution.md`
  editado y artefactos `docs/spec-*.md` / `docs/plan-*.md` / `docs/impl-*.md` /
  `docs/review-*.md`, When corro `setup_harness.sh --reset`, Then los tres docs
  generados se borran y la constitution y los artefactos de feature sobreviven.
- AC-7: Given el mismo arbol de fixtures, When corro `setup_harness.ps1` (limpio,
  reinstall, migracion y `--reset`), Then produce exactamente las mismas rutas y
  las mismas garantias de no-pisa que `setup_harness.sh` (AC-1 a AC-6).
- AC-8: Given los roles y agentes instalados, When busco referencias a
  `verification.md`, `architecture.md` o `conventions.md` en `roles/`,
  `.claude/agents/`, `.codex/agents/` y `.gemini/agents/`, Then ninguna apunta a
  `<subcarpeta del arnes>/docs/` y todas resuelven al `docs/` de la raiz.
- AC-9: Given el repo del arnes, When corro `bash tests/setup_smoke.sh`,
  `pwsh tests/setup_smoke.ps1`, `cargo test` y `cargo clippy -- -D warnings`,
  Then todo pasa y el smoke incluye asserts nuevos para AC-1, AC-3, AC-4 y AC-6.
- AC-10: Given la documentacion del repo, When leo `README.md`, `UPDATING.md`,
  `AGENTS.md` y `docs/architecture.md`, Then describen la ubicacion nueva y
  `UPDATING.md` explica la migracion para instalaciones existentes.

## No funcionales
- SLOs: el instalador no crece en pasos interactivos; una reinstalacion sobre una
  instalacion ya migrada es idempotente (`--dry-run` refleja las rutas nuevas sin
  escribir nada).
- Seguridad: la migracion solo MUEVE archivos dentro del proyecto; nunca borra ni
  sobrescribe un destino existente. `--reset` mantiene su lista blanca explicita
  (nunca barre `docs/` entero).
- Observabilidad: cada migracion emite una linea de log identificable y cuenta en
  el reporte final de acciones (incluido el `--json`).

## Fuera de alcance
- Mover `progress/current.md` e `history.md`: siguen viviendo en la carpeta del
  arnes (`rust/src/paths.rs` los resuelve desde el directorio del ejecutable y no
  se toca en esta feature).
- `docs/constitution.md`, specs y planes: ya se generan en el `docs/` de la raiz;
  esta feature no cambia su comportamiento (solo lo blinda con tests).
- Cambiar el layout plano de specs (nada de carpetas `specs/NNN/`).
- Migrar artefactos historicos de features cerradas que hayan quedado fuera del
  `docs/` de la raiz por el footgun de `.harness_layout` en el checkout fuente.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Alcance de la mudanza: DECIDIDO por el usuario (2026-07-24) — se mueven los
  tres docs del arnes (`architecture.md`, `conventions.md`, `verification.md`).
  `progress/` NO se mueve.
- Instalaciones existentes: DECIDIDO por el usuario (2026-07-24) — migrar solo si
  falta en la raiz (mover y avisar); si ya existe en la raiz, no pisar nada.
- Semantica de refresco en reinstall: DECIDIDO por el usuario (2026-07-24,
  observacion levantada durante la implementacion) — los tres docs pasan a
  sembrarse SOLO SI FALTAN, igual que `docs/constitution.md`. Motivo: ya
  comparten carpeta con la documentacion del equipo y `docs/conventions.md` o
  `docs/architecture.md` son nombres que un proyecto probablemente ya usa; el
  comportamiento anterior (respaldar y regenerar en cada reinstall) los habria
  reemplazado. Para refrescar la plantilla: borrar el archivo y reinstalar, o
  usar `--force` (que por contrato sobrescribe sin backup).
