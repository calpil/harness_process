# Estado archivado - Feature #19: perfil_de_usuario
Cerrada: 2026-08-17T03:26:09Z - status=done - Perfil de usuario: docs/perfil-usuario.md con limite duro de 1500 chars que falla en vez de recortar, escritura solo con --yes, escaneo que bloquea secretos y unicode invisible antes de escribir, inyeccion idempotente en las 4 superficies reales como snapshot congelado, y perfil sugerir que junta la evidencia de history+planes+specs y emite el contrato sin escribir nada. 20 AC cubiertos, sin dependencias nuevas y sin tocar el hub.

---

# Feature #19: perfil_de_usuario

Estado: in_progress
Plan: docs/plan-feature-19-perfil-de-usuario.md
Spec: docs/spec-feature-19-perfil-de-usuario.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-16T23:48:31Z Plan de la #19 escrito: D1-D10 citando cada AC, impacto (hub caido), consulta al grafo (write_agent_surface + el patron de bloque entre marcadores de write_kimi_hooks + PRD_DOCS como lista de documentos del usuario) y riesgos. Las 5 observaciones quedaron decididas por Alan. Se instalaron y aplicaron 3 skills de Rust (best-practices, patterns, testing); rstest/proptest quedan fuera por ser dependencias nuevas sin ADR.
- 2026-08-17T03:25:52Z D1-D10 implementados: modulo perfil.rs (limite duro 1500 que falla en vez de recortar, Coincidencia como enum, escaneo de secretos y unicode invisible que bloquea, bloque para superficies, recolectar de history+planes+specs), comando perfil show|add|replace|remove|sugerir|check|bloque con --yes obligatorio, siembra via USER_DOCS e inyeccion idempotente en las 4 superficies en ambos instaladores, gate en harness_check, docs/roles y 28 tests nuevos. El pase de reviewer agrego tolerancia a binario viejo tras git pull; el dogfooding agrego el filtro de anti-senales.
