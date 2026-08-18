---
nombre: criterios-de-cierre-que-se-pueden-fallar
descripcion: Un criterio que no se puede fallar no verifica nada: solo tranquiliza.
triggers: [criterios de cierre, plan, reviewer, verificacion, ranking, heuristica, SLO, exit code, comando, verde falso, AC ejecutable]
relacionadas: [hitos-del-prd, probar-contra-datos-reales, promesas-estructurales-vs-disciplina]
origen: [20, 23]
usos: 1
ultimo_uso: 2026-08-17
ultima_actualizacion: 2026-08-17
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
