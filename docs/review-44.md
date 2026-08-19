# Review - Feature #44: verify_detecta_filtro_vacio

## Veredicto

Los **17 AC** en verde en `docs/verify-44.md`, corridos con el binario de esta
feature. La suite completa, clippy con `-D warnings`, `setup_smoke`,
`parity_check`, `harness_check`, `conventions_check` y `deudas_check` limpios.

## Lo que esta feature demuestra sobre si misma

El unico falso verde que existia en el repo era el AC-12 de la #28. Las tres
corridas de abajo se ejecutaron de verdad, en este orden:

**1. Lo que decia el reporte commiteado** (`git show HEAD:docs/verify-28.md`):

```
| AC-12 | verde | `cd rust && cargo test consolidar_without_..._anything` | 0 | 79 |
```

**2. El contrafactico, medido.** Se renombro a proposito la funcion del test y
se corrio `verify --feature 28 --solo AC-12` con el binario de esta feature:

```
| AC-12 | vacio | `cd rust && cargo test consolidar_without_..._anything` | 0 | 175 |

0 verde(s), 0 en rojo, 0 manual(es), 1 sin casos.
Un AC `sin casos` corrio y salio 0, pero no ejecuto ningun test:
revisa que el nombre del filtro exista de verdad.
AC en rojo: AC-12
```

Despues se restauro el nombre y el reporte. Este paso se hizo porque la primera
version de este documento afirmaba ese resultado **sin haberlo corrido**, que es
exactamente lo que la leccion `probar-contra-datos-reales` prohibe.

**3. Con el test escrito de verdad**, el reporte final:

```
| AC-12 | verde | `cd rust && cargo test consolidar_without_..._anything` | 0 | 871 |
```

79 ms era el tiempo de no correr nada. 871 ms es el tiempo de correr el test.

### Un detalle que aparecio haciendo el experimento

Renombrar la funcion a `..._anything_RENOMBRADO_TEMPORAL` **no la saco del
filtro**: `cargo test` matchea por SUBCADENA, asi que el nombre viejo seguia
adentro del nuevo y el test corria igual. Hubo que renombrarla a algo sin el
prefijo. Vale anotarlo porque empeora el problema original: un `Comando:` puede
seguir verde por matchear un test que ni siquiera es el que nombra.

## Lo que se reviso con desconfianza

- **¿El detector puede poner en rojo trabajo sano?** El contrato de `None` es lo
  que lo evita, y esta fijado con cinco salidas reales que NO son de libtest
  (vacia, un `grep`, un `[Ok]` de los chequeos de shell, una linea de cargo,
  un `warning`). La suite completa (497 tests) y los chequeos de shell
  corrieron despues del cambio: cero regresiones.
- **¿El test de la deuda discrimina?** Tiene un paso de CONTROL: el mismo caso
  con `--aplicar` tiene que mover el arbol. Sin eso pasaria igual con un
  `consolidar` roto. Es el error que cometi en el AC-6 de la #37 y que un pase
  de refutacion tuvo que señalarme.
- **¿Se filtro el estado nuevo por algun consumidor?** Es la pregunta que dejo
  la #37. Los tres `match` sobre `Estado` los marco el compilador. El que NO
  podia marcar era `rojos_del_reporte`, que comparaba contra `"rojo"` y
  `"timeout"` a mano: se reescribio para derivar del enum, que es el arreglo
  estructural y no solo el parche.

## Lo que la refutacion encontro DESPUES de cerrar

Cinco defectos, y el primero **invalidaba la feature**: `ejecutar` recortaba la
salida a las ultimas 20 lineas ANTES de medirla, asi que cualquier ruido despues
del resumen de libtest apagaba el detector — y como stderr se pega al final,
`cargo test` compilando lo apagaba solo. El falso verde seguia vivo adentro de la
feature escrita para matarlo.

Los otros cuatro: el lector del reporte fallaba abierto ante un estado
desconocido (o sea que el AC-11 prometia algo que no daba), el test de la deuda
era ciego fuera de `docs/` y miraba el `bkp` en la ruta equivocada, su guarda
anti-tautologia matcheaba tambien el mensaje de descarte, y el chequeo de shell
le hablaba al hub PostgreSQL real (3:38 -> 1.3 s al aislarlo).

Los cinco estan arreglados y fijados con tests; el detalle esta en
`docs/impl-44.md`. Uno mas quedo registrado como feature #46: `verify` se cuelga
con salidas de mas de ~64 KB porque espera al proceso antes de leer los pipes
(`seq 1 400000` se reporta como timeout a los 10001 ms).

**Lo que esto dice de la feature**: los 17 AC estaban verdes y ninguno probaba el
caso que importaba, porque todos los comandos de prueba producian salidas cortas.
El detector se verifico con la forma de salida que YO elegi, no con la que el
mundo produce. Es la leccion `probar-contra-datos-reales` otra vez, un nivel mas
arriba: no alcanza con usar datos reales si elegis los faciles.

## Riesgo que queda vivo

Un AC cuyo comando corra la suite entera y quede con `0 passed` por una razon
legitima saldria `vacio`. No hay ninguno asi en el repo y el escape honesto ya
existe (no declarar `Comando:`), pero es el falso positivo que hay que mirar si
aparece.

Y el limite de fondo: esto detecta el AC que **no midio**, no el AC que mide
algo trivial. `Comando: true` sigue saliendo verde.
