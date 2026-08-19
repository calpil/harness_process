# Spec - Feature #37: estado_superseded

Estado: approved
Aprobado: 2026-08-18T22:55:04Z por USUARIO (confirmacion explicita) - Alan aprobo en el chat tras el ritual. OBS-1: una feature superseded no cuenta ni en el numerador ni en el denominador de prd tree. OBS-2: el flag es --absorbida-por, en espanol como el resto; el campo del dato sigue siendo superseded_by.
Plan: docs/plan-feature-37-estado-superseded.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: seis entradas del backlog (#27, #31-#35) figuran como **`blocked`**, y no
lo estan. Su trabajo **esta hecho y verificado**, solo que dentro de otra feature
(la #36, que las agrupo por decision de Alan). Se cerraron asi porque `done`
exige spec aprobado —y nunca tuvieron spec propio— y `pending` las haria volver
en `next`.

O sea: el arnes no tiene una palabra para "esto ya se hizo, pero en otro lado".
Las consecuencias se ven:

- `status` las lista como bloqueadas, sugiriendo un problema donde no lo hay.
- `prd tree` cuenta **19/21 done** en el maestro: las seis inflan el denominador
  sin sumar al numerador, porque `feature_counts` solo cuenta `done`
  (`prd.rs:686`).
- La unica trazabilidad de que la #36 las absorbio es una nota en prosa dentro
  del campo `note`.

DESPUES: existe el estado **`superseded`**, y cerrar con el **exige decir cual
feature absorbio el trabajo**. Deja de ser una nota que alguien escribio bien y
pasa a ser un campo que se puede consultar.

## Hoy -> Como va a funcionar

```
HOY                                DESPUES
#31 [blocked] close_exit_codes     #31 [superseded por #36] close_exit_codes
  -> parece un problema              -> se lee: "hecho en otra feature"
  -> infla el denominador del PRD    -> no cuenta ni arriba ni abajo
  -> la trazabilidad es prosa        -> superseded_by: 36, consultable
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero mirar `status` y distinguir de un vistazo lo que esta
  trabado de lo que ya se hizo en otro lado.
- P1: Como Alan, quiero que `prd tree` no me diga que faltan seis cosas que en
  realidad estan hechas.
- P2: Como agente, quiero poder cerrar una entrada absorbida sin mentir: ni
  `done` (no tuvo spec), ni `blocked` (no esta trabada).

## Criterios de aceptacion (Given/When/Then)

<!-- Comportamiento con tests; documentacion con greps. Ningun comando repetido. -->

### El estado nuevo

- AC-1: Given `close --status superseded`, Then el valor se acepta: `superseded`
  se suma a los que valida clap (`cli.rs:38`), junto a `done`, `blocked` y
  `pending`.
  Comando: `cd rust && cargo test close_should_accept_the_superseded_status`
- AC-2: Given `close --status superseded` **sin** decir cual feature absorbio el
  trabajo, Then exit 2: la trazabilidad es el punto entero del estado.
  Comando: `cd rust && cargo test superseded_should_demand_the_absorbing_feature`
- AC-3: Given `--absorbida-por <id>` con una feature que **no existe**, Then exit
  2 nombrandola: una referencia rota es peor que ninguna.
  Comando: `cd rust && cargo test superseded_should_refuse_an_unknown_absorber`
- AC-4: Given un cierre valido, Then queda `superseded_by: <id>` en la entrada de
  `feature_list.json`, consultable sin leer prosa.
  Comando: `cd rust && cargo test superseded_should_record_the_absorbing_feature`

### No pasa por los gates de `done`, y con razon

- AC-5: Given `--status superseded`, Then **no** exige spec aprobado, ni leccion,
  ni reporte de verify, ni propuesta de documentos: el trabajo y su evidencia
  viven en la feature que la absorbio.
  Comando: `cd rust && cargo test superseded_should_not_trigger_the_done_gates`
- AC-6: Given una feature `superseded`, When corre `next`, Then **no** se ofrece:
  no es trabajo pendiente.
  Comando: `cd rust && cargo test next_should_not_offer_a_superseded_feature`

### Se lee distinto

- AC-7: Given `status`, Then una feature superseded se muestra **nombrando quien
  la absorbio**, no como un estado suelto.
  Comando: `cd rust && cargo test status_should_show_who_absorbed_a_superseded_feature`
- AC-8: Given `prd tree`, Then una feature superseded **no cuenta ni en el
  numerador ni en el denominador**: no es trabajo hecho ni pendiente, es una
  entrada que se pleg en otra.
  Comando: `cd rust && cargo test prd_tree_should_ignore_superseded_features`
- AC-9: Given `journey`, Then una feature superseded no aparece como cierre sin
  leccion: su aprendizaje se declaro en la que la absorbio.
  Comando: `cd rust && cargo test journey_should_not_flag_a_superseded_feature`

### Las seis de hoy

- AC-10: Given las seis entradas que la #36 absorbio (#27, #31-#35), Then quedan
  en `superseded` con `superseded_by: 36`, y `prd tree` deja de contarlas.
  Comando: `bash tests/superseded_check.sh migradas`
- AC-11: Given una instalacion con features `blocked` de verdad, Then **nada
  cambia** para ellas: la migracion es explicita, no automatica.
  Comando: `cd rust && cargo test blocked_features_should_stay_blocked`

### Integracion, docs y verificacion

- AC-12: Given `README.md` y `UPDATING.md` (+ espejo), Then documentan el estado,
  cuando usarlo y por que no es ni `done` ni `blocked`.
  Comando: `grep -q "superseded" README.md UPDATING.md templates/UPDATING.md`
- AC-13: Given el rol del reviewer, Then dice que una entrada absorbida se cierra
  con `superseded --absorbida-por <id>`, no con `blocked`.
  Comando: `grep -q "absorbida-por" roles/reviewer.md templates/roles/reviewer.md`
- AC-14: Given el plan, Then declara `Peldano elegido:` con su razon.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-37-estado-superseded.md`
- AC-15: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `tests/setup_smoke.sh`, `tests/parity_check.sh` y `harness_check.sh` siguen
  verdes.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: `close --status superseded --absorbida-por <id>`.
- interruptor: no aplica; es un valor mas, y no cambia nada de lo existente.
- candado: la referencia se valida contra el backlog antes de escribirse.

## Pseudo-codigo (el acuerdo)

```
CUANDO se cierra con --status superseded
  ¿vino --absorbida-por?           -> si no, exit 2
  ¿esa feature existe?             -> si no, exit 2 nombrandola
  se escribe status + superseded_by
  NO se pasa por los gates de done (spec, leccion, verify, documentos)

CUANDO se lee
  status    -> "[superseded por #36]"
  next      -> no la ofrece
  prd tree  -> no la cuenta, ni arriba ni abajo
  journey   -> no la reporta como cierre sin leccion
```

## No funcionales

- Sin dependencias nuevas (Articulo 6) y sin comandos nuevos: es un valor mas de
  un flag que ya existe.
- Compatibilidad total: una instalacion que nunca use el estado se comporta
  exactamente como hoy.

## Fuera de alcance

- **Migrar automaticamente** features `blocked` a `superseded`: el arnes no puede
  saber cuales estaban absorbidas y cuales trabadas de verdad (AC-11).
- Un estado para "abandonada" o "descartada": son otra cosa y no hay caso real.
- Cambiar el significado de `blocked`.

## Observaciones (decididas por Alan el 2026-08-18)

- OBS-1 **DECIDIDA: una feature superseded no cuenta ni en el numerador ni en el
  denominador de `prd tree`.** No es trabajo hecho ni pendiente: es una entrada
  que se plego en otra. El maestro pasa de "19/21" a "19/19", que es la verdad.
  Contarla como `done` inflaria el numerador con features que nunca tuvieron
  spec ni evidencia propia. -> AC-8.
- OBS-2 **DECIDIDA: el flag es `--absorbida-por <id>`**, en espanol como el resto
  (`--leccion`, `--leccion-motivo`, `--nota`). El campo que queda en
  `feature_list.json` sigue siendo `superseded_by`, que es el vocabulario del
  dato. -> AC-2, AC-4.
