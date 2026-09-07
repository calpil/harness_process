---
nombre: criterios-de-cierre-que-se-pueden-fallar
descripcion: Un criterio que no se puede fallar no verifica nada: solo tranquiliza.
triggers: [criterios de cierre, precondicion, rojo falso, verde falso por etiqueta, IFS, efecto y no llamada, plan, reviewer, verificacion, ranking, heuristica, SLO, exit code, comando, verde falso, AC ejecutable, timeout, herramienta externa, portabilidad, macOS, skip, prueba del rojo, oraculo, test que acompaña, mutacion, invariante falso, recorrido, alcance, codigo inalcanzable, nombre del test, regla ancha, gate de mas, paralelismo, pieza de mas, lugar nuevo, ciclo de vida]
relacionadas: [hitos-del-prd, probar-contra-datos-reales, promesas-estructurales-vs-disciplina]
origen: [20, 23, 63, 73, 75, 76, 77, 78]
usos: 8
ultimo_uso: 2026-09-07
ultima_actualizacion: 2026-09-06
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

## Referencias (el detalle, caso por caso)

Movidas a `criterios-de-cierre-que-se-pueden-fallar/referencias/` el 2026-09-06 (feature #80). Cada una es el caso que sostiene una regla de arriba: se leen cuando hace falta el detalle, no para entender la clase.

- [El PIPE que se traga el exit code (feature #64)](criterios-de-cierre-que-se-pueden-fallar/referencias/el-pipe-que-se-traga-el-exit-code.md)
- [La herramienta externa que no esta convierte el test en un placebo](criterios-de-cierre-que-se-pueden-fallar/referencias/la-herramienta-externa-que-no-esta-convierte-el-test-en-un-p.md)
- [El oraculo copiado de la implementacion (feature #73)](criterios-de-cierre-que-se-pueden-fallar/referencias/el-oraculo-copiado-de-la-implementacion.md)
- [Caminar el recorrido P1 es la prueba de alcance (feature #75)](criterios-de-cierre-que-se-pueden-fallar/referencias/caminar-el-recorrido-p1-es-la-prueba-de-alcance.md)
- [El nombre del test tiene que describir el CUERPO, no el AC (feature #75)](criterios-de-cierre-que-se-pueden-fallar/referencias/el-nombre-del-test-tiene-que-describir-el-cuerpo-no-el-ac.md)
- [La regla mas ancha que la evidencia (feature #76)](criterios-de-cierre-que-se-pueden-fallar/referencias/la-regla-mas-ancha-que-la-evidencia.md)
- [La pieza mas grande que el problema (feature #77)](criterios-de-cierre-que-se-pueden-fallar/referencias/la-pieza-mas-grande-que-el-problema.md)
- [El arnes que prueba el rojo tambien miente, y de dos formas](criterios-de-cierre-que-se-pueden-fallar/referencias/el-arnes-que-prueba-el-rojo-tambien-miente-y-de-dos-formas.md)
- [El rojo que fallo antes de llegar al aserto (feature #78)](criterios-de-cierre-que-se-pueden-fallar/referencias/el-rojo-que-fallo-antes-de-llegar-al-aserto.md)
