# Analisis de Hermes Agent (Nous Research) y oportunidades para el arnes

Fecha: 2026-08-16
Fuentes revisadas:

- Repo local `~/Downloads/hermes-agent-main` (162 MB, MIT, Python + TS).
- `AGENTS.md` del repo (81 KB: reglas de contribucion, escalera de huella,
  politica de tests).
- `website/docs/` (documentacion completa: 200+ paginas).
- Codigo de las piezas de aprendizaje: `agent/background_review.py`,
  `agent/turn_context.py`, `agent/turn_finalizer.py`, `agent/curator.py`,
  `agent/learning_graph.py`, `hermes_state_search.py`, `hermes_cli/curator.py`,
  `hermes_cli/journey.py`.

El foco de este documento es **el loop de aprendizaje** ("el agente que crece
contigo"), que es la parte que pediste mirar en serio. Al final hay un catalogo
corto del resto de ideas aprovechables.

---

## 1. Que es exactamente el "loop de aprendizaje" de Hermes

Hermes se vende como "el unico agente con un loop de aprendizaje integrado:
crea skills a partir de la experiencia, las mejora durante el uso, se auto-empuja
a persistir conocimiento, busca en sus propias conversaciones pasadas y construye
un modelo cada vez mas profundo de quien sos".

No es marketing vacio: son **seis mecanismos concretos** que se pueden leer en el
codigo. Los describo con el nivel de detalle necesario para copiarlos bien.

### 1.1 El nudge: el agente se auto-empuja (turn-based, no cron)

Dos contadores independientes viven en el agente:

| Contador | Donde | Default | Dispara |
| --- | --- | --- | --- |
| `_turns_since_memory` | `agent/turn_context.py:704` | cada **10 turnos** de usuario | revision de MEMORIA |
| `_iters_since_skill` | `agent/turn_finalizer.py:741` | cada **10 iteraciones** de tool-loop | revision de SKILLS |

Configurables (`memory.nudge_interval`, `skills.creation_nudge_interval`).
Cuando alguno se cumple, al **terminar** el turno (despues de entregar la
respuesta al usuario, nunca compitiendo con la tarea) se lanza un **fork en
background** del agente con un prompt de revision. Es best-effort absoluto: si
falla, se traga la excepcion (`turn_finalizer.py:765-775`).

Detalles que importan:

- Se **suprime** en sesiones de cron (`skip_background_review`): un fork cuesta
  ~30K tokens y sin humano presente no aporta.
- Puede correr en un **modelo mas barato** (`auxiliary.background_review`). Si el
  modelo es distinto al principal, no puede reusar el prompt cache, asi que en vez
  de re-enviar la conversacion entera manda un **digest** (turnos recientes
  verbatim + resumen de los viejos). Medido: 3-5x mas barato, con captura
  practicamente identica.

### 1.2 El prompt de revision: el activo mas valioso del repo

`agent/background_review.py` contiene tres prompts (`_MEMORY_REVIEW_PROMPT`,
`_SKILL_REVIEW_PROMPT`, `_COMBINED_REVIEW_PROMPT`). El de skills es el que vale
oro, porque resuelve el problema real de todo sistema auto-aprendiente: **no
envenenarse a si mismo**. Sus reglas, textuales en espiritu:

**Sesgo a actuar.** "Se ACTIVO. La mayoria de las sesiones produce al menos una
actualizacion. Una pasada que no hace nada es una oportunidad de aprendizaje
perdida, no un resultado neutro." Y al mismo tiempo: "'Nada que guardar' es una
opcion real pero NO deberia ser el default."

**Forma objetivo de la biblioteca.** Skills **a nivel de clase** con un SKILL.md
rico y un directorio `references/` para el detalle de cada sesion. **No** una
lista larga y plana de una-skill-por-sesion.

**Orden de preferencia (elegir la primera que aplique):**

1. **Patchear la skill que estuvo en juego** en esta sesion (la que se cargo o se
   leyo). Es la que corresponde extender.
2. Patchear un **paraguas existente** que cubra la clase (agregar subseccion,
   pitfall, o ampliar el trigger).
3. Agregar un **archivo de apoyo** bajo un paraguas existente:
   - `references/<tema>.md` — detalle de sesion (transcripciones de error,
     recetas de reproduccion, rarezas de un proveedor) **y** bancos de
     conocimiento condensado (docs de API, extractos autoritativos).
   - `templates/<nombre>.<ext>` — archivos semilla para copiar y modificar.
   - `scripts/<nombre>.<ext>` — acciones re-ejecutables (verificacion, probes).
   El SKILL.md gana un puntero de una linea al archivo nuevo.
4. **Crear una skill nueva a nivel de clase**, solo si nada existente cubre la
   clase. Regla dura de nombres: **prohibido** un numero de PR, un string de
   error, un nombre en clave, o `fix-X / debug-Y / audit-Z-hoy`. "Si el nombre
   propuesto solo tiene sentido para la tarea de hoy, esta mal: volve a 1, 2 o 3."

**Senales que justifican actuar** (cualquiera alcanza):

- El usuario corrigio tu estilo, tono, formato, verbosidad o legibilidad. La
  frustracion ("basta de X", "no formatees asi", "solo dame la respuesta") es
  una senal de skill de PRIMERA CLASE, no solo de memoria.
- El usuario corrigio tu **flujo de trabajo** o el orden de los pasos.
- Aparecio una tecnica, fix, workaround o camino de debug no trivial.
- Una skill que se consulto resulto **equivocada o desactualizada**: patchearla YA.

**Donde va cada cosa.** "La memoria dice *quien es el usuario y cual es el estado
actual de tus operaciones*; las skills dicen *como se hace esta clase de tarea
para este usuario*." Cuando el usuario se queja de como manejaste una tarea, la
leccion va en la skill que gobierna esa tarea — la memoria sola no alcanza.

**Que NO capturar jamas** (la lista anti-veneno; esto es lo que separa un sistema
que aprende de uno que se auto-sabotea):

- **Fallas dependientes del entorno**: binarios faltantes, errores de instalacion
  fresca, `command not found`, credenciales sin configurar. El usuario las
  arregla; no son reglas durables.
- **Afirmaciones negativas sobre herramientas** ("las tools de browser no
  funcionan", "X esta roto"). Se endurecen en negativas que el agente se cita a
  si mismo durante meses despues de que el problema se arreglo.
- **Errores transitorios** que se resolvieron solos. Si el reintento funciono, la
  leccion es el patron de reintento, no la falla.
- **Narrativas de tarea unica**. "Resumi el mercado de hoy" no es una clase de
  trabajo que merezca skill.
- **Fracasos no resueltos**: si la sesion termino sin encontrar un metodo que
  funcione, **no** escribir esos intentos como "flujo confiable". Eso presenta una
  secuencia de fracasos no testeada como guia validada que una sesion futura va a
  creer y repetir.

Y la regla de reencuadre: si una tool fallo por estado de setup, capturar **el
fix** (comando de instalacion, paso de config, env var) bajo una skill de setup o
troubleshooting — nunca "esta tool no funciona" como restriccion suelta.

**Skills protegidas** (el fork autonomo no las toca): bundled, instaladas del hub,
externas, **pinneadas**, y las **del usuario** (todo lo que no sea gestionado por
el curador). Si una de esas esta mal, el fork lo *dice* y recomienda adoptarla —
no la edita. Racional explicito: "sos un actor autonomo sin usuario presente".

### 1.3 Memoria acotada: el limite duro que obliga a consolidar

Dos archivos en `~/.hermes/memories/`:

| Archivo | Que guarda | Limite |
| --- | --- | --- |
| `MEMORY.md` | notas del agente: entorno, convenciones, lecciones | **2.200 chars** (~800 tokens) |
| `USER.md` | perfil del usuario: preferencias, estilo, expectativas | **1.375 chars** (~500 tokens) |

Decisiones de diseno que copiaria tal cual:

- **No auto-compacta.** Cuando una escritura excede el limite, la tool devuelve un
  **error accionable** con las entradas actuales listadas y la instruccion
  "consolida ahora: usa replace para fusionar entradas solapadas o remove para las
  rancias, y despues reintenta el add — todo en este mismo turno". El sistema
  nunca tira contenido en silencio; obliga al agente a decidir que sobra.
- **Snapshot congelado.** El bloque se inyecta al system prompt una sola vez al
  arrancar la sesion y no cambia a mitad de sesion (preserva el prefix cache). Las
  escrituras van a disco al instante pero recien se ven en la sesion siguiente.
- El bloque muestra el **porcentaje de uso** (`MEMORY [67% - 1.474/2.200 chars]`)
  para que el agente sepa cuanta capacidad le queda.
- Entradas separadas por `§`; `replace`/`remove` matchean por **substring unico**
  (si matchea dos entradas, error pidiendo mas especificidad).
- Rechazo de duplicados exactos y **escaneo de inyeccion/exfiltracion** antes de
  aceptar (la memoria entra al system prompt: es superficie de ataque).

### 1.4 Gate de aprobacion: `write_approval`

Porque un agente que escribe su propia memoria sin control es un agente que
eventualmente guarda una suposicion equivocada sobre vos y la arrastra para
siempre:

```yaml
memory:  { write_approval: true }   # stagea en vez de escribir
skills:  { write_approval: true }   # las skills SIEMPRE stagean (un SKILL.md no entra en un chat)
```

Con el gate prendido, las escrituras quedan en `~/.hermes/pending/` y se revisan
con `/memory pending|approve|reject` y `/skills pending|diff|approve|reject`.
Sobrevive reinicios. El diff completo se lee fuera de banda; en el chat solo va el
"gist" de una linea.

### 1.5 El curador: el ciclo de vida que evita la biblioteca-basura

Sin mantenimiento, un agente que crea skills termina con docenas de
casi-duplicados que contaminan el catalogo. `agent/curator.py` + `hermes_cli/curator.py`:

- **Disparo por inactividad, no por cron**: corre si pasaron `interval_hours`
  (default **168 h = 7 dias**) Y el agente estuvo idle `min_idle_hours` (default
  **2 h**). En una maquina activa, solo corre en los ratos muertos.
- **Fase 1, determinista (sin LLM)**: sin uso por `stale_after_days` (30) →
  `stale`; sin uso por `archive_after_days` (90) → archivada en `.archive/`.
  **Nunca borra.** Las nunca-usadas (`use_count == 0`) tienen piso de gracia: cero
  usos es ausencia de evidencia, no prueba de que sobra.
- **Fase 2, consolidacion con LLM**: **apagada por default** (cuesta tokens y hace
  cambios estructurales amplios). Fusiona solapadas en paraguas de clase.
- **Backup antes de cada pasada mutante** (`skills.tar.gz`, se conservan 5) y
  `curator rollback` — que a su vez toma un snapshot `pre-rollback`, asi que el
  rollback tambien es reversible.
- **Telemetria** en `.usage.json`: `use_count`, `view_count`, `patch_count`,
  `last_used_at`, `state`, `pinned`.
- **Reporte por corrida** en `logs/curator/<ts>/` con `run.json` + `REPORT.md`.
- **Primera corrida diferida**: en una instalacion nueva, la primera observacion
  siembra `last_run_at = ahora` y difiere la pasada real un intervalo completo.
  Te da tiempo de revisar y pinnear antes de que toque nada.
- `pin`, `adopt`, `restore`, `list-unmanaged`. Y una nota de diseno excelente:
  **la procedencia se declara, nunca se infiere.** "Una skill con miles de patches
  prueba que el agente la MANTIENE, no que la ESCRIBIO. Una heuristica
  'parece hecha por el agente, adoptala' eventualmente archivaria algo que
  escribiste vos."

### 1.6 Busqueda del propio pasado + mapa de lo aprendido

- **`session_search`**: SQLite con **FTS5** sobre todas las sesiones
  (`~/.hermes/state.db`). ~20 ms por consulta, ~1 ms por scroll, **sin llamadas a
  LLM y sin truncar**. Tres formas: descubrimiento, scroll dentro de una sesion, y
  browse. La tabla comparativa del doc es la tesis: la memoria es para hechos
  criticos siempre en contexto (costo fijo por sesion); la busqueda es para
  "¿hablamos de X la semana pasada?" (costo cero hasta que se usa).
- **`/journey`**: el grafo de "aprendizaje hecho visible" — skills aprendidas +
  chunks de memoria como nodos de primera clase, ploteados en el tiempo, con
  aristas por `related_skills` y por solapamiento lexico memoria↔skill. Y no es
  solo visual: `journey list|delete|edit` es donde **podas y corregis** lo que el
  agente aprendio.

---

## 2. Por que el arnes puede aprender MEJOR que Hermes

Hermes aprende de un transcript de chat: senal ruidosa, hay que adivinar que fue
una correccion y que fue charla. **El arnes ya produce, por diseno, un corpus de
aprendizaje muchisimo mejor**, y hoy lo tira:

| Material que el arnes ya genera | Que es, en terminos de Hermes |
| --- | --- |
| `docs/spec-feature-<id>.md` con AC-n Given/When/Then | el contrato de la tarea, explicito |
| `docs/plan-feature-<id>.md` con Observaciones OBS-n | los forks de diseno detectados |
| `nota=Alan aprobo ... Decisiones OBS-1..OBS-9` en `history.md` | **correcciones y preferencias del usuario, ya etiquetadas** |
| `docs/impl-<id>.md` (evidencia por AC) | que funciono, con prueba |
| `docs/review-<id>.md` (veredicto) | que se verifico y que quedo debiendo |
| `progress/history.md` append-only | la linea de tiempo completa, ya estructurada |
| Hub PostgreSQL + graphify | el grafo, ya existe |

Ejemplo real de `progress/history.md` de este repo:

```
2026-08-14T03:43:37Z approve-spec feature #14 estado=approved
  nota=... ante el fork del candado por proyecto eligio la opcion segura:
  escribir solo el delta
```

Eso es exactamente lo que el `_MEMORY_REVIEW_PROMPT` de Hermes intenta pescar de
un chat — y aca ya viene etiquetado, fechado y firmado. **La preferencia "ante un
fork de concurrencia, Alan elige la opcion segura" esta escrita en el repo desde
agosto y ningun agente la usa nunca.** Ese es el desperdicio que este programa
viene a cerrar.

Dos restricciones que condicionan todo el diseno:

1. **El arnes no es un agente.** No intercepta tool calls ni tiene un "turno". Su
   equivalente al nudge son los **hooks que ya instala** (`SessionStart`,
   `PostToolUse` con matcher `Edit|Write`, `Stop` en Claude y Kimi) y las
   transiciones del ciclo (`start`, `advance`, `approve-spec`, `close`).
2. **Multi-LLM obligatorio** (principio del arnes). Nada puede quedar pinneado a
   Claude. El patron es el ya acordado: override explicito → auto-deteccion por
   API key → fallback a CLI del backend → **skip limpio**. Y la mayor parte del
   loop **no necesita LLM**: el arnes puede emitir el *contrato* de la revision y
   dejar que el agente que este corriendo (el que sea) la ejecute, igual que hoy
   `atlassian drain` emite un plan de llamadas que ejecuta el MCP del agente.

---

## 3. El programa propuesto: "el arnes que aprende"

Seis piezas. Cada una es una feature del backlog; juntas son un PRD anidado
(`docs/prd/aprendizaje/`). Estan ordenadas por dependencia: A y D son la base.

### A. Lecciones — memoria procedural del proyecto

**Hoy:** cuando se cierra una feature, el conocimiento queda en `impl-<id>.md` y
`review-<id>.md`, ordenado **por feature**. Nadie los vuelve a leer: para
encontrar "como se hace un espejo de roles" hay que saber que fue la #7.

**Propuesta:** `docs/lecciones/<clase>.md`, organizadas **por clase de trabajo**,
no por feature. Formato tipo SKILL.md con frontmatter:

```markdown
---
nombre: espejo-de-roles
descripcion: Mantener roles/ como fuente unica y sus espejos por backend.  # <= 80 chars, una oracion
triggers: [roles, .claude/agents, .kimi-code/agents, espejo, harness_check]
relacionadas: [instalador-idempotente, gates-de-integridad]
origen: [feature #7, feature #9]
usos: 4
ultima_actualizacion: 2026-08-16
estado: activa
---

## Cuando aplica
## Procedimiento
## Pitfalls
## Verificacion
```

Comandos:

```bash
sh harness_cli leccion list                    # catalogo con usos y estado
sh harness_cli leccion show <clase>
sh harness_cli leccion nueva <clase>           # crea el esqueleto (falla si el nombre no es de clase)
sh harness_cli leccion usar <clase>            # +1 uso, sella last_used (telemetria del curador)
```

**Reglas portadas de Hermes, literales** (van en la plantilla y en los roles):

- Orden de preferencia: patchear la leccion que estuvo en juego → patchear el
  paraguas existente → agregar `docs/lecciones/<clase>/referencias/<tema>.md` →
  recien entonces crear una leccion nueva.
- **Nombres a nivel de clase.** `leccion nueva` **rechaza** nombres que contengan
  un id de feature, un mensaje de error, o el patron `fix-*/debug-*`. Si el nombre
  solo tiene sentido para la feature de hoy, esta mal.
- La lista completa de **"que NO capturar"** de la seccion 1.2, tal cual.

**Gate:** con la regla `require_leccion` activa, `close --status done` pide que la
feature declare o `leccion: <clase>` (patcheada/creada) o `leccion: ninguna` con
motivo. Igual que Hermes: "nada que guardar" es valido, pero hay que decirlo.

### B. Nudge de aprendizaje — el arnes se auto-empuja

**Hoy:** `harness nudge` solo avisa "sin feature activa" o "plan actualizado por
otro LLM", con debounce fijo de 600 s (`rust/src/commands/nudge.rs`).

**Propuesta:** dos disparadores nuevos, con el mismo caracter best-effort
(exit 0 siempre, cualquier error se traga):

1. **Por volumen de trabajo** (analogo al `_iters_since_skill` de Hermes): el hook
   `PostToolUse` que ya existe cuenta escrituras; cada N (default 10) el nudge
   emite el **contrato de revision de lecciones** por stderr, para que el agente
   que este corriendo lo ejecute con sus propias tools.
2. **Al cerrar** (`close --status done`): el nudge emite el contrato completo
   (memoria + leccion), que es el momento de maxima senal.

Ademas, **backoff adaptativo** copiado de `/loop` self-paced: hoy el debounce es
fijo en 600 s; si nada cambio, escalar 10m → 20m → 40m hasta un techo, y volver al
piso apenas cambia algo. Menos ruido, misma cobertura.

**Clave:** el nudge nunca escribe nada. Emite el contrato; el agente decide y
escribe; el gate del cierre verifica. Eso lo mantiene backend-agnostico.

### C. Perfil del usuario — el modelo de Alan que se profundiza

**Hoy:** las decisiones de Alan estan en `history.md` y en los `Observaciones` de
los planes, una por feature, sin agregacion. Cada agente nuevo arranca sin saber
que Alan ya decidio doce veces lo mismo.

**Propuesta:** `docs/perfil-usuario.md` — el `USER.md` del arnes, con las reglas
de Hermes:

- **Limite duro** (~1.500 chars). No auto-compacta: al pasarse, el comando falla
  con el error accionable y la lista de entradas actuales ("consolida y reintenta
  en este mismo turno").
- Se inyecta como **bloque delimitado** en las superficies generadas
  (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`, `GROK.md`) — el instalador ya
  las regenera desde heredocs, asi que el bloque entra sin infraestructura nueva.
  Snapshot congelado: cambia recien en la sesion siguiente.
- **Alimentacion semiautomatica**: `harness_cli perfil sugerir` lee `history.md` y
  los `Decision usuario:` de los planes y propone entradas candidatas **con la
  evidencia** (que features, que fechas). No escribe.
- **Escritura solo con tu si**, exactamente el ritual del spec: el agente te
  MUESTRA la entrada propuesta, PREGUNTA, y solo con el si corre
  `harness_cli perfil add --yes`. Es tu documento; el `write_approval` de Hermes
  aca es obligatorio, no opcional.

Ejemplo de entradas que ya se pueden derivar de este repo hoy:

```
Ante un fork de concurrencia o consistencia, elige la opcion segura aunque
cueste mas (candado por proyecto -> escribir solo el delta, feature #14).
§
Prefiere features amplias y completas antes que incrementales: amplio el spec
de la #15 a 25 AC y el de la #16 a 29 AC en vez de partirlas.
§
Exige sincronia total con sistemas externos: si algo se refleja en Jira, se
refleja TODO, incluido el backfill de lo ya cerrado (OBS-12/OBS-14, #16).
```

### D. Busqueda del propio historial

**Hoy:** para responder "¿donde decidimos usar ureq?" hay que hacer grep a mano
por `docs/` y `progress/history.md`.

**Propuesta:** `sh harness_cli buscar "<consulta>"` sobre specs, planes, impl,
review, lecciones, `history.md` y el hub. Salida rankeada con
`archivo:linea` + feature + fecha. **Sin LLM, sin dependencias nuevas**: escaneo
local (los artefactos son texto y son pocos), y si el hub Postgres esta arriba,
`to_tsvector` para la parte de eventos. Degrada limpio sin hub.

Es la pieza de mejor relacion valor/costo de todo el programa y no toca ningun
gate existente.

### E. Curador de lecciones

**Hoy:** nada (no hay lecciones todavia). Pero si A funciona, en seis meses hay 40
lecciones y la mitad son casi-duplicados.

**Propuesta:** `harness_cli lecciones status|pin|adoptar|archivar|restaurar`,
con el ciclo de vida de Hermes adaptado:

- Transiciones deterministas, sin LLM: sin uso 30 dias → `stale`; 90 dias →
  archivada en `docs/lecciones/.archivo/`. **Nunca borra.**
- Piso de gracia para las nunca usadas.
- `pin` para las que no se tocan.
- Backup en `bkp/` antes de cada pasada mutante (el arnes ya tiene `bkp/` y
  politica de backups del instalador) y `rollback` reversible.
- Reporte por corrida en `progress/lecciones/<ts>/REPORT.md`.
- **La procedencia se declara, nunca se infiere** — regla textual de Hermes.
- La consolidacion con LLM: **apagada por default**, opt-in, y multi-LLM.

### F. Mapa de aprendizaje (`journey`)

`harness_cli journey` — linea de tiempo de lo que el arnes aprendio: lecciones,
entradas de perfil y features cerradas, con sus enlaces. El arnes ya tiene el hub
y graphify: es render sobre datos que ya existen. Y, como en Hermes, con
`journey list|delete|edit` para **podar** — el valor no es el dibujo, es poder
corregir lo aprendido.

---

## 4. El resto del catalogo (fuera del loop de aprendizaje)

Ordenado por valor/costo. Ninguno es prerrequisito del programa de arriba.

| # | Idea de Hermes | Que falta en el arnes | Propuesta |
| --- | --- | --- | --- |
| 1 | **Quality gates** del `/goal` (comando shell que debe pasar antes de dar algo por hecho) + **completion contract** (`outcome / verification / constraints / boundaries / stop_when`) | los AC-n son prosa; `require_tests_to_close` es declarativo, lo verifica el reviewer a mano | el spec declara `Comando:` por AC-n; `harness_cli verify` los corre y escribe `docs/verify-<id>.md`; `close` exige verde. Convierte cada AC en contrato ejecutable |
| 2 | **Escalera de huella** (`AGENTS.md`): elegir siempre el peldano de menor huella, extender > CLI+skill > tool con gate > plugin > MCP > tool nueva | la constitution solo cubre dependencias nuevas (Art. 6) | portar la escalera a `docs/conventions.md`, adaptada: extender comando > flag > comando nuevo > superficie nueva > dependencia con ADR |
| 3 | **Politica de tests** de Hermes: contratos de comportamiento, **nunca leer el codigo fuente en un test**, prohibido el test detector-de-cambios | no esta escrito en ningun lado | tres reglas a `docs/conventions.md`; el reviewer las verifica |
| 4 | **`hermes doctor`** (3k lineas de diagnostico con remedio por linea) | `harness_check.sh` cubre el *proceso*, no la *instalacion* (binario, hooks, espejos, hub, PATH) | `harness_cli doctor [--json]`: por cada falla, el comando exacto de remedio |
| 5 | **Watchdog de frescura** (workflow que sondea cada 4 h y abre **un** issue, con dedupe por titulo: comenta en vez de duplicar) | `harness_check` corre a mano | workflow de ejemplo en `templates/`; con binding Atlassian, abre/actualiza **un** issue de Jira. Encaja con la #15/#16 |
| 6 | **`approvals.deny`**: globs que se bloquean **antes** del modo yolo | el README dice que los PRD son del usuario y nadie los reescribe, pero **no hay gate** | `harness.deny` con rutas protegidas (`docs/prd/**`, `.env`, `docs/constitution.md`); el hook `PostToolUse` bloquea la escritura |
| 7 | **Idempotency key** en `kanban create` | dos `add` seguidos crean dos features duplicadas | `add --idempotency-key` (o dedupe por nombre con aviso) |
| 8 | **Estados y dependencias** del kanban (`triage/todo/ready/running/blocked/review/done`), links padre→hijo, y el **circuit breaker**: tras N bloqueos por la misma causa la tarea va a `triage` para decision humana | `feature_list.json` tiene 4 estados planos y sin dependencias | `depends_on` en el backlog + estado `review`; `next` respeta dependencias; contador de re-bloqueos → exige decision tuya |
| 9 | **Cadena de AGENTS.md** por directorio (git-root → cwd, con dedupe y header de procedencia) + descubrimiento progresivo | superficies solo en la raiz; en multi-repo cada microservicio podria tener la suya | `--per-service-surfaces` en el instalador; `harness_check` valida el espejo |
| 10 | **Micro-compaction** (absorber un intercambio por turno en vez de una compresion grande; los mensajes del usuario **nunca** se compactan) | no aplica: el arnes no maneja contexto | solo la idea de fondo: "las instrucciones del usuario son de otra naturaleza, no se reconstruyen desde el trabajo que siguio". Ya es el espiritu del PRD que nadie reescribe |

### Lo que NO conviene traer

- **Gateway de mensajeria** (Telegram/Discord/Slack/WhatsApp): el arnes ya refleja
  el flujo en Jira/Confluence; una segunda superficie de notificacion es costo sin
  valor nuevo.
- **Backends de terminal remotos** (Docker/SSH/Modal/Daytona/Vercel Sandbox): el
  arnes no ejecuta, orquesta proceso.
- **Dispatcher con SQLite y workers como procesos**: `one_feature_at_a_time` es
  una decision de diseno deliberada del arnes; un tablero multi-worker la
  contradice. De ese subsistema solo vale lo de la fila 8.
- **Voice mode, TTS, pets, skins**: fuera de alcance.

---

## 5. Riesgos del programa de aprendizaje

1. **Que el arnes aprenda cosas falsas.** Mitigacion: la lista anti-veneno de 1.2
   portada literal, el gate de aprobacion para el perfil (es tu documento), y que
   el curador **nunca borre**.
2. **Solapamiento con el Memory Hub y graphify.** El hub guarda *eventos*; las
   lecciones guardan *procedimiento*; el perfil guarda *preferencias*. Son tres
   cosas distintas, pero hay que escribirlo en `architecture.md` para que nadie las
   confunda. **Es la decision de diseno mas importante a tomar antes de empezar.**
3. **LLM dentro del binario Rust.** Se evita: el arnes emite contratos, el agente
   ejecuta. Solo la consolidacion del curador (E) querria LLM, y va opt-in y
   multi-LLM.
4. **Ruido.** Un nudge cada 10 escrituras puede cansar. Por eso el backoff
   adaptativo y que el nudge escriba a stderr, best-effort, exit 0.

---

## 6. Estado del backlog al escribir esto

La feature **#16 (`atlassian_auto_push`) esta en progreso** con spec aprobado
(29 AC) e implementacion en curso. Con `one_feature_at_a_time` activa, nada de
esto puede arrancar hasta cerrarla: lo que sigue es cargar el backlog y decidir el
orden.
