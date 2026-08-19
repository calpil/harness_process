# Impl - Feature #44: verify_detecta_filtro_vacio

## Que se hizo

`verify` mira la **salida** de un comando que sale 0, no solo su exit code. Si
reconoce el formato de libtest y la suma de `passed` es cero, el AC queda en
`vacio`, que bloquea el cierre y se cuenta aparte en el resumen.

Seis piezas, todas dentro de modulos que ya existian (peldano 1):

| Pieza | Donde | Que hace |
| --- | --- | --- |
| `casos_corridos` | `verificacion.rs` | Puro. Suma los `N passed` de las lineas `test result:`. `None` si no hay ninguna. |
| `Estado::Vacio` | `verificacion.rs` | Variante nueva; `bloquea()` la incluye. |
| `ESTADOS` + `desde_etiqueta` | `verificacion.rs` | La vuelta de `etiqueta()`, para que el lector del reporte salga del enum. |
| `ejecutar` | `verificacion.rs` | El camino feliz deja de tirar la salida y la clasifica. |
| `render_reporte` | `verificacion.rs` | "N sin casos" aparte de "N en rojo". |
| resumen de consola | `commands/verify.rs` | Igual que el reporte, mas la linea que dice que revisar. |

## El detector mira la salida, no el comando

La version facil era `comando.contains("cargo test")`. Se descarto por dos
razones concretas: un `cargo test` adentro de un script de shell quedaria
afuera (y varios AC del repo son `bash tests/*.sh`), y un comando que se llama
"test" sin serlo entraria. La forma de la salida es el dato; el texto del
comando es una pista.

De ahi sale el contrato de `None`: **no opinar es parte de la funcion**. Si la
salida no tiene lineas `test result:` —un `grep`, un `bash`, un compilador— el
estado no cambia. Eso es lo unico que evita que esta feature ponga en rojo
trabajo sano.

## Lo que se decidio contar como "sin evidencia"

- Un filtro que no matchea: obvio, es el caso que la origino.
- **Todos los tests `ignored`**: tambien. `0 passed; 4 ignored` no verifico
  nada, y un AC no puede apoyarse en un test que no corre.
- Varios binarios donde UNO corrio: eso SI es evidencia. `cargo test <nombre>`
  corre todos los binarios de test y casi siempre matchea en uno solo; sumar en
  vez de exigir que todos corran es lo que evita el falso positivo mas comun.

## El lector del reporte dejo de comparar cadenas

`rojos_del_reporte` decia `estado == "rojo" || estado == "timeout"`. Es
**exactamente** la forma del defecto que la feature #37 encontro en el emisor de
Jira: un consumidor que compara por igualdad no se entera cuando aparece un
valor nuevo, y el compilador no puede ayudar.

Ahora sale de `Estado::desde_etiqueta(...)` y pregunta `bloquea()`. El invariante
que lo sostiene es un round-trip sobre `ESTADOS`, no una lista escrita a mano:
un estado sexto que se olvide de cerrar el circuito rompe ese test.

## La deuda que esto destapo, pagada

El AC-12 de la feature #28 declaraba
`consolidar_without_aplicar_should_not_touch_anything`. Esa funcion no existia:

```
$ cd rust && cargo test consolidar_without_aplicar_should_not_touch_anything
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 322 filtered out
```

Exit 0. El invariante mas citado de ese comando —el que el README, el help de
clap y el comentario del modulo prometen— estaba registrado como verificado con
nada detras.

Se escribio de verdad, y con dos cuidados que la #37 enseño por las malas:

1. **Backend falso que SI propone.** Con un backend mudo el test seria
   tautologico: no se escribe nada porque no hay nada que escribir. El falso
   devuelve un candidato valido, y el test exige ver ese candidato en la salida.
2. **Un paso de CONTROL.** El mismo caso con `--aplicar` tiene que mover el
   arbol. Sin eso, el test pasaria igual si `consolidar` estuviera roto y no
   hiciera nada nunca — que es, palabra por palabra, como el AC-12 llego a estar
   verde sin existir.

De paso no gasta cuota: es el mismo backend falso que el backlog #43 pide para
`tests/consolidar_check.sh`.

## Auditoria: ¿habia mas falsos verdes?

Se cruzaron los **99** nombres de test declarados en todos los `spec-feature-*`
y `verify-*` contra las funciones que existen en `rust/`. Dos no matchean por
nombre exacto y solo uno es real:

- `consolidar_without_aplicar_should_not_touch_anything` (#28) — el falso verde.
- `verificacion` (#23) — es un filtro de MODULO, matchea
  `verificacion::tests::*`. Verificado corriendolo: corre casos.

## Cinco defectos que encontro la revision adversarial, DESPUES de cerrar

Los 17 AC estaban verdes y la feature cerrada cuando un pase de refutacion de
tres lentes encontro esto. **El primero invalidaba la feature entera.**

### 1. El detector medía sobre la salida ya recortada (BLOQUEANTE)

`ejecutar` hacia `let salida = leer_salida(...)`, y `leer_salida` terminaba en
`recortar_salida(&texto)` —las **ultimas 20 lineas**—. Recien despues llamaba a
`casos_corridos(&salida)`.

O sea: **el detector opinaba sobre una copia con perdida del dato**. Cualquier
comando que imprima ~17 lineas despues del resumen de libtest lo empuja fuera de
la ventana, `casos_corridos` no encuentra ninguna linea `test result:`, devuelve
`None` y el AC vuelve a salir **verde**.

Y no es un caso raro: `leer_salida` pega **stderr despues de stdout**, y `cargo
test` manda por stderr los diagnosticos de compilacion. En un repo con warnings,
o con el target frio, el falso verde de la #28 seguia vivo — dentro de la
feature escrita para matarlo.

Reproducido antes de tocar nada, con un test que fallo como corresponde:

```
assertion `left == right` failed: el resumen de libtest quedo fuera de las
ultimas 20 lineas y el detector se apago solo
  left: Verde
 right: Vacio
```

Arreglo: se **mide sobre la salida completa** y se **recorta solo para el
reporte**. `leer_salida` devuelve el texto entero; el recorte pasa a `ejecutar`.

### 2. El lector del reporte fallaba ABIERTO

El AC-11 prometia que "agregar un estado sexto no puede pasar de largo por el
cierre". No era cierto: `ESTADOS` es un array escrito **a mano**, y
`rojos_del_reporte` descartaba en silencio (`filter_map` + `?`) cualquier
etiqueta que no reconociera. Un agente lo demostro agregando una variante
`Sospechoso` con `bloquea() == true` sin tocar `ESTADOS`: compila, pasa la suite
entera, y el cierre no la ve.

Arreglo: el lector **falla cerrado** — una etiqueta desconocida bloquea. La
garantia ya no depende de que alguien se acuerde de tocar un array.

### 3 y 4. El test de la deuda tenia dos agujeros

- **Era ciego fuera de `docs/`**, y peor: la assert del backup miraba
  `<raiz>/bkp`, una ruta que **nunca existe** en el layout subdir. Los backups
  van a `<raiz>/hp/bkp` (usan `paths.root`). Se demostro con una mutacion que
  respalda antes de informar: el test pasaba igual.
- **La guarda anti-tautologia no guardaba nada.** `.stdout(contains("una-cosa"))`
  matchea tambien el mensaje de **descarte**, que nombra la leccion. Con `validar`
  roto —todos los candidatos descartados— no hay nada que fusionar, no se escribe
  nada, y el test pasaba por vacuidad. Ahora exige
  `"1 candidato(s) a consolidar"`, que solo se imprime con candidatos vivos.

### 5. El chequeo de shell le hablaba al hub REAL

`tests/verify_vacio_check.sh` no aislaba `HARNESS_HUB` ni las `DB_*`, asi que sus
sandboxes escribian en el Memory Hub PostgreSQL de la maquina. Medido: **3:38**
de corrida con `connection reset by peer` y `statement timeout`. Con el hub
aislado, **1.3 s**.

### Y uno mas, que se registro como feature #46

`ejecutar` llama `wait_timeout` **antes** de leer los pipes, asi que un comando
que imprime mas que el buffer (~64 KB) se bloquea escribiendo y nunca termina.
Verificado: `seq 1 400000` —que tarda milisegundos— se reporta como
**`timeout` a los 10001 ms**. Es anterior a esta feature, pero importa mas desde
que el veredicto depende de leer la salida entera.

## Limites declarados

- **Solo libtest.** nextest, pytest y jest imprimen otra cosa y el detector
  calla ante ellos. Se agregan cuando aparezcan.
- **No detecta el otro tipo de AC decorativo**: el que siempre puede salir bien
  (`true`, `... || true`, un `grep` sobre algo que siempre existe). Es un
  problema distinto y mas dificil; UPDATING ya lo advierte desde la #23.
- **La salida grande sigue siendo un problema abierto** (feature #46): un AC que
  imprima mas de ~64 KB se reporta como timeout aunque haya terminado.
- **Un AC que corre la suite entera con todo filtrado** por una razon legitima
  quedaria en `vacio`. No hay ninguno asi en el repo, y el escape honesto —no
  declarar `Comando:` y dejarlo `manual`— ya existe.
