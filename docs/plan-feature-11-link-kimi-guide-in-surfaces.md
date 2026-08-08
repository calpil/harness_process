# Plan - Feature #11: link_kimi_guide_in_surfaces

Estado: in_progress
Microservicios:
- harness

## Alcance

Sembrar la guia `docs/kimi-cli-uso-eficiente.md` como plantilla del arnes
(`templates/docs/`, array `HARNESS_DOCS`) y enlazarla desde las superficies que
generan `setup_harness.sh` y `setup_harness.ps1`, con dogfooding en el
`AGENTS.md` raiz de este repo, smoke actualizado y docs al dia. Spec aprobado:
`docs/spec-feature-11-link-kimi-guide-in-surfaces.md` (decision registrada: la
guia se trata como `HARNESS_DOCS`).

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->
Solo el propio arnes: instaladores (sh/ps1), plantillas, smoke tests y docs.
No toca el binario Rust ni los scripts runtime (`harness_check.sh` y hermanos);
cero impacto sobre proyectos instalados hasta que re-corran el instalador, y
para ellos el cambio es aditivo (un doc nuevo + una linea en superficies).

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->
No aplica: cambio confinado a heredocs del instalador, arrays de assets y
tests; las rutas exactas ya estan localizadas en el spec (lineas citadas).

## Delegacion (implementer)
- U1 (AC-1, AC-5): crear `templates/docs/kimi-cli-uso-eficiente.md` con
  contenido IDENTICO a `docs/kimi-cli-uso-eficiente.md` del repo (verificar con
  `diff`); ambas quedan versionadas.
- U2 (AC-1, AC-2, AC-3): `setup_harness.sh` — agregar `kimi-cli-uso-eficiente.md`
  al array `HARNESS_DOCS` (~linea 369) y `docs/kimi-cli-uso-eficiente.md` a
  `required_assets` (~linea 1618); verificar que siembra, backup y reset targets
  se derivan del array (sin listas duplicadas que actualizar); agregar el bullet
  de la guia en la lista "Archivos principales" del heredoc de
  `write_agent_surface`, junto al bullet de `.kimirules`/`.kimiignore`
  (~lineas 956-958). NO tocar `write_basic_agent_surface` ni `.grok/GROK.md`.
- U3 (AC-4): `setup_harness.ps1` — paridad: su equivalente de `HARNESS_DOCS`,
  sus required assets (~lineas 438), y UNA linea en ingles referenciando la
  guia en `Write-AgentSurface` (~linea 626).
- U4 (AC-5): dogfooding — agregar el bullet de la guia en la lista "Archivos
  principales" del `AGENTS.md` raiz de este repo (edicion manual de docs, como
  en features #3/#4/#5).
- U5 (AC-6, AC-7): `tests/setup_smoke.sh` — asserts: guia sembrada en `docs/`
  en layout subdir y root, `AGENTS.md` instalado contiene la linea de la guia,
  y reset/backup la trata como `HARNESS_DOCS`. `tests/setup_smoke.ps1` en
  paridad (revision estatica; sin pwsh en la maquina).
- U6 (AC-8): docs — `README.md`, `UPDATING.md` (raiz y `templates/UPDATING.md`
  si existe) y `docs/architecture.md` mencionan la guia, su siembra via
  `HARNESS_DOCS` y el enlace desde superficies.
- U7 (AC-9): evidencia por AC en `docs/impl-11.md` y corrida de los comandos
  oficiales de `docs/verification.md`.

## Criterios de cierre (reviewer)
- `sh harness_cli check-spec` limpio (spec approved y fresco).
- Evidencia por AC-1..AC-9 en `docs/impl-11.md`; veredicto en `docs/review-11.md`.
- `bash tests/setup_smoke.sh` rc=0 con los asserts nuevos.
- `cargo test` y `cargo clippy -- -D warnings` verdes (sin cambios Rust, pero se
  corren igual por AC-9).
- `bash harness_check.sh` limpio (incluye gate de espejo de roles, intacto).
- `diff` repo `docs/kimi-cli-uso-eficiente.md` vs `templates/docs/` identicas.
- Estado Git conocido: solo archivos de la feature modificados.

## Riesgos
- Listas duplicadas de assets en los instaladores: si siembra/reset no se
  derivan solo del array `HARNESS_DOCS`, hay que actualizar cada lista
  (verificar en U2/U3 antes de dar por cerrada la unidad).
- Smoke fragil: los asserts deben buscar texto estable de la linea nueva, no la
  linea entera con formato exacto.
- Deriva futura repo vs templates: AC-5 exige `diff` limpio en la evidencia.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- DECIDIDO por el usuario (2026-08-01, spec #11): la guia se trata como
  `HARNESS_DOCS` (backup + regeneracion en update; entra en reset targets).

### Avance 2026-08-05T16:06:22Z
Re-sincronizado con plan actualizado por otro agente (feature #11, U1..U7); trabajo previo no registrado retomado: U1/U2/U4/U5-guia/U6 parcial ya estaban en el arbol; se completo U3 (loop KimiDotfiles ps1), asserts de dotfiles en ambos smoke y U6 (architecture + templates/UPDATING)

---
Cerrado: 2026-08-07T16:15:48Z - status=done - Guia kimi-cli-uso-eficiente sembrada como HARNESS_DOCS (templates + arrays sh/ps1) y enlazada desde superficies sh/ps1 y AGENTS.md raiz (dogfooding). Companero: siembra de dotfiles .kimiignore/.kimirules como docs del usuario (loop ps1 completado). Smoke sh rc=0 con asserts nuevos, clippy limpio, cargo test 50+27 verdes, harness_check rc=0, diff guia repo/template identica. AC-7 estatico (sin pwsh). docs/impl-11.md y docs/review-11.md (approved). graph impacto no corrio: hub inalcanzable (impacto verificado manual contra git status/plan)
