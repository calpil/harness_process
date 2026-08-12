# Plan - Feature #12: prd_story_method

Estado: in_progress
Microservicios:
- harness

Spec: docs/spec-feature-12-prd-story-method.md (Estado: approved, 2026-08-12)

## Alcance

Instalar el metodo de `how-i-spec.pdf` ("Escribe tu maldito PRD") en las tres
superficies donde el arnes crea especificacion:

1. `docs/prd/PRD-master.md` (planilla del USUARIO): anatomia nueva con la
   historia ANTES/DESPUES al frente, resumen hoy->despues, objetivos numerados
   O-n/NO-n, flujo dibujado dos veces, Los datos y Pseudo-codigo a nivel
   producto, y la regla dura "sin codigo final". Conserva la cadena
   hitos -> `feature_list.json`.
2. `docs/prd/COMO-ESCRIBIR-UN-PRD.md` (NUEVO, plantilla del arnes): el metodo
   completo, incluyendo el tamano por tipo de cambio y el anidamiento de PRDs.
3. `spec_template()` en `rust/src/spec.rs`: cada spec de feature nace con
   Historia, Hoy -> Como va a funcionar, Los datos que se tocan y Pseudo-codigo
   (el acuerdo), ademas de Recorridos y AC-n.

Mas la plomeria que hace que eso llegue a un proyecto instalado: siembra en
ambos instaladores, enlace desde las superficies multi-LLM, smoke tests y docs.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

Microservicio unico (`harness`). El hub Postgres no esta accesible desde esta
maquina (`error connecting to server: connection timed out` al correr `start`),
asi que el impacto se calcula por lectura directa del repo, como en features
anteriores con el hub caido.

Superficies tocadas y su radio:
- `templates/docs/prd/*` -> se copia a la RAIZ de cada proyecto instalado.
  Cambio de contenido, no de contrato: `PRD_DOCS` se sigue sembrando solo-si-falta.
- `HARNESS_DOCS` gana la ruta `prd/COMO-ESCRIBIR-UN-PRD.md`. Los tres bucles que
  la consumen aceptan subdirectorio sin cambios (`install_asset` hace
  `mkdir -p`, reset targets arma la ruta completa, `migrate_harness_docs` tambien
  hace `mkdir -p`). Verificado por lectura: `setup_harness.sh:555`, `:1657`,
  `:1690`, `:2178`.
- `spec_template()` solo se usa en `write_spec`, que crea el archivo SOLO si no
  existe: ningun spec cerrado (#1..#11) se reescribe.
- Espejo raiz/`templates/` (Articulo 6): toda edicion en `templates/docs/prd/`
  se replica identica en `docs/prd/` del repo.
- `roles/*.md` NO se tocan (fuera de alcance del spec): el gate de espejo de
  roles de `harness_check.sh` queda intacto.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

No aplicable en esta feature: el cambio es de plantillas y de una funcion pura
de Rust, sin dependencias cruzadas que descubrir. El mapa relevante ya esta en
`docs/architecture.md` (cadena PRD -> backlog -> spec -> impl).

## Delegacion (implementer)
- U1 (AC-1, AC-2, AC-3): reescribir `templates/docs/prd/PRD-master.md` con las
  12 secciones de la anatomia, la seccion 2 con bloques ANTES/DESPUES narrados,
  objetivos con IDs `O1`/`NO1`, secciones 7 y 8 (Los datos / Pseudo-codigo) a
  nivel producto declarando que cada feature refina el suyo en su spec, la regla
  dura "sin codigo final", y la seccion 10 con la tabla de hitos y la linea
  `harness_cli add` intactas. Espejar en `docs/prd/PRD-master.md`.
- U2 (AC-4): crear `templates/docs/prd/COMO-ESCRIBIR-UN-PRD.md` con las cinco
  piezas del metodo (contiene/nunca contiene; la historia con contraste
  asi-no/asi-si; la tabla de tamano y el anidamiento; la anatomia con el ejemplo
  en miniatura; el mapeo al arnes). Espejar en `docs/prd/`.
- U3 (AC-5, AC-8): `setup_harness.sh` — agregar `prd/COMO-ESCRIBIR-UN-PRD.md` a
  `HARNESS_DOCS` y a `required_assets`, y la linea de la guia en la lista
  "Archivos principales" de `write_agent_surface`. Sin tocar
  `write_basic_agent_surface` ni `.grok/GROK.md`.
- U4 (AC-5, AC-8): `setup_harness.ps1` — misma alta en `$script:HarnessDocs` y
  en los required assets, mas la referencia en `Write-AgentSurface` (variante
  inglesa corta, una linea, como la paridad de la feature #11).
- U5 (AC-6, AC-7): `rust/src/spec.rs` — nuevas secciones en `spec_template()` en
  el orden aprobado, con `Estado: draft` en la linea 3 y puntero a la guia en el
  encabezado; actualizar `spec_template_should_declare_draft_and_sections` para
  cubrir las cuatro secciones nuevas.
- U6 (AC-9): `tests/setup_smoke.sh` y `tests/setup_smoke.ps1` — asserts de
  siembra de la guia (subdir y root), secciones nuevas del PRD (incluida la
  renumeracion de `## 7. Hitos -> features` a `## 10.`), enlace en la superficie
  instalada y sentinels de no-pisado ya existentes.
- U7 (AC-10): `README.md`, `AGENTS.md`, `UPDATING.md` (raiz y `templates/`) y
  `docs/architecture.md` — describir la guia, su siembra y las secciones nuevas
  del spec generado.
- U8 (AC-11): correr la verificacion oficial completa y dejar evidencia por AC-n
  en `docs/impl-12.md`.

## Criterios de cierre (reviewer)
- Evidencia por AC-1..AC-11 en `docs/impl-12.md`, con rutas y lineas.
- `bash harness_check.sh` limpio (incluido el gate de espejo de roles, que no
  deberia moverse porque `roles/*.md` no se tocan).
- `cargo test --locked` y `cargo clippy --all-targets --all-features --locked --
  -D warnings` en verde.
- `bash tests/setup_smoke.sh` sale 0.
- Espejo exacto `templates/docs/prd/*` == `docs/prd/*` (`diff` sin salida) para
  los dos archivos tocados.
- Commits Conventional SIN trailers de IA (`commit_guard.sh`).

## Riesgos
- Renumeracion de secciones del PRD: los smoke tests (sh y ps1) buscan
  `## 7. Hitos -> features`. Si no se actualizan ambos, el smoke rompe.
  Mitigacion: U6 los actualiza y U8 corre el smoke completo.
- `HARNESS_DOCS` con subdirectorio: es el primer elemento del array con `/`.
  Mitigacion: verificados por lectura los tres consumidores (siembra, reset,
  migracion); el smoke lo confirma en fixtures reales.
- Specs existentes: cambiar la plantilla no debe reescribirlos.
  Mitigacion: `write_spec` solo crea si falta (ya cubierto por AC-7).
- Instalador en el checkout fuente: NO se corre `setup_harness.sh` aqui (footgun
  conocido); toda la verificacion de siembra pasa por los fixtures del smoke.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Alcance y vehiculo: DECIDIDO (Alan, 2026-08-11) — metodo en las tres
  superficies (planilla, guia nueva, `spec_template()`), ejecutado como feature
  #12 por el flujo del arnes.
- Regimen de la guia: DECIDIDO (Alan, 2026-08-12, al aprobar el spec) —
  `COMO-ESCRIBIR-UN-PRD.md` es plantilla del arnes (`HARNESS_DOCS`), no
  documento del usuario; `PRD-master.md`/`SDD-master.md` no cambian de regimen.
- Datos y pseudo-codigo: DECIDIDO (Alan, 2026-08-12) — van en los DOS niveles:
  el maestro a nivel producto y el spec de cada feature a nivel cambio.
- Sin decisiones abiertas: el implementer puede ejecutar U1..U8 completo.

### Avance 2026-08-12T02:28:01Z
Plan del lider escrito y re-sincronizado (U1..U8 delegadas, decisiones de Alan registradas)

---
Cerrado: 2026-08-12T02:34:02Z - status=done - 
