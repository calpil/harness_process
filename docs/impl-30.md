# Evidencia de implementacion - Feature #30: paridad_ps1_verificable

Spec: `docs/spec-feature-30-paridad-ps1-verificable.md` (`Estado: approved`, 11 AC)
Plan: `docs/plan-feature-30-paridad-ps1-verificable.md` (D1-D6, `Peldano elegido: 1`)
PRD: `docs/prd/PRD-master.md`

## La deuda que cierra

Once features seguidas (#17 a #26) cerraron con la misma linea: *"esta maquina no
tiene pwsh"*. Cuatro revisiones la levantaron sin decision. Mientras tanto, los
dos instaladores se desincronizaban en silencio.

No se cierra ejecutando el instalador de Windows —eso exige PowerShell, y Alan
decidio no instalarlo— sino comparando lo que los dos **declaran**. El chequeo
corre en cualquier maquina y falla cuando uno se adelanta al otro, que es el
problema real: no que el `.ps1` no se ejecute, sino que **nadie se entere**.

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `tests/parity_check.sh` | D1-D3 | NUEVO. Ocho modos, uno por AC, con la lista de asimetrias declaradas |
| `harness_check.sh` (+ espejo) | D4 | El aviso, que **no** toca `failures` |
| `docs/verification.md` (+ espejo) | D5 | La instruccion del smoke ps1 condicionada, con el sustituto nombrado |
| `README.md` | D5 | La seccion de paridad con la tabla de asimetrias |

## Evidencia por AC

`sh harness_cli verify --feature 30`: **11 verde, 0 rojo, 0 manual**.

| AC | Evidencia |
| --- | --- |
| AC-1 | `parity_check.sh opciones` sobre los dos instaladores reales |
| AC-2 | `parity_check.sh detecta-opcion` — la prueba del rojo, sobre copias |
| AC-3 | `parity_check.sh asimetrias-declaradas` (5, cada una con razon y lado) |
| AC-4 | `parity_check.sh superficies` |
| AC-5 | `parity_check.sh smokes` |
| AC-6 | README y `verification.md` dicen "no ejecuta el instalador de Windows" |
| AC-7 | `parity_check.sh promesa-acotada` |
| AC-8 | `parity_check.sh en-harness-check` (corre y no toca `failures`) |
| AC-9 | `parity_check.sh sin-ps1` |
| AC-10 | `Peldano elegido:` en el plan |
| AC-11 | 279 + 126 tests, clippy 0, `setup_smoke.sh` verde |

## Las cinco asimetrias, y las dos razones que escribi mal

El criterio de cierre pedia verificar **a mano** que cada razon fuera cierta. Dos
no lo eran:

| Opcion | Razon que escribi primero | Lo que dice el codigo |
| --- | --- | --- |
| `--with-postgres` | "afirmativa de un default ya encendido" | **Falso.** `setup_harness.sh:463` es `--with-postgres) ;;` — un **no-op**, y la ayuda dice "PostgreSQL es obligatorio; se mantiene por compatibilidad" |
| `-CargoTargetDir` | "rustup no siempre actualizo el PATH de la sesion" | **Falso.** Setea `CARGO_TARGET_DIR` (`setup_harness.ps1:672`), que no tiene nada que ver con el PATH |

Las otras tres si eran ciertas: `WITH_SUBAGENTS=1`, `INSTALL_GRAPHIFY=1` e
`INSTALL_ANTIGRAVITY=1` son defaults (lineas 29, 35, 36) y sus `--with-*` /
`--install-*` los vuelven a poner en 1.

Corregidas las dos, y el hallazgo quedo escrito **dentro del script**, arriba de
la lista, para que nadie agregue una razon decorativa despues:

> Las razones se verificaron UNA POR UNA contra el codigo antes de escribirlas, y
> dos salieron mal en el primer intento. Una razon decorativa es peor que
> ninguna: se cita como cierta.

Es la leccion `probar-contra-datos-reales` aplicada a la documentacion de una
excepcion: escribir la razon no la hace verdadera.

## Dos defectos del chequeo, encontrados corriendolo

1. **`\L` no existe en BSD sed.** La conversion PascalCase -> kebab reportaba
   basura (`--LNo-LGraphify` en vez de `--no-graphify`) y el chequeo "encontraba"
   nueve diferencias inexistentes. Reescrita en `awk`, que es portable.
2. **El parser no leia las ramas agrupadas del `case`.** `--dry-run|--preview)`
   se le escapaba, asi que creia que al `.sh` le faltaba `--dry-run`. Un falso
   positivo en la primera opcion que cualquiera mira. Arreglado partiendo por
   `|`.

Los dos son el mismo tipo de error: el chequeo daba **rojo falso**, que es el
otro lado de la moneda de la #25 (donde daba verde falso). Un instrumento
descalibrado miente en las dos direcciones.

## Lo que NO cubre, dicho en la doc

`docs/verification.md` y el README lo dicen con esas palabras: **el chequeo no
ejecuta el instalador de Windows**. Un `.ps1` estructuralmente paritario puede
fallar igual al correr. Prometer mas seria repetir exactamente el error del hub
en la #25 ("alcanzable" cuando solo se midio TCP).

La instruccion del smoke `.ps1` quedo condicionada en vez de borrada:

```powershell
# si tenes Windows a mano (si no, `parity_check.sh` es lo que hay)
.\tests\setup_smoke.ps1
```

## Para el backlog

- **La comparacion de smokes es por palabras clave**, no por escenarios
  nombrados: el `.sh` marca bloques con `[Ok] <tema>` y el `.ps1` usa 132
  `Assert-True` sin secciones. Si el `.ps1` adoptara marcadores, la comparacion
  podria ser por escenario en vez de por keyword.
- **Las superficies se comparan por presencia del nombre**, no por como se
  escriben. Un instalador podria nombrar `GEMINI.md` en un comentario y no
  escribirlo.
