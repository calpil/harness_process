# Spec - Feature #73: verify corre UN comando por AC y no lo dice

Estado: approved
Aprobado: 2026-09-05T12:01:15Z por USUARIO (confirmacion explicita) - Aprobado por Alan en chat, con OBS-1 decidida: se corren todos los comandos
Plan: docs/plan-feature-73-verify-corre-un-comando-por-ac-y-no-lo-dice-un-a.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md

## La historia (antes -> despues)

ANTES: Alan escribe el AC-8 de la #72 con las cuatro verificaciones que ese
criterio necesita —tests, clippy, el smoke del instalador y el gate de
paridad—, una debajo de la otra. Corre `harness verify --feature 72` y lee:

    Verificando 1 AC con comando declarado (10 en total)
    AC-8  $ cd rust && cargo test --locked
           [ok] verde (155505 ms)
    1 verde(s), 0 en rojo, 9 manual(es).

El AC quedo verde. Tres de sus cuatro comandos no se corrieron y nada lo dijo.

DESPUES: los cuatro se corren y los cuatro aparecen en el reporte. Un AC con
varias verificaciones no puede quedar verde por la primera.

## Lo que se midio antes de escribir esto

Sobre el corpus REAL —63 specs de este repo, con la gramatica de `parsear`—
hay **un solo** AC con mas de un `Comando:`: el AC-8 de la #72. O sea que esto
no viene rompiendo nada en silencio desde hace meses: es una trampa que se
disparo la primera vez que alguien la piso, y que va a volver a dispararse
porque escribir cuatro comandos debajo de un criterio es lo natural.

La causa es una linea de `rust/src/verificacion.rs`: el `Comando:` se le cuelga
al ultimo AC abierto **solo si `ultimo.comando.is_none()`**. El segundo y los
siguientes se descartan sin marca. El modelo de datos lo hace inevitable:
`Verificacion { ac, comando: Option<String> }` no tiene donde poner el segundo.

## Objetivos y no objetivos

- O-1: Un AC verifica con TODOS los comandos que declara.
- O-2: El reporte muestra cada comando con su estado propio.
- O-3: Ningun AC puede quedar verde con una verificacion sin correr.
- NO-1: No se cambia la sintaxis del spec: sigue siendo una linea `Comando:`
  por verificacion, debajo del criterio.
- NO-2: No se migran los reportes `docs/verify-*.md` ya escritos.

## Hoy -> Como va a funcionar

```
HOY:     - AC-8: ...        DESPUES: - AC-8: ...
           Comando: A   -> corre A            Comando: A   -> corre A
           Comando: B   -> DESCARTADO         Comando: B   -> corre B
           Comando: C   -> DESCARTADO         Comando: C   -> corre C
         reporte: 1 fila, verde             reporte: 3 filas, una por comando
```

## Recorridos de usuario (priorizados)

- P1: Alan escribe un AC con tres comandos y `verify` corre los tres.
- P1: Si uno de los tres falla, el AC no queda verde y el cierre se bloquea.
- P2: Un AC con un solo comando se comporta exactamente igual que antes.
- P2: Un AC sin comandos sigue siendo MANUAL y no bloquea.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un AC con N lineas `Comando:`, When corre `verify`, Then se
  ejecutan las N, en el orden en que estan escritas. Un test toma un spec con
  tres comandos en el mismo AC y afirma que los tres se ejecutaron.
  Comando: `cd rust && cargo test --locked verificacion`
- AC-2: Given ese mismo AC, When se escribe `docs/verify-<id>.md`, Then hay una
  fila por COMANDO, cada una con su estado, su exit code y su duracion; el AC
  aparece en todas y se distingue cual comando es cada fila.
- AC-3: Given un AC con tres comandos donde el segundo falla, When corre
  `verify`, Then el resumen cuenta ese AC como rojo, el exit code es 2 y
  `rojos_del_reporte` lo nombra UNA sola vez aunque falle mas de un comando.
- AC-4: Given un AC con un unico `Comando:`, When corre `verify`, Then el
  resultado es identico al de antes de esta feature: misma fila, mismo estado,
  mismo resumen. La compatibilidad se prueba sobre los AC reales del repo.
  Comando: `cd rust && cargo test --locked --test cli_basics verify`
- AC-5: Given un AC sin ningun `Comando:`, When corre `verify`, Then sigue
  siendo `manual` y no bloquea el cierre.
- AC-6: Given una linea `Comando:` que pertenece a un AC ilegible, When corre
  `parsear`, Then se sigue descartando y NO se le cuelga al AC de arriba: la
  guarda de la feature #68 no se puede perder al ampliar la gramatica.
- AC-7: Given los reportes `docs/verify-*.md` ya escritos con una fila por AC,
  When el gate del cierre los lee, Then los sigue interpretando igual: la
  ampliacion no invalida lo ya emitido.
- AC-8: Given el cambio completo, When se corre la suite, Then quedan verdes
  los tests, clippy, el smoke del instalador y el gate de paridad.
  Comando: `cd rust && cargo test --locked`
- AC-9 (MANUAL): Given el AC-8 del spec de la feature #72 —el que disparo
  esto—, When se lo verifica con el binario nuevo, Then se corren sus cuatro
  comandos y el reporte muestra las cuatro filas.

## Los datos que se tocan

- `Verificacion { ac, comando: Option<String> }` pasa a llevar la LISTA de
  comandos. Vacia = manual, que es lo que hoy significa `None`.
- `Resultado` deja de ser uno por AC y pasa a ser uno por comando ejecutado.
- El reporte `docs/verify-<id>.md`: mismas columnas, mas filas.
- El JSON de `verify --json`: mismas claves por fila.

## Pseudo-codigo (el acuerdo)

```
PARSEAR: por cada AC -> juntar TODAS sus lineas Comando:, en orden
         una linea Comando: de un AC ilegible se descarta (guarda #68)
VERIFY:  por cada AC -> por cada comando -> ejecutar y guardar un resultado
         AC sin comandos -> un resultado manual, como hoy
REPORTE: una fila por resultado
GATE:    cualquier fila que bloquea, bloquea el cierre
```

Promesas: ningun comando declarado se descarta en silencio; un AC no queda
verde con una verificacion sin correr; un AC de un solo comando no cambia.

## No funcionales y verificacion

- Verificacion: tests sobre la funcion PURA `parsear` (que ya se corre contra
  los 310+ AC reales del repo) y de comportamiento sobre el binario, con un
  spec de fixture que declara tres comandos.
- Prueba del rojo: cada test nuevo tiene que fallar contra el codigo actual.
- Compatibilidad: el corpus real tiene 62 specs con un comando por AC y uno con
  cuatro; los 62 no pueden cambiar de comportamiento.
- Limite conocido: esto no acota cuanto tarda un AC. Un AC con cuatro comandos
  tarda la suma de los cuatro, y cada uno tiene su propio timeout.

## Alcance de instalacion y fuera de alcance

Se corrige `harness_process`. No se distribuyen cambios a otros proyectos. No
se reescriben los reportes ya emitidos ni los specs ya escritos.

## Observaciones (decisiones pendientes)

- OBS-1: Que hacer con los comandos de mas. La propuesta es CORRERLOS TODOS
  (una fila por comando). La alternativa que la ficha dejaba abierta era
  conservar un comando por AC y solo AVISAR que hay mas: es menos codigo, pero
  deja al autor partiendo el AC a mano, que es disciplina y no estructura.
  DECIDIDA por el usuario el 2026-09-05: correrlos todos.
- OBS-2: La rama de integracion se pregunta antes de `close --status done --to`.
