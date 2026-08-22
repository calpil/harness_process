# Veredicto del reviewer - Feature #58: el_guard_no_bloquea_por_lo_que_escribe_el_arnes

Veredicto: **approved**
Fecha: 2026-08-22
Spec: `docs/spec-feature-58-el-guard-no-bloquea-por-lo-que-escribe-el-arnes.md` (approved, 10 AC)
Evidencia: `docs/impl-58.md`

Revision adversarial (feature #51): el objetivo fue que el guard dejara pasar
algo que no debia.

## Lo que se rompio intentando romperlo

**Un defecto real, encontrado y arreglado durante la revision.**

**El nombre no alcanzaba.** La primera version comparaba solo el nombre del
archivo, aceptando la ruta con o sin prefijo `docs/`. Resultado, reproducido:

```
$ echo "notas del servicio" > ms-auth/impl-notas.md
$ commit_guard.sh
[i] ms-auth: solo artefactos del arnes sin commitear [...]; no cuenta como sucio.
rc=0
```

Un documento del microservicio, llamado `impl-notas.md`, se eximia como si fuera
un artefacto del arnes: el guard dejaba de mirar un archivo real. Es el mismo
error que la feature venia a arreglar, del otro lado.

Arreglado exigiendo la UBICACION ademas del nombre: la ruta empieza con `docs/`
o el repo sucio ES el `docs/`. Sigue siendo por artefacto (un `docs/runbook.md`
bloquea igual), pero un `impl-*.md` dentro de un microservicio ya no engaña a
nadie. Con el arreglo, el mismo caso da `rc=2`. Assert nuevo en el smoke.

**Dos defectos en el propio test.**

El bloque nuevo del smoke usaba
`var="$(guard)"; rc=$?`, y bajo `set -e` una asignacion desde un comando que
sale 2 aborta el script **en silencio**: el smoke terminaba con exit 2 sin
imprimir un solo `[!]`. Corregido con `var="$(guard)" || rc=$?`. Vale anotarlo
porque un test que muere callado se lee como "fallo la feature".

Y `verify` **no puede correr el smoke**: lo cuelga. AC-9 salio rojo con
`timeout` a los 300.001 ms, y subir `rules.verify_timeout_segundos` a 900 no
arreglo nada — el smoke siguio once minutos sin avanzar. La causa no era el
tiempo: el instalador estaba **bloqueado escribiendo a un pipe lleno**
(`lsof`: stdin `/dev/null`, stdout y stderr `PIPE`, sin hijos). Es la feature
**#46** del backlog, textual: *"`ejecutar()` llama `wait_timeout` antes de leer
los pipes, asi que un comando que imprime mas que el buffer del pipe (~64KB) se
cuelga"*. Reproducida en vivo por primera vez.

Consecuencia para esta feature: AC-9 se queda **sin `Comando:`** y se verifica a
mano (smoke exit 0). Declarar un comando que cuelga el gate es peor que no
declararlo. El primer sintoma —`grep: .../post-commit: No such file or
directory`— era ruido: al cortar el comando, el `trap` del smoke borra su
`TMP_ROOT` mientras un paso posterior todavia lo usa.

De paso, y aunque NO era la causa, el smoke quedo aislado del entorno de quien
lo llama (`unset HARNESS_REPO_ROOT` en su cabecera): un test cuyo resultado
depende de las variables del proceso padre adivina en vez de verificar.

## Intentos que NO rompieron nada

| Intento | Resultado |
| --- | --- |
| Artefacto sin trackear (`??`) | Eximido |
| Artefacto MODIFICADO (` M`) | Eximido: no depende del estado de git |
| `docs/runbook.md` (doc ajeno, misma carpeta) | Bloquea: la exencion es por archivo |
| Codigo en otro repo | Bloquea ese repo, y `docs` se sigue eximiendo |
| El proyecto real con 8 microservicios sucios | `docs` sale de la lista, los 8 siguen bloqueando |

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | 362 unit + 177 integracion = **539 en verde** (sin cambios de Rust en esta feature) |
| `cargo clippy --all-targets -- -D warnings` | 0 hallazgos |
| `bash tests/setup_smoke.sh` | exit 0, con el bloque `Guard #58` |
| `bash harness_check.sh` | limpio |
| `sh -n commit_guard.sh` | sintaxis OK (los dos archivos, fuente y plantilla) |

## Constitution

- **Articulo 1**: seis casos nuevos en el smoke, sobre el guard **instalado**, no
  sobre la fuente.
- **Articulo 2**: spec `approved` con las dos observaciones decididas por el
  usuario antes de tocar codigo.
- **Articulo 3**: la tabla de `impl-58.md` cita AC-1..AC-10.
- **Articulo 4**: el guard no deja de mirar nada que hoy mire, salvo lo que
  escribio el propio arnes; y cuando se saltea algo, lo dice.
- **Articulo 5**: OBS-1 (por artefacto) y OBS-2 (linea `[i]`) decididas por Alan.
- **Articulo 6**: `templates/commit_guard.sh` y `commit_guard.sh` identicos.

## Reparos

1. **La lista de patrones es una constante en bash.** Si mañana el arnes escribe
   un tipo de documento nuevo con otro nombre, hay que acordarse de sumarlo aca.
   Nada lo detecta: el sintoma seria volver a ver el turno bloqueado.
2. **`docs/prd/**` y `docs/lecciones/**` se eximen enteros**, y ahi adentro hay
   documentos del USUARIO (el PRD es suyo). Es deliberado —el `close` los
   commitea— pero significa que un PRD editado a mano tampoco dispara el guard.
   La red que si los cubre es la de rutas protegidas, que sigue activa.
3. **El caso que disparo todo sigue teniendo 8 microservicios con codigo sin
   commitear** en `realestate`. Eso el guard lo sigue diciendo, y con razon.
