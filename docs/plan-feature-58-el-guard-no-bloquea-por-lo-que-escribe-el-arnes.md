# Plan - Feature #58: el_guard_no_bloquea_por_lo_que_escribe_el_arnes

Estado: in_progress
Microservicios:
- harness

## Alcance

`commit_guard.sh` deja de contar como "sucios" los documentos que escribio el
propio arnes, y sigue bloqueando por todo lo demas. Un solo archivo (mas su
plantilla) y casos en el smoke.

Fuera: cambiar como se decide que es un microservicio, y que el arnes commitee
sus documentos fuera del `close`.

## Impacto entre microservicios

`harness` solo. El guard corre en los proyectos INSTALADOS, asi que el cambio
viaja por `templates/commit_guard.sh` y llega re-corriendo el instalador.
Ningun consumidor del binario cambia: esto es bash puro.

## Consulta al grafo (graphify)

No aplica: el cambio es de 40 lineas en un script, sin dependencias.

## Delegacion (implementer)

1. `es_artefacto_del_arnes()`: la lista de patrones del arnes, exigiendo ademas
   la UBICACION (ruta bajo `docs/`, o el repo sucio ES `docs/`) — AC-1, AC-5.
2. `solo_artefactos_del_arnes()`: 0 solo si hubo cambios y TODOS son artefactos;
   maneja renombrados (`R old -> new`) y rutas entre comillas — AC-1, AC-4.
3. El bucle de `DIRTY`: cuando la exencion aplica, imprime la linea `[i]` con el
   repo y la razon en vez de sumarlo — AC-2; si no, bloquea como hoy — AC-3.
4. Espejo `templates/commit_guard.sh` identico — AC-6, Articulo 6.
5. Bloque nuevo en `tests/setup_smoke.sh` sobre el guard INSTALADO, con los seis
   casos — AC-9.
6. Prueba A/B contra `GolandProjects/realestate` — AC-10.

## Criterios de cierre (reviewer)

- Los 10 AC con evidencia; el smoke y el check limpios (AC-8).
- Intentar que el guard deje pasar algo que no debe: nombres de artefacto fuera
  de `docs/`, artefactos modificados vs sin trackear, mezclas en un mismo repo.
- Que el espejo de la plantilla sea identico.

## Riesgos

- **Que el guard deje de mirar algo real**: mitigado exigiendo ubicacion ademas
  de nombre, y con el caso del microservicio en el smoke.
- **Que la lista de patrones envejezca**: si el arnes suma un tipo de documento
  nuevo, hay que sumarlo aca. Anotado como reparo en el veredicto.

## Observaciones (decisiones pendientes)

- OBS-1 [DECIDIDA]: exencion por ARTEFACTO, no por carpeta.
- OBS-2 [DECIDIDA]: cuando aplica, se dice en una linea `[i]`.

### Avance 2026-08-22T17:04:39Z
Guard #58 implementado: exencion por artefacto Y por ubicacion (docs/), linea [i] cuando aplica, seis casos en el smoke sobre el guard instalado y A/B contra realestate. Un defecto propio encontrado en la revision: un impl-*.md dentro de un microservicio se eximia por nombre.

### Avance 2026-08-22T17:24:40Z
AC-9 queda sin Comando: el smoke se cuelga dentro de verify por la feature #46 (los pipes se leen despues de esperar al proceso). Reproducido: el instalador bloqueado 11 minutos escribiendo a un pipe lleno. Se verifica a mano.

### Avance 2026-08-22T17:32:54Z
AC-9 queda sin Comando ejecutable: el smoke imprime mas que el buffer del pipe y verify se cuelga (feature #46, reproducida: instalador bloqueado 11 minutos escribiendo a un pipe lleno). Se verifica a mano, con el smoke en exit 0.

---
Cerrado: 2026-08-22T17:33:32Z - status=done - 
