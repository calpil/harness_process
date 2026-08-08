# Estado archivado - Feature #11: link_kimi_guide_in_surfaces
Cerrada: 2026-08-07T16:15:48Z - status=done - Guia kimi-cli-uso-eficiente sembrada como HARNESS_DOCS (templates + arrays sh/ps1) y enlazada desde superficies sh/ps1 y AGENTS.md raiz (dogfooding). Companero: siembra de dotfiles .kimiignore/.kimirules como docs del usuario (loop ps1 completado). Smoke sh rc=0 con asserts nuevos, clippy limpio, cargo test 50+27 verdes, harness_check rc=0, diff guia repo/template identica. AC-7 estatico (sin pwsh). docs/impl-11.md y docs/review-11.md (approved). graph impacto no corrio: hub inalcanzable (impacto verificado manual contra git status/plan)

---

# Feature #11: link_kimi_guide_in_surfaces

Estado: in_progress
Plan: docs/plan-feature-11-link-kimi-guide-in-surfaces.md
Spec: docs/spec-feature-11-link-kimi-guide-in-surfaces.md

Microservicios:
- harness

Evidencia:
- docs/impl-11.md (evidencia por AC-1..AC-9)
- Verificacion: `bash tests/setup_smoke.sh` rc=0 (asserts nuevos: guia en docs/
  root+subdir, enlace en AGENTS.md instalado, reset la limpia; dotfiles
  .kimiignore/.kimirules sembrados y sobreviven al reset);
  `cargo clippy --locked -- -D warnings` limpio; `cargo test --locked` 50+27
  verdes; `diff` guia repo vs templates identica.
- Nota de proceso: una sesion previa dejo U1/U2/U4/U5-parcial/U6-parcial en el
  working tree sin registrar; se reconstruyo el estado desde `git diff` y se
  completo U3 (loop `$script:KimiDotfiles` en ps1, declarado pero nunca usado),
  asserts de dotfiles en ambos smoke y U6 (architecture.md, templates/UPDATING.md).
- Cambio companero documentado en impl-11: siembra de `.kimiignore`/`.kimirules`
  (KIMI_DOTFILES) como documentos del USUARIO; la guia (contenido congelado por
  el spec) afirma que el instalador siembra `.kimiignore`.
- Pendiente: veredicto del reviewer (docs/review-11.md) y cierre.
