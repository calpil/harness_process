# Spec - Feature #5: prd_master_templates

Estado: draft
Plan: docs/plan-feature-5-prd-master-templates.md
Constitution: docs/constitution.md

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario que arranca un proyecto desde cero, quiero que el instalador
  deje en `docs/prd/` de la RAIZ dos planillas maestras listas para completar
  (`PRD-master.md` y `SDD-master.md`), para escribir que se construye y como
  antes de cargar la primera feature al backlog.
- P1: Como usuario que ya escribio su PRD, quiero que un reinstall NUNCA lo pise,
  para no perder el documento del proyecto — misma garantia que
  `docs/constitution.md` y que los docs del arnes tras la feature #4.
- P1: Como usuario, quiero que `--reset` NO borre `docs/prd/`, porque son
  documentos mios y no superficie generada del arnes.
- P1: Como lider (o cualquier LLM del arnes), quiero que la planilla encadene con
  el flujo existente (tabla "Hitos -> features" que alimenta `harness_cli add`, y
  recorridos P1/P2 que alimentan la seccion homonima de cada spec), para que el
  PRD no sea un documento suelto sino el origen del backlog.
- P2: Como usuario de Windows, quiero que `setup_harness.ps1` haga exactamente lo
  mismo que `setup_harness.sh`.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given un proyecto limpio con el arnes en `<raiz>/harness_process/` (layout
  subdir), When corro `setup_harness.sh`, Then existen
  `<raiz>/docs/prd/PRD-master.md` y `<raiz>/docs/prd/SDD-master.md`, y
  `<raiz>/harness_process/docs/prd/` NO existe.
- AC-2: Given un proyecto en layout root (`--root`), When corro
  `setup_harness.sh`, Then las dos planillas quedan en `docs/prd/` y una segunda
  corrida es idempotente (no las duplica ni las reescribe).
- AC-3: Given `docs/prd/PRD-master.md` ya completado por el usuario, When
  reinstalo, Then su contenido queda intacto byte a byte (siembra solo-si-falta,
  sin backup ni regeneracion).
- AC-4: Given una instalacion con `docs/prd/` completado, When corro
  `setup_harness.sh --reset`, Then `docs/prd/` y sus dos archivos SOBREVIVEN
  intactos (no son superficie generada), igual que `docs/constitution.md`.
- AC-5: Given la distribucion del arnes, When faltan
  `templates/docs/prd/PRD-master.md` o `templates/docs/prd/SDD-master.md`, Then
  el instalador falla en el preflight de required assets con exit 2 y mensaje
  accionable, en ambos instaladores.
- AC-6: Given el mismo arbol de fixtures, When corro `setup_harness.ps1` (limpio,
  reinstall y `--reset`), Then produce las mismas rutas y las mismas garantias de
  no-pisa y de supervivencia al reset que `setup_harness.sh` (AC-1 a AC-4).
- AC-7: Given `docs/prd/PRD-master.md` recien sembrado, When lo leo, Then contiene
  las secciones Problema, Usuarios y jobs-to-be-done, Metricas de exito, Alcance
  y no-objetivos, Restricciones y supuestos, Experiencia esperada (recorridos
  P1/P2), Hitos -> features, Riesgos y Decisiones abiertas; y la tabla de hitos
  documenta el comando `harness_cli add` que carga cada hito al backlog.
- AC-8: Given `docs/prd/SDD-master.md` recien sembrado, When lo leo, Then contiene
  Arquitectura objetivo, Stack y dependencias, Contratos entre componentes,
  Decisiones tecnicas, Datos, No funcionales, Estrategia de verificacion, Riesgos
  y Decisiones abiertas; y distingue explicitamente su rol del de
  `docs/architecture.md` (mapa de lo que ya existe).
- AC-9: Given el repo del arnes, When corro `bash tests/setup_smoke.sh`,
  `pwsh tests/setup_smoke.ps1`, `cargo test` y `cargo clippy -- -D warnings`,
  Then todo pasa y el smoke cubre AC-1, AC-3 y AC-4.
- AC-10: Given la documentacion del repo, When leo `README.md`, `UPDATING.md`,
  `AGENTS.md` y `docs/architecture.md`, Then describen `docs/prd/` y el flujo
  PRD master -> `feature_list.json` -> spec por feature -> plan -> implementacion.

## No funcionales
- SLOs: cero pasos interactivos nuevos; el instalador no crece en dependencias.
- Seguridad: las planillas no piden ni contienen secretos; son markdown estatico.
- Observabilidad: la siembra cuenta en el reporte final de acciones y el
  `--dry-run` refleja las rutas nuevas sin escribir nada.

## Fuera de alcance
- Integrar el PRD con el binario Rust (`harness_cli`): no hay subcomando nuevo ni
  gate que exija PRD completado. Las planillas son documentos, no un flujo.
- Generar features automaticamente desde la tabla "Hitos -> features": la carga al
  backlog sigue siendo manual con `harness_cli add`.
- Migrar PRDs existentes desde otras ubicaciones: no habia `docs/prd/` antes.
- Tocar `docs/architecture.md` como plantilla (sigue siendo el mapa de lo que ya
  existe, distinto del SDD master).

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Ubicacion: DECIDIDO por el usuario (2026-07-24) — `docs/prd/` en la RAIZ del
  proyecto (dentro del `docs/` unificado por la feature #4), no una carpeta `prd/`
  hermana ni dentro de la subcarpeta del arnes.
- Que planillas: DECIDIDO por el usuario (2026-07-24) — dos documentos separados,
  `PRD-master.md` (el que y el por que) y `SDD-master.md` (el como, a nivel
  proyecto). Sin `README.md` adicional en `docs/prd/`.
- Reset: DECIDIDO por el usuario (2026-07-24) — `docs/prd/` NO entra en los reset
  targets; son documentos del usuario, como la constitution.
