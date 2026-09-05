# Review - Feature #75: el backlog no sabe de dependencias ni de features que se traban una y otra vez
Revisado: approved · 2026-09-05T14:06:00Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento. Metodo: medir las premisas de la ficha
ANTES de escribir los AC, y mutar produccion para confirmar que cada test cae.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/dependencias.rs:45 | CUBIERTO. `motivo_invalido` es pura y el test de comportamiento afirma que la feature NO se creo —no solo que el exit fue 2—, que es la diferencia entre "el comando fallo" y "el backlog quedo intacto". |
| AC-2 | rust/src/commands/next.rs:19 | CUBIERTO. Dos tests: uno que comprueba que la que espera no se ofrece y se libera al cerrar su dependencia, y otro que comprueba que cuando NO queda ninguna se dice por que, con nombres. Probado en rojo. |
| AC-3 | rust/src/commands/start.rs:289 | CUBIERTO. El test afirma el aviso Y que la feature arranco: un aviso que ademas bloqueara pasaria la mitad del test. |
| AC-4 | rust/src/dependencias.rs:199 | CUBIERTO CON RESERVA DECLARADA. Dos tests, uno que dispara el gate y otro que comprueba que no se dispara antes. Es toda su evidencia: la condicion no se observo nunca en 84 cierres reales, y eso esta escrito en el spec, no escondido. |
| AC-5 | rust/src/dependencias.rs:80 | CUBIERTO, y solo despues de agregar `harness depende`: por `add` el ciclo era inalcanzable. Ver abajo. |
| AC-6 | rust/src/dependencias.rs:145 | CUBIERTO. El test comprueba sobre el JSON que ni `depends_on` ni `bloqueos` nacen solos, y que `next` y `start` funcionan igual sin ellos. |
| AC-7 | rust/src/commands/status.rs:73 | CUBIERTO. |
| AC-8 | rust/src/dependencias.rs:45 | CUBIERTO. Clippy limpio, suite verde, paridad diez modos, commit_guard ocho, stop_hook diez. |

## Lo que el review encontro en el propio trabajo

**1. Un test mio que no probaba lo que su nombre decia.** Escribi
`add_should_refuse_a_dependency_cycle` y su cuerpo terminaba comprobando una
cadena valida, porque por `add` el ciclo es imposible: una feature nueva solo
puede depender de ids anteriores, asi que el grafo es un DAG por construccion.
Es el defecto que la feature #73 documento hace unas horas —el test cuyo nombre
afirma algo que el cuerpo no hace— aparecido en el trabajo siguiente. Se
reemplazo por uno que si prueba un ciclo.

**2. La feature no cumplia su propio recorrido P1.** Con solo
`add --depends-on`, "Alan declara que la #21 depende de la #17" es imposible:
las dos ya existen y `add` crea. Se agrego `harness depende`, que es ALCANCE
NUEVO respecto de la ficha y esta dicho como tal en el plan (OBS-3), en el impl
y aca. Sin el, el AC-5 tampoco era alcanzable desde el CLI.

## Lo que NO esta verificado

- **El valor del AC-4.** Que el gate funciona esta probado; que HAGA FALTA, no.
  Su condicion nunca se dio en 84 cierres. Es una decision del usuario tomada
  con la medicion delante, y queda registrada por si dentro de seis meses
  alguien pregunta por que existe ese contador.
- **El valor de las dependencias tampoco esta medido retrospectivamente.**
  Ninguna feature del historial se rompio por falta de este campo. El caso a
  favor es prospectivo: el programa de aprendizaje (#17-#22) estaba "ordenado
  por dependencia" y ese orden vivia en prosa.
- ~~No se probo sobre el backlog real.~~ **Se probo.** El recorrido P1 se camino
  sobre el `feature_list.json` de este repo, no sobre un sandbox:

  ```
  $ harness depende --feature 21 --de 17
  Feature #21 declarada(s): depende de #17

  $ harness depende --feature 17 --de 21
  No se puede declarar esa dependencia para la feature #17: formaria un ciclo:
  #17 -> #21 -> #17.
      El backlog no se toco.
  ```

  El backlog se restauro despues: quedo con **0 features con `depends_on` y 0 con
  `bloqueos`**, que es lo que el AC-6 exige. No se dejaron declaraciones que el
  usuario no pidio.

## Riesgos que el cambio introduce

- **Un comando nuevo** (`depende`), peldano 3 de la escalera. La razon esta
  escrita: ningun comando existente edita una feature.
- **`next` cambia de comportamiento** cuando hay dependencias declaradas. Sin
  ninguna declarada —el estado actual del repo— se comporta igual que antes.

## Veredicto

Los ocho AC tienen cobertura. Las tres mutaciones ponen en rojo al menos un test
cada una. Las dos reservas —el valor del breaker y el de las dependencias— estan
dichas, no disimuladas.
