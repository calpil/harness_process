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
