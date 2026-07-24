# Plan - Feature #6: interactive_spec_approval

Estado: in_progress
Spec: docs/spec-feature-6-interactive-spec-approval.md
Constitution: docs/constitution.md
Microservicios:
- harness

## Alcance

Reemplazar el tramite manual de aprobacion de specs por un flujo conversacional:
el agente lee el spec, lo muestra al usuario, se lo abre en el editor, PREGUNTA
si lo aprueba y solo con el si explicito REGISTRA la aprobacion con un comando
nuevo (`harness_cli approve-spec --yes`) que ademas re-firma la sig para no disparar
la falsa alarma de "spec actualizado por otro LLM".

La decision de aprobar sigue siendo del usuario. Lo que cambia es quien mueve
los caracteres: antes el usuario editando Markdown, ahora el agente ejecutando
un comando auditable bajo confirmacion explicita.

Dentro: `rust/src/` (comando + gate + mensajes), `docs/constitution.md`
(Articulo 2), `roles/`, `.claude/agents/`, `AGENTS.md`, `harness_check.sh`,
`setup_harness.sh`, `setup_harness.ps1`, `templates/`, `tests/`, docs de raiz.

Fuera: `--revocar`, abrir el editor desde el binario, tocar la ventana de 10
lineas o el algoritmo de firma, cerrar la feature #5.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

Microservicio unico: `harness`. Sin contratos compartidos con otros servicios.
El impacto real es de SUPERFICIE (lo que un reinstall siembra en proyectos que
ya usan el arnes): raiz y `templates/` deben quedar espejados o el reinstall
revive el texto viejo (regla de mantenedor de UPDATING.md, Articulo 6).

Superficies acopladas que deben moverse juntas:

| Origen (raiz) | Espejo | Riesgo si se desincroniza |
| --- | --- | --- |
| `roles/*.md` | `templates/roles/*.md`, `.claude/agents/*.md` | el agente instalado sigue el flujo viejo |
| `docs/constitution.md` | `templates/docs/constitution.md` | proyectos nuevos nacen con el Articulo 2 viejo |
| `harness_check.sh` | `templates/harness_check.sh` | mensaje del gate desactualizado |
| `UPDATING.md` | `templates/UPDATING.md` | el mantenedor no se entera del comando |
| `setup_harness.sh` (heredocs) | `setup_harness.ps1` | Windows queda sin paridad |

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

Pendiente para el implementer: `graphify query "flujo de aprobacion del spec"`
antes de tocar codigo; si no hay grafo utilizable, se justifica en `impl-6.md`.
La exploracion inicial del lider se hizo por lectura directa
(`rust/src/spec.rs`, `rust/src/commands/check_spec.rs`, `roles/`).

## Delegacion (implementer)

Orden sugerido: nucleo Rust primero (define el contrato), superficies despues.

1. **U1 - Nucleo del comando** (AC-1, AC-4, AC-6): en `rust/src/spec.rs`, funcion
   `approve_spec` que reescribe la PRIMERA linea `Estado:` de la ventana de 10
   lineas a `Estado: approved`, inserta el sello
   `Aprobado: <stamp> por USUARIO (confirmacion explicita)[ - nota]` debajo, y es
   idempotente si ya estaba aprobado (no duplica sello).
2. **U2 - Comando CLI** (AC-1, AC-3, AC-5, AC-6): `rust/src/commands/approve_spec.rs`
   + alta en `commands/mod.rs` y `cli.rs` (`approve-spec`, flags
   `--feature`, `--yes`, `--nota`). La barrera se valida en
   codigo propio (no `required` de clap) para dar mensaje accionable y exit 2.
   Exit codes: 0 OK/ya aprobado; 1 sin feature `in_progress`; 2 sin confirmacion
   o spec ausente. Registra en `progress/history.md` (`log`) y en el hub
   best-effort.
3. **U3 - Re-firma** (AC-2): tras escribir, llamar `update_spec_sig` y persistir
   `feature_list.json`, de modo que `check-spec` salga limpio inmediatamente
   despues. Es el corazon del bug reportado.
4. **U4 - Mensajes de gates** (AC-9): `spec.rs::spec_gate`, `commands/check_spec.rs`,
   `commands/start.rs` y `harness_check.sh` pasan a instruir el flujo nuevo.
5. **U5 - Constitution** (AC-8): Articulo 2 de `docs/constitution.md` y
   `templates/docs/constitution.md`. SOLO el Articulo 2 (el resto es del usuario).
6. **U6 - Roles** (AC-7): `roles/leader.md` (paso 5), `roles/implementer.md`
   (paso 0.2), `roles/reviewer.md`, `roles/README.md` y sus espejos
   `templates/roles/` + `.claude/agents/`. El protocolo explicito es:
   leer -> mostrar en chat -> abrir en editor -> preguntar -> registrar.
7. **U7 - Instaladores** (AC-10, AC-11): heredocs de superficie de
   `setup_harness.sh` (bloques 0.2 de CLAUDE.md/AGENTS.md/etc. y el resumen
   final) y paridad literal en `setup_harness.ps1`.
8. **U8 - Tests** (AC-12): unitarios en `rust/src/spec.rs` y en el comando para
   AC-1..AC-6; en `tests/setup_smoke.sh` verificar que la superficie sembrada
   menciona `approve-spec` y no el texto viejo; portar a `tests/setup_smoke.ps1`.
9. **U9 - Docs** (AC-13): `README.md`, `UPDATING.md`, `templates/UPDATING.md`,
   `AGENTS.md`, `docs/architecture.md`.

## Criterios de cierre (reviewer)

- `cargo test`, `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` en 0.
- Evidencia por AC-1..AC-13 en `docs/impl-6.md`.
- Raiz y `templates/` espejados; sin texto del flujo viejo (`grep` de control).
- El propio spec de esta feature aprobado con el flujo NUEVO (dogfooding), o
  explicacion de por que no.
- `bash harness_check.sh` limpio.

## Riesgos

- **Relajar el Articulo 2 mal.** Si el comando aprueba sin barrera, se pierde la
  garantia de que ningun agente auto-aprueba. Mitigacion: AC-3 lo testea y el
  sello + `history.md` dejan rastro.
- **Superficies olvidadas.** El texto viejo esta repartido en raiz, `templates/`,
  dos instaladores y `.claude/agents/`. Mitigacion: `grep -rn` de control como
  criterio de cierre (AC-10).
- **Falsa alarma de frescura al aprobar.** Si U3 se implementa mal, el bug que
  motiva la feature sobrevive. Mitigacion: AC-2 es un test de integracion, no una
  inspeccion visual.
- **Windows sin ejecucion.** No hay `pwsh` en esta maquina; AC-11 se revisa
  estaticamente (mismo tratamiento que features #4 y #5).
- **Footgun del checkout fuente.** `.harness_layout=subdir` hace que
  `repo_root` sea `/Users/alan`; todo comando del arnes en este repo se corre con
  `HARNESS_REPO_ROOT=/Users/alan/harness_process` o escribe en `$HOME/docs`.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Quien escribe `Estado: approved`: DECIDIDO por el usuario (2026-07-24): el
  AGENTE lo registra tras confirmacion explicita, via `approve-spec`.
- Como se muestra el spec al aprobar: DECIDIDO (2026-07-24): contenido en el chat
  MAS apertura en el editor del usuario.
- Entrega como feature del backlog: DECIDIDO (2026-07-24): feature #6 con flujo
  SDD completo; la #5 quedo aparcada como `pending`.
- Nombre del flag de barrera: DECIDIDO por el usuario (2026-07-24): `--yes`.
- `/Users/alan/docs/` tiene residuos de corridas viejas del arnes
  (`plan-feature-1`, `plan-feature-2`) por el footgun del layout. SIN DECISION:
  preguntar al usuario si se borran; no es parte del alcance de esta feature.

### Avance 2026-07-24T22:34:08Z
Spec #6 aprobado por el USUARIO en chat (confirmacion explicita); flag de barrera decidido: --yes

### Avance 2026-07-24T22:48:26Z
Feature #6 implementada: approve-spec (ritual mostrar+preguntar+registrar), re-firma anti falsa-alarma, constitution Art.2, roles y espejos, instaladores, tests (40+19+smoke)

---
Cerrado: 2026-07-24T22:51:37Z - status=done - Ritual de aprobacion interactiva del spec; AC-11 (Windows) verificado estaticamente
