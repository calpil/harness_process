# Mapa de Agentes

Este arnes usa un mapa progresivo: lee solo lo necesario para la tarea actual.

## Orden de trabajo

1. Lider revisa `feature_list.json`, `progress/current.md`, hub e impacto, y
   completa el spec (`docs/spec-feature-<id>-<slug>.md`, `Estado: draft`) con
   los AC-n en Given/When/Then ANTES del plan; cada item de la Delegacion cita
   su AC-n.
2. Ritual de aprobacion: el agente le MUESTRA el spec al USUARIO (contenido en
   el chat + abierto en su editor), le PREGUNTA si lo aprueba y solo con su SI
   lo REGISTRA con `sh harness_cli approve-spec --yes` (escribe
   `Estado: approved`, sella quien/cuando y re-firma el spec). Sin `--yes` el
   comando se niega: ningun agente aprueba por su cuenta. Con la regla
   `require_spec_approved` activa, el gate (`check-spec`, `advance`,
   `close --status done`) bloquea hasta esa aprobacion.
3. Implementer verifica `check-spec` limpio, trabaja una unidad concreta y
   escribe evidencia por AC-n en `docs/impl-<feature>.md`.
4. Reviewer verifica spec aprobado y fresco, evidencia por AC, impacto, tests,
   checkpoints y estado Git, y REGISTRA su veredicto con
   `sh harness_cli revision --feature <id> --veredicto approved` (o
   `changes_requested` / `blocked`): el binario estampa la linea `Revisado: ...`
   en `docs/review-<feature>.md` y deja rastro en `progress/history.md`. Un
   `Veredicto:` tipeado a mano NO cuenta —el gate lee solo el sello que escribe
   el binario— y el comando se niega si el review no responde por cada AC-n del
   spec con una fila que lo nombre y cite `archivo:linea`. Con la regla
   `require_review` activa, `close --status done` exige ese sello en `approved`.
5. El cierre requiere `harness_check.sh` limpio o decision explicita de bloqueo.
   El check incluye el gate de espejo de roles: los cuerpos embebidos de
   `.claude/agents/*.md`, `.gemini/agents/*.md`, `.codex/agents/*.toml` y
   `.kimi-code/agents/*.md` deben coincidir con `roles/*.md` (fuente unica), y
   `roles/*.md` con `templates/roles/*.md` modulo `__HREL__`; un espejo stale
   bloquea y el remedio es re-correr el instalador.

## Archivos principales

- `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `GROK.md`, `LLM.md`: superficies
  raiz para distintos agentes.
- `CHECKPOINTS.md`: criterios de cierre.
- `feature_list.json`: backlog ejecutable.
- `progress/current.md`: INDICE de las features en curso (feature #47). El
  estado vivo de cada una esta en `progress/current-<id>.md`.
- Features en paralelo: `start` crea la rama `feature/<id>-<slug>` (o `bugfix/`
  si es `kind: bug`) y su worktree hermano `../<repo>-wt/<id>-<slug>`. Trabaja
  DENTRO de ese worktree: ahi los comandos infieren la feature sin `--feature`,
  y el spec, el plan y la evidencia viven en su rama. El backlog y `progress/`
  son unicos (del repo principal). Al cerrar como `done` hay que decir a que
  rama se integra: `close --feature <id> --status done --to <rama>`; el arnes se
  niega sin `--to` y hay que PREGUNTARLE AL USUARIO a cual va.
- Aislamiento OBLIGATORIO (feature #72): un arranque que no consigue rama y
  worktree NO arranca — no queda `in_progress` a medias. `--sin-worktree` solo
  vale si no hay ninguna otra feature abierta, y deja la feature declarada NO
  AISLADA, lo que le niega el paralelo a la siguiente. Sin repo git pasa lo
  mismo: una feature a la vez. Si `docs/` es otro repo git, tiene su propio
  worktree (`../docs-wt/<id>-<slug>`) y ahi viven el spec y el plan.
- La integracion MUESTRA el rango completo de commits antes de mergear, y se
  niega si arrastra trabajo de otra feature (paso: se publico un arreglo y con
  el se fue un commit ajeno que era su padre). El cierre integra LOCAL: publicar
  es una decision aparte, `close ... --publicar`.
- Delegar en paralelo: hasta cuatro tareas por etapa, cada una citando su AC-n,
  y la cuenta declarada antes con `revision --esperar-tareas <n>`. Una tarea que
  fallo se registra y bloquea el cierre; borrar su linea no completa la
  cobertura. El detalle esta en `harness_process/roles/leader.md`.
- `progress/history.md`: bitacora append-only.
- `docs/constitution.md`: principios no negociables (el spec y el plan los
  cumplen; el reviewer los verifica).
- `docs/spec-feature-<id>-<slug>.md`: spec de la feature; es el PRD del cambio
  (historia antes/despues, hoy -> como va a funcionar, los datos, pseudo-codigo
  del acuerdo) con sus AC-n (Given/When/Then); se aprueba (draft -> approved)
  antes de implementar, via `harness_cli approve-spec --yes` tras el si explicito
  del usuario.
- `docs/review-<feature>.md`: el veredicto del reviewer, con una fila por cada
  AC-n del spec que lo nombre y cite `archivo:linea`. No vale escrito: se
  REGISTRA con `sh harness_cli revision --feature <id> --veredicto approved`,
  que estampa la linea `Revisado: ...` —lo unico que el gate lee— y deja
  bitacora. Sin `--veredicto`, `revision --feature <id>` arma el paquete de
  revision (AC, evidencia, archivos tocados, diff) y es de solo lectura.
- `docs/prd/COMO-ESCRIBIR-UN-PRD.md` (RAIZ): el metodo para escribir un PRD (la
  historia primero, el tamano que decide el cambio, PRDs anidados, y la regla
  dura: pseudo-codigo y explicaciones, nunca codigo final). Leela antes de
  escribir o completar un PRD o un spec.
- `docs/prd/PRD-master.md` (RAIZ): planilla maestra de producto (historia,
  objetivos O-n/NO-n, los datos, el acuerdo, hitos); sus hitos alimentan
  `feature_list.json`.
- `docs/prd/<parte>/PRD-<cadena>.md` (RAIZ): PRDs anidados, el arbol de
  producto. `sh harness_cli prd add --name <parte> [--parent <ruta>]` crea el
  hijo (12 secciones + `Padre:`) y lo enlaza en su padre; `prd tree` dibuja el
  arbol; `add ... --prd <ruta>` encadena hito -> feature -> spec (el spec cita
  su PRD) y `close --status done` vuelve al PRD a marcar el hito y dejar
  bitacora. El cuerpo del PRD no lo reescribe nadie: es del USUARIO.
- `docs/prd/SDD-master.md` (RAIZ): planilla maestra de diseno tecnico del
  proyecto (distinta de `docs/architecture.md`, que mapea lo que ya existe).
- `docs/architecture.md` (RAIZ): mapa de arquitectura.
- `docs/conventions.md` (RAIZ): convenciones del equipo.
- `docs/verification.md` (RAIZ): comandos de validacion.
- `docs/lecciones/<clase>.md` (RAIZ): memoria procedural, por CLASE de trabajo y
  no por id de feature. `sh harness_cli leccion list` ANTES de disenar;
  `leccion usar <clase>` cuando te sirva; y al aprender algo, PATCHEA la que
  estuvo en juego antes de crear otra. El metodo y la lista de que NO capturar
  (fallas de entorno, negativas sobre herramientas, errores transitorios,
  narrativas de tarea unica, fracasos disfrazados de practica) estan en
  `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`. Con la regla `require_leccion`
  activa, `close --status done` exige `--leccion <clase>` o
  `--leccion ninguna --leccion-motivo "<por que>"`.
- `docs/kimi-cli-uso-eficiente.md` (RAIZ): guia de uso eficiente de Kimi Code
  CLI (exclusiones de contexto, `.kimirules`, acotamiento por archivo, `/new`
  entre tareas).
- `docs/atlassian-integracion.md` (RAIZ): como el flujo se refleja en Jira y
  Confluence. Si el proyecto tiene `atlassian.json`, cada transicion deja un
  intent en `progress/atlassian/outbox/`; drenalo con
  `sh harness_cli atlassian drain`, ejecutalo con tu MCP de Atlassian y
  registra la clave con `atlassian ack --intent <id> --key <ADR-n>`. Con token
  configurado no hace falta: el flujo empuja solo en cada transicion (worker
  detached) y `atlassian status` muestra si esta encendido. Si NO hay binding y el usuario quiere integrar
  Jira, PREGUNTALE a que proyecto y space pertenece el repo: el arnes no lo
  adivina.
- `.claude/agents/leader.md`: rol lider.
- `.claude/agents/implementer.md`: rol implementador.
- `.claude/agents/reviewer.md`: rol revisor.

## Regla anti perdida de contexto

Todo hallazgo relevante se escribe en `progress/`. Una respuesta corta en chat
no reemplaza evidencia persistida.
