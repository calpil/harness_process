# Plan - Feature #73: verify corre UN comando por AC y no lo dice

Estado: in_progress
Microservicios:
- harness_process (rust: verificacion/verify)

## Alcance

Un cambio de MODELO —de `Option<String>` a `Vec<String>`— y sus consecuencias.
El resto sale solo: el bucle de ejecucion, el reporte y la deduplicacion.

## Peldano de huella

`Peldano elegido: 1 (extender lo que existe) porque el arreglo es cambiar la
cardinalidad de un campo que ya existe.` No hay flag, comando ni sintaxis nueva:
el spec se sigue escribiendo igual, una linea `Comando:` por verificacion.

## Delegacion (implementer)

- D-1 (AC-1, AC-6): `Verificacion.comando: Option<String>` pasa a `comandos:
  Vec<String>`, con `es_manual()` para que los call sites no pregunten por el
  largo. `parsear` acumula en vez de quedarse con el primero; la guarda
  `!ac_ilegible` de la #68 no se toca.
- D-2 (AC-1, AC-2): el bucle de `verify` emite un `Resultado` POR COMANDO.
- D-3 (AC-3): `sin_repetir` — UNA funcion que deduplica nombres de AC, usada por
  el mensaje de `verify` y por `rojos_del_reporte` que lee el gate. Dos listas
  distintas de "AC en rojo" seria la divergencia de siempre.
- D-4 (AC-1): arreglar el ORACULO del test del corpus, que contaba solo el
  primer comando "como en `parsear`" — o sea que imitaba el bug.
- D-5 (AC-4, AC-5, AC-7): tests de no-regresion para el AC de un comando, el AC
  manual y los reportes ya escritos.

## Criterios de cierre (reviewer)

- Cada mutacion (volver al primer comando, sacar la deduplicacion) pone en rojo
  al menos un test.
- El test del corpus real tiene que DETECTAR el bug con la mutacion puesta, no
  solo pasar sin ella.

## Riesgos

- R-1: el reporte gana filas. El gate las parsea fila por fila, asi que no se
  rompe; se comprueba con un test.
- R-2: un AC con N comandos tarda la suma de los N. Declarado en el spec.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por el usuario 2026-09-05): se corren TODOS los comandos, con
  una fila por comando. Alternativa descartada: conservar uno por AC y solo
  avisar, que deja al autor partiendo el AC a mano.
- OBS-2: la rama de integracion se pregunta antes de `close --status done --to`.

---
Cerrado: 2026-09-05T12:18:34Z - status=done - Un AC verifica con todos los comandos que declara; el oraculo del test del corpus dejaba de imitar al parser
