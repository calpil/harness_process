# Estado archivado - Feature #24: conventions_escalera_y_tests
Cerrada: 2026-08-17T18:25:04Z - status=done - conventions.md pasa de 7 lineas a la escalera de huella (5 peldanos con ejemplo real de este repo) y las tres reglas de test. La feature se aplico su propia escalera (peldano 1: cero comandos, flags y dependencias) y pago en el acto la deuda que su regla descubre.

---

# Feature #24: conventions_escalera_y_tests

Estado: in_progress
Plan: docs/plan-feature-24-conventions-escalera-y-tests.md
Spec: docs/spec-feature-24-conventions-escalera-y-tests.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-17T18:13:39Z Plan de la #24 escrito: D1-D8 citando cada AC. La feature se aplica a si misma: pasa por su propia escalera (peldano 1, con la tabla de por que los otros cuatro no hacian falta) y paga en D3 la deuda que su propia regla descubre en el test de la #23. Los AC de documentacion se verifican con greps estructurales, no con tests de Rust: un test que grepea un markdown es el detector-de-cambios que la feature prohibe.
- 2026-08-17T18:24:47Z Feature #24 implementada: conventions.md lleva la escalera de huella (5 peldanos con ejemplo real) y las tres reglas de test; harness_check avisa (no bloquea) cuando un test lee el fuente. La feature se aplico su propia escalera (peldano 1) y pago la deuda que su regla descubre: el test de la #23 reescrito como contrato de comportamiento. Hallazgo: la regla del detector-de-cambios condeno tambien el test de compatibilidad que la #23 celebro, reescrito como invariante.
