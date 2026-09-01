---
nombre: criterios-de-cierre-que-se-pueden-fallar
descripcion: Un criterio que no se puede fallar no verifica nada: solo tranquiliza.
triggers: [criterios de cierre, plan, reviewer, verificacion, ranking, heuristica, SLO, exit code, comando, verde falso, AC ejecutable, timeout, herramienta externa, portabilidad, macOS, skip, prueba del rojo]
relacionadas: [hitos-del-prd, probar-contra-datos-reales, promesas-estructurales-vs-disciplina]
origen: [20, 23, 63]
usos: 1
ultimo_uso: 2026-08-17
ultima_actualizacion: 2026-08-27
estado: activa
---

## Cuando aplica

Cuando escribis la seccion **Criterios de cierre (reviewer)** de un plan, y sobre
todo cuando la feature produce algo **heuristico**: un ranking, un umbral, un
score, un mensaje que "tiene que ser claro", una performance que "tiene que ser
buena".

Sintoma de que lo escribiste mal: al cerrar, el criterio se marca cumplido
leyendolo, sin haber ejecutado nada. Si no existe una corrida que lo pueda poner
en rojo, no es un criterio: es una intencion.

## Procedimiento

1. Por cada cosa que la feature promete y **no** verifica un test unitario,
   escribi el criterio como **una corrida con un resultado esperado concreto**.
2. Elegi el caso que motivo la feature, no uno comodo. Si la historia del spec
   dice "Alan pregunta ¿donde decidimos usar ureq?", el criterio es
   *esa* consulta, con *ese* resultado esperado.
3. Para lo numerico, exigi **publicar el numero**, no adjetivos. "Responde
   rapido" no se puede fallar; "medir y publicar la latencia sobre el corpus
   real" si.
4. Para los efectos que se prometen por omision (no escribe, no llama a la red,
   no depende de X), exigi la **comprobacion negativa**: un comando que muestre
   que no paso.
5. Corre los criterios ANTES de escribir la evidencia. Si uno falla, ese es el
   valor de la feature, no un obstaculo.

Ejemplos de la reescritura, de decorativo a verificable:

| Decorativo | Verificable |
| --- | --- |
| "el ranking debe ser razonable" | "`buscar ureq` devuelve el ADR-0001 en el primer puesto" |
| "responde rapido" | "medir 5 corridas sobre el corpus real y publicar el numero" |
| "no deberia escribir nada" | "`find docs progress -newermt '-5 seconds' -type f` devuelve 0" |
| "degrada bien sin hub" | "el stdout con hub y sin hub es byte a byte identico" |

## Cuando el criterio ya es un comando

Automatizarlo no lo salva: **un comando tambien puede ser incapaz de fallar**, y
entonces el verde es peor que no tener nada, porque tranquiliza.

Los dos casos que aparecieron de verdad en la feature #23, corriendo la
verificacion sobre su propio spec:

| Comando | Por que no verifica |
| --- | --- |
| `cargo test nombre_que_no_existe` | Un filtro sin coincidencias **sale 0**. En la primera corrida, 8 de 20 AC dieron verde sin ejecutar un solo test |
| `... \| grep -c "patron" \|\| true` | `grep -c` devuelve 1 cuando cuenta 0, y el `\|\| true` se lo traga: sale 0 siempre |

Aplica a cualquier runner con filtro por nombre (`pytest -k`, `go test -run`,
`jest -t`, `dotnet test --filter`): todos salen 0 cuando el filtro no matchea.

Antes de aceptar un comando como verificacion, hacele **la prueba del rojo**:

1. Rompe a proposito lo que el comando deberia detectar (borra el test, invertí
   la condicion, sacale el texto al archivo).
2. Corre el comando. Si sigue en verde, no verifica: decora.
3. Restaura.

Y para los filtros por nombre, pedile al comando que **muestre cuanto corrio**:
`cargo test <nombre> 2>&1 | grep "N passed"` con N > 0 falla cuando el nombre
esta mal escrito; `cargo test <nombre>` a secas, no.

## Pitfalls

- **El criterio que solo se puede cumplir.** Es el pitfall que origino esta
  leccion: la feature #20 exigia que una consulta real devolviera el ADR primero,
  y **la primera corrida lo puso en el puesto 10**, debajo de un ejemplo de
  nombre malo sacado de una guia. Con un criterio del tipo "el ranking debe ser
  razonable", esa feature se cerraba rota y nadie se enteraba hasta usarla.
- **Verificar con la fixture en vez de con lo real.** Los tests con corpus
  sembrado pasaban perfecto; el bug solo aparecio contra los 114 archivos del
  repo. Una fixture prueba la mecanica, no la calibracion.
- **Escribir la evidencia antes de correr los criterios.** Invita a redactar
  alrededor de lo que salio en vez de a corregir lo que salio mal.
- **Confundir "no fallo" con "se verifico".** Si un criterio no tiene un comando
  y una salida esperada, no se verifico: se leyo.
- **Confiar en el exit code de un comando que nunca viste fallar.** Es la version
  automatizada del mismo error. Un exit 0 dice "el comando termino bien", no "lo
  que queriamos comprobar es cierto". La prueba del rojo cuesta dos minutos y es
  la unica evidencia de que el instrumento mide algo.

## Verificacion

```bash
# 1. Cada criterio del plan tiene que tener un comando al lado, y su salida
#    esperada. Si no la tiene, reescribilo antes de implementar.
grep -n "Criterios de cierre" -A 20 docs/plan-feature-<id>-*.md

# 2. Si los AC declaran `Comando:`, que cada uno haya corrido ALGO:
sh harness_cli verify --feature <id>
grep -n "Comando:" docs/spec-feature-<id>-*.md   # y leelos: ¿alguno no puede fallar?

# 3. La prueba del rojo, sobre el comando que mas confianza te da:
#    rompé lo que deberia detectar, corrélo, y solo entonces restaurá.
```

Regla practica: si no podes escribir la corrida que lo pondria en rojo, todavia
no es un criterio.

## El PIPE que se traga el exit code (feature #64)

El caso mas barato de "comando que no puede fallar", y el mas facil de escribir
sin darse cuenta:

```
Comando: `bash tests/setup_smoke.sh 2>&1 | tail -5`
```

`verify` ejecuta con `sh -c` y **sin `pipefail`** (`verificacion.rs:228`), asi
que el rc del pipeline es el de `tail`: **siempre 0**. Comprobado:
`sh -c 'false | tail -5'` sale 0. El smoke podia romperse entero y el AC seguia
verde. Dos AC de la #64 nacieron asi (`| tail -5` y `| tail -3`) y los encontro
el reviewer, no el autor.

El agravante es que el pipe se agrega por una razon buena —"que no me llene la
pantalla"— y el costo no se ve: el reporte queda igual de verde.

Regla corta: **en un `Comando:` el ultimo proceso del pipeline es el que decide
si el AC esta verde.** Si lo que te importa es el rc del PRIMERO, no uses pipe:
manda la salida a `/dev/null` (`cmd >/dev/null 2>&1`) o antepone
`set -o pipefail;`. Y `grep` como ultimo eslabon si sirve, porque `grep` falla
cuando no encuentra: `... | grep -E "[1-9][0-9]* passed"` es un buen criterio
justamente porque su rc significa algo.

Al corregir los dos comandos de la #64, uno de ellos **paso a rojo de
inmediato**: `harness_check.sh` fallaba y el `| tail -3` lo venia tapando. Ese
rojo era el valor de la correccion.

## El criterio que no se puede correr desde donde se implementa

Corolario del anterior, tambien de la #64. El AC-11 declaraba
`bash harness_check.sh`, y ese check **no puede pasar dentro de un worktree**:
su gate de espejo expande `__HREL__` con el basename del directorio —que en un
worktree es el de la feature, no `harness_process/`— y reporta divergencia falsa
en los tres roles; ademas `progress/` no existe ahi, asi que ve `current.md`
vacio. Cuatro problemas, ninguno real.

Un criterio que solo puede pasar en otro directorio no es un criterio: es una
trampa que invita a marcarlo MANUAL y seguir. Se reemplazo por un comando que
verifica **lo que el AC promete** (que ningun rol afirme lo que el arnes ya no
hace, y que los espejos coincidan bajo la expansion correcta), con su prueba del
rojo: sembrada la afirmacion falsa, rc=1.

## Un AC que nace de una hipotesis NO REPRODUCIDA arrastra el error hasta el final

Es el caso mas caro medido hasta ahora, y no lo produjo un bug: lo produjo un
hallazgo teorico que nadie cerro antes de convertirlo en criterio.

En la #66, una revision teorizo que `printf '%s' "$x" | grep -q ...` bajo
`set -o pipefail` podia devolver el EPIPE de `printf` y dar un falso negativo. Se
escribio el AC-11 sobre esa hipotesis. Lo que siguio:

| vuelta | que se hizo | que costo |
| --- | --- | --- |
| 0 | el codigo era `printf \| grep -q`. **Funcionaba.** | — |
| 1 | se midio la hipotesis: **no se reproduce** (200 KB, 1 MB, 8 MB; `rc=0` siempre) | — |
| 2 | se cambio igual "por robustez" a un `case *'"clave"'*true*` | **falso positivo**: el JSON real trae `cwd`, y un `/Users/alan/truenorth` ponia el flag en 1 con el JSON diciendo `false`. La primera vuelta dejo de bloquear |
| 3 | se arreglo recortando el prefijo (`${x#*"clave"}`) | **cuadratico en bash**: 200 KB = 20.5 s contra 0.032 s del `grep`; 1 MB no termino en 2 minutos, con un timeout de hook de 120 s |
| 4 | se volvio al `grep`, con here-string en vez de pipe | lo unico que valia del cambio |

Tres vueltas de revision adversarial para volver, casi exactamente, a donde
estaba. Y cada arreglo fue **consecuencia del anterior**: el falso positivo nacio
de arreglar un bug inexistente, y el cuadratico nacio de arreglar el falso
positivo.

## Procedimiento: cerrar la hipotesis ANTES de escribir el AC

1. **Reproducila primero.** Un hallazgo que dice "puede pasar X" no es un
   hallazgo hasta que hay una corrida que muestra X. Si no se reproduce, el
   resultado de la investigacion es *"no se reproduce"*, y eso se escribe — no se
   escribe un AC.
2. **Si no se reproduce, no toques el codigo.** "Ya que estoy, lo endurezco" es
   la frase que arranca la cadena. Codigo que funciona y no tiene bug demostrado
   se deja quieto: el riesgo de la edicion es real y el beneficio es hipotetico.
3. **Si igual hay que cambiarlo** (porque simplifica de verdad, no "por las
   dudas"), el reemplazo se prueba contra la MATRIZ del original, no solo contra
   el caso que motivo el cambio. El `case` nunca se probo contra un payload real
   con `cwd`; el `grep` lo manejaba bien desde siempre.
4. **Medi el costo, no solo la correccion.** El recorte de prefijo era correcto y
   640 veces mas lento. En un hook con timeout, "correcto pero lento" es
   incorrecto: **un hook que no termina es peor que uno que decide mal.**
5. **Y el AC se corrige, no se cumple a la fuerza.** Cuando la premisa cae, lo
   honesto es reescribir el criterio diciendo lo que se midio. Ver
   [[promesas-estructurales-vs-disciplina]] y
   [[reglas-que-se-aplican-a-si-mismas]].

Regla corta: **no se endurece codigo que funciona contra un bug que no se pudo
reproducir.** El bug hipotetico cuesta cero; el que introduce el arreglo, no.

## La herramienta externa que no esta convierte el test en un placebo

Un criterio puede nacer bien y volverse imposible de fallar **sin que nadie lo
toque**, cuando depende de una herramienta que en otra maquina no existe.

`tests/commit_guard_check.sh` decidia si un script se colgaba con `timeout 10`.
En Linux funciona. En macOS `timeout(1)` **no viene con el sistema**: el
subshell devuelve `127` ("no existe"), y el test solo consideraba colgado el
`124` ("se corto"). Resultado: el modo salia **verde pase lo que pase**, en la
maquina donde mas se corre.

El patron a reconocer: **traducir el codigo de salida de una herramienta sin
comprobar que la herramienta corrio**. `127` y `124` son dos cosas
completamente distintas y el test las metia en la misma bolsa ("no es 124,
entonces termino bien").

Procedimiento cuando una prueba depende de algo externo:

1. **Elegi entre varios**: `timeout`, `gtimeout`, `perl alarm`. Alguno hay.
2. **Si no hay ninguno, FALLA** nombrando cual instalar. Un skip verde es la
   forma mas cara de no enterarse: parece cobertura y no lo es.
3. **Proba el mecanismo elegido** contra un caso que debe cortar y uno que no.
   Sin eso, "ahora si mide" es otra afirmacion sin comprobar — el mismo error
   una capa mas arriba.
4. **No traduzcas codigos que no distinguen**: separa "la herramienta corto" de
   "la herramienta no estaba".

Y una advertencia sobre la prueba del rojo, que es la que salva esto: **tambien
se pudre**. La de este test reconstruia UNA de las dos defensas contra el
cuelgue (habia una por feature, la #52 y la #53), asi que el rojo dejo de
aparecer y su fallo se leyo como ruido de un test viejo durante semanas. Si tu
prueba del rojo empieza a fallar, la primera hipotesis no es "el test esta
viejo": es **"el instrumento dejo de medir"**.

## El arnes que prueba el rojo tambien miente, y de dos formas

Si la prueba del rojo es lo que salva a los criterios, hace falta decir como se
rompe **ella**. En la #67 se automatizo —mutar el codigo, correr el test, esperar
ROJO, revertir— y la primera corrida dio **seis falsos verdes seguidos**. Dos
causas distintas, las dos silenciosas.

**1. El filtro que no matchea nada corre cero tests y no imprime `FAILED`.**

`cargo test -- --exact los_parsers_no_discrepan` no corre nada: el nombre real es
`markdown::tests::los_parsers_no_discrepan`. Cero tests corridos, salida sin la
palabra `FAILED`, y un arnes que decide por `"FAILED" in salida` lo lee como
verde. Es el 127-vs-124 de mas arriba con otra ropa: **traducir la ausencia de
una señal de fallo en una señal de exito**, sin comprobar que la medicion ocurrio.

El arreglo no es corregir los nombres —eso arregla hoy y no manana— es **exigir
que el test haya corrido**: parsear `running N tests` y tratar `N == 0` como un
estado propio, distinto de verde y de rojo.

**2. Restaurar el archivo mutado deja a `cargo` con el binario mutado.**

`shutil.copy` no preserva la mtime, asi que el `.bak` nace con la hora de la
copia. Al restaurarlo, el archivo bueno queda con una mtime **anterior** a la del
build hecho sobre el codigo mutado: `cargo` compara mtimes, concluye que no hay
nada nuevo y **no recompila**. Las corridas siguientes usan el binario mutado.

Como se vio: un test empezo a fallar en la suite completa y a pasar aislado, y
despues a pasar en las dos en cuanto un edit cualquiera forzo el rebuild. La
tentacion ahi es archivarlo como flaky. No era flaky: era el arnes de mutacion
dejando el arbol de compilacion mintiendo, y el test que fallaba era exactamente
el de la ultima mutacion.

Reglas para un arnes de mutacion:

1. **Comproba que la mutacion cambio el archivo** (`cmp` contra el backup). Un
   ancla que ya no existe muta nada y el verde no significa nada.
2. **Comproba que el test corrio.** Cero tests no es verde.
3. **Toca las fuentes despues de restaurar**, o usa un `target/` aparte. Si no,
   lo que corre despues no es lo que dice el archivo.
4. **Un test que pasa aislado y falla en la suite** —o al reves— es primero una
   sospecha sobre el instrumento, no sobre el test.
