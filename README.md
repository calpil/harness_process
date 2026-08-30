# Harness Process

Instalador de un arnes multi-repo para Claude Code, Codex, Gemini, Grok,
Kimi Code, Antigravity y otros agentes CLI. Genera superficies de
instrucciones, hooks, launchers, memoria compartida y una capa opcional de
subagentes.

## Requisitos

- Bash 3.2 o superior para `setup_harness.sh` y los hooks POSIX existentes
- Windows PowerShell 5.1 o PowerShell 7 para `setup_harness.ps1`
- Git
- Rust + cargo (requerido): compila el binario nativo `harness` durante el setup.
  `harness_cli` (sh/ps1) despacha exclusivamente al binario; sin el binario falla.
- `curl`, `uv` o `pipx` solo cuando se instalan herramientas opcionales (graphify etc)

## Instalacion

El layout predeterminado es `subdir`: este repositorio vive dentro de la raiz
multi-repo y escribe las superficies de agente en el directorio padre.
La carpeta `templates/` pertenece a este repositorio fuente. Una distribucion
aplanada puede dejar esos archivos junto a `setup_harness.sh`; el instalador no
exige ni crea `templates/` en el proyecto destino.

```bash
cd /ruta/al/proyecto/harness_process
./setup_harness.sh
```

En Windows:

```powershell
cd C:\ruta\al\proyecto\harness_process
.\setup_harness.ps1
```

PowerShell busca `cargo.exe` en `PATH`, `$env:CARGO_HOME\bin` y
`$HOME\.cargo\bin`. Si rustup todavia no actualizo la sesion, agrega la carpeta
de Cargo al `PATH` del proceso antes de compilar. Se puede fijar el target:

```powershell
$env:CARGO_HOME = "$HOME\.cargo"
.\setup_harness.ps1 -CargoTargetDir "$PWD\.cargo-target"
```

El instalador agrega `harness_cli.ps1`, que ejecuta `harness.exe` (Rust). Git for Windows Bash sigue siendo necesario para scripts/hook POSIX historicos; ambos instaladores se mantienen. (Sin fallback Python desde feature #2).

Desde `cmd.exe`, sin abrir PowerShell a mano:

```bat
setup_harness.cmd
setup_harness.cmd --dry-run
harness_cli.cmd status
```

`setup_harness.cmd` **no es un tercer instalador**: una tercera implementacion
garantizaba drift con las otras dos, que es justo lo que `tests/parity_check.sh`
existe para evitar. Resuelve solo lo que `cmd.exe` no sabe hacer solo: encuentra
PowerShell (pwsh o el 5.1 del sistema), saltea la ExecutionPolicy que rechaza un
`.ps1` sin firmar —del alcance de ese proceso, sin tocar la configuracion de la
maquina— traduce las opciones estilo `.sh` (`--dry-run` -> `-DryRun`) y devuelve
el exit code de verdad.

`harness_cli.cmd` es el comando del dia a dia desde `cmd.exe`: va directo a
`harness.exe` sin pagar el arranque de otro shell, y traduce el binario mas
viejo que los scripts al mismo remedio que la version sh. Los dos se instalan
junto a `harness_cli` y `harness_cli.ps1`.

Para instalar el arnes directamente en la raiz multi-repo:

```bash
./setup_harness.sh --root
```

```powershell
.\setup_harness.ps1 -Root
```

Instalacion sin graphify ni cambios globales adicionales:

```bash
./setup_harness.sh \
  --no-graphify \
  --no-graphify-skills \
  --no-antigravity
```

```powershell
.\setup_harness.ps1 -NoGraphify -NoGraphifySkills -NoAntigravity
```

El Memory Hub usa exclusivamente PostgreSQL. Configura la conexion en el entorno:

```bash
export DB_HOST=localhost
export DB_USER=harness
export DB_PASSWORD='...'
export DB_NAME=harness
export DB_SSL_MODE=require
./setup_harness.sh
```

```powershell
$env:DB_HOST = "localhost"
$env:DB_USER = "harness"
$env:DB_PASSWORD = "..."
$env:DB_NAME = "harness"
$env:DB_SSL_MODE = "require"
.\setup_harness.ps1
```

Tambien se pueden guardar esas variables en `$HARNESS_HUB/.env`.
`DB_SSL_MODE` usa `require` por defecto.

`DB_STATEMENT_TIMEOUT` (milisegundos, `30000` por defecto, `0` desactiva) corta
del lado del servidor cualquier sentencia que se pase de ese tiempo: un hub que
deja de responder falla con un error legible en vez de colgar el comando para
siempre (`connect_timeout` solo cubre el saludo inicial). Junto con eso, el
candado del hub es **por proyecto** (`$HARNESS_HUB/.lock-<proyecto>`), asi que
varios repos de la misma maquina ya no hacen fila entre ellos; el guardado
escribe unicamente las filas que el comando toco, en lotes, de modo que ningun
proyecto reescribe las filas de otro.

Al actualizar una instalacion antigua, `graph_db.json` y `progress/` se migran
a PostgreSQL. Luego se respaldan bajo `bkp/memory-hub/` y se eliminan del Hub
activo. Las consultas posteriores se realizan solo en PostgreSQL.

## Opciones

Ejecuta `./setup_harness.sh --help` para ver todas las opciones. Las mas utiles:

- `--root` / `--subdir`: selecciona el layout.
- `--no-subagents`: omite roles y backlog ejecutable.
- `--no-graphify`: no instala el CLI de graphify.
- `--no-graphify-skills`: no modifica skills globales de agentes.
- `--no-antigravity`: no instala Antigravity CLI.
- `--no-kimi`: no escribe el bloque de hooks globales de Kimi Code en
  `KIMI_CODE_HOME/config.toml` (los artefactos de proyecto se generan igual).
- `--force`: sobrescribe sin crear backups.
- `--dry-run` (o `--preview`): modo simulado, no escribe ni instala nada (ideal para auditar).
- `--reset`: limpia todas las superficies, hooks, agentes, binarios y marcadores generados por el arnes (respaldando primero). No toca tu codigo.
- `--version`: muestra la version del instalador.
- `--json`: emite al final un reporte JSON con contadores de acciones.
- `--log-file <ruta>`: escribe log plano (sin ANSI) a un archivo.
- `--config <ruta>`: carga variables de entorno extra desde un archivo (se evalua temprano).

PowerShell usa los equivalentes `-Root`, `-Subdir`, `-NoSubagents`,
`-NoGraphify`, `-NoGraphifySkills`, `-NoAntigravity`, `-NoKimi`, `-Force`,
`-DryRun`, `-Reset`, `-Version`, `-Help`, `-Json`, `-LogFile`, `-Config` y
`-CargoTargetDir`.

Los backups se guardan en `bkp/`. Usa `HARNESS_BKP_DIR` para cambiar la ruta.

Nuevas mejoras (2026 best practices aplicadas):
- shebang portable (`#!/usr/bin/env bash`) + shellcheck-ready
- logging con colores + niveles
- lockfile anti-concurrencia
- reintentos con backoff en descargas
- descarga verificada (no pipe ciego) para Antigravity CLI
- guidance de PATH despues de installs --user
- soporte config file + dry-run + reset + reporte de idempotencia

Ejemplo dry-run:
```bash
./setup_harness.sh --dry-run --json
```

## Actualizacion (proceso explicito)

El Harness Process se actualiza **re-correndo el instalador**. Esto es intencional y explicito:

- Las superficies (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`) y los subagentes se generan desde los heredocs del instalador.
- Los scripts (`harness_cli`, `harness_check.sh`, roles, etc.) se copian desde `templates/`.
- El binario Rust `harness` se compila desde `rust/` con cargo (requerido); `harness_cli` despacha exclusivamente a el.

Para recibir mejoras (nuevo protocolo de `check-plan`, recordatorios de planes actualizados por otros LLMs, fixes, nuevas opciones, etc.):

```bash
# Ve a la carpeta del harness_process (la fuente)
cd /ruta/al/harness_process

# Actualizacion normal (hace backups de lo anterior)
./setup_harness.sh

# O para una reinstalacion limpia de las superficies:
./setup_harness.sh --reset
```

El instalador respalda archivos existentes en `bkp/` (a menos que uses `--force`).

**NUNCA commitees la carpeta del harness** (`harness_process/` o el subdirectorio donde está `setup_harness.sh`). 

El instalador la agrega automáticamente a `.gitignore` del proyecto. El harness es una **herramienta** que vive en su propio repositorio fuente separado. No forma parte del código de tu proyecto.

No existe (ni se recomienda) un comando magico `harness_cli upgrade` dentro del proyecto. La forma correcta y explícita de actualizar es volver a ejecutar el instalador desde la carpeta fuente de `harness_process`.

## harness_cli: binario Rust

Todos los hooks, scripts y docs invocan `sh .../harness_cli <cmd>`:

- `harness_cli` (sh/ps1) despacha exclusivamente al binario `harness` (o
  `harness.exe` en Windows). Es un solo ejecutable multi-OS
  (macOS/Windows/Linux) con los comandos de ciclo de vida al tope (`status`,
  `start`, `check-plan`, `check-spec`, ...) y el Memory Hub bajo
  `harness graph <cmd>` (`mapa`, `impacto`, `vincular`, ...).
- Sin el binario (cargo/rustup ausente) `harness_cli` falla pidiendo compilar;
  no hay fallback Python desde la feature #2.

Regla de mantenedor: cualquier cambio de comportamiento vive en `rust/src/` con
sus tests (`cargo test`, `cargo clippy -- -D warnings`) verdes antes de push.
Detalles en `templates/UPDATING.md`.

## superseded: lo que se hizo, pero en otra feature (feature #37)

El arnes tenia cuatro estados y le faltaba una palabra. Cuando la #36 agrupo seis
entradas del backlog en una sola feature, esas seis no eran `done` —nunca
tuvieron spec ni evidencia propia— ni `blocked` —no estaban trabadas—. Quedaron
en `blocked`, que sugeria un problema donde no lo habia, y ademas inflaban el
denominador de `prd tree`.

```bash
sh harness_cli close --feature 31 --status superseded --absorbida-por 36
```

**Exige decir cual feature absorbio el trabajo**, y esa referencia se valida: no
se puede citar una feature inexistente, ni absorberse a si misma. Queda como
campo `superseded_by`, no como prosa en `note`.

| Como se lee | |
| --- | --- |
| `status` | `#31 [superseded por #36] close_exit_codes_unificados` |
| `next` | no la ofrece: no es trabajo pendiente |
| `prd tree` | **no la cuenta ni arriba ni abajo**: no es trabajo hecho ni pendiente |
| `journey` | no la reporta como cierre sin leccion: su aprendizaje se declaro en la que absorbio |

Y **no pasa por los gates de `done`** —spec, leccion, verify, documentos— a
proposito: el trabajo y su evidencia viven en la feature que la absorbio.
Exigirle spec propio seria exactamente el problema que el estado vino a resolver.

La migracion es **explicita**: el arnes no puede saber cuales de tus `blocked`
estaban absorbidas y cuales trabadas de verdad.

## lecciones consolidar: cuando dos lecciones cuentan lo mismo (feature #28)

La guia manda **patchear el paraguas existente** antes de crear una leccion
nueva, y cerrar con "pocas lecciones de clase, ricas". El curador (#21) detecta
las frias, pero **no ve solapamientos**.

Y ya habia pasado: `docs-generados-por-el-instalador` y
`documentos-del-usuario-vs-plantillas` compartian 4 triggers, el mismo pitfall
del `.ps1` casi palabra por palabra y el mismo bloque de Verificacion. La mas
nueva **declaraba el solapamiento en su propia prosa** y aun asi era un archivo
aparte.

```bash
sh harness_cli lecciones consolidar          # pregunta e INFORMA; no toca nada
sh harness_cli lecciones consolidar --preparar --en <paraguas> --de a,b
sh harness_cli lecciones consolidar --aplicar     --en <paraguas> --de a,b --motivo "<por que>"
```

`--preparar` crea un borrador nuevo con la unión de triggers y un
`[[miembro]]` por cada selección; no archiva nada y, si el archivo ya existe,
lo preserva. Solo después de escribir la prosa humana corresponde `--aplicar`.

El tramo de modelo está apagado por default y de forma **estructural**: sin
`rules.consolidar_backend` no se resuelve backend, no se spawnea nada y ni
siquiera se mira el entorno. Las referencias mutuas `relacionadas` sí se pueden
informar localmente, porque no gastan cuota ni envían contenido fuera del repo.

```json
{ "rules": { "consolidar_backend": "auto" } }
```

### Que puede y que no puede hacer el modelo

| | |
| --- | --- |
| Ve | `nombre`, `descripcion` y `triggers` |
| **Nunca ve** | **el cuerpo**: los procedimientos y los pitfalls no salen de `docs/` |
| Puede | proponer grupos con un motivo y una confianza |
| **Nunca puede** | escribir. `detectar()` no recibe `&HarnessPaths`: no tiene con que |

Y el prompt viaja como **un item de argv**, jamas por `sh -c`: una descripcion
con backticks o `$(...)` no puede ejecutar nada. Por eso este modulo NO reusa
`verificacion::ejecutar`, que si corre con shell.

Lo que el modelo devuelve se **valida** antes de mostrarse: un miembro que no
existe se descarta **y se dice**, y un grupo que toca una leccion `pinneada` se
descarta.

### La cadena de backend

`HARNESS_CONSOLIDAR_CMD` (override) -> primer CLI de la tabla (`claude -p`,
`kimi -p`) -> **skip limpio**. El override elige *cual* backend, nunca *enciende*
la feature.

**El tramo de API key no esta implementado**, y el mensaje de skip lo dice: el
arnes no habla HTTP. Serian tres formatos de request/respuesta/error escritos sin
poder verificarlos. Se nombra en vez de disimularse.

### La fusion la pide una persona

`--aplicar` toma la fusion de **argv**, no de lo que dijo el modelo. Y antes de
archivar nada exige que el paraguas pueda reemplazarlo:

- no puede tener los placeholders de la plantilla;
- tiene que heredar **todos los triggers** de cada miembro — `buscar` puntua una
  leccion activa 100 y una archivada 30, asi que sin eso el conocimiento deja de
  encontrarse;
- tiene que citar `[[cada-miembro]]` como puntero de recuperacion.

**Nunca borra**: las miembros van a `docs/lecciones/archivo/` con su cuerpo
intacto byte a byte, hay backup previo y `lecciones rollback` lo deshace.

### La confianza se reporta sin filtrar

El modelo devuelve una confianza por candidato y se imprime tal cual. **No hay
umbral**: con 9 lecciones y un solo solapamiento real (Jaccard 0.400 sobre
triggers contra 0.050 del siguiente) no hay nada en la zona gris con que
calibrarlo, y un umbral no calibrable es un numero inventado con aspecto de
rigor. El numero ordena; la persona decide.

## prd propose / prd apply: los documentos dejan de poder mentir (feature #29)

El arnes verificaba que el codigo cumpliera el **spec** (#23) y nada mas. El
cuerpo del PRD, el SDD y `docs/architecture.md` podian decir cualquier cosa.

No era un descuido: "es del USUARIO" esta escrito en cuatro lugares del repo. El
problema es que nunca quiso decir "que quede mintiendo". Tres hechos medidos:

- `docs/prd/SDD-master.md` era una plantilla con **27 `<placeholder>`** — y se
  publica a Confluence como el diseno tecnico del proyecto.
- `docs/architecture.md` no mencionaba `doctor.rs` (#25) ni `rutas.rs` (#26), dos
  features cerradas el mismo dia con todo en verde.
- `grep -rn "architecture.md" roles/ CHECKPOINTS.md` daba **cero**.

```bash
sh harness_cli prd propose --feature <id>   # el BINARIO pregunta, uno por documento
# el agente contesta cada bloque
sh harness_cli prd apply --feature <id>     # muestra que escribiria; NO escribe
sh harness_cli prd apply --feature <id> --yes   # solo con el SI del usuario
```

**El alcance lo calcula el binario**, no el agente: el PRD de origen, todos sus
padres hasta el maestro, el SDD y `architecture.md`. Si lo eligiera el agente,
"el SDD ya lo refleja" seria una afirmacion sin contraparte.

### Los tres veredictos

| Veredicto | Cuando | Como se verifica |
| --- | --- | --- |
| `cambio` + `Antes:` / `Despues:` | hay que escribir | el `Antes:` tiene que aparecer **exactamente una vez** en el documento |
| `ya-esta <archivo>:<L1>-<L2>` | ya estaba documentado | **el binario abre el archivo y comprueba la cita** |
| `no-aplica <razon>` | la feature no toca ese documento | la razon no puede estar vacia |

El del medio es el importante: convierte la mentira mas probable del agente —"eso
ya esta documentado"— en algo refutable sobre bytes, sin heuristica y sin LLM.

Y el agente **no puede agregar, quitar ni renombrar bloques**: si la lista no
coincide con el alcance recomputado, `prd apply` sale 2. Sin eso podria colapsar
cuatro preguntas en una respuesta.

### Dos decisiones que parecen detalles y no lo son

- **El anclaje es por texto literal, no por seccion.** `prd::aplicar_vuelta`
  corta secciones con `starts_with("## ")` y `docs/architecture.md` tiene tres
  `###` que ese predicado se tragaria enteros.
- **La idempotencia sale del contenido, no de una firma.** El spec es 1:1 con su
  feature y por eso se puede firmar (`last_spec_sig`); un PRD lo comparten N
  features —`PRD-master.md` lo comparten 28 del backlog— asi que una firma por
  feature mentiria desde la segunda.

### El gate

```json
{ "rules": { "require_docs_al_dia": true } }
```

Con la regla activa, `close --status done` exige la propuesta **resuelta y
aplicada con el SI del usuario**. Y **no** compara frescura contra
`docs/verify-<id>.md`: `verify` reescribe su reporte en cada corrida y
`prd apply` es idempotente, asi que esa regla dejaria la propuesta vieja para
siempre, sin ningun comando capaz de refrescarla.

**Un aviso si escribis specs**: ningun `Comando:` de ningun AC puede invocar
`prd apply --yes`. `verify` los ejecuta con `sh -c`, asi que aplicaria la
propuesta sin el si del usuario, salteandose el ritual entero. Hay un test que lo
prohibe sobre los specs reales del repo.

## La vuelta al PRD no se pierde ni miente (feature #60)

Cerrar una feature como `done` **vuelve al PRD**: marca la fila de su hito y
deja una linea en `## Bitacora`. Durante un tiempo esa promesa fue falsa de dos
maneras distintas, y las dos salieron de la misma linea mal ubicada.

### Lo que pasaba

El cierre resolvia el PRD con las rutas de la FEATURE (`para_feature`), asi que
escribia en la copia del PRD que vive dentro del worktree, y lo hacia **antes**
de integrar. Como el PRD es un documento que comparten todas las features, dos
cierres en paralelo apendeaban al final de la misma seccion:

```
main            ...#46
 |__ wt A (fork en #46) -> agrega #55 -> commit en la rama -> merge  ─┐
 |__ wt B (fork en #46) -> agrega #57, #38, #39 -> ya en main         │
                                                                     v
                                        CONFLICTO en la ultima linea
                                        resuelto a favor de main
                                        la linea de A desaparece
```

En el repo del propio arnes eso se llevo **7 de 18** cierres: las bitacoras de
las features #40, #41, #42, #43, #53, #54 y #55 hubo que transcribirlas a mano
(commit `docs(prd): preserva cierres 40-55`). El cierre no fallaba: el `[i]`
correspondiente se perdia entre el resto de la salida.

Y el puntero al spec se calculaba con `relpath` contra el spec del worktree, o
sea `../<repo>-wt/<id>-<slug>/docs/spec-*.md`: una ruta al arbol que el propio
cierre borra con `git worktree remove --force` **unos segundos despues**. Habia
18 asi. El `impl: docs/impl-<id>.md` era fijo, sin comprobar que existiera.

### Lo que hace ahora

```
close --status done
 |__ integrar(): commit + merge + borrar worktree
 |__ ¿el merge salio bien?  -> si no, no se marca ningun hito
 |__ decidir_vuelta()   funcion PURA: arma la linea y valida los punteros
 |__ aplicar_vuelta()   la UNICA que escribe, y escribe en la RAIZ
```

- **En la raiz.** El log de cierre no pertenece a ninguna rama, asi que deja de
  viajar en una. No hay dos lados que apendeen al mismo lugar: no hay conflicto
  que resolver mal. El PRD queda modificado sin commitear en el checkout
  principal, como el resto de los documentos que el arnes toca.
- **Despues de integrar.** Un hito marcado afirma que el trabajo esta en la rama
  destino. Sin merge no lo esta, asi que no se marca.
- **Ningun puntero sin verificar.** Cada ruta tiene que ser relativa a la raiz y
  abrir un archivo que existe; la que no cumple no se escribe y se dice por que.
  Que la promesa la sostenga la ESTRUCTURA y no la disciplina es deliberado: la
  parte que decide es pura y no tiene con que escribir, y la que escribe solo
  sabe ejecutar un plan ya validado (leccion `promesas-estructurales-vs-disciplina`).

### `prd doctor`: el pendiente que no depende de que alguien lo anote

```sh
sh harness_cli prd doctor            # informe: NO escribe. Sale 2 si hay hallazgos
sh harness_cli prd doctor --reparar  # aplica los arreglos
```

Contrasta el backlog con el arbol de PRDs y encuentra:

| Hallazgo | Que hace `--reparar` |
| --- | --- |
| Puntero que no resuelve (escapa de la raiz, o el archivo no existe) | lo reescribe al archivo que si existe en `docs/`, o lo quita antes que mentir |
| Feature `done` sin linea de bitacora en su PRD | la agrega, con la fecha de SU cierre (`closed_at`), no la de hoy |
| Fila de hito de una feature `done` sin marcar | la marca `done (YYYY-MM-DD)` |

La gracia esta en de donde sale el pendiente: **del backlog**, no de un archivo
que el cierre tenga que acordarse de escribir cuando algo sale mal. Una feature
`done` que no esta en su PRD **es** el hallazgo, aunque la perdida haya sido
hace meses y nadie la haya registrado. Por eso el comando encontro, y reparo,
los 18 punteros rotos y las 13 bitacoras que faltaban desde antes de que la
vuelta al PRD existiera.

`harness_check.sh` lo corre en modo informe y lo reporta con `[i]`: **avisa y no
bloquea**, por la misma razon que la paridad de instaladores — un PRD
desactualizado no impide trabajar hoy, y un proyecto que arrastraba punteros
rotos no puede quedarse sin poder cerrar por un arreglo del arnes.

## Rutas protegidas: los PRD dejan de depender de la buena fe (feature #26)

El README dice que los PRD son del usuario y que ningun agente los reescribe.
Hasta esta feature eso era **solo una frase**: no habia un gate, y con un backend
en modo permisivo tampoco un prompt en el medio.

```json
{ "rules": { "rutas_protegidas": ["docs/prd/**", "docs/constitution.md", ".env"] } }
```

Tres capas, y cada una dice lo que **no** puede:

| Capa | Que puede | Que **no** puede |
| --- | --- | --- |
| `PreToolUse` | impedir la escritura, incluso en modo permisivo | existir donde el backend no tiene el evento (hoy: solo Claude Code) |
| `PostToolUse` | avisar en el acto con el comando de reversion | **prevenir**: corre despues, ya se escribio |
| `harness_check.sh` | bloquear con exit 2 | actuar en el momento del dano |

```bash
sh harness_cli rutas                     # que esta protegido
sh harness_cli rutas --check <ruta>      # ¿esta protegida? (exit 2 si si)
sh harness_cli rutas --violaciones       # tocadas y sin commitear
```

**El arnes no se bloquea a si mismo.** `close` escribe en el PRD cada vez que
marca un hito y `prd add` crea PRDs: las dos son rutas protegidas. Cuando el
binario escribe una, la anota con su mtime, y la exencion caduca en cuanto
alguien vuelve a tocar el archivo.

**Adoptarla con trabajo en curso**: `sh harness_cli rutas --aceptar-estado-actual`
toma el estado actual como linea de base, para que el gate no arranque en rojo
por cambios legitimos que ya estaban.

**El remedio dice lo que destruye.** El aviso trae `git diff` primero y despues
el comando destructivo, etiquetado:

```
docs/constitution.md
    mira que cambio: git diff -- docs/constitution.md | y si no fue tuyo:
    git checkout -- docs/constitution.md (DESCARTA todo lo no commiteado de ese archivo)
```

No es cortesia: durante el desarrollo de esta feature el aviso decia solo
`git checkout -- <ruta>`, se corrio tal cual, y borro los hitos de tres features
que estaban sin commitear. Detalle completo en `docs/rutas-protegidas.md`.

## doctor: ¿esto esta bien instalado? (feature #25)

`harness_check.sh` contesta "¿el proceso va bien?". `doctor` contesta la otra
pregunta, la que hasta ahora no contestaba nadie:

```bash
sh harness_cli doctor          # las siete areas, con remedio por cada problema
sh harness_cli doctor --json   # area, estado, detalle y remedio, parseable
```

```
== Harness Doctor: la instalacion ==
   arnes: /Users/alan/proyecto/harness_process
   raiz:  /Users/alan/proyecto

[ok] binario       harness presente, ejecutable y al dia
[!!] hooks         claude instalado pero falta bin/harness-hook
                   Remedio: bash setup_harness.sh
[--] superficies   ningun backend instalado
[ok] marker        marker 'subdir', raiz resuelta: /Users/alan/proyecto
[i]  hub           no acepta conexiones; el arnes sigue funcionando sin el
                   Remedio: verifica la red o ~/.harness-hub/.env
[ok] herramientas  requeridas y opcionales presentes (git)
[i]  graphify      no esta en el PATH: el arnes funciona igual
                   Remedio: bash setup_harness.sh --with-graphify
```

Cada problema trae **el comando que lo arregla**, copiable tal cual. Y el exit
code no miente: **2 solo si algo te impide trabajar** (binario roto, hook
apuntando a la nada, herramienta requerida ausente). El hub caido, graphify
ausente y las herramientas opcionales son avisos `[i]` y salen **0** — toda una
sesion de trabajo de este repo transcurrio con el hub caido sin un solo problema.

### Las siete areas diagnostican fallas que ya ocurrieron

Ninguna se invento:

| Area | La falla real que cubre |
| --- | --- |
| binario | `git pull` deja los scripts nuevos y el binario viejo. El sintoma era `unrecognized subcommand 'perfil'`, tres pasos despues y con otro nombre |
| marker | `.harness_layout` perdido y la raiz resuelta al lugar equivocado: costo la feature #10 entera |
| superficies / hooks | instalacion a medias, o el checkout FUENTE confundido con una instalacion (feature #7) |
| hub | inalcanzable, que es normal y no deberia parecer una rotura |
| herramientas | falta `git` (o `cargo` donde hay que compilar) |
| graphify | ausente, que es perfectamente valido |

### Lo que doctor NO puede hacer, dicho de frente

Un doctor que vive en el binario **no puede diagnosticar un binario ausente**.
Esa mitad la cubre el lanzador `harness_cli`, que ahora traduce dos casos al
mismo remedio en vez de dejar salir un error cripitico:

```
[harness_cli] El binario instalado no conoce el subcomando 'doctor': es mas viejo
              que los scripts que lo invocan (tipico de 'git pull' sin
              re-correr el instalador).
              Remedio: bash setup_harness.sh
```

Y tampoco valida el handshake de PostgreSQL: comprueba que el hub **acepte
conexiones TCP**, nada mas. Lo dice asi en la salida porque decir "alcanzable" a
secas seria un OK falso — durante el desarrollo de esta feature el hub aceptaba
TCP y aun asi las operaciones morian con `connection reset`.

`doctor` **no arregla nada**: imprime el comando y lo corres vos. Misma decision
que el curador (#21) y el mapa (#22).

## Convenciones que se pueden usar para rechazar algo (feature #24)

`docs/conventions.md` lleva dos cosas que no son consejos: son criterios que el
reviewer aplica.

### La escalera de huella

Cada peldano deja mas superficie permanente que el anterior. Se elige el de
**menor huella que resuelva el problema**:

```
1. extender lo que ya existe      cero superficie nueva
2. flag en un comando existente   la diferencia es un parametro, no un flujo
3. comando nuevo                  hay un verbo propio, con su exit code
4. superficie nueva               tiene que sobrevivir a la sesion
5. dependencia nueva              ultimo recurso, exige ADR (Articulo 6)
```

Si no tomas el mas alto, el plan lo declara con la linea que el reviewer busca:

```
Peldano elegido: 3 (comando nuevo) porque <por que el peldano de arriba no alcanzaba>
```

La razon tiene que decir por que el peldano de arriba **no alcanzaba**. "Queda
mas claro asi" no es una razon. Esta feature se aplico la escalera a si misma:
salio peldano 1 (documentacion + un bloque en `harness_check.sh`), cero comandos
y cero dependencias — si hubiera necesitado un comando, habria nacido
contradiciendo la escalera que introduce.

### Las tres reglas de test

1. **Contratos de comportamiento, no snapshots.** Assertea como se relacionan
   dos cosas, no el valor de hoy. `assert_eq!(fuentes().len(), 12)` se rompe cada
   vez que alguien agrega una fuente legitima; `assert!(f.peso() > 0)` no.
2. **Prohibido leer el codigo fuente en un test.** Un test que lee el texto de un
   `.rs` prueba la *forma* del codigo: pasa con la implementacion sutilmente rota
   y falla ante un refactor correcto. **Unica excepcion**: el archivo es *dato de
   entrada* del codigo bajo prueba (los specs que parsea el verificador, las
   plantillas que siembra el instalador). El corte: *¿el test seguiria valiendo
   si la implementacion se reescribiera entera?*
3. **Prohibido el test detector-de-cambios.** El que falla cada vez que se
   actualiza un dato que se espera que cambie (catalogos, versiones, conteos).
   No agrega cobertura: solo rompe CI cuando alguien hace una actualizacion
   rutinaria.

`harness_check.sh` **avisa** cuando un test lee el fuente, con archivo, linea y
nombre del test — y **no bloquea**, porque la regla tiene una excepcion legitima
y un gate duro empujaria a inventar un `--force`, que es peor que el aviso. Las
otras dos reglas no se chequean solas: saber que dato "se espera que cambie" no
se grepea, y las verifica el reviewer.

La feature #24 cobro su primera deuda en el acto: el test
`verify_should_not_be_wired_into_any_hook` de la #23 grepeaba `src/**/*.rs` y
quedo reescrito como `only_verify_should_execute_declared_commands`, que declara
`Comando: touch rastro.txt` en un spec, corre el arnes entero y mira el disco.
Declararlo excepcion habria vaciado la regla en su primera aplicacion.

## Spec-Driven Development (SDD)

Cada feature arranca con un spec antes de tocar codigo (inspirado en spec-kit,
adaptado y en layout plano):

- `sh harness_cli start --feature <id>` genera, ademas del plan,
  `docs/spec-feature-<id>-<slug>.md` en el `docs/` de la RAIZ (junto a los
  planes, sin carpetas `specs/NNN/`) con `Estado: draft`: recorridos de usuario
  priorizados (P1/P2), criterios de aceptacion AC-n en Given/When/Then, no
  funcionales y fuera de alcance.
- El LIDER completa el spec y el plan (cada item de la Delegacion cita su AC-n)
  y ejecuta el **ritual de aprobacion**: le MUESTRA el spec al usuario (contenido
  en el chat + abierto en su editor), le PREGUNTA si lo aprueba y solo con su SI
  lo REGISTRA con `sh harness_cli approve-spec --yes [--nota "<como aprobo>"]`.
  El comando escribe `Estado: approved`, sella quien/cuando y re-firma el spec
  (por eso aprobar no dispara la alarma de "spec actualizado por otro LLM").
  Sin `--yes` el comando se niega con exit 2: ningun agente aprueba por su
  cuenta, y la decision sigue siendo exclusivamente del usuario.
- Gate `require_spec_approved`: con la regla `"require_spec_approved": true` en
  `rules` de `feature_list.json` y el spec sin aprobar, `advance`,
  `close --status done` y `harness_check.sh` bloquean con mensaje accionable.
  Sin la regla (o en `false`) el gate queda apagado (compat con instalaciones
  previas).
- `sh harness_cli check-spec` reporta el estado del gate (exit 0 aprobado o
  regla apagada, 1 sin feature activa, 2 spec stale o sin aprobar con la regla
  activa); `check-plan` vigila la frescura de spec y plan frente a ediciones de
  otros LLMs.
- `docs/constitution.md` (principios del proyecto) lo siembra el instalador
  (`setup_harness.sh` / `setup_harness.ps1`) solo si falta y nunca lo pisa;
  specs y planes deben cumplirlo y el reviewer lo verifica.

## Lecciones: la memoria procedural del proyecto (feature #17)

Los artefactos de una feature (`spec-*`, `plan-*`, `impl-*`, `review-*`) cuentan
**que paso en la feature N**: quedan ordenados por id, que es el orden en que
nadie los busca. Una **leccion** es lo mismo reordenado por **clase de trabajo**,
que es como se busca de verdad seis meses despues.

```bash
sh harness_cli leccion list             # el catalogo, ordenado por uso
sh harness_cli leccion show <clase>
sh harness_cli leccion nueva <clase>    # crear es el ULTIMO recurso
sh harness_cli leccion usar <clase>     # +1 uso: distingue lo vivo de lo muerto
```

Cada leccion es `docs/lecciones/<clase>.md` con frontmatter (`nombre`,
`descripcion`, `triggers`, `relacionadas`, `origen`, `usos`, `ultimo_uso`,
`ultima_actualizacion`, `estado`) y cuatro secciones: **Cuando aplica**,
**Procedimiento**, **Pitfalls** y **Verificacion**. El metodo completo —el orden
de preferencia y, sobre todo, la lista de **que NO capturar**— vive en
`docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`.

Dos reglas que sostienen todo lo demas:

- **Primero patchear, crear al final.** El orden es: patchear la leccion que
  estuvo en juego > patchear el paraguas existente > agregar
  `docs/lecciones/<clase>/referencias/<tema>.md` > recien entonces crear una
  clase nueva. La forma que se busca es pocas lecciones de clase, ricas, no una
  lista plana de una-por-feature.
- **El nombre es de CLASE.** `leccion nueva` rechaza (exit 2, sin escribir nada)
  el nombre que contenga `feature` o `#`, empiece con `fix-`/`debug-`/`audit-`/
  `hotfix-`, lleve una fecha o un numero de tres o mas digitos. **No hay
  `--force`**: si el nombre solo tiene sentido para la tarea de hoy, esta mal.

El gate del cierre es **opt-in**. Con `"require_leccion": true` en `rules` de
`feature_list.json`, `close --status done` exige declarar que se aprendio:

```bash
sh harness_cli close --feature 17 --status done --leccion espejo-de-roles
sh harness_cli close --feature 17 --status done \
   --leccion ninguna --leccion-motivo "trabajo mecanico, sin tecnica nueva"
```

Sin la regla (ausente o en `false`, el default) el cierre se comporta exactamente
como antes. `ninguna` siempre es una salida valida —pero no deberia ser la
respuesta por default—; sin motivo, el comando se niega.

`harness_check.sh` valida el arbol: frontmatter ilegible o un `nombre:` que no
coincide con el archivo **bloquean** nombrando el archivo; una leccion sin
`triggers` solo avisa con `[i]`. Sin `docs/lecciones/` el bloque entero se omite.

Las lecciones son **conocimiento ganado**: `--reset` no las borra (solo refresca
la guia, que es plantilla del arnes). Y son archivos versionados del repo, asi
que no llevan secretos.

### El arnes te empuja solo (feature #18)

Una memoria en la que hay que acordarse de escribir no se llena. El arnes empuja
en los dos momentos donde hay senal, siempre por **stderr** y siempre con exit 0:

- **Cada N escrituras** (`rules.leccion_nudge_interval`, default **25**): el hook
  `PostToolUse` ya invoca `harness_cli nudge` en cada tool-use, asi que el arnes
  cuenta y, al llegar al intervalo, recuerda en cuatro lineas mirar el catalogo y
  patchear antes que crear. Con `0` queda apagado.
- **Al cerrar sin declarar**: si `close --status done` no trae `--leccion`, el
  arnes emite el **contrato** completo. Su texto **se lee de
  `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`**, no vive duplicado en el
  binario: editas la guia y cambia el contrato, sin recompilar y sin que puedan
  divergir. Si la guia falta o esta incompleta, degrada a un puntero de dos
  lineas — leer la guia nunca puede romper un cierre.

Nada de esto corre en un proyecto **sin** `docs/lecciones/`: ni se emite una
linea ni se crea un archivo. Y el nudge nunca escribe una leccion: emite el
contrato, el agente decide y escribe, el gate del cierre verifica.

De paso, el aviso de "sin feature activa" dejo de repetir lo mismo cada diez
minutos para siempre: ahora **escala** mientras nada cambia (600 s -> 1200 ->
2400 -> 3600, y ahi se estaciona) y vuelve al piso apenas aparece una feature
activa.

## Perfil de usuario: lo que ya decidiste, en la superficie (feature #19)

El arnes acumula decisiones tuyas en `progress/history.md`, en los planes y en
los specs — y ningun agente las lee nunca. El perfil las destila y las **inyecta
en las superficies** que cada backend lee al arrancar.

```bash
sh harness_cli perfil sugerir      # junta la evidencia ya escrita; NO escribe nada
sh harness_cli perfil show         # el perfil y cuanto ocupa
sh harness_cli perfil add     --texto "<entrada>" --yes
sh harness_cli perfil replace --old "<fragmento>" --texto "<nuevo>" --yes
sh harness_cli perfil remove  --old "<fragmento>" --yes
```

El flujo es explicito sobre quien decide: **el arnes junta** (`sugerir` agrupa
los registros por feature y emite el contrato de como destilar una entrada), **el
agente propone**, y **vos decidis**. Los tres comandos de escritura exigen
`--yes` y se niegan sin el, igual que `approve-spec`.

Cuatro reglas que sostienen el resto:

- **Limite duro de 1500 caracteres** (solo las entradas, no el encabezado). Al
  pasarse, el comando **falla** mostrando las entradas actuales y pidiendo
  consolidar en el mismo turno: nunca recorta nada en silencio. Es chico a
  proposito — cada caracter se paga en todas las sesiones de todos los backends.
- **Nada entra sin tu si.** `docs/perfil-usuario.md` es documento tuyo: se siembra
  si falta, un reinstall no lo pisa y `--reset` no lo borra.
- **Se bloquean los secretos.** Una entrada con pinta de credencial, clave privada
  o Unicode invisible se rechaza **antes** de escribir: este archivo se versiona
  (queda en git para siempre) y ademas viaja al prompt de cada agente.
- **Snapshot congelado.** `perfil add` escribe el archivo, pero el bloque de las
  superficies se refresca al **reinstalar**: la sesion en curso no cambia bajo los
  pies del agente.

El bloque se inyecta entre marcadores propios en `CLAUDE.md`, `AGENTS.md`,
`GEMINI.md` y `LLM.md`, de forma idempotente (reinstalar no lo duplica ni toca
nada fuera de los marcadores). Sin `docs/perfil-usuario.md` —o con el archivo sin
entradas— no se inyecta nada y las superficies quedan exactamente como antes.

`harness_check.sh` valida el perfil con `perfil check`: pasarse del limite
**bloquea**.

## verify: que un AC diga como se prueba (feature #23)

Hasta aca, la relacion entre un criterio de aceptacion y el test que lo cubre
vivia **en la cabeza de quien lo escribio**. Si el test se borraba, el AC seguia
diciendo "cubierto" para siempre.

Un AC puede declarar, en la linea de abajo, **como se prueba**:

```
- AC-5: Given un spec en draft, When corre `verify`, Then se niega con exit 2.
  Comando: `cd rust && cargo test verify_should_refuse_to_run_commands_from_a_draft_spec`
```

```bash
sh harness_cli verify --feature 23              # los corre y escribe el reporte
sh harness_cli verify --feature 23 --solo AC-5  # iterar sobre uno solo
sh harness_cli verify --feature 23 --json       # estado por AC, parseable
```

```
AC-1  $ cd rust && cargo test verificacion::tests::parse
       [ok] verde (272 ms)
AC-5  $ cd rust && cargo test verify_should_refuse_to_run_commands_from_a_draft_spec
       [ok] verde (80 ms)
AC-18  $ grep -q "require_verify_green" README.md
       [!!] rojo (5 ms)

18 verde(s), 1 en rojo, 1 manual(es).
Reporte: docs/verify-23.md
```

Un AC **sin** `Comando:` queda como **manual**: lo verifica el reviewer, igual que
siempre, y **no cuenta como fallo**. Por eso los specs ya escritos siguen valiendo
sin tocar una linea.

### `vacio`: el AC que salio 0 sin medir nada (feature #44)

`cargo test un_nombre_que_no_existe` imprime `running 0 tests`, dice
`test result: ok` y **sale 0**. Un AC asi no esta verificado: esta **sin medir**,
y hasta la #44 quedaba registrado como verde. Le paso al AC-12 de la feature
#28, que declaraba el invariante mas citado de ese comando —sin `--aplicar` no
toca nada— contra un test que no existia.

Por eso `verify` mira la **salida** ademas del exit code:

```
AC-1  $ cd rust && cargo test consolidar_without_aplicar_should_not_touch_anything
       [??] vacio (312 ms)

26 verde(s), 0 en rojo, 0 manual(es), 1 sin casos.
Un AC `sin casos` corrio y salio 0, pero no ejecuto ningun test:
revisa que el nombre del filtro exista de verdad.
```

`vacio` **bloquea el cierre** igual que un rojo, y se cuenta aparte en el resumen
para no esconderlo entre ellos. Tres cosas que el detector NO hace:

- **No mira el texto del comando.** Mira la forma de la salida, asi que un
  `cargo test` adentro de un script tambien queda cubierto, y un comando que se
  llama "test" sin serlo, no.
- **No opina de lo que no entiende.** Si la salida no tiene lineas
  `test result:` —un `grep`, un `bash`, un compilador— el estado no cambia.
- **No se apaga con un flag.** Un AC que no midio nada no es una preferencia; si
  de verdad se verifica a mano, el camino honesto es no declarar `Comando:` y
  dejarlo en `manual`.

### Las tres barreras

`verify` es el **unico** comando del arnes que ejecuta shell, y eso define su
diseno:

1. **Exige el spec aprobado.** En `draft` se niega con exit 2 y no ejecuta ni un
   comando. Aprobar el spec es el acto en el que el USUARIO leyo esos comandos:
   es lo que impide que corra algo que escribio un agente y nadie miro.
2. **Se invoca a mano.** Ningun hook ni ningun otro comando lo llama.
3. **Imprime cada comando antes de correrlo.** Nada a ciegas.

Y **cerrar nunca ejecuta**: el gate LEE `docs/verify-<id>.md`. Por eso el reporte
se versiona en vez de recalcularse al cerrar.

### El gate del cierre

```json
{ "rules": { "require_verify_green": true } }
```

Con la regla activa, `close --status done` exige que el reporte **exista**, sea
**mas nuevo que el spec** (un verde de antes de cambiar los criterios no prueba
nada) y **no tenga rojos** —nombrando cuales fallaron—. Sin la regla, o con un
spec cuyos AC no declaran comandos, cerrar se comporta exactamente como siempre.

`rules.verify_timeout_segundos` (default 300) corta un comando colgado; el AC
queda en `timeout` y **la corrida sigue** con los demas.

### Lo que un comando declarado NO garantiza

Un comando trivial pasa igual. Dos trampas concretas, las dos encontradas
corriendo esto sobre su propio spec:

- **`cargo test <nombre>` con cero coincidencias sale 0.** Un nombre de test mal
  escrito da verde sin ejecutar nada. Verifica que el comando imprima el test que
  corrio, no solo que salga 0.
- **`... | grep -c ... || true` nunca falla.** Un comando que no puede fallar no
  verifica: decora.

El reporte muestra el comando de cada AC precisamente para que el reviewer pueda
juzgar si prueba algo.

## revision: el veredicto que el cierre puede leer (feature #64)

`docs/review-<id>.md` existe desde la primera feature, pero cerrar nunca lo miro:
40 de los 55 cierres `done` tienen review, los otros 15 no, y nadie se entero. La
regla `require_review` lo convierte en el quinto gate de `close --status done`:

```json
{ "rules": { "require_review": true } }
```

Con la regla activa, cerrar como `done` exige que `docs/review-<id>.md` **exista**,
lleve el **sello del binario** y que ese sello diga **`approved`**; si no, exit 2
con un mensaje `[GATE]` que nombra cual de las tres cosas falta. Sin la regla
(ausente o en `false`, el default de las cinco) cerrar se comporta exactamente
como siempre. El molde de `feature_list.json` la trae en `true`, y los
instaladores AGREGAN al `rules` de un proyecto ya instalado las claves que le
falten **sin pisar jamas un valor existente**: si la apagaste, sigue apagada.

El sello lo estampa el unico comando que escribe en el review:

```bash
sh harness_cli revision --feature 64                       # el paquete de revision: SOLO lee
sh harness_cli revision --feature 64 --veredicto approved  # lo unico que escribe
```

La linea canonica queda tras el titulo del review, donde se lee sin scrollear:

```
Revisado: approved · 2026-08-28T13:05:41Z · estampado por `harness revision --veredicto`
```

Los tres veredictos son los de `roles/reviewer.md` (`approved`,
`changes_requested`, `blocked`), estampar es idempotente (reemplaza el sello
anterior si ya habia) y deja el rastro `revision feature #64 veredicto=approved`
en `progress/history.md`.

### Por que el sello lo escribe el binario y no el reviewer

Porque el gate **no parsea prosa**, y no la parsea porque los reviews reales no
se dejan: de los 40 que ya existen, 7 no tienen ninguna linea parseable, y
`docs/review-3.md:3` dice

> Veredicto: approved (implementación) — cierre BLOQUEADO por una acción humana
> pendiente (aprobación del spec por el usuario).

Un `contains("approved")` habria aprobado el cierre de un review que dice que el
cierre esta bloqueado. Por eso el gate lee **unicamente** la linea que estampo el
binario: un `Veredicto: approved` tipeado a mano no cuenta como revision.

### Una fila por AC, y con cita

`revision --veredicto` se niega —sin escribir nada— si el review no responde por
**cada AC-n que declara el spec** con una fila que lo nombre y cite
`archivo:linea`. La lista sale del SPEC, no del review: si saliera del review, un
review vacio estaria "completo". Y el corte es la **cita**, porque una fila sin
`archivo:linea` es una afirmacion, y una afirmacion es justo lo que un review de
cinco segundos sabe escribir.

Lo que el gate **no** hace: comparar el review contra `docs/impl-<id>.md`. El
ciclo normal es reviewer pide cambios -> implementer corrige, asi que el impl
queda mas nuevo casi siempre; esa regla bloquearia para siempre con una unica
salida barata, `touch`, y lo que entrenaria es el `touch`.

Los 15 cierres viejos sin review **no se reconstruyen**: la regla aplica de la
#64 en adelante. Un review escrito despues de que el codigo se integro y funciona
no intenta romper nada, solo llena el casillero, y `roles/reviewer.md` define el
rol como exactamente lo contrario, con un titulo de seccion: "Tu trabajo es
intentar ROMPER, no confirmar".

## journey: el mapa de lo aprendido (feature #22)

Los tres almacenes juntos, con sus enlaces y —lo que de verdad importa— **sus
huecos**:

```bash
sh harness_cli journey          # linea de tiempo + huecos
sh harness_cli journey --json   # nodos, enlaces y huecos
```

```
2026-08-16
  #17 lecciones_memoria_procedural
      `-- [leccion declarada] docs-generados-por-el-instalador
      `-- [leccion (origen)] hitos-del-prd
2026-08-17
  #19 perfil_de_usuario
      `-- [perfil] Ante un gate, prefiere bloquear a avisar... (#17, #19)

[Ok] Sin huecos: los tres almacenes son coherentes entre si.
```

Una feature puede **declarar** una leccion al cerrar y ademas haber **parido**
otra por el camino: el mapa muestra las dos, porque mostrar solo la declarada
perderia la mitad de lo aprendido.

Los **huecos** son lo que hace util al mapa: enlaces rotos (una leccion o una
entrada del perfil que cita una feature inexistente), features que cerraron sin
declarar nada, lecciones huerfanas y archivos ilegibles. Por cada uno imprime
**el comando que lo corrige**.

`journey` es de **solo lectura**: no tiene `delete` ni `edit`. Podar pasa por los
comandos de cada almacen (`lecciones archivar`, `perfil remove --yes`), que ya
tienen sus garantias — una segunda puerta podria saltear el "nunca borra" del
curador y el gate del `--yes` del perfil.

## El curador: que la biblioteca no se pudra (feature #21)

Una biblioteca sin mantenimiento se llena de casi-duplicados y de cosas que ya no
son ciertas. El curador da el ciclo de vida, **determinista y sin modelo**:

```bash
sh harness_cli lecciones status              # que vive, que se enfria, que vence
sh harness_cli lecciones curar               # que pasaria (SOLO informa)
sh harness_cli lecciones curar --aplicar     # respalda, aplica y deja reporte
sh harness_cli lecciones rollback            # deshace la ultima pasada
sh harness_cli lecciones pin <clase>         # congela: nada automatico la toca
sh harness_cli lecciones archivar|restaurar <clase>
```

Las transiciones son aritmetica de fechas: **30 dias** sin uso pasa a `stale`,
**90 dias** archiva (`rules.leccion_stale_dias` / `leccion_archivo_dias` los
ajustan; `0` apaga ese tramo). El uso **resucita**: una `stale` que vuelve a
usarse vuelve a `activa`.

Cuatro garantias, y las cuatro se probaron pudiendo fallar:

- **Nunca borra.** Archivar es *mover* a `docs/lecciones/archivo/`. No existe
  ningun subcomando que elimine una leccion.
- **Nada se mueve sin `--aplicar`.** La pasada por defecto solo informa: mover
  archivos de alguien en un hook, sin que lo pida, no es curar.
- **Toda pasada mutante respalda antes**, en `bkp/lecciones/<ts>/`, y el
  `rollback` **tambien es reversible** (respalda el estado actual antes de
  restaurar).
- **Archivar no la esconde.** `buscar` sigue encontrando las archivadas, por
  debajo de cualquier fuente activa. Por eso la carpeta es `archivo/` y no
  `.archivo/`: `buscar` saltea los directorios ocultos, y el conocimiento
  archivado desapareceria.

Cada pasada aplicada deja `progress/lecciones/<ts>/REPORT.md` con que transiciono,
cuantos dias llevaba inactiva cada una, que se salteo por `pin` y donde quedo el
backup.

## buscar: preguntarle al repo (feature #20)

Una memoria que no se puede consultar no es memoria. `buscar` recorre los
artefactos del proceso y responde con archivo, linea, feature y fecha.

```bash
sh harness_cli buscar "ureq adr"        # terminos en cualquier orden
sh harness_cli buscar "opcion segura" --json
sh harness_cli buscar "gate" --todos    # sin el tope de 20
```

Lo que lo separa de un `grep -r` es el **orden**, que va de lo mas curado a lo
mas crudo:

```
lecciones y perfil   ->  conocimiento curado, escrito para reusarse
specs, planes, ADRs  ->  decisiones, lo que se acordo antes de hacer
impl, review, estado ->  evidencia, lo que efectivamente paso
history.md           ->  bitacora cruda
```

Dentro de cada nivel pesan mas los encabezados, las frases contiguas y las
features recientes. El `score` va en `--json` para que el ranking sea
**auditable** y no una caja negra.

Tres garantias:

- **Sin indice.** El corpus de un proyecto son ~1 MB de texto: escanearlo entero
  toma milisegundos (medido: **10 ms** sobre 114 archivos y 28.000 lineas de este
  repo) y un indice desactualizado miente, que es peor.
- **Sin LLM y sin hub.** No hay modelo en el camino ni conexion que pueda fallar.
- **Solo lectura.** No escribe un byte, ni en `docs/` ni en `progress/`.

Si ninguna linea tiene todos los terminos, cae a las que tienen alguno **y lo
dice**; si hay mas de 20 resultados, muestra los primeros **y dice cuantos
quedaron fuera**. No encontrar sale con 0: no encontrar no es un error.

Es complementario de `graphify query`, que responde sobre el **grafo del codigo**;
`buscar` responde sobre los **artefactos del proceso**.

## harness_check.sh: gates de integridad

`bash harness_check.sh` (exit 0 limpio / 2 con fallos; `HARNESS_CHECK_MODE=block|warn|off`)
valida estado del backlog, frescura de plan/spec, checkpoints, commit guard y el
mapa de agentes. Desde la feature #7 incluye ademas:

- **Gate de espejo de roles**: `roles/*.md` es la fuente unica. El check compara
  el cuerpo embebido de `.claude/agents/*.md` (tambien leidos por Grok),
  `.gemini/agents/*.md` y `.codex/agents/*.toml` contra `roles/<rol>.md`, y
  `roles/*.md` contra `templates/roles/*.md` (modulo `__HREL__`). Un espejo
  desincronizado bloquea, nombrando el archivo; el remedio es re-correr el
  instalador (o propagar el cambio a `roles/` si lo editado fue el espejo). Los
  espejos que no existen no fallan.
- **Resolucion de raiz robusta**: los cuatro scripts (`harness_check.sh`,
  `harness_status.sh`, `init.sh`, `commit_guard.sh`) y el binario Rust resuelven
  `REPO_ROOT` con la misma regla: overrides primero (`HARNESS_REPO_ROOT`,
  variables de agente), luego el marker `.harness_layout`; y si el marker dice
  `subdir` pero el directorio es un checkout FUENTE (tiene
  `templates/harness_cli` + `rust/`) con un padre sin huella de instalacion (o
  el padre es `$HOME` sin `HARNESS_ALLOW_HOME_SURFACE=1`), la raiz es el propio
  checkout, con aviso informativo `[i]`. El marker ya no esta versionado (es
  estado local que escribe el instalador).
- **Layout inferido cuando falta el marker** (feature #10): como des-versionar
  `.harness_layout` lo borra del working tree de toda instalacion que hace
  `git pull`, su AUSENCIA ya no se interpreta como layout root. Si el padre tiene
  huella de instalacion (`docs/constitution.md`, `CLAUDE.md`, `AGENTS.md`,
  `.claude/settings.json`) y no es `$HOME`, se infiere `subdir` y la raiz es el
  padre, con un aviso `[i]` que recuerda que re-correr el instalador regenera el
  marker (los scripts nunca lo escriben). Sin huella no se infiere nada, y un
  marker presente con otro valor (`root`) se respeta al pie de la letra.

## Kimi Code CLI: backend con hooks globales (unica excepcion de `$HOME`)

Kimi Code CLI (v0.29.x) es backend de primera clase: lee el `AGENTS.md`
generado (verificado empiricamente: lo inyecta a su system prompt), recibe los
tres roles como subagentes nativos en `.kimi-code/agents/*.md` (allowlist de
`tools` por rol) y arranca con `bin/harness-kimi`.

Su particularidad: **Kimi no soporta hooks por proyecto** — el unico lugar
donde existen es el config global `${KIMI_CODE_HOME:-~/.kimi-code}/config.toml`.
Por decision del usuario (2026-07-28) el instalador escribe alli un bloque
`[[hooks]]` para `SessionStart`, `PostToolUse` (matcher `Edit|Write`) y `Stop`.
Es la **unica** escritura del arnes fuera del proyecto, blindada:

- Solo si se detecta Kimi en la maquina (`kimi` en PATH o
  `KIMI_CODE_HOME/bin/kimi`); `--no-kimi` / `-NoKimi` la excluye.
- Backup previo en `bkp/` antes de tocar el archivo.
- Bloque delimitado por marcadores propios, con reemplazo idempotente SOLO
  entre marcadores: los hooks y config del usuario quedan intactos.
- Validacion best-effort con `kimi doctor` + rollback si el TOML quedo
  invalido (nunca rompe el resto del setup).
- Cada comando del bloque es un guard: solo actua si `$PWD/bin/harness-hook`
  existe (proyecto con arnes); en cualquier otro proyecto es no-op silencioso.
- `--reset` NO lo toca (es compartido por todos los proyectos de la maquina);
  la remocion manual esta documentada en `UPDATING.md`.

## Documentacion del proceso: toda en el `docs/` de la RAIZ

Con el arnes en una subcarpeta (`<proyecto>/harness_process/`, layout por
defecto), TODA la documentacion del proceso vive en el `docs/` de la RAIZ del
proyecto, junto a los docs del equipo:

```
miproyecto/
|-- docs/
|   |-- constitution.md              principios del proyecto (documento del usuario)
|   |-- architecture.md              mapa de arquitectura
|   |-- conventions.md               convenciones del equipo
|   |-- verification.md              comandos de validacion
|   |-- kimi-cli-uso-eficiente.md    guia de uso eficiente de Kimi CLI
|   |-- prd/
|   |   |-- COMO-ESCRIBIR-UN-PRD.md  el metodo para escribir un PRD (guia del arnes)
|   |   |-- PRD-master.md            que se construye y por que (planilla)
|   |   |-- SDD-master.md            como se construye, a nivel proyecto (planilla)
|   |   `-- <parte>/                 PRDs anidados: una carpeta por parte del producto
|   |       |-- PRD-<parte>.md       su historia, sus datos, sus hitos
|   |       `-- <pieza>/PRD-<parte>-<pieza>.md
|   |-- spec-feature-<id>-<slug>.md  spec de la feature (AC-n)
|   |-- plan-feature-<id>-<slug>.md  plan del lider
|   `-- impl-<id>.md / review-<id>.md
`-- harness_process/                 binario, roles, progress/ (estado vivo)
```

### Proyectos que arrancan de cero: `docs/prd/`

`docs/prd/PRD-master.md` y `docs/prd/SDD-master.md` son planillas para completar
antes de cargar la primera feature, y `docs/prd/COMO-ESCRIBIR-UN-PRD.md` es el
metodo con el que se escriben: la historia (antes/despues) primero, el tamano lo
decide el cambio (1 pagina un ajuste, 3-8 una funcionalidad, PRDs anidados para
un producto nuevo) y la regla dura de que el PRD fija la estructura en
pseudo-codigo y explicaciones, **nunca** en codigo final.

El PRD maestro cuenta el producto (historia, objetivos `O-n`/`NO-n`, los datos y
el acuerdo en pseudo-codigo, hitos) y cada `docs/spec-feature-<id>-<slug>.md` es
el **PRD de ese cambio**: nace con Historia, Hoy -> Como va a funcionar, Los
datos que se tocan y Pseudo-codigo (el acuerdo), ademas de los recorridos y los
AC-n. El flujo queda encadenado:

```
docs/prd/PRD-master.md   (el producto entero)
        |  sh harness_cli prd add --name <parte> [--parent <ruta>]
        v
docs/prd/<parte>/PRD-<cadena>.md   (PRD anidado: hitos de esa parte)
        |  sh harness_cli add --name <slug> --service <svc> --acceptance "<criterio>" --prd <ruta>
        v
feature_list.json        (backlog ejecutable, con su PRD de origen)
        |  sh harness_cli start --feature <id>
        v
docs/spec-feature-<id>-<slug>.md  +  docs/plan-feature-<id>-<slug>.md
        |  aprobacion del usuario (Estado: approved)
        v
implementacion -> docs/impl-<id>.md -> docs/review-<id>.md
        |  sh harness_cli close --feature <id> --status done
        v
el PRD de origen: hito marcado `done (fecha)` + linea en su `## Bitacora`
```

### PRDs anidados: el arbol de producto

Un producto grande no entra en un documento. `prd add` parte el PRD en hijos
reales — carpetas bajo `docs/prd/`, con la carpeta llevando el segmento propio y
el archivo la cadena completa, asi cada nombre es unico en el repo:

```
$ sh harness_cli prd add --name cobranza
PRD anidado creado: docs/prd/cobranza/PRD-cobranza.md
Enlazado en docs/prd/PRD-master.md (seccion "PRDs anidados")

$ sh harness_cli prd add --name mora --parent cobranza
PRD anidado creado: docs/prd/cobranza/mora/PRD-cobranza-mora.md

$ sh harness_cli prd tree
PRD-master                  2 hitos | features: 1/2 done
 `-- PRD-cobranza           [!] sin hitos
     `-- PRD-cobranza-mora  1 hito | features: 1/1 done
```

El hijo nace con las mismas 12 secciones del metodo y su `Padre:` declarado, y
queda enlazado en la seccion `## PRDs anidados` del padre. `--prd` acepta la ruta
completa (`cobranza/mora`) o el ultimo segmento si es unico (`mora`); una feature
sin `--prd` cuenta para el maestro.

`harness_check.sh` valida el arbol: PRD fuera de lugar, carpeta sin PRD,
encabezado `Padre:` que no coincide con su ubicacion o feature que apunta a un
PRD inexistente **bloquean**; un PRD sin hitos solo avisa con `[i]`. Sin
`docs/prd/` el bloque entero se omite.

`PRD-master.md`, `SDD-master.md` y todo PRD anidado son documentos **tuyos**: se
siembran (o se crean) una sola vez, ningun reinstall los pisa y **`--reset` no
los borra** (a
diferencia de los docs del arnes, que si se limpian por ser plantillas
regenerables). `COMO-ESCRIBIR-UN-PRD.md`, en cambio, es plantilla del arnes: vive
en la misma carpeta pero se refresca reinstalando (o con `--force`) y entra en
los reset targets, igual que `conventions.md` o `verification.md`.

Los docs base se siembran **solo si faltan** y un reinstall **nunca los
pisa**: si tu equipo ya tiene un `docs/conventions.md`, queda intacto. Para
refrescar una plantilla, borra el archivo y reinstala (o usa `--force`, que por
contrato sobrescribe sin backup). `--reset` limpia solo los docs generados
y conserva la constitution y los artefactos de feature.

Instalaciones anteriores que tengan esos docs en `harness_process/docs/` se
migran solas al reinstalar: se mueven a la raiz si alli no existen, y si ya
existen no se pisa nada (el instalador avisa y deja la copia vieja donde esta).

## Atlassian: Jira y Confluence como reflejo del flujo (feature #15)

El arnes puede dejar rastro de cada movimiento del desarrollo en Jira y publicar
el PRD, el SDD y los specs en Confluence, sin copiar nada a mano. Es **opt-in
por repo** y arranca por lo primero: a que proyecto pertenece este repositorio.

```bash
# Al instalar (lo normal)
sh setup_harness.sh --atlassian-site acme.atlassian.net \
                    --jira-project ADR \
                    --confluence-space SD

# O despues, desde el repo instalado
sh harness_cli atlassian bind --site acme.atlassian.net --jira-project ADR --confluence-space SD
```

Eso escribe `atlassian.json` en la raiz del proyecto (versionable: solo nombra
sitio, proyecto y space, nunca credenciales). **Sin ese archivo no cambia nada**:
mismo flujo, mismos exit codes, sin carpetas nuevas. El arnes no adivina el
proyecto ni el space; si no los sabe, se niega y pide preguntarle al usuario.

Con binding activo, el mapeo es:

| En el arnes | En Jira |
| --- | --- |
| PRD (maestro o anidado) | Epic |
| Feature del backlog | Historia (`Story` por default) |
| AC-n del spec | Subtask `AC-n · <texto>` |
| `start` / `close --status done` | In Progress / Done (y entra al sprint vigente) |
| `advance` y `approve-spec` | Comentarios con la bitacora |
| `close --status blocked` | Flag `Impediment` |

Con token configurado **el envio es automatico** (feature #16): cada transicion
lanza un worker en segundo plano que aplica lo pendiente en Jira y republica los
documentos en Confluence, sin frenar el comando. La primera vez carga lo que ya
existe en el repo (un epic por PRD, una historia por feature, adoptando los
epics que ya existan con ese titulo). Se apaga con `"auto": false` en
`atlassian.json` o `HARNESS_ATLASSIAN_AUTO=0`.

Un bugfix entra como `Bug` si lo cargas con `add --kind bug` (validos:
`feature`, `bug`, `task`).

Cada transicion escribe un intent en `progress/atlassian/outbox/` y hay dos
ejecutores que producen lo mismo (los dos siguen disponibles a mano):

```bash
# (a) con un agente que tenga MCP de Atlassian, sin credenciales
sh harness_cli atlassian drain                          # plan de llamadas, no muta nada
sh harness_cli atlassian ack --intent 0003 --key ADR-42 # el agente devuelve la clave

# (b) con token en .harness.env (HARNESS_ATLASSIAN_EMAIL / HARNESS_ATLASSIAN_TOKEN)
sh harness_cli atlassian apply
sh harness_cli atlassian sprint start --name "Sprint 12" --days 14
sh harness_cli atlassian sprint close
sh harness_cli atlassian publish        # PRD + PRDs anidados + SDD + specs
sh harness_cli atlassian backfill       # carga en Jira lo que ya existe en el repo
```

Los sprints necesitan la ruta (b): el MCP oficial de Atlassian no expone boards
ni sprints. `atlassian status` muestra binding, mapeo, sprint vigente y
pendientes (del token solo dice si esta, nunca su valor). Guia completa en
`docs/atlassian-integracion.md`.

## Paridad entre los dos instaladores (feature #30)

El repo mantiene `setup_harness.sh` y `setup_harness.ps1` en paridad. Esa promesa
**no la verificaba nadie**: once features seguidas cerraron declarando "esta
maquina no tiene pwsh", y mientras tanto los dos se fueron desincronizando en
silencio.

```bash
bash tests/parity_check.sh
```

Compara lo que los dos **declaran** —opciones (traduciendo `--kebab-case` a
`-PascalCase`) y superficies escritas— y falla cuando uno se adelanta al otro.
Corre en cualquier maquina, sin PowerShell.

Las asimetrias legitimas se **declaran con su razon** en el propio script, no se
silencian:

| Opcion | Solo en | Por que |
| --- | --- | --- |
| `--with-subagents`, `--install-graphify`, `--install-antigravity` | `.sh` | afirmativas de un default ya encendido; en ps1 solo existe el `-No*` que lo apaga |
| `--with-postgres` | `.sh` | no-op historico: PostgreSQL es obligatorio desde la #14 y el flag se mantiene para no romper invocaciones viejas |
| `-CargoTargetDir` | `.ps1` | en Unix se logra exportando `CARGO_TARGET_DIR`; en PowerShell el flag ahorra tocar el entorno de la sesion |

`harness_check.sh` lo corre y **avisa** sin bloquear: una opcion desincronizada
no te impide trabajar hoy, pero tiene que verse.

### Lo que aparecio al correr el smoke de PowerShell por primera vez

La deuda no duro once features solo por falta de maquinas: `tests/setup_smoke.ps1`
usaba `-Encoding utf8NoBOM`, **un valor que solo existe en PowerShell 7**, y en
el Windows PowerShell 5.1 que trae el sistema moria antes de la primera
asercion, aunque el archivo declare `#requires -Version 5.1`. Corregido, junto
con un `.Name` sobre un array vacio que `Set-StrictMode -Version Latest` rechaza
en 5.1.

Con eso, lo verificado en 5.1 es que **`setup_harness.ps1` completa una
instalacion root de punta a punta**. El smoke todavia **no pasa entero**: siembra
un `harness.exe` falso (un archivo de texto) y mas adelante le pide al CLI que
ejecute `prd add` de verdad. Necesita sembrar el binario real, como hace el smoke
de sh. Queda dicho para que nadie lea "el smoke ps1 ya corre" como "el smoke ps1
ya pasa".

**Lo que esto NO hace**: no ejecuta el instalador de Windows. Un `.ps1`
estructuralmente paritario puede fallar igual al correr. Verificar eso exige una
maquina con PowerShell, y esta dicho asi en vez de dejarlo creer.

## Features en paralelo: una rama y un worktree por feature (feature #47)

Varias features pueden estar en curso a la vez sin pisarse. Al arrancar una, el
arnes crea su rama GitFlow y su carpeta de trabajo:

```bash
sh harness_cli start --feature 47
#   Rama y worktree creados: feature/47-<slug> en ../<repo>-wt/47-<slug>
#   Trabaja ahi: cd ../<repo>-wt/47-<slug>
```

- La rama es `feature/<id>-<slug>`, o `bugfix/<id>-<slug>` si la feature se
  cargo con `add --kind bug`. Sale de `develop` si existe, y si no de `main`;
  el arnes **nunca crea** la rama base.
- El worktree es hermano del repo, asi que dos features editan archivos
  distintos en disco: es imposible que se pisen.
- El checkout principal **no cambia de rama** en ningun momento.
- `start --sin-worktree` vuelve al modo clasico, y en un directorio sin git el
  arnes avisa y sigue trabajando como siempre.

El **estado del arnes sigue siendo uno solo**: aunque invoques los comandos
desde un worktree, `feature_list.json` y `progress/` se leen y escriben en el
repo principal. Lo que se parte es el estado vivo:

| Archivo | Que es |
| --- | --- |
| `progress/current-<id>.md` | el estado vivo de ESA feature |
| `progress/current.md` | el indice de lo que hay abierto (id, rama, worktree) |
| `progress/.last_autocheck-<id>` | el checkpoint de esa feature |

Por eso cerrar una feature ya no puede tocar el estado de otra.

Dentro de un worktree no hace falta `--feature`: la carpeta dice en que feature
estas trabajando. Fuera de todo worktree y con varias activas, los comandos
piden `--feature <id>`, como siempre.

### Cerrar: la rama destino la elegis vos

```bash
sh harness_cli close --feature 47 --status done --to develop
```

`--to` es obligatorio para `done`: sin el, el arnes se niega y le pide al
agente que te pregunte a que rama va (y lista las disponibles). Con el:

0. **verifica que el merge no pise trabajo tuyo sin commitear** (feature #61),
1. commitea lo que quede en el worktree de la feature,
2. mergea la rama en la destino (`--no-ff`, en un worktree temporal, sin tocar
   tu checkout ni exigirte tener el arbol limpio),
3. publica la rama destino,
4. borra el worktree y **conserva la rama**,
5. y **recien entonces** escribe el estado del cierre (feature #62).

Ningun commit que haga el arnes lleva trailers de IA. Si el merge tiene
conflicto, se aborta y no queda nada a medias: el repo sigue como estaba y el
mensaje dice que resolver. Los cierres `blocked`, `pending` y `superseded` no
integran nada y conservan el worktree para poder retomar.

### El cierre no declara hecho lo que no hizo (feature #62)

El orden importaba mas de lo que parecia. El cierre escribia TODO su estado
antes de integrar: marcaba la feature `done` en el backlog, emitia la transicion
a Jira, anotaba el plan, archivaba el estado vivo y lo borraba, reescribia
`current.md`, dejaba la linea en `history.md`, guardaba la memoria en el hub e
imprimia "Feature #N cerrada". Y **despues** integraba. Si la integracion
fallaba —falta `--to`, colision con trabajo sin commitear, conflicto real— esas
nueve afirmaciones ya estaban hechas sobre un trabajo que no estaba integrado.

Ahora el cierre corre en cuatro fases:

| Fase | Que hace | Si falla |
| --- | --- | --- |
| 0 | gates, `--to`, colisiones | nada escrito, ni siquiera los artefactos |
| 1 | anota el plan y archiva el estado en el `docs/` de la feature | — |
| 2 | commitea, mergea, publica, borra el worktree | **el estado no se toco** |
| 3 | backlog, Jira, `progress/`, `history.md`, memorias, "cerrada", vuelta al PRD | — |

La FASE 1 existe por una razon concreta: esos dos artefactos viven en el `docs/`
del worktree y el merge **borra** ese worktree, asi que escribirlos despues
seria no escribirlos nunca. Por eso son idempotentes — la linea `Cerrado:` del
plan se agrega solo si no esta, buscando por `status` y no por fecha (el sello
de tiempo cambia en cada corrida y nunca coincidiria).

No hay rollback, y es a proposito: un rollback quedaria **parcial** —el intent
ya emitido a Jira y la memoria ya escrita en el hub no se deshacen— y ademas
habria que acordarse de mantenerlo cada vez que el cierre gane un efecto nuevo.
No hay estado que revertir porque no se escribe hasta que el cierre ocurrio.

Consecuencia visible: `Feature #N cerrada` ahora se imprime **despues** de la
salida de `[GitFlow]`, que es el orden real de los hechos.

### El merge no corre en tu checkout (feature #61)

El punto 2 decia "sin tocar tu checkout" con una excepcion que no estaba
escrita: el merge se hacia en un worktree temporal **solo si el destino no era
la rama que tenias abierta**. Y cerrar hacia `main` estando parado en `main` es
el caso mas comun de todos. Ahi el merge corria en tu arbol, y si tocaba un
archivo que tenias sin commitear, el cierre moria con el texto crudo de git
**despues** de haber commiteado el worktree de la feature.

Ahora el merge corre siempre en un worktree temporal `--detach` (git no permite
dos worktrees sobre la misma rama, pero si uno en HEAD detached sobre su
commit), y despues se avanza la rama destino:

| Situacion | Que hace |
| --- | --- |
| Tenes el destino abierto | `git reset --keep <merge>`: mueve la rama y el arbol **conservando** lo que tengas sin commitear |
| No lo tenes abierto | mueve la referencia con guarda del valor viejo |

Queda un caso que no se puede resolver sin decidir por vos: que el merge cambie
un archivo que **vos** tenes modificado sin commitear. El arnes lo detecta
**antes de tocar nada** —antes de commitear la feature, antes de mergear— y se
detiene nombrandolos:

```
[GitFlow] No puedo integrar en main sin pisar trabajo tuyo sin commitear.
    Tenes estos archivos modificados en tu checkout y el merge tambien los cambia:
      docs/prd/PRD-master.md
    NO toque nada: la rama no se movio y la feature no se commiteo.
    Son TUS cambios, asi que elegis vos:
      git add -A && git commit      # los queres conservar
      git stash                     # los queres guardar para despues
      git checkout -- <archivo>     # no te interesan (DESCARTA lo no commiteado)
```

No stashea ni descarta por su cuenta: son tus cambios. Y tampoco avanza la rama
dejando tu arbol atras, que era la otra salida posible — se midio y es
peligrosa: `git status` pasa a mostrar la **reversion** del merge, y un commit
distraido desharia lo recien integrado.

## Verificacion

```bash
bash init.sh
bash harness_status.sh
bash harness_check.sh

# Suites del repo fuente
bash tests/setup_smoke.sh     # instalador (layouts, hooks, build-on-setup)
(cd rust && cargo clippy --all-targets -- -D warnings && cargo test)
```

```powershell
.\tests\setup_smoke.ps1
.\harness_cli.ps1 status
```
