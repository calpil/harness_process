# Spec - Feature #45: close_no_pisa_el_estado_de_la_activa

Estado: draft
Plan: docs/plan-feature-45-close-no-pisa-el-estado-de-la-activa.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan esta trabajando en la feature #36. A mitad de camino cierra la #27,
que es una deuda vieja que la #36 ya pago. `close` hace tres cosas que no
deberia hacer, porque el archivo `progress/current.md` **no es de la #27**:

1. **Archiva el estado de la #36 adentro de
   `docs/estado-feature-27-leccion-list-alineacion-dinamica.md`.** Ese archivo
   dice "Estado archivado - Feature #27" arriba y abajo tiene el estado de otra.
2. **Resetea `progress/current.md`** a "Sin feature activa", aunque la #36 sigue
   activa en `feature_list.json`.
3. **Borra `progress/.last_autocheck`**, que cierra el ciclo de checkpoints de un
   trabajo que no termino.

Y hay un cuarto efecto, que es el caro y el que no se ve: cuando despues se
cierra la #36 **de verdad**, `current.md` ya fue reseteado, asi que la guardia
`!content.contains("Sin feature activa")` se activa y **no archiva nada**.
`docs/estado-feature-36-deudas-anotadas-del-arnes.md` **no existe en este repo**.
El estado de la feature grande se perdio de su propio archivo y quedo adentro
del de una feature chica que nunca lo tuvo.

DESPUES: cerrar una feature que no es la activa no toca nada de la activa. La
#36 conserva su `current.md`, su stamp de checkpoints y, cuando le toque cerrar,
su propio archivo con su propio estado.

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES
close #27 (con la #36 activa)           close #27 (con la #36 activa)
  |__ archiva current.md como             |__ ¿la #27 es la activa? NO
  |   "estado-feature-27" (es de la #36)   |__ no archiva, no resetea,
  |__ resetea current.md                   |   no borra el stamp
  |__ borra .last_autocheck                |__ el cierre hace el resto igual
close #36 (la activa, despues)          close #36 (la activa, despues)
  |__ current.md ya estaba reseteado      |__ archiva SU estado en
      -> NO archiva nada. Se perdio.          "estado-feature-36"
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero cerrar una deuda vieja sin perder el estado del trabajo
  en curso, para que cerrar no sea una operacion peligrosa a mitad de camino.
- P1: Como Alan, quiero que la feature activa conserve su propio archivo cuando
  al fin cierre, que es lo que hoy se pierde en silencio.
- P2: Como Alan, quiero que el mensaje del cierre no diga "Estado archivado en
  X" cuando no archivo nada.

## Criterios de aceptacion (Given/When/Then)

### Cerrar una feature que NO es la activa

- AC-1: Given la #2 en `in_progress` con su estado escrito en
  `progress/current.md`, When se cierra la #1, Then `current.md` sigue
  describiendo la #2, byte a byte.
  Comando: `cd rust && cargo test closing_another_feature_should_not_reset_the_active_state`

- AC-2: Given lo mismo, When se cierra la #1, Then NO se crea
  `docs/estado-feature-1-*.md`: la #1 no tenia estado vivo que archivar.
  Comando: `cd rust && cargo test closing_another_feature_should_not_archive_someone_elses_state`

- AC-3: Given lo mismo y `progress/.last_autocheck` existente, When se cierra la
  #1, Then el stamp sigue ahi: el ciclo de checkpoints de la #2 no se cierra.
  Comando: `cd rust && cargo test closing_another_feature_should_not_clear_the_checkpoint_stamp`

- AC-4: Given lo mismo, When se cierra la #1, Then el mensaje NO dice "Estado
  archivado en": no se anuncia un archivo que no se escribio.
  Comando: `cd rust && cargo test closing_another_feature_should_not_announce_an_archive`

### El efecto caro: que la activa conserve SU archivo

- AC-5: Given la #2 activa, When se cierra la #1 y DESPUES se cierra la #2,
  Then `docs/estado-feature-2-*.md` existe y su cuerpo es el estado de la #2.
  Es la regresion que hoy hace desaparecer el archivo de la feature grande.
  Comando: `cd rust && cargo test the_active_feature_should_keep_its_own_archive_after_closing_another`

### Lo que NO cambia

- AC-6: Given la #1 activa (es ella la que se cierra), When se cierra, Then todo
  sigue igual que hoy: archiva su estado, resetea `current.md` y borra el stamp.
  Comando: `cd rust && cargo test closing_the_active_feature_should_archive_reset_and_clear_as_before`

- AC-7: Given ninguna feature activa (`current.md` dice "Sin feature activa"),
  When se cierra una, Then no se archiva nada y no hay error, como hoy.
  Comando: `cd rust && cargo test closing_with_no_active_feature_should_stay_quiet`

- AC-8: Given cualquiera de los casos, When se cierra, Then el resto del cierre
  —gates, PRD, bitacora, leccion, log, Atlassian— se comporta identico: esta
  feature toca SOLO el archivado del estado local.
  Comando: `cd rust && cargo test close_should_keep_every_other_effect_untouched`

### El dato ya danado, reparado

- AC-9: Given el repo real, When se revisan los archivos de estado archivado,
  Then ninguno guarda el estado de OTRA feature. Hoy hay exactamente uno:
  `estado-feature-27` guarda el de la #36, y `estado-feature-36` no existe. La
  reparacion MUEVE ese cuerpo a su archivo (OBS-1).
  Comando: `bash tests/estado_archivado_check.sh`

### Los de siempre

- AC-10: Given el plan, When se lo lee, Then declara `Peldano elegido:`.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-45-close-no-pisa-el-estado-de-la-activa.md`

- AC-11: Given la documentacion, When se busca, Then README y UPDATING (con su
  espejo) cuentan que cerrar una feature que no es la activa no toca el estado
  en curso.
  Comando: `grep -q "no es la activa" README.md UPDATING.md templates/UPDATING.md`

- AC-12: Given el arbol, When corre clippy con `-D warnings`, Then limpio.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: `close --feature <id>` con cualquier status.
- interruptor: ninguno. No es una preferencia: archivar el estado de A adentro
  del archivo de B es un dato equivocado.
- candado: la pregunta "¿la feature que se cierra es la activa?", contestada
  contra `feature_list.json` (`status == "in_progress"`), que es la misma fuente
  que `start` usa para prohibir dos activas a la vez.

## Pseudo-codigo (el acuerdo)

```
CUANDO se cierra la feature F

  ¿F estaba en in_progress?  -> si NO:
        no archivar, no resetear current.md, no borrar el stamp.
        (el resto del cierre sigue igual)

  ENTONCES, solo si F era la activa:
        archivar current.md, resetearlo y borrar el stamp,
        exactamente como hoy.
```
Promesas: no cambia nada del cierre de la feature activa · no inventa archivos ·
no borra ninguno de los que ya existen.

## No funcionales

- SLOs: una comparacion de string mas. Sin costo.
- Seguridad: reduce escrituras, no las agrega.
- Observabilidad: el mensaje deja de mencionar un archivo que no existe.

## Fuera de alcance

- Permitir dos features activas. `start` sigue prohibiendolo.
- Reconstruir el estado que ya se perdio de la #36 en su momento: lo que se
  puede reparar es DONDE vive el texto que sobrevivio, no resucitar lo que nunca
  se escribio.
- Cambiar el formato del archivo de estado.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por Alan, 2026-08-19): el cuerpo se **mueve** a
  `docs/estado-feature-36-deudas-anotadas-del-arnes.md`, que es de quien es, y el
  archivo de la #27 queda solo con su encabezado de cierre —que si le
  corresponde: dice que se cerro como `superseded` absorbida por la #36—. Se
  descarto duplicar el cuerpo en los dos archivos: dos copias del mismo texto
  vuelven a ser dos lugares donde puede quedar mintiendo.
