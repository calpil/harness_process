# Spec - Feature #28: consolidacion_de_lecciones_con_llm

Estado: approved
Aprobado: 2026-08-18T14:15:34Z por USUARIO (confirmacion explicita) - Alan aprobo en el chat tras el ritual. OBS-1: el camino HTTP con API key queda fuera de alcance, nombrado en el mensaje de skip; la cadena es override -> CLI -> skip. OBS-2: se fusionan de verdad las dos lecciones reales, con el paraguas mostrado y aprobado por separado. OBS-3: la confianza se reporta sin filtrar, porque con 9 lecciones no hay zona gris con que calibrar un umbral.
Plan: docs/plan-feature-28-consolidacion-de-lecciones-con-llm.md
PRD: docs/prd/aprendizaje/PRD-aprendizaje.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: la guia del arnes manda **patchear el paraguas existente** antes de crear
una leccion nueva (`COMO-ESCRIBIR-UNA-LECCION.md`, paso 2) y cierra pidiendo
"pocas lecciones de clase, ricas... no una lista larga y plana". El curador (#21)
detecta lecciones frias y las archiva, pero **no ve solapamientos**: dos lecciones
que cuentan lo mismo pueden vivir para siempre una al lado de la otra.

Y ya paso. Medido sobre las **9 lecciones reales** de este repo:

- `docs-generados-por-el-instalador` y `documentos-del-usuario-vs-plantillas`
  comparten **4 de sus triggers** (Jaccard 0.400), el **mismo pitfall del `.ps1`
  casi palabra por palabra**, la misma regla ("lo que no esta en ninguna lista
  sobrevive") y el **mismo bloque de Verificacion**.
- La mas nueva de las dos **declara el solapamiento en su propia prosa**: *"La
  leccion [[docs-generados-por-el-instalador]] explica el mecanismo; esta explica
  cual lista"*. Alguien vio el paraguas, decidio que era el mismo territorio, y
  aun asi creo un archivo nuevo.
- **Ningun otro par se le acerca**: el segundo Jaccard mas alto es 0.050, y 36 de
  los 45 pares tienen interseccion vacia.

DESPUES: `lecciones consolidar` le pregunta a un LLM cuales lecciones se solapan
—viendo **solo nombre, descripcion y triggers**, nunca el cuerpo— e informa. Con
`--aplicar` y los argumentos explicitos de la fusion, el binario mueve las
miembros a `archivo/` con backup y rollback. **El modelo nunca escribe**, el
paraguas lo redacta una persona, y nada se borra.

## Hoy -> Como va a funcionar

```
HOY                                DESPUES
9 lecciones, dos de ellas casi     sh harness_cli lecciones consolidar
la misma, y nadie se entera          -> [i] candidato: docs-generados-por-el-instalador
                                          + documentos-del-usuario-vs-plantillas
                                          motivo: <una oracion del modelo>
                                     (informa; no toca nada)

                                   <una persona escribe el paraguas>

                                   lecciones consolidar --aplicar \
                                     --en <paraguas> --de a,b --motivo "..."
                                     -> backup, archiva a y b, reporta
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero enterarme de que dos lecciones cuentan lo mismo, sin
  tener que releer las nueve cada vez.
- P1: Como Alan, quiero que un modelo **nunca** reescriba mi memoria procedural:
  que sugiera, y que la prosa la escriba una persona.
- P2: Como usuario de otro proyecto, quiero que esto funcione con el backend que
  yo tenga, o que se apague limpio si no tengo ninguno.

## Criterios de aceptacion (Given/When/Then)

<!-- Comportamiento con tests; documentacion con greps. Ningun comando repetido.
     Los AC que necesitan modelo estan separados de los que no, a proposito: la
     mitad que muta se verifica sin backend y la del modelo sin mutar. -->

### Apagada por default, y de forma estructural

- AC-1: Given `rules.consolidar_backend` ausente o vacio, When corre
  `lecciones consolidar`, Then **no resuelve backend, no spawnea nada y no
  escribe nada**, e informa como encenderla. Ni siquiera mira el entorno.
  Comando: `cd rust && cargo test consolidar_should_be_off_without_the_rule`
- AC-2: Given la regla activa pero **ningun** backend disponible, Then hace
  **skip limpio** con exit 0 y un mensaje que dice que falto, sin dejar rastro.
  Comando: `cd rust && cargo test consolidar_should_skip_cleanly_without_a_backend`
- AC-3: Given una API key en el entorno pero ningun CLI, Then el mensaje de skip
  **lo dice explicitamente**: el arnes no habla HTTP y hay que declarar un CLI.
  Comando: `cd rust && cargo test consolidar_should_name_the_api_key_limitation`

### La cadena de backend, agnostica y verificada con dos

- AC-4: Given `HARNESS_CONSOLIDAR_CMD`, Then ese comando gana sobre cualquier
  CLI detectado. El override elige **cual** backend, nunca **enciende** la
  feature.
  Comando: `cd rust && cargo test consolidar_override_should_win_over_detection`
- AC-5: Given ningun override, Then se detecta el primer CLI disponible de una
  tabla corta (`claude -p`, `kimi -p`), en orden y sin pinnear ninguno.
  Comando: `cd rust && cargo test consolidar_should_detect_the_first_available_cli`
- AC-6: Given la salida real de **dos** backends distintos (`claude`, que
  devuelve JSON pelado, y `kimi`, que lo envuelve en `• ...`), Then el parser
  extrae el mismo resultado de las dos. Los fixtures son salidas reales, no
  inventadas.
  Comando: `cd rust && cargo test consolidar_should_parse_the_output_of_both_backends`

### El modelo no puede hacer dano

- AC-7: Given cualquier corrida, Then al modelo se le manda **solo** nombre,
  descripcion y triggers de cada leccion: **jamas el cuerpo**. Los
  procedimientos y los pitfalls —la parte cara— no salen de `docs/`.
  Comando: `cd rust && cargo test consolidar_should_never_send_the_lesson_body`
- AC-8: Given el prompt, Then viaja como **un item de argv**, nunca por `sh -c`:
  una descripcion con backticks o `$(...)` no puede ejecutar nada.
  Comando: `cd rust && cargo test consolidar_should_not_pass_the_prompt_through_a_shell`
- AC-9: Given una respuesta que nombra una leccion **inexistente**, Then ese
  candidato se descarta y se dice que se descarto. Una alucinacion no puede
  llegar al reporte en silencio.
  Comando: `cd rust && cargo test consolidar_should_drop_hallucinated_members`
- AC-10: Given un candidato que toca una leccion `pinneada: true`, Then se
  descarta: el pin ya significa "esta no se toca".
  Comando: `cd rust && cargo test consolidar_should_respect_the_pin`
- AC-11: Given una respuesta que no es JSON, o vacia, o con basura alrededor,
  Then el comando **no falla**: informa que el backend no devolvio nada usable y
  sale 0. Un modelo que balbucea no puede romper el flujo.
  Comando: `cd rust && cargo test consolidar_should_survive_a_garbage_answer`

### La deteccion informa; la fusion la pide una persona

- AC-12: Given `lecciones consolidar` **sin** `--aplicar`, Then solo informa:
  cero escrituras, cero backups, y el arbol queda byte a byte igual. Misma
  simetria que `lecciones curar`.
  Comando: `cd rust && cargo test consolidar_without_aplicar_should_not_touch_anything`
- AC-13: Given `--aplicar`, Then la fusion se toma de **argv**
  (`--en <paraguas> --de <a,b> --motivo "<por que>"`), **no** de lo que el modelo
  dijo: la mitad que muta se verifica sin backend y de forma determinista.
  Comando: `cd rust && cargo test consolidar_aplicar_should_take_the_merge_from_argv`
- AC-14: Given `--aplicar` sin `--motivo`, Then sale 2: una fusion sin motivo
  escrito es la que nadie va a poder revisar despues.
  Comando: `cd rust && cargo test consolidar_aplicar_should_demand_a_motivo`

### El paraguas tiene que contener lo que las miembros ensenaban

- AC-15: Given `--en <paraguas>`, Then el paraguas **puede ser una de las
  miembros**: es lo que la guia manda ("patchea el paraguas existente"), y es
  exactamente la forma del unico solapamiento real de este repo.
  Comando: `cd rust && cargo test consolidar_should_allow_an_existing_member_as_the_umbrella`
- AC-16: Given el paraguas todavia con los placeholders de la plantilla, Then
  `--aplicar` sale 2: archivar las miembros contra un esqueleto perderia el
  conocimiento de forma estructural.
  Comando: `cd rust && cargo test consolidar_should_refuse_a_skeleton_umbrella`
- AC-17: Given el paraguas, Then tiene que contener **todos los triggers** de
  cada miembro que se archiva. `buscar` puntua una leccion activa 100 y una
  archivada 30: si el paraguas no hereda los triggers, el conocimiento deja de
  encontrarse.
  Comando: `cd rust && cargo test consolidar_should_demand_the_union_of_triggers`
- AC-18: Given el paraguas, Then cita a cada miembro archivada con `[[nombre]]`,
  para que quede el puntero de recuperacion.
  Comando: `cd rust && cargo test consolidar_should_demand_a_pointer_to_each_member`

### Nunca borra, y se puede deshacer

- AC-19: Given `--aplicar`, Then cada miembro queda en
  `docs/lecciones/archivo/<nombre>.md` con su **cuerpo identico byte a byte**, y
  hay backup previo en `bkp/lecciones/<ts>/`.
  Comando: `cd rust && cargo test consolidar_should_archive_byte_for_byte_with_backup`
- AC-20: Given una fusion aplicada, Then `lecciones rollback` la deshace y el
  catalogo vuelve a su estado anterior.
  Comando: `cd rust && cargo test consolidar_should_be_undoable_with_rollback`
- AC-21: Given el reporte, Then lista **cada fusion con su motivo** y el backup
  donde quedo lo anterior.
  Comando: `cd rust && cargo test consolidar_report_should_list_each_merge_with_its_reason`

### Verificado de punta a punta, con backend de verdad

- AC-22: Given este repo y `claude` disponible, When corre
  `lecciones consolidar` con la regla activa, Then el modelo devuelve JSON
  valido y el comando lo procesa. **No se cierra con el camino sin ejecutar.**
  Comando: `bash tests/consolidar_check.sh backend-real`
- AC-23: Given un catalogo deliberadamente **sin** solapamientos, Then la
  propuesta vacia es un resultado de primera clase: informa "catalogo limpio",
  sale 0 y no crea backup. No es una rama muerta.
  Comando: `bash tests/consolidar_check.sh catalogo-limpio`
- AC-24: Given el corpus **real** de este repo, Then la corrida se documenta en
  `docs/impl-28.md`: que propuso el modelo, que se descarto y por que. La
  calibracion no se declara, se muestra.
  Comando: `grep -q "Corrida contra el corpus real" docs/impl-28.md`

### Integracion, docs y verificacion

- AC-25: Given el plan, Then declara `Peldano elegido:` con su razon.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-28-consolidacion-de-lecciones-con-llm.md`
- AC-26: Given `README.md` y `UPDATING.md` (+ espejo) y la guia de lecciones,
  Then documentan el comando, que el modelo no ve el cuerpo, y como encenderlo.
  Comando: `grep -q "lecciones consolidar" README.md UPDATING.md templates/UPDATING.md`
- AC-27: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `tests/setup_smoke.sh`, `tests/parity_check.sh` y `harness_check.sh` siguen
  verdes.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: nada automatico. El comando se corre a mano.
- interruptor: `rules.consolidar_backend` ausente = apagada, estructuralmente.
- candado: el backup previo y `archivo/`; ningun camino borra.

## Pseudo-codigo (el acuerdo)

```
CUANDO corre `lecciones consolidar`
  ¿rules.consolidar_backend?        -> si no, apagada: no se mira nada mas
  ¿override HARNESS_CONSOLIDAR_CMD? -> ese
  ¿algun CLI de la tabla?           -> el primero
  si no                             -> skip limpio, exit 0

  se manda: nombre + descripcion + triggers   (NUNCA el cuerpo)
  se recibe: {"candidatos":[{"miembros":[...],"motivo":"...","confianza":0.0}]}
  se descarta: lo que no sea JSON, miembros inexistentes, grupos con pinneadas
  se informa. NO se toca nada.

CUANDO corre con `--aplicar --en <paraguas> --de a,b --motivo "..."`
  ¿el paraguas tiene placeholders?        -> exit 2
  ¿tiene todos los triggers de a y b?     -> si no, exit 2
  ¿cita [[a]] y [[b]]?                    -> si no, exit 2
  backup -> archiva a y b -> reporta con el motivo
```

## No funcionales

- Sin dependencias nuevas (Articulo 6): el LLM se invoca como proceso, con
  `wait-timeout`, que ya es dependencia.
- Timeout corto y configurable: un backend colgado no puede colgar el comando.
- Cero red desde el binario: se habla con un CLI local, no con una API.

## Fuera de alcance

- **Que el modelo redacte el paraguas.** La prosa la escribe una persona. El
  binario nunca escribe el cuerpo de una leccion (limite heredado de la #21:
  *"el curador mueve y marca, no edita"*).
- **Borrar.** No existe camino que borre.
- **Consolidacion automatica** en un hook o en `close`.
- **El camino HTTP con API key**: ver OBS-1.

## Observaciones (decididas por Alan el 2026-08-18)

- OBS-1 **DECIDIDA: el camino HTTP con API key queda FUERA DE ALCANCE**, dicho
  explicitamente en el mensaje de skip (AC-3). Implementar tres formatos de
  request/respuesta/error a ciegas seria la unica parte de la feature sin
  verificacion de punta a punta, que es exactamente lo que la #30 acaba de
  ensenar a no hacer. La cadena queda: **override -> CLI -> skip limpio**, y el
  tramo faltante se nombra en vez de disimularse.
- OBS-2 **DECIDIDA: se fusionan de verdad las dos lecciones reales.** Se redacta
  el paraguas fusionando `docs-generados-por-el-instalador` y
  `documentos-del-usuario-vs-plantillas` sin perder ningun pitfall, se le MUESTRA
  completo a Alan, y solo con su SI se corre `--aplicar`. Si el resultado le
  parece peor que las dos separadas, no se aplica. -> AC-24.
- OBS-3 **DECIDIDA: la confianza se reporta sin filtrar.** Un umbral que no se
  puede calibrar es un numero inventado con aspecto de rigor: con 9 lecciones y
  un solo par real, cualquier valor entre 0.1 y 0.4 da identico resultado. Se
  imprime junto al candidato y decide quien lee. -> AC-21.
