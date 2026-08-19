# Estado archivado - Feature #44: verify_detecta_filtro_vacio
Cerrada: 2026-08-19T00:42:37Z - status=done - verify mira la salida ademas del exit code y marca vacio al AC que salio 0 sin ejecutar ningun caso. Cerro el falso verde del AC-12 de la #28, medido de los dos lados: 175 ms en vacio con el test renombrado, 871 ms en verde con el test escrito, contra los 79 ms que tardaba en no correr nada. rojos_del_reporte dejo de comparar cadenas y deriva del enum.

---

# Feature #44: verify_detecta_filtro_vacio

Estado: in_progress
Plan: docs/plan-feature-44-verify-detecta-filtro-vacio.md
Spec: docs/spec-feature-44-verify-detecta-filtro-vacio.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-19T00:32:50Z Feature #44 implementada: verify mira la salida ademas del exit code y marca vacio al AC que salio 0 sin ejecutar ningun caso. Contrafactico medido de verdad: con el test renombrado, el AC-12 de la #28 pasa de verde a vacio (175 ms) y el cierre lo nombra; con el test escrito, vuelve a verde midiendo 871 ms contra los 79 ms que tardaba en no correr nada. rojos_del_reporte dejo de comparar cadenas y deriva del enum, que es la forma estructural del defecto de la #37. 17/17 AC verdes.
