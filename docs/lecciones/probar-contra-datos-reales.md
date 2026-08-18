---
nombre: probar-contra-datos-reales
descripcion: Verde no dice que este bien: dice que midio lo que sabias medir.
triggers: [fixtures, ranking, umbral, reporte, falso positivo, calibracion, datos reales, diagnostico, ok falso, alcance, health check]
relacionadas: [criterios-de-cierre-que-se-pueden-fallar, promesas-estructurales-vs-disciplina, reglas-que-se-aplican-a-si-mismas]
origen: [22, 25]
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
