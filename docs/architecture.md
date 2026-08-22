# Arquitectura — harness_process

Este repositorio es el **fuente del instalador** de un arnes multi-LLM (Claude,
Codex, Gemini, Grok, Antigravity y otros CLIs). No tiene microservicios propios;
sus componentes son el binario de ciclo de vida, los instaladores, las
superficies de instruccion y la capa de subagentes.

## Vision general

```
setup_harness.sh / .ps1  ->  superficies + hooks + subagentes + binario `harness`
                                     |
        roles/ (leader, implementer, reviewer)  +  docs/constitution.md (principios)
                                     |
   feature_list.json  --start-->  docs/spec-feature-*.md (draft) + docs/plan-feature-*.md
                                     |
                 usuario aprueba spec (draft -> approved)
                                     |
     advance / close / check-spec / harness_check.sh  (gate require_spec_approved)
```

## Binario Rust (`harness`)

`harness_cli` (POSIX) y `harness_cli.ps1` (PowerShell) despachan **exclusivamente**
al binario nativo `harness` / `harness.exe` (un solo ejecutable multi-OS,
compilado desde `rust/` con `cargo build --release --locked`). No hay fallback
Python desde la feature #2. Version actual: `rust/Cargo.toml` = 0.3.0.

### Modulos nucleo (`rust/src/`)

- `main.rs`: declara los modulos y delega en `cli::run`.
- `cli.rs`: definicion `clap` del `enum Command` (subcomandos de ciclo de vida al
  tope y `graph <cmd>` para el hub) y el dispatch a `commands::*`.
- `exit.rs`: `Exit { code, message }`, equivalente al `SystemExit` de Python.
  `Exit::msg(...)` => code 1 con mensaje a stderr; `Exit::code(2)` => code 2
  silencioso; ausencia de error => code 0.
- `paths.rs`: `HarnessPaths::resolve()` localiza raiz del proyecto, `docs/`,
  `progress/`, `feature_list.json` y honra `HARNESS_REPO_ROOT` / markers de layout.
  `repo_root_from_marker` (unico punto del marker, tambien usado por
  `GraphEnv::resolve`) distingue tres casos: marker `subdir` => el padre, con el
  guardrail de checkout fuente (senales de fuente `templates/harness_cli` +
  `rust/` + padre sin huella de instalacion, o `$HOME` sin
  `HARNESS_ALLOW_HOME_SURFACE=1` => la raiz es el propio checkout, con aviso
  `[i]` a stderr, feature #7); marker AUSENTE => se infiere `subdir` si el padre
  tiene huella y no es `$HOME`, tambien con aviso `[i]` (feature #10); marker con
  otro valor (`root`) => el propio dir, sin inferencia ni aviso.
- `features.rs`: carga/guarda `feature_list.json` y selecciona la feature activa.
- `plan.rs`: plantilla y firma del plan (`plan_signature` = dict
  path/mtime/size/hash), `is_plan_stale`, `plan_staleness_message`, `write_plan`,
  `update_plan_sig`.
- `spec.rs`: gemelo de `plan.rs` para el spec (ver seccion SDD). Su encabezado
  incluye `PRD: <ruta>`, derivado del campo `prd` de la feature (o el maestro).
- `prd.rs`: el arbol de PRDs anidados. La identidad de un PRD es su cadena de
  segmentos (`cobranza/mora`), y de ella salen carpeta y archivo sin registro
  intermedio: el FILESYSTEM es la fuente de verdad y el `Padre:` del encabezado
  es una declaracion contrastable. Expone `scan` (walk acotado a `docs/prd/`,
  ignora lo que esta fuera de lugar), `resolve` (ruta completa, `master`, o
  ultimo segmento si es unico; ambiguo/inexistente => `Exit` con candidatos),
  `child_template` (12 secciones), `link_child` (seccion `## PRDs anidados` del
  padre: la crea al final si falta, agrega fila, nunca duplica),
  `milestone_rows` / `milestone_count` (ignora encabezado, separador y el
  ejemplo `<...>` de la plantilla), `echo_close` (marca el hito y deja
  `## Bitacora`; idempotente, conserva la fecha del primer cierre) y
  `render_tree`.
- `lecciones.rs`: la memoria procedural (`docs/lecciones/<clase>.md`, feature
  #17). Expone `validar_nombre_de_clase` (rechaza nombres de sesion: con
  `feature`/`#`, con prefijo `fix-`/`debug-`/`audit-`/`hotfix-`, con fecha o con
  numeros de 3+ digitos; **sin escape hatch**), `Leccion::parse` (frontmatter
  como lineas crudas, asi que preserva orden y claves desconocidas; el cuerpo va
  verbatim), `registrar_uso` (telemetria `usos`/`ultimo_uso` sin tocar el cuerpo
  ni `ultima_actualizacion`), `scan` (validas + rotas con su motivo, para el gate
  de `harness_check.sh`), `parecidas` (sugerencias ante un typo) y `gate` (el
  gate opcional del cierre). Desde la feature #18 tambien expone el **contrato**:
  `contrato()` extrae de la guia (`COMO-ESCRIBIR-UNA-LECCION.md`) las dos
  secciones que lo forman —el orden de preferencia y la lista de que no
  capturar— y `texto_contrato_de_cierre()` lo arma para stderr, degradando a un
  puntero si la guia falta o esta incompleta. El texto NO vive duplicado en el
  binario: la guia es la unica fuente, y un test de integracion la copia desde
  `templates/` para que renombrar una seccion ponga el build en rojo. Ningun
  camino de este modulo abre conexion al hub.
- `perfil.rs`: el perfil del usuario (`docs/perfil-usuario.md`, feature #19).
  Expone `Perfil` (parseo que preserva el encabezado y las claves del usuario),
  `usados`/`LIMITE` (1500, contando solo las entradas), `Coincidencia`
  (`Ninguna`/`Unica`/`Ambigua`: el caso ambiguo es un estado del dominio con su
  propio remedio, no un `None`), `motivo_inseguro` (escaneo de credenciales y
  Unicode invisible que BLOQUEA antes de escribir), `bloque` (lo que el
  instalador inyecta entre marcadores) y `recolectar` (la evidencia de
  `history.md`, planes y specs para `perfil sugerir`). No abre conexion al hub.
- `buscar.rs`: la busqueda sobre los artefactos del proceso (feature #20).
  `Fuente` es un enum cuyo ORDEN es el orden de relevancia (leccion/perfil >
  spec/plan/adr/prd > impl/review/estado > doc > historia) y cuyo `peso()` tiene
  saltos grandes a proposito, para que la frescura nunca de vuelta el orden entre
  fuentes. `score()` es una funcion pura (peso + encabezado + frase contigua +
  id de feature) y por eso todo el ranking se testea sin tocar el filesystem.
  `corpus()` excluye `bkp/` y los directorios ocultos. No hay indice, no hay
  modelo y no se consulta el hub; es de solo lectura.
- `curador.rs`: el mantenimiento de la biblioteca de lecciones (feature #21).
  `planificar()` calcula el plan de transiciones **leyendo**, y `aplicar()` lo
  ejecuta: esa separacion es la que permite que la pasada por defecto solo
  informe. `respaldar()`/`rollback()` copian el arbol a `bkp/lecciones/<ts>/`
  (honrando `HARNESS_BKP_DIR`) y el rollback respalda ANTES de restaurar, asi que
  deshacer tambien se deshace. Ningun camino borra: archivar es mover a
  `docs/lecciones/archivo/`.
- `journey.rs`: el mapa de lo aprendido (feature #22). `construir()` **solo lee**
  las tres fuentes y devuelve `Mapa` (nodos, enlaces, huecos); el render vive en
  el comando, y esa separacion es lo que hace estructural la promesa de solo
  lectura. `Clase::prioridad()` resuelve el caso de una leccion **declarada y de
  origen** a la vez (sale una vez, como declarada), y una entrada de perfil se
  ancla a la feature mas reciente que cita. El hueco `CierreSinLeccion` solo
  aplica a features cerradas DESPUES de que el proyecto empezo a declarar
  lecciones, comparando timestamps completos y no fechas.
- `verificacion.rs`: AC ejecutables (feature #23). `parsear()` es **pura** —lee
  el texto del spec y devuelve `Verificacion { ac, comando: Option<String> }`—, y
  esa pureza es lo que permite probar la compatibilidad contra los 310 AC reales
  del repo sin ejecutar un solo comando. `ejecutar()` corre UN comando desde la
  raiz con `wait-timeout` (`rules.verify_timeout_segundos`, default 300) y
  clasifica en el enum `Estado` (`Verde` / `Rojo` / `Timeout` / `Manual` /
  `Vacio`); un AC sin comando es `Manual` y **nunca** bloquea. `Vacio` (feature
  #44) es el AC que salio 0 **sin ejecutar ningun caso**: lo decide
  `casos_corridos()`, otra funcion pura, que suma los `N passed` de las lineas
  `test result:` y devuelve `None` —"no opino"— cuando la salida no tiene esa
  forma. Ese `None` es lo que evita que el detector opine sobre un `grep` o un
  `bash`. `rojos_del_reporte()` deriva del enum via `Estado::desde_etiqueta()` en
  vez de comparar contra cadenas sueltas, para que un estado nuevo no se filtre
  por el cierre — que es como la #37 se llevo puesto el emisor de Jira. `gate()` es lo unico que usa el
  cierre: LEE `docs/verify-<id>.md` y no llama a `ejecutar()` — la promesa "cerrar
  no dispara shell" es estructural, no disciplina. El parser saltea los bloques
  ``` porque un spec que documenta la sintaxis no puede terminar ejecutando su
  propio ejemplo (bug encontrado en la primera corrida real).
- `doctor.rs`: diagnostico de la INSTALACION (feature #25). `diagnosticar()` es
  **pura** y devuelve un `Hallazgo` por area (`Estado::{Ok,Falla,Aviso,NoAplica}`);
  solo `Falla` cambia el exit code, asi que un hub caido no puede hacerlo mentir.
  En el checkout fuente del arnes, superficies y hooks dan `NoAplica`: su ausencia
  ahi es lo correcto.
- `rutas.rs`: rutas protegidas (feature #26). `esta_protegida()` es un matcher
  **puro** de globs (`*` un segmento, `**` cualquier profundidad) sobre
  `rules.rutas_protegidas`. Las escrituras del propio binario quedan exentas por
  un registro con mtime que **caduca** en cuanto alguien vuelve a tocar el
  archivo, y por eso `close` y `prd apply` pueden escribir el PRD sin dispararse
  la red de seguridad.
- `documentos.rs`: que el PRD, el SDD y `architecture.md` no queden mintiendo
  (feature #29). `alcance()` deriva los documentos del **arbol real** de PRDs;
  `parsear()` y `planificar()` son puras y devuelven el plan de escritura sin
  tocar disco. El anclaje es por **texto literal** y no por seccion, porque
  cortar en `## ` se tragaria las subsecciones `###`; y la idempotencia sale del
  CONTENIDO, no de una firma, porque un PRD lo comparten N features.
- `consolidacion.rs`: deteccion de lecciones solapadas con un LLM (feature #28),
  la UNICA parte del arnes que usa un modelo. `resolver_backend()` implementa
  override -> CLI -> skip limpio y es pura (el override llega por parametro, no
  del entorno). `detectar` no recibe `&HarnessPaths`: **no puede escribir aunque
  quiera**. Al modelo se le manda solo nombre, descripcion y triggers —nunca el
  cuerpo— y el prompt viaja como item de argv, jamas por `sh -c`, asi que una
  descripcion con backticks no puede inyectar nada. `revisar_paraguas()` exige
  que el paraguas herede todos los triggers de lo que archiva, porque `buscar`
  puntua una leccion activa 100 y una archivada 30.
- `progress.rs`: `current.md` / `history.md` (estado vivo y bitacora).
- `memories.rs`, `graphify.rs`, `graph/` (`commands`, `derive`, `ids`, `store`,
  `tls`): Memory Hub y su integracion con graphify.
- `pycompat.rs`: utilidades de formato de salida (compatibilidad historica).

### Comandos (`rust/src/commands/`)

`add` (con `--prd <ref>` opcional), `next`, `start`, `status`, `advance`,
`close` (que ademas devuelve el cierre al PRD de origen, best-effort: un PRD
ausente NUNCA impide cerrar), `autocheck`, `nudge`, `check_plan`, `check_spec`,
`prd` (`add` / `tree`), `leccion` (`list` / `show` / `nueva` / `usar`),
`perfil` (`show` / `add` / `replace` / `remove` / `sugerir` / `check`),
`buscar` (solo lectura, `--json` / `--todos`),
`lecciones` (`status` / `curar` / `pin` / `unpin` / `archivar` / `restaurar` /
`rollback`), `journey` (solo lectura, `--json`),
`verify` (`--solo AC-n` / `--json`; el UNICO que ejecuta shell, y por eso exige
`Estado: approved` y no lo llama ningun hook). Los gates
duros viven en `advance`, `close` (solo `--status done`: spec aprobado, leccion
declarada y `require_verify_green`), `check_spec` y
`harness_check.sh`; `autocheck` y `nudge` son best-effort y NUNCA bloquean
(tragan errores y re-firman en segundo plano).

`nudge` corre en CADA tool-use (lo invoca el hook `PostToolUse` con matcher
`Bash|Edit|Write|apply_patch`), asi que todo lo que hace es barato y silencioso
salvo cuando tiene algo que decir. Su estado local vive en `progress/`:

- `.last_nudge` — el **nivel** de backoff del aviso "sin feature activa" (su
  mtime sigue siendo el reloj). Intervalo = `min(600 * 2^nivel, 3600)`: escala
  mientras nada cambia y vuelve al piso cuando aparece una feature activa. Un
  archivo vacio (formato previo a la #18) se lee como nivel 0.
- `.nudge_lecciones` — contador `<id-feature>:<n>` del recordatorio de lecciones.
  Cambiar de feature reinicia la cuenta, y sin `docs/lecciones/` el archivo ni
  siquiera se crea.

### Estados de una feature

`pending` / `in_progress` / `done` / `blocked` / `superseded`. El campo es un
`&str` y **no** un enum: catorce lugares lo comparan por igualdad contra un valor
concreto, lo que hace barato agregar uno nuevo (la #37 agrego `superseded` con un
cambio real de una linea) y a la vez significa que un valor invalido escrito a
mano solo lo detecta clap. `superseded` exige `superseded_by`, que se valida
contra el backlog al cerrar.

### Exit codes (estables para hooks)

- `0`: ok (o gate apagado / spec aprobado y fresco).
- `1`: error accionable con mensaje (por ejemplo, sin feature `in_progress`).
- `2`: gate — plan o spec stale (editado por otro LLM sin re-firmar), o spec sin
  aprobar con la regla `require_spec_approved` activa. El stdout distingue el
  caso (plan vs spec).

## Paquete de revision (feature #51)

`rust/src/revision.rs` + `harness revision --feature <id> [--max-lineas N]
[--json]`: junta los AC del spec con su estado en `verify-<id>.md`, las filas de
evidencia de `impl-<id>.md`, los archivos tocados por la feature (incluido lo
sin commitear y lo no indexado, marcado aparte), el diff acotado y las rutas
protegidas tocadas. Es de SOLO LECTURA, declara lo que recorta y reporta su
tamaño en lineas y tokens estimados.

El modelo y el esfuerzo de los subagentes de Claude salen de la tabla de roles
de cada instalador (`CLAUDE_MODEL_*` en `setup_harness.sh`, `$claudeModels` en
`setup_harness.ps1`), no del espejo `.claude/agents/*.md`, que es generado.

## Features en paralelo (feature #47)

`start` le da a cada feature su rama GitFlow (`feature/<id>-<slug>`, o
`bugfix/<id>-<slug>` si se cargo con `add --kind bug`) y su worktree hermano
(`../<repo>-wt/<id>-<slug>`), creado ANTES de escribir el plan y el spec para
que nazcan en esa rama. El checkout principal no cambia de rama nunca.

- `rust/src/git.rs`: ramas, worktrees, merge (en un worktree temporal), push y
  commit sin trailers de IA. Sin repo git, todo degrada a no hacer nada.
- Estado: `feature_list.json` y `progress/` son unicos y del repo principal;
  el estado vivo es `progress/current-<id>.md` por feature y `current.md` pasa a
  ser el indice de lo abierto, con `.last_autocheck-<id>` por feature.
- Documentos: los tres del alcance del cierre (el PRD de origen y sus padres, el
  SDD y `architecture.md`) se resuelven contra el `docs/` de la feature, asi que
  `prd apply` los escribe dentro de su worktree y el merge se los lleva.
- Foco: dentro de un worktree los comandos infieren la feature por la carpeta
  (`feature_por_worktree`); fuera y con varias activas, exigen `--feature`.
- Cierre: `close --status done` exige `--to <rama>` (el arnes no la elige),
  mergea, publica, borra el worktree y conserva la rama. Un conflicto aborta.

## Flujo Spec-Driven Development (SDD)

Inspirado en spec-kit, adaptado y con **layout plano** (specs junto a los planes
en el `docs/` de la RAIZ, sin carpetas `specs/NNN/`).

0. (Proyecto nuevo) El USUARIO completa `docs/prd/PRD-master.md` (que se
   construye y por que, con la historia antes/despues, objetivos O-n/NO-n, los
   datos y el acuerdo en pseudo-codigo) y `docs/prd/SDD-master.md` (como, a nivel
   proyecto), siguiendo el metodo de `docs/prd/COMO-ESCRIBIR-UN-PRD.md`. La
   tabla "Hitos -> features" del PRD se carga al backlog con `harness_cli add`.
   Paso opcional: ningun gate lo exige, y las planillas no las genera ni vigila
   el binario, solo las siembra el instalador.
0b. (Producto grande) `harness_cli prd add --name <parte> [--parent <ruta>]`
   parte el PRD en PRDs ANIDADOS: carpetas reales bajo `docs/prd/`, con la
   carpeta llevando el segmento propio y el archivo la cadena completa
   (`docs/prd/cobranza/mora/PRD-cobranza-mora.md`). El hijo nace con las 12
   secciones del metodo y su `Padre:`, y queda enlazado en la seccion
   `## PRDs anidados` del padre. `prd tree` dibuja el arbol con hitos y estado
   de features; `harness_check.sh` valida su integridad. Detalle en el modulo
   `prd.rs` mas abajo.
1. `harness_cli start --feature <id>` siembra SIEMPRE (aunque la regla este
   apagada) `docs/spec-feature-<id>-<slug>.md` con `Estado: draft` ademas del
   plan, y firma ambos (`last_spec_sig` reusa `plan::plan_signature`).
2. `spec.rs` expone: `spec_path`, `spec_template`, `write_spec` (solo si falta),
   `get_spec_sig` / `update_spec_sig` (clave `last_spec_sig`), `is_spec_stale` /
   `spec_staleness_message` (hash distinto o drift mtime > 1s; falso sin archivo
   o sin firma previa), el enum `SpecState { Missing, Draft, Approved, Other }`
   con `spec_state` (primera linea `Estado:` dentro de las 10 primeras lineas,
   valor trim + case-insensitive), `require_spec_approved(data)` (lee
   `rules.require_spec_approved`, default `false`), `close_requires_spec` (solo
   `done` gatea) y `spec_gate` (mensaje accionable: ruta, estado y accion).
3. El LIDER completa spec y plan (cada item de la Delegacion cita su AC-n) y
   ejecuta el ritual de aprobacion: muestra el spec al USUARIO (chat + editor),
   le pregunta y solo con su SI corre `approve-spec --yes`
   (`commands/approve_spec.rs` + `spec::approve_spec`), que escribe
   `Estado: approved`, inserta el sello `Aprobado: <stamp> por USUARIO ...` y
   re-firma `last_spec_sig` para que la aprobacion no se lea como edicion de
   otro LLM. Sin `--yes`: exit 2. Ningun agente aprueba por su cuenta.
4. Con `require_spec_approved: true`, `advance`, `close --status done` y
   `harness_check.sh` (via `check-spec`) bloquean mientras el spec no este
   aprobado. `check-plan` vigila la frescura de spec y plan (exit 2 si cualquiera
   esta stale). El gate resuelve en <1s, solo filesystem, sin red.
5. `docs/constitution.md` (principios del proyecto) lo siembran ambos
   instaladores solo si falta y nunca lo pisan; specs, planes e implementacion
   deben cumplirlo y el reviewer lo verifica.

## Instaladores y superficies

- `setup_harness.sh` (Bash 3.2+) y `setup_harness.ps1` (PowerShell 5.1/7)
  generan las superficies (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `GROK.md`,
  `LLM.md`), hooks, launchers y la capa de subagentes.
- Los subagentes nativos se ensamblan desde `roles/*.md`: `.claude/agents/*.md`,
  `.codex/agents/*.toml`, `.gemini/agents/*.md` y `.kimi-code/agents/*.md`
  (leader, implementer, reviewer; los espejos Kimi llevan `tools` con allowlist
  por rol, decision usuario 2026-07-28).
- Kimi Code CLI (v0.29.x, feature #8): lee `AGENTS.md` nativamente (verificado
  empiricamente) y sus hooks son SOLO globales, asi que `write_kimi_hooks` /
  `Write-KimiGlobalHooks` escriben un bloque `[[hooks]]` delimitado en
  `${KIMI_CODE_HOME:-$HOME/.kimi-code}/config.toml` — la UNICA escritura fuera
  del proyecto, blindada: solo con Kimi detectado (`--no-kimi` la excluye),
  backup previo en `bkp/`, reemplazo idempotente entre marcadores, validacion
  `kimi doctor` con rollback, y guard por proyecto (`$PWD/bin/harness-hook`;
  no-op silencioso fuera de proyectos con arnes). Eventos SessionStart/
  PostToolUse(`Edit|Write`)/Stop hacia `bin/harness-hook plain <evento>`; sin
  `SessionEnd` (duplicaria el Stop). `--reset` NO toca el bloque global
  (compartido entre proyectos; remocion manual en `UPDATING.md`).
- Los assets versionados (`harness_cli`, `harness_check.sh`, roles,
  `CHECKPOINTS.md`, `UPDATING.md`, `docs/constitution.md`, ...) se copian desde
  `templates/`. Regla de mantenedor: `templates/` y la raiz se mantienen
  espejados; `roles/*.md` es el espejo de `templates/roles/*.md` con el
  placeholder `__HREL__` sustituido por la ruta relativa del arnes.
- Gate de espejo de roles (feature #7; extendido a Kimi en la #8):
  `harness_check.sh` compara el cuerpo embebido de `.claude/agents/*.md`
  (tambien leidos por Grok), `.gemini/agents/*.md` y `.kimi-code/agents/*.md`
  (tras el frontmatter, extractor comun `extract_agent_body`) y
  `.codex/agents/*.toml` (bloque `developer_instructions`) contra
  `roles/<rol>.md`, y `roles/*.md` contra `templates/roles/*.md` modulo
  `__HREL__` (ambas expansiones validas: prefijo del arnes o vacio). Un espejo
  desincronizado bloquea como los demas checks (`HARNESS_CHECK_MODE` degrada
  igual); el check solo reporta y el remedio es re-correr el instalador. Los
  espejos ausentes no fallan (condicionalidad por existencia).
- TODA la documentacion del proceso se instala en el `docs/` de la RAIZ
  (`SURFACE_DIR/docs`): `constitution.md` mas los docs del arnes
  (`architecture.md`, `conventions.md`, `verification.md`,
  `kimi-cli-uso-eficiente.md`, lista `HARNESS_DOCS` / `$script:HarnessDocs`).
  Ninguno esta en los assets regenerables: se siembran solo-si-faltan, no se
  respaldan y un reinstall no los pisa (solo `--force`).
  El arnes ya no crea un `docs/` propio.
- La guia `docs/kimi-cli-uso-eficiente.md` (feature #11) es un `HARNESS_DOCS`
  mas (mismo ciclo: backup en `--reset`, refresh con reinstall o `--force`) y
  las superficies generadas (`write_agent_surface` sh, `Write-AgentSurface`
  ps1) la enlazan en su lista de archivos principales; el `AGENTS.md` raiz de
  este repo la enlaza a mano (dogfooding).
- Dotfiles de contexto para agentes: `.kimiignore` (exclusiones de contexto,
  espejo de `.gitignore`) y `.kimirules` (reglas fijas del proyecto), listas
  `KIMI_DOTFILES` / `$script:KimiDotfiles`. Documentos del USUARIO en la RAIZ:
  se siembran solo-si-faltan, ni `--force` los pisa y NO entran en los reset
  targets (mismo criterio que `PRD_DOCS`).
- Planillas maestras del proyecto: `docs/prd/PRD-master.md` y
  `docs/prd/SDD-master.md` (listas `PRD_DOCS` / `$script:PrdDocs`) se siembran en
  `SURFACE_DIR/docs/prd` solo-si-faltan. Son documentos del USUARIO: ni `--force`
  las pisa y NO figuran en los reset targets, a diferencia de `HARNESS_DOCS`, que
  son plantillas regenerables del arnes. Los PRDs anidados que crea
  `harness_cli prd add` heredan ese regimen: el instalador no los conoce y nadie
  los pisa.
- Guia del metodo: `docs/prd/COMO-ESCRIBIR-UN-PRD.md` es la excepcion en esa
  carpeta: entra en `HARNESS_DOCS` / `$script:HarnessDocs` con la ruta
  `prd/COMO-ESCRIBIR-UN-PRD.md` (primer elemento con subdirectorio; la siembra,
  los reset targets y la migracion crean el directorio destino). Se refresca
  reinstalando o con `--force` y las superficies multi-LLM la enlazan.
- Migracion: `migrate_harness_docs()` (sh) / `Move-HarnessDocsToRoot` (ps1)
  mueven los docs que quedaron en `<harness>/docs/` de instalaciones previas,
  solo cuando faltan en la raiz; si ya existen, avisan y no pisan nada.
- `--reset` borra los docs generados del arnes en ambas ubicaciones (nueva y
  vieja) y conserva la constitution, los dotfiles Kimi y los artefactos de
  feature.

## Memory Hub

El hub usa exclusivamente PostgreSQL; se accede bajo `harness graph <cmd>`
(`mapa`, `impacto`, `vincular`, ...). La conexion se configura por entorno o
`$HARNESS_HUB/.env` (parseado linea a linea). El gate SDD nunca consulta el hub.

## Los tres almacenes de memoria (decision usuario 2026-08-16)

El arnes recuerda en tres lugares distintos, y **no se solapan**. El limite es
explicito porque tres memorias sin frontera terminan diciendo cosas distintas:

| Almacen | Que guarda | Donde vive |
| --- | --- | --- |
| **Memory Hub** | **eventos**: que paso, cuando, en que microservicio, y el grafo de dependencias entre ellos | PostgreSQL (`harness graph <cmd>`) |
| **Lecciones** | **procedimiento**: como se hace esta CLASE de tarea en este proyecto | `docs/lecciones/<clase>.md` (feature #17) |
| **Perfil** | **preferencias**: como quiere trabajar el usuario | `docs/perfil-usuario.md` (feature #19) |

Consecuencias vinculantes de esa decision:

- Las lecciones y el perfil son **archivos versionados** del repositorio: se leen
  con `grep`, se revisan en un PR y viajan con el `git clone`.
- **No agregan tablas ni filas al hub.** El aprendizaje funciona con el hub
  caido: ningun camino de `lecciones.rs` abre conexion.
- Los artefactos de feature (`spec-*`, `plan-*`, `impl-*`, `review-*`) no son un
  cuarto almacen: cuentan **que paso en la feature N**, ordenados por id. Una
  leccion es lo mismo reordenado por clase, que es el orden en que despues se
  busca.

## Layouts

- `subdir` (por defecto): el arnes vive en `harness_process/` dentro de la raiz
  multi-repo y escribe superficies en el directorio padre. Toda la documentacion
  del proceso (constitution, docs del arnes, spec y plan) vive en el `docs/` de
  la RAIZ; el arnes no tiene `docs/` propio.
- `root`: el arnes se instala directamente en la raiz (`SURFACE_DIR == HARNESS_DIR`).
- El marker `.harness_layout` es estado LOCAL de cada instalacion (lo escribe el
  instalador; NO esta versionado en el repo fuente desde la feature #7). La
  resolucion de `REPO_ROOT` es la misma en `harness_check.sh`,
  `harness_status.sh`, `init.sh`, `commit_guard.sh` y `rust/src/paths.rs`:
  overrides (`HARNESS_REPO_ROOT`, variables de agente) > marker `subdir` =>
  padre, salvo el guardrail de checkout fuente (senales de fuente + padre sin
  huella o `$HOME`) que resuelve al propio dir con aviso `[i]`.
- Marker AUSENTE (feature #10): des-versionarlo dejo sin marker a toda
  instalacion que hizo `git pull`, asi que la ausencia ya no significa "layout
  root". Si el padre tiene huella de instalacion (`docs/constitution.md`,
  `CLAUDE.md`, `AGENTS.md`, `.claude/settings.json`) y no es `$HOME`, se infiere
  `subdir` con aviso `[i]` que nombra el remedio (re-correr el instalador
  regenera el marker); sin huella, la raiz sigue siendo el dir del arnes. Las
  cuatro huellas y la guarda de `$HOME` son las MISMAS que usa el guardrail, y
  los scripts siguen siendo read-only: nunca escriben el marker.
- Un marker presente con cualquier valor distinto de `subdir` (`root`) se
  respeta al pie de la letra: la inferencia aplica SOLO cuando el archivo no
  existe.

## Riesgos conocidos

- Exit code 2 sobrecargado (plan vs spec stale): el stdout debe distinguir; no se
  cambia la semantica 0/1/2.
- Instalaciones existentes no reciben la regla `require_spec_approved` (seed
  solo-si-falta): gate apagado por defecto, opt-in documentado en `UPDATING.md`.
- Paridad sh vs ps1: las superficies PowerShell son un resumen conceptual, no
  copia literal; la ejecucion Windows real se valida cuando hay entorno.
- Inferencia de layout por huella (feature #10): un `harness_process/` colocado
  dentro de un directorio que casualmente tenga `CLAUDE.md` o `AGENTS.md`
  resolveria al padre cuando falta el marker. Es el mismo criterio de huella que
  ya usa el guardrail, `HARNESS_REPO_ROOT` sigue como override y el aviso `[i]`
  deja rastro; riesgo aceptado y acotado.
- No correr `setup_harness.sh` en este checkout fuente (escribiria superficies
  en `$HOME`): el binario raiz se refresca con `cargo build` + `cp`. Desde la
  feature #7 los scripts y el binario resuelven el checkout fuente a si mismo
  (guardrail + marker des-versionado), asi que el check y `start` ya no
  producen falsos fallos ni basura en `$HOME`.
