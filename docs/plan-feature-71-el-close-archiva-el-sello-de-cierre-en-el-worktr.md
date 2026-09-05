# Plan - Feature #71: El close archiva el sello de cierre en el worktree que acaba de borrar, y lo pierde

Estado: in_progress
Microservicios:
- harness_process (rust: close/archivado de estado)

## Alcance

Un cambio de UBICACION y uno de ORDEN, y la eliminacion del caso especial que
existia para tapar el primero.

## Peldano de huella

`Peldano elegido: 1 (extender lo que existe) porque el arreglo es mover donde
escribe una funcion que ya existe y correrla una fase mas tarde.` No hace falta
flag, comando ni superficie: no hay nada nuevo que el usuario tenga que aprender.

## Delegacion (implementer)

- D-1 (AC-1, AC-2): `archivar_estado` escribe en `raiz_del_prd(paths)/docs` en
  vez de `paths.plans`. Es la misma raiz que ya usa la vuelta al PRD, o sea que
  no se inventa un concepto nuevo de "la raiz".
- D-2 (AC-5): la llamada se mueve de la FASE 1 a la FASE 3, despues de
  `ejecutar_integracion`. Estaba en la fase 1 por una razon que dejo de existir.
- D-3 (AC-3): se borra `ruta_del_estado_archivado` y su test. Con una sola ruta
  posible no hay entre que elegir.
- D-4 (AC-1, AC-3, AC-4, AC-5): tests de comportamiento con fixture de repo docs
  aparte y con merge conflictivo.
- D-5 (AC-6): nada de migracion. Ni una linea que toque un sello ya escrito.

## Criterios de cierre (reviewer)

- Los tests afirman sobre el ARCHIVO y su CONTENIDO, nunca sobre el exit code
  del cierre: el exit code ya era 0 con el bug.
- Las dos mutaciones (volver la ubicacion, volver el orden) tienen que poner en
  rojo un test cada una.

## Riesgos

- R-1: el sello deja de viajar en la rama y queda sin commitear en la raiz. Es
  el mismo trato que la bitacora del PRD. El cierre lo dice en el mensaje.
- R-2: un test de la #72 apuntaba al `docs/` del worktree; vuelve a la raiz, que
  es donde apuntaba antes de la #72.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por el usuario 2026-09-05): el sello va al `docs/` del repo
  PRINCIPAL. Alternativas descartadas: escribirlo en los dos lados (deja dos
  copias que pueden divergir, la familia de bug mas repetida del repo) y
  arreglar solo el mensaje (el archivo seguiria en una rama que nadie mergea).
- OBS-2: la rama de integracion se pregunta antes de `close --status done --to`.

---
Cerrado: 2026-09-05T04:20:22Z - status=done - El sello de cierre va al docs de la raiz y despues de integrar; el caso especial de la #63 se elimina porque ya no hay dos rutas
