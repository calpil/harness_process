# Spec - Feature #29: prd_y_sdd_siempre_al_dia

Estado: approved
Aprobado: 2026-08-18T04:04:11Z por USUARIO (confirmacion explicita) - Alan aprobo en el chat tras el ritual. OBS-1: el gate exige la propuesta APLICADA con su SI. OBS-2: require_docs_al_dia encendida en este repo. OBS-3: los cuatro documentos del alcance D-2 desde el dia uno. El diseno salio de un workflow de 18 agentes y evita tres bloqueos verificados contra el codigo (deadlock de frescura, la auto-aplicacion via verify, y el slicing por ## que se traga los ###).
Plan: docs/plan-feature-29-prd-y-sdd-siempre-al-dia.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: el arnes verifica que el codigo cumpla el **spec** (feature #23) y nada
mas. El cuerpo del PRD, el SDD y `docs/architecture.md` pueden decir cualquier
cosa: nadie los contrasta y, peor, **a nadie se le recuerda que existen**.

Tres hechos medidos en este repo, no supuestos:

1. **`docs/prd/SDD-master.md` es una plantilla vacia que se publica como diseno
   real.** 92 lineas, **27 con `<placeholder>`**; la linea 1 dice
   `# SDD Master - <nombre del proyecto>` y la 4
   `Ultima actualizacion: <YYYY-MM-DD>`. Y `commands/atlassian.rs:942` la publica
   a Confluence como el diseno tecnico del proyecto.
2. **`docs/architecture.md` esta driftando ahora mismo.** Documenta
   `lecciones.rs`, `perfil.rs`, `buscar.rs`, `curador.rs`, `journey.rs` y
   `verificacion.rs`, pero **no menciona `doctor.rs` (#25) ni `rutas.rs` (#26)**
   — dos features cerradas hoy, con `verify` verde y revision escrita. La
   disciplina no alcanzo ni en la sesion en que se escribio la regla.
3. **Nadie recuerda que existen.** `grep -rn "architecture.md" roles/*.md
   CHECKPOINTS.md docs/conventions.md` da **cero**. Idem `prd` y `sdd`.

"Es del USUARIO" nunca quiso decir "que quede mintiendo".

DESPUES: al cerrar, el arnes **calcula el alcance** (el PRD de origen, sus PRDs
padres, el SDD y `architecture.md`), **siembra una pregunta cerrada por cada
documento** en `docs/prd-diff-<id>.md`, el agente la contesta, el usuario la lee
y **solo con su SI** el binario la escribe. El documento sigue siendo del
usuario; lo que cambia es que ya no puede quedar desactualizado en silencio.

## Hoy -> Como va a funcionar

```
HOY                                  DESPUES
close --status done                  prd propose --feature <id>   (el BINARIO siembra
  -> marca el hito                        |                        una pregunta por documento)
  -> deja bitacora                   <el agente contesta cada una>
  -> el cuerpo del PRD queda igual   prd apply --feature <id>     (el BINARIO muestra que
  -> el SDD sigue vacio                   |                        escribiria; exit 2)
                                     <el usuario lee y dice que si>
                                     prd apply --feature <id> --yes (escribe y sella)
                                     close --status done          (el gate lo exige)
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero enterarme al cerrar de que el PRD o el SDD quedaron
  desactualizados, con la propuesta ya escrita, no con un recordatorio.
- P1: Como Alan, quiero leer lo que se va a escribir en **mis** documentos y
  decir si o no, igual que con el spec.
- P2: Como agente, quiero que el arnes me diga **cuales** documentos tengo que
  mirar, en vez de tener que acordarme de que existen.

## Criterios de aceptacion (Given/When/Then)

<!-- Comportamiento con tests; documentacion con greps. Ningun comando repetido.
     ATENCION (AC-19): ningun `Comando:` de este spec puede invocar
     `prd apply --yes`, porque `verify` los ejecuta con `sh -c`. -->

### El alcance lo calcula el binario, no el agente

- AC-1: Given una feature con su `prd`, When corre `prd propose --feature <id>`,
  Then el alcance sale del **arbol real**: el PRD de origen, **todos** sus PRDs
  padres hasta el maestro, `docs/prd/SDD-master.md` y `docs/architecture.md`.
  Comando: `cd rust && cargo test documentos_alcance_should_include_the_prd_chain_sdd_and_architecture`
- AC-2: Given una feature con PRD anidado (por ejemplo `aprendizaje`), Then el
  alcance incluye **el hijo y el maestro**, en ese orden, sin repetir.
  Comando: `cd rust && cargo test documentos_alcance_should_walk_nested_prds_without_repeating`
- AC-3: Given un documento del alcance que **no existe** en el repo, Then se
  omite sin fallar: una instalacion sin SDD sigue funcionando.
  Comando: `cd rust && cargo test documentos_alcance_should_skip_missing_documents`

### La propuesta: una pregunta cerrada por documento

- AC-4: Given `prd propose --feature <id>`, Then siembra
  `docs/prd-diff-<id>.md` con **un bloque por documento del alcance**, cada uno
  con su `Veredicto: PENDIENTE`, y sale con exit 2 mientras haya pendientes.
  Comando: `cd rust && cargo test prd_propose_should_seed_one_block_per_document`
- AC-5: Given que el archivo ya existe, Then `propose` **no lo pisa**: conserva
  los veredictos ya escritos y solo agrega los bloques que falten.
  Comando: `cd rust && cargo test prd_propose_should_not_clobber_existing_verdicts`
- AC-6: Given cada bloque, Then el **binario** precomputa y escribe
  `Presente en:` / `Ausente en:` buscando en ese documento las senales de la
  feature (su nombre y sus modulos nuevos), para que el agente no parta de cero.
  Comando: `cd rust && cargo test prd_propose_should_precompute_presence_signals`
- AC-7: Given la lista de bloques que el binario sembro, Then el agente **no
  puede agregar, quitar ni renombrar** bloques: si la lista no coincide con la
  recomputada, `prd apply` sale 2 nombrando la diferencia.
  Comando: `cd rust && cargo test prd_apply_should_reject_a_tampered_block_list`

### Los tres veredictos, y el que se puede refutar

- AC-8: Given un bloque con `Veredicto: cambio`, Then trae el texto `Antes:` y
  `Despues:` y `prd apply` reemplaza **ese texto literal** en el documento. El
  anclaje es por CONTENIDO, no por seccion: anclar por `## ` se tragaria las
  subsecciones `###` (hay 3 en `docs/architecture.md`).
  Comando: `cd rust && cargo test prd_apply_should_replace_the_literal_anchor_not_the_section`
- AC-9: Given un bloque con `Veredicto: ya-esta <archivo>:<L1>-<L2>`, Then el
  binario **verifica la cita**: si ese rango de lineas no contiene el literal que
  el bloque dice, sale 2. La mentira mas probable del agente ("eso ya esta
  documentado") pasa a ser refutable por maquina, sin heuristica.
  Comando: `cd rust && cargo test prd_apply_should_refuse_a_citation_that_does_not_hold`
- AC-10: Given un bloque con `Veredicto: no-aplica <razon>`, Then se acepta si la
  razon no esta vacia: una feature que de verdad no toca el producto tiene una
  salida honesta.
  Comando: `cd rust && cargo test prd_apply_should_accept_no_aplica_with_a_reason`
- AC-11: Given un veredicto desconocido o `PENDIENTE`, Then `prd apply` sale 2
  nombrando el bloque sin resolver.
  Comando: `cd rust && cargo test prd_apply_should_name_the_unresolved_block`

### El ritual: el usuario aprueba, como con el spec

- AC-12: Given `prd apply --feature <id>` **sin** `--yes`, Then **no escribe
  nada** e imprime lo que escribiria, documento por documento, mas el bloque
  `[GATE]` de tres pasos, y sale 2. Mismo molde que `approve-spec`.
  Comando: `cd rust && cargo test prd_apply_without_yes_should_show_and_refuse_to_write`
- AC-13: Given `prd apply --feature <id> --yes`, Then escribe los cambios, sella
  la propuesta con `Aplicado: <stamp> por USUARIO (confirmacion explicita)` y
  deja una linea en `progress/history.md`.
  Comando: `cd rust && cargo test prd_apply_with_yes_should_write_seal_and_log`
- AC-14: Given una propuesta **ya aplicada**, When se corre `prd apply --yes` de
  nuevo, Then es idempotente y **no escribe**. La idempotencia sale del
  contenido (el `Antes:` ya no esta y el `Despues:` si), **no** de una firma tipo
  `last_spec_sig`: el spec es 1:1 con su feature, pero un PRD lo comparten N
  features y una firma por feature mentiria desde la segunda.
  Comando: `cd rust && cargo test prd_apply_should_be_idempotent_by_content`

### Compone con las rutas protegidas (#26)

- AC-15: Given que `docs/prd/**` esta protegida, Then la propuesta del agente se
  escribe en `docs/prd-diff-<id>.md`, **fuera** de esa ruta, y ninguna
  herramienta del agente toca `docs/prd/**`.
  Comando: `cd rust && cargo test prd_diff_should_live_outside_the_protected_path`
- AC-16: Given `prd apply --yes`, Then escribe en `docs/prd/**` como el binario
  —no como el agente— y **registra sus escrituras** igual que `close`, para no
  dispararse a si mismo la red de seguridad.
  Comando: `cd rust && cargo test prd_apply_should_register_its_own_writes`

### El gate del cierre

- AC-17: Given `rules.require_docs_al_dia` y una propuesta con bloques sin
  resolver o sin aplicar, When se cierra como done, Then exit 2 nombrando que
  falta. Sin la regla, cerrar se comporta exactamente como hoy.
  Comando: `cd rust && cargo test close_should_demand_the_docs_proposal_when_the_rule_is_on`
- AC-18: Given el gate, Then **no usa frescura contra `docs/verify-<id>.md`**:
  `verify` reescribe su reporte en cada corrida y `prd apply` es idempotente, asi
  que exigir `mtime(propuesta) >= mtime(reporte)` dejaria la propuesta vieja para
  siempre, sin ningun comando que pueda refrescarla.
  Comando: `cd rust && cargo test docs_gate_should_not_depend_on_verify_report_freshness`

### La trampa que este repo se puso solo

- AC-19: Given que `verify` ejecuta los `Comando:` de los AC con `sh -c`
  (`verificacion.rs:163`), Then **ningun** `Comando:` de ningun spec puede
  invocar `prd apply --yes`: correr `verify` aplicaria la propuesta sin el SI del
  usuario, saltandose el ritual entero. Hay un test que lo prohibe.
  Comando: `cd rust && cargo test no_spec_command_should_invoke_prd_apply_yes`

### Distribucion, docs y verificacion

- AC-20: Given `CHECKPOINTS.md` y los tres roles (+ espejo en `templates/`),
  Then declaran el deber: hoy `grep` da cero. El lider dice que el PRD/SDD son
  parte del entregable, el implementer corre `prd propose`, y el reviewer exige
  la propuesta resuelta.
  Comando: `grep -q "prd propose" CHECKPOINTS.md roles/implementer.md templates/CHECKPOINTS.md`
- AC-21: Given `README.md` y `UPDATING.md` (+ espejo), Then documentan el ritual,
  los tres veredictos y por que el gate no usa frescura.
  Comando: `grep -q "prd apply" README.md UPDATING.md templates/UPDATING.md`
- AC-22: Given el plan, Then declara `Peldano elegido:` con su razon.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-29-prd-y-sdd-siempre-al-dia.md`
- AC-23: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `tests/setup_smoke.sh`, `tests/parity_check.sh` y `harness_check.sh` siguen
  verdes.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: el cierre de una feature con `rules.require_docs_al_dia` activa.
- interruptor: la regla ausente o `false` deja todo como hoy.
- candado: la idempotencia por contenido — un bloque ya aplicado no se reaplica.

## Pseudo-codigo (el acuerdo)

```
CUANDO el agente corre `prd propose --feature <id>`
  el BINARIO calcula el alcance desde el arbol real
  y siembra un bloque por documento, con Presente/Ausente precomputado

CUANDO el agente contesta cada bloque
  cambio    -> Antes: / Despues:   (texto literal)
  ya-esta   -> archivo:L1-L2       (el binario verifica que la cita diga eso)
  no-aplica -> razon               (no puede estar vacia)

CUANDO el agente corre `prd apply --feature <id>`
  ¿algun bloque sin resolver?  -> exit 2, nombrando cual
  ¿la lista fue manipulada?    -> exit 2, nombrando la diferencia
  ¿alguna cita no se sostiene? -> exit 2, nombrando la cita
  si no -> imprime lo que escribiria y exit 2 con [GATE]

CUANDO el usuario dice que si y se corre con --yes
  escribe, sella, registra las rutas y deja bitacora

CUANDO se cierra con la regla activa
  ¿propuesta resuelta y aplicada? -> pasa
  si no -> exit 2
```

## No funcionales

- Sin dependencias nuevas (Articulo 6): el anclaje es reemplazo de texto
  literal, no un motor de patch.
- El gate solo lee; escribir es siempre `prd apply --yes`.
- Cero comandos nuevos de nivel superior: `propose` y `apply` entran como
  variantes del grupo `prd` que ya existe.

## Fuera de alcance

- Que el arnes **escriba solo** el PRD (descartado en D-1: rompe la regla
  replicada en cuatro lugares de que el documento es del usuario).
- Un gate de frescura a secas (descartado en D-1: no ayuda a escribir; y el
  AC-18 muestra que ademas se deadlockea).
- Verificar que el contenido del PRD sea *correcto*: solo se verifica que la
  pregunta se haya contestado y que las citas se sostengan.
- `harness_check.sh` no suma bloque: el drift solo importa al cerrar, y nagear
  en cada turno seria ruido.

## Observaciones (decididas por Alan el 2026-08-18)

- OBS-1 **DECIDIDA: el gate exige la propuesta APLICADA, con el SI del usuario.**
  Es la D-1 al pie de la letra y el unico modo que da control real sobre los
  documentos del usuario. Costo aceptado: un paso de aprobacion en cada cierre —
  mitigado porque cuando todos los veredictos son `ya-esta`/`no-aplica` lo que se
  lee es una tabla de cuatro renglones. -> AC-17.
- OBS-2 **DECIDIDA: `require_docs_al_dia` encendida en este repo.** Como las
  otras tres reglas. Es donde el problema se midio: el SDD vacio publicado a
  Confluence y el `architecture.md` driftado son de aca.
- OBS-3 **DECIDIDA: los cuatro documentos desde el dia uno**, como dice D-2. Un
  documento que no aplica se contesta `no-aplica <razon>`: el costo real es leer
  cuatro renglones, no escribir cuatro documentos. -> AC-1, AC-2.
