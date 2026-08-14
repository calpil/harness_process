# Estado archivado - Feature #14: hub_batch_upserts_atomic_install
Cerrada: 2026-08-14T04:10:09Z - status=done - Hub por lotes con UNNEST y escritura solo del delta (2688 sentencias -> 4; 456,31 s -> 2,74 s medidos), candado por proyecto, DB_STATEMENT_TIMEOUT, e instalacion atomica del binario en sh y ps1 (adios SIGKILL en cada actualizacion)

---

# Feature #14: hub_batch_upserts_atomic_install

Estado: in_progress
Plan: docs/plan-feature-14-hub-batch-upserts-atomic-install.md
Spec: docs/spec-feature-14-hub-batch-upserts-atomic-install.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-14T03:49:12Z D1-D5 implementados: save() por lotes con UNNEST y solo el delta sucio, candado por proyecto y DB_STATEMENT_TIMEOUT. Plan completo escrito; se re-firman plan y spec.
- 2026-08-14T04:09:22Z D6-D9 completos: instaladores sh/ps1 con mv atomico, smoke de doble instalacion (inode nuevo) en sh y ps1, README + UPDATING (y su espejo en templates) documentando DB_STATEMENT_TIMEOUT, candado por proyecto y la ventana de transicion. Evidencia por AC en docs/impl-14.md y veredicto en docs/review-14.md. Medicion: 2688 sentencias = 456,31 s vs 4 por lote = 2,74 s.
