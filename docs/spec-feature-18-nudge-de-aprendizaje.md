# Spec - Feature #18: nudge_de_aprendizaje

Estado: approved
Aprobado: 2026-08-16T23:06:20Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #18 en el chat (AskUserQuestion: 'Si, lo apruebo'), con el spec mostrado en el chat y abierto en su editor. 21 AC. Decisiones OBS-1..OBS-7: sin docs/lecciones no se emite nada, intervalo en rules y no en env, contrato de cierre solo cuando no hubo declaracion, backoff 600s->3600s (techo 1 hora, se descarto el de un dia a proposito), recordatorio y aviso de plan stale independientes, el contrato se LEE de la guia (una sola fuente de verdad, con degradacion a puntero en AC-21) y default de cadencia 25 en vez de 10 para que no se vuelva ruido de fondo.
Plan: docs/plan-feature-18-nudge-de-aprendizaje.md
PRD: docs/prd/aprendizaje/PRD-aprendizaje.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: la feature #17 le dio al arnes un lugar donde guardar lo que aprende
(`docs/lecciones/<clase>.md`) y un gate opcional que lo exige al cerrar. Pero
**nadie se lo recuerda a nadie**. Un agente que trabaja tres horas seguidas
resolviendo un problema no trivial no tiene ningun momento en que se le ocurra
mirar el catalogo, y al cerrar —si la regla `require_leccion` esta apagada, que
es el default— simplemente cierra. El repositorio termino la #17 con dos
lecciones porque **yo me acorde**, no porque el arnes lo pidiera.

Un sistema de aprendizaje que depende de que alguien se acuerde no es un sistema
de aprendizaje.

DESPUES: el arnes empuja solo, en los dos momentos donde hay senal de verdad.
Cada N escrituras le recuerda al agente que esta corriendo —el que sea— que mire
si apareco algo que patchear, en tres lineas por stderr que no interrumpen nada.
Y al cerrar una feature sin declarar leccion, le pone delante el **contrato**
completo: el orden de preferencia y la lista de que NO capturar. El arnes nunca
escribe la leccion: emite el contrato, el agente decide, el gate verifica.

De paso, el recordatorio de "sin feature activa" —hoy un debounce fijo de 600s
que repite lo mismo cada diez minutos para siempre— aprende a callarse: si nada
cambia, se espacia; apenas algo cambia, vuelve al piso.

## Hoy -> Como va a funcionar

```
HOY                                      DESPUES

hook PostToolUse -> harness_cli nudge    hook PostToolUse -> harness_cli nudge
  |__ ¿sin feature activa?                 |__ ¿sin feature activa?
  |     `__ aviso, debounce fijo 600s      |     `__ aviso, debounce que ESCALA
  |          (repite igual para siempre)   |          600s -> 1200s -> ... -> techo
  |                                        |          y vuelve al piso si algo cambio
  `__ ¿plan stale? -> aviso                `__ ¿plan stale? -> aviso
                                           `__ +1 al contador de la feature
                                                 ¿llego a N (default 25)?
                                                   `__ recordatorio corto de leccion

close --status done                      close --status done
  |__ ... cierra ...                       |__ ... cierra ...
  (fin)                                    `__ ¿NO se declaro leccion?
                                                 `__ CONTRATO leido de la GUIA,
                                                     por stderr (orden de
                                                     preferencia + que NO
                                                     capturar + comandos)
```

## Recorridos de usuario (priorizados)

- P1: Como agente de cualquier backend, quiero que el arnes me recuerde mirar las
  lecciones mientras trabajo, para no terminar la feature sin haber capturado lo
  que costo aprender.
- P1: Como agente que cierra una feature sin gate, quiero tener delante el
  contrato completo en ese momento, porque es cuando se sabe que funciono y que
  no.
- P1: Como Alan, quiero que el recordatorio NO sea ruido: que se calle cuando no
  aporta y que nunca frene ni rompa un turno.
- P2: Como usuario de un proyecto que no usa lecciones, quiero no ver nada nuevo:
  sin `docs/lecciones/` el arnes se comporta exactamente como antes.

## Criterios de aceptacion (Given/When/Then)

### Disparador por volumen de trabajo

- AC-1: Given una feature activa y `docs/lecciones/` presente, When el hook
  `PostToolUse` invoca `nudge` por N-esima vez (N = `leccion_nudge_interval`,
  default **25**), Then se emite por **stderr** un recordatorio corto (<= 5
  lineas) que nombra el comando `leccion list` y recuerda que patchear va antes
  que crear; el contador se resetea; y el exit code sigue siendo **0**.
- AC-2: Given las invocaciones intermedias (1..N-1), When corre `nudge`, Then
  **no** se emite el recordatorio de lecciones (el resto del comportamiento
  —aviso de plan stale— no cambia).
- AC-3: Given un proyecto **sin** `docs/lecciones/`, When corre `nudge` cualquier
  cantidad de veces, Then nunca se emite el recordatorio de lecciones y no se
  crea ningun archivo de contador: el arnes se comporta byte a byte como antes.
- AC-4: Given que la feature activa cambia (se cierra una y se arranca otra),
  When corre `nudge`, Then el contador arranca de cero para la feature nueva.
- AC-5: Given `rules.leccion_nudge_interval` con un valor distinto del default de
  25 (por ejemplo 3), When corre `nudge`, Then el recordatorio sale cada 3
  invocaciones; con valor `0` o negativo el recordatorio queda **apagado**.

### Contrato al cerrar

- AC-6: Given `close --status done` **sin** declaracion de leccion, When el
  cierre termina, Then se emite por stderr el **contrato completo**, y su texto
  se **LEE de `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`** — no vive duplicado
  en el binario. Se emiten las dos secciones que son el contrato de verdad:
  `## La regla que ordena todo: primero patchear, crear al final` y
  `## Que NO capturar`, seguidas de los comandos (`leccion list` / `leccion
  nueva` / `leccion usar`) y del recordatorio de que `ninguna` es valido pero no
  es el default. Editar la guia cambia el contrato sin recompilar: **una sola
  fuente de verdad**.
- AC-7: Given `close --status done --leccion <clase>` (o `--leccion ninguna` con
  motivo), When el cierre termina, Then **no** se emite el contrato: el trabajo
  ya se hizo.
- AC-8: Given `close --status blocked` o `--status pending`, When el cierre
  termina, Then **no** se emite el contrato (no se le pide una leccion a algo que
  no se termino).
- AC-9: Given un proyecto sin `docs/lecciones/`, When se cierra como done sin
  declaracion, Then **no** se emite nada nuevo.
- AC-10: Given cualquiera de los casos anteriores, When el cierre termina, Then
  el **exit code y el stdout** del comando `close` son identicos a los de hoy: el
  contrato va a stderr y nunca cambia el resultado de la operacion.

### Backoff adaptativo del aviso "sin feature activa"

- AC-11: Given que no hay feature activa, When corre `nudge` por primera vez,
  Then se emite el aviso y el debounce queda en el **piso** (600 s).
- AC-12: Given que ya se emitio el aviso y nada cambio, When vuelve a correr
  `nudge` pasado el debounce, Then se emite de nuevo y el intervalo **se duplica**
  (600 -> 1200 -> 2400 -> ...) hasta un **techo** de 3600 s, donde se estaciona.
- AC-13: Given que el intervalo escalo, When aparece una feature activa, Then el
  nivel vuelve al **piso**, de modo que el proximo periodo sin feature avisa a
  los 600 s otra vez.
- AC-14: Given una instalacion previa cuyo `progress/.last_nudge` esta vacio,
  When corre `nudge`, Then se interpreta como nivel 0 (piso) sin error: el
  formato nuevo es compatible hacia atras.

### Invariantes del nudge

- AC-15: Given cualquier error interno (JSON corrupto, `progress/` sin permisos,
  contador ilegible), When corre `nudge`, Then el exit code es **0**, no se
  propaga ninguna excepcion y ningun turno se rompe: sigue siendo best-effort
  absoluto.
- AC-16: Given cualquier ejecucion de `nudge`, Then **no** se crea, modifica ni
  borra ninguna leccion, ningun spec, plan, impl o review: el nudge solo emite
  texto y su propio contador/stamp.
- AC-17: Given el hub PostgreSQL caido, When corre `nudge` o `close`, Then el
  comportamiento del recordatorio y del contrato es identico (no dependen del
  hub).

### Documentacion y verificacion

- AC-18: Given `README.md`, `UPDATING.md` (+ espejo) y las superficies generadas
  por ambos instaladores, Then documentan los dos disparadores, la regla
  `leccion_nudge_interval` con su default y el backoff; y `docs/architecture.md`
  describe el contador y el stamp como estado local de `progress/`.
- AC-19: Given los tres roles, Then el implementer explica que hacer cuando ve el
  recordatorio (mirar el catalogo y patchear antes que crear) y el reviewer
  verifica que un cierre sin declaracion haya recibido el contrato.
- AC-20: Given el repo fuente, When se corre la verificacion oficial, Then
  `cargo test` y `cargo clippy --all-targets -- -D warnings` estan verdes con
  tests del contador, del backoff (incluido el reset), de las tres ramas del
  contrato, de la degradacion de AC-21 y del invariante de exit 0; y
  `tests/setup_smoke.sh` sigue verde.
- AC-21: Given que `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md` falta, esta
  vacio o no contiene las dos secciones esperadas, When se cierra como done sin
  declaracion, Then el contrato **degrada** a un puntero de dos lineas (que paso
  + donde esta el metodo) en vez de fallar o de emitir un texto truncado; el exit
  code del cierre sigue siendo el mismo. Leer la guia nunca puede romper un
  cierre.

## Los datos que se tocan

- **disparador**: la invocacion de `nudge` desde el hook `PostToolUse` (que ya
  existe y ya filtra por matcher `Bash|Edit|Write|apply_patch`), y el final de
  `close --status done`.
- **interruptor**: `rules.leccion_nudge_interval` en `feature_list.json`
  (default 25; `0` apaga el recordatorio periodico). Y el interruptor implicito y
  mas importante: **si no existe `docs/lecciones/`, nada de esto corre**.
- **fuente del contrato**: `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`. El
  binario lo LEE (secciones `## La regla que ordena todo...` y
  `## Que NO capturar`); no guarda una copia. Si falta o esta incompleto, degrada
  a un puntero.
- **candado**: `progress/.nudge_lecciones` guarda `<id-feature>:<contador>`, asi
  que cambiar de feature resetea; `progress/.last_nudge` guarda el **nivel** de
  backoff y su mtime es el ultimo aviso.
- **lo que NO se toca**: ninguna leccion, ningun artefacto de feature. El nudge
  escribe unicamente sus dos archivos de estado local en `progress/`.

## Pseudo-codigo (el acuerdo)

```
CUANDO el hook post-tool invoca el nudge

  ¿existe docs/lecciones/?        -> si no, no hacemos nada nuevo
  ¿hay una feature activa?        -> si no, avisamos con backoff y salimos
  ¿el intervalo esta apagado (0)? -> si si, no hacemos nada

  sumamos uno al contador de ESTA feature
  ¿llego al intervalo?            -> si no, no hacemos nada

  ENTONCES emitimos el recordatorio CORTO por stderr y reseteamos el contador,
           con la restriccion de que el exit code sigue siendo 0 pase lo que pase.


CUANDO termina un close --status done

  ¿existe docs/lecciones/?        -> si no, no hacemos nada
  ¿el cierre declaro leccion?     -> si si, no hacemos nada (ya se hizo)

  leemos el contrato de la GUIA (sus dos secciones)
  ¿la guia falta o esta incompleta? -> emitimos solo el puntero y salimos

  ENTONCES emitimos el CONTRATO completo por stderr,
           con la restriccion de que no cambia ni el exit code ni el stdout
           del cierre.
```

**Promesas:** el arnes empuja, nunca escribe · nada frena un turno (exit 0
siempre) · un proyecto sin lecciones no ve nada nuevo · el aviso repetido se
espacia solo.

## No funcionales

- **SLOs**: el nudge corre en cada tool-use, asi que su costo tiene que ser
  despreciable: dos lecturas de archivos chicos y ninguna conexion de red ni al
  hub.
- **Seguridad**: no escribe fuera de `progress/`; el texto emitido es fijo (no
  interpola contenido del usuario que pueda inyectar instrucciones).
- **Observabilidad**: todo va a **stderr** para no contaminar el stdout que
  consumen los hooks; exit code 0 invariante.

## Fuera de alcance

- El perfil del usuario y su inyeccion en superficies (feature #19).
- `buscar` (feature #20), el curador (#21) y el mapa `journey` (#22).
- Cambiar el gate `require_leccion` o su default (es de la #17; sigue opt-in).
- Cualquier llamada a un modelo: el arnes emite el contrato, no lo ejecuta.
- Tocar el matcher de los hooks o agregar eventos nuevos a los backends.

## Observaciones (decisiones pendientes)

Todas decididas por Alan el 2026-08-16, en el mismo acto de aprobacion del spec.
No queda ninguna observacion abierta: el implementer puede avanzar sin preguntar.

- OBS-1: ¿El recordatorio se calla sin `docs/lecciones/`? — **DECIDIDO: si.**
  Mismo principio que el bloque de `harness_check.sh`: un proyecto que no usa
  lecciones no se entera de que existen. Vinculante para AC-3 y AC-9.
- OBS-2: ¿El intervalo en `rules` o en env var? — **DECIDIDO: en `rules`**
  (`leccion_nudge_interval`, default 10), nada de variables de entorno para
  config no-secreta. Vinculante para AC-5.
- OBS-3: ¿El contrato de cierre siempre o solo sin declaracion? — **DECIDIDO:
  solo cuando no hubo declaracion.** Si declaraste, ya hiciste el trabajo y
  repetirlo es ruido. Vinculante para AC-6 y AC-7.
- OBS-4: ¿Hasta donde escala el backoff? — **DECIDIDO: piso 600 s, duplicando por
  cada aviso sin cambios, techo 3600 s (1 hora).** Se descarto el techo de un dia
  a proposito: un repo en el que se trabaja sin cargar feature tiene que seguir
  avisando una vez por hora, porque el silencio total es justo el escenario en
  que no se captura nada. Vinculante para AC-11, AC-12 y AC-13.
- OBS-5: ¿El recordatorio convive con el aviso de plan stale? — **DECIDIDO: si**,
  son independientes y pueden salir juntos; el aviso de plan stale no se toca.
- OBS-6: ¿De donde sale el texto del contrato? — **DECIDIDO: se LEE de
  `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`**, no se duplica en el binario.
  Una sola fuente de verdad: editar la guia cambia el contrato sin recompilar, y
  guia y contrato no pueden divergir. El costo aceptado es que hay que degradar
  cuando la guia falta o esta incompleta, y por eso existe el AC-21.
- OBS-7: ¿Cada cuantas invocaciones? — **DECIDIDO: default 25, no 10.** El hook
  matchea `Bash|Edit|Write|apply_patch`, asi que 10 invocaciones son un par de
  minutos de trabajo: a esa frecuencia el recordatorio se convierte en ruido de
  fondo que todos aprenden a ignorar, que es la peor falla posible para un aviso.
  Con 25 son unos pocos por hora. Sigue configurable por `rules`. Vinculante para
  AC-1 y AC-5.
