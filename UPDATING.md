# Actualización del Harness Process

El Harness Process se mantiene actualizado **re-ejecutando el instalador** desde la carpeta fuente (`harness_process`). Esto es intencional y explícito.

No existe un comando mágico `harness_cli upgrade` dentro de tus proyectos. La forma correcta de traer mejoras es volver a correr el instalador.

## Por qué funciona así

- Las mejoras al protocolo (por ejemplo: `check-plan` para detectar si otros LLMs actualizaron planes, mejores instrucciones para implementer/reviewer, nuevos comandos, etc.) viven en este repositorio.
- Las superficies (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`) y los subagentes se **generan** desde el instalador.
- Los scripts (`harness_cli`, `harness_check.sh`, roles, etc.) se copian desde `templates/`, y el binario Rust `harness` se compila desde `rust/` durante el setup (cargo requerido).
- Re-correr el instalador asegura que todos los proyectos y todos los agentes (Claude, Gemini, Antigravity, Grok, Codex...) usen la misma versión actualizada del flujo.

## Windows: instalador y comando desde cmd.exe

Ya no hace falta abrir PowerShell a mano para instalar ni para usar el arnes:

```bat
setup_harness.cmd                 :: instala (delega en setup_harness.ps1)
setup_harness.cmd --dry-run       :: las opciones estilo .sh se traducen a -DryRun
harness_cli.cmd status            :: el comando del dia a dia, sin pasar por PowerShell
```

`setup_harness.cmd` **no es un tercer instalador**: encuentra PowerShell (pwsh o
el 5.1 del sistema), saltea la ExecutionPolicy que rechaza un `.ps1` sin firmar
—solo para ese proceso, no toca la configuracion de la maquina— y devuelve el
exit code de verdad. `harness_cli.cmd` va directo a `harness.exe` y traduce el
"binario mas viejo que los scripts" al mismo remedio que la version sh.

Los dos se instalan en tu proyecto junto a `harness_cli` y `harness_cli.ps1`.

## El commit_guard ya no cuelga a quien no es un hook

`commit_guard.sh` arranca leyendo stdin porque su uso normal es COMO hook: el
agente le manda el JSON del evento por la entrada. Cuando lo llamaba
`harness_check.sh` no habia hook ni JSON, y con la entrada abierta se quedaba
esperando un EOF que nadie iba a mandar: en una corrida en segundo plano o en CI
el check entero se colgaba (medido: 18 minutos). Ahora se lo invoca con la
entrada cerrada y el unico dato que ese JSON traia —`stop_hook_active`— viaja
por `HARNESS_STOP_HOOK_ACTIVE`, que el hook exporta despues de leerlo una vez.

En el camino aparecio otra: en Git Bash el guard **no encontraba ningun repo**.
Comparaba `pwd -P` (`/c/Users/...`) contra `git rev-parse --show-toplevel`
(`C:/Users/...`), que nunca dan iguales en Windows, asi que salia en verde sin
haber mirado nada. Ahora se lo pregunta a git (`--show-prefix` vacio), que no
depende de la forma de la ruta.

## El sello de cierre deja de escribirse en el worktree que el cierre borra (feature #71)

Al cerrar, el arnes escribe `docs/estado-feature-<id>-<slug>.md`: el sello del
cierre, con el cuerpo de `progress/current-<id>.md` adentro. Ese cuerpo es lo
unico que documenta la evidencia de la feature, y no tiene otra copia porque
`progress/` esta gitignorado.

Hasta ahora ese archivo se escribia en el `docs/` de la FEATURE —el del
worktree— y el mismo `close` borraba el worktree a continuacion. Sobrevivia solo
por una coincidencia: en un repo donde `docs/` es parte del repo principal, el
merge se lo llevaba antes del borrado. **En un proyecto donde `docs/` es un repo
git aparte, no viajaba y se perdia.** Paso de verdad al cerrar la #124 de
realestate: hubo que reconstruir el sello a mano desde `feature_list.json` y
`history.md`, y el cuerpo literal es irrecuperable.

Ahora:

- El sello se escribe en el `docs/` del repo **PRINCIPAL**, que es donde estan
  todos los anteriores. Ningun cierre lo borra.
- Se escribe **despues de integrar**, no antes: una integracion que falla ya no
  deja un sello afirmando un cierre que no ocurrio.
- El mensaje del cierre nombra **esa** ruta, y avisa que queda sin commitear:

```
Feature #12 cerrada como done. Estado archivado en docs/estado-feature-12-cobranza.md
  (sin commitear: vive en la raiz, no en la rama).
```

**Lo que cambia para vos:** el sello ya no viaja en el merge. Queda como archivo
sin commitear en la raiz, igual que la bitacora del PRD, y lo commiteas vos
cuando corresponda. A cambio, existe siempre — que es lo que antes no pasaba.

**Nada se migra.** Los sellos ya escritos no se mueven, no se reescriben y no se
borran.

## El paralelo aisla, y el cierre no publica solo (feature #72)

Un diagnostico del 2026-09-04 encontro tres features (`#98`, `#122`, `#126`)
marcadas `in_progress` sin rama ni worktree, escribiendo las tres en el mismo
checkout, y un incidente verificado: se publico el arreglo de una feature y con
el se fue un commit de otra que se habia acordado dejar local, porque era su
padre. Cuatro cambios de comportamiento salen de ahi.

**1. Un arranque que no consigue aislamiento ya no arranca.** Antes `start`
marcaba `in_progress` primero y despues intentaba el worktree; si git fallaba, lo
imprimia con `[i]` y seguia. Ahora el aislamiento se resuelve ANTES de escribir
nada y un rechazo deja el backlog intacto. En concreto:

- `--sin-worktree` solo vale si NO hay otra feature abierta. Con otra abierta,
  se rechaza y se dice cual.
- Una feature abierta sin worktree ocupa el checkout compartido: mientras siga
  abierta, no arranca ninguna otra.
- Sin repo git pasa lo mismo: **una feature a la vez**. Esto REVOCA, para el caso
  sin git, la promesa de la feature #47 de tener varias en paralelo. Es
  deliberado: sin worktrees no hay forma de atribuir un cambio a una feature.
- Un `worktree` declarado en el backlog cuya carpeta ya no existe NO cuenta como
  aislamiento.

**2. Un `docs/` que es otro repo git tiene su propio worktree** (`../docs-wt/<id>-<slug>`).
Antes quedaba vacio dentro del worktree principal, y ese directorio vacio fue la
excusa con la que una sesion arranco `--sin-worktree`. Al cerrar, sus artefactos
se commitean y el arnes **no** los integra ni borra ese worktree: mergear en el
repo de documentacion del usuario es decision suya, y el cierre te da el comando.

**3. El cierre muestra el rango completo y ya no publica solo.**
`close --status done --to <rama>` imprime origen, destino y todos los commits que
el merge se lleva, y se NIEGA si alguno pertenece tambien a la rama de otra
feature. La publicacion pasa a ser explicita:

```sh
harness_cli close --feature 12 --status done --to develop              # integra LOCAL
harness_cli close --feature 12 --status done --to develop --publicar   # ademas hace push
```

Sin `--publicar` el cierre deja el `git push` escrito para que lo corras vos.
Ademas, dos cierres del arnes hacia el mismo destino se serializan con un
candado: no corren a la vez.

**4. El Stop deja de reclamarte lo de otra sesion.** `commit_guard.sh` recorria
todos los repos hermanos sin atribuir nada, y —peor— nunca miraba el worktree
donde la sesion trabaja. Ahora, si corres desde el worktree de una feature de
este proyecto, revisa ESE worktree; lo que hay en los checkouts compartidos se
informa una vez y no bloquea. El barrido completo sigue disponible:

```sh
HARNESS_COMMIT_GUARD_SCOPE=global sh harness_process/commit_guard.sh
```

**Y una para la delegacion.** Si delegas en paralelo, declara la cuenta antes y
registra CADA resultado, incluidos los que fallaron:

```sh
harness_cli revision --feature 12 --esperar-tareas 4
harness_cli revision --feature 12 --tarea rev-a --tarea-ac AC-1 --tarea-estado ok
harness_cli revision --feature 12 --tarea rev-b --tarea-ac AC-2 --tarea-estado fallo
```

Solo `ok` cubre; `fallo`, `cancelada`, `sin-resultado` y `sin-evidencia` bloquean
`approved` y `done` hasta que esa verificacion quede cubierta. Vale el ULTIMO
estado de cada tarea, asi que una que fallo y despues se cubre desbloquea sin
perder su historia. Borrar las lineas de las fallidas no completa la cobertura:
para eso esta la cuenta declarada antes. El motivo es concreto — un workflow de
revision registro 74 arranques para 14 tareas con 12 fallidas, y su script
filtro los nulos: el resultado que llego decia "sin hallazgos".

**Antes de actualizar**, `harness_cli status` te lista las features abiertas sin
worktree, que son las que esta regla afecta. El arnes no las migra solo: no mueve
commits, no cambia ramas, no borra worktrees y no para procesos.

**Lo que esto NO hace.** No es un sandbox: un `git push` a mano desde otra
terminal no pasa por ningun hook. Y no controla los reintentos internos del
runtime del agente — la preferencia de tamano de workflow es un consejo
documentado del proveedor, no un limite configurable, y el arnes no la presenta
como garantia.

## El paquete de contexto (feature #56)

Antes de leer el repo, el agente pide el material ya juntado:

```bash
sh harness_cli contexto --feature <id>
sh harness_cli contexto --tema "motor de reajuste"     # sin feature todavia
sh harness_cli contexto --feature <id> --max-lineas 150 # mas apretado
sh harness_cli contexto --feature <id> --con-grafo      # ademas corre graphify query
```

Trae el mapa de arquitectura **siguiendo el puntero** si `docs/architecture.md`
apunta a otro archivo, si ese mapa **cubre el tema**, el impacto del hub (con
limite de 5s: si no contesta es un hueco, no un error), la edad del grafo de
graphify (vencido a los 7 dias), la historia acotada, las lecciones que aplican
y las features anteriores del mismo servicio. Declara su tamaño en lineas y
tokens, y lista sus huecos con el comando que consigue cada uno.

Lo mas importante que hace es avisar cuando **no** hay material:

```
EL MAPA NO CUBRE ESTE TEMA: 'docs/architecture.md' no menciona ninguno de estos
terminos: motor, reajuste.
```

Ese aviso sale ademas solo en cada `start`, sin pedirlo. Existe porque un mapeo
exploratorio de cuatro agentes costo 693.6k tokens para descubrir exactamente
eso. El comando es de SOLO LECTURA: no escribe archivos ni toca estado.

## El veredicto del reviewer es un gate (feature #64)

`close --status done` suma un quinto gate: `require_review`. Con la regla activa,
cerrar como `done` exige `docs/review-<id>.md` con veredicto `approved`; sin ella
—el default— el cierre se comporta exactamente como antes.

El sello lo escribe **solo el binario**:

```bash
sh harness_cli revision --feature <id>                        # el paquete de revisión (solo lectura)
sh harness_cli revision --feature <id> --veredicto approved   # registra el veredicto
```

`--veredicto` estampa en el review, justo debajo del título, la línea canónica:

```
Revisado: approved · 2026-08-28T12:00:00Z · estampado por `harness revision --veredicto`
```

y deja `revision feature #<id> veredicto=<v>` en `progress/history.md`. Los
veredictos son los tres de `roles/reviewer.md` —`approved`, `changes_requested`,
`blocked`—; solo `approved` deja cerrar.

**Un `Veredicto: approved` tipeado a mano NO cuenta.** No es una precaución
teórica: el gate no parsea prosa porque la prosa no se deja parsear. De los 40
reviews que ya existen en este repo, 7 no son parseables, y `docs/review-3.md:3`
dice *"Veredicto: approved (implementación) — cierre BLOQUEADO"*. Un gate que
buscara `approved` en el texto aprobaría un review que dice que el cierre está
bloqueado. Por eso lo único que lee es la línea que estampó el binario.

Y estampar no es gratis: `revision --veredicto` **se niega, sin escribir nada**, si
el review no responde por **cada AC-n que declara el spec** con una fila que lo
nombre y cite `archivo:linea`. Una fila sin cita es una afirmación, y una
afirmación es justo lo que un review de cinco segundos sabe escribir.

Lo que el gate **no** hace: no compara la fecha del review contra la de
`docs/impl-<id>.md`. Eso sería un deadlock en el ciclo normal —el reviewer pide
cambios, el implementer corrige, el impl queda más nuevo, el gate bloquea para
siempre— con una única salida barata, `touch`, que es justo el hábito que no hay
que entrenar.

### Las reglas nuevas ahora llegan a los proyectos ya instalados

Hasta acá los instaladores sembraban `feature_list.json` **solo si faltaba** y no
volvían a mirarlo nunca. Consecuencia: una regla nueva no llegaba jamás a un
proyecto ya instalado, y por eso cada gate anterior se documentó como "editá el
JSON a mano". Desde esta versión `setup_harness.sh` (`migrate_rules`) y
`setup_harness.ps1` (`Migrate-HarnessRules`) **agregan las claves de `rules` que
falten**, tomándolas del molde.

El contrato es estrecho a propósito, porque tocan un archivo del usuario:

- **Jamás pisan un valor existente.** Si apagaste una regla, sigue apagada.
- **Solo agregan**: no sacan claves —ni las inertes—, no tocan `features` ni
  ninguna otra parte del archivo, y un `feature_list.json` ilegible no se toca.
- **Backup antes de escribir**, y si no hay nada que agregar no se escribe nada.
- `--dry-run` / `-DryRun` dice qué reglas agregaría sin tocar el archivo. Sin
  `python3`, la versión sh avisa con el remedio en vez de saltearlo en silencio.

Efecto concreto al actualizar: un proyecto que ya tenía el arnés recibe
`require_review: true` —el molde la trae en `true`— y el gate empieza a correr en
el próximo `close --status done`. Si todavía no lo querés, ponela en `false`: la
migración no vuelve a tocarla.

### La deuda de reviews y su corte

La regla aplica **de la #64 en adelante**. En este repo hay 55 features `done` y
solo 40 tienen su `docs/review-<id>.md`; las 15 que no son **#38-43, #53-55, #57
y #59-63**.

El corte es cronológico y limpio: el último cierre **con** review es el de la #46
(2026-08-22) y el primero **sin** review es el de la #57 (2026-08-26). No hay
interleaving en el medio, y no hay una sola línea en `progress/history.md`
decidiendo saltearlo: fue una práctica que se dejó de hacer, no una excepción que
alguien evaluó.

**Esos 15 no se reconstruyen.** Un review escrito después de que el código se
integró y funciona no intenta romper nada: rellena el casillero. Y
`roles/reviewer.md:6` define el rol como exactamente lo contrario —*"tu trabajo es
intentar ROMPER, no confirmar"*—, así que 15 reviews retroactivos serían 15
documentos afirmando que alguien revisó cuando nadie revisó: la misma promesa
vacía que la regla vino a cerrar, ahora con archivos de respaldo.

## Cómo actualizar

Desde la carpeta del `harness_process` (la fuente):

```bash
# Actualización normal (recomendada)
./setup_harness.sh

# O reinstalación limpia (borra superficies anteriores y las regenera)
./setup_harness.sh --reset
```

En Windows se mantiene un instalador paralelo:

```powershell
.\setup_harness.ps1
.\setup_harness.ps1 -Reset
```

`setup_harness.ps1` configura Cargo para la sesion desde `PATH`,
`$env:CARGO_HOME\bin` o `$HOME\.cargo\bin`, compila `harness.exe` con
`cargo build --release --locked` y despliega `harness_cli.ps1`. El instalador
Bash sigue disponible y tambien copia el shim PowerShell.

El instalador hace backups automáticos de los archivos que reemplaza (en `bkp/`) a menos que uses `--force`.

## Cuándo actualizar

- Después de hacer `git pull` o `git fetch` en la carpeta `harness_process`.
- Cuando `harness_status.sh` o las superficies muestren recordatorios de nuevas funcionalidades.
- Periódicamente para beneficiarte de mejoras en el manejo multi-LLM (detección de planes actualizados por otros agentes, mejores checkpoints, etc.).
- Cuando agregues un nuevo LLM al equipo (para asegurarte de que tenga los últimos roles y hooks).

## Qué se actualiza

- Superficies de instrucciones (CLAUDE.md, AGENTS.md, etc.)
- Subagentes nativos (`.claude/agents/`, `.codex/agents/`, `.gemini/agents/`)
- Scripts del arnés (`harness_cli`, `harness_check.sh`, `harness_status.sh`, roles, etc.)
- El binario Rust `harness` (recompilado con cargo, requerido; sin cargo/rustup `harness_cli` falla pidiendo instalarlo y re-correr el setup)
- Hooks y launchers
- Documentación interna como `CHECKPOINTS.md` y este mismo `UPDATING.md`
- `docs/constitution.md` (sembrada solo si falta; nunca pisa la del usuario)
- `docs/architecture.md`, `docs/conventions.md`, `docs/verification.md`,
  `docs/kimi-cli-uso-eficiente.md` y `docs/prd/COMO-ESCRIBIR-UN-PRD.md` en el
  `docs/` de la **raíz** del proyecto (mismo criterio: solo si faltan)
- `docs/prd/PRD-master.md` y `docs/prd/SDD-master.md` (planillas maestras del
  proyecto; solo si faltan, y `--reset` no las borra)
- `.kimiignore` y `.kimirules` en la **raíz** del proyecto (exclusiones de
  contexto y reglas fijas para agentes; solo si faltan, y ni `--force` ni
  `--reset` los toca)
- El subcomando `harness_cli check-spec` y el gate de spec aprobado
  (`require_spec_approved`) en `advance`, `close --status done` y
  `harness_check.sh`
- El subcomando `harness_cli approve-spec --yes`, que registra la aprobación del
  usuario (sello + re-firma del spec)
- El gate de espejo de roles en `harness_check.sh` (compara el cuerpo embebido de
  `.claude/agents/*.md`, `.gemini/agents/*.md`, `.codex/agents/*.toml` y
  `.kimi-code/agents/*.md` contra `roles/*.md`) y la resolución de raíz robusta
  ante el checkout fuente
- El soporte de Kimi Code CLI: subagentes `.kimi-code/agents/*.md`, launcher
  `bin/harness-kimi` y el bloque de hooks globales en
  `KIMI_CODE_HOME/config.toml` (solo si se detecta Kimi; `--no-kimi` lo excluye)

## Spec-Driven Development (opt-in en instalaciones existentes)

Desde esta versión, `setup_harness.sh` / `setup_harness.ps1` siembran
`docs/constitution.md` (solo si falta) y `harness_cli` incorpora el subcomando
`check-spec` y el gate de spec aprobado. En instalaciones **nuevas** la regla
llega activada desde `templates/feature_list.json`.

En instalaciones **existentes** el gate quedaba **apagado por defecto**: el
`feature_list.json` de cada proyecto no se versiona ni se pisa, y el seed era
solo-si-falta, así que re-correr el instalador NO agregaba la regla. Para activar
el gate había que editar a mano el `feature_list.json` del proyecto y agregar la
regla a `rules`:

```json
{
  "rules": {
    "require_spec_approved": true
  }
}
```

**Desde la feature #64 el instalador migra las reglas** y agrega las claves del
molde que falten (ver *"Las reglas nuevas ahora llegan a los proyectos ya
instalados"*), así que `require_spec_approved` llega sola al re-correrlo. Si no la
querés activa, declarala en `false`: la migración nunca pisa un valor existente.

Con la regla activa, `advance`, `close --status done` y `harness_check.sh`
exigen un spec `docs/spec-feature-<id>-<slug>.md` con `Estado: approved`. Sin la
regla, el flujo sigue como antes (gate apagado, compatibilidad total).

### Aprobación interactiva del spec (`approve-spec`)

La aprobación dejó de ser una edición manual del Markdown. El agente ejecuta el
**ritual de aprobación**: lee el spec, se lo **muestra** al usuario (contenido en
el chat y abierto en su editor), le **pregunta** si lo aprueba y solo con su
**sí** explícito lo **registra**:

```bash
sh harness_cli approve-spec --yes --nota "aprobado en el chat"
```

El comando escribe `Estado: approved`, inserta el sello
`Aprobado: <fecha> por USUARIO (confirmacion explicita)` y **re-firma**
`last_spec_sig`. Esa re-firma corrige un problema real del flujo manual: al
editar el spec a mano cambiaba su hash y `check-spec` reportaba la aprobación del
propio usuario como *"SPEC ACTUALIZADO POR OTRO LLM"*, obligando a un `advance`
para resincronizar. Si ya aprobaste un spec a mano, correr `approve-spec --yes`
lo re-firma y limpia esa falsa alarma sin duplicar el sello (es idempotente).

La decisión sigue siendo **exclusivamente del usuario**: sin `--yes` el comando
se niega (exit 2) y ningún agente puede aprobar por su cuenta. Exit codes: `0`
aprobado o ya aprobado, `1` sin feature `in_progress`, `2` sin confirmación o
spec ausente.

## Docs del arnés en el `docs/` de la raíz (migración automática)

Desde esta versión, `docs/architecture.md`, `docs/conventions.md` y
`docs/verification.md` se instalan en el `docs/` de la **raíz del proyecto**,
junto a `docs/constitution.md`, los specs y los planes. Antes vivían en
`<proyecto>/harness_process/docs/`, partiendo la documentación en dos lugares.

**No hay que hacer nada manualmente.** Al re-correr el instalador
(`setup_harness.sh` o `setup_harness.ps1`):

- Si el doc está en `harness_process/docs/` y **no** existe en el `docs/` de la
  raíz, se **mueve** (conserva tu contenido, no se regenera desde la plantilla) y
  el instalador lo informa con una línea `Migrado al docs/ de la raiz: ...`.
- Si **ya existe** en la raíz, no se pisa nada: se conserva tu archivo, la copia
  vieja queda donde está y el instalador avisa con un `WARN`.
- Si `harness_process/docs/` queda vacío tras la migración, se elimina.

Además cambió el criterio de refresco: estos docs del arnés ahora se siembran
**solo si faltan** (igual que la constitution), porque comparten carpeta con la
documentación del equipo y un `docs/conventions.md` propio no debe perderse en un
reinstall. Para refrescar una plantilla: borra el archivo y reinstala, o usa
`--force` (que por contrato sobrescribe **sin** backup).

`--reset` sigue limpiando solo lo generado —los docs del arnés, en su ubicación
nueva y en la vieja— y conserva la constitution y los artefactos de feature
(`spec-*`, `plan-*`, `impl-*`, `review-*`).

## PRDs anidados: el árbol de producto (`prd add` / `prd tree`)

Desde esta versión la promesa de "un PRD puede contener otros PRDs" dejó de ser
sólo prosa de la guía: el árbol es real y el arnés lo crea, lo dibuja y lo
valida.

**Layout.** La identidad de un PRD es su **cadena de segmentos**. La carpeta
lleva el segmento propio y el archivo la cadena completa, así cada nombre de
archivo es único en todo el repo:

```
docs/prd/
  PRD-master.md                        el producto entero
  cobranza/
    PRD-cobranza.md                    Padre: master
    mora/
      PRD-cobranza-mora.md             Padre: cobranza
```

**Comandos nuevos:**

- `sh harness_cli prd add --name <parte> [--parent <ruta>]` crea el PRD hijo con
  las mismas 12 secciones del método y su `Padre:` declarado, y lo enlaza en la
  sección `## PRDs anidados` del padre (la crea al final si falta; nunca
  duplica una fila ni reordena el documento). Se niega si el padre no existe
  (listando los PRDs disponibles), si el destino ya existe o si el nombre no
  deja ningún carácter utilizable.
- `sh harness_cli prd tree [--prd <ref>]` dibuja el árbol con los hitos que
  declara cada PRD y cuántas de sus features están `done`.
- `sh harness_cli add ... --prd <ref>` guarda el PRD de origen en la feature.
  `<ref>` acepta la ruta completa (`cobranza/mora`), `master`, o el último
  segmento si es único en el árbol (`mora`); ambigua o inexistente falla con
  exit 1 y lista los candidatos.

**La cadena, ahora en los dos sentidos.** El spec que genera `start` cita su PRD
de origen en el encabezado (`PRD: docs/prd/...`; sin `--prd`, el maestro), y al
cerrar la feature con `close --status done` el arnés **vuelve al PRD**: marca la
fila del hito (`Estado` → `done (YYYY-MM-DD)`) y agrega una línea a su
`## Bitacora` con la feature, su spec y su `docs/impl-<id>.md`. Es idempotente
(re-cerrar no duplica ni reescribe la fecha del primer cierre) y **best-effort**:
un PRD ausente o ilegible jamás impide cerrar.

**Dónde y cuándo escribe (feature #60).** El PRD es un documento **raíz y
compartido**: la vuelta al PRD se escribe en el `docs/prd/` del checkout
principal, y **después** de que la integración salió bien. Las dos cosas
importan:

- *En la raíz*, porque el log de cierre no pertenece a ninguna rama. Cuando se
  escribía dentro del `docs/` del worktree, dos features cerrando en paralelo
  apendeaban al final de la misma sección y el merge conflictuaba; la línea de
  la rama desaparecía en la resolución. En el repo del arnés **7 de 18** cierres
  perdieron así su bitácora, y hubo que transcribirlas a mano.
- *Después de integrar*, porque un hito marcado afirma que el trabajo está en la
  rama destino. Si el merge falla o falta `--to`, no se marca nada.

El PRD queda modificado **sin commitear** en el checkout principal, como el resto
de los documentos que el arnés toca: commitealo cuando corresponda.

**Los punteros se verifican antes de escribirse.** Cada ruta que entra en la
bitácora tiene que ser relativa a la raíz y abrir un archivo que existe. La que
no cumple **no se escribe** y se dice por qué (`[i] sin puntero impl: ... (el
archivo no existe)`). Antes se escribía la ruta al worktree — que el propio
cierre borra segundos después — y un `docs/impl-<id>.md` fijo que nadie
garantiza que exista.

**El pendiente durable: `prd doctor`.**

```sh
sh harness_cli prd doctor            # informe: NO escribe nada, sale 2 si hay algo
sh harness_cli prd doctor --reparar  # aplica los arreglos
```

Contrasta el backlog con los PRD y encuentra dos cosas: **punteros que no
resuelven** (los reescribe al archivo que sí existe, o los quita antes que
mentir) y **features cerradas como `done` que no están en la bitácora de su
PRD** (las agrega con la fecha de su cierre real y marca su hito). No depende de
que el cierre haya anotado el pendiente en ningún lado: una feature `done` que
no está en su PRD **es** el hallazgo, aunque la pérdida sea de hace meses.
`harness_check.sh` lo reporta con `[i]` y **no bloquea** por ello: un PRD
desactualizado no impide trabajar hoy.

El **cuerpo** del PRD (historia, datos, pseudo-código) no lo reescribe nadie: si
lo implementado difiere de lo que promete el documento, actualizarlo es tuyo.

**Gate en `harness_check.sh`.** Si existe `docs/prd/`, el check valida el árbol:
un `PRD-*.md` fuera de lugar, una carpeta sin su PRD, un encabezado `Padre:` que
no coincide con la ubicación real o una feature que apunta a un PRD inexistente
**suman fallo** (exit 2 en modo `block`); un PRD sin hitos sólo avisa con `[i]`.
Sin `docs/prd/` el bloque entero se omite.

**Compatibilidad.** El campo `prd` de `feature_list.json` es **opcional**: las
features que ya tenías siguen válidas y cuentan para el maestro. Los PRDs
anidados heredan el régimen de las planillas maestras: son documentos del
USUARIO, ningún reinstall ni `--force` los pisa y `--reset` no los borra.

## Planillas maestras PRD y SDD (`docs/prd/`)

Desde esta versión el instalador siembra dos planillas para proyectos que
arrancan de cero, en el `docs/prd/` de la **raíz del proyecto**:

- `docs/prd/COMO-ESCRIBIR-UN-PRD.md` — **el método** para escribir un PRD: qué
  contiene y qué nunca contiene, la historia (antes/después) como corazón del
  documento, el tamaño que decide el cambio (1 página un ajuste, 3-8 una
  funcionalidad, 10+ una grande, PRDs anidados para un producto nuevo), la
  anatomía sección por sección con un ejemplo, y la regla dura: el PRD fija la
  estructura en pseudo-código y explicaciones, **nunca** en código final.
  A diferencia de las otras dos, **es plantilla del arnés**: se refresca
  reinstalando (o con `--force`) y entra en los reset targets.
- `docs/prd/PRD-master.md` — qué se construye y por qué: resumen hoy → después,
  **la historia** (antes/después, con nombre y momento), objetivos y no-objetivos
  numerados (`O1`, `NO1`), usuarios y jobs-to-be-done, métricas de éxito, el flujo
  dibujado dos veces (hoy → cómo va a funcionar), **los datos** (disparador,
  interruptor, candado) y el **pseudo-código del acuerdo** a nivel producto,
  restricciones, tabla **Hitos → features** (cada fila se carga al backlog con
  `harness_cli add`), riesgos y decisiones abiertas, más las secciones
  `## PRDs anidados` (donde `prd add` engancha a los hijos) y `## Bitacora`
  (donde `close` deja el rastro de cada hito cerrado).
- `docs/prd/SDD-master.md` — cómo se construye, a nivel proyecto: arquitectura
  objetivo, stack, contratos entre componentes, decisiones técnicas tipo ADR,
  datos, no funcionales, estrategia de verificación, riesgos y decisiones
  abiertas. Es distinto de `docs/architecture.md`, que mapea lo que **ya** existe.

Garantías (mismo criterio que `docs/constitution.md`):

- Se siembran **solo si faltan**. Un reinstall nunca las pisa, y **ni `--force`**
  las sobrescribe: lo que hay escrito ahí es tu proyecto, no una plantilla
  refrescable.
- **`--reset` no las borra.** No son superficie generada del arnés. Los docs
  del arnés (`architecture.md`, `conventions.md`, `verification.md`,
  `kimi-cli-uso-eficiente.md` y la guía `prd/COMO-ESCRIBIR-UN-PRD.md`) sí se
  limpian con `--reset`; el PRD y el SDD del proyecto no.

El mismo método baja un nivel: cada `docs/spec-feature-<id>-<slug>.md` que genera
`harness_cli start` es **el PRD de ese cambio** y nace con las secciones
`La historia (antes -> despues)`, `Hoy -> Como va a funcionar`,
`Los datos que se tocan` y `Pseudo-codigo (el acuerdo)`, además de los recorridos
priorizados y los AC-n. Los specs ya existentes no se reescriben: la plantilla
nueva rige para los que se generen de aquí en adelante.

En instalaciones existentes aparecen al re-correr el instalador, sin tocar nada
de lo que ya tengas.

## Marker `.harness_layout` des-versionado + gate de espejo de roles

Desde esta versión (feature #7, decisión del usuario 2026-07-28):

- **`.harness_layout` ya no está versionado** en el repo fuente. Es estado
  **local** de instalación (no código): versionado con valor `subdir` hacía que
  todo clon naciera declarando "mi raíz es mi padre", y el checkout fuente
  resolvía su raíz a `$HOME` (falsos fallos en `harness_check.sh` y basura en
  `$HOME/docs`). El instalador lo escribe en cada instalación, como siempre.
- **Migración en instalaciones subdir existentes**: al hacer `git pull` en tu
  `harness_process`, Git elimina el `.harness_layout` local (dejó de estar
  tracked). **Ya no hace falta re-correr el instalador para que la raíz vuelva a
  ser tu proyecto**: desde la feature #10 el layout `subdir` se infiere de la
  huella del padre (ver la sección siguiente). Re-correr el instalador
  (`./setup_harness.sh` / `.\setup_harness.ps1`) sigue siendo el flujo canónico
  tras un pull y es lo que **regenera el marker** —y con él, el aviso `[i]`
  desaparece—, pero ya no es un requisito para no perder la raíz.
- **Guardrail de checkout fuente**: aunque el marker diga `subdir`, si el
  directorio tiene señales de fuente (`templates/harness_cli` + `rust/`) y el
  padre no tiene huella de instalación (`docs/constitution.md`, `CLAUDE.md`,
  `AGENTS.md`, `.claude/settings.json`) o el padre es `$HOME` (sin
  `HARNESS_ALLOW_HOME_SURFACE=1`), los scripts y el binario resuelven la raíz al
  **propio checkout** con un aviso informativo `[i]`. Las instalaciones subdir
  legítimas (padre con huella) no cambian, y `HARNESS_REPO_ROOT` /
  `CLAUDE_PROJECT_DIR` (y variables de agente) siguen mandando sobre cualquier
  detección.
- **Gate de espejo de roles en `harness_check.sh`**: `roles/*.md` es la fuente
  única; el check ahora compara el cuerpo embebido de `.claude/agents/*.md`
  (también leídos por Grok), `.gemini/agents/*.md` y `.codex/agents/*.toml`
  contra `roles/<rol>.md`, y `roles/*.md` contra `templates/roles/*.md` (módulo
  `__HREL__`). Un espejo desincronizado **bloquea** como los demás checks
  (`HARNESS_CHECK_MODE=warn|off` degradan igual que siempre). El check solo
  **reporta**: el remedio es re-correr el instalador (o propagar el cambio a
  `roles/` si lo que editaste fue el espejo).

## Layout `subdir` inferido cuando falta `.harness_layout`

Desde esta versión (feature #10, decisión del usuario 2026-07-29). Corrige el
efecto colateral de des-versionar el marker: el commit que lo sacó de git graba
`D .harness_layout`, así que **cualquier instalación que hizo `git pull` se
quedó sin marker** y pasaba a tratar `harness_process/` como raíz, en silencio
(specs, planes y veredictos escritos dentro del arnés en vez de en el `docs/` de
tu proyecto).

Cómo se resuelve ahora la raíz (misma regla en los cuatro scripts —
`harness_check.sh`, `harness_status.sh`, `init.sh`, `commit_guard.sh`— y en el
binario Rust), en orden:

1. `HARNESS_REPO_ROOT` o una variable de agente (`CLAUDE_PROJECT_DIR`,
   `CODEX_PROJECT_DIR`, ...) siguen mandando sobre todo lo demás, sin avisos.
2. **Marker `subdir`**: la raíz es el padre, con el guardrail de checkout fuente
   de la feature #7 intacto.
3. **Marker ausente**: si el padre tiene huella de instalación
   (`docs/constitution.md`, `CLAUDE.md`, `AGENTS.md` o `.claude/settings.json`) y
   no es `$HOME` (sin `HARNESS_ALLOW_HOME_SURFACE=1`), se **infiere layout
   `subdir`** y la raíz es el padre, con un aviso informativo:

   ```
   [i] .harness_layout ausente: layout subdir inferido por la huella de
       instalacion del padre: REPO_ROOT=<tu proyecto>. Re-corre el instalador
       (setup_harness.sh / setup_harness.ps1) para regenerar el marker.
   ```

   No es un fallo: los exit codes no cambian. Sin huella en el padre no se
   infiere nada (la raíz es el directorio del arnés, como antes).
4. **Marker con cualquier otro valor** (`root`): se respeta al pie de la letra.
   La inferencia aplica **solo** cuando el archivo NO existe, así que una
   instalación en layout root nunca cambia de raíz.

Qué tienes que hacer: **nada**. Tu instalación se repara sola al actualizar. Si
quieres que el aviso `[i]` desaparezca, re-corre el instalador: es lo único que
escribe el marker (los scripts son de solo lectura y nunca lo regeneran).

## Kimi Code CLI: hooks globales (única excepción de escritura en `$HOME`)

Desde esta versión (feature #8) Kimi Code CLI (v0.29.x) es backend de primera
clase: lee el `AGENTS.md` generado (verificado empíricamente contra v0.29.2:
lo inyecta al system prompt), recibe los roles como subagentes nativos en
`.kimi-code/agents/*.md` (frontmatter `name`/`description`/`tools` con
allowlist por rol: leader/reviewer `Read, Grep, Glob, Bash`; implementer
además `Edit, Write`), y tiene launcher `bin/harness-kimi`. Todo eso vive en
el proyecto, como con los demás backends.

**La excepción**: Kimi **no soporta hooks por proyecto** (verificado: un
`[[hooks]]` en el config del proyecto no se ejecuta). El único lugar donde
existen es el config **global** `${KIMI_CODE_HOME:-~/.kimi-code}/config.toml`.
El usuario decidió (2026-07-28) aceptar esa única excepción a la regla de no
escribir fuera del proyecto, con estas salvaguardas:

- **Detección previa**: el bloque solo se escribe si Kimi está presente
  (`kimi` en PATH o `KIMI_CODE_HOME/bin/kimi` ejecutable). `--no-kimi`
  (`-NoKimi` en PowerShell) lo excluye explícitamente.
- **Backup previo** del `config.toml` en `bkp/` (mecanismo `HARNESS_BKP_DIR`
  de siempre) antes de cualquier escritura.
- **Bloque delimitado** por los marcadores `# >>> harness-process hooks >>>` y
  `# <<< harness-process hooks <<<`, con **reemplazo idempotente** solo entre
  marcadores: re-instalar no duplica nada y los hooks/config propios del
  usuario quedan intactos byte a byte.
- **Validación best-effort**: tras escribir se corre `kimi doctor`; si reporta
  config inválido se restaura el estado previo (o se retira el archivo recién
  creado) con un aviso accionable, sin cambiar el exit code del setup.
- **Guard por proyecto**: cada `command` del bloque solo actúa si
  `$PWD/bin/harness-hook` existe (el hook corre con cwd = proyecto,
  verificado); en proyectos sin arnés es un no-op silencioso que cuesta un
  `stat`.

Eventos registrados: `SessionStart` (timeout 120), `PostToolUse` con matcher
`Edit|Write` (timeout 30) y `Stop` (timeout 120), despachando a
`bin/harness-hook plain <evento>`. `SessionEnd` NO se registra a propósito
(el runtime lo trataría como otro `stop` y el check correría dos veces por
turno). El `Stop` con exit 2 + stderr bloquea el cierre del turno en Kimi
(verificado), igual que en Claude Code.

**`--reset` NO remueve el bloque global** (decisión del usuario 2026-07-28):
el bloque es compartido por TODOS los proyectos con arnés de la máquina y
`--reset` es por-proyecto — removerlo desde el proyecto A dejaría sin hooks al
proyecto B. Es inofensivo en máquinas que abandonan el arnés (el guard sale 0
en silencio). Para quitarlo a mano:

1. Abre `${KIMI_CODE_HOME:-~/.kimi-code}/config.toml`.
2. Borra desde la línea `# >>> harness-process hooks >>>` hasta la línea
   `# <<< harness-process hooks <<<` (inclusive).
3. Si algo sale mal, hay backups `config.toml.bak.*` bajo `bkp/` del arnés
   desde la primera instalación.

Instalaciones existentes: re-corre el instalador (`./setup_harness.sh` /
`.\setup_harness.ps1`) y aparecen los subagentes, el launcher y —si Kimi está
instalado— el bloque global. Nota de acoplamiento: los nombres de eventos y
tools están verificados contra v0.29.2; si una versión futura de Kimi los
renombra, el fallo es benigno (el hook no matchea) y el gate de espejo de
roles no depende de Kimi.

## Hub por lotes + instalación atómica del binario (feature #14)

Desde esta versión (feature #14, decisiones del usuario 2026-08-14). Dos
problemas que se notaban todos los días:

- **El sync del hub tardaba minutos.** Cada comando que tocaba el hub
  (`sync_git`, `start`, `advance`, `approve-spec`, `autocheck`…) reescribía el
  grafo **entero, fila por fila**: en el hub de referencia eran 1047 nodos y
  1641 aristas, o sea 2688 ida-y-vuelta contra un PostgreSQL remoto a 164 ms por
  consulta (≈7 min por commit). Ahora el guardado escribe **solo lo que el
  comando tocó**, y en **lotes** (`INSERT … SELECT * FROM UNNEST(…)`, hasta 1000
  filas por sentencia, dentro de una única transacción). Un `sync_git` típico
  pasó de 2688 sentencias a 2.
- **El candado del hub era único para toda la máquina.** `$HARNESS_HUB/.lock`
  serializaba TODOS los proyectos: un sync lento en un repo dejaba en cola a
  `start`, `advance` y `approve-spec` de todos los demás. Ahora el candado es
  **por proyecto**: `$HARNESS_HUB/.lock-<proyecto>`. Separarlo es seguro
  justamente porque cada comando escribe solo sus filas.
- **Nuevo `DB_STATEMENT_TIMEOUT`** (milisegundos; `30000` por defecto; `0`
  desactiva), en el entorno o en `$HARNESS_HUB/.env`. Corta del lado del
  servidor la sentencia que se pase de ese tiempo, más keepalives TCP: un hub
  que deja de responder ahora falla con error legible en vez de colgar el
  comando —y el candado— indefinidamente. `connect_timeout` solo cubría el
  saludo inicial.
- **El instalador ya no copia encima del binario vivo.** `setup_harness.sh` y
  `setup_harness.ps1` escriben `harness` / `harness.exe` en un temporal
  **hermano** del destino y lo mueven con un rename atómico. Copiar encima
  reescribía el mismo inode, y en macOS eso le invalida al kernel la firma
  cacheada del Mach-O: la corrida siguiente moría con `zsh: killed  harness`
  (SIGKILL) **en cada actualización**. En Windows, si `harness.exe` está en uso,
  el instalador aparta el destino en vez de dejarlo a medio escribir.

**Importante al actualizar**: re-corre el instalador en **todos** los proyectos
donde tengas el arnés instalado, no solo en uno. Mientras un proyecto conserve
el binario viejo, ese binario sigue tomando el candado global (que el binario
nuevo ya no mira) y sigue reescribiendo el grafo entero dentro de una
transacción larga: puede bloquear las escrituras nuevas (que cortarán por
`DB_STATEMENT_TIMEOUT`) y pisar con datos viejos filas recién actualizadas.

## Lecciones: memoria procedural (feature #17)

Primer hito del PRD `docs/prd/aprendizaje/`. El arnés guardaba todo lo aprendido
**por id de feature** (`impl-7.md`, `impl-14.md`), que es el orden en que nadie
lo busca. Ahora existe `docs/lecciones/<clase>.md`, ordenado por **clase de
trabajo**, con el comando `leccion` (`list`, `show`, `nueva`, `usar`).

Al re-correr el instalador aparece `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`
(la guía del arnés, con el método y la lista de qué **no** capturar). No hay nada
más que hacer: sin lecciones escritas el comportamiento del arnés no cambia.

**Nada se rompe en una instalación existente.** El gate del cierre es opt-in: si
`feature_list.json` no declara `"require_leccion": true` en `rules` —el default—,
`close --status done` se comporta exactamente como antes. Para prenderlo:

```json
"rules": {
  "require_spec_approved": true,
  "require_review": true,
  "require_leccion": true
}
```

Las dos primeras son las únicas que trae el molde de un proyecto nuevo
(`templates/feature_list.json` no declara ninguna otra). Las reglas que el arnés
**lee** hoy en el cierre son cinco, en el orden en que las corre `close --status
done`:

- `require_spec_approved` — exige `docs/spec-feature-<id>-<slug>.md` con
  `Estado: approved` (también en `advance` y en `harness_check.sh`).
- `require_verify_green` — exige el reporte de `verify` más nuevo que el spec, sin
  rojos y sin `vacio` (features #23 y #44).
- `require_docs_al_dia` — exige la propuesta de `prd propose` resuelta y aplicada
  con tu sí (feature #29).
- `require_review` — exige el veredicto `approved` **estampado por el binario** en
  `docs/review-<id>.md` (feature #64).
- `require_leccion` — la de esta sección: exige `--leccion <clase>` o
  `--leccion ninguna` con motivo.

Las tres que no vienen en el molde (`require_verify_green`, `require_docs_al_dia`
y `require_leccion`) nacen apagadas y se prenden agregándolas a mano, como acá
arriba.

Hasta la #64 este bloque también publicaba `one_feature_at_a_time`,
`require_tests_to_close` y `require_impact_check` en `true`. Ninguna de las tres
la leía ningún gate: nacieron decorativas en el molde y nunca hicieron nada. Se
borraron de `templates/feature_list.json` para dejar de prometer un enforcement
que no existía; si siguen en el `feature_list.json` de tu proyecto, son inertes.

Con la regla activa, el cierre exige `--leccion <clase>` o
`--leccion ninguna --leccion-motivo "<por qué>"`. Declarar que no se aprendió
nada es válido; hacerlo sin motivo, no.

Dos cosas que conviene saber antes de escribir la primera:

- **`leccion nueva` rechaza nombres de sesión** (con `feature`, con `#`, con
  prefijo `fix-`/`debug-`/`audit-`/`hotfix-`, con fecha, o con números de tres o
  más dígitos) y **no existe `--force`**. Si el nombre solo tiene sentido para la
  tarea de hoy, lo correcto es patchear una lección existente.
- **`--reset` NO borra tus lecciones.** Solo refresca la guía, que es plantilla
  del arnés. Las lecciones son conocimiento ganado, como el PRD y la constitution.

`harness_check.sh` suma un bloque: un frontmatter ilegible o un `nombre:` que no
coincide con el archivo **bloquean**; la falta de `triggers` solo avisa. Sin
`docs/lecciones/` el bloque entero se omite, así que una instalación que no use
lecciones no ve ninguna diferencia.

## Estado `superseded` (feature #37)

`close --status superseded --absorbida-por <id>` para una entrada cuyo trabajo se
hizo en OTRA feature. No es `done` (nunca tuvo spec propio) ni `blocked` (no está
trabada).

- **Exige la referencia** y la valida: no se puede citar una feature inexistente
  ni absorberse a sí misma. Queda como campo `superseded_by`.
- **No pasa por los gates de `done`**: spec, lección, verify y documentos viven
  en la feature que absorbió.
- `status` la muestra como `[superseded por #N]`; `next` no la ofrece;
  **`prd tree` no la cuenta ni arriba ni abajo**; `journey` no la reporta como
  cierre sin lección.
- La migración de `blocked` a `superseded` es **explícita**: el arnés no puede
  adivinar cuáles estaban absorbidas.

## Consolidacion de lecciones con LLM (feature #28)

Aparece `sh harness_cli lecciones consolidar`, la **unica** parte del arnes que
usa un modelo. Detecta lecciones que cuentan lo mismo e informa; con `--aplicar`
fusiona bajo un paraguas y archiva las miembros.

**Apagada por default, estructuralmente**: sin `rules.consolidar_backend` no se
resuelve backend ni se mira el entorno. Encendela con
`{ "rules": { "consolidar_backend": "auto" } }`.

Lo que hay que saber:

- **El modelo nunca ve el cuerpo de una leccion**: solo nombre, descripción y
  triggers. Los procedimientos y los pitfalls no salen de `docs/`.
- **El modelo nunca escribe**: `detectar()` no recibe `&HarnessPaths`. Y el
  prompt viaja como item de argv, nunca por `sh -c`.
- **La fusión la pide una persona**: `--aplicar` toma `--en/--de/--motivo` de
  argv, no de lo que dijo el modelo. Sin `--motivo`, exit 2.
- **El paraguas tiene que poder reemplazar lo que archiva**: sin placeholders,
  con **todos** los triggers de cada miembro y citando `[[cada-una]]`. Si no,
  exit 2 — archivar contra un esqueleto pierde el conocimiento.
- **Nunca borra**: `docs/lecciones/archivo/` con el cuerpo intacto, backup previo
  y `lecciones rollback`.
- **Cadena de backend**: `HARNESS_CONSOLIDAR_CMD` → primer CLI (`claude -p`,
  `kimi -p`) → skip limpio. El tramo de API key **no está implementado** y el
  skip lo dice: el arnés no habla HTTP.
- **La confianza se reporta sin filtrar**: no hay umbral, porque con este corpus
  no se puede calibrar ninguno.

## Documentos al dia: `prd propose` / `prd apply` (feature #29)

El PRD, el SDD y `docs/architecture.md` dejan de poder quedar mintiendo. **El
agente propone, el usuario aprueba, el binario escribe** — el mismo ritual que
`approve-spec`.

```bash
sh harness_cli prd propose --feature <id>       # siembra una pregunta por documento
sh harness_cli prd apply --feature <id>         # muestra que escribiría; NO escribe
sh harness_cli prd apply --feature <id> --yes   # solo con tu sí
```

El alcance lo calcula el binario desde el árbol real: el PRD de origen, sus
padres, `docs/prd/SDD-master.md` y `docs/architecture.md`. Cada bloque se
contesta con `cambio` (texto literal), `ya-esta <archivo>:<L1>-<L2>` (**el
binario verifica la cita**) o `no-aplica <razón>`.

Lo que hay que saber al actualizar:

- **Regla nueva, apagada por defecto**: `require_docs_al_dia`. Con ella,
  `close --status done` exige la propuesta resuelta **y aplicada con tu sí**.
- **El gate no usa frescura**: `verify` reescribe su reporte en cada corrida y
  `prd apply` es idempotente, así que compararlos dejaría la propuesta vieja para
  siempre. Está dicho en un test para que nadie lo "mejore".
- **Ningún `Comando:` de un AC puede invocar `prd apply --yes`**: `verify` los
  ejecuta con `sh -c` y aplicaría sin tu sí. Hay un test que lo prohíbe sobre los
  specs reales.
- **La propuesta vive en `docs/prd-diff-<id>.md`**, fuera de `docs/prd/**`, para
  componer con las rutas protegidas de la #26. `prd apply --yes` escribe como el
  binario y registra sus escrituras, igual que `close`.
- `CHECKPOINTS.md` y los tres roles declaran el deber: antes no lo mencionaba
  nadie.

## Rutas protegidas (feature #26)

Los PRD y la constitution dejan de depender de la buena fe. Lista en
`rules.rutas_protegidas` (defaults: `docs/prd/**`, `docs/constitution.md`,
`.env`), con tres capas: `PreToolUse` **impide** la escritura donde el backend lo
soporta (hoy Claude Code), `PostToolUse` **avisa** con el comando de reversion, y
`harness_check.sh` **bloquea** el cierre con exit 2.

La capa de detección **no puede prevenir** — corre después de la herramienta —, y
eso está dicho en la doc en vez de prometerse como bloqueo.

Lo que hay que saber al actualizar:

- **`harness_check.sh` ahora puede bloquear por esto.** Si adoptás la protección
  con trabajo en curso, corré una vez
  `sh harness_cli rutas --aceptar-estado-actual`: toma el estado actual como
  línea de base para que el gate no arranque en rojo por cambios que ya estaban.
- **El arnés no se bloquea a sí mismo**: `close` (que marca hitos en el PRD) y
  `prd add` registran sus propias escrituras. La exención caduca en cuanto
  alguien vuelve a tocar el archivo.
- **Para apagarla**: `"rutas_protegidas": []`.
- **El aviso muestra `git diff` antes del comando destructivo**, y etiqueta que
  `git checkout --` descarta *todo* lo no commiteado de ese archivo. Se aprendió
  a la mala: la primera versión borró hitos sin commitear de tres features.

Se suma `.claude/settings.json` con un hook `PreToolUse` sobre `Edit|Write|MultiEdit`.
Una instalación que no re-corra el instalador se queda sin la capa de prevención,
pero conserva las otras dos.

## doctor: diagnostico de la instalacion (feature #25)

Aparece `sh harness_cli doctor [--json]`. Revisa **la instalacion** —binario,
hooks, superficies, marker, hub, herramientas, graphify— e imprime **el comando
exacto de remedio** por cada problema.

No se solapa con `harness_check.sh`: ese mira el **proceso** (spec, plan, PRDs,
lecciones, perfil, convenciones) y sigue igual. Cada salida remite a la otra.

El exit code: **2 solo si algo impide trabajar** (binario roto, hook apuntando a
la nada, herramienta requerida ausente). Hub caido, graphify ausente y
herramientas opcionales son avisos `[i]` con exit **0** — si el hub caido saliera
2, el exit code mentiria.

Tres cosas que conviene saber:

- **En el checkout fuente del arnes**, superficies y hooks se reportan
  `no_aplica`, no como falla: ahi su ausencia es lo correcto.
- **Solo exige la superficie de los backends instalados**: si no usás Gemini, no
  te pide `GEMINI.md`.
- **No arregla nada** ni tiene `--fix`, y no escribe un solo byte.

Y el lanzador `harness_cli` mejoró: cuando el binario falta, o cuando es tan
viejo que no conoce el subcomando (lo tipico tras `git pull` sin re-instalar),
imprime `Remedio: bash setup_harness.sh` en vez de un error de clap. Es la mitad
del diagnostico que un doctor dentro del binario no puede cubrir.

## Convenciones: la escalera de huella y las reglas de test (feature #24)

`docs/conventions.md` pasó de 7 líneas de buenos deseos a dos criterios que el
reviewer usa para **rechazar**.

**La escalera de huella**: extender lo que ya existe > flag en un comando
existente > comando nuevo > superficie nueva > dependencia con ADR. Se elige el
peldaño de **menor huella que resuelva el problema**, y si no tomás el más alto,
el plan lo declara con la línea que el reviewer busca:

```
Peldano elegido: <n> (<nombre>) porque <por qué el de arriba no alcanzaba>
```

**Las tres reglas de test**: contratos de comportamiento y no snapshots;
prohibido leer el código fuente en un test; prohibido el test
detector-de-cambios. La segunda admite **una** excepción: que el archivo sea
*dato de entrada* del código bajo prueba, con este corte — *¿el test seguiría
valiendo si la implementación se reescribiera entera?*

Qué cambia en la práctica:

- `harness_check.sh` suma un bloque que **avisa** (`[i]`, con archivo, línea y
  nombre del test) cuando un test lee el fuente. **No bloquea** y no cambia el
  exit code: la regla tiene una excepción legítima, y un gate duro empujaría a
  inventar un `--force`. Sin `rust/tests/` el bloque se omite entero, así que un
  proyecto que no es Rust no ve ninguna diferencia.
- Los tres roles la aplican: el líder elige el peldaño y lo justifica, el
  implementer conoce las reglas antes de escribir tests, el reviewer **rechaza**
  los que las violan (no los anota como observación).
- Las otras dos reglas no se chequean solas: entender qué dato "se espera que
  cambie" no se grepea.

No hay comando nuevo, ni flag, ni dependencia: la feature se aplicó su propia
escalera y salió peldaño 1.

## verify: AC ejecutables (feature #23)

Un AC puede declarar **cómo se prueba**, en la línea de abajo:

```
- AC-5: Given un spec en draft, When corre `verify`, Then se niega con exit 2.
  Comando: `cd rust && cargo test verify_should_refuse_to_run_commands_from_a_draft_spec`
```

`sh harness_cli verify --feature <id>` los ejecuta y escribe `docs/verify-<id>.md`
con el estado de cada AC. Los flags: `--solo AC-n` para iterar sobre uno,
`--json` para consumirlo desde un script.

**Nada cambia si no lo usás.** Un AC sin `Comando:` queda como *manual* (lo
verifica el reviewer, como siempre) y no cuenta como fallo; un spec sin ninguna
línea `Comando:` informa que no hay nada que verificar y sale **0**. Los 310 AC ya
escritos en este repo siguen valiendo sin tocarse — y eso es un test, no una
promesa.

**Es el único comando del arnés que ejecuta shell.** Por eso:

- **Exige `Estado: approved`.** En draft se niega con exit 2 y no ejecuta ni un
  comando: aprobar el spec es el acto en el que el usuario leyó esos comandos.
- **Se invoca a mano.** Ningún hook lo llama.
- **Imprime cada comando antes de correrlo.**
- **Cerrar nunca ejecuta**: el gate lee el reporte.

Regla opcional, apagada por defecto:

```json
{ "rules": { "require_verify_green": true, "verify_timeout_segundos": 300 } }
```

Con `require_verify_green` activa, `close --status done` exige el reporte
existente, **más nuevo que el spec** y sin rojos (exit 2 nombrando cuáles
fallaron). Con la regla apagada o con un spec sin comandos declarados, cerrar se
comporta exactamente como antes.

Dos trampas que conviene conocer antes de declarar un comando, las dos
encontradas corriendo `verify` sobre su propio spec: `cargo test <nombre>` con
cero coincidencias **sale 0** (un nombre mal escrito da verde sin ejecutar nada),
y `... | grep -c ... || true` **nunca falla**. Un comando que no puede fallar no
verifica: decora.

### `vacio`: la primera de esas dos trampas ya no depende de que te acuerdes (feature #44)

La #23 encontró que `cargo test <nombre-inexistente>` sale 0, lo arregló a mano
renombrando los tests y dejó escrita la advertencia de arriba. Cinco features
después volvió a pasar: el **AC-12 de la #28** declaraba
`consolidar_without_aplicar_should_not_touch_anything`, esa función no existía, y
el invariante más citado de ese comando quedó registrado verde sin nada detrás.

Desde la #44 `verify` mira la **salida** además del exit code. Si reconoce el
formato de libtest y la suma de `passed` es **cero**, el AC queda en `vacio`:

```
26 verde(s), 0 en rojo, 0 manual(es), 1 sin casos.
```

`vacio` **bloquea el cierre** igual que un rojo y se cuenta aparte para no
esconderse entre ellos. El detector mira la forma de la SALIDA, no el texto del
comando (así cubre un `cargo test` adentro de un script), y **no opina** cuando
la salida no tiene líneas `test result:` — un `grep`, un `bash` o un compilador
siguen exactamente como antes. No hay flag para apagarlo: si un AC de verdad se
verifica a mano, el camino honesto es no declarar `Comando:` y dejarlo `manual`.

## journey: el mapa de lo aprendido (feature #22)

Último hito del PRD de aprendizaje. `sh harness_cli journey` cruza los tres
almacenes (lecciones, perfil, features cerradas), muestra sus enlaces y **señala
los huecos**: enlaces rotos, features que cerraron sin declarar nada, lecciones
huérfanas.

Es **solo lectura** y no hay nada que configurar. No tiene `delete` ni `edit` a
propósito: podar sigue pasando por `lecciones archivar` y `perfil remove --yes`,
que ya tienen sus garantías.

Detalle que evita ruido: una feature que cerró **antes** de que el proyecto
empezara a declarar lecciones no cuenta como hueco. En este repo eso bajó el
reporte de 16 huecos (ninguno corregible) a 0.

## El curador de lecciones (feature #21)

Hito 5 del PRD de aprendizaje: el mantenimiento de la biblioteca. Aparece el
comando `lecciones` con el ciclo de vida (`activa` → `stale` → `archivada`), pin,
backup y rollback.

**No cambia nada por sí solo.** La pasada por defecto (`lecciones curar`) **solo
informa**; para que algo se mueva hay que pedir `--aplicar` a mano. Y aún así
nunca borra: archivar es mover a `docs/lecciones/archivo/`, con backup previo en
`bkp/lecciones/<ts>/` y `lecciones rollback` para deshacer.

Umbrales por defecto: 30 días sin uso → `stale`, 90 → archivada. Se ajustan con
`leccion_stale_dias` / `leccion_archivo_dias` en `rules`, y con `0` se apaga ese
tramo.

Dos cosas que conviene saber:

- `leccion list` deja de mostrar las archivadas (se ven con `--archivadas`), pero
  **`buscar` las sigue encontrando**, rankeadas por debajo de las activas.
- `harness_check.sh` ahora valida también el formato de las archivadas.

Lo que salió de esta feature a propósito: la **consolidación asistida por LLM**
quedó como feature aparte (#28), porque es la única parte que necesita un modelo
y no se podía verificar de punta a punta.

## buscar: preguntarle al repo (feature #20)

Hito 4 del PRD de aprendizaje. Las features #17-#19 le dieron memoria al arnés;
esta la hace **preguntable**.

```bash
sh harness_cli buscar "ureq adr"
```

No hay nada que configurar y no cambia ningún flujo existente: es un comando
nuevo, de **solo lectura**, sin estado y sin regla que lo apague. Si el proyecto
no tiene `docs/`, lo dice y sale con 0.

Lo que aporta sobre `grep -r` es el orden: primero lecciones y perfil, después
specs/planes/ADRs, después impl/review/estado, y al final `history.md`. El
`score` viaja en `--json` para que el orden se pueda auditar.

Sin índice (el corpus son ~1 MB: escanearlo toma milisegundos y un índice viejo
miente), sin LLM y sin hub.

## Perfil de usuario (feature #19)

Hito 3 del PRD de aprendizaje, y el tercer almacén de memoria: el hub guarda
**eventos**, `docs/lecciones/` guarda **procedimiento** y `docs/perfil-usuario.md`
guarda **preferencias** — cómo querés trabajar. Es el único de los tres que viaja
solo hasta la superficie que lee cada agente al arrancar.

Al re-correr el instalador aparece `docs/perfil-usuario.md` **vacío** (solo su
encabezado). Mientras no tenga entradas, **nada cambia**: no se inyecta ningún
bloque y las superficies quedan byte a byte como antes.

Para llenarlo:

```bash
sh harness_cli perfil sugerir     # junta lo que ya decidiste, agrupado por feature
# el agente te propone una entrada; vos decidís
sh harness_cli perfil add --texto "Ante un fork de consistencia, elige la opcion segura. (#14)" --yes
```

Cosas que conviene saber antes:

- **Los tres comandos de escritura exigen `--yes`** y se niegan sin él. Es tu
  documento: `--reset` no lo borra y un reinstall no lo pisa.
- **El límite es 1500 caracteres y es duro.** Al pasarse, el comando falla y te
  muestra las entradas actuales para que consolides. No recorta nada.
- **Se bloquean los secretos.** Una entrada con pinta de credencial, clave privada
  o Unicode invisible se rechaza antes de escribir: el archivo se versiona *y* se
  inyecta en cada prompt.
- **El bloque de las superficies es un snapshot congelado**: se refresca al
  reinstalar, no en la sesión en curso.

`harness_check.sh` suma un gate: si el perfil supera el límite, **bloquea** (es lo
que se inyecta en el prompt de cada agente). Sin el archivo, el gate no corre.

## El arnés te empuja a capturar lo aprendido (feature #18)

Hito 2 del PRD de aprendizaje. La #17 dio el lugar donde guardar; esta hace que
el arnés **lo pida solo**. Dos disparadores, los dos por stderr y los dos con
exit 0 siempre:

- **Cada 25 invocaciones** del hook `PostToolUse` (que ya existía): un
  recordatorio de cuatro líneas para mirar el catálogo y patchear antes que
  crear. Se ajusta con `"leccion_nudge_interval": <n>` en `rules`, y con `0` se
  apaga.
- **Al cerrar como done sin `--leccion`**: el contrato completo, **leído de
  `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`**. Editás la guía y cambia el
  contrato; no hay copia en el binario que pueda divergir. Si la guía falta o
  está incompleta, degrada a un puntero: leer la guía nunca rompe un cierre.

**Un proyecto sin `docs/lecciones/` no ve absolutamente nada nuevo** — ni una
línea de stderr ni un archivo en `progress/`.

El único cambio de comportamiento en instalaciones existentes es el aviso de
"sin feature activa": antes repetía lo mismo cada 10 minutos para siempre, ahora
escala 600 s → 1200 → 2400 → 3600 mientras nada cambia y vuelve al piso apenas
aparece una feature activa. El estado vive en `progress/.last_nudge` (que ahora
guarda el nivel) y en `progress/.nudge_lecciones` (el contador por feature); un
`.last_nudge` vacío de una instalación previa se lee como nivel 0.

## El MCP de Atlassian llega a los cuatro backends (feature #52)

`atlassian drain` siempre asumió que el agente tenía el MCP de Atlassian
conectado — y el arnés nunca lo instalaba. Ahora, si el repo tiene binding, el
instalador deja el MCP configurado **por proyecto** en los backends que lo
admiten: `.mcp.json` (Claude), `.kimi-code/mcp.json` (Kimi) y `.grok/config.toml`
(Grok, vía `mcp-remote`).

Dos rarezas que se verificaron contra los CLIs reales y quedaron resueltas:

- **Grok** no completa el flujo OAuth sobre HTTP (falla con `OAuth authorization
  required`), así que su configuración va por el bridge `npx mcp-remote`.
- **Codex** necesita el plugin `atlassian-rovo@openai-curated` **además** del
  servidor MCP: con el servidor solo, el agente contesta "necesito que instales
  el plugin"; con el plugin solo, "no hay acceso visible".

Codex no admite MCP por proyecto, así que **su configuración global no se toca**:
el instalador imprime los dos comandos para que los corras vos.

Nada de esto ocurre sin `atlassian.json`, `--no-mcp-atlassian` lo apaga, no se
pisa lo que ya tengas (ni se pierden otros servidores MCP) y **el arnés no hace
el OAuth**: al terminar te dice cómo autorizar en cada CLI. Los archivos se
versionan a propósito —no llevan credenciales— para que el próximo que clone el
repo no repita la configuración.

## Features en paralelo con worktrees y GitFlow (feature #47)

Hasta ahora el arnés imponía **una feature a la vez**: `start` rechazaba la
segunda con "Ya hay feature in_progress". Eso se terminó.

Ahora cada feature que arranca se lleva su propia rama (`feature/<id>-<slug>`, o
`bugfix/` si es `--kind bug`) y su propio worktree hermano del repo
(`../<repo>-wt/<id>-<slug>`), así que dos implementaciones nunca comparten
archivos en disco. El checkout principal no cambia de rama nunca.

**Qué cambia en tu repo al actualizar:**

- `progress/current.md` deja de ser el estado de la feature activa y pasa a ser
  el **índice** de lo que está en curso. El estado vivo de cada feature vive en
  `progress/current-<id>.md`, y su checkpoint en `.last_autocheck-<id>`.
  Consecuencia directa: cerrar una feature ya no puede pisar el estado de otra
  (era un bug real, la feature #45).
- `close --status done` ahora exige `--to <rama>`: el arnés no elige a dónde
  integrar, te lo pregunta. Con `--to`, commitea lo que quede en el worktree,
  mergea (`--no-ff`), publica la rama destino, borra el worktree y conserva la
  rama. `blocked`, `pending` y `superseded` no integran y conservan todo.
- `one_feature_at_a_time` **ya no está en el molde** (feature #64): además de
  quedar contradicha acá, ningún código la leía. Si aparece en el
  `feature_list.json` de un proyecto ya instalado es inerte y se puede borrar a
  mano. La migración de reglas **no** la saca: solo agrega las claves que faltan,
  nunca pisa ni quita lo que decidió el usuario.
- Dentro de un worktree, los comandos infieren la feature por la carpeta: no
  hace falta `--feature`.

**Nada de esto exige git.** En un directorio sin repo, o con
`start --sin-worktree`, el arnés avisa y funciona como siempre. La rama base es
`develop` si existe, y si no `main`; el arnés nunca la crea.

**El cierre no declara hecho lo que no hizo (feature #62).** El cierre escribía
todo su estado —backlog en `done`, transición a Jira, anotación del plan, estado
archivado, índice, `history.md`, memorias del hub y el mensaje "cerrada"— **antes**
de integrar. Si la integración fallaba, esas nueve afirmaciones ya estaban hechas
sobre un trabajo que no estaba integrado. Ahora corre en cuatro fases: lo que
puede negarse (gates, `--to`, colisiones), los artefactos que tienen que viajar
en la rama (la anotación del plan y el estado archivado, hechos **idempotentes**
porque el merge borra el worktree donde viven), la integración, y recién después
el estado. Si la integración falla, no hay nada que revertir porque no se
escribió nada: la feature sigue `in_progress`, su `current-<id>.md` sigue vivo y
`history.md` no tiene la línea. Resolvés y volvés a correr el mismo comando.

Sin rollback a propósito: quedaría parcial (el intent ya emitido a Jira y la
memoria ya escrita en el hub no se deshacen) y habría que acordarse de
mantenerlo cada vez que el cierre gane un efecto nuevo. **Cambio visible:**
`Feature #N cerrada` ahora se imprime después de la salida de `[GitFlow]`.

**El merge no corre en tu checkout (feature #61).** La promesa de arriba —"el
cierre de una feature no puede exigirte tener el escritorio ordenado"— tenía una
excepción que no estaba escrita: el worktree temporal se usaba **solo si el
destino no era la rama que tenías abierta**. Cerrar hacia `main` estando en
`main` mergeaba en tu árbol, y moría con el texto crudo de git si tocaba algo
que tenías sin commitear — después de haber commiteado el worktree de la
feature. Ahora el merge corre siempre en un worktree temporal `--detach`, y la
rama destino se avanza con `git reset --keep` (que **conserva** tus cambios sin
commitear) o moviendo la referencia si no la tenés abierta.

El único caso que no se puede resolver sin decidir por vos —que el merge cambie
un archivo que vos tenés modificado— se detecta **antes de tocar nada** y el
cierre se niega nombrando los archivos y las tres salidas (commitear, `git
stash`, o descartar). El arnés no stashea ni descarta por su cuenta: son tus
cambios. Tampoco avanza la rama dejando tu árbol atrás: se midió y `git status`
pasa a mostrar la **reversión** del merge, con lo que un commit distraído
desharía lo recién integrado.

## Envio automatico a Atlassian (feature #16)

La feature #15 dejó el arnés y Atlassian hablando el mismo idioma, pero había
que empujar a mano (`atlassian apply` / `atlassian publish`). Ahora **el flujo
empuja solo**: cada transición (`prd add`, `add`, `start`, `advance`,
`approve-spec`, `close`) lanza un worker en segundo plano que aplica lo
pendiente en Jira y republica PRD, SDD y specs en Confluence.

El comando que escribís sigue siendo instantáneo: el envío ocurre detached (el
mismo patrón que ya usa graphify), así que Atlassian nunca te frena y lo que
falle se reintenta en la próxima transición. El detalle queda en
`progress/atlassian/last-push.log` y el estado en `atlassian status`.

Novedades que trae:

- **Backfill**: al activar el binding en un repo con historia, el primer envío
  carga lo que ya existe (un epic por PRD, una historia por feature con su
  estado y sus subtasks AC-n). Si ya hay un epic con el mismo título que tu PRD,
  lo **adopta** en vez de duplicarlo. Se re-corre con `atlassian backfill`
  (idempotente) y `--sin-acs` omite las subtasks.
- **`add --kind bug|feature|task`**: un bugfix entra a Jira como `Bug` y no como
  historia. Sin el flag, todo sigue igual que antes.
- **Verificación del binding**: con token, `atlassian bind`, el instalador y
  `atlassian status` comprueban que el proyecto Jira y el space existan. Si
  faltan, avisan cómo crearlos; solo los crean si lo pedís con
  `--create-project` / `--create-space` (o `--create-jira-project` /
  `--create-confluence-space` en el instalador) y tenés permiso de admin.
- **Interruptor**: `"auto": false` en `atlassian.json` lo apaga para el repo, y
  `HARNESS_ATLASSIAN_AUTO=0` para una corrida puntual. Sin token no hay envío
  automático: los intents esperan al agente con MCP, como antes.

Nada de esto se activa en un repo sin `atlassian.json`.

## Atlassian: Jira y Confluence en el flujo (feature #15)

Novedad opt-in y **por repo**: el arnés puede reflejar cada movimiento del
desarrollo en Jira (epics, historias, subtasks de los AC-n, transiciones,
comentarios y sprints) y publicar el PRD, el SDD y los specs en Confluence.

**Nada cambia si no lo activás.** Sin `atlassian.json` en la raíz del proyecto,
el flujo se comporta exactamente como antes: mismos comandos, mismos exit codes
y ninguna carpeta nueva. Las instalaciones existentes siguen igual hasta que
vuelvan a correr el instalador con los flags nuevos.

Para activarlo hay que decirle **a qué proyecto y a qué space pertenece el
repo** (el arnés no lo adivina: si no lo sabe, se niega y pide preguntar):

```bash
sh setup_harness.sh --atlassian-site acme.atlassian.net \
                    --jira-project ADR \
                    --confluence-space SD
# o, ya instalado:
sh harness_cli atlassian bind --site acme.atlassian.net --jira-project ADR --confluence-space SD
```

Los cuatro valores también se leen del config file (`.harness.env`,
`~/.config/harness/config`) como `HARNESS_ATLASSIAN_SITE`, `HARNESS_JIRA_PROJECT`,
`HARNESS_CONFLUENCE_SPACE` y `HARNESS_JIRA_ISSUE_TYPE`. El binding existente
**no se pisa** al reinstalar.

Dos ejecutores, una misma outbox (`progress/atlassian/outbox/`):

- **Sin credenciales**: `atlassian drain` imprime el plan de llamadas MCP
  ordenado por dependencia y el agente lo ejecuta, devolviendo cada clave con
  `atlassian ack --intent <id> --key <ADR-n>`.
- **Con token** (`HARNESS_ATLASSIAN_EMAIL` + `HARNESS_ATLASSIAN_TOKEN` en
  `.harness.env`, que no se versiona): `atlassian apply` lo hace solo, y habilita
  `atlassian sprint start|close` y `atlassian publish`. Los sprints solo existen
  por esta vía: el MCP oficial de Atlassian no expone boards ni sprints.

Detalles y tabla de mapeo completa: `docs/atlassian-integracion.md`. La
dependencia HTTP nueva (`ureq`) está justificada en
`docs/adr/ADR-0001-cliente-http-ureq.md`, como exige el Artículo 6.

## Mantenimiento Rust only (post feature #2)

El punto de entrada es **`harness_cli`** (sh y .ps1): ejecuta **exclusivamente** el binario Rust `harness` / `harness.exe` (compilado desde `rust/`). 

- Sin binario: harness_cli falla con mensaje claro pidiendo cargo/rustup + re-setup.
- No hay fallback Python, ni parity, ni .py en templates/ o raiz.
- Cambios en rust/src + shims/setup + tests + docs deben ser verificados con cargo test/clippy + setup_smoke (bash+ps1) + harness_check.sh .

Sube version en Cargo.toml cuando haya cambios de comportamiento visibles en el CLI/hub.

## Recomendación

Mantén este repositorio (`harness_process`) actualizado y re-instala en tus proyectos cuando haya cambios relevantes. Así el protocolo de trabajo multi-agente se mantiene consistente y mejora con el tiempo.

**NUNCA commitees la carpeta del harness** en los proyectos donde lo instalas. El instalador la agrega automáticamente a `.gitignore`.

Si usas `--reset` + re-instalación, las superficies se regeneran desde cero con la versión más reciente del protocolo.

## Para maintainers de este repositorio harness_process

Cuando realizas mejoras (nuevo protocolo, fixes en rust/src + shims/setup, actualizaciones por otros LLMs, etc.):

1. Haz los cambios en este repo (el "fuente").
2. Una vez hecho el commit **sin co-author** (sin `Co-Authored-By`, sin "Generated with", sin trailers de IA):
   ```bash
   git commit -m "tu mensaje limpio"
   ```
3. Haz push del cambio:
   ```bash
   git push origin main
   ```

Esto hace que el cambio esté disponible **incluso si aplica en otros proyectos**. Los demás proyectos que usan este harness_process como fuente recibirán las mejoras la próxima vez que ejecuten:

```bash
./setup_harness.sh
# o
./setup_harness.sh --reset
```

Mantener el proceso explícito asegura consistencia multi-LLM a través de todos los proyectos.

## Cómo obtener este archivo si te falta al actualizar

Si al correr `./setup_harness.sh` ves el error "Falta el recurso requerido: UPDATING.md", significa que estás usando una versión actualizada del instalador pero aún no tienes el archivo UPDATING.md en tu carpeta `harness_process`.

Solución rápida:
- Copia este archivo `UPDATING.md` a tu carpeta `harness_process/` (junto a `setup_harness.sh`).
- O, si usas la estructura con `templates/`, colócalo dentro de `templates/`.
- Luego vuelve a ejecutar `./setup_harness.sh` (recomendado con `--reset` para una actualización limpia).

Este archivo se copiará automáticamente a los proyectos destino en futuras instalaciones.
