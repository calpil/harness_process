# Atlassian: el board y la wiki como reflejo del flujo

El arnes puede dejar rastro de cada movimiento del desarrollo en Jira y en
Confluence, sin que nadie copie nada a mano. Esta guia explica como se enciende,
que se crea del otro lado y como se ejecuta lo pendiente.

## Lo primero: a que proyecto pertenece este repo

El arnes NO adivina. O se lo decis al instalar, o te lo pregunta y no sigue:

```bash
# Al instalar (lo normal)
sh setup_harness.sh --atlassian-site acme.atlassian.net \
                    --jira-project ADR \
                    --confluence-space SD

# O despues, desde el repo ya instalado
sh harness_cli atlassian bind --site acme.atlassian.net \
                             --jira-project ADR \
                             --confluence-space SD
```

Eso escribe `atlassian.json` en la raiz del proyecto. Es versionable a
proposito: solo nombra el sitio, el proyecto y el space — nunca credenciales.

Si hay token configurado, el arnes **verifica** en el momento que el proyecto y
el space existan de verdad y te avisa si falta alguno (tambien lo revisa en cada
`atlassian status`). No los crea por su cuenta: para eso hay que pedirlo
explicitamente con `--create-project` / `--create-space` en `atlassian bind`, o
`--create-jira-project` / `--create-confluence-space` en el instalador, y hace
falta permiso de administracion en Atlassian.

**Sin ese archivo no pasa nada.** El arnes se comporta exactamente como si la
integracion no existiera: mismo flujo, mismos exit codes, sin carpetas nuevas.

Los cuatro valores tambien se pueden dejar en el config file (`.harness.env`,
`~/.config/harness/config`) como `HARNESS_ATLASSIAN_SITE`,
`HARNESS_JIRA_PROJECT`, `HARNESS_CONFLUENCE_SPACE` y `HARNESS_JIRA_ISSUE_TYPE`.

## Que se crea del otro lado

| En el arnes | En Jira |
| --- | --- |
| PRD (maestro o anidado) | Epic |
| Feature del backlog | Historia (`Story` por default) |
| Feature cargada con `add --kind bug` | `Bug` |
| Feature cargada con `add --kind task` | `Task` |
| AC-n del spec | Subtask `AC-n · <texto>` |
| `start` | Transicion a **In Progress** (y entra al sprint vigente, si hay) |
| `advance --nota` | Comentario con la nota |
| `approve-spec --yes` | Comentario con el sello de la aprobacion |
| `close --status done` | Transicion a **Done** + comentario con la nota |
| `close --status blocked` | Flag **Impediment** (la historia no cambia de columna) |

Y en Confluence, con `atlassian publish`: el PRD maestro, cada PRD anidado (como
pagina hija, respetando el arbol de `prd tree`), el SDD maestro y cada spec
(colgado del PRD que lo origina). Cada pagina enlaza a su issue y cada issue a
su pagina.

## Se envia solo (feature #16)

Con binding + token, **no hay que correr nada a mano**: cada transicion del
flujo (`prd add`, `add`, `start`, `advance`, `approve-spec`, `close`) lanza un
worker en segundo plano que aplica lo pendiente en Jira y republica los
documentos en Confluence. El comando que escribiste vuelve al instante: si
Atlassian esta lento o caido, no te frena y lo pendiente se reintenta en la
proxima transicion.

```bash
sh harness_cli atlassian status   # dice si el envio automatico esta encendido
cat progress/atlassian/last-push.log   # que hizo el ultimo envio
```

Se apaga de dos maneras:

- `"auto": false` en `atlassian.json` (este repo, de forma permanente);
- `HARNESS_ATLASSIAN_AUTO=0` delante de un comando (solo esa corrida).

Sin token no hay envio automatico posible: los intents quedan en la outbox
esperando al agente con MCP, como antes.

### La primera vez: se carga lo que ya existe

Si activas el binding en un repo que ya tiene historia, el primer envio hace un
**backfill**: crea un epic por cada PRD del arbol y una historia por cada
feature del backlog, con su estado actual (y sus subtasks AC-n). La idea es que
el board sea espejo del repo, no un resumen de lo nuevo.

- Si el proyecto ya tiene un epic con el mismo titulo que tu PRD, el arnes lo
  **adopta** en vez de crear un duplicado.
- `sh harness_cli atlassian backfill` lo vuelve a correr cuando lo necesites
  (es idempotente), y `--sin-acs` carga sin las subtasks de los AC-n.

## Como se ejecuta lo pendiente (a mano)

El binario del arnes no habla MCP, asi que primero **escribe la intencion** en
`progress/atlassian/outbox/` y despues hay dos maneras de ejecutarla. Las dos
producen exactamente lo mismo.

### Ruta A — un agente con MCP de Atlassian (sin credenciales)

```bash
sh harness_cli atlassian drain     # imprime el plan de llamadas, no muta nada
# el agente ejecuta cada llamada con su MCP y devuelve la clave creada:
sh harness_cli atlassian ack --intent 0003 --key ADR-42
```

El plan viene ordenado por dependencia (epic -> historia -> subtasks ->
transiciones -> comentarios) y cada paso trae la tool exacta y sus argumentos.
Cuando un paso depende de algo que todavia no existe, el plan lo dice en `needs`
en vez de inventar una clave.

### Ruta B — el arnes solo (con token)

#### Donde van el email y el token

El instalador ya deja un **`.harness.env` en la raiz del proyecto** con las
claves comentadas y lo agrega al `.gitignore`. Descomentalas y listo:

```
HARNESS_ATLASSIAN_EMAIL=vos@empresa.cl
HARNESS_ATLASSIAN_TOKEN=<API token de https://id.atlassian.com/manage-profile/security/api-tokens>
```

Hay tres lugares posibles, y el arnes los busca en este orden:

| Donde | Cuando conviene |
| --- | --- |
| Variables de entorno | CI, o una sesion puntual |
| `.harness.env` del proyecto | credenciales distintas por proyecto |
| `~/.config/harness/config` (o `~/.harnessrc`) | **lo normal**: el token una sola vez para TODOS tus proyectos |

Lo local siempre gana sobre lo global, asi que podes tener el token global y
pisarlo en un repo puntual. El archivo del proyecto nunca se pisa al reinstalar
y jamas se commitea; el token no aparece en la outbox, el estado, los logs ni
la salida de `status` (que solo dice `presente` o `ausente`).

```bash
sh harness_cli atlassian apply     # ejecuta lo pendiente contra la API
```

Solo se marca aplicado lo que Atlassian confirmo. Si algo falla, el intent
queda pendiente con el error real (codigo + mensaje) y el proximo `apply` lo
reintenta sin repetir lo ya hecho.

## Sprints

El MCP oficial de Atlassian no expone boards ni sprints, asi que esta parte
necesita token (ruta B):

```bash
sh harness_cli atlassian sprint start --name "Sprint 12" --goal "cerrar cobranza" --days 14
sh harness_cli atlassian sprint close
```

Mientras haya un sprint vigente, cada feature que arranques con `start` entra
sola. Si no hay ninguno, la historia se queda en el backlog y no se rompe nada.
Al cerrar, el arnes lista que historias quedaron sin terminar.

## Ver el estado

```bash
sh harness_cli atlassian status
```

Muestra el binding vigente, si hay token (solo presente o ausente: el token
nunca se imprime), el mapeo local -> remoto (`feature #15 -> ADR-42`), el sprint
vigente y cuantos intents quedan pendientes.

## Reglas que la integracion respeta siempre

- **El flujo manda.** Jira y Confluence reflejan; no al reves. Un issue movido a
  mano en el board no reescribe `feature_list.json`.
- **Nunca bloquea.** Registrar la intencion es best-effort: si Atlassian esta
  caido o el disco falla, el comando del arnes sigue su curso igual.
- **Una sola vez por caso.** Cada intent tiene su clave de dedupe; repetir un
  comando no duplica issues ni comentarios.
- **El PRD es del USUARIO.** Publicar lo lee; jamas lo reescribe.
- **Sin secretos.** El token vive solo en el entorno o en `.harness.env`
  (ignorado por git) y no aparece en la outbox, el estado, los logs ni los
  commits.
