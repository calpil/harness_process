# Estado archivado - Feature #46: verify_no_se_cuelga_con_salida_grande
Cerrada: 2026-08-22T17:58:57Z - status=done - 

---

# Feature #46: verify_no_se_cuelga_con_salida_grande

Estado: in_progress
Plan: docs/plan-feature-46-verify-no-se-cuelga-con-salida-grande.md
Spec: docs/spec-feature-46-verify-no-se-cuelga-con-salida-grande.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-22T17:48:28Z AC-8 pasa a declarar 'bash tests/setup_smoke.sh': con los lectores en hilos el comando ya no cuelga el gate. Timeout del repo a 900s porque el smoke tarda ~6 minutos reales.
- 2026-08-22T17:58:14Z Feature #46 lista: lectores en hilos con tope de 4 MB y gracia de 2s, seis tests que antes se colgaban mas el del nieto, y el smoke declarado como AC-8 corriendo verde en 63s dentro de verify.
