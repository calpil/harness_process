# Plan - Feature #26: rutas_protegidas_deny

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 4 del `PRD-master`: los PRD y la constitution dejan de depender de la buena
fe. Una lista de rutas protegidas (`rules.rutas_protegidas`, con
`docs/prd/**`, `docs/constitution.md` y `.env` por defecto) y **tres capas** que
la hacen valer, cada una con su alcance declarado:

| Capa | Que puede | Que NO puede |
| --- | --- | --- |
| `PreToolUse` (Claude) | **impedir** la escritura | existir en backends que no tienen el evento |
| `PostToolUse` (todos) | avisar al instante con el comando de reversion | **impedir**: corre despues |
| `harness_check.sh` | bloquear el cierre (exit 2) | actuar en el momento del dano |

Spec aprobado (21 AC, cada uno con su `Comando:`):
`docs/spec-feature-26-rutas-protegidas-deny.md`.

## Peldano elegido: 1 (extender lo que ya existe)

Segunda aplicacion de la escalera de la #24, y esta vez **contradice al PRD**,
que proponia un archivo `harness.deny` (peldano 4).

| Peldano | ¿Alcanzaba? |
| --- | --- |
| **1. extender lo que existe** | **SI, elegido.** La lista va en `rules` de `feature_list.json`, donde ya viven `require_spec_approved`, `require_leccion` y `require_verify_green` — el usuario ya edita ese objeto a mano. El matcher entra en el binario, que ya se invoca desde los hooks. El chequeo entra en `harness_check.sh`, que ya tiene bloques opcionales |
| 2. flag en un comando existente | no aplica: no hay nada que parametrizar |
| 3. comando nuevo | innecesario: la consulta la hacen los hooks, no una persona |
| 4. superficie nueva (`harness.deny`) | **descartado.** Sumaria un archivo que sembrar, espejar, documentar y mantener sincronizado, para guardar tres lineas de configuracion que tienen su lugar natural al lado de las otras tres reglas |

**Peldano elegido: 1 (extender lo que ya existe) porque la configuracion cabe
donde ya viven las otras tres reglas del arnes, y un archivo aparte solo agregaria
superficie que sembrar, espejar y documentar sin darle nada al usuario.**

Decision del usuario (OBS-1). Vale registrar que la escalera **cambio lo que
decia el PRD**: es la segunda vez que hace trabajo real y no tramite.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder, como en las nueve features anteriores.

Impacto por inspeccion (un microservicio, `harness`):

- `rust/src/rutas.rs` (NUEVO): el matcher de globs, **puro**.
- `rust/src/commands/rutas.rs` (NUEVO, interno): la consulta que hacen los hooks.
- `rust/src/doctor.rs`: un area mas (AC-16), sin duplicar el chequeo.
- `harness_check.sh` (+ espejo): la red de seguridad, que **bloquea**.
- `setup_harness.sh` / `.ps1`: el `PreToolUse` de Claude y el `PostToolUse`.
- `tests/deny_check.sh` (NUEVO): los AC de shell.
- `docs/rutas-protegidas.md` (+ plantilla), README, UPDATING, roles.

**El riesgo de esta feature es distinto y grave**: es la primera que puede
**impedirle trabajar al agente**. Un matcher demasiado ancho, o que no distinga
al binario del arnes del agente, deja el proyecto trabado. Por eso el AC-9 y el
AC-10 existen y por eso los defaults son tres rutas y no un directorio entero.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "hooks PostToolUse settings"` y
`"constitution PRD del usuario"`. Lo que decidio el plan:

- **`PostToolUse` corre despues de la herramienta.** El PRD decia "el hook
  PostToolUse bloquea la escritura", y eso no es alcanzable: el arnes cablea
  `SessionStart`, `PostToolUse` y `Stop`, ninguno previo. De ahi salieron las
  tres capas con su alcance declarado en vez de una promesa de bloqueo.
- **`close` escribe en `docs/prd/PRD-master.md`** cada vez que marca un hito (se
  vio en el cierre de la #24 y la #25). La proteccion tiene que ser contra las
  herramientas del AGENTE, no contra el binario. El hook matchea
  `Edit|Write|MultiEdit`, asi que la prevencion lo respeta sola; la red de
  seguridad, que mira `git status`, **no** — y ahi hace falta el AC-10.
- El bloque de conventions de la #24 y el de lecciones de la #17 dan la forma
  exacta del bloque nuevo en `harness_check.sh`: se omite entero si no hay
  configuracion, y aca ademas **suma a `failures`** (decision OBS-4).

## Delegacion (implementer)

- **D1 (AC-1..AC-4)** — `rust/src/rutas.rs`: matcher **puro** de globs con `*`
  (un segmento) y `**` (cualquier profundidad), normalizacion de rutas absolutas
  y relativas contra la raiz, y lectura de `rules.rutas_protegidas` con los tres
  defaults. Sin heuristicas: la lista es la unica fuente.
- **D2 (AC-11, AC-12, AC-13)** — La configuracion: lista ausente -> defaults;
  lista propia -> se respeta; lista **vacia explicita** -> proteccion apagada.
  Los tres estados son distintos y cada uno tiene su test.
- **D3 (AC-6)** — El aviso del `PostToolUse`: el hook consulta al binario y, si
  hay violacion, imprime la ruta y `git checkout -- <ruta>`. **No revierte**
  (OBS-2): avisa, como doctor (#25) y el curador (#21).
- **D4 (AC-5)** — El `PreToolUse` de Claude en `setup_harness.sh` (+ `.ps1`):
  deniega la escritura sobre una ruta protegida. **Limite declarado**: no se
  puede probar de punta a punta aca; el AC verifica el JSON generado y el
  comportamiento del script, no una denegacion real de Claude Code.
- **D5 (AC-7, AC-10, AC-14)** — La red de seguridad en `harness_check.sh`:
  reporta las rutas protegidas modificadas y sin commitear, con su comando de
  reversion, y **suma a `failures`** (exit 2). Sin configuracion, el bloque se
  omite entero.
- **D6 (AC-9)** — Que el arnes no se bloquee a si mismo: test de que `close`
  marca el hito en el PRD protegido y cierra normalmente.
- **D7 (AC-15, AC-16)** — Costo del hook (sin violacion no cuesta ni bloquea) y
  el area de `doctor` que informa si la proteccion esta activa, sin duplicar el
  chequeo de violaciones.
- **D8 (AC-8, AC-17..AC-21)** — `docs/rutas-protegidas.md` (+ plantilla) diciendo
  **explicitamente que PostToolUse no puede prevenir**, docs, roles, y la
  verificacion oficial.

## Criterios de cierre (reviewer)

Escritos para poder fallar (`criterios-de-cierre-que-se-pueden-fallar`) y
verificados contra datos reales (`probar-contra-datos-reales`):

- Evidencia por AC-1..AC-21 en `docs/impl-26.md`; veredicto en `docs/review-26.md`.
- `sh harness_cli verify --feature 26` **verde**, con sus 21 comandos.
- **La prueba del rojo, sobre este repo**: tocar `docs/constitution.md` a mano,
  confirmar que `harness_check.sh` lo reporta con el comando de reversion y sale
  2; revertir; confirmar que vuelve a limpio.
- **El arnes cierra la propia #26** con el PRD protegido: si el cierre de esta
  feature se bloquea a si mismo, la feature esta mal hecha, y se ve en el acto.
- **Cero falsos positivos**: `harness_check.sh` en este repo, con trabajo en
  curso, no reporta rutas que no esten en la lista.
- **El aviso da un comando que funciona**: se copia el `git checkout -- <ruta>`
  que imprime y se comprueba que restaura de verdad.
- **La proteccion se puede apagar**: lista vacia -> ni aviso ni bloqueo.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash harness_check.sh`: todo verde.
- Hito 4 del `PRD-master` marcado por el cierre, con declaracion de leccion.

## Riesgos

- **Trabar el proyecto.** Es el riesgo central y es nuevo: ninguna feature
  anterior podia impedirle escribir al agente. Mitigado por tres defaults
  acotados, por la lista vacia como interruptor (AC-13), por
  `HARNESS_CHECK_MODE=warn` y porque el arnes queda fuera de la proteccion
  (AC-9).
- **Prometer bloqueo donde solo hay deteccion.** Seria repetir el error que la
  #25 evito con el hub ("alcanzable" cuando solo se midio TCP). Mitigado por el
  AC-8, que lo vuelve verificable en la documentacion.
- **El `PreToolUse` sin probar de punta a punta.** Declarado como limite, no
  disimulado. Si el contrato de hooks de Claude Code cambiara, la capa 1 dejaria
  de funcionar en silencio — por eso las capas 2 y 3 no dependen de ella.
- **Falsos positivos en la red de seguridad.** `git status` ve todo lo
  modificado, incluido lo que escribio el propio arnes. El AC-10 lo cubre para el
  PRD; conviene revisar la salida real antes de cerrar.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cuatro del spec fueron decididas por Alan el 2026-08-17
antes de aprobar:

- OBS-1 la lista vive en **`rules.rutas_protegidas`** -> D1, D2 y el peldano.
- OBS-2 la deteccion **avisa**, no revierte -> D3.
- OBS-3 **se agrega `PreToolUse`** para Claude, con el limite declarado -> D4.
- OBS-4 la red de seguridad **bloquea** (exit 2) -> D5.

## Skills aplicadas

- **`rust-patterns`**: el matcher es una funcion **pura** (ruta + lista ->
  bool). Ninguna de las tres capas puede escribir, porque el modulo no importa
  nada que escriba (leccion `promesas-estructurales-vs-disciplina`).
- **`rust-best-practices`**: peldano 1 — la configuracion donde ya vive la
  configuracion, el matcher donde ya se invoca el binario, el chequeo donde ya
  hay chequeos. Cero dependencias nuevas (Articulo 6).
- **`rust-testing`**: los tres estados de la configuracion (ausente, propia,
  vacia) son tres tests distintos, porque "ausente" y "vacia" significan cosas
  opuestas y confundirlos dejaria el proyecto desprotegido creyendo lo contrario.

### Avance 2026-08-17T20:20:00Z
Plan de la #26 escrito: D1-D8 citando cada AC. La escalera contradijo al PRD (peldano 1 en vez del archivo harness.deny que proponia) y el diseno cambio dos veces por hechos verificados: PostToolUse no puede prevenir (corre despues), y close escribe en el PRD protegido, asi que la proteccion es contra las herramientas del agente y no contra el binario.

### Avance 2026-08-17T20:14:40Z
Plan de la #26 escrito: D1-D8 citando cada AC. La escalera contradijo al PRD (peldano 1, rules.rutas_protegidas, en vez del archivo harness.deny que proponia) y el diseno cambio dos veces por hechos verificados: PostToolUse corre DESPUES y no puede prevenir, y close escribe en docs/prd/PRD-master.md, que es la primera ruta a proteger, asi que la proteccion es contra las herramientas del agente y no contra el binario del arnes.

### Avance 2026-08-18T01:16:25Z
Feature #26 implementada: rutas protegidas con tres capas (PreToolUse previene en Claude, PostToolUse avisa con el comando de reversion, harness_check bloquea con exit 2), lista en rules.rutas_protegidas y el arnes exento de su propia proteccion por registro con mtime. Incidente grave y corregido: el remedio que la herramienta imprimia (git checkout -- ) se corrio y borro los hitos sin commitear de #23-#25; reconstruidos y verificados con prd tree. El remedio ahora muestra el diff primero y etiqueta que DESCARTA.

---
Cerrado: 2026-08-18T01:16:36Z - status=done - Rutas protegidas con tres capas y su alcance declarado: PreToolUse previene (solo Claude, limite de prueba declarado), PostToolUse avisa con el comando de reversion, harness_check bloquea con exit 2. El arnes queda exento de su propia proteccion por registro con mtime, que caduca en cuanto alguien vuelve a tocar el archivo.
