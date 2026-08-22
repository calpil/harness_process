Aplicado: 2026-08-22T17:58:08Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #46: verify_no_se_cuelga_con_salida_grande

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 46`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'verify_no_se_cuelga_con_salida_grande')
Veredicto: cambio
Antes:
| 12 | El arnes no se bloquea a si mismo | el_guard_no_bloquea_por_lo_que_escribe_el_arnes | <O1> | El commit guard deja de contar como sucios los documentos que escribio el propio arnes (specs, planes, impl, review, verify, estados, prd-diff, `docs/prd/**`, `docs/lecciones/**`, architecture y perfil), exigiendo nombre Y ubicacion bajo `docs/`; sigue bloqueando por codigo y por cualquier documento ajeno, y dice en una linea `[i]` cada vez que se saltea un repo. Disparador: en un proyecto donde `docs/` es su propio repo, cada start/advance/prd apply terminaba el turno pidiendo un commit por microservicio de archivos que el `close` iba a commitear | done (2026-08-22) |
Despues:
| 12 | El arnes no se bloquea a si mismo | el_guard_no_bloquea_por_lo_que_escribe_el_arnes | <O1> | El commit guard deja de contar como sucios los documentos que escribio el propio arnes (specs, planes, impl, review, verify, estados, prd-diff, `docs/prd/**`, `docs/lecciones/**`, architecture y perfil), exigiendo nombre Y ubicacion bajo `docs/`; sigue bloqueando por codigo y por cualquier documento ajeno, y dice en una linea `[i]` cada vez que se saltea un repo. Disparador: en un proyecto donde `docs/` es su propio repo, cada start/advance/prd apply terminaba el turno pidiendo un commit por microservicio de archivos que el `close` iba a commitear | done (2026-08-22) |
| 13 | Verificar lo que de verdad prueba, aunque hable mucho | verify_no_se_cuelga_con_salida_grande | <O1> | `verify` lee los pipes con un hilo por descriptor MIENTRAS el comando corre, en vez de leerlos despues de esperarlo: un comando que imprime mas que el buffer del pipe (~64 KB) ya no cuelga el gate. Retiene la cola con tope de 4 MB declarando el recorte, sigue midiendo el estado sobre la salida completa (leccion #44), sigue cortando por timeout y no se deja pisar por un nieto que hereda el pipe. Disparador: el smoke del instalador dejo a verify once minutos colgado y quedo sin poder declararse como AC | done (2026-08-22) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'verify_no_se_cuelga_con_salida_grande')
Veredicto: cambio
Antes:
**El arnes no se bloquea a si mismo, tambien en el guard** (feature #58).
Despues:
**Un gate solo verifica lo que puede ejecutar** (feature #46). El comando que
mejor prueba una feature suele ser el mas verboso, y era justo el que no se
podia declarar: `verify` leia los pipes DESPUES de esperar al proceso, asi que
cualquier comando que pasara los ~64 KB del buffer trababa a los dos. Tres
decisiones:

- **Leer mientras corre, no despues.** Un hilo por descriptor, lanzado antes de
  esperar. Es la unica forma de que el productor no se bloquee.
- **Ningun camino puede esperar sin limite.** El timeout corta al proceso y una
  gracia corta corta a los lectores: si un nieto heredo el pipe, se reporta lo
  leido y se sigue. Cambiar un cuelgue por otro es no haber arreglado nada.
- **Lo que se recorta se declara.** Tope de 4 MB reteniendo la cola —donde estan
  los resumenes que deciden el estado— y una linea en el reporte diciendo
  cuanto quedo afuera y sobre que se midio.

**El arnes no se bloquea a si mismo, tambien en el guard** (feature #58).

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'verify_no_se_cuelga_con_salida_grande')
Veredicto: cambio
Antes:
## El commit guard y los artefactos del arnes (feature #58)
Despues:
## Lectura de la salida en `verify` (feature #46)

`rust/src/verificacion.rs`: `ejecutar()` lanza `lanzar_lectores()` **antes** de
`wait_timeout`. Cada pipe tiene su hilo (`lector()`), que vacia hasta el EOF
sobre un `VecDeque` compartido (`Arc<Mutex<Buf>>`) reteniendo la **cola** con
tope `MAX_SALIDA_BYTES` (4 MB). `juntar_lectores()` espera a cada hilo como
mucho `GRACIA_LECTOR` (2 s) despues de que el proceso murio y, si alguno quedo
abierto —un nieto con el descriptor heredado—, toma una foto del buffer y lo
declara en el reporte. El estado se sigue midiendo sobre la salida retenida
completa (leccion de la #44: el resumen llega al final, detras de la
compilacion). Sin dependencias nuevas: `wait-timeout` sigue solo para el corte.

## El commit guard y los artefactos del arnes (feature #58)

