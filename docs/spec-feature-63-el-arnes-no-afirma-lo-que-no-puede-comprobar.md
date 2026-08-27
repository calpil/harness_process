# Spec - Feature #63: el_arnes_no_afirma_lo_que_no_puede_comprobar

Estado: approved
Aprobado: 2026-08-27T19:30:12Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-63-el-arnes-no-afirma-lo-que-no-puede-comprobar.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: dos afirmaciones del arnes que no se sostienen — un test que sale
verde sin medir, y un mensaje que nombra una ruta que el propio cierre acaba de
borrar.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Alan corre la bateria de `tests/` en su Mac y ve
`[Ok] no-cuelga: harness_check.sh termina con la entrada abierta`. Esa linea es
mentira: `timeout(1)` no existe en macOS, el subshell devuelve 127 y el test
solo considera "colgado" al 124, asi que el modo sale verde **pase lo que
pase**. En la misma corrida, `prueba-del-rojo` falla — y falla con razon: existe
justamente para avisar cuando el modo de al lado dejo de medir. Alan la vio
fallar varias veces y la leyo como ruido de un test viejo.

Y cuando cierra una feature, el arnes le dice
`Estado archivado en ../harness_process-wt/59-cmd-smoke-real-en-windows/docs/estado-feature-59-....md`.
Si copia esa ruta no encuentra nada: el cierre borro ese worktree tres lineas
antes de imprimirla. El archivo existe, pero en `docs/` de la raiz.

DESPUES: el modo `no-cuelga` mide de verdad en macOS y en Linux, y si el
mecanismo de limite no esta disponible el test **falla** en vez de tranquilizar.
`prueba-del-rojo` vuelve a demostrar que el modo de al lado puede ponerse rojo.
Y la ruta que imprime el cierre se puede copiar y pegar: apunta a donde el
archivo esta despues del merge.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                                    DESPUES
(1) el test del cuelgue                (1) el test del cuelgue
    sleep | timeout 10 bash script         sleep | con_limite 10 bash script
     |__ macOS: 127 (no existe)             |__ timeout / gtimeout si estan
     |__ rc != 124 -> "no se colgo"         |__ si no, perl alarm (esta en las dos)
     |__ VERDE pase lo que pase             |__ ninguno -> el test FALLA, no saltea
                                            |__ y un modo nuevo prueba el limite mismo

(2) el mensaje del cierre              (2) el mensaje del cierre
    relpath(archivo_en_worktree,           ¿el cierre integro y borro el worktree?
            raiz_principal)                 |__ SI -> docs/estado-feature-<id>-<slug>.md
     |__ ../<repo>-wt/<id>/docs/...         |__ NO -> la ruta real, que sigue existiendo
     |__ el worktree ya no existe
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como quien corre la bateria en una Mac, quiero que un test que no puede
  medir se ponga rojo, para no confundir "no medi" con "esta bien".
- P1: Como quien acaba de cerrar una feature, quiero poder abrir la ruta que el
  arnes me imprime.
- P2: Como quien lee el andamiaje, quiero que el mecanismo de limite de tiempo
  este en un solo lugar y probado, para poder reusarlo.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->

- AC-1: Given una maquina sin `timeout(1)` (macOS), When corre
  `tests/commit_guard_check.sh`, Then los cuatro modos miden de verdad y el
  script sale 0.
  Comando: `bash tests/commit_guard_check.sh`
- AC-2: Given el mecanismo de limite, When se le da un comando que se cuelga,
  Then lo corta y lo reporta como colgado; y con uno que termina, lo reporta
  como terminado. Es el auto-test del andamiaje: sin esto, "mide" es otra
  afirmacion sin comprobar.
  Comando: `bash tests/commit_guard_check.sh limite`
- AC-3: Given una maquina sin NINGUN mecanismo de limite disponible, When corre
  el test, Then FALLA diciendo cual falta — nunca informa un modo verde ni un
  skip.
  Comando: `bash tests/commit_guard_check.sh limite`
- AC-4: Given la version previa al arreglo de la #52 (invocacion sin
  `</dev/null` y guard sin la guarda `-t 0`), When corre con la entrada abierta,
  Then se cuelga: la prueba-del-rojo vuelve a demostrar que el modo `no-cuelga`
  puede ponerse rojo.
  Comando: `bash tests/commit_guard_check.sh prueba-del-rojo`
- AC-5: Given un cierre `done` que integro y borro el worktree, When imprime
  donde quedo el estado archivado, Then la ruta es `docs/estado-feature-<id>-<slug>.md`
  relativa a la raiz y el archivo existe ahi.
  Comando: `cd rust && cargo test estado_archivado_apunta_a_donde_quedo_el_archivo`
- AC-6: Given un cierre que NO integra (`pending`, `blocked`, `superseded`) o una
  feature sin worktree, When imprime la ruta, Then apunta al archivo tal como
  quedo, que sigue existiendo.
  Comando: `cd rust && cargo test estado_archivado_sin_integrar_mantiene_la_ruta_real`
- AC-7: Given la funcion que decide esa ruta, When se lee, Then es pura: recibe
  si el cierre integro y devuelve el texto, sin consultar el filesystem.
  Comando: `cd rust && cargo test ruta_del_estado_archivado_es_pura`

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- (1) disparador: correr `tests/commit_guard_check.sh`. Mecanismo de limite:
  `timeout` o `gtimeout` (coreutils) si estan, si no `perl -e 'alarm N; exec'`
  (esta en macOS y en Linux). Sin ninguno: error explicito.
- (2) disparador: el cierre imprime donde archivo el estado. Selector: si la
  integracion ocurrio y borro el worktree, la ruta canonica post-merge; si no,
  la ruta real. Se decide con lo que el cierre YA sabe, no consultando el disco.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
(1) CUANDO el test necesita saber si algo se colgo
      elegir mecanismo: timeout | gtimeout | perl alarm
      ¿ninguno? -> FALLAR nombrando cual instalar. Nunca saltear en verde.
      correr con ese limite y traducir su codigo a "se colgo" / "termino"
      y ANTES de usarlo, probar el mecanismo contra un caso que se cuelga
      y uno que no.

(2) CUANDO el cierre informa donde quedo el estado archivado
      ¿la integracion borro el worktree donde lo escribi?
        -> si: la ruta canonica de la raiz, que es donde el merge lo dejo
        -> no: la ruta real, que sigue existiendo
```
Promesas: un test que no puede medir se pone rojo · la ruta que el arnes
imprime se puede abrir.

## No funcionales
- SLOs: el auto-test del limite agrega ~2s a la bateria.
- Seguridad: sin cambios; nada nuevo escribe.
- Observabilidad: el test dice que mecanismo de limite eligio.

## Fuera de alcance
- Reescribir los demas tests de `tests/` que usen `timeout` (se revisa si hay
  otros y se anota, pero cambiarlos no entra aca).
- Cambiar el comportamiento del cierre: solo cambia el TEXTO que informa.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Las dos correcciones en una sola feature — DECIDIDO (USUARIO, 2026-08-27):
  comparten el acuerdo y son chicas.
- Sin skip verde cuando falta el mecanismo — DECIDIDO por la leccion
  `criterios-de-cierre-que-se-pueden-fallar`, que es la que este bug viola: un
  criterio que no se puede fallar no verifica nada.
