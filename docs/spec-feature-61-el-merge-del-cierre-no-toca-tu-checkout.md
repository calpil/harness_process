# Spec - Feature #61: el_merge_del_cierre_no_toca_tu_checkout

Estado: approved
Aprobado: 2026-08-27T18:19:59Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-61-el-merge-del-cierre-no-toca-tu-checkout.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: que el merge del cierre corra SIEMPRE aislado, y que la unica
situacion que de verdad no se puede resolver sola se detecte antes de tocar
nada. Hermano de la #60: la misma promesa que se cae cuando cambia el contexto.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Alan cierra la #60 con `--to main` estando parado en `main`. El arnes
pasa los cuatro gates, commitea el worktree de la feature... y ahi el merge
explota con el texto crudo de git: "Your local changes to the following files
would be overwritten by merge". El cierre queda a medias: la feature ya figura
`done` en el backlog, el worktree ya tiene un commit de cierre, y el trabajo no
esta integrado. `git.rs` promete en su cabecera que "el merge corre en un
worktree temporal (no toca tu checkout)" y que "el cierre de una feature no
puede exigirte tener el escritorio ordenado". Las dos frases son ciertas salvo
en el caso mas comun de todos: cerrar hacia la rama que tenes abierta.

DESPUES: el merge corre aislado SIEMPRE, este o no el destino checkouteado, asi
que un conflicto nunca deja el escritorio de Alan a medio mergear. Y en el unico
caso que no se puede resolver sin decidir por el —un archivo que el tiene
modificado y que el merge tambien toca— el arnes se da cuenta ANTES de commitear
nada, se niega, y le dice exactamente cuales son esos archivos y como seguir.
Falla temprano y con nombre propio, en vez de tarde y en idioma git.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                                   DESPUES
merge_en(principal, destino, rama)    merge_en(principal, destino, rama)
 |__ ¿destino == rama abierta?         |__ colisiones(principal, destino, rama)
 |     |__ SI  -> merge_aqui(PRINCIPAL)|     |__ ¿alguna? -> Err ANTES de nada,
 |     |          <-- tu checkout      |     |               con la lista y el
 |     |__ NO  -> worktree temporal    |     |               remedio
 |__ conflicto -> merge --abort        |__ worktree temporal --detach SIEMPRE
                                       |__ merge ahi (tu checkout nunca se toca)
el cierre ya commiteo el worktree      |__ sincronizar el principal:
antes de llegar aca                    |     |__ ¿tiene el destino abierto?
                                       |          SI -> git reset --keep <nuevo>
                                       |               (preserva tus cambios)
                                       |          NO -> update-ref con old-value
                                       |__ conflicto -> se limpia el temporal
```

La deteccion se corre desde `integrar`, ANTES de `commit_todo`, para que un
cierre que no va a poder integrar no deje un commit de cierre huerfano en la
rama.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como usuario parado en la rama destino con trabajo sin commitear que NO
  choca con el merge, quiero cerrar sin ordenar el escritorio, porque eso es lo
  que el arnes ya promete.
- P1: Como usuario cuyo trabajo sin commitear SI choca, quiero enterarme antes
  de que el cierre haga nada, con los archivos por nombre, para decidir yo.
- P2: Como lector del codigo, quiero que la cabecera de `git.rs` diga la verdad
  sin excepciones.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->

- AC-1: Given el destino es la rama abierta en el checkout principal, When corre
  el merge del cierre, Then se hace en un worktree temporal `--detach` y el
  checkout principal NUNCA queda en estado de merge (sin `MERGE_HEAD`, sin
  archivos con marcadores de conflicto).
  Comando: `cd rust && cargo test merge_en_la_rama_abierta_no_usa_el_checkout_principal`
- AC-2: Given trabajo sin commitear en archivos que el merge NO toca, When se
  cierra hacia la rama abierta, Then el merge se completa y esos cambios quedan
  intactos: el escritorio desordenado deja de ser un requisito.
  Comando: `cd rust && cargo test cierre_con_cambios_sin_commitear_que_no_chocan`
- AC-3: Given trabajo sin commitear en un archivo que el merge SI toca, When se
  cierra, Then el arnes se niega ANTES de commitear el worktree, nombrando cada
  archivo que colisiona, y el repo queda exactamente como estaba: sin merge
  commit, sin la rama movida y sin commit de cierre en la rama.
  Comando: `cd rust && cargo test colision_se_detecta_antes_de_tocar_nada`
- AC-4: Given esa negativa, When el usuario la lee, Then el mensaje dice el
  remedio concreto (commitear o `git stash`) y como retomar el cierre, sin texto
  crudo de git.
  Comando: `cd rust && cargo test mensaje_de_colision_nombra_archivos_y_remedio`
- AC-5: Given el destino NO esta checkouteado en el principal, When se integra,
  Then sigue funcionando como hasta hoy y la rama queda avanzada.
  Comando: `cd rust && cargo test merge_a_rama_no_checkouteada_sigue_funcionando`
- AC-6: Given un conflicto de merge de verdad (los dos lados tocaron lo mismo),
  When se integra, Then el merge falla, se limpia el worktree temporal y NI la
  rama destino NI el checkout principal quedan modificados (AC-18 de la #47
  sigue valiendo).
  Comando: `cd rust && cargo test conflicto_real_no_deja_nada_a_medias`
- AC-7: Given la deteccion de colisiones, When se lee el codigo, Then es una
  funcion que SOLO consulta (`git status --porcelain` + los archivos que el
  merge cambiaria) y devuelve la lista; no muta nada y se puede testear sin
  hacer el merge.
  Comando: `cd rust && cargo test colisiones_solo_consulta_y_no_muta`
- AC-8: Given la cabecera de `git.rs`, When se lee, Then la promesa "el merge no
  toca tu checkout" ya no tiene la excepcion silenciosa, y el caso irreductible
  esta escrito con su razon.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: `close --status done --to <rama>` cuando hay algo que integrar.
- lo que se consulta: los archivos sucios del checkout principal
  (`git status --porcelain`) y los archivos que el merge cambiaria respecto del
  destino; la interseccion es la colision.
- el aislamiento: un worktree temporal `--detach` en el commit del destino, que
  se borra siempre (salga bien o mal).
- la sincronizacion: `git reset --keep <nuevo>` cuando el principal tiene el
  destino abierto (mueve la rama y el arbol preservando lo local, y aborta
  atomicamente si no puede); `update-ref <nuevo> <viejo>` cuando no.
- candado: la deteccion corre antes de `commit_todo`, asi que un cierre que se
  va a negar no deja rastro en la rama.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO el cierre va a integrar <rama> en <destino>

  colisiones = sucios(principal) ∩ archivos_que_cambia(destino -> rama)
  ¿hay colisiones? -> negarse ACA, nombrarlas, decir el remedio.
                      (no se commitea el worktree, no se mergea, nada)

  temporal = worktree --detach en el commit de <destino>
  mergear ahi
  ¿conflicto? -> borrar el temporal y fallar; nada quedo a medias

  ¿el principal tiene <destino> abierto?
     SI -> reset --keep <nuevo>   (mueve rama + arbol, preserva lo tuyo)
     NO -> update-ref <destino> <nuevo> <viejo>

  ENTONCES el trabajo queda integrado sin que tu checkout haya
           participado del merge, y lo que tenias sin commitear sigue ahi.
```
Promesas: tu checkout nunca mergea · lo sucio que no choca no molesta · lo que
choca se dice antes y por nombre · nada queda a medias.

## No funcionales
- SLOs: un worktree temporal por cierre, borrado siempre; sin red.
- Seguridad: sin `--force` sobre ramas, sin rebase, sin squash, sin stash
  automatico. El arnes no toca el trabajo sin commitear del usuario: lo
  reporta y se detiene (Articulo 4 / decision USUARIO 2026-08-27).
- Observabilidad: el mensaje de negativa nombra cada archivo y el remedio.

## Fuera de alcance
- Guardar/restaurar los cambios del usuario (`git stash`) automaticamente o
  detras de un flag: decision del USUARIO 2026-08-27, el arnes no decide que
  gana entre su merge y tu trabajo.
- Cambiar la exigencia de `--to`, el mensaje de conservacion o la vuelta al PRD.
- Que el cierre sea transaccional de punta a punta (que `close` revierta el
  `status: done` si la integracion falla): es real, es otro bug, y no es este.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Que hacer ante la colision irreductible — DECIDIDO (USUARIO, 2026-08-27):
  negarse ANTES de tocar nada, con la lista exacta de archivos y el remedio.
  Sin flag de stash: son cambios del usuario y el arnes no elige por el.
- Por que no se avanza la rama dejando el checkout atras — MEDIDO
  (2026-08-27): tras `update-ref` con el arbol en el commit viejo, `git status`
  muestra `MM` y `git diff` muestra la REVERSION del merge; un commit distraido
  desharia el trabajo recien integrado. Descartado por peligroso.
- Que el `status: done` quede escrito aunque la integracion falle es una
  deuda REAL que este spec deja anotada y NO arregla (esta en Fuera de alcance).
