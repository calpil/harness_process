# Estado archivado - Feature #58: el_guard_no_bloquea_por_lo_que_escribe_el_arnes
Cerrada: 2026-08-22T17:33:32Z - status=done - 

---

# Feature #58: el_guard_no_bloquea_por_lo_que_escribe_el_arnes

Estado: in_progress
Plan: docs/plan-feature-58-el-guard-no-bloquea-por-lo-que-escribe-el-arnes.md
Spec: docs/spec-feature-58-el-guard-no-bloquea-por-lo-que-escribe-el-arnes.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-22T17:04:39Z Guard #58 implementado: exencion por artefacto Y por ubicacion (docs/), linea [i] cuando aplica, seis casos en el smoke sobre el guard instalado y A/B contra realestate. Un defecto propio encontrado en la revision: un impl-*.md dentro de un microservicio se eximia por nombre.
- 2026-08-22T17:24:40Z AC-9 queda sin Comando: el smoke se cuelga dentro de verify por la feature #46 (los pipes se leen despues de esperar al proceso). Reproducido: el instalador bloqueado 11 minutos escribiendo a un pipe lleno. Se verifica a mano.
- 2026-08-22T17:32:54Z AC-9 queda sin Comando ejecutable: el smoke imprime mas que el buffer del pipe y verify se cuelga (feature #46, reproducida: instalador bloqueado 11 minutos escribiendo a un pipe lleno). Se verifica a mano, con el smoke en exit 0.
