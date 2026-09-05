# Spec - Feature #71: El close archiva el sello de cierre en el worktree que acaba de borrar, y lo pierde

Estado: approved
Aprobado: 2026-09-05T03:57:40Z por USUARIO (confirmacion explicita) - Aprobado por Alan en chat, con OBS-1 decidida: el sello va al docs/ del repo principal
Plan: docs/plan-feature-71-el-close-archiva-el-sello-de-cierre-en-el-worktr.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md

## La historia (antes -> despues)

ANTES: Alan cierra la #124 en realestate. El arnes imprime "Estado archivado en
`docs/estado-feature-124-...md`" y sale 0. Dias despues busca ese archivo para
saber que se hizo y no existe: `docs/` es un repo aparte, el sello se escribio en
el `docs/` del worktree y el mismo cierre borro ese worktree. Con el se fue el
cuerpo de `progress/current-124.md`, que vive ADENTRO de ese archivo y no tiene
otra copia porque `progress/` esta gitignorado. Hubo que reconstruirlo a mano
desde `feature_list.json` y `history.md`; el cuerpo literal es irrecuperable.

DESPUES: el sello de cierre queda donde estan los otros cuarenta —el `docs/` del
repo principal— y el mensaje nombra esa ruta, que existe. Cerrar deja de ser una
forma de perder la unica copia de lo que se hizo.

## Lo que se midio antes de escribir esto

La ficha describe dos perdidas y son UNA. `archivar_estado`
(`rust/src/commands/close.rs:398`) copia el cuerpo entero de
`progress/current-<id>.md` dentro del archivo que escribe: si ese archivo
sobrevive, la evidencia sobrevive. Comprobado en el cierre de la #72, cuyo
`docs/estado-feature-72-*.md` conserva las 15 lineas de Evidencia.

Y el estado del bug cambio con la feature #72, que se cerro el 2026-09-05:

| Momento | Que pasa con un `docs/` que es repo aparte |
| --- | --- |
| Antes de la #72 | el sello se escribe en el `docs/` VACIO del worktree y el cierre borra el worktree: se PIERDE (caso #124) |
| Despues de la #72 | el repo docs tiene su propio worktree, que el cierre NO borra: el archivo sobrevive en `../docs-wt/<id>-<slug>/` |

O sea que la #72 tapo la perdida de datos sin proponerselo. Lo que sigue roto,
medido el 2026-09-05 con un fixture de repo docs aparte, es esto:

    Feature #1 cerrada como done. Estado archivado en docs/estado-feature-1-se-pierde.md.
    $ find . -name "estado-feature-1-*.md"
    ./docs-wt/1-se-pierde/estado-feature-1-se-pierde.md

El mensaje nombra una ruta que NO existe, y el archivo real quedo en una rama
del repo docs que nadie mergeo. Es exactamente "el cierre no declara hecho lo que
no hizo" (feature #62), un nivel mas abajo.

## Objetivos y no objetivos

- O-1: El sello de cierre queda en una ruta que existe despues del cierre.
- O-2: El mensaje nombra esa ruta, sin excepciones que haya que recordar.
- O-3: El cuerpo del estado vivo no se pierde al cerrar.
- NO-1: No se migran los sellos ya escritos ni se toca ningun cierre pasado.
- NO-2: No se integra ni se mergea el repo `docs/` del usuario (eso lo decide el).

## Hoy -> Como va a funcionar

```
HOY:     close -> escribe el sello en el docs/ de la FEATURE (fase 1)
                -> integra -> borra el worktree
                -> imprime una ruta canonica que puede no existir
DESPUES: close -> integra
                -> escribe el sello en el docs/ del repo PRINCIPAL (fase 3)
                -> imprime esa ruta, que es la que quedo en disco
```

## Recorridos de usuario (priorizados)

- P1: Alan cierra una feature con worktree y despues encuentra el sello en la
  ruta que el cierre nombro.
- P1: Alan cierra una feature en un proyecto donde `docs/` es un repo aparte y
  el sello queda igual de accesible que en cualquier otro proyecto.
- P2: Alan lee un sello viejo y sigue estando donde estaba: nada se migra.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una feature con worktree que se cierra integrando, When termina el
  `close`, Then el archivo `estado-feature-<id>-<slug>.md` EXISTE en el `docs/`
  del repo principal, y el test lo afirma comprobando el archivo, no el exit
  code del comando.
  Comando: `cd rust && cargo test --locked close_should_not_archive_the_state_into_the_worktree_it_deletes`
- AC-2: Given un proyecto donde `docs/` es un repo git aparte, When se cierra una
  feature con worktree, Then el sello queda en el `docs/` del repo principal —no
  en el worktree del repo docs, que vive en una rama sin integrar— y sobrevive al
  borrado del worktree de la feature.
- AC-3: Given cualquier cierre que archive el estado, When se imprime el mensaje,
  Then la ruta que nombra es la ruta real en disco. El caso especial de la
  feature #63 (`ruta_del_estado_archivado`, que elegia entre la ruta real y la
  canonica segun si el cierre borraba el worktree) desaparece: si las dos
  coinciden siempre, no hay entre que elegir. Si se conserva alguna forma de esa
  eleccion, el spec exige decir por que.
- AC-4: Given el cuerpo de `progress/current-<id>.md`, When el cierre lo archiva,
  Then queda integro dentro del sello, con sus lineas de Evidencia. Un test lo
  afirma sobre el CONTENIDO, no sobre la existencia del archivo.
- AC-5: Given que la integracion falla, When se aborta el cierre, Then no queda
  escrito un sello de cierre que afirme un cierre que no ocurrio. Mover la
  escritura despues de integrar es lo que lo consigue; no hace falta rollback.
- AC-6: Given los sellos ya escritos por cierres anteriores, When se instala este
  cambio, Then no se mueven, no se reescriben y no se borran.
- AC-7: Given el cambio completo, When se corre la suite, Then quedan verdes los
  tests, clippy, el smoke del instalador y el gate de paridad, y los espejos
  siguen coherentes.
  Comando: `cd rust && cargo test --locked`
- AC-8 (MANUAL): Given un lector del `docs/` del repo principal, When busca los
  sellos, Then los nuevos estan junto a los cuarenta que ya existen, sin una
  segunda ubicacion que haya que conocer.

## Los datos que se tocan

- `docs/estado-feature-<id>-<slug>.md`: el sello de cierre, con el cuerpo del
  estado vivo adentro. Pasa de vivir en el `docs/` de la feature al del repo
  principal.
- `progress/current-<id>.md`: se sigue borrando al cerrar, y sigue estando
  gitignorado. Su unica copia es el sello: por eso el sello no puede perderse.
- El ORDEN de las fases del `close`: el sello sale de la fase 1 (los artefactos
  que viajan en la rama) y pasa a la fase 3 (el estado, despues de integrar).

## Pseudo-codigo (el acuerdo)

```
AL CERRAR: validar lo que puede negarse
           escribir lo que viaja en la rama (el plan anotado)
           integrar  -> si falla, no hay sello escrito
           escribir el sello en el docs/ del repo PRINCIPAL
           imprimir ESA ruta
```

Promesas: no se nombra una ruta que no existe; no se archiva un cierre que no
ocurrio; no se migra nada de lo ya escrito.

## No funcionales y verificacion

- Verificacion: fixture con repo `docs` aparte y feature con worktree; el test
  comprueba que el ARCHIVO existe y que su CONTENIDO tiene la evidencia. Nada de
  afirmar sobre el exit code del cierre, que es justo lo que dejo pasar el bug.
- Prueba del rojo: el test tiene que fallar contra el codigo actual antes del
  arreglo (ya se comprobo: falla nombrando la ruta que falta).
- Compatibilidad: los sellos existentes no se tocan (AC-6).
- Limite conocido: el sello queda SIN COMMITEAR en el checkout principal, igual
  que la bitacora del PRD. El arnes no lo commitea solo; el cierre lo dice.

## Alcance de instalacion y fuera de alcance

Se corrige `harness_process`. No se distribuyen cambios a otros proyectos ni se
toca la instalacion de realestate en esta feature. No se recuperan los sellos ya
perdidos (el de la #124 es irrecuperable y se reconstruyo a mano en su momento).

## Observaciones (decisiones pendientes)

- OBS-1: DONDE queda el sello. La propuesta es el `docs/` del repo PRINCIPAL,
  porque es donde estan los otros cuarenta y porque un sello de feature CERRADA
  es historia compartida, no contenido de una rama —el mismo razonamiento que la
  feature #60 aplico a la bitacora del PRD—. La alternativa es escribirlo en los
  dos lados (rama y principal), que deja dos copias que pueden divergir: la
  familia de bug mas repetida de este repo. DECIDIDA por el usuario el
  2026-09-05: el `docs/` del repo PRINCIPAL.
- OBS-2: La rama de integracion se pregunta antes de `close --status done --to`.
