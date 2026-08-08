# Review - Feature #11: link_kimi_guide_in_surfaces

Spec: docs/spec-feature-11-link-kimi-guide-in-surfaces.md (Estado: approved, sellado 2026-08-01T16:00:48Z por USUARIO)
Plan: docs/plan-feature-11-link-kimi-guide-in-surfaces.md
Impl: docs/impl-11.md

## Veredicto global

**approved**, con dos salvedades conocidas (ninguna bloquea):

1. AC-7 (paridad Windows) queda **parcial**: revision estatica real, sin
   ejecucion, porque no hay `pwsh` en esta maquina (verificado con `which`,
   rc=1). Misma limitacion aceptada en las features #1 y #4 a #10.
2. La sesion que inicio la implementacion dejo el trabajo en el working tree
   SIN registrar avance ni evidencia. Esta revision no acepto esa omision:
   el estado se reconstruyo desde `git diff`, se completo lo faltante (loop
   `$script:KimiDotfiles` del ps1, asserts de dotfiles en ambos smoke,
   `docs/architecture.md`, `templates/UPDATING.md`) y yo re-verifique cada
   punto contra el arbol real antes de firmar.

## Aprobacion del spec

```
Estado: approved
Aprobado: 2026-08-01T16:00:48Z por USUARIO (confirmacion explicita)
```

`progress/history.md` tiene la linea `approve-spec feature #11 ... nota=Alan
aprobo el spec #11 en el chat (2026-08-01)` (2026-08-01T16:06:39Z).
`bash harness_check.sh` (2026-08-05) reporta `[spec] #11 approved (fresco)` y
`[plan] #11 fresco` tras el advance que re-firmo el plan. La unica decision
de Observaciones (guia tratada como `HARNESS_DOCS`) esta DECIDIDA por el
usuario en spec y plan, y la implementacion la cumple (Articulo 5).

## Estado por AC

- **AC-1 — cubierto**: `templates/docs/kimi-cli-uso-eficiente.md` existe y es
  IDENTICA a la copia del repo (`diff` sin salida, re-ejecutado por mi).
  Listada en `HARNESS_DOCS` (`setup_harness.sh:373`) y en `required_assets`
  (`setup_harness.sh:1625`), como manda el spec.
- **AC-2 — cubierto**: siembra solo-si-falta o `--force`
  (`setup_harness.sh:2178-2183`), reset targets derivados del mismo array
  (`setup_harness.sh:555-560`): sin listas duplicadas (riesgo del plan,
  verificado). Smoke en verde con asserts de siembra root/subdir y de reset
  (`tests/setup_smoke.sh:153-154, :215-216, :463-465`).
- **AC-3 — cubierto**: bullets en el heredoc de `write_agent_surface`
  (`setup_harness.sh:957-962`), guia descrita con exclusiones, reglas fijas,
  acotamiento y `/new`, junto al bullet de `.kimirules`/`.kimiignore`.
  `write_basic_agent_surface` y `.grok/GROK.md` sin tocar (diff acotado a
  arrays, heredoc de superficie completa, required_assets y siembra). El
  `AGENTS.md` instalado contiene la linea (assert del smoke, rc=0).
- **AC-4 — cubierto (estatico)**: `$script:HarnessDocs` + guia
  (`setup_harness.ps1:83`), required assets (:436), UNA linea en ingles en
  `Write-AgentSurface` (:656-658). El loop de siembra de
  `$script:KimiDotfiles` (:1500-1511) existia como array declarado pero nunca
  usado; se completo en esta sesion (sin el, el ps1 habria exigido los
  templates en `Assert-HarnessAssets` sin instalarlos nunca).
- **AC-5 — cubierto**: `AGENTS.md` raiz con el bullet de la guia en "Archivos
  principales"; `diff` repo vs template limpio (ver AC-1).
- **AC-6 — cubierto**: `bash tests/setup_smoke.sh` rc=0 (2026-08-05) con los
  asserts (a), (b) y (c) citados en AC-2/AC-3, mas los companeros de dotfiles
  (siembra root/subdir y supervivencia al reset).
- **AC-7 — parcial (estatico)**: sin `pwsh` en la maquina; el smoke ps1 espeja
  los asserts del sh (`tests/setup_smoke.ps1:132, :154, :180, :137-139,
  :185-187, :219-222`). Revision estatica registrada como tal, criterio
  aceptado desde la feature #1.
- **AC-8 — cubierto**: `README.md` (arbol :283 y redaccion de refresco),
  `UPDATING.md` (:56-58 + dotfiles, redaccion de `--reset` :146 y garantias
  PRD :169-171), `templates/UPDATING.md` (mismos puntos espejados) y
  `docs/architecture.md` (bullet `HARNESS_DOCS` con la guia, bullet propio de
  la guia + enlace desde superficies, bullet `KIMI_DOTFILES`, `--reset`
  actualizado). Sin referencias stale a "tres docs" (grep limpio).
- **AC-9 — cubierto**: corrida 2026-08-05 de los comandos oficiales:
  `bash tests/setup_smoke.sh` rc=0;
  `(cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)`
  rc=0; `(cd rust && cargo test --locked)` rc=0 (50 unit + 27 integracion, 0
  fallos); `bash harness_check.sh` rc=0 ("Harness Check limpio").

## Chequeo de impacto

El radio declarado en el plan (instaladores sh/ps1, `templates/docs/`, smoke
sh/ps1, `AGENTS.md` raiz, README/UPDATING/architecture) coincide con el
`git status`: los archivos de la feature son exactamente esos mas
`templates/.kimiignore`/`templates/.kimirules` (el companero documentado).
Ningun archivo de runtime (`harness_check.sh` y hermanos) ni de Rust tocado,
como manda el plan. El cambio es aditivo para proyectos instalados (un doc
nuevo + una linea en superficies) y solo se propaga al re-correr el
instalador.

`graphify query`: no aplicada, con la justificacion del plan (cambio confinado
a heredocs/arrays/tests; rutas ya localizadas por linea en el spec). Existe
`graphify-out/graph.json`, pero la consulta no aportaba nada a este radio.

`sh harness_cli graph impacto --microservicio ADR/harness`: intentado el
2026-08-05 y colgo sin emitir nada hasta su timeout (90 s): el hub PostgreSQL
no es alcanzable desde esta maquina en este momento (mismo sintoma que el
advance, que SI completo su registro en disco antes de colgar en el paso de
hub/graphify; `connect_timeout` de 10 s por conexion en
`rust/src/graph/store.rs:33`, pero la resolucion del host queda fuera de ese
limite). El chequeo de impacto se hizo manual contra `git status`/plan, como
arriba; condicion de entorno, no de la feature.

Estado Git al firmar (conocido): los 10 archivos modificados y los nuevos de
la feature listados arriba; ademas quedan sin commit los artefactos de la
feature #10 ya cerrada (`docs/impl-10.md`, `docs/review-10.md`,
`docs/estado-feature-10-*.md` y la firma de cierre en su plan), ajenos a esta
revision. Nada de ello esta commiteado: el commit queda para el usuario, por
politica del repo.

## Hallazgos (ninguno bloquea)

1. Trabajo sin registrar en `progress/` por la sesion previa (regla
   anti-perdida de contexto incumplida). Mitigado: estado reconstruido desde
   `git diff`, evidencia completa en `docs/impl-11.md` y todo re-ejecutado.
2. El plan daba por existente el bullet de `.kimirules`/`.kimiignore` en la
   superficie (premisa del spec, redactada sobre el working tree sucio); en
   HEAD no existia. La sesion previa agrego ambos bullets y la siembra de los
   dotfiles; queda documentado como cambio companero en `docs/impl-11.md`. La
   guia (contenido congelado por el spec) afirma que el instalador siembra
   `.kimiignore`, asi que sin el companero la guia sembrada mentiria.
3. `sh harness_cli advance` registro correctamente (linea en `history.md`,
   plan re-firmado) pero el proceso colgo despues en el paso de hub/graphify
   (hub inalcanzable desde esta maquina; timeout de 120 s). Condicion de
   entorno, no de la feature: el registro en disco quedo completo y
   `harness_check.sh` sale rc=0.

## Veredicto

La feature cumple AC-1..AC-9 con evidencia re-ejecutada (AC-7 estatico por
ausencia de `pwsh`), respeta la decision registrada del usuario y deja verdes
los comandos oficiales. **Apta para `close --status done`**.
