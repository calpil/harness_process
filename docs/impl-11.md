# Impl - Feature #11: link_kimi_guide_in_surfaces

Spec: docs/spec-feature-11-link-kimi-guide-in-surfaces.md (Estado: approved, sellado 2026-08-01T16:00:48Z por USUARIO)
Plan: docs/plan-feature-11-link-kimi-guide-in-surfaces.md

> Nota de proceso: una sesion previa dejo U1, U2, U4, la parte de la guia de
> U5 y parte de U6 en el working tree SIN registrar avance en `progress/`.
> Esta sesion reconstruyo el estado desde `git diff`, verifico lo heredado,
> completo lo que faltaba (loop de siembra de dotfiles en el ps1 — el array
> `$script:KimiDotfiles` estaba declarado pero nunca se usaba—, asserts de
> dotfiles en ambos smoke, `docs/architecture.md` y `templates/UPDATING.md`) y
> registro la evidencia completa aqui.

## Que cambio

La guia `docs/kimi-cli-uso-eficiente.md` pasa a ser plantilla del arnes
(`templates/docs/`, array `HARNESS_DOCS`): se siembra en el `docs/` de la RAIZ
solo si falta, entra en los reset targets y se refresca con reinstall o
`--force`. Las superficies generadas (`write_agent_surface` en sh,
`Write-AgentSurface` en ps1) la enlazan; el `AGENTS.md` raiz de este repo la
enlaza a mano (dogfooding).

Cambio companero (necesario para que la guia sembrada no mienta): siembra de
los dotfiles `.kimiignore`/`.kimirules` como documentos del USUARIO en la RAIZ
(arrays `KIMI_DOTFILES` / `$script:KimiDotfiles`, solo-si-faltan, ni `--force`
ni `--reset` los tocan, mismo criterio que PRD/SDD). La guia —cuyo contenido
congelo el spec— afirma "El instalador siembra `.kimiignore` en la raiz del
proyecto", y la premisa del AC-3 del spec referencia el bullet de
`.kimirules`/`.kimiignore` en la superficie; ambas cosas solo son ciertas con
esta siembra.

## Unidades

| Unidad | AC | Archivos |
| --- | --- | --- |
| U1 template de la guia | AC-1, AC-5 | `templates/docs/kimi-cli-uso-eficiente.md` |
| U2 instalador sh | AC-1, AC-2, AC-3 | `setup_harness.sh` (`HARNESS_DOCS` :373, `KIMI_DOTFILES` :391-394, bullets superficie :957-962, `required_assets` :1625 y :1629-1630, siembra dotfiles :2197-2207) |
| U3 paridad ps1 | AC-4 | `setup_harness.ps1` (`HarnessDocs` :83, `KimiDotfiles` :101-104, required assets :436 y :440-441, linea en superficie :656-658, loop de siembra :1500-1511) |
| U4 dogfooding | AC-5 | `AGENTS.md` (raiz) |
| U5 smoke sh + ps1 | AC-6, AC-7 | `tests/setup_smoke.sh`, `tests/setup_smoke.ps1` |
| U6 docs | AC-8 | `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `docs/architecture.md` |
| U7 evidencia + verificacion | AC-9 | `docs/impl-11.md` (este archivo) |

## Evidencia por AC

- **AC-1** (guia como plantilla): existe
  `templates/docs/kimi-cli-uso-eficiente.md` y
  `diff docs/kimi-cli-uso-eficiente.md templates/docs/kimi-cli-uso-eficiente.md`
  sale limpio (identicas). Listada en `HARNESS_DOCS`
  (`setup_harness.sh:373`, mismo array que `conventions.md`) y en
  `required_assets` (`setup_harness.sh:1625`).
- **AC-2** (siembra/reset/force derivados del array): la siembra itera
  `HARNESS_DOCS` (`setup_harness.sh:2178-2183`: solo si falta o con `--force`);
  los reset targets iteran el mismo array (`setup_harness.sh:555-558`), asi que
  `--reset` la respalda y borra, y reinstalar la vuelve a sembrar. Sin listas
  duplicadas: migracion (`migrate_harness_docs`, :1672) y reset usan el array.
  Verificado en verde por el smoke (asserts citados en AC-6, rc=0).
- **AC-3** (enlace en la superficie sh): bullets en
  `setup_harness.sh:957-962` dentro del heredoc de `write_agent_surface`
  (lista "Archivos principales"): la guia con su descripcion (exclusiones de
  contexto, reglas fijas, acotamiento por archivo, `/new` entre tareas), junto
  al bullet de `.kimirules`/`.kimiignore`. `write_basic_agent_surface` y
  `.grok/GROK.md` intactos (el diff de `setup_harness.sh` solo toca arrays, el
  heredoc de la superficie completa, `required_assets` y el bloque de siembra).
- **AC-4** (paridad ps1): `kimi-cli-uso-eficiente.md` en `$script:HarnessDocs`
  (`setup_harness.ps1:83`) y en sus required assets (:436); UNA linea en
  ingles en `Write-AgentSurface` (:656-658: "Efficient Kimi Code CLI usage:
  see `docs/kimi-cli-uso-eficiente.md`..."). Completado en esta sesion: el loop
  de siembra de `$script:KimiDotfiles` (:1500-1511), declarado pero nunca
  usado en la sesion previa.
- **AC-5** (dogfooding): el `AGENTS.md` raiz enlaza la guia en "Archivos
  principales" (bullet propio, +3 lineas). `diff` repo vs
  `templates/docs/kimi-cli-uso-eficiente.md`: identicas (ver AC-1).
- **AC-6** (smoke sh): `bash tests/setup_smoke.sh` rc=0. Asserts nuevos:
  (a) guia sembrada en `docs/` en layout root (`tests/setup_smoke.sh:153-154`)
  y subdir (:215-216); (b) el `AGENTS.md` instalado contiene la linea de la
  guia (:232-234); (c) reset la limpia por ser `HARNESS_DOCS` (:463-465).
  Companero dotfiles: sembrados en root y subdir, y sobreviven al reset como
  documentos del usuario.
- **AC-7** (paridad ps1, estatica): `which pwsh` -> ausente en la maquina;
  revision estatica como en las features #1 y #4 a #10. El smoke ps1 espeja
  los asserts: siembra de la guia en el fixture root (:132), referencia en el
  `AGENTS.md` generado (:148-149), limpieza en reset (:175-177) y los mismos
  asserts de dotfiles (siembra root/subdir + supervivencia al reset).
- **AC-8** (docs al dia): `README.md` (arbol de archivos :283 y redaccion
  "los docs del arnes" en la seccion de refresco), `UPDATING.md` (lista de
  siembra :56-58 + linea de dotfiles, redaccion de `--reset` y de garantias
  PRD), `templates/UPDATING.md` (mismos tres puntos espejados + dotfiles) y
  `docs/architecture.md` (bullet `HARNESS_DOCS` incluye la guia, bullet propio
  de la guia y su enlace desde superficies, bullet `KIMI_DOTFILES`, `--reset`
  actualizado).
- **AC-9** (comandos oficiales de `docs/verification.md`):
  - `bash tests/setup_smoke.sh` -> rc=0 (incluye los asserts nuevos).
  - `(cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)` -> limpio.
  - `(cd rust && cargo test --locked)` -> 50 unit + 27 integracion, 0 fallos.
  - `bash harness_check.sh` -> rc=0 (tras re-firmar el plan con
    `harness_cli advance`; el gate de espejo de roles quedo intacto).
