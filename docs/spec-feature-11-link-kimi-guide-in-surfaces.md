# Spec - Feature #11: link_kimi_guide_in_surfaces

Estado: approved
Aprobado: 2026-08-01T16:00:48Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-11-link-kimi-guide-in-surfaces.md
Constitution: docs/constitution.md

## Problema

La guia `docs/kimi-cli-uso-eficiente.md` (exclusiones de contexto, `.kimirules`,
acotamiento por archivo, `/new` entre tareas) existe solo en este repo y esta
huerfana: ninguna superficie generada por el instalador la menciona, asi que
ningun agente la descubre en una instalacion nueva, y ningun proyecto que
instale el arnes recibe el archivo (el enlace quedaria colgado).

Las superficies raiz (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`) se
generan desde heredocs embebidos en `setup_harness.sh`
(`write_agent_surface`, lista "Archivos principales", lineas 933-958, que ya
cita `.kimirules`/`.kimiignore`) y en `setup_harness.ps1` (`Write-AgentSurface`,
linea 626, variante inglesa corta SIN lista de archivos). Los docs del arnes se
siembran desde `templates/docs/` via el array `HARNESS_DOCS`
(`setup_harness.sh:369-373`): se respaldan y regeneran en cada instalacion y
entran en los reset targets. Editar a mano solo el `AGENTS.md` de un proyecto
deja el instalador desincronizado y la proxima actualizacion lo pisa.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como duenno de un proyecto donde instalo el arnes, quiero que la guia de
  uso eficiente de Kimi CLI quede sembrada en `docs/` y enlazada desde las
  superficies, para que cualquier agente (de cualquier backend) la descubra sin
  saber de antemano que existe.
- P1: Como duenno de una instalacion existente, quiero poder refrescar la guia
  igual que `conventions.md`/`verification.md` (reinstalar tras `--reset` o con
  `--force`), para no quedarme con una version vieja.
- P2: Como usuario de Windows, quiero `setup_harness.ps1` en paridad: siembra
  del doc y referencia a la guia en su superficie.
- P2: Como mantenedor del repo fuente, quiero que el `AGENTS.md` raiz de ESTE
  repo (dogfooding) enlace la guia, para predicar con el ejemplo.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given la guia, When la ubico como plantilla del arnes, Then existe
  `templates/docs/kimi-cli-uso-eficiente.md` (mismo contenido que la copia de
  este repo) y esta listada en `HARNESS_DOCS` y en `required_assets` de
  `setup_harness.sh` (como `docs/conventions.md`).
- AC-2: Given `setup_harness.sh`, When instalo en layouts subdir y root, Then
  siembra `docs/kimi-cli-uso-eficiente.md` en la RAIZ del proyecto
  (`SURFACE_DIR/docs`) SOLO si falta, como el resto de `HARNESS_DOCS`; entra en
  los reset targets (`--reset` lo respalda y borra, reinstalar lo vuelve a
  sembrar) y `--force` lo refresca sobrescribiendo.
- AC-3: Given la superficie completa de `setup_harness.sh`
  (`write_agent_surface`), When se generan `CLAUDE.md`/`AGENTS.md`/`GEMINI.md`/
  `LLM.md`, Then la lista "Archivos principales" incluye una linea que enlaza
  `docs/kimi-cli-uso-eficiente.md` junto al bullet de `.kimirules`/`.kimiignore`,
  describiendo que es la guia de uso eficiente (exclusiones, reglas fijas,
  acotamiento por archivo, `/new`). La variante basica
  (`write_basic_agent_surface`, sin lista de archivos) y `.grok/GROK.md` (solo
  puntero) NO cambian.
- AC-4: Given `setup_harness.ps1`, When genera las superficies
  (`Write-AgentSurface`) e instala docs, Then replica la siembra (su
  `HARNESS_DOCS` + required assets) y agrega UNA linea en ingles referenciando
  la guia (su superficie no tiene lista de archivos: la linea es la paridad
  razonable, no una reestructura).
- AC-5: Given este repo (dogfooding), When leo el `AGENTS.md` raiz, Then su
  lista "Archivos principales" enlaza la guia (edicion manual de docs, como se
  hizo en las features #3/#4/#5), y la copia `docs/kimi-cli-uso-eficiente.md`
  del repo queda identica a `templates/docs/kimi-cli-uso-eficiente.md`.
- AC-6: Given `tests/setup_smoke.sh`, When corre, Then verifica con fixtures:
  (a) la guia sembrada en `docs/` en layout subdir y root; (b) el `AGENTS.md`
  instalado contiene la linea de la guia; (c) el comportamiento heredado de
  `HARNESS_DOCS` (backup/reset) cubre el archivo nuevo.
  `bash tests/setup_smoke.sh` sale 0.
- AC-7: Given `setup_harness.ps1` y `tests/setup_smoke.ps1`, When se comparan
  con sus pares `.sh`, Then replican siembra y asserts; sin `pwsh` en la
  maquina se verifica estaticamente, como en las features #1 y #4 a #10.
- AC-8: Given las docs del repo, When leo `README.md`, `UPDATING.md` (raiz y
  template) y `docs/architecture.md`, Then mencionan la guia, su siembra via
  `HARNESS_DOCS` y su enlace desde las superficies.
- AC-9: Given el repo, When corro los comandos oficiales de
  `docs/verification.md`, Then todos pasan (harness_check, cargo test, clippy).

## No funcionales
- SLOs: solo siembra de un archivo de texto y una linea en heredocs; sin
  dependencias nuevas ni logica en Rust.
- Seguridad: sin secretos ni rutas fuera del proyecto; la guia es documentacion
  publica del arnes.
- Observabilidad: el instalador reporta la siembra con su aviso habitual
  (`write_file_notice`), igual que los otros `HARNESS_DOCS`.
- Multi-LLM: el enlace queda en las 4 superficies sh + la variante ps1; lo
  hereda cualquier backend.

## Fuera de alcance
- Cambiar el contenido de la guia (ya escrita; solo se reubica como plantilla).
- Reestructurar la superficie ps1 para que sea espejo exacto de la sh (hoy son
  variantes distintas por diseno).
- Tocar `write_basic_agent_surface` (variante `--no-subagents`, sin lista de
  archivos) ni `.grok/GROK.md` (puntero minimo).
- Traducir la guia a ingles (las docs del arnes se mantienen en espanol; la
  linea de la superficie ps1 es solo la referencia).

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- DECIDIDO por el usuario (2026-08-01): la guia se trata como `HARNESS_DOCS`
  (siembra solo-si-falta; entra en reset targets con backup; refresh via
  reinstalar o `--force`), NO como documento del usuario. Motivo: es
  documentacion del arnes y conviene que siga el ciclo de las demas plantillas
  (`conventions.md`, `verification.md`).
