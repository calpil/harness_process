# Spec - Feature #46: verify_no_se_cuelga_con_salida_grande

Estado: approved
Aprobado: 2026-08-22T17:45:23Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #46 en el chat (9 AC): verify lee los pipes con un hilo por descriptor MIENTRAS el comando corre, para que un comando verboso deje de colgar el gate; el timeout y la medicion sobre la salida completa (leccion #44) no cambian. Decidio las dos observaciones: tope de 4 MB reteniendo la COLA, y el estado se mide sobre lo retenido diciendolo en el reporte.
Plan: docs/plan-feature-46-verify-no-se-cuelga-con-salida-grande.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan declara en un AC el comando que de verdad prueba la feature —el
smoke del instalador, la suite completa, su `verificar-controles`— y
`harness verify` se cuelga. No falla: se queda. Ayer el instalador estuvo **once
minutos** sin avanzar dentro de `verify`, y subir el timeout de 300 a 900
segundos no cambio nada.

La causa esta en `ejecutar()` (`rust/src/verificacion.rs`), y es de manual:

1. lanza el comando con `stdout` y `stderr` en pipes,
2. **espera a que termine** (`wait_timeout`),
3. recien despues lee los pipes (`leer_salida`).

El buffer de un pipe del sistema son ~64 KB. Un comando que imprime mas que eso
se bloquea escribiendo, porque del otro lado nadie lee; y `verify` se bloquea
esperandolo. Se traban mutuamente. Diagnosticado con `lsof` sobre el proceso
colgado: `0r /dev/null`, `1 PIPE`, `2 PIPE`, sin un solo hijo.

La consecuencia no es solo la espera. Es que **los comandos mas completos son
justo los que no se pueden declarar**: el AC-9 de la feature #58 tuvo que quedar
sin `Comando:` y verificarse a mano. Un gate que solo admite comandos calladitos
verifica menos de lo que cree.

DESPUES: `verify` lee los pipes MIENTRAS el comando corre. El smoke —o
cualquier comando verboso— termina, reporta su estado real, y vuelve a ser algo
que se puede escribir en un AC.

## Hoy -> Como va a funcionar

```
HOY                                    DESPUES
spawn(cmd) con pipes                   spawn(cmd) con pipes
  |__ wait_timeout(hijo)  <-- espera     |__ hilo lector de stdout ---> buffer
  |__ leer_salida(hijo)   <-- lee        |__ hilo lector de stderr ---> buffer
                                         |__ wait_timeout(hijo)
  el hijo se bloquea al llenar el pipe   |__ join de los dos hilos
  y nadie lo desbloquea: DEADLOCK        el hijo nunca se bloquea: el pipe se
                                         vacia mientras el escribe
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero declarar como AC el comando que de verdad prueba la
  feature, aunque imprima miles de lineas, para que el gate verifique lo que
  importa y no solo lo que entra en 64 KB.
- P1: Como Alan, quiero que un comando que se cuelga de verdad se siga cortando
  por timeout, para que el gate no espere para siempre por otra razon.
- P2: Como Alan, quiero que una salida gigante no se me coma la memoria ni el
  reporte, y que si se recorta me lo digan.

## Criterios de aceptacion (Given/When/Then)

### Que no se cuelgue

- AC-1: Given un comando que imprime **mas de 64 KB por stdout**, When `verify`
  lo ejecuta, Then termina y reporta su estado real en vez de colgarse.
  Comando: `cd rust && cargo test verify_salida_grande_stdout`
- AC-2: Given lo mismo **por stderr**, Then igual: termina y reporta.
  Comando: `cd rust && cargo test verify_salida_grande_stderr`
- AC-3: Given un comando que escribe mucho **por los dos a la vez** (el caso del
  instalador), Then termina y reporta.
  Comando: `cd rust && cargo test verify_salida_grande_ambos`

### Que no se rompa lo que ya andaba

- AC-4: Given un comando cuyo resumen esta al final de una salida larga
  (`cargo test` imprime la compilacion por stderr y el `test result:` al final),
  When `verify` lo ejecuta, Then el estado se sigue midiendo sobre la salida
  COMPLETA y `casos_corridos` la detecta igual. Es la leccion de la #44 y no se
  toca.
  Comando: `cd rust && cargo test verify_estado_sobre_salida_completa`
- AC-5: Given un comando que NO termina, When corre con timeout, Then se corta y
  se reporta `timeout` (no `rojo`), como hoy.
  Comando: `cd rust && cargo test verify_timeout_sigue_cortando`
- AC-6: Given el reporte, Then sigue mostrando las ultimas `LINEAS_SALIDA` con
  el aviso de cuantas se omitieron.

### Que no se coma la maquina

- AC-7: Given un comando que imprime una salida ENORME, When `verify` lo
  ejecuta, Then no se retiene sin limite: se guarda como mucho el tope decidido
  en OBS-1 y, si hubo recorte, el reporte lo dice (cuanto quedo afuera).
  Comando: `cd rust && cargo test verify_salida_acotada`

### La prueba real

- AC-8: Given un spec que declara `bash tests/setup_smoke.sh` como comando de un
  AC, When corro `harness verify`, Then termina con el estado real del smoke
  —ni `timeout` ni cuelgue— y el reporte lo muestra. Es el caso que dejo a la
  #58 sin comando ejecutable.
  Comando: `bash tests/setup_smoke.sh`
- AC-9: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, `bash tests/setup_smoke.sh` y
  `bash harness_check.sh`, Then los cuatro terminan limpios.
  Comando: `cd rust && cargo clippy --all-targets -- -D warnings`

## Los datos que se tocan

- `rust/src/verificacion.rs`: `ejecutar()` y `leer_salida()`. Nada mas.
- Sin dependencias nuevas: `std::thread` y `std::sync::mpsc` alcanzan
  (`wait-timeout` sigue igual para el corte).
- Sin cambios de formato en `docs/verify-<id>.md` salvo la linea de recorte de
  AC-7.
- `rules.verify_timeout_segundos` sigue significando lo mismo.

## Pseudo-codigo (el acuerdo)

```
lanzar el comando con stdout y stderr en pipes
lanzar UN HILO por pipe, que lee hasta el EOF y acumula (con tope)
esperar al proceso con timeout
   si se pasa -> matarlo (los pipes se cierran y los hilos terminan solos)
juntar los dos hilos
medir el estado sobre la salida COMPLETA (leccion #44)
recortar SOLO para el reporte, diciendo cuanto se omitio
```

Promesas: un comando verboso termina · un comando colgado se corta igual · la
memoria tiene tope y el recorte se declara.

## No funcionales

- SLOs: dos hilos por comando verificado; el costo es despreciable al lado de lo
  que tarda el comando.
- Seguridad: no cambia que se ejecuta ni con que permisos.
- Observabilidad: si se recorta por tope, el reporte lo dice.

## Fuera de alcance

- Mostrar la salida en vivo mientras corre (streaming al terminal).
- Cambiar el timeout por default ni su regla.
- Paralelizar la ejecucion de varios AC.

## Observaciones y decisiones

- OBS-1 [DECIDIDA por el USUARIO, 2026-08-22]: el tope de retencion es **4 MB**
  por comando. Mas que suficiente para cualquier suite (el smoke completo son
  ~200 KB) y con defensa contra un comando que imprime infinito.
- OBS-2 [DECIDIDA por el USUARIO, 2026-08-22]: cuando el tope se aplica, se
  retiene la **cola** (los ultimos 4 MB) y el estado se mide sobre eso, porque
  ahi estan los resumenes (`test result:`, `FAILED`). El reporte declara que
  hubo recorte y cuanto quedo afuera.
