# Evidencia de implementacion - Feature #46: verify_no_se_cuelga_con_salida_grande

Spec: `docs/spec-feature-46-verify-no-se-cuelga-con-salida-grande.md` (approved, 9 AC)
Plan: `docs/plan-feature-46-verify-no-se-cuelga-con-salida-grande.md`

## Que se construyo

En `rust/src/verificacion.rs`: `lector()` (un hilo por pipe, que retiene la cola
con tope de 4 MB sobre un `VecDeque`), `lanzar_lectores()` y
`juntar_lectores()`. `ejecutar()` ahora lanza los lectores **antes** de
`wait_timeout` en vez de leer despues. Esa inversion es toda la feature.

## La prueba que importa

El comando que ayer dejo a `verify` **once minutos colgado** —el smoke del
instalador— ahora es un AC declarado, y corre:

```
AC-8  $ bash tests/setup_smoke.sh
       [ok] verde (63216 ms)

8 verde(s), 0 en rojo, 1 manual(es).
```

| AC-8 | verde | `bash tests/setup_smoke.sh` | 0 | 63216 |

La feature se verifica a si misma: el AC que prueba que el gate ya no se cuelga
es, justamente, el comando que lo colgaba.

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 >64 KB por stdout | OK | `verify_salida_grande_stdout` (4.000 lineas, ~120 KB): verde y termina solo. Se colgaba antes del arreglo |
| AC-2 >64 KB por stderr | OK | `verify_salida_grande_stderr` |
| AC-3 por los dos a la vez | OK | `verify_salida_grande_ambos` — el caso real del instalador, que con un lector secuencial se cuelga igual (el segundo pipe se llena mientras se drena el primero) |
| AC-4 estado sobre la salida COMPLETA | OK | `verify_estado_sobre_salida_completa`: 4.000 lineas y el `test result:` al final -> **verde**; con `0 passed` -> **vacio**. La leccion de la #44 sigue viva |
| AC-5 el timeout sigue cortando | OK | `verify_timeout_sigue_cortando`: `sleep 30` con 1s -> `Timeout` y sin codigo de salida. Y `verify_nieto_que_hereda_el_pipe`, que nacio de la revision: un nieto con el pipe heredado ya no puede pisar el corte |
| AC-6 el reporte sigue recortando a 20 lineas | OK | `recortar_salida` no se toco; los reportes de esta corrida lo muestran |
| AC-7 tope y aviso | OK | `verify_salida_acotada`: 5 MB -> el reporte trae `omitidos por el tope` y `el estado se midio sobre lo retenido` |
| AC-8 el smoke declarado | OK | Tabla de arriba: verde, exit 0, 63 s |
| AC-9 los cuatro comandos | OK | Ver `docs/review-46.md` |

## Lo que aparecio al intentar romperlo

Un comando que deja un proceso en segundo plano con el pipe heredado no produce
EOF, y el `join` de los lectores se quedaba esperando **ignorando el timeout**:
`(sleep 30 &) ; echo listo` con timeout de 3 s tardaba 30.013 ms. Cambiar el
cuelgue del pipe lleno por el cuelgue del nieto habria sido el peor final
posible. Se arreglo con una gracia de 2 s y un buffer compartido del que se
puede tomar una foto sin esperar al hilo; el mismo caso vuelve ahora en
4.020 ms y el reporte avisa que un hijo dejo el pipe abierto. Detalle en
`docs/review-46.md`.

## Como se diagnostico (para la proxima vez)

No se dedujo: se midio. Con el proceso colgado, `lsof -p <pid>` mostro
`0r /dev/null`, `1 PIPE`, `2 PIPE` y **cero procesos hijos** — un bash que no
espera a nadie y no lee de nadie solo puede estar bloqueado escribiendo. Subir
`rules.verify_timeout_segundos` de 300 a 900 no cambio nada, que fue la pista
final de que el tiempo no era el problema.
