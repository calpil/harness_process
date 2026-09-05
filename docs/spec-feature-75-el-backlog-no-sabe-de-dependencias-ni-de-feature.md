# Spec - Feature #75: el backlog no sabe de dependencias ni de features que se traban una y otra vez

Estado: approved
Aprobado: 2026-09-05T13:51:09Z por USUARIO (confirmacion explicita) - Aprobado por Alan en chat; OBS-1 decidida: los siete AC incluido el circuit breaker, con la medicion a la vista
Plan: docs/plan-feature-75-el-backlog-no-sabe-de-dependencias-ni-de-feature.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md

## La historia (antes -> despues)

ANTES: el programa de aprendizaje del arnes (#17 a #22) estaba, textualmente,
"ordenado por dependencia: A y D son la base". Ese orden vivia en un documento
en prosa. El backlog no lo sabia: `next` ofrecia la primera pending, y si alguien
arrancaba la #21 antes que la #17 el arnes no tenia nada que decir.

DESPUES: una feature declara de que otras depende, `next` no la ofrece hasta que
esas esten cerradas, y `start` avisa si la arrancas igual — sin bloquearte,
porque la decision sigue siendo tuya. Y una feature que se traba repetidamente
deja de trabarse en silencio.

## Lo que se midio antes de escribir esto

Sobre el backlog real, el 2026-09-05:

| Premisa de la ficha | Medicion |
| --- | --- |
| Features que se traban una y otra vez | **84 cierres registrados, CERO features cerradas `blocked` mas de una vez** |
| Los 6 cierres `blocked` del historial | Son UN evento: #27 y #31-#35, en 52 segundos del 2026-08-18, todos con la nota "Absorbida por la feature #36". Una reclasificacion en bloque, no seis features trabadas |
| `depends_on` en el backlog | 0 features lo tienen (el campo no existe) |
| Dependencias que dolieron | Ninguna medida. El caso a favor es prospectivo, no retrospectivo |

**Decision del usuario (2026-09-05), con esta medicion a la vista: se
implementan los siete AC, incluido el circuit breaker.** Queda registrado que su
condicion nunca se disparo en 84 cierres.

Eso obliga a algo: un gate cuya condicion nunca ocurrio es exactamente el que
despues nadie sabe si funciona. Por eso el AC-4 no se conforma con que el codigo
exista — exige un test que **dispare** el breaker de verdad y otro que
demuestre que NO se dispara antes de tiempo.

## Alcance: lo que esta feature NO hace

El catalogo original (idea 8 de `docs/analisis-hermes-agent.md`) proponia
tambien un ESTADO `review`. **No se agrega.** Ya esta resuelto por otro camino y
mejor: `require_review` + el sello que escribe `revision --veredicto` (feature
#64). Un estado se pone a mano; un sello lo escribe el binario. Reimplementarlo
como estado seria dos respuestas a la misma pregunta, que es la familia de bug
mas repetida de este repo.

## Hoy -> Como va a funcionar

```
HOY:     add        -> feature suelta
         next       -> la primera pending, sepa o no de que depende
         close blocked, otra vez, y otra -> nadie cuenta

DESPUES: add/edit   -> depends_on: [id, ...]  (validado contra el backlog)
         next       -> saltea las que tienen dependencias abiertas, y lo DICE
         start      -> avisa si arrancas con dependencias abiertas; no bloquea
         close blocked N-esima vez -> exige que digas si es la misma causa
```

## Recorridos de usuario (priorizados)

- P1: Alan declara que la #21 depende de la #17 y `next` deja de ofrecerle la
  #21 hasta que la #17 cierre.
- P1: Alan arranca igual una feature con dependencias abiertas; el arnes se lo
  dice y lo deja seguir.
- P2: Alan cierra una feature como `blocked` por tercera vez y el arnes le exige
  decir si la causa es la misma.
- P2: Las 75 features que ya existen, sin el campo, se comportan igual que hoy.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una feature, When se le declara `depends_on: [<id>, ...]`, Then el
  campo se valida contra el backlog al escribirse: un id inexistente se rechaza
  con un mensaje que lo nombra y NO se guarda nada. La validacion es una funcion
  PURA sobre el backlog, testeable sin filesystem.
  Comando: `cd rust && cargo test --locked depend`
- AC-2: Given una feature cuyas dependencias no estan todas en un estado
  terminal (`done`, `superseded`, `resuelto-aguas-arriba`), When se corre `next`,
  Then esa feature NO se ofrece; y si `next` no ofrece ninguna POR ESE MOTIVO, lo
  dice nombrando que feature esta esperando y a que.
- AC-3: Given una feature con dependencias abiertas, When se la arranca con
  `start`, Then el arnes AVISA nombrandolas y arranca igual. No bloquea: la
  decision es del usuario, pero no puede ser silenciosa.
- AC-4: Given una feature que ya se cerro como `blocked` N veces
  (`rules.bloqueos_antes_de_decidir`, default 2), When se la vuelve a cerrar
  como `blocked`, Then el cierre EXIGE una nota que diga si la causa es la misma
  y se niega sin ella. Dos tests, porque la condicion nunca se disparo en 84
  cierres reales y un gate que nadie vio funcionar no esta verificado: uno que
  cierra `blocked` las N+1 veces y comprueba que el gate BLOQUEA, y otro que
  cierra N veces y comprueba que NO bloquea todavia.
  Comando: `cd rust && cargo test --locked --test cli_basics blocked`
- AC-5: Given un `depends_on` que formaria un ciclo (directo o transitivo),
  When se intenta escribir, Then se rechaza antes de tocar el backlog, con el
  ciclo nombrado.
- AC-6: Given las 75 features que ya existen sin el campo, When se corre
  cualquier comando, Then se comportan EXACTAMENTE como hoy. El campo es
  opcional y su ausencia no cambia nada: se prueba sobre el backlog real.
- AC-7: Given features activas con dependencias abiertas, When se corre
  `status`, Then se ven, junto a la feature que espera.
- AC-8: Given el cambio completo, When se corre la suite, Then quedan verdes los
  tests, clippy, el smoke del instalador y el gate de paridad.
  Comando: `cd rust && cargo test --locked`
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`
  Comando: `bash tests/parity_check.sh`

## Los datos que se tocan

- `feature_list.json`: campo OPCIONAL `depends_on: [<id>, ...]` por feature, y
  `bloqueos: <n>` que cuenta los cierres `blocked` de esa feature.
- `rules.bloqueos_antes_de_decidir`: umbral, default 2. Como las otras reglas,
  nace con un default que no rompe instalaciones existentes.
- Ningun campo existente cambia de forma ni de significado.

## Pseudo-codigo (el acuerdo)

```
AL DECLARAR: validar ids contra el backlog -> detectar ciclos -> recien escribir
NEXT:        saltear las que tienen dependencias abiertas
             si no queda ninguna POR ESO, decirlo con nombres
START:       avisar las dependencias abiertas y seguir
CLOSE blocked: contar. Pasado el umbral, exigir la nota de causa
```

Promesas: un `depends_on` invalido no se guarda; un ciclo no se guarda; una
feature sin el campo se comporta como siempre; `start` avisa pero nunca bloquea.

## No funcionales y verificacion

- Verificacion: funciones PURAS para validar y detectar ciclos (testeables sin
  filesystem) y tests de comportamiento sobre el binario para `next`, `start`,
  `close` y `status`.
- Prueba del rojo: cada test nuevo tiene que fallar contra el codigo actual.
- Compatibilidad: se prueba sobre el `feature_list.json` REAL, no sobre uno de
  fixture, que ninguna de las 75 features cambie de comportamiento.
- Riesgo declarado: el AC-4 implementa un gate cuya condicion no se observo
  nunca. Se acepta por decision del usuario; el precio es que su unica evidencia
  de que funciona van a ser sus dos tests.

## Alcance de instalacion y fuera de alcance

Se corrige `harness_process`. No se distribuye a otros proyectos. No se agrega
el estado `review` (ya resuelto como gate por la #64). No se migran las features
existentes: el campo es opcional y nace ausente.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por el usuario 2026-09-05): se implementan los siete AC de la
  ficha, incluido el circuit breaker, con la medicion de arriba a la vista.
- OBS-2: la rama de integracion se pregunta antes de `close --status done --to`.
