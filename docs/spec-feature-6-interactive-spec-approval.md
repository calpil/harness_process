# Spec - Feature #6: interactive_spec_approval

Estado: approved
Aprobado: 2026-07-24 por USUARIO (confirmacion explicita en chat) - Alan aprobo
el spec tras leerlo en el chat y en el editor; flag de barrera decidido: `--yes`.
Plan: docs/plan-feature-6-interactive-spec-approval.md
Constitution: docs/constitution.md

## Problema

Hoy la aprobacion de un spec es un tramite manual y a ciegas:

1. El agente se detiene y ordena "edita `Estado: draft` -> `Estado: approved`".
   Nadie muestra el spec ni pregunta: el usuario abre el archivo, busca la linea
   y escribe el valor exacto (`approved`, dentro de las primeras 10 lineas).
2. Esa edicion a mano cambia el hash del spec y `check-spec` valida FRESCURA
   antes que estado, asi que la propia aprobacion del usuario sale reportada
   como "SPEC ACTUALIZADO POR OTRO LLM" y exige un `advance` para re-firmar.
   Reproducido en la feature #5 (ver `docs/estado-feature-5-*.md`).

Decision del usuario (2026-07-24): el agente pregunta, muestra el spec y
REGISTRA la aprobacion; la decision sigue siendo exclusivamente del usuario.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario, quiero que el agente me muestre el spec completo en el chat
  y me lo abra en el editor cuando llega el momento de aprobar, para decidir
  leyendo el contenido y no la ruta del archivo.
- P1: Como usuario, quiero que el agente me PREGUNTE explicitamente si apruebo y,
  solo con mi si, registre la aprobacion por mi, para no editar Markdown a mano.
- P1: Como usuario, quiero que aprobar NO dispare la alarma de "spec actualizado
  por otro LLM", porque mi propia aprobacion no es una edicion hostil.
- P1: Como usuario, quiero que siga siendo imposible que un agente apruebe solo:
  sin mi confirmacion explicita el comando debe negarse.
- P2: Como usuario de Windows, quiero que `setup_harness.ps1` siembre exactamente
  el mismo protocolo que `setup_harness.sh`.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given una feature `in_progress` con su spec en `Estado: draft`, When corro
  `harness_cli approve-spec --yes`, Then la primera linea
  `Estado:` del spec queda `Estado: approved`, se escribe debajo un sello
  `Aprobado: <timestamp> por USUARIO (confirmacion explicita)` y el comando
  sale 0.
- AC-2: Given un spec recien aprobado con `approve-spec`, When corro `check-spec`
  inmediatamente despues, Then sale rc=0 con `[OK] Spec aprobado y fresco` y NO
  aparece "SPEC ACTUALIZADO POR OTRO LLM" (el comando re-firma `last_spec_sig`).
- AC-3: Given el mismo escenario, When corro `approve-spec` SIN `--yes`, Then el spec NO se modifica, el comando sale 2 y el
  mensaje explica que la aprobacion exige confirmacion explicita del usuario.
- AC-4: Given un spec ya en `Estado: approved`, When vuelvo a correr
  `approve-spec --yes`, Then informa que ya estaba aprobado,
  no duplica el sello y sale 0 (idempotente).
- AC-5: Given una feature cuyo spec no existe en `docs/`, When corro
  `approve-spec --yes`, Then sale 2 con un mensaje accionable
  que indica correr `start` para sembrarlo; sin feature `in_progress` sale 1
  (mismos exit codes que `check-spec`).
- AC-6: Given `approve-spec --yes --nota "<texto>"`, When se
  ejecuta, Then el sello del spec incluye la nota y `progress/history.md` registra
  la aprobacion.
- AC-7: Given un LLM que lee `roles/leader.md`, `roles/implementer.md` o
  `roles/reviewer.md` (y sus espejos en `templates/roles/` y `.claude/agents/`),
  When el spec de la feature activa esta en draft, Then el protocolo le exige en
  este orden: leer el spec, mostrarlo al usuario, abrirlo en su editor,
  PREGUNTAR si aprueba y recien con el si ejecutar `approve-spec`; ya no aparece
  la instruccion de pedirle al usuario que edite la linea a mano.
- AC-8: Given `docs/constitution.md` y `templates/docs/constitution.md`, When leo
  el Articulo 2, Then dice que la decision de aprobar es exclusiva del usuario y
  que el agente solo la REGISTRA tras confirmacion explicita (prohibido aprobar
  sin ese si), en vez de prohibir tocar la linea `Estado:`.
- AC-9: Given los gates (`check-spec`, `spec_gate` usado por `advance` y
  `close --status done`, la salida de `start` y `harness_check.sh`), When
  bloquean por spec sin aprobar, Then el mensaje indica el flujo nuevo
  (mostrar + preguntar + `approve-spec`) y no "editalo a mano".
- AC-10: Given una instalacion nueva o un reinstall con `setup_harness.sh`, When
  reviso las superficies sembradas (`CLAUDE.md`/`AGENTS.md`/`GEMINI.md`/`GROK.md`/
  `LLM.md`, `roles/`, `harness_check.sh`, `UPDATING.md`), Then describen el flujo
  nuevo y no queda texto del flujo viejo.
- AC-11: Given `setup_harness.ps1` y `tests/setup_smoke.ps1`, When se comparan con
  sus pares `.sh`, Then siembran el mismo protocolo (paridad Windows).
- AC-12: Given el repo, When corro `cargo test`, `cargo clippy -- -D warnings` y
  `bash tests/setup_smoke.sh`, Then los tres salen 0, con tests nuevos que cubren
  AC-1 a AC-6 y una verificacion de superficie sembrada para AC-10.
- AC-13: Given `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `AGENTS.md` y
  `docs/architecture.md`, When busco `approve-spec`, Then el comando esta
  documentado en el flujo SDD (paso entre "completar spec" e "implementar").

## No funcionales

- SLOs: `approve-spec` es una escritura local sin red; no debe depender del
  Memory Hub ni de graphify para completar (el registro en el hub es
  best-effort, como en `advance`).
- Seguridad: el comando NUNCA aprueba sin `--yes`; el sello
  deja rastro auditable (quien, cuando, nota) en el propio spec y en
  `progress/history.md`.
- Observabilidad: exit codes estables 0/1/2 y mensajes accionables (Articulo 4
  de la constitution).
- Multi-LLM: el flujo es agnostico de backend (Claude, Gemini, Grok, Codex);
  ningun paso depende de una CLI de proveedor.

## Fuera de alcance

- `--revocar` (volver de `approved` a `draft`): no se implementa en esta feature.
- Abrir el editor desde el binario Rust: lo hace el AGENTE por shell
  (`open`/`xdg-open`/`start`) segun su protocolo de rol; el binario no lanza
  procesos externos.
- Cambiar la ventana de deteccion de 10 lineas o el algoritmo de firma.
- Cerrar la feature #5 (aparcada como `pending`; se retoma con este flujo).

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Nombre del flag de barrera: DECIDIDO por el usuario (2026-07-24): `--yes`
  (convencional en CLIs). Se descarto `--confirmado-por-usuario`.
- La constitution es "documento del usuario" y los agentes no la editan. El
  usuario autorizo explicitamente la enmienda del Articulo 2 al elegir esta
  opcion (2026-07-24). DECISION REGISTRADA: se edita solo el Articulo 2.
