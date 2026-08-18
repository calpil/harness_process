# Estado archivado - Feature #26: rutas_protegidas_deny
Cerrada: 2026-08-18T01:16:36Z - status=done - Rutas protegidas con tres capas y su alcance declarado: PreToolUse previene (solo Claude, limite de prueba declarado), PostToolUse avisa con el comando de reversion, harness_check bloquea con exit 2. El arnes queda exento de su propia proteccion por registro con mtime, que caduca en cuanto alguien vuelve a tocar el archivo.

---

# Feature #26: rutas_protegidas_deny

Estado: in_progress
Plan: docs/plan-feature-26-rutas-protegidas-deny.md
Spec: docs/spec-feature-26-rutas-protegidas-deny.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-17T20:14:40Z Plan de la #26 escrito: D1-D8 citando cada AC. La escalera contradijo al PRD (peldano 1, rules.rutas_protegidas, en vez del archivo harness.deny que proponia) y el diseno cambio dos veces por hechos verificados: PostToolUse corre DESPUES y no puede prevenir, y close escribe en docs/prd/PRD-master.md, que es la primera ruta a proteger, asi que la proteccion es contra las herramientas del agente y no contra el binario del arnes.
- 2026-08-18T01:16:25Z Feature #26 implementada: rutas protegidas con tres capas (PreToolUse previene en Claude, PostToolUse avisa con el comando de reversion, harness_check bloquea con exit 2), lista en rules.rutas_protegidas y el arnes exento de su propia proteccion por registro con mtime. Incidente grave y corregido: el remedio que la herramienta imprimia (git checkout -- ) se corrio y borro los hitos sin commitear de #23-#25; reconstruidos y verificados con prd tree. El remedio ahora muestra el diff primero y etiqueta que DESCARTA.
