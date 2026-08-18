# Evidencia de implementacion - Feature #17: lecciones_memoria_procedural

Spec: `docs/spec-feature-17-lecciones-memoria-procedural.md` (Estado: approved,
20 AC, sellado 2026-08-16T20:00:57Z)
Plan: `docs/plan-feature-17-lecciones-memoria-procedural.md` (D1-D10)
PRD: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 1)

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `templates/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md` | D1 | NUEVO. La guia: formato, orden de preferencia y la lista de que NO capturar |
| `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md` | D1 | Copia sembrada en este checkout (byte a byte igual al template) |
| `setup_harness.sh` | D2, D7 | `lecciones/...` en `HARNESS_DOCS`; superficie generada describe el comando y el gate |
| `setup_harness.ps1` | D2, D7 | Idem en `$script:HarnessDocs` y en `Write-AgentSurface` |
| `rust/src/lecciones.rs` | D3, D5 | NUEVO. Modelo, validacion de nombre, telemetria, `scan`, `parecidas` y el gate del cierre |
| `rust/src/commands/leccion.rs` | D4 | NUEVO. `list` / `show` / `nueva` / `usar` |
| `rust/src/cli.rs` | D4, D5 | `Command::Leccion` + `LeccionCommand`; `--leccion` y `--leccion-motivo` en `close` |
| `rust/src/main.rs`, `rust/src/commands/mod.rs` | D4 | Declaracion de los modulos nuevos |
| `rust/src/commands/close.rs` | D5 | Gate antes de mutar, campos opcionales en la feature, bitacora y mensaje |
| `roles/*.md` + `templates/roles/*.md` | D6 | Reglas de captura por rol |
| `.claude/agents/*.md` | D6 | Espejos regenerados (frontmatter + cuerpo del rol) |
| `docs/architecture.md`, `templates/docs/architecture.md` | D7 | Modulo `lecciones.rs` + seccion "Los tres almacenes de memoria" |
| `README.md`, `UPDATING.md` (+ espejo), `AGENTS.md` | D7 | Documentacion del comando, el formato y el gate |
| `harness_check.sh` + `templates/harness_check.sh` | D8 | Bloque de integridad del arbol de lecciones |
| `rust/tests/cli_basics.rs` | D9 | 6 tests de integracion |
| `tests/setup_smoke.sh`, `tests/setup_smoke.ps1` | D9 | Siembra, no-pisa, contenido de la guia, roles, superficie y supervivencia al reset |
| `docs/lecciones/docs-generados-por-el-instalador.md` | dogfood | La primera leccion real del proyecto, escrita en esta feature |

## Evidencia por AC

### AC-1 — Siembra e idempotencia

`lecciones/COMO-ESCRIBIR-UNA-LECCION.md` entra en `HARNESS_DOCS` /
`$script:HarnessDocs`, la unica lista que ya alimentaba siembra, reset targets y
migracion. **Cero codigo nuevo de instalador** (es el peldano de menor huella).

`tests/setup_smoke.sh` (verde, exit 0):

```
test -f "$SUBDIR_ROOT/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md"
test ! -d "$SUBDIR_HARNESS/docs/lecciones"          # vive en el docs/ de la RAIZ
test -z "$(find ... ! -name 'COMO-ESCRIBIR-UNA-LECCION.md')"   # nace sin lecciones
grep -q "$LECCION_GUIA_SENTINEL" ...                # el reinstall no pisa la guia
grep -q "$LECCION_SENTINEL" ...                     # ni una leccion ya escrita
```

Paridad en `tests/setup_smoke.ps1` (mismas aserciones; ver nota en AC-20).

### AC-2 — Formato

Frontmatter con `nombre`, `descripcion`, `triggers`, `relacionadas`, `origen`,
`usos`, `ultimo_uso`, `ultima_actualizacion`, `estado`; cuerpo con las cuatro
secciones. `lecciones::plantilla()` lo produce y el test
`plantilla_should_parse_as_a_valid_leccion` verifica que la propia plantilla
parsea y trae las cuatro secciones. La leccion real del repo
(`docs/lecciones/docs-generados-por-el-instalador.md`) es la prueba de uso.

### AC-3 — `leccion nueva`

```
$ sh harness_cli leccion nueva docs-generados-por-el-instalador
Leccion creada: docs/lecciones/docs-generados-por-el-instalador.md
  Completa descripcion (una oracion, max 80 caracteres) y triggers: son los
  campos que deciden si alguien la encuentra dentro de seis meses.
  Metodo: docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md
```

El archivo nace con `usos: 0`, `estado: activa`, `ultima_actualizacion` de hoy y
`origen: [17]` (la feature activa). Test:
`leccion_should_create_list_use_and_refuse_duplicates` (asserta `origen: [1]` en
el sandbox).

### AC-4 — Nombres de clase, sin escape hatch

```
$ sh harness_cli leccion nueva fix-espejo-16
'fix-espejo-16' no es un nombre de CLASE: empieza con 'fix-' (describe un arreglo puntual, no una clase).
    Si el nombre solo tiene sentido para la tarea de hoy, esta mal: patchea una
    leccion existente ('sh harness_cli leccion list') en vez de crear otra.
    Validos: 'espejo-de-roles', 'instalador-idempotente'.
$ echo $?
2
```

Las cinco reglas cubiertas por `validar_nombre_should_reject_session_artifacts`
(8 nombres: `fix-`, `debug-`, `audit-`, `hotfix-`, `feature`, `#`, fecha,
numero largo) y `validar_nombre_should_accept_class_level_names` (incluye
`hub-postgres-17`, que un rechazo por "cualquier digito" habria roto).
`leccion_nueva_should_reject_session_names_without_writing_anything` verifica que
**no se crea ni la carpeta**. No existe flag para saltear la regla (OBS-1).

### AC-5 — Duplicado empuja a patchear

```
$ sh harness_cli leccion nueva espejo-de-roles     # ya existente
La leccion 'espejo-de-roles' ya existe: docs/lecciones/espejo-de-roles.md
    Patchea esa (mirala con 'sh harness_cli leccion show espejo-de-roles') en vez de crear otra:
    la biblioteca busca POCAS lecciones de clase, ricas, no una lista plana.
$ echo $?
2
```

### AC-6 — `leccion list`

```
$ sh harness_cli leccion list
Lecciones: 1 (por uso)
  docs-generados-por-el-instalador    0 usos | nunca      | activa
      Sumar un doc al arnes es una linea en HARNESS_DOCS, no codigo nuevo.

$ sh harness_cli leccion list --json | tail -6
      "usos": 0,
      "ultimo_uso": "",
      "estado": "activa"
    }
  ],
  "rotas": []
}
```

Catalogo vacio (`leccion_list_should_explain_how_to_start_when_empty`):
`Sin lecciones todavia.` + como crear la primera, exit 0. Orden por uso
descendente verificado en `scan_should_sort_by_uses_desc`.

### AC-7 — `leccion show` y sugerencias

```
$ sh harness_cli leccion show espejo-de-rol
No existe la leccion 'espejo-de-rol' (docs/lecciones/espejo-de-rol.md).
    ¿Quisiste decir? espejo-de-roles
    Vela con 'sh harness_cli leccion list' o creala con 'leccion nueva'.
$ echo $?
2
```

Ranking por prefijo comun, sin dependencias nuevas
(`parecidas_should_rank_by_common_prefix`).

### AC-8 — `leccion usar` no toca el contenido

```
$ sh harness_cli leccion usar espejo-de-roles
Uso registrado en docs/lecciones/espejo-de-roles.md: 1 usos (ultimo: 2026-08-16).
```

`registrar_uso_should_not_touch_body_nor_ultima_actualizacion` verifica
`usos+1`, `ultimo_uso` de hoy, `ultima_actualizacion` **intacta** y cuerpo
identico. `parse_should_round_trip_unknown_keys_and_body` garantiza que una
leccion editada a mano (con claves que el binario no conoce) sale byte a byte
igual.

Hallazgo del pase de revision, corregido antes de cerrar:
`.gitattributes` normaliza a LF `*.sh` y los shims, **no** `*.md`, asi que un
checkout Windows puede traer una leccion con CRLF. La primera version de
`render()` unia el frontmatter con `\n` fijo y dejaba el archivo mixto (cabecera
LF, cuerpo CRLF) apenas se corria `leccion usar`. Ahora `Frontmatter` recuerda el
fin de linea del original y re-renderiza con el mismo; cubierto por
`parse_should_round_trip_crlf_files`, que ademas verifica el round-trip exacto y
que tras `registrar_uso` no quede ninguna linea en LF suelta.

### AC-9 — Funciona sin hub

`leccion_should_work_with_the_hub_unreachable` corre los cuatro subcomandos dos
veces —una normal y otra con `DB_HOST=127.0.0.1 DB_PORT=1`— y compara **exit
code y stderr**. Ningun camino de `lecciones.rs` ni de `commands/leccion.rs`
importa `graph`. Evidencia adicional del entorno real: el hub de esta maquina no
responde (`error connecting to server: connection timed out` en `start` y en
`graph impacto`) y todos los comandos de la feature funcionaron igual.

### AC-10 — Compatibilidad total sin la regla

`close_should_stay_identical_without_the_leccion_rule`: cierra sin `--leccion` y
asserta que `feature_list.json` **no contiene** la clave `"leccion"`. Las 16
features ya cerradas no se migran ni se tocan (OBS-5). La regla se lee con
`unwrap_or(false)`: ausente o `false` => gate mudo.

### AC-11 — Gate sin declaracion

```
[GATE] El cierre no declara que se aprendio y la regla require_leccion esta activa.
    Dos salidas validas:
      --leccion <clase>                        (patcheaste o creaste esa leccion)
      --leccion ninguna --leccion-motivo "..."   (no hubo nada que aprender, y por que)
    'Ninguna' es una salida real, pero no deberia ser la respuesta por default.
    Catalogo: sh harness_cli leccion list
```

exit 2, y el test verifica que la feature **sigue** `in_progress` (el gate corre
antes de mutar, como el gate SDD).

### AC-12 — Clase inexistente falla; clase existente se registra

```
[GATE] El cierre declara la leccion 'espejo-de-roles' y no existe (docs/lecciones/espejo-de-roles.md).
    Crea la clase con 'sh harness_cli leccion nueva espejo-de-roles' o corregi el nombre.
```

Con la clase creada: `Leccion declarada: espejo-de-roles`, `"leccion":
"espejo-de-roles"` en la entrada y la clase en la linea de `history.md`.
Cubierto por `close_gate_should_demand_a_declaration_and_accept_both_exits`.

### AC-13 — `ninguna` exige motivo

`--leccion ninguna` sin motivo => exit 2 nombrando `--leccion-motivo`. Con
motivo cierra, escribe `"leccion": "ninguna"` + `"leccion_motivo": "..."` y deja
`leccion=ninguna (trabajo mecanico)` en `history.md`. Mismo test.

Complemento: `close_gate_should_not_ask_for_a_leccion_when_blocking_a_feature`
— `blocked`/`pending` no piden leccion (son valvulas de escape).

### AC-14 — La guia lleva las reglas portadas

`docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md` tiene las secciones
`## La regla que ordena todo: primero patchear, crear al final`,
`## El nombre tiene que ser de CLASE` y `## Que NO capturar`, esta ultima con los
cinco items literales. Verificado por el smoke sh y ps1 con un `grep` por cada
uno de los cinco.

### AC-15 — Los tres roles

- `leader`: paso 4.1 (consultar `leccion list` antes de disenar) + regla de
  decidir la clase al cierre.
- `implementer`: paso 6 (`leccion usar`) + seccion completa
  "Aprendizaje: primero patchear, crear al final" con el orden y la lista
  anti-veneno.
- `reviewer`: verifica que la declaracion sea **honesta** (`ninguna` tras una
  feature con correcciones es `changes_requested`) y que la leccion no capture
  nada de la lista prohibida.

Espejos regenerados con la misma regla que `build_claude_agent` (frontmatter +
linea en blanco + cuerpo del rol). Gate de espejo verde:

```
$ bash harness_check.sh
[Ok] Harness Check limpio.
```

Diff `templates/roles/*.md` (con `__HREL__` sustituido) contra `roles/*.md`: OK
en los tres.

### AC-16 — Los tres almacenes en architecture.md

Seccion nueva "Los tres almacenes de memoria (decision usuario 2026-08-16)" con
la tabla hub=eventos / lecciones=procedimiento / perfil=preferencias y las tres
consecuencias vinculantes (archivos versionados, no agregan nada al hub, los
artefactos de feature no son un cuarto almacen). Espejado en
`templates/docs/architecture.md` en su forma de plantilla.

### AC-17 — README, UPDATING y superficies

- `README.md`: seccion "Lecciones: la memoria procedural del proyecto".
- `UPDATING.md` (+ espejo en `templates/`): seccion con el JSON de la regla y las
  dos advertencias (sin `--force`; `--reset` no borra lecciones).
- `AGENTS.md` de la raiz y las superficies generadas por ambos instaladores.
- Smoke: `grep -q 'docs/lecciones/'` y `grep -q 'require_leccion'` sobre el
  `AGENTS.md` **instalado**.

### AC-18 — Gate de integridad

Prueba real con tres lecciones sembradas a mano:

```
[!] docs/lecciones/desalineada.md declara 'nombre: otro-nombre' y el archivo se llama 'desalineada.md'. Corregi el frontmatter o renombra el archivo.
[!] docs/lecciones/rota.md no empieza con el frontmatter '---'. Formato en docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md.
[i] docs/lecciones/sin-triggers.md no declara 'triggers': nadie la va a encontrar por tema.
[Harness] Check fallo con 2 problema(s).
```

Dos bloqueos + un aviso, exactamente lo especificado (OBS-4). La guia se saltea
(no es una leccion). Sin `docs/lecciones/` el bloque entero se omite. Espejado en
`templates/harness_check.sh` (diff vacio).

### AC-19 — `--reset` no borra lecciones

La guia esta en `HARNESS_DOCS` (se limpia y se refresca); las lecciones **no
estan en ninguna lista**, y esa ausencia es lo que las salva. Smoke:

```
test ! -f "$RESET_TEST/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md"   # limpia la guia
grep -q "$LECCION_RESET_SENTINEL" "$RESET_TEST/docs/lecciones/espejo-de-roles.md"  # conserva la leccion
```

### AC-20 — Verificacion oficial

```
$ (cd rust && cargo test --locked)
test result: ok. 143 passed; 0 failed   (unitarios, 20 nuevos en lecciones.rs)
test result: ok.  50 passed; 0 failed   (integracion, 6 nuevos)

$ (cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)
Finished `dev` profile

$ bash tests/setup_smoke.sh
[Ok] setup smoke: Rust-only, gate de credenciales, layouts, reinstall, dry-run, version, reset.
[exited with code 0]

$ bash harness_check.sh
[Ok] Harness Check limpio.
```

**Limitacion declarada**: `tests/setup_smoke.ps1` **no se ejecuto** — esta
maquina es macOS y no tiene `pwsh` ni Windows PowerShell instalado
(`command -v pwsh` => no disponible). Las aserciones estan escritas en paridad
con las del smoke `sh` (mismos archivos, mismos greps) y quedan pendientes de
correrse en Windows. Es la unica parte del AC-20 sin evidencia de ejecucion.

## Decisiones aplicadas (todas del spec, ninguna tomada por el agente)

| OBS | Decision de Alan (2026-08-16) | Donde vive |
| --- | --- | --- |
| OBS-1 | Sin `--force` en la validacion de nombre | `lecciones::validar_nombre_de_clase` no acepta ningun bypass |
| OBS-2 | Clase inexistente al cerrar: **falla** | `lecciones::gate`, rama de existencia |
| OBS-3 | La guia es plantilla; las lecciones no | `HARNESS_DOCS` lleva solo la guia |
| OBS-4 | `harness_check.sh` **bloquea** por frontmatter ilegible | bloque nuevo, `failures++` |
| OBS-5 | Campo `leccion` opcional, sin migrar lo cerrado | `close.rs`, `if let Some(decl)` |
| PRD | Archivos en `docs/`, funciona sin hub | AC-9 + D10, y la seccion de architecture.md |
| PRD | `require_leccion` apagada por default | `unwrap_or(false)` |

## Nota del cierre (post-veredicto)

El primer `close --status done` dejo `PRD actualizado (bitacora)` sin marcar el
hito: la celda del slug en la tabla `## 10. Hitos -> features` estaba escrita
entre backticks y `prd::echo_close` la compara **literal** contra
`feature["name"]`. Error de formato del PRD, no del codigo. Corregidas las seis
filas y re-ejecutado el cierre, que ahora reporta
`PRD actualizado (hito marcado done + bitacora)` y `prd tree` cuenta
`features: 1/6 done`. Costo del re-cierre: una segunda linea `close` en
`progress/history.md` y un segundo `Cerrado:` al pie del plan (la bitacora del
PRD no se duplica, la detecta por id+nombre).

El hallazgo quedo capturado como la segunda leccion del repo,
`docs/lecciones/hitos-del-prd.md` — es durable, de clase, y verificado en esta
sesion.

## Riesgos pendientes para el reviewer

1. **`setup_smoke.ps1` sin ejecutar** (ver AC-20). Es la unica brecha de
   evidencia; el codigo del `.ps1` se modifico a ciegas respecto de su ejecucion.
2. **Exit code del gate**: `close --status done` con el gate de lecciones sale
   con **2** (lo que fija el AC-11 y lo que dice `architecture.md`: "2 = gate"),
   mientras que el gate SDD del mismo comando sale con **1**. Es una
   inconsistencia *preexistente* del gate SDD, no introducida aca, pero conviene
   que quede registrada: un hook que discrimine por exit code vera 1 para spec y
   2 para leccion.
3. **Los espejos `.claude/agents/*.md` se regeneraron a mano** con la misma regla
   que `build_claude_agent` (verificado por el gate de espejo, que quedo limpio),
   en vez de re-corriendo el instalador: este repo es el checkout FUENTE y correr
   `setup_harness.sh` aca es el footgun conocido. En una instalacion normal el
   remedio documentado (re-correr el instalador) sigue siendo el correcto.
