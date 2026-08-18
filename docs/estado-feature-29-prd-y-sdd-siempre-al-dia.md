# Estado archivado - Feature #29: prd_y_sdd_siempre_al_dia
Cerrada: 2026-08-18T13:17:46Z - status=done - El PRD, el SDD y architecture.md dejan de poder quedar mintiendo: el binario calcula el alcance desde el arbol real, siembra una pregunta por documento, verifica las citas contra el disco, y solo con el SI del usuario prd apply --yes escribe. Aplicada sobre este repo: corrigio el drift real de architecture.md y el SDD que se publicaba a Confluence con placeholders.

---

# Feature #29: prd_y_sdd_siempre_al_dia

Estado: in_progress
Plan: docs/plan-feature-29-prd-y-sdd-siempre-al-dia.md
Spec: docs/spec-feature-29-prd-y-sdd-siempre-al-dia.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-18T13:14:59Z Feature #29 implementada: prd propose siembra una pregunta por documento (alcance calculado por el binario desde el arbol real), el agente contesta con cambio/ya-esta/no-aplica, el binario VERIFICA las citas contra el disco, y solo con el SI del usuario prd apply --yes escribe. Aplicada sobre este repo: architecture.md ya documenta doctor.rs, rutas.rs y documentos.rs, y el SDD dejo de publicar <nombre del proyecto> a Confluence. Bug encontrado dogfooding y arreglado: la idempotencia por contenido fallaba cuando Despues contiene a Antes (el patron 'insertar antes de esta linea') y DUPLICABA el texto.
