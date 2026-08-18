# Spec - Feature #25: harness_doctor

Estado: approved
Aprobado: 2026-08-17T19:03:27Z por USUARIO (confirmacion explicita) - Alan aprobo en el chat tras el ritual (spec mostrado + abierto en editor). Decisiones OBS-1..OBS-4 tomadas por el en la misma vuelta: peldano 3 + arreglo del lanzador en peldano 1, sin solapar con harness_check, no_aplica en checkout fuente, falla solo lo que impide trabajar.
Plan: docs/plan-feature-25-harness-doctor.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: cuando la **instalacion** esta rota, el arnes no lo dice. Lo dice el
sintoma, tres pasos despues y con otro nombre. Esto ya paso, varias veces y en
este mismo repo:

- Alguien hace `git pull` y se queda con los scripts nuevos y el **binario
  viejo**. El sintoma es `error: unrecognized subcommand 'perfil'`. Hubo que
  parchear `harness_check.sh` a mano para que no lo reportara como un problema
  del perfil (feature #19) y despues otra vez por lo mismo.
- El marker `.harness_layout` se pierde y la raiz se resuelve al lugar
  equivocado: fue una feature entera (#10) descubrirlo.
- El instalador corrido dentro del checkout FUENTE del arnes se pisaba a si
  mismo: otra feature entera (#7).
- El hub no responde. Toda esta sesion trabajo con el hub caido, y esta bien —
  pero nadie que llegue nuevo sabe si eso es normal o esta roto.

`harness_check.sh` no cubre nada de esto: mira el **proceso** (spec aprobado,
plan fresco, PRDs, lecciones, perfil, convenciones). La **instalacion** —binario,
hooks, superficies, marker, hub, herramientas, graphify— no la mira nadie.

DESPUES: `sh harness_cli doctor` revisa la instalacion y, por cada problema,
imprime **el comando exacto que lo arregla**. No arregla nada por su cuenta y no
repite ni un chequeo de `harness_check.sh`: son dos preguntas distintas
("¿el proceso va bien?" contra "¿esto esta bien instalado?").

Y sabe donde esta parado: en el checkout FUENTE del arnes, la ausencia de
`CLAUDE.md` y de hooks **no es un problema** — es lo correcto. Un doctor que
grita ocho falsos positivos en el repo del propio arnes se ignora en dos dias.

## Hoy -> Como va a funcionar

```
HOY                                   DESPUES
instalacion rota -> sintoma raro      instalacion rota -> sh harness_cli doctor
  "unrecognized subcommand 'perfil'"       |__ [!!] binario mas viejo que los scripts
  (y a adivinar)                           |__     Remedio: bash setup_harness.sh
```

## Recorridos de usuario (priorizados)

- P1: Como usuario que acaba de hacer `git pull`, quiero saber en un comando si
  me falta re-correr el instalador, en vez de descubrirlo por un error cripitico.
- P1: Como usuario con un problema, quiero que cada falla venga con **el comando
  que la arregla**, no con una descripcion.
- P2: Como script de CI, quiero un exit code confiable y `--json` para decidir.

## Criterios de aceptacion (Given/When/Then)

<!-- Los AC de comportamiento se verifican con tests; los de documentacion con
     greps (leccion `criterios-de-cierre-que-se-pueden-fallar`). Ningun comando
     se repite entre dos AC. -->

### El diagnostico

- AC-1: Given una instalacion sana, When corre `sh harness_cli doctor`, Then
  reporta cada area revisada con su estado y sale **0**.
  Comando: `cd rust && cargo test doctor_should_report_every_area_on_a_healthy_install`
- AC-2: Given cualquier problema detectado, Then la linea trae **el comando
  exacto de remedio**, no una descripcion de que hacer.
  Comando: `cd rust && cargo test doctor_should_print_an_exact_remedy_for_every_problem`
- AC-3: Given el diagnostico, Then distingue **falla** (`[!!]`, impide trabajar)
  de **aviso** (`[i]`, funciona pero conviene saberlo), y solo las fallas cambian
  el exit code.
  Comando: `cd rust && cargo test doctor_should_separate_failures_from_warnings`
- AC-4: Given `--json`, Then expone por area: `area`, `estado`
  (`ok`/`falla`/`aviso`/`no_aplica`), `detalle` y `remedio`.
  Comando: `cd rust && cargo test doctor_json_should_expose_area_state_and_remedy`

### Las siete areas

- AC-5: **Binario.** Given el binario ausente, no ejecutable, o **mas viejo que
  los scripts que lo invocan**, Then es una falla con remedio
  `bash setup_harness.sh`. El caso "mas viejo" es el que ya rompio dos veces:
  `git pull` sin re-correr el instalador.
  Comando: `cd rust && cargo test doctor_should_detect_a_binary_older_than_the_scripts`
- AC-6: **Hooks.** Given un backend instalado cuyo hook apunta a un
  `bin/harness-hook` que no existe o no es ejecutable, Then es una falla que
  nombra el backend y el archivo.
  Comando: `cd rust && cargo test doctor_should_detect_a_hook_pointing_nowhere`
- AC-7: **Superficies.** Given una instalacion a la que le falta una superficie
  que su backend necesita (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`),
  Then lo reporta con el remedio; y las que ese backend **no** usa no se
  reportan.
  Comando: `cd rust && cargo test doctor_should_only_demand_surfaces_the_backend_uses`
- AC-8: **Marker y raiz.** Given `.harness_layout` ausente o incoherente con la
  raiz que el binario resuelve, Then lo reporta diciendo **que raiz resolvio y
  por que**, que es lo que costo la feature #10.
  Comando: `cd rust && cargo test doctor_should_explain_which_root_it_resolved_and_why`
- AC-9: **Hub.** Given el hub inalcanzable, Then es **aviso** y no falla: todo el
  aprendizaje del arnes funciona con el hub caido, y tratarlo como falla haria
  que el exit code mienta.
  Comando: `cd rust && cargo test doctor_should_treat_an_unreachable_hub_as_a_warning`
- AC-10: **Herramientas.** Given que falta una herramienta que el arnes invoca
  (`git`, y `cargo` solo si hay `rust/`), Then es falla; las opcionales
  (`graphify`, `curl`, `kimi`, `uv`, `pipx`) son aviso.
  Comando: `cd rust && cargo test doctor_should_split_required_and_optional_tools`
- AC-11: **Graphify.** Given graphify no instalado, Then es aviso con el comando
  de instalacion, porque el arnes funciona sin el.
  Comando: `cd rust && cargo test doctor_should_report_graphify_as_optional`

### Donde esta parado

- AC-12: Given el **checkout fuente** del arnes (marker `subdir` sin huella de
  instalacion en el padre), When corre `doctor`, Then las superficies y los hooks
  se reportan `no_aplica` y **no** cuentan como falla: en el repo del arnes su
  ausencia es lo correcto.
  Comando: `cd rust && cargo test doctor_should_not_demand_surfaces_in_a_source_checkout`
- AC-13: Given este repo (que ES un checkout fuente), When corre
  `sh harness_cli doctor`, Then sale **0**. Es la prueba contra datos reales: un
  doctor que falla en el repo del propio arnes no lo usa nadie.
  Comando: `sh harness_cli doctor`

### Sin solaparse, sin arreglar

- AC-14: Given `doctor`, Then **no repite** ningun chequeo de `harness_check.sh`
  (spec, plan, PRDs, lecciones, perfil, convenciones, espejo de roles) y su
  salida remite a el para el proceso.
  Comando: `cd rust && cargo test doctor_should_not_duplicate_the_process_checks`
- AC-15: Given `doctor`, Then **no modifica nada**: no tiene `--fix` ni escribe
  archivos. Solo lee e imprime. Misma decision que el curador (#21) y el mapa
  (#22).
  Comando: `cd rust && cargo test doctor_should_not_write_anything`

### El limite que el propio diseno tiene

- AC-16: Given que `doctor` vive en el binario, Then **no puede diagnosticar un
  binario ausente**. Ese caso lo cubre el lanzador `harness_cli`, que ademas
  ahora reconoce el binario **viejo** (`unrecognized subcommand`) e imprime el
  mismo remedio en vez del error de clap.
  Comando: `bash tests/doctor_launcher_check.sh`

### Integracion, docs y verificacion

- AC-17: Given `README.md` y `UPDATING.md` (+ espejo), Then documentan `doctor`,
  las siete areas, la diferencia con `harness_check.sh` y el limite del AC-16.
  Comando: `grep -q "harness_cli doctor" README.md UPDATING.md templates/UPDATING.md`
- AC-18: Given los tres roles y las superficies del instalador, Then mencionan
  `doctor` como el primer comando ante un problema de instalacion.
  Comando: `grep -q "doctor" roles/implementer.md setup_harness.sh setup_harness.ps1`
- AC-19: Given el plan, Then declara `Peldano elegido:` con la razon por la que
  extender `harness_check.sh` no alcanzaba, como exige `docs/conventions.md`.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-25-harness-doctor.md`
- AC-20: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test`, `cargo clippy --all-targets -- -D warnings`, `tests/setup_smoke.sh`
  y `harness_check.sh` siguen verdes.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: no hay evento. `doctor` se corre a mano, tipicamente despues de
  instalar o de un `git pull`.
- interruptor: no aplica; no hay nada que apagar porque no muta nada.
- candado: no aplica (solo lectura).

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien corre `doctor`

  ¿estoy en el checkout FUENTE del arnes?  -> superficies y hooks: no_aplica

  por cada area (binario, hooks, superficies, marker, hub, herramientas, graphify):
      miro
      ¿esta mal y me impide trabajar?  -> [!!] falla + comando de remedio
      ¿esta mal pero se puede seguir?  -> [i]  aviso  + comando de remedio
      ¿no aplica aca?                  -> se dice, y no cuenta

  ¿hubo alguna falla? -> exit 2
  si no                -> exit 0   (los avisos NO cambian el exit code)

  y nunca escribo nada
```

## No funcionales

- Sin dependencias nuevas (Articulo 6): todo se resuelve con lo que ya hay.
- El chequeo del hub tiene timeout corto: un doctor que tarda 30 segundos porque
  el hub no responde es un doctor que nadie corre.
- Cero escrituras: verificable comparando mtimes antes y despues.

## Fuera de alcance

- **Arreglar** (`--fix`). Ver AC-15: imprime el comando, lo corre el usuario.
- Diagnosticar el binario **ausente** desde el propio binario: es imposible, y el
  AC-16 lo resuelve donde corresponde (el lanzador).
- Re-chequear el proceso: eso es `harness_check.sh` (AC-14).
- Diagnosticar el hub por dentro (esquema, migraciones). Solo alcanzable o no.

## Observaciones (decididas por Alan el 2026-08-17)

- OBS-1 **DECIDIDA: comando nuevo (peldano 3) + arreglo del lanzador (peldano 1).**
  Hibrido, con cada mitad en su peldano mas alto posible. `harness_cli doctor`
  se justifica porque necesita `--json`, exit codes propios y la logica de
  resolucion de rutas del binario, que es justo lo que hay que diagnosticar;
  reimplementarla en shell seria reabrir el bug de la #10. Y el caso que el
  binario no puede cubrir por definicion —binario ausente o viejo— se arregla en
  `harness_cli`, que ya existe: peldano 1. El plan escribe `Peldano elegido:`
  con esta razon (AC-19). -> AC-16.
- OBS-2 **DECIDIDA: doctor no repite nada de `harness_check.sh` y lo dice.**
  harness_check sigue con el **proceso** (spec, plan, PRDs, lecciones, perfil,
  convenciones, espejo de roles); doctor solo con la **instalacion**. Cada salida
  remite a la otra. Cero movimiento de codigo: mover gates que hoy funcionan
  agrandaria el diff sin beneficio para el usuario. -> AC-14.
- OBS-3 **DECIDIDA: `no_aplica` para superficies y hooks en el checkout fuente.**
  Aca no hay `CLAUDE.md` ni hooks y eso es lo correcto (correr el instalador en
  el checkout fuente es el footgun de la #7). Un doctor que grita ocho falsos
  positivos en el repo del propio arnes se ignora en dos dias, y con el se
  ignoran los avisos que si importan. -> AC-12, AC-13.
- OBS-4 **DECIDIDA: falla solo lo que impide trabajar.** Exit 2: binario roto,
  hook apuntando a la nada, herramienta requerida ausente. Aviso (`[i]`, exit 0):
  hub caido, graphify ausente, herramientas opcionales. Toda esta sesion trabajo
  con el hub caido sin un solo problema: si eso saliera 2, el exit code mentiria.
  -> AC-3, AC-9, AC-10, AC-11.
