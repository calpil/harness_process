# Impl - Feature #75: el backlog no sabe de dependencias ni de features que se traban una y otra vez

Spec: docs/spec-feature-75-el-backlog-no-sabe-de-dependencias-ni-de-feature.md
Plan: docs/plan-feature-75-el-backlog-no-sabe-de-dependencias-ni-de-feature.md

## Lo que se midio antes de escribir el spec

| Premisa de la ficha | Medicion sobre el backlog real (2026-09-05) |
| --- | --- |
| Features que se traban una y otra vez | **84 cierres, CERO features cerradas `blocked` mas de una vez** |
| Los 6 `blocked` del historial | UN evento: #27 y #31-#35, en **52 segundos** del 2026-08-18, todos "Absorbida por la feature #36" |
| Dependencias que hayan dolido | Ninguna medida; el caso a favor es prospectivo |

Se le mostro al usuario antes de escribir los AC. **Decidio implementar los
siete, incluido el circuit breaker.** Eso obliga a algo concreto: un gate cuya
condicion nunca se disparo tiene sus tests como UNICA evidencia, asi que el AC-4
pide dos —uno que lo dispare y otro que compruebe que no se dispara antes.

## Alcance nuevo respecto de la ficha: `harness depende`

Implementando salio a la luz que `add --depends-on` **no cumple el recorrido P1
del propio spec**. El ejemplo que lo motiva es "Alan declara que la #21 depende
de la #17", y las dos ya existen: `add` crea una feature nueva, no edita una.
Con solo ese camino, la feature no servia para su caso de uso.

Y habia una segunda consecuencia: **la deteccion de ciclos del AC-5 seria codigo
inalcanzable**. Por `add`, una feature nueva solo puede depender de ids
anteriores, asi que el grafo es un DAG por construccion y el ciclo no puede
ocurrir. El AC-5 solo se podia satisfacer con tests unitarios.

Por eso se agrego `harness depende --feature N --de M [--quitar]`. Es peldano 3
de la escalera (comando nuevo) y la razon esta escrita: ningun comando existente
edita una feature.

## Evidencia por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/dependencias.rs:45 | `motivo_invalido` es PURA y comprueba, en orden, que los ids existan, que no haya auto-referencia y que no se forme ciclo. El orden importa: decir "ciclo" sobre un id que ni existe confundiria dos problemas. Test de comportamiento en rust/tests/cli_basics.rs que afirma que la feature NO se creo, no solo que el exit fue 2. |
| AC-2 | rust/src/commands/next.rs:19 | `next` saltea las que tienen dependencias abiertas y, si no ofrece ninguna POR ESE MOTIVO, lo dice nombrando quien espera a que. Un "no hay features pending" sobre un backlog lleno de pendings era justo el silencio a cerrar. |
| AC-3 | rust/src/commands/start.rs:289 | Avisa y arranca igual. El test comprueba las dos mitades: que el aviso salga y que `progress/current-2.md` exista despues. |
| AC-4 | rust/src/dependencias.rs:199 | El gate vive en la FASE 0 de `close` (rust/src/commands/close.rs:240), que es lo que puede negarse, y el contador en la FASE 3 (rust/src/commands/close.rs:300), con el resto del estado: contar antes haria que un cierre negado sumara igual. |
| AC-5 | rust/src/dependencias.rs:80 | `ciclo_que_formaria` devuelve el CAMINO completo (`#3 -> #1 -> #2 -> #3`), porque un mensaje que dice "hay un ciclo" sin decir cual obliga a buscarlo a mano. |
| AC-6 | rust/src/dependencias.rs:145 | El campo es opcional: sin `--depends-on` no se escribe, sin `blocked` el contador no nace, y `abiertas` devuelve vacio. Test que lo afirma sobre el JSON. |
| AC-7 | rust/src/commands/status.rs:73 | Las dependencias abiertas de las features en curso, en una linea aparte y no como columna: la mayoria no declara ninguna y una columna vacia en 75 filas es ruido. |
| AC-8 | rust/src/dependencias.rs:45 | Suite, clippy y paridad. |

## Una decision del dominio que vale explicar

`superseded` y `resuelto-aguas-arriba` **satisfacen** una dependencia; `blocked`
y `pending` no. El trabajo de una feature absorbida existe —en otra feature o en
otro repo— y esperar a algo que no va a cerrar nunca dejaria a la que depende
colgada para siempre. Una dependencia a un id que ya no esta en el backlog se
reporta abierta con estado `ausente`: desaparecer no es lo mismo que estar hecha.

## Las tres mutaciones

| Mutacion | Que cae |
| --- | --- |
| `abiertas` devuelve siempre vacio | `next_should_name_what_is_waiting_when_nothing_is_available` y `start_should_warn_about_open_dependencies_without_blocking` |
| el breaker no exige nunca | `close_should_demand_a_reason_after_repeated_blocks` |
| sin deteccion de ciclos | `depende_should_refuse_a_cycle_and_leave_the_backlog_untouched` |

## Un test mio que estaba mal, encontrado por la leccion de la #73

Escribi `add_should_refuse_a_dependency_cycle` y **no probaba un ciclo**: por
`add` el ciclo es inalcanzable, asi que el cuerpo terminaba comprobando una
cadena valida. El nombre afirmaba algo que el cuerpo no hacia — exactamente el
defecto que la #73 documento hace unas horas. Se reemplazo por
`depende_should_refuse_a_cycle_and_leave_the_backlog_untouched`, que si lo prueba
porque ahora el camino existe.

## El recorrido P1, caminado sobre el backlog real

No sobre un sandbox: sobre el `feature_list.json` de este repo, con las features
#17 y #21 que ya existian.

```
$ harness depende --feature 21 --de 17
Feature #21 declarada(s): depende de #17

$ harness depende --feature 17 --de 21
No se puede declarar esa dependencia para la feature #17: formaria un ciclo:
#17 -> #21 -> #17.
    El backlog no se toco.
```

El backlog se restauro despues (0 con `depends_on`, 0 con `bloqueos`): la prueba
no podia dejar declaraciones que el usuario no pidio.

## Lo que NO hace

- **No agrega el estado `review`** que proponia la idea 8 del catalogo: ya esta
  resuelto como GATE por la #64, y ponerlo tambien como estado serian dos
  respuestas a la misma pregunta.
- **No migra nada.** Las 75 features existentes no tienen el campo y se
  comportan igual.
- **No reusa el `depends_on` del grafo** (`graph/derive.rs`): aquel habla de
  relaciones entre piezas de codigo, este de features. Mismo nombre, dominios
  distintos.
