# Veredicto del reviewer - Feature #17: lecciones_memoria_procedural

Spec: `docs/spec-feature-17-lecciones-memoria-procedural.md` (`Estado: approved`,
sello `Aprobado: 2026-08-16T20:00:57Z por USUARIO (confirmacion explicita)`,
20 AC)
Plan: `docs/plan-feature-17-lecciones-memoria-procedural.md` (D1-D10)
Evidencia: `docs/impl-17.md`
PRD de origen: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 1)

## Veredicto global: `approved`

19 AC cubiertos con evidencia ejecutada; **AC-20 parcial** por la unica brecha
conocida (`tests/setup_smoke.ps1` sin correr, sin PowerShell en esta maquina), el
mismo limite aceptado y documentado en las features #15 y #16.

## Trazabilidad de la aprobacion (Articulo 2)

- El spec lleva el sello de `approve-spec` con quien/cuando y las decisiones
  OBS-1..OBS-5.
- `progress/history.md` tiene su linea `approve-spec feature #17`.
- `sh harness_cli check-spec` => `[OK] Spec aprobado y fresco`.
- `sh harness_cli check-plan` => `[OK] Plan fresco para implementacion`.

Ningun agente aprobo por su cuenta: las cinco observaciones se decidieron en el
chat, en el mismo acto de aprobacion, y quedaron escritas en el spec **antes** de
la firma (por eso el spec no quedo stale).

## Estado por AC

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | Entrada unica en `HARNESS_DOCS`/`$script:HarnessDocs`; smoke sh asserta siembra en la RAIZ, carpeta sin lecciones, y dos sentinels de no-pisa (guia y leccion) |
| AC-2 | cubierto | `plantilla_should_parse_as_a_valid_leccion` (frontmatter completo + 4 secciones); la leccion real del repo lo usa |
| AC-3 | cubierto | Corrida real: crea con `usos: 0`, `estado: activa`, `origen: [17]`; test asserta `origen: [1]` en sandbox |
| AC-4 | cubierto | 8 nombres rechazados en unit test + integracion que verifica que **no se crea ni la carpeta**; `hub-postgres-17` sigue siendo valido |
| AC-5 | cubierto | Corrida real: exit 2 y mensaje que empuja a patchear |
| AC-6 | cubierto | `list` y `--json` verificados en real y en test; catalogo vacio explica como empezar; orden por uso en `scan_should_sort_by_uses_desc` |
| AC-7 | cubierto | Corrida real con typo: `¿Quisiste decir? espejo-de-roles` |
| AC-8 | cubierto | Test que asserta `usos+1`, `ultimo_uso` de hoy, `ultima_actualizacion` intacta y cuerpo identico |
| AC-9 | cubierto | Test que compara exit code y stderr con hub vivo vs `DB_HOST=127.0.0.1 DB_PORT=1`; ademas el hub real de la maquina esta caido y todo funciono |
| AC-10 | cubierto | Test que cierra sin la regla y asserta que **no** aparece la clave `leccion` |
| AC-11 | cubierto | exit 2, mensaje con las dos salidas, y la feature sigue `in_progress` |
| AC-12 | cubierto | Clase inexistente => exit 2 + sugerencia; clase existente => registra en la entrada y en `history.md` |
| AC-13 | cubierto | `ninguna` sin motivo => exit 2; con motivo => cierra y registra ambos campos |
| AC-14 | cubierto | Las tres secciones y los cinco items de "que NO capturar", con `grep` por cada uno en el smoke |
| AC-15 | cubierto | Los tres roles citan lo que les toca; gate de espejo limpio; diff `templates/roles` (con `__HREL__` sustituido) vs `roles/` OK en los tres |
| AC-16 | cubierto | Seccion "Los tres almacenes de memoria" con la tabla y las tres consecuencias; espejo en `templates/docs/architecture.md` |
| AC-17 | cubierto | README, UPDATING (+ espejo), AGENTS.md y ambos instaladores; el smoke greppea el `AGENTS.md` **instalado** |
| AC-18 | cubierto | Prueba real con 3 lecciones sembradas: 2 bloqueos + 1 aviso; sin `docs/lecciones/` el check sale limpio (verificado moviendo la carpeta) |
| AC-19 | cubierto | Smoke: la guia se limpia, la leccion con sentinel sobrevive |
| AC-20 | **parcial** | `cargo test` 143+50 verdes, `clippy --all-targets --all-features --locked -D warnings` limpio, `tests/setup_smoke.sh` exit 0, `harness_check.sh` limpio. **`tests/setup_smoke.ps1` NO ejecutado** (sin `pwsh` ni Windows PowerShell) |

## Constitution (`docs/constitution.md`)

| Articulo | Verificacion |
| --- | --- |
| 1 - Calidad y tests | `cargo test` (143 unit + 50 integracion, 26 nuevos), `cargo clippy` limpio, `tests/setup_smoke.sh` exit 0. Ningun test saltado. Brecha declarada: el smoke ps1 |
| 2 - Spec aprobado | Sello + linea en history + `check-spec` verde. Cumple |
| 3 - Trazabilidad AC-n | Cada D del plan cita sus AC; `impl-17.md` esta organizado por AC; este veredicto lista AC-1..AC-20 |
| 4 - Seguridad y observabilidad | Sin secretos (la guia ademas lo prohibe explicitamente y el grep de credenciales solo devuelve esa advertencia); errores accionables que nombran archivo y remedio; exit codes 0/1/2 estables |
| 5 - Decisiones del usuario | Las 5 OBS decididas por Alan antes de implementar; ninguna quedo abierta |
| 6 - Reglas puente | **Cero dependencias nuevas** (`git diff` sobre `rust/Cargo.toml` y `Cargo.lock` vacio: se reusan `regex` y `serde_json`); `templates/` y raiz espejados (verificado archivo por archivo); la feature es backend-agnostica por construccion — no invoca ningun modelo |

## Checkpoints

- [x] Feature activa refleja el estado real.
- [x] `check-plan` limpio.
- [x] `check-spec` limpio (`approved` + fresco).
- [x] Sin observaciones pendientes: las 5 con decision registrada.
- [x] Plan en `docs/` de la raiz y al dia con lo implementado.
- [x] `progress/current.md` apunta al plan con evidencia.
- [~] **Impacto**: `graph impacto` se intento y el hub **no responde**
  (`connection timed out`). Documentado en el plan, con el impacto derivado por
  inspeccion (un solo microservicio, `harness`). No bloquea: la feature esta
  especificada para funcionar sin hub y eso mismo se testea (AC-9).
- [x] `graphify query` consultado; sus hallazgos cambiaron el diseno (la entrada
      unica en `HARNESS_DOCS` en vez de codigo nuevo de instalador).
- [x] Tests relevantes ejecutados.
- [ ] `validate_ui.sh`: no aplica (sin frontend).
- [x] `docs/impl-17.md` y este veredicto mapean cada AC.
- [x] `harness_check.sh` limpio.

## Lo que el reviewer encontro y se corrigio antes de cerrar

**CRLF en lecciones (Windows).** `.gitattributes` normaliza a LF `*.sh` y los
shims, pero **no** `*.md`. Una leccion escrita en un checkout Windows puede venir
con CRLF, y la primera version de `render()` unia el frontmatter con `\n` fijo:
el primer `leccion usar` dejaba el archivo **mixto** (cabecera LF, cuerpo CRLF),
lo que ensucia el diff entero en el siguiente commit. Corregido: `Frontmatter`
recuerda el fin de linea del original y re-renderiza con el mismo. Cubierto por
`parse_should_round_trip_crlf_files`, que ademas atrapo un segundo detalle real
(la ultima linea del frontmatter llega con su `\r` pegado porque `str::lines`
solo saca el `\r` que precede a un `\n`).

## Riesgos que quedan abiertos

1. **`setup_smoke.ps1` sin ejecutar.** Las aserciones estan escritas en paridad
   exacta con las del smoke `sh`, pero el codigo `.ps1` se modifico sin correrlo.
   La primera corrida en Windows deberia confirmar siembra, carpeta sin
   lecciones, contenido de la guia y los dos greps sobre `AGENTS.md`.
2. **Exit codes distintos entre gates hermanos.** `close --status done` sale con
   **1** cuando bloquea el gate SDD y con **2** cuando bloquea el de lecciones.
   El 2 es lo que fija el AC-11 y lo que documenta `architecture.md` ("2 = gate");
   el 1 del gate SDD es **preexistente**. No se toco en esta feature (habria sido
   scope creep sobre un comportamiento que otros hooks pueden estar usando), pero
   conviene unificarlo en una feature propia.
3. **Los espejos `.claude/agents/*.md` se regeneraron a mano**, con la misma
   regla que `build_claude_agent`, porque este repo es el checkout FUENTE y
   correr el instalador aca es el footgun conocido. El gate de espejo quedo
   limpio, que es exactamente lo que valida esa equivalencia. En una instalacion
   normal, el remedio documentado (re-correr el instalador) sigue siendo correcto.

## Nota sobre la declaracion de cierre

Esta feature **si** deja leccion: la clase `docs-generados-por-el-instalador`,
escrita durante la implementacion y ya presente en el repo. Sale de un
aprendizaje real y verificable de esta sesion (la lista `HARNESS_DOCS` resuelve
siembra + no-pisa + reset + migracion con una sola entrada, y lo que NO se lista
es lo que sobrevive al `--reset`), no de una narrativa de la tarea. Cumple las
reglas de captura que la propia feature introduce.
