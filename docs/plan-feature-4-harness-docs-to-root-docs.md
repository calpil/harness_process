# Plan - Feature #4: harness_docs_to_root_docs

Estado: in_progress
Microservicios:
- harness

Spec: docs/spec-feature-4-harness-docs-to-root-docs.md (Estado: draft — PENDIENTE
de aprobacion del USUARIO; hasta entonces NADIE implementa)
Constitution: docs/constitution.md

## Alcance

Unificar la documentacion del proceso en el `docs/` de la RAIZ del proyecto.

Hoy, en layout subdir, el instalador reparte los docs en dos lugares (verificado
empiricamente con una instalacion real en un fixture aislado):

```
miproyecto/docs/                    -> constitution.md            (SURFACE_DIR)
miproyecto/harness_process/docs/    -> architecture.md
                                       conventions.md
                                       verification.md            (HARNESS_DIR)
```

Los artefactos SDD (`constitution.md`, `spec-feature-*.md`, `plan-feature-*.md`)
YA caen en la raiz: `setup_harness.sh:1882` usa `$SURFACE_DIR/docs`, y el binario
resuelve `plans = repo_root/docs` (`rust/src/paths.rs:49`, con `repo_root` = padre
cuando `.harness_layout` == `subdir`). Lo que falta mudar son los tres docs del
arnes, instalados con el destino por defecto de `install_asset` (`$HARNESS_DIR/$asset`).

Entra en el alcance: destino de instalacion, migracion de instalaciones previas,
targets de `--reset`, superficies multi-LLM, roles/agentes, tests y docs, con
paridad exacta entre `setup_harness.sh` y `setup_harness.ps1`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` ->
"Ningun microservicio registrado depende de 'harness_process/harness'".

Impacto real acotado al propio arnes, pero con alcance transversal a TODAS las
instalaciones: cualquier proyecto que reinstale cambia de rutas. Por eso la
migracion (AC-3) y el no-pisa (AC-4) son parte del alcance y no un extra.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`graphify-out/graph.json` (787 nodos) confirma que la superficie tocada es el par
de instaladores: los nodos `setup_harness_*` cubren las funciones PowerShell
(`Backup-HarnessPath`, `Install-HarnessAsset`, `Ensure-Directory`, ...) que hay
que mantener en paridad con las funciones sh equivalentes (`backup_file`,
`install_asset`, `do_mkdir`). No hay consumidores fuera del arnes.

## Delegacion (implementer)

- U1 (AC-1, AC-2): en `setup_harness.sh`, instalar los tres docs en la raiz —
  `install_asset "docs/architecture.md" "$SURFACE_DIR/docs/architecture.md"` y
  analogos para `conventions.md` y `verification.md` (hoy `setup_harness.sh:1886-1888`
  usan el destino por defecto `$HARNESS_DIR/$asset`). Ajustar el array `generated`
  (`setup_harness.sh:1590-1592`) para que `backup_file` respalde la ruta nueva.
  NO tocar `required_assets` (`setup_harness.sh:1432-1434`): esa lista valida que la
  plantilla exista en `$ASSET_DIR`, no el destino. Revisar `do_mkdir "docs"`
  (`setup_harness.sh:1538`): la subcarpeta ya no necesita `docs/`.
- U2 (AC-3, AC-4): en `setup_harness.sh`, migracion previa a la instalacion: si
  `$HARNESS_DIR/docs/<archivo>` existe y `$SURFACE_DIR/docs/<archivo>` NO existe,
  mover y loguear; si el destino existe, dejar ambos intactos y no pisar. En layout
  root (`HARNESS_DIR == SURFACE_DIR`) la migracion debe ser un no-op. Respetar
  `--dry-run` (loguear sin escribir) y contar la accion en el reporte `--json`.
- U3 (AC-6): actualizar los reset targets (`setup_harness.sh:497-499`) a
  `$SURFACE_DIR/docs/...`, manteniendo la lista blanca explicita y el comentario que
  explica por que NO se barre `docs/` entero (constitution + artefactos de feature).
- U4 (AC-5): en el texto de superficie (`setup_harness.sh:880-882`), cambiar
  `__HREL__docs/architecture.md|conventions.md|verification.md` por `docs/...`
  (relativo a la raiz, como ya se citan `docs/constitution.md` y los specs).
- U5 (AC-8): actualizar las referencias en `roles/implementer.md:50` y en los
  heredocs de agentes de ambos instaladores (`.claude/agents/implementer.md:35` cita
  `harness_process/docs/verification.md`), mas los equivalentes `.codex/agents/` y
  `.gemini/agents/`. Verificar con grep que no queda ninguna ruta al docs del arnes.
- U6 (AC-7): paridad exacta en `setup_harness.ps1` — destino
  (`setup_harness.ps1:1187-1189` + el loop `Join-Path $script:HarnessDir $asset` de
  `setup_harness.ps1:1196-1199`), reset targets (`setup_harness.ps1:1064-1066`),
  migracion equivalente a U2 y texto de superficie (`setup_harness.ps1:561`).
  `$required` (`setup_harness.ps1:356-358`) no cambia, por la misma razon que U1.
- U7 (AC-9): tests. En `tests/setup_smoke.sh`, sobre el fixture subdir ya existente:
  asserts de destino raiz + ausencia en la subcarpeta (AC-1), fixture de migracion
  (AC-3), fixture de no-pisa con sentinel al estilo del ya usado para la
  constitution (AC-4) y assert de que `--reset` no borra constitution ni
  `spec-*/plan-*` (AC-6, hoy `tests/setup_smoke.sh:328` mira la ruta vieja).
  Espejar en `tests/setup_smoke.ps1`. Agregar tambien asserts de regresion de que
  constitution y spec siguen cayendo en la raiz.
- U8 (AC-10): documentar en `README.md`, `UPDATING.md` (seccion de migracion para
  instalaciones existentes), `AGENTS.md:32-34` y `docs/architecture.md`.

Orden sugerido: U1 -> U2 -> U3 -> U4 -> U5 -> U6 -> U7 -> U8. U6 solo despues de que
U1-U5 esten estables, para portar una sola vez.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-10 en `docs/impl-4.md`, con la salida real de los comandos.
- `bash tests/setup_smoke.sh` y `pwsh tests/setup_smoke.ps1` verdes (AC-9).
- `cargo test` y `cargo clippy -- -D warnings` verdes (AC-9). Nota: esta feature no
  deberia tocar `rust/src/`; si lo hace, justificarlo en el impl.
- `grep -rn "HARNESS_DIR/docs\|HarnessDir \"docs" setup_harness.sh setup_harness.ps1`
  sin hits residuales, y grep sin rutas al docs del arnes en roles/agentes (AC-8).
- Instalacion limpia en fixture subdir + root, reinstall idempotente, migracion y
  `--reset` ejercitados en un directorio temporal (NUNCA en el checkout fuente:
  su `.harness_layout` dice `subdir` y la raiz resuelta seria `$HOME`).
- Commits sin trailers de IA (regla de `UPDATING.md`).

## Riesgos

- **Perder ediciones del usuario en la migracion.** Mitigacion: mover solo cuando
  el destino NO existe (decision del usuario), mas cobertura explicita en AC-4.
- **`--reset` destructivo en la raiz.** Ahora los targets viven junto a docs del
  equipo; si alguien reemplaza la lista blanca por un barrido de `docs/`, se lleva
  la constitution y los artefactos de feature. Mitigacion: U3 conserva la lista
  explicita y AC-6 lo testea.
- **Deriva sh/ps1.** Es la deuda historica del arnes (motivo de la feature #1).
  Mitigacion: U6 despues de U1-U5 y smoke espejado en U7.
- **Instalaciones a medio migrar** (docs viejos en la subcarpeta y nuevos en la
  raiz). Mitigacion: la migracion corre en cada reinstall y es idempotente.
- **Footgun del checkout fuente**: correr `setup_harness.sh` aqui resolveria la
  raiz a `$HOME` y sembraria docs en el home del usuario. Mitigacion: todos los
  ensayos van a fixtures temporales, como en `tests/setup_smoke.sh`.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Que archivos se mudan: DECIDIDO (usuario, 2026-07-24) — los tres docs del arnes
  (`architecture.md`, `conventions.md`, `verification.md`). `progress/` se queda en
  la carpeta del arnes y `rust/src/paths.rs` no se toca.
- Instalaciones existentes: DECIDIDO (usuario, 2026-07-24) — migrar solo si falta en
  la raiz (mover + avisar); si ya existe en la raiz, no pisar.
- Semantica de refresco en reinstall: DECIDIDO (usuario, 2026-07-24) — los tres
  docs se siembran SOLO SI FALTAN (como la constitution) en vez de respaldarse y
  regenerarse en cada reinstall. Observacion levantada durante U1: ya en la raiz,
  `docs/conventions.md` y `docs/architecture.md` chocan con nombres que el equipo
  probablemente ya usa. Consecuencias aplicadas: salen del array `generated` /
  `$generatedAssets` (no se respaldan), `--force` sigue siendo la via de
  sobrescritura, y AC-4 cubre tanto la migracion como el reinstall.
- Sin decisiones pendientes abiertas: la implementacion avanzo por decision
  explicita del usuario CON el gate activo (spec en `Estado: draft`). Falta la
  aprobacion del USUARIO para poder registrar `advance` y cerrar la feature.
