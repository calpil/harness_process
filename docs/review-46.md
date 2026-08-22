# Veredicto del reviewer - Feature #46: verify_no_se_cuelga_con_salida_grande

Veredicto: **approved**
Fecha: 2026-08-22
Spec: `docs/spec-feature-46-verify-no-se-cuelga-con-salida-grande.md` (approved, 9 AC)
Evidencia: `docs/impl-46.md`

Revision adversarial (feature #51): el objetivo fue encontrar OTRA forma de
colgar el gate, ahora que la del pipe lleno esta tapada.

## Lo que se rompio intentando romperlo

**Un defecto real, encontrado y arreglado durante la revision.**

**El nieto que hereda el pipe.** Un comando que deja un proceso en segundo plano
con el descriptor heredado no produce EOF cuando el hijo termina, y el `join` de
los hilos lectores se quedaba esperando **ignorando el timeout**. Medido:

```
ejecutar("(sleep 30 &) ; echo listo", timeout = 3s)
  -> 30.013 ms, estado Verde
```

El corte por timeout existia y el join lo pisaba: el gate esperaba treinta
segundos con un timeout de tres. Cambiar un cuelgue por otro habria sido el peor
resultado posible de esta feature.

Arreglado con `GRACIA_LECTOR` (2 s despues de que el proceso termina): el buffer
pasa a ser compartido (`Arc<Mutex<..>>`) para poder tomar una foto de lo leido
sin esperar al hilo, y si un lector queda abierto el reporte lo dice
(`un proceso hijo dejo el pipe abierto: se reporta lo leido hasta el corte`).
Con el arreglo, el mismo caso vuelve en **4.020 ms**.
Test: `verify_nieto_que_hereda_el_pipe`.

Vale aclarar que este riesgo **ya existia** antes de la feature (`leer_salida`
tambien bloqueaba), y que los procesos detached del propio arnes —el push de
Atlassian y graphify— no caen aca: los dos redirigen sus descriptores a `null`.

## Intentos que NO rompieron nada

| Intento | Resultado |
| --- | --- |
| 120 KB por stdout / stderr / los dos | Termina y reporta; antes se colgaba |
| 5 MB de salida | Retiene la cola (4 MB), declara el recorte y sigue |
| `sleep 30` con timeout de 1 s | `Timeout` y sin codigo de salida, como antes |
| Salida larga con el resumen al final | El estado se sigue midiendo sobre la salida completa (leccion #44) |
| El smoke del instalador declarado como AC | **verde, exit 0, 63 s** |

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | 369 unit + 177 integracion en verde |
| `cargo clippy --all-targets -- -D warnings` | 0 hallazgos |
| `bash tests/setup_smoke.sh` | exit 0 — y ahora corre DENTRO de `verify` (AC-8) |
| `bash harness_check.sh` | limpio |
| `harness verify --feature 46` | 8 verdes, 0 en rojo, 1 manual |

## Constitution

- **Articulo 1**: 7 tests nuevos, seis de los cuales se colgaban antes del
  arreglo y uno que nacio de la revision.
- **Articulo 2**: spec `approved` con las dos observaciones decididas antes de
  escribir codigo.
- **Articulo 3**: la tabla de `impl-46.md` cita AC-1..AC-9.
- **Articulo 4**: sin secretos; el reporte declara recorte y pipe abierto en vez
  de callarlos.
- **Articulo 5**: OBS-1 (4 MB) y OBS-2 (cola + medir sobre lo retenido)
  decididas por Alan.
- **Articulo 6**: sin dependencias nuevas (`std::thread` y `std::sync`);
  `wait-timeout` sigue solo para el corte.

## Reparos

1. **La gracia son 2 segundos fijos.** Si un lector todavia esta drenando una
   salida gigante justo cuando el proceso termina, se le corta la cola. Es poco
   probable (el hilo lee mientras el comando corre), pero no es imposible y no
   hay palanca para ajustarlo.
2. **`rules.verify_timeout_segundos` quedo en 900 en este repo** porque el smoke
   compilando desde cero se acerca a los 300 por default. En un repo nuevo, el
   default sigue siendo 300 y un comando lento va a salir `timeout`: es
   correcto, pero conviene saberlo antes de declarar comandos largos.
3. **Nada mide cuanta memoria usa el gate** con varios AC verbosos seguidos. El
   tope es por comando, no por corrida.
