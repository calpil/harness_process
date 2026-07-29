# Spec - Feature #10: layout_inferred_from_footprint

Estado: approved
Aprobado: 2026-07-29T04:19:41Z por USUARIO (confirmacion explicita) - Alan aprobo el spec #10 en el chat (2026-07-29): inferencia del layout por huella del padre cuando falta el marker, con aviso [i]
Plan: docs/plan-feature-10-layout-inferred-from-footprint.md
Constitution: docs/constitution.md

## Problema

La feature #7 des-versiono `.harness_layout` (`git rm --cached` + `.gitignore`,
commit c8392f5) para que ningun clon del repo fuente naciera declarando una
raiz falsa. El efecto colateral no se dimensiono bien: `git rm --cached` deja
el archivo en la copia local del que ejecuta el comando, pero **graba
`D .harness_layout` en el commit**, asi que git lo BORRA del working tree de
toda instalacion existente que haga `git pull`.

Sin marker, la resolucion actual cae al `else` y toma el directorio del arnes
como raiz (`harness_check.sh:35-37` y sus tres scripts hermanos). Una
instalacion subdir legitima pasa a escribir specs, planes y veredictos dentro
de `harness_process/` en vez del `docs/` del proyecto.

Reportado en la practica el 2026-07-29: un proyecto front actualizo y perdio el
marker; el backend, que no habia actualizado, lo conservaba. En esta maquina
hay 15 instalaciones con el marker todavia presente y repos aun en commits
previos a c8392f5: todas caen igual en cuanto actualicen.

La feature #7 lo anoto como "ventana BENIGNA entre `git pull` y re-correr el
setup", documentada en `UPDATING.md`. La practica muestra que no es benigna:
degrada silenciosamente (sin aviso alguno) y obliga a re-correr el instalador
en cada instalacion para repararla.

### Reproduccion (2026-07-29, bloque de resolucion real extraido del script)

Fixture con padre CON huella de instalacion (`docs/constitution.md`,
`CLAUDE.md`, `AGENTS.md`) y el arnes en `harness_process/`:

```
CON  .harness_layout=subdir -> REPO_ROOT=<fixture>/proyecto            (correcto)
SIN  .harness_layout        -> REPO_ROOT=<fixture>/proyecto/harness_process  (roto)
```

La huella del padre estaba ahi en ambos casos: la informacion para resolver
bien existia y no se estaba mirando.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como duenno de una instalacion subdir que hizo `git pull`, quiero que el
  arnes siga resolviendo la raiz a mi proyecto aunque el marker haya
  desaparecido, para que specs, planes y veredictos no terminen dentro de
  `harness_process/` sin que yo me entere.
- P1: Como duenno de esa instalacion, quiero ENTERARME de que el marker falta y
  de que se esta operando por inferencia, para poder regenerarlo cuando me
  convenga.
- P1: Como usuario de una instalacion en layout root, quiero que un
  `.harness_layout` que dice `root` se siga respetando al pie de la letra, para
  que la inferencia nueva no me cambie la raiz.
- P2: Como mantenedor del checkout fuente, quiero que el guardrail de la
  feature #7 siga intacto: el fuente bajo `$HOME` no debe inferir subdir ni con
  marker ni sin el.
- P2: Como usuario de Windows, quiero `setup_harness.ps1` y los scripts en
  paridad exacta con sus pares.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given una instalacion subdir SIN `.harness_layout` cuyo padre tiene al
  menos una huella de instalacion (`docs/constitution.md`, `CLAUDE.md`,
  `AGENTS.md`, `.claude/settings.json`), When corro `harness_check.sh`,
  `harness_status.sh`, `init.sh` o `commit_guard.sh` sin variables de entorno,
  Then `REPO_ROOT` resuelve al PADRE (el proyecto), no al directorio del arnes.
- AC-2: Given ese mismo escenario, When se resuelve la raiz por inferencia,
  Then se emite UN aviso informativo `[i]` que dice que el marker falta, que el
  layout subdir se infirio por la huella del padre, y que re-correr el
  instalador lo regenera. No es un fallo: el exit code no cambia.
- AC-3: Given un `.harness_layout` que existe y dice `root` (o cualquier valor
  distinto de `subdir`), When se resuelve la raiz, Then se respeta el marker
  explicito y `REPO_ROOT` es el directorio del arnes, SIN inferencia y SIN
  aviso. La inferencia solo aplica cuando el archivo NO existe.
- AC-4: Given un directorio del arnes SIN marker cuyo padre NO tiene ninguna
  huella de instalacion, When se resuelve la raiz, Then `REPO_ROOT` es el
  directorio del arnes (comportamiento actual, sin aviso): no hay evidencia
  para inferir nada.
- AC-5: Given el checkout fuente SIN marker cuyo padre es `$HOME` (sin
  `HARNESS_ALLOW_HOME_SURFACE=1`), When se resuelve la raiz, Then NO se infiere
  subdir aunque `$HOME` tenga `CLAUDE.md` o `AGENTS.md` sueltos: la guarda de
  `$HOME` de la feature #7 aplica tambien a la inferencia nueva.
- AC-6: Given `HARNESS_REPO_ROOT` o una variable de agente
  (`CLAUDE_PROJECT_DIR`, `CODEX_PROJECT_DIR`, ...) definida, When se resuelve la
  raiz, Then esa precedencia sigue mandando sobre la inferencia, sin aviso.
- AC-7: Given el guardrail de checkout fuente de la feature #7 (marker
  `subdir` + senales de fuente + padre sin huella o `$HOME`), When corro los
  scripts en este checkout, Then sigue comportandose igual que antes de esta
  feature (cero regresion): `[i] Checkout fuente del arnes detectado ...` y
  `REPO_ROOT` = el propio checkout.
- AC-8: Given el binario Rust, When `harness_cli` resuelve rutas sin variables
  de entorno en una instalacion subdir sin marker con padre con huella, Then
  aplica la MISMA regla que los scripts
  (`rust/src/paths.rs::repo_root_from_marker`, punto unico que consumen
  `HarnessPaths` y `GraphEnv`), con tests unitarios nuevos que cubran: sin
  marker + huella -> padre; sin marker + sin huella -> propio dir; marker
  `root` -> propio dir sin inferencia.
- AC-9: Given los 4 scripts (`harness_check.sh`, `harness_status.sh`,
  `init.sh`, `commit_guard.sh`), When los comparo con sus espejos en
  `templates/`, Then son identicos por `diff` y todos aplican la misma regla
  (Articulo 6).
- AC-10: Given `tests/setup_smoke.sh`, When corre, Then cubre con fixtures
  propias: (a) subdir sin marker con huella -> raiz al padre + aviso `[i]`;
  (b) sin marker sin huella -> raiz al propio arnes, sin aviso; (c) marker
  `root` explicito -> sin inferencia; (d) el guardrail de checkout fuente de la
  #7 sigue verde. `bash tests/setup_smoke.sh` sale 0.
- AC-11: Given `setup_harness.ps1` y `tests/setup_smoke.ps1`, When se comparan
  con sus pares `.sh`, Then replican la regla nueva; sin `pwsh`/`powershell` en
  la maquina se verifica estaticamente, como en las features #1, #4, #5, #6,
  #7, #8 y #9.
- AC-12: Given `UPDATING.md` (raiz y template) y `docs/architecture.md`, When
  leo la nota de migracion de la feature #7, Then queda corregida: la
  desaparicion del marker tras `git pull` ya no exige re-correr el instalador
  porque la raiz se infiere sola, y se explica cuando SI conviene regenerarlo.
- AC-13: Given el repo, When corro los comandos oficiales de
  `docs/verification.md`, Then los tres pasan.

## No funcionales
- SLOs: solo comprobaciones locales de existencia de archivos (las mismas rutas
  de huella que ya consulta el guardrail de la #7); sin dependencias nuevas.
- Seguridad: la inferencia NUNCA amplia el alcance por encima del padre
  inmediato, respeta la guarda de `$HOME` y no escribe nada (no regenera el
  marker: los scripts siguen siendo de solo lectura; regenerarlo es trabajo del
  instalador).
- Observabilidad: un unico aviso `[i]` por corrida, con el remedio explicito.
  Exit codes sin cambios.
- Multi-LLM: la regla vive en los 4 scripts y en el punto unico de Rust; no
  depende de ningun backend.

## Fuera de alcance
- Re-versionar `.harness_layout` (decision del usuario 2026-07-29: se arregla
  por inferencia, no volviendo a meter estado local en git).
- Que los scripts REGENEREN el marker: siguen siendo read-only; lo escribe el
  instalador.
- Cambiar el guardrail de checkout fuente de la feature #7.
- Reparar a mano las 15 instalaciones existentes: con esta feature se reparan
  solas al actualizar.
- Ampliar la lista de huellas de instalacion mas alla de las cuatro que ya usa
  el guardrail de la #7.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- DECIDIDO por el usuario (2026-07-29): arreglar por INFERENCIA por huella del
  padre, no re-versionando el marker. Motivo: no reintroduce estado local en
  git, no hace que un clon en layout root nazca declarando `subdir`, y repara
  solas las instalaciones que ya hicieron `git pull` sin re-correr el
  instalador en 15 sitios.
- DECIDIDO por el usuario (2026-07-29): cuando el layout se infiere, se emite
  un aviso `[i]` discreto (no silencioso), para dejar rastro de que se opera
  por inferencia y sugerir la regeneracion del marker.
