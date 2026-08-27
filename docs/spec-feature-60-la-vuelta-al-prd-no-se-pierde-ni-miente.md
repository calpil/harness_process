# Spec - Feature #60: la_vuelta_al_prd_no_se_pierde_ni_miente

Estado: approved
Aprobado: 2026-08-27T15:55:54Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-60-la-vuelta-al-prd-no-se-pierde-ni-miente.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: sacar el log de cierre del branch de la feature y no escribir nunca un
puntero que no resuelve. Dos bugs reportados aguas abajo (#91 y #92) que son la
misma linea de codigo mal ubicada: `echo_to_prd`.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Alan cierra cuatro features el mismo dia, cada una en su worktree, como
el arnes le enseño a trabajar desde la #47. El arnes le dice "PRD actualizado
(hito marcado done + bitacora)" cuatro veces. Cuatro veces es mentira: cada
cierre escribio esa linea en la copia del PRD que vive DENTRO de su worktree, y
las cuatro apendean al final de la misma seccion. El merge de la segunda
conflictua contra la primera; quien resuelve el conflicto se queda con el lado
de `main` y la linea de la rama desaparece sin que nadie lo note. El 25 de
agosto Alan tuvo que abrir el PRD y volver a tipear SIETE lineas a mano
(commit `cf62b24 docs(prd): preserva cierres 40-55`). Y al tipearlas copio el
formato que ya venia roto: 18 de las 30 entradas del PRD maestro apuntan hoy a
`../harness_process-wt/<id>-<slug>/docs/spec-*.md`, worktrees que el propio
cierre borro con `--force` unos segundos despues de escribir el puntero.

DESPUES: Alan cierra las cuatro features y las cuatro lineas estan en el PRD,
en el orden en que cerro, sin conflictos, porque la bitacora dejo de vivir
dentro de las ramas: se escribe en el `docs/prd/` de la raiz DESPUES de que el
merge salio bien. Cada puntero que el arnes escribe abre el archivo que promete.
Y cuando algo impide la vuelta al PRD, Alan se entera en el momento y le queda
un pendiente que `harness_check.sh` le recuerda hasta que lo repare — no un
`[i]` perdido en el scroll.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                                       DESPUES
close --status done                       close --status done
 |__ paths = para_feature(f)               |__ paths_feature = para_feature(f)  (spec, plan, evidencia)
 |__ echo_to_prd(paths_worktree)           |__ integrar(): commit + merge + borrar worktree
 |     |__ escribe <wt>/docs/prd/PRD.md    |     |__ ¿fallo? -> Err: no se marca ningun hito
 |     |__ puntero ../<wt>/docs/spec.md    |__ echo_to_prd(paths_RAIZ)   <-- despues, y en la raiz
 |__ integrar()                            |     |__ decidir_vuelta()  (funcion PURA: arma y valida)
 |     |__ commit en la rama               |     |__ escribir()        (unica que toca el PRD)
 |     |__ merge  <-- CONFLICTO entre      |     |__ puntero docs/spec-feature-<id>-<slug>.md
 |     |             cierres paralelos     |           que RESUELVE, o no se escribe
 |     |__ borrar worktree --force         |__ ¿no se pudo? -> aviso a stderr, cierre igual OK
 |__ <la linea se perdio: 7 de 18>
                                          prd doctor [--reparar]   <-- el pendiente durable
                                           |__ features done sin hito/bitacora
                                           |__ punteros que no resuelven
                                           |__ harness_check.sh lo corre en modo informe
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario que cierra varias features en paralelo, quiero que la vuelta
  al PRD de cada una sobreviva, para no volver a transcribir bitacoras a mano.
- P1: Como lector del PRD, quiero que cada puntero abra el archivo que promete,
  para que la trazabilidad sirva de algo seis meses despues.
- P2: Como dueño de un repo que ya acumulo punteros rotos, quiero un comando que
  me diga cuales son y me los repare, para no auditarlos a ojo.
- P2: Como agente que cierra, quiero enterarme en el momento si la vuelta al PRD
  no se pudo completar, para no reportar un cierre completo que no lo fue.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->

- AC-1: Given una feature con `worktree` registrado, When cierra como `done`,
  Then el hito y la bitacora se escriben en el `docs/prd/` de la RAIZ y la copia
  del PRD que vive en el worktree queda intacta.
  Comando: `cd rust && cargo test close_should_write_the_prd_echo_in_the_root_not_the_worktree`
- AC-2: Given un cierre `done` que integra, When el merge falla o falta `--to`,
  Then NO se marca ningun hito ni se escribe bitacora: un hito marcado afirma que
  el trabajo esta en la rama destino, y no lo esta.
  Comando: `cd rust && cargo test close_should_not_touch_the_prd_when_integration_fails`
- AC-3: Given dos features cerrando contra el mismo PRD desde worktrees
  distintos, When cierran una despues de la otra, Then las DOS lineas quedan en
  el PRD de la raiz y ningun merge conflictua por el PRD. Es la regresion exacta
  de las 7 perdidas medidas en este repo.
  Comando: `cd rust && cargo test dos_cierres_en_paralelo_conservan_las_dos_bitacoras`
- AC-4: Given la bitacora de un cierre, When se escribe el puntero al spec, Then
  es relativo a la raiz (`docs/spec-feature-<id>-<slug>.md`), nunca empieza con
  `../` y nunca nombra un worktree.
  Comando: `cd rust && cargo test punteros_de_bitacora_son_relativos_a_la_raiz`
- AC-5: Given un `docs/impl-<id>.md` que no existe, When se escribe la bitacora,
  Then el segmento `· impl: ...` se OMITE en vez de escribir un puntero roto; y
  lo mismo vale para cualquier otro puntero que no resuelva.
  Comando: `cd rust && cargo test bitacora_omite_el_puntero_que_no_resuelve`
- AC-6: Given un PRD con features cerradas como `done` sin su linea de bitacora o
  con el hito sin marcar, y con punteros que no resuelven, When corre
  `harness prd doctor`, Then los lista con su archivo y su linea y sale con
  codigo distinto de 0; sin hallazgos sale 0 y NO escribe nada.
  Comando: `cd rust && cargo test prd_doctor_reporta_y_no_escribe`
- AC-7: Given los hallazgos de `prd doctor`, When corre con `--reparar`, Then
  reescribe cada puntero al archivo que si existe, elimina el segmento del que no
  existe en ningun lado, agrega la bitacora faltante con la fecha de `closed_at`
  de la feature y marca el hito; y no toca ninguna otra linea del documento.
  Comando: `cd rust && cargo test prd_doctor_reparar_arregla_punteros_y_bitacoras_faltantes`
- AC-8: Given un cierre en el que la vuelta al PRD no se pudo completar (PRD
  ausente, ilegible o no escribible), When termina el cierre, Then el cierre NO
  falla, el aviso sale por **stderr** al final y nombra el comando exacto que lo
  repara; el exit code del cierre no cambia.
  Comando: `cd rust && cargo test aviso_de_vuelta_al_prd_fallida_no_cambia_el_cierre`
- AC-9: Given un repo con punteros rotos o bitacoras faltantes, When corre
  `harness_check.sh`, Then lo reporta como hallazgo (modo informe, sin escribir).
  Comando: `bash tests/prd_doctor_check.sh check`
- AC-10: Given una feature ya registrada en el PRD, When se vuelve a cerrar o se
  corre `prd doctor --reparar` dos veces, Then no se duplica la entrada ni se
  reescribe la fecha del primer cierre (idempotencia, como hoy).
  Comando: `cd rust && cargo test vuelta_al_prd_es_idempotente`
- AC-11: Given el modulo que arma la vuelta al PRD, When se lee el codigo, Then
  la parte que DECIDE (armar la entrada, validar los punteros) es una funcion
  pura que devuelve un plan, y la unica que escribe toma ese plan: la promesa
  "no escribe un puntero roto" la sostiene la estructura, no la disciplina
  (leccion `promesas-estructurales-vs-disciplina`).
  Comando: `cd rust && cargo test decidir_vuelta_es_pura_y_no_escribe`
- AC-12: Given este repo, When se aplica el arreglo, Then los 18 punteros rotos
  de `docs/prd/PRD-master.md` quedan reparados y las features `done` sin bitacora
  aparecen o quedan explicadas.
  Comando: `bash tests/prd_doctor_check.sh repo`

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: `close --status done` que integro bien; y `prd doctor` a pedido.
- raiz de resolucion: la del REPO PRINCIPAL para el PRD (documento raiz,
  compartido, append-only); la del WORKTREE sigue mandando para spec, plan y
  evidencia (doctrina #47/#49/#54, que no se toca).
- punteros: `docs/spec-feature-<id>-<slug>.md` e `docs/impl-<id>.md`, ambos
  relativos a la raiz y verificados contra el filesystem antes de escribirse.
- candado: el mismo de hoy — `- #<id> <name> -> done` como cabeza de entrada;
  una feature ya registrada no se vuelve a anotar y la fecha del primer cierre
  no se reescribe.
- fuente del pendiente: NO un archivo que el cierre tenga que acordarse de
  escribir, sino `feature_list.json` contrastado con el PRD. Una feature `done`
  sin su linea ES el pendiente, la haya anotado alguien o no.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO un cierre `done` termino de integrar

  ¿el merge salio bien?        -> si no, no se marca ningun hito
  ¿existe el PRD en la RAIZ?   -> si no, avisar fuerte y seguir

  decidir_vuelta(feature, raiz) -> PLAN            (funcion pura, no toca disco)
     arma la cabeza de la entrada y la fecha
     por cada puntero candidato:
        ¿es relativo a la raiz y NO empieza con ../?  -> si no, se descarta
        ¿el archivo existe?                           -> si no, se descarta
     devuelve: fila de hito a marcar (o ninguna) + linea de bitacora + descartes

  aplicar(PLAN) -> escribe el PRD de la RAIZ        (la unica que toca el disco)
     ¿ya estaba registrada la feature? -> no duplica, no reescribe la fecha

  ENTONCES el PRD de la raiz queda con el hito y la bitacora,
           con los descartes dichos en voz alta,
           sin que ninguna rama haya tenido que llevar el log.

Y APARTE, a pedido:
  prd doctor [--reparar]
     lee todos los PRD del arbol y el backlog
     reporta: punteros que no resuelven | features done sin hito o sin bitacora
     con --reparar aplica el mismo PLAN que usa el cierre
```
Promesas: el log compartido no viaja en ninguna rama · ningun puntero escrito
sin verificar · el cierre nunca falla por el PRD · idempotente.

## No funcionales
- SLOs: `prd doctor` lee el arbol de `docs/prd/` y el backlog; sin red, sin hub,
  sin escaneo global del repo.
- Seguridad: el PRD es documento del USUARIO — se marca la celda de estado y se
  apendea a la bitacora; el cuerpo no se reescribe nunca. La escritura del arnes
  sobre esa ruta protegida se sigue registrando con
  `registrar_escritura_del_arnes` (feature #26/#58).
- Observabilidad: el cierre dice en que archivo escribio y que descarto; los
  avisos de fallo van a stderr y no cambian el exit code.

## Fuera de alcance
- Cambiar el algoritmo de merge, el orden de integracion o la exigencia de `--to`.
- Cambiar donde viven el spec, el plan y la evidencia (siguen en el worktree).
- Que el cierre commitee el PRD de la raiz: queda como cambio sin commitear, como
  hoy hacen el resto de los documentos del arnes en el checkout principal.
- Reparar los PRD de proyectos aguas abajo automaticamente: se les entrega el
  comando (`prd doctor --reparar`), no el cambio.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Donde aterriza la bitacora — DECIDIDO (USUARIO, 2026-08-27): en el `docs/prd/`
  de la RAIZ, DESPUES de integrar. Sale del branch: elimina la clase de conflicto
  en vez de mitigarla.
- Que pasa si no se puede escribir — DECIDIDO (USUARIO, 2026-08-27): no bloquea
  el cierre, pero deja pendiente durable que `harness_check` y `doctor` reportan.
- Donde vive el gate de punteros — DECIDIDO (USUARIO, 2026-08-27): al escribir
  (no se escribe lo que no resuelve) + check que escanea + reparador de lo ya roto.
- REFINAMIENTO que propone el implementer sobre la 2a decision: el "pendiente
  durable" NO se implementa como un archivo que el cierre escribe al fallar
  (eso vuelve a depender de que el cierre se acuerde), sino derivandolo del
  backlog: una feature `done` cuyo hito o bitacora falta en su PRD ES el
  pendiente, lo haya anotado alguien o no. Es la misma leccion
  `promesas-estructurales-vs-disciplina` aplicada al pendiente. Si preferis el
  archivo explicito, decilo y se agrega.
- Tension con el AC-5 de la #54 ("los documentos escritos en el worktree viajan
  en el merge sin pasos de copia especiales"): sigue valiendo para el CUERPO del
  PRD que `prd propose/apply` modifica. Lo que sale del branch es solo el LOG de
  cierre (hito + bitacora), que es de todas las features y de ninguna rama.
