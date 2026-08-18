---
nombre: probar-contra-datos-reales
descripcion: Verde no dice que este bien: dice que midio lo que sabias medir.
triggers: [fixtures, ranking, umbral, reporte, falso positivo, calibracion, datos reales, diagnostico, ok falso, alcance, health check]
relacionadas: [criterios-de-cierre-que-se-pueden-fallar, promesas-estructurales-vs-disciplina, reglas-que-se-aplican-a-si-mismas]
origen: [22, 25, 30, 36, 29, 28]
usos: 0
ultimo_uso:
ultima_actualizacion: 2026-08-17
estado: activa
---

## Cuando aplica

Cuando la feature produce algo **calibrado** en vez de algo binario:

- un ranking (que sale primero)
- un umbral (a partir de cuando avisa)
- un reporte de problemas (que cuenta como problema)
- un resumen o una agregacion

Los tests con fixtures prueban la **mecanica** ("ordena por score", "detecta el
caso X"). No prueban la **calibracion** ("el orden es util", "lo que reporta vale
la pena leerlo"), porque la fixture la escribiste vos con los casos que ya tenias
en la cabeza.

## Procedimiento

1. Escribi los tests con fixtures como siempre: son los que evitan regresiones.
2. **Ademas**, poné en los criterios de cierre del plan una corrida contra el
   repo real, con un resultado esperado concreto. Ver
   [[criterios-de-cierre-que-se-pueden-fallar]].
3. Corré esa verificación **antes** de escribir la evidencia.
4. Cuando algo salga mal, arreglalo **y dejalo con test**, con el hallazgo escrito
   adentro del test para que nadie lo "simplifique" despues.
5. Para los reportes de problemas, verificá **a mano** que cada uno reportado sea
   real. Un falso positivo cuesta mas que un falso negativo: el primero hace que
   se ignore la herramienta entera.

## El otro lado: el OK que dice de mas

Casi todo lo de arriba es sobre la falsa alarma. La falsa **calma** es peor, y
mas dificil de ver: nadie investiga un `[ok]`.

Pasa cuando el chequeo mide algo **adyacente** a lo que importa y reporta como si
hubiera medido lo que importa:

| Se midio | Se reporto | Lo que el lector entendio |
| --- | --- | --- |
| el TCP conecta | "hub alcanzable" | "el hub anda" — y las operaciones morian con `connection reset` |
| el comando salio 0 | "AC verificado" | "el test corrio" — y el filtro no matcheaba ningun test |
| el archivo existe | "configurado" | "esta bien configurado" |

Procedimiento:

1. Por cada chequeo, escribi en una linea **que mide exactamente**. Si esa frase
   es mas angosta que el nombre del chequeo, el nombre miente.
2. Poné esa frase **en la salida**, no en un comentario del codigo. El que lee el
   `[ok]` es quien necesita saber el alcance.
3. Nombra el sintoma que indicaria que el problema esta **mas adentro**: "si un
   comando falla con X, esto no lo cubre". Convierte un OK ciego en un OK que
   orienta.
4. Si el chequeo barato y el caro miden cosas distintas, decidí con el costo a la
   vista y **declaralo**: un diagnostico de 10 segundos es un diagnostico que
   nadie corre; uno de 2 que dice lo que no cubre, sirve.

## Y las razones que escribis tambien son datos

Una excepcion documentada con su razon parece rigor. Pero la razon **tambien hay
que verificarla**, y es facil que no: se escribe de memoria, suena plausible, y
queda citada como cierta durante meses.

En la #30 se declararon cinco asimetrias entre los dos instaladores, cada una con
su razon. Al verificarlas contra el codigo, **dos estaban mal**:

| Razon escrita | Lo que decia el codigo |
| --- | --- |
| "`--with-postgres` es la afirmativa de un default encendido" | es un **no-op**: `--with-postgres) ;;` |
| "`-CargoTargetDir` existe porque rustup no actualiza el PATH" | setea `CARGO_TARGET_DIR`, que no es el PATH |

Procedimiento: por cada excepcion, abri el archivo y confirmá la razon **antes**
de escribirla. Si la razon no se puede señalar con un numero de linea, todavia no
es una razon: es una corazonada prolija.

## El chequeo que muere antes de hablar

"No reporto nada" tiene dos causas posibles y se ven identicas: **no encontro
nada** o **no llego a mirar**. Hay que poder distinguirlas.

En la #36, el chequeo de convenciones dejo de reportar al ampliarle el alcance.
No porque no encontrara: porque moria antes. La causa, en shell:

```bash
set -Eeuo pipefail
nombre="$(head -n 5 "$f" | grep -E '^fn ' | sed ...)"   # grep sin match -> 1
                                                        # pipefail -> la sustitucion falla
                                                        # set -e -> el script entero muere
```

Y llevaba dos features asi: en el directorio original los `fn` estaban al tope y
el `grep` siempre encontraba algo, asi que el bug estaba latente y nadie podia
verlo. En shell con `pipefail`, **todo `grep`/`find` que legitimamente puede no
encontrar nada necesita `|| true`**, y el comentario que diga por que, para que
nadie lo "limpie" despues.

Procedimiento: por cada chequeo que escribas, corrélo una vez en el caso donde
**no** hay nada que encontrar. Si en vez de decir "nada que reportar" se queda
callado, no esta pasando: esta muriendo.

## La forma que el uso real produce, y tu fixture no

Un test unitario elige la forma del dato. El uso real la produce. Cuando eligiste
la comoda, el test pasa y la funcion esta rota.

Caso de la #29: la idempotencia de un reemplazo se decidia asi:

```rust
if !texto.contains(antes) && texto.contains(despues) { /* ya aplicado */ }
```

El test usaba `antes = "pendiente"`, `despues = "ya escrito"`. Correcto y verde.
Pero la forma que el uso real produce en el PRIMER intento es
**"insertar antes de esta linea"**, donde el `despues` CONTIENE al `antes`:

```
antes:   - `progress.rs`: estado vivo
despues: - `doctor.rs`: diagnostico
         - `progress.rs`: estado vivo      <- el antes sigue ahi
```

Tras aplicar, el `antes` sigue presente, el bloque no se reconoce como aplicado,
y la segunda corrida **duplica** el texto en un documento del usuario.

Procedimiento: antes de escribir el test, preguntate **que forma va a tener el
dato la primera vez que alguien lo use de verdad**. Si es una forma degenerada
—un subconjunto, un vacio, un solapamiento, un duplicado— esa es la que va al
test, no la limpia.

Formas degeneradas que casi siempre faltan:

| Caso | La forma que rompe |
| --- | --- |
| reemplazo de texto | el `despues` contiene al `antes` (insercion) |
| busqueda | la aguja aparece 0 veces, o 2 |
| ranking | todos los items empatan |
| parseo | el ejemplo vive dentro del documento que se parsea (#23) |
| filtro por nombre | el filtro no matchea nada y sale 0 (#23) |

## Cuando la herramienta que medis es un modelo

Un LLM no es un chequeo: es una opinion con formato. Todo lo de arriba sigue
valiendo, y ademas:

1. **Comparalo contra una metrica tonta.** Antes de creerle, medi lo mismo con
   algo deterministico (Jaccard, conteo, diff). No para reemplazarlo: para saber
   **donde discrepan**, que es lo unico interesante.

   En la #28, sobre 9 lecciones: el modelo y el Jaccard coincidieron en el par
   real (0.400 / 0.85) y discreparon en otro (0.048 / 0.60). El segundo era una
   vecindad semantica que la metrica lexica no podia ver — y aun asi se decidio
   NO fusionarlo. Las dos senales juntas dijeron mas que cualquiera sola.

2. **Un umbral que no podes calibrar es un numero inventado.** Con 9 items y un
   solo caso positivo, cualquier corte entre 0.1 y 0.4 da identico resultado: no
   hay nada en la zona gris. Reportá el numero y dejá decidir a quien lee, en vez
   de fingir una precision que el corpus no soporta.

3. **Verificá de punta a punta con el backend de verdad, no con uno falso.** Un
   mock prueba tu parser; no prueba que puedas hablar con un modelo. Y capturá
   la salida REAL como fixture: `claude -p` devuelve JSON pelado y `kimi -p` lo
   envuelve en vinnetas con banner y linea de sesion. Ninguna de las dos formas
   se te habria ocurrido inventarla.

4. **Recortá lo que ve.** Si el modelo no necesita el dato para su tarea, no se
   lo mandes. En la #28 ve nombre, descripcion y triggers, y **nunca** el cuerpo
   de la leccion: asi lo peor que puede hacer es equivocarse, no filtrar.

## Pitfalls

- **La fixture no tiene historia.** El caso que rompe suele necesitar un pasado:
  en la #22, una feature que declaro una leccion Y ademas pario otra, una
  preferencia que cita dos features, y dieciseis features anteriores a que la
  maquinaria existiera. Nada de eso aparece en un sandbox de tres archivos.
- **Reportar cosas que nadie puede corregir.** El primer mapa de la #22 reporto
  16 huecos, todos de features cerradas antes de que existieran las lecciones. Un
  reporte que grita por cosas que estan bien se ignora en dos dias, y con el se
  ignoran los huecos que si importan. Si un problema no es accionable, no es un
  problema: es ruido.
- **Comparar fechas cuando tenes timestamps.** El mismo mapa seguia reportando 2
  huecos falsos porque comparaba `2026-08-16` en vez de
  `2026-08-16T05:36:00Z`: tres features del mismo dia parecian simultaneas. Si el
  dato tiene mas precision, usala.
- **Confiar en que "la suite esta verde".** En la #20 el ADR salia en el puesto 10
  con todos los tests pasando; en la #22 habia tres bugs con la suite en verde.
  Verde significa "no rompi lo que ya sabia", no "esta bien calibrado".
- **Escribir la razon de una excepcion sin abrir el archivo.** Una razon
  decorativa es peor que ninguna: la excepcion queda con aspecto de decidida y
  nadie la vuelve a mirar.
- **Revisar solo el exit code de tu propia herramienta.** En la #25 `doctor`
  salia 0 y las siete areas parecian bien; recien al comparar **cada linea**
  contra el estado real del filesystem aparecio que el `[ok]` del hub era falso.
  El exit code es un resumen: los resumenes esconden justo lo que hay que ver.

## Verificacion

```bash
# 1. La suite, como siempre
cargo test

# 2. Y despues, lo que de verdad calibra: el comando sobre ESTE repo,
#    con un resultado esperado concreto
sh harness_cli <comando> | head -20

# 3. Para reportes: verificar a mano que cada item reportado es real
#    (un script chico que recalcule lo mismo por otro camino sirve)

# 4. Y para cada [ok], la pregunta del alcance: ¿que midio EXACTAMENTE?
#    Si la respuesta honesta es mas angosta que lo que dice la linea, corregí
#    la linea, no la conclusion.
```

Si la unica evidencia de que algo funciona es que los tests pasan, todavia no
sabes si funciona: sabes que no rompiste lo que ya habias previsto.
