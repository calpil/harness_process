# Spec - Feature #26: rutas_protegidas_deny

Estado: approved
Aprobado: 2026-08-17T20:13:23Z por USUARIO (confirmacion explicita) - Alan aprobo en el chat tras el ritual (spec mostrado + abierto en editor). Decisiones OBS-1..OBS-4: rules.rutas_protegidas en feature_list.json, la deteccion avisa con el comando de reversion, se agrega PreToolUse para Claude con el limite de prueba declarado, y la red de seguridad bloquea con exit 2.
Plan: docs/plan-feature-26-rutas-protegidas-deny.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: el README dice que los PRD son del usuario y que ningun agente los
reescribe. La constitution dice que sus articulos se cambian con decision humana.
**Las dos cosas son buena fe.** No hay un solo gate: cualquier agente con
permisos de escritura puede reescribir `docs/prd/PRD-master.md` o
`docs/constitution.md` en un `Edit`, y nadie se entera hasta que alguien lee el
diff. Con un backend en modo permisivo (yolo/auto-approve), ni siquiera hay un
prompt en el medio.

Es el unico lugar del arnes donde una regla importante depende de que el agente
se porte bien. Todas las demas —spec aprobado, leccion declarada, reporte verde—
tienen su gate.

DESPUES: hay una lista de **rutas protegidas** y tres capas que la hacen valer,
declaradas por lo que cada una puede y no puede hacer:

1. **Prevenir** (`PreToolUse`, donde el backend lo soporta): la escritura no
   ocurre. Es la unica capa que de verdad impide.
2. **Detectar al instante** (`PostToolUse`, en todos los backends): la escritura
   ya ocurrio; el agente recibe el aviso y **el comando exacto para revertirla**.
3. **Red de seguridad** (`harness_check.sh`): la feature no cierra con una ruta
   protegida modificada.

Nada de esto vale si el arnes se bloquea a si mismo: `close` **escribe** en
`docs/prd/PRD-master.md` cada vez que marca un hito. La proteccion es contra las
herramientas del agente, no contra el binario del arnes.

## Hoy -> Como va a funcionar

```
HOY                                    DESPUES
agente edita docs/prd/PRD-master.md    agente intenta editarlo
  -> se escribe                          -> [PreToolUse] denegado, no se escribe
  -> nadie se entera                     -> o [PostToolUse] "revertilo con: git checkout -- <ruta>"
                                         -> y harness_check no deja cerrar

harness close marca el hito             harness close marca el hito
  -> escribe en el PRD                   -> escribe en el PRD (no es el agente)
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero que mis PRD y mi constitution no se puedan reescribir por
  accidente, ni siquiera con el backend en modo permisivo.
- P1: Como agente, quiero que cuando toco algo protegido me lo digan **con el
  comando para deshacerlo**, no con un reproche.
- P2: Como usuario de otro proyecto, quiero poder agregar mis propias rutas
  (`.env`, `infra/**`) sin tocar el codigo del arnes.

## Criterios de aceptacion (Given/When/Then)

<!-- Comportamiento con tests; documentacion con greps (leccion
     `criterios-de-cierre-que-se-pueden-fallar`). Ningun comando repetido. -->

### La lista y el matcher

- AC-1: Given la configuracion por defecto, Then las rutas protegidas son
  `docs/prd/**`, `docs/constitution.md` y `.env`, y se pueden ampliar sin tocar
  el codigo del arnes.
  Comando: `cd rust && cargo test deny_should_protect_the_three_defaults`
- AC-2: Given un patron con `**`, Then matchea a cualquier profundidad
  (`docs/prd/**` cubre `docs/prd/PRD-master.md` y `docs/prd/aprendizaje/x.md`);
  con `*` matchea un solo segmento.
  Comando: `cd rust && cargo test deny_should_match_globs_at_any_depth`
- AC-3: Given una ruta absoluta o relativa a la raiz, Then el matcher decide lo
  mismo: la forma de escribir la ruta no puede cambiar si esta protegida.
  Comando: `cd rust && cargo test deny_should_normalize_absolute_and_relative_paths`
- AC-4: Given una ruta que NO esta en la lista, Then no se reporta: la lista es
  la unica fuente, sin heuristicas ni "parece un PRD".
  Comando: `cd rust && cargo test deny_should_not_guess_beyond_the_list`

### Las tres capas, cada una con su alcance declarado

- AC-5: **Prevenir.** Given un backend que soporta un evento previo a la
  herramienta, When el agente intenta escribir una ruta protegida, Then la
  escritura **no ocurre** y el agente recibe el motivo.
  Comando: `bash tests/deny_check.sh previene`
- AC-6: **Detectar.** Given un backend sin evento previo, When el agente escribe
  una ruta protegida, Then el `PostToolUse` lo reporta al instante con **el
  comando exacto para revertir** (`git checkout -- <ruta>`).
  Comando: `bash tests/deny_check.sh detecta`
- AC-7: **Red de seguridad.** Given una ruta protegida modificada y sin
  commitear, When corre `harness_check.sh`, Then lo reporta con la ruta y el
  comando de reversion.
  Comando: `bash tests/deny_check.sh red-de-seguridad`
- AC-8: Given las tres capas, Then la documentacion dice **explicitamente que
  PostToolUse no puede prevenir**, solo detectar: prometer bloqueo donde solo hay
  deteccion es peor que no prometer nada.
  Comando: `grep -q "no puede prevenir" docs/rutas-protegidas.md`

### El arnes no se bloquea a si mismo

- AC-9: Given `sh harness_cli close --status done`, When marca el hito en
  `docs/prd/PRD-master.md`, Then **no se bloquea ni se reporta**: la proteccion
  es contra las herramientas del agente, no contra el binario del arnes.
  Comando: `cd rust && cargo test close_should_still_write_the_prd_milestone_when_protected`
- AC-10: Given `harness_check.sh` tras un `close` legitimo, Then no reporta
  violacion por el PRD que el propio arnes acaba de escribir.
  Comando: `bash tests/deny_check.sh no-se-autobloquea`

### Configurable sin tocar el arnes

- AC-11: Given un proyecto que agrega sus rutas, When corre cualquiera de las
  tres capas, Then las respeta sin haber tocado codigo del arnes.
  Comando: `cd rust && cargo test deny_should_read_user_defined_paths`
- AC-12: Given una configuracion ausente, Then valen los tres defaults y nada
  falla: una instalacion que no configura nada queda protegida igual.
  Comando: `cd rust && cargo test deny_should_fall_back_to_defaults_when_unconfigured`
- AC-13: Given la lista vacia explicitamente, Then la proteccion queda apagada
  sin errores: el usuario puede decidir que no la quiere.
  Comando: `cd rust && cargo test deny_should_be_disablable_with_an_empty_list`

### Que no rompa lo que ya anda

- AC-14: Given una instalacion existente que nunca oyo hablar de esto, When
  corre `harness_check.sh`, Then se comporta igual que antes salvo por las
  violaciones reales.
  Comando: `bash tests/deny_check.sh compatible`
- AC-15: Given el hook `PostToolUse`, Then el chequeo no agrega latencia
  perceptible ni bloquea el turno cuando no hay violacion.
  Comando: `bash tests/deny_check.sh sin-costo`
- AC-16: Given `doctor`, Then reporta si las rutas protegidas estan activas y
  cuantas hay, sin duplicar el chequeo de violaciones (que es de
  `harness_check.sh`).
  Comando: `cd rust && cargo test doctor_should_report_protected_paths_status`

### Integracion, docs y verificacion

- AC-17: Given `docs/rutas-protegidas.md` (+ plantilla), Then explica las tres
  capas, que puede cada una, como agregar rutas y como apagarlo.
  Comando: `diff -q docs/rutas-protegidas.md templates/docs/rutas-protegidas.md`
- AC-18: Given `README.md` y `UPDATING.md` (+ espejo), Then documentan la
  proteccion y el limite de la capa de deteccion.
  Comando: `grep -q "rutas protegidas" README.md UPDATING.md templates/UPDATING.md`
- AC-19: Given los tres roles, Then el agente sabe que hacer cuando toca una ruta
  protegida (revertir y pedirle al usuario), y el reviewer lo verifica.
  Comando: `grep -q "ruta protegida" roles/implementer.md roles/reviewer.md`
- AC-20: Given el plan, Then declara `Peldano elegido:` con la razon, como exige
  `docs/conventions.md`.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-26-rutas-protegidas-deny.md`
- AC-21: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test`, `cargo clippy --all-targets -- -D warnings`, `tests/setup_smoke.sh`
  y `harness_check.sh` siguen verdes.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: una escritura del agente sobre una ruta que matchea la lista.
- interruptor: lista vacia = proteccion apagada (AC-13).
- candado: no aplica; el chequeo es idempotente y no muta nada.

## Pseudo-codigo (el acuerdo)

```
CUANDO el agente va a escribir <ruta>

  ¿<ruta> matchea la lista de protegidas?
      -> si el backend tiene evento PREVIO: se deniega, no se escribe
      -> si no: se escribe (no hay forma de impedirlo) y se avisa al instante
         con "revertilo: git checkout -- <ruta>"

CUANDO corre harness_check.sh

  ¿hay rutas protegidas modificadas y sin commitear?
      -> se reportan con su comando de reversion

CUANDO el binario del arnes escribe el PRD (close marcando un hito)

  -> no pasa por ninguna de las tres capas: no es el agente
```

## No funcionales

- Sin dependencias nuevas (Articulo 6).
- El chequeo del `PostToolUse` corre en cada `Edit|Write|MultiEdit`: tiene que
  ser barato (AC-15).
- Cero escrituras: ninguna capa modifica archivos del usuario por su cuenta.

## Fuera de alcance

- **Revertir automaticamente.** Ver OBS-2: se decide, pero la propuesta es
  avisar con el comando y que lo corra quien decida.
- Proteger contra escrituras fuera del agente (un `vim`, otro script): eso es
  `.gitignore`/permisos del sistema, no el arnes.
- Proteger rutas **fuera** de la raiz del repo.
- Firmar o versionar los archivos protegidos.

## Observaciones (decididas por Alan el 2026-08-17)

- OBS-1 **DECIDIDA: `rules.rutas_protegidas` en `feature_list.json`.** Peldano 1
  de la escalera: ahi ya viven `require_spec_approved`, `require_leccion` y
  `require_verify_green`, y el usuario ya las edita a mano. Cero superficie
  nueva y un solo lugar para toda la configuracion del arnes. Se descarta el
  `harness.deny` que proponia el PRD; el plan lo justifica. -> AC-1, AC-11.
- OBS-2 **DECIDIDA: la capa de deteccion avisa con el comando de reversion, no
  revierte.** `git checkout -- <ruta>` va en el mensaje y lo corre quien decida.
  Coherente con el curador (#21: nunca mueve nada sin `--aplicar`) y con doctor
  (#25: imprime el remedio y no lo ejecuta). Revertir solo podria borrar trabajo
  legitimo que el usuario pidio, sin aviso. -> AC-6.
- OBS-3 **DECIDIDA: se agrega `PreToolUse` para Claude Code, con el limite de
  prueba declarado.** Es el backend en uso y el que soporta denegar. El limite,
  dicho de frente: no se puede probar de punta a punta en esta maquina (haria
  falta correr Claude Code de verdad), asi que el AC-5 se verifica sobre el JSON
  generado y el comportamiento del script, **no** sobre una denegacion real.
  -> AC-5.
- OBS-4 **DECIDIDA: la red de seguridad BLOQUEA (exit 2).** Es el proposito de la
  feature: dejar de depender de la buena fe. A diferencia de la regla de tests de
  la #24, aca no hay excepcion legitima — nadie deberia editar un PRD desde un
  agente. `HARNESS_CHECK_MODE=warn` sigue siendo la valvula de escape. -> AC-7.
