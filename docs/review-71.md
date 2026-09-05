# Review - Feature #71: El close archiva el sello de cierre en el worktree que acaba de borrar, y lo pierde
Revisado: approved · 2026-09-05T04:19:17Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento. Metodo: reproducir el bug con un
fixture real ANTES de tocar codigo, y despues mutar produccion para confirmar
que cada test detecta el bug que dice detectar.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/commands/close.rs:438 | CUBIERTO. El sello se escribe en `raiz_del_prd(paths)/docs`. El test de rust/tests/cli_basics.rs:8014 afirma que el archivo EXISTE tras el cierre, que es lo que la ficha pedia explicitamente ("no que el close salio 0"). |
| AC-2 | rust/tests/cli_basics.rs:8014 | CUBIERTO. El fixture tiene `docs/` como repo git aparte —el layout de realestate, el que perdio la #124— y el sello queda en el docs del principal. Probado en rojo: devolviendo la escritura a `paths.plans`, FALLA. |
| AC-3 | rust/src/commands/close.rs:538 | CUBIERTO. `ruta_del_estado_archivado` se BORRO en vez de dejarla devolviendo siempre lo mismo, y su test se fue con ella (rust/tests/cli_basics.rs:8014 cubre la propiedad por comportamiento: resuelve contra el disco la ruta que el cierre imprimio). El spec exigia decir por que si se conservaba alguna forma de esa eleccion; no se conservo ninguna. |
| AC-4 | rust/tests/cli_basics.rs:8014 | CUBIERTO. El test afirma sobre el CONTENIDO (`Evidencia` adentro del sello) y ademas que `progress/current-1.md` se borro, que es lo que hace del sello la unica copia. |
| AC-5 | rust/src/commands/close.rs:262 | CUBIERTO. La escritura paso a la fase 3. El test de rust/tests/cli_basics.rs:8064 fuerza un conflicto de merge REAL —las dos ramas tocan la misma linea— y afirma que no queda sello. Probado en rojo: devolviendola a la fase 1, FALLA. |
| AC-6 | rust/src/commands/close.rs:440 | CUBIERTO. Ni una linea de migracion: `estado-feature` aparece UNA sola vez en todo `close.rs`, en la escritura del sello nuevo. Ningun otro modulo escribe ese archivo; `rust/src/buscar.rs:123` solo lo CLASIFICA al indexar. |
| AC-7 | rust/tests/cli_basics.rs:8014 | CUBIERTO. Suite completa, clippy limpio, smoke del instalador y gate de paridad (diez modos), mas los ocho de commit_guard y los diez de stop_hook. |
| AC-8 | rust/src/commands/close.rs:438 | CUBIERTO. Los sellos nuevos caen en `<raiz>/docs/`, exactamente donde estan los 63 anteriores (contados el 2026-09-05; el spec dice "cuarenta" de memoria y la cifra real es esa). No hay segunda ubicacion. |

## Lo que la ficha decia y no era asi

La ficha afirmaba dos perdidas independientes: el sello y "las lineas de
Evidencia porque `progress/current-<id>.md` se borra al cerrar y no queda
copia". Medido: `archivar_estado` copia el cuerpo entero del estado vivo DENTRO
del sello, asi que hay una sola perdida con dos consecuencias. Se corrigio en el
spec y en el impl en vez de repetir la ficha.

## Lo que cambio bajo nuestros pies

La feature #72, cerrada horas antes, le dio al repo `docs/` su propio worktree y
dejo de borrarlo. Eso tapo la PERDIDA de datos sin proponerselo. Lo que quedaba
al empezar la #71, medido con fixture el 2026-09-05, era que el cierre nombraba
`docs/estado-feature-1-se-pierde.md` mientras el archivo estaba en
`docs-wt/1-se-pierde/`, en una rama que nadie mergea. El review lo dice porque
cambia lo que esta feature arregla de verdad: no evita una perdida que ya no
ocurria, evita una MENTIRA y saca el archivo de una rama muerta.

## Riesgos que el cambio introduce, dichos

- **El sello ya no viaja en el merge.** Queda sin commitear en la raiz, igual
  que la bitacora del PRD. El cierre lo anuncia en su mensaje; no se commitea
  solo. **Seis tests existentes** tuvieron que cambiar, y ninguno se borro:

  | Test | De que feature | Que cambio |
  | --- | --- | --- |
  | `cierre_exitoso_hace_todo_lo_de_siempre` | #62 | comprobaba el sello con `git show main:docs/...`; ahora en disco |
  | `estado_archivado_apunta_a_donde_quedo_el_archivo` | #63 | idem: la ruta impresa se resuelve contra el filesystem |
  | `estado_archivado_sin_integrar_mantiene_la_ruta_real` | #63 | afirmaba que sin integrar la ruta que vale es la del worktree; esa distincion desaparecio, y ahora ademas exige que NO quede copia en el worktree |
  | `close_should_archive_current_state_and_reset_it` | base | el mensaje ahora dice ademas "sin commitear" |
  | `close_should_not_touch_the_state_of_the_other_active_feature` | #45/#72 | la #72 lo habia apuntado al docs del worktree; vuelve a la raiz |
  | `los_siete_que_faltaban_y_ninguno_mas` | #68 | el corpus real gano un AC con la forma `- AC-8 (MANUAL):` |

  Los cinco primeros siguen cuidando exactamente lo mismo que cuidaban —que el
  archivo exista y lleve sello, que la ruta impresa sea real—; lo que cambio es
  DONDE hay que mirarlo, y uno de ellos quedo mas fuerte que antes. El sexto es
  una medicion del corpus, no una asercion sobre este codigo.

- **Se borro un test**: `ruta_del_estado_archivado_es_pura` (#63), junto con la
  funcion que probaba. Se deja dicho en su lugar
  (rust/src/commands/close.rs:538) y la propiedad pasa a probarse por
  comportamiento.
- **Sobre el ultimo de la tabla**: `los_siete_que_faltaban_y_ninguno_mas` paso a
  esperar nueve entradas porque el `AC-8 (MANUAL)` de ESTA feature tiene la forma
  que el parser viejo descartaba. No es un ajuste para que pase: es la medicion
  del corpus real, y que el AC nuevo aparezca ahi es evidencia de que el arreglo
  de la #68 sigue haciendo falta en el uso normal.

## Lo que NO esta verificado

- **AC-8 es MANUAL**: que los sellos nuevos queden "junto a los cuarenta" se
  comprueba mirando el directorio, y el test solo puede afirmar la ruta. En este
  repo se verifico a mano: `ls docs/estado-feature-*.md | wc -l` da 63, y el
  proximo cierre cae en el mismo directorio.
- **No se probo en realestate.** El AC-2 se prueba con un fixture que reproduce
  su layout, no contra su instalacion: esta feature no toca otros proyectos.

## Veredicto

Los ocho AC tienen cobertura. Los dos arreglos —ubicacion y orden— tienen cada
uno su mutacion que los pone en rojo, y el caso especial que existia para tapar
el primero se elimino en lugar de quedar como codigo muerto que devuelve siempre
lo mismo.
