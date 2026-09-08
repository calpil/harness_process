# Spec - Feature #84: leccion partir: parte una leccion sobre el tope a referencias/ con informe y --aplicar, reconoce las secciones por feature en sus formas reales, y el check resume en una linea las lecciones sobre el tope

Estado: approved
Aprobado: 2026-09-08T23:13:22Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado' a los diez AC; OBS-1 titulo con #N o fecha salvo canonicas; OBS-2 lo movido queda y exit 2; OBS-3 --seccion se niega sobre canonicas
Plan: docs/plan-feature-84-leccion-partir-parte-una-leccion-sobre-el-tope-a.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)
ANTES: Alan cierra una feature en realestate y el Stop le muestra cuatro
parrafos `[i]`: cuatro lecciones sobre el tope de 250, una de 2361 lineas. El
arnes le dice "partila" y le describe el procedimiento a mano de la guia; el
proximo `close --leccion el-verde-no-es-evidencia` se va a negar. Pero el
contrato de particion no le nombra ninguna seccion, porque busca titulos con
`(feature #N)` y esas lecciones escriben `(#115, 2026-08-30)` o `feature #100
(2026-08-28)`. Partir 2361 lineas a mano es una tarde, y nadie la tiene.
DESPUES: Alan corre `leccion partir el-verde-no-es-evidencia` y ve el informe:
ocho secciones cuentan una sola feature (y cuantas lineas son), la leccion
quedaria en 1990, faltan 1740 para el tope, y las secciones no canonicas mas
grandes que podria mover con `--seccion`. Con `--aplicar` las ocho se mueven
tal cual a `referencias/`, cada una con su cabecera de referencia, la leccion
queda con ocho punteros de una linea y un respaldo en `bkp/lecciones/`. Lo que
falta es editorial, y el comando se lo dice con el numero. El Stop, mientras
tanto, le muestra UNA linea con las cuatro y el comando.

## Hoy -> Como va a funcionar
```
HOY                                                  DESPUES
leccion sobre el tope                                leccion sobre el tope
  close/usar -> [GATE] contrato de particion           close/usar -> [GATE] contrato: "sh harness_cli leccion partir <clase>"
                (solo titulos con "(feature #N)")                     + las secciones por feature (todas sus formas)
  harness_check -> un [i] largo POR leccion            harness_check -> UN [i]: "N leccion(es) sobre el tope: a (398), b (275)... leccion partir"
  partir: a mano (guia, paso 3)                        leccion partir <clase>            -> informe, no escribe
                                                       leccion partir <clase> --aplicar  -> respaldo, mueve, punteros, cuenta
                                                       leccion partir <clase> --aplicar --seccion "<titulo>"  -> mueve tambien esa
```

## Recorridos de usuario (priorizados)
- P1: Como agente que cierra una feature y el gate le pide partir la leccion,
  quiero un comando que mueva lo mecanico (las secciones que cuentan una sola
  feature) a `referencias/` y me diga que falta, para no hacer a mano lo que
  el arnes puede hacer igual en todos los repos.
- P1: Como usuario mirando el Stop, quiero una linea por check con las
  lecciones sobre el tope y el comando, no un parrafo por leccion.
- P2: Como agente al que el informe le dice que faltan lineas, quiero elegir
  con `--seccion` que seccion mas se va, para partir sin reescribir.
- P2: Como usuario que se arrepiente, quiero deshacerlo con
  `lecciones rollback`, como cualquier cambio del curador.

## Criterios de aceptacion (Given/When/Then)
- AC-1: Given una leccion de clase sobre `rules.leccion_max_lineas`, When se
  corre `leccion partir <clase>` sin `--aplicar`, Then no escribe nada e
  informa: lineas y tope, cada seccion candidata con sus lineas, cuantas
  lineas quedarian, y si no baja del tope cuanto falta y las secciones no
  canonicas mas grandes (para `--seccion`). Con una leccion bajo el tope lo
  dice y no propone nada.
  Comando: `cd rust && cargo test --locked partir_informa`
- AC-2: Given titulos `## ` con las formas reales (`(feature #N)`,
  `(#N, fecha)`, `feature #N (fecha)`, `(fecha, front #N)`, `Patch #N:`, y una
  fecha sola), When se buscan las secciones por feature, Then todas se
  reconocen, las canonicas (Cuando aplica, Procedimiento, Pitfalls,
  Verificacion, Referencias) nunca, y `contrato_de_particion` las lista con
  el mismo criterio. Los titulos de prueba son los de las cuatro lecciones de
  realestate del 2026-09-08 (datos reales).
  Comando: `cd rust && cargo test --locked secciones_por_feature`
- AC-3: Given la misma leccion, When `--aplicar`, Then primero respalda en
  `bkp/lecciones/<ts>/` (el mismo respaldo de `curar`, asi `lecciones
  rollback` lo deshace), mueve cada seccion candidata (del `## ` al siguiente
  `## `, con sus `###`) a `docs/lecciones/<clase>/referencias/<slug>.md` con la
  cabecera de referencia (titulo, clase, fecha, "movida por `leccion partir`",
  texto original sin reescribir), deja un puntero `- [titulo](<clase>/referencias/<slug>.md)`
  en el indice `## Referencias` de la leccion (lo crea al final si no existe),
  pone `ultima_actualizacion` en hoy, guarda atomico e imprime que movio y
  las lineas resultantes. Cuando aplica, procedimiento, pitfalls y
  verificacion quedan byte-identicos.
  Comando: `cd rust && cargo test --locked partir_aplica`
- AC-4: Given `--seccion <titulo>` (exacto o subcadena unica), When se aplica,
  Then esa seccion se mueve ademas de las candidatas; una subcadena ambigua
  lista las coincidencias y no escribe; una seccion canonica se niega con el
  motivo (es la clase, no un caso).
  Comando: `cd rust && cargo test --locked partir_seccion`
- AC-5: Given una leccion que sigue sobre el tope despues de mover todo lo
  candidato, When `--aplicar`, Then lo movido queda movido, el comando sale
  con exit 2 y dice cuantas lineas faltan y que secciones podrian ir con
  `--seccion`; sin candidatas y sin `--seccion` no escribe nada y sale 2 con
  esa misma lista.
  Comando: `cd rust && cargo test --locked partir_sigue_sobre_el_tope`
- AC-6: Given una leccion ya partida, When se vuelve a correr, Then no duplica
  punteros ni archivos (no hay candidatas: informa y sale 0 si esta bajo el
  tope); un slug que colisiona recibe sufijo `-2`.
  Comando: `cd rust && cargo test --locked partir_dos_veces`
- AC-7: Given N lecciones sobre el tope, When corre `harness_check.sh`, Then
  emite UNA linea `[i]` con las N (nombre y lineas) y el comando
  `leccion partir`; `referencias/` no cuenta y `0` apaga, como antes.
  Comando: `bash tests/leccion_tope_check.sh`
- AC-8: Given una leccion sobre el tope, When `close --leccion` o `leccion
  usar` se niegan, Then el contrato nombra `sh harness_cli leccion partir
  <clase>` como primer paso y lista las secciones por feature con el criterio
  de AC-2; `lecciones status` sugiere el comando cuando hay lecciones sobre
  el tope.
  Comando: `cd rust && cargo test --locked contrato_de_particion`
- AC-9 (MANUAL): Given una copia de las cuatro lecciones de realestate, When
  se corre el informe y `--aplicar` sobre cada una, Then se registra en el
  review que movio cada una, en cuantas lineas quedo y cuales bajan del tope
  solo con lo mecanico (medido antes de escribir el spec: ninguna; lo que
  falta es editorial y el comando lo dice con el numero).
- AC-10: Given README, `UPDATING.md` (dos copias), `docs/architecture.md`, la
  guia `COMO-ESCRIBIR-UNA-LECCION.md` (dos copias) y el texto AGENTS de los
  dos instaladores, When se cierra, Then nombran `leccion partir` y las
  copias son identicas.
  Comando: `bash tests/parity_check.sh && cmp UPDATING.md templates/UPDATING.md && cmp docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md templates/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md && grep -q "leccion partir" README.md docs/architecture.md setup_harness.sh setup_harness.ps1`

## Los datos que se tocan
- disparador: `leccion partir <clase>` a mano, o por el remedio que imprimen
  el gate de `close`/`usar`, `harness_check.sh` y `lecciones status`.
- interruptor: `rules.leccion_max_lineas: 0` apaga el tope; con el tope
  apagado el comando igual parte si se lo piden (informa que no hay tope).
- candado: idempotente (AC-6); respaldo previo en `bkp/lecciones/<ts>/`.
- entradas: `docs/lecciones/<clase>.md`; salidas: `docs/lecciones/<clase>/referencias/<slug>.md`
  y la leccion reescrita (solo se quitan secciones y se agregan punteros).

## Pseudo-codigo (el acuerdo)
```
CUANDO leccion partir <clase> [--aplicar] [--seccion T]...

  leccion = cargar(clase)            (ilegible -> exit 2 con el motivo, como usar)
  secciones = partir el cuerpo en bloques "## ..." (el preambulo antes del primer ## no se toca)
  candidatas = las que cuentan UNA feature/sesion (AC-2) + las --seccion resueltas (AC-4)
  saldo = lineas - sum(candidatas)

  INFORME (siempre): tope, candidatas con lineas, saldo, falta = max(0, saldo - tope),
                     y si falta > 0: las no canonicas mas grandes que no estan en candidatas
  ¿sin --aplicar?    -> exit 0 (no escribe)
  ¿sin candidatas?   -> exit 2 (no escribe)
  ENTONCES respaldar; por cada candidata: escribir referencias/<slug>.md, quitar el bloque,
           agregar el puntero al indice (crearlo si falta); ultima_actualizacion = hoy; guardar
  ¿saldo > tope?     -> exit 2 con falta y la lista   ; si no -> exit 0
```
Promesas: mueve, no reescribe · no toca las secciones canonicas · respaldo
antes de escribir · el informe nunca escribe · el gate y el check dicen el
mismo comando.

## No funcionales
- SLOs: un archivo, sin red, sin hub.
- Seguridad: solo escribe bajo `docs/lecciones/<clase>/` y la propia leccion.
- Observabilidad: el informe es la salida; `lecciones rollback` deshace.

## Fuera de alcance
- Reescribir o resumir texto: lo editorial (el Procedimiento de 464 lineas y
  los Pitfalls de 540 del gigante de realestate) lo decide quien escribe.
- Partir las lecciones de realestate desde aca: se entrega el comando.
- Cambiar la forma de los titulos existentes.

## Observaciones (decisiones pendientes)
- OBS-1 (que es una seccion por feature): propuesta: todo `## ` cuyo titulo
  trae `#<numero>` o una fecha `YYYY-MM-DD`, salvo las canonicas. Alternativas:
  solo `(feature #N)` como hoy (no ve ninguna de las de realestate); pedir que
  el usuario nombre todas con `--seccion`. DECIDIDO (Alan, 2026-09-08): titulo
  con `#N` o con fecha, salvo las canonicas.
- OBS-2 (sigue sobre el tope tras aplicar): propuesta: lo movido queda y el
  comando sale 2 con lo que falta. Alternativas: todo o nada (no escribe si
  no llega al tope); salir 0 y avisar. DECIDIDO (Alan, 2026-09-08): lo movido
  queda, exit 2 con lo que falta.
- OBS-3 (`--seccion` sobre una canonica): propuesta: se niega (es la clase).
  Alternativa: permitirlo bajo responsabilidad del que lo pide. DECIDIDO (Alan,
  2026-09-08): se niega.
