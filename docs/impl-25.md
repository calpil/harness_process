# Evidencia de implementacion - Feature #25: harness_doctor

Spec: `docs/spec-feature-25-harness-doctor.md` (`Estado: approved`, 20 AC, cada
uno con su `Comando:`)
Plan: `docs/plan-feature-25-harness-doctor.md` (D1-D8, con `Peldano elegido:`)
PRD: `docs/prd/PRD-master.md` (hito 3)

## Peldano elegido: 3 para el diagnostico, 1 para el lanzador

Primera aplicacion real de la escalera de la #24, y salio **hibrida**. La tabla
completa esta en el plan; el resumen:

- El **peldano 1** (un bloque en `harness_check.sh`) se descarto porque ese
  script **bloquea el proceso** desde los hooks: una instalacion a medias pasaria
  a impedir commitear, que es una regresion disfrazada de mejora.
- El **peldano 2** (`harness_check.sh --install`) tiene una ventaja real y
  grande —funcionaria con el binario roto— pero obligaria a **reimplementar en
  shell la resolucion de rutas del binario**. Esa duplicacion ya costo la feature
  #10 entera; hacerla de nuevo para diagnosticarla es reabrir el bug que se
  quiere detectar.
- El **peldano 3** (comando nuevo) quedo para el diagnostico, y **la mitad que
  el peldano 3 no puede cubrir por definicion tomo el peldano 1**: un doctor que
  vive en el binario no puede diagnosticar un binario ausente, asi que ese caso
  se resolvio extendiendo `harness_cli`, que ya existia.

La escalera no fue un tramite: partio la feature en dos y cambio el diseno.

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/doctor.rs` | D1-D4 | NUEVO. `Estado`/`Area` como enums, `diagnosticar()` **pura**, las siete areas; 13 tests |
| `rust/src/commands/doctor.rs` | D5 | NUEVO. Render con remedio por linea, `--json`, pie que remite a harness_check |
| `rust/src/cli.rs`, `main.rs`, `commands/mod.rs` | D5 | Cableado |
| `harness_cli` (+ espejo) | D6 | Traduce binario ausente y binario viejo al mismo remedio |
| `tests/doctor_launcher_check.sh` | D6 | NUEVO. Cuatro modos, uno por criterio |
| `rust/tests/cli_basics.rs` | D7 | 14 tests de integracion |
| `README.md`, `UPDATING.md` (+ espejo) | D7 | Las siete areas, el exit code y los dos limites |
| `setup_harness.sh` / `.ps1`, `templates/roles/implementer.md` -> `roles/` -> `.claude/agents/` | D7 | Superficies y rol |

## Evidencia por AC

`sh harness_cli verify --feature 25`: **20 verde, 0 rojo, 0 manual**
(`docs/verify-25.md`).

| AC | Evidencia |
| --- | --- |
| AC-1 | `doctor_should_report_every_area_on_a_healthy_install` (las siete areas presentes) |
| AC-2 | `doctor_should_print_an_exact_remedy_for_every_problem` (con una falla sembrada, no sobre un sandbox sano) |
| AC-3 | `doctor_should_separate_failures_from_warnings` |
| AC-4 | `doctor_json_should_expose_area_state_and_remedy` |
| AC-5 | `doctor_should_detect_a_binary_older_than_the_scripts` + la prueba del rojo a mano (abajo) |
| AC-6 | `doctor_should_detect_a_hook_pointing_nowhere` |
| AC-7 | `doctor_should_only_demand_surfaces_the_backend_uses` (pide CLAUDE.md, NO pide GEMINI.md) |
| AC-8 | `doctor_should_explain_which_root_it_resolved_and_why` |
| AC-9 | `doctor_should_treat_an_unreachable_hub_as_a_warning` |
| AC-10 | `doctor_should_split_required_and_optional_tools` |
| AC-11 | `doctor_should_report_graphify_as_optional` |
| AC-12 | `doctor_should_not_demand_surfaces_in_a_source_checkout` (0 fallas) |
| AC-13 | `sh harness_cli doctor` en ESTE repo: exit 0 |
| AC-14 | `doctor_should_not_duplicate_the_process_checks` |
| AC-15 | `doctor_should_not_write_anything` (huella ruta+mtime+tamano de todo el arbol) |
| AC-16 | `bash tests/doctor_launcher_check.sh` (cuatro modos) |
| AC-17 | README + UPDATING + espejo |
| AC-18 | rol del implementer + las dos superficies del instalador |
| AC-19 | `Peldano elegido:` en el plan |
| AC-20 | 263 + 123 tests, clippy 0, setup_smoke verde, harness_check limpio |

## La corrida real, verificada linea por linea

El criterio de cierre no era "sale 0": era revisar **a mano** que cada linea diga
la verdad.

```
[ok] binario       harness presente, ejecutable y al dia
[ok] marker        marker 'subdir' con guardrail de checkout fuente aplicado
[--] hooks         checkout fuente del arnes: aca no se instalan hooks
[--] superficies   checkout fuente del arnes: se generan al instalar
[ok] hub           ...:25605 acepta conexiones TCP (doctor no valida el handshake)
[i]  herramientas  requeridas ok (git, cargo); opcionales ausentes: pipx
[ok] graphify      graphify en el PATH
```

Verificacion manual de cada una:

| Linea | ¿Es verdad? |
| --- | --- |
| binario al dia | Si: recien compilado y copiado |
| marker + guardrail | Si: `.harness_layout` dice `subdir` y la raiz quedo aca por el guardrail de la #7 |
| hooks / superficies `no_aplica` | Si: este es el checkout fuente, correr el instalador aca es el footgun de la #7. `ls` confirma que no hay `CLAUDE.md`, `.claude/settings.json` ni `bin/` |
| hub | Ver abajo: **este es el que casi sale mal** |
| pipx ausente | Si: `command -v pipx` no devuelve nada |
| graphify en PATH | Si |

### El OK falso que casi se cuela

La primera version imprimia `[ok] hub  <host>:<port> alcanzable`. Es cierto que
el TCP conecta — y es **falso** como diagnostico: durante toda la sesion en que
se escribio esta feature, las operaciones del hub morian con
`connection reset by peer` y `connection timed out`. Un usuario que leyera
"alcanzable" habria descartado el hub como causa y buscado el problema en otro
lado.

La linea ahora dice exactamente lo que se midio:

```
acepta conexiones TCP (doctor no valida el handshake de PostgreSQL: si un
comando falla con 'connection reset' o 'timed out', el problema esta mas adentro)
```

No es una mejora cosmetica. Es la diferencia entre un diagnostico y un OK que
tranquiliza sin informar — el mismo fallo que la #23 encontro con `cargo test`
saliendo 0 sin correr nada, en otra forma. La leccion
`probar-contra-datos-reales` lo dice: verde no significa calibrado.

## La prueba del rojo, sobre el caso que mas duele

Los tests cubren las siete areas. Ademas se corrio a mano la que ya rompio dos
veces en este repo:

```
$ touch -t 202001010000 harness      # el binario "viejo"
$ sh harness_cli doctor
[!!] binario  el binario es mas viejo que harness_cli, harness_check.sh,
              setup_harness.sh: tipico de `git pull` sin re-correr el instalador
              Remedio: bash setup_harness.sh
exit=2
$ # restaurado
exit=0
```

Detecta, nombra **cuales** scripts lo superan, da el comando y cambia el exit
code. Y al restaurar vuelve a 0: el chequeo no quedo pegado en rojo.

## Tres cosas que se descubrieron implementando

### 1. Capturar stderr en el lanzador habria dejado mudos los comandos lentos

La primera version de `harness_cli` guardaba stderr en un archivo para poder
buscar `unrecognized subcommand`. Funcionaba, y habria sido un bug feo: con el
hub sin responder, `close` tarda ~90 segundos, y el usuario **no habria visto
nada** hasta el final. Se cambio por mirar el exit code y, solo cuando es el 2 de
clap, preguntarle al binario si conoce el subcomando (`harness help <sub>`). Un
subcomando conocido nunca dispara el aviso, y la salida nunca se buferea.

Quedo como modo propio del test (`no-buferea`), que corre el comando en segundo
plano y comprueba que la salida aparece **mientras** sigue vivo.

### 2. El test de no-solapamiento fallo dos veces por grepear prosa

- Primera version: prohibia las palabras "spec", "leccion", "perfil", "prd" en
  todo el stdout. Fallo por **la linea que el propio AC-14 exige**, la que remite
  a `harness_check.sh` para el proceso.
- Segunda version: las prohibia solo en el `detalle` de cada area. Fallo porque
  el area del hub explica que "lecciones, perfil, buscar y journey son archivos",
  que es informacion util para decidir si el hub caido importa.
- Version final: asserta sobre el **conjunto de areas**, que es dato estructurado
  — doctor no agrega un area de spec ni de leccion. Ya no depende de como este
  redactada la prosa.

Es la regla 1 de `docs/conventions.md` aplicada a mi propio test: contratos, no
snapshots de texto.

### 3. `head -n 1` con `pipefail` mata el script en vez de medir

El modo `no-buferea` original leia la primera linea con `| head -n 1`. Con
`set -Eeuo pipefail`, `head` cierra el pipe, el productor recibe SIGPIPE y el
script entero muere sin decir por que — el modo simplemente no aparecia en la
salida. Se reescribio sin pipe: background + archivo + `kill -0` para confirmar
que el proceso sigue vivo cuando ya hay salida.

## Limites declarados

- **Doctor no valida el handshake de PostgreSQL.** Comprueba TCP con timeout de
  2 segundos. Un handshake real usaria `connect_timeout` de 10s y haria del
  doctor un comando lento, que es un comando que nadie corre. La salida lo dice.
- **La deteccion de "binario viejo" es por mtime.** Un `touch` la engana. Se
  acepta: el caso real es `git pull`, que actualiza mtimes, y el remedio
  (re-correr el instalador) es idempotente, asi que un falso positivo cuesta poco.
- **`doctor` no puede diagnosticar un binario ausente.** Declarado en el AC-16 y
  cubierto por el lanzador, que es donde corresponde.
- **`tests/setup_smoke.ps1` sigue sin correrse** (decima feature). Ya levantado
  como deuda del repo en `review-23.md` y `review-24.md`.

## Para el backlog

- **El area de hooks no verifica el CONTENIDO de cada hook**, solo que el runtime
  exista y sea ejecutable. Un `.claude/settings.json` que apunte a otra ruta se
  le escapa.
- **`doctor --fix`** quedo fuera por decision (AC-15). Si alguna vez se agrega,
  tiene que ser detras de un flag explicito y avisando antes, como `--aplicar`
  del curador.
- **Un chequeo real del hub** (handshake, no TCP) como flag opcional
  `doctor --hub`, para quien quiera pagar los 10 segundos.
