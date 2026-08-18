# Evidencia de implementacion - Feature #26: rutas_protegidas_deny

Spec: `docs/spec-feature-26-rutas-protegidas-deny.md` (`Estado: approved`, 21 AC)
Plan: `docs/plan-feature-26-rutas-protegidas-deny.md` (D1-D8, con `Peldano elegido:`)
PRD: `docs/prd/PRD-master.md` (hito 4)

## Lo primero, porque es lo mas importante

**El remedio que esta herramienta imprimia destruyo trabajo real durante su
propio desarrollo.** La primera version, ante una ruta protegida tocada, decia:

```
docs/prd/PRD-master.md    git checkout -- docs/prd/PRD-master.md
```

Lo corri tal cual, sobre este repo, y **borro los hitos y la bitacora de las
features #23, #24 y #25**, que estaban marcados pero sin commitear. `git checkout`
no revierte "el cambio del agente": revierte el archivo entero a HEAD, con todo
el trabajo legitimo que hubiera encima. En un repo donde nada esta commiteado
—como este ahora mismo— eso es "tirar todo".

Reconstrui el PRD (tabla de 5 hitos, fila del PRD anidado y bitacora de #16 y
#23-#25) y lo verifique con `prd tree`: `PRD-master 5 hitos | features: 19/21
done`, con `PRD-aprendizaje` colgando. No se perdio nada, pero se pudo perder, y
por eso el hallazgo esta arriba de todo en vez de en una nota al pie.

El remedio ahora es:

```
docs/constitution.md
    mira que cambio: git diff -- docs/constitution.md | y si no fue tuyo:
    git checkout -- docs/constitution.md (DESCARTA todo lo no commiteado de ese archivo)
```

Primero mirar, despues decidir, y el comando destructivo **dice que destruye**.
Esta encodeado en un test (`remedio_should_show_the_diff_before_the_destructive_command`)
que exige que el `git diff` aparezca ANTES del `git checkout` y que este la
palabra `DESCARTA`.

## Peldano elegido: 1 (extender lo que ya existe)

Segunda aplicacion de la escalera de la #24, y **contradijo al PRD**, que pedia
un archivo `harness.deny` (peldano 4). La lista vive en `rules.rutas_protegidas`
de `feature_list.json`, donde ya viven las otras tres reglas y donde el usuario
ya edita a mano. Cero superficie nueva.

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/rutas.rs` | D1, D2 | NUEVO. Matcher **puro** de globs, config de tres estados, `violaciones()`, exenciones; 15 tests |
| `rust/src/commands/rutas.rs` | D3, D5 | NUEVO. `--check`, `--violaciones`, `--aceptar-estado-actual`, y el registro de escrituras del arnes |
| `rust/src/commands/close.rs`, `commands/prd.rs` | D6 | Registran sus propias escrituras sobre rutas protegidas |
| `rust/src/doctor.rs` | D7 | Area `rutas_protegidas`: informa el estado, no las violaciones |
| `harness_check.sh` (+ espejo) | D5 | La red de seguridad, que **bloquea** (exit 2) |
| `setup_harness.sh` | D3, D4 | `run_pre_tool` (deniega), aviso en `run_post_tool`, `PreToolUse` en `.claude/settings.json` |
| `tests/deny_check.sh` | D3-D5 | NUEVO. Seis modos, uno por AC |
| `docs/rutas-protegidas.md` (+ plantilla), README, UPDATING (+ espejo), roles | D8 | Las tres capas con su alcance |

## Evidencia por AC

`sh harness_cli verify --feature 26`: **21 verde, 0 rojo, 0 manual**.

| AC | Evidencia |
| --- | --- |
| AC-1 | `deny_should_protect_the_three_defaults` |
| AC-2 | `deny_should_match_globs_at_any_depth` (`**` a cualquier profundidad, `*` un segmento, `*` parcial) |
| AC-3 | `deny_should_normalize_absolute_and_relative_paths` |
| AC-4 | `deny_should_not_guess_beyond_the_list` (`constitution.md.bak` NO se protege) |
| AC-5 | `bash tests/deny_check.sh previene` — con el limite declarado abajo |
| AC-6 | `bash tests/deny_check.sh detecta` |
| AC-7 | `bash tests/deny_check.sh red-de-seguridad` (exit 2) |
| AC-8 | `docs/rutas-protegidas.md` dice "no puede prevenir" en la tabla de capas |
| AC-9 | `close_should_still_write_the_prd_milestone_when_protected` |
| AC-10 | `bash tests/deny_check.sh no-se-autobloquea` |
| AC-11 | `deny_should_read_user_defined_paths` (la lista propia reemplaza, no suma) |
| AC-12 | `deny_should_fall_back_to_defaults_when_unconfigured` (incluye tipo equivocado) |
| AC-13 | `deny_should_be_disablable_with_an_empty_list` |
| AC-14 | `bash tests/deny_check.sh compatible` |
| AC-15 | `bash tests/deny_check.sh sin-costo` |
| AC-16 | `doctor_should_report_protected_paths_status` |
| AC-17 | `diff -q` doc vs plantilla |
| AC-18 | README + UPDATING + espejo |
| AC-19 | rol del implementer y del reviewer |
| AC-20 | `Peldano elegido:` en el plan |
| AC-21 | 279 + 126 tests, clippy 0, setup_smoke verde, harness_check limpio |

## Las tres capas, y por que hay tres

El PRD decia "el hook `PostToolUse` bloquea la escritura". **No es alcanzable**:
`PostToolUse` corre despues de la herramienta. El arnes cablea `SessionStart`,
`PostToolUse` y `Stop`, ninguno previo. De ahi salio el diseno en capas, cada una
con su alcance escrito en la tabla de `docs/rutas-protegidas.md`:

- **Prevenir** — `PreToolUse` (nuevo) para Claude Code, que devuelve
  `permissionDecision: deny`. Es lo unico que actua antes del modo permisivo.
- **Detectar** — `PostToolUse`, en todos los backends, con el comando de
  reversion.
- **Red de seguridad** — `harness_check.sh`, que bloquea el cierre.

Las capas 2 y 3 **no dependen** de la 1, precisamente porque la 1 es la que no se
pudo probar de punta a punta.

## El arnes no se bloquea a si mismo

`close` escribe en `docs/prd/PRD-master.md` al marcar un hito y `prd add` crea
PRDs bajo `docs/prd/`: las dos son rutas protegidas por defecto. Si la proteccion
las alcanzara, el arnes se trabaria a si mismo en cada cierre.

La solucion: cuando el binario escribe una ruta protegida, la anota en
`progress/.rutas_arnes` junto con el **mtime**. La exencion vale solo mientras
nadie vuelva a tocar el archivo — si el agente lo edita despues, el mtime cambia
y la exencion caduca sola. Probado en `exenciones_should_expire_when_the_file_changes_again`.

## Dos falsos positivos que aparecieron en la primera corrida real

### 1. `git checkout --` sobre una ruta sin trackear no hace nada

`docs/prd/aprendizaje/` es un directorio **sin trackear** (lo creo `prd add`).
El remedio ofrecido era `git checkout -- docs/prd/aprendizaje/`, que sobre algo
que git no conoce no revierte nada: un remedio que no remedia. Ahora el remedio
depende del estado de tracking, y para lo no trackeado ofrece `rm -r` etiquetado
como `BORRA`, condicionado ("si no la pusiste vos") en vez de como orden.

### 2. Adoptar la proteccion con trabajo en curso arranca en rojo

En este repo, al activar la red de seguridad, aparecieron dos violaciones que
**nadie hizo mal**: escrituras legitimas del arnes anteriores al registro. Un
gate que arranca en rojo por algo que esta bien se apaga en dos dias.

Por eso existe `sh harness_cli rutas --aceptar-estado-actual`: toma el estado
actual como linea de base. Es explicito y lo corre una persona, como `--aplicar`
del curador (#21); el chequeo por si solo nunca escribe. Sin esto, el AC-14
(compatibilidad con instalaciones existentes) seria una promesa vacia, porque
**toda** instalacion existente esta en esa situacion.

## La prueba del rojo, sobre este repo

```
$ printf '\n<!-- toque de prueba -->\n' >> docs/constitution.md
$ bash harness_check.sh
[!] Rutas PROTEGIDAS modificadas y sin commitear:
    docs/constitution.md
        mira que cambio: git diff -- docs/constitution.md | y si no fue tuyo:
        git checkout -- docs/constitution.md (DESCARTA todo lo no commiteado de ese archivo)
rc=2
$ # restaurada
[Ok] Harness Check limpio.
```

Detecta, nombra la ruta, da el remedio honesto y cambia el exit code. Al
restaurar vuelve a limpio: el gate no queda pegado en rojo.

## Un defecto de mi propio bloque, encontrado corriendolo

La primera version del bloque en `harness_check.sh` capturaba
`rutas --violaciones 2>&1`, y el aviso informativo que el binario emite por
stderr (`[i] Checkout fuente del arnes detectado...`) aparecia **como si fuera una
ruta violada**. Se arreglo separando stdout de stderr y descartando cualquier
linea sin tab. Es el mismo error de la #25 con el hub, en otra forma: mostrar
como resultado algo que no lo era.

## Limites declarados

- **El `PreToolUse` no se probo de punta a punta** (OBS-3, decidido por Alan). El
  test verifica el JSON que el hook emite y que el instalador lo cablea sobre
  `Edit|Write|MultiEdit`, **no** una denegacion real de Claude Code, que exigiria
  correrlo. Por eso las capas 2 y 3 no dependen de la 1.
- **La extraccion de la ruta del tool call es por `sed`, no por parser JSON.** Si
  falla, el hook **deja pasar** (nunca bloquea el turno por no entender el
  input), y queda la capa de deteccion.
- **Solo Claude Code tiene prevencion.** Los demas backends conservan las capas 2
  y 3. Esta dicho en la tabla en vez de insinuar que todos estan cubiertos.
- **`tests/setup_smoke.ps1` sigue sin correrse** (undecima feature). Deuda del
  repo, levantada en `review-23.md`, `review-24.md` y `review-25.md`.

## Para el backlog

- **El registro `progress/.rutas_arnes` crece sin poda.** Una linea por escritura
  del arnes sobre ruta protegida; hoy son dos. Conviene limpiarlo cuando el
  archivo se commitea.
- **`PreToolUse` para los demas backends** cuando se pueda verificar el contrato
  de hooks de cada uno.
- **El hook consulta al binario en cada `Edit|Write|MultiEdit`.** Hoy cuesta poco
  (AC-15 lo mide), pero es un proceso por tool call: si alguna vez pesa, el
  matcher podria vivir en el propio hook.
