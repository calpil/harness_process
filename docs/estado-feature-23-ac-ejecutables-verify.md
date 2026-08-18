# Estado archivado - Feature #23: ac_ejecutables_verify
Cerrada: 2026-08-17T05:52:08Z - status=done - AC ejecutables: el spec declara Comando: por criterio, verify los corre (20/20 verde sobre su propio spec) y close exige el reporte verde y fresco. Tres barreras: spec aprobado, invocacion manual, comando impreso; el cierre LEE, nunca ejecuta.

---

# Feature #23: ac_ejecutables_verify

Estado: in_progress
Plan: docs/plan-feature-23-ac-ejecutables-verify.md
Spec: docs/spec-feature-23-ac-ejecutables-verify.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-17T05:13:19Z Plan de la #23 escrito: D1-D8 citando cada AC. El riesgo central es distinto al de las features anteriores: es la primera vez que el binario ejecuta comandos arbitrarios, y las tres barreras (spec aprobado, invocacion manual, comando impreso) estan en los AC y no en la prosa. wait-timeout ya es dependencia, asi que el timeout no agrega ninguna (Articulo 6 sin ADR).
- 2026-08-17T05:51:49Z Feature #23 implementada: verify ejecuta los Comando: que declaran los AC y close los exige con require_verify_green (encendida en este repo por decision de Alan). Dos hallazgos de la corrida real sobre su propio spec: el parser ejecutaba el ejemplo del bloque ``` que ENSENA el formato, y 8 de 20 AC daban verde sin correr nada porque cargo test con un filtro sin coincidencias sale 0. Los dos arreglados y escritos en la leccion.
