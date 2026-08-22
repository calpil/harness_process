# Verificacion

Registra aqui los comandos oficiales por tipo de proyecto.

Ejemplos:

```bash
go test ./...
npm test
npm run lint
bash validate_ui.sh http://localhost:5173
```

Para cambios del instalador:

```bash
bash tests/setup_smoke.sh
# (parity_smoke.sh removido con los .py; solo Rust) # bash tests/parity_smoke.sh
(cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)
(cd rust && cargo test --locked)
```

Si preferis que cada AC declare su propia prueba, escribila en el spec debajo
del criterio y corre la verificacion desde aca:

```bash
sh harness_cli verify --feature <id>            # corre lo declarado, escribe el reporte
sh harness_cli verify --feature <id> --solo AC-3
```

Dos trampas al elegir el comando: uno que filtra tests por nombre suele salir
**0 cuando no encuentra ninguno**, y cualquier cosa que termine en `|| true` no
puede fallar. Un comando que no puede fallar no verifica: decora.

Paridad de los dos instaladores, **sin PowerShell**:

```bash
bash tests/parity_check.sh
```

Compara lo que `setup_harness.sh` y `setup_harness.ps1` **declaran** (opciones y
superficies) y falla cuando uno se adelanta al otro. Las asimetrias legitimas
estan declaradas en el propio script, cada una con su razon.

Lo que **no** hace, dicho de frente: **no ejecuta el instalador de Windows**. Un
`.ps1` estructuralmente paritario puede fallar igual al correr. Para eso hace
falta una maquina con PowerShell:

```powershell
# si tenes Windows a mano (si no, `parity_check.sh` es lo que hay)
.\tests\setup_smoke.ps1
.\harness_cli.ps1 status
```

Al correrlo por primera vez en Windows PowerShell 5.1 quedo claro por que la
deuda duro once features: el smoke usaba `-Encoding utf8NoBOM`, que **solo
existe en PowerShell 7**, y moria antes de la primera asercion aunque el archivo
declare `#requires -Version 5.1`. Corregido. Sigue **sin pasar entero**: siembra
un `harness.exe` falso (un archivo de texto) y despues le pide al CLI que
ejecute `prd add` de verdad, asi que necesita sembrar el binario real como hace
el smoke de sh. Lo que si quedo verificado en 5.1: `setup_harness.ps1` completa
una instalacion root de punta a punta.

El commit_guard, que es el gate que mas veces corre:

```bash
bash tests/commit_guard_check.sh
```

Cinco modos, e incluye la **prueba del rojo**: reconstruye la invocacion previa
al arreglo y verifica que esa si se cuelga. Sin ese modo, el que dice "no se
cuelga" podria estar pasando por casualidad.

El instalador para `cmd.exe`:

```bash
bash tests/cmd_installer_check.sh
```

Los modos que necesitan `cmd.exe` se saltean **con un `[Ok]` explicito** fuera de
Windows: un skip silencioso se lee igual que un verde, y no lo es.

Actualizacion: los dos smokes pasan enteros en Windows (`bash tests/setup_smoke.sh`
y `.\tests\setup_smoke.ps1`, los dos exit 0). El del `.ps1` no habia corrido nunca
y encontro cuatro fallas reales del arnes en Windows: el instalador `.ps1`
corrompia el UTF-8 de los templates, `.harness_layout` con CRLF no matcheaba,
`init.sh` no conectaba hooks de git, y el binario y los scripts no coincidian
sobre que es `$HOME`. Estan en el README, en la seccion de paridad.
