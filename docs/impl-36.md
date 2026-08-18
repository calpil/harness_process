# Evidencia de implementacion - Feature #36: deudas_anotadas_del_arnes

Spec: `docs/spec-feature-36-deudas-anotadas-del-arnes.md` (`Estado: approved`, 15 AC)
Plan: `docs/plan-feature-36-deudas-anotadas-del-arnes.md` (D1-D8, `Peldano elegido: 1`)
PRD: `docs/prd/PRD-master.md`

## Las seis deudas, pagadas

| Deuda | Anotada en | Que se hizo |
| --- | --- | --- |
| exit code de los gates de `close` | `impl-23` | `spec_gate` pasa de 1 a 2; los tres gates comparten codigo |
| `--solo` con varios AC | `impl-23` | `--solo AC-3,AC-7`, nombrando **cual** falta si alguno no existe |
| conventions solo miraba `rust/tests/` | `impl-24` | tambien recorre `rust/src/` |
| `.rutas_arnes` crecia sin poda | `impl-26` | se poda en cada consulta de violaciones |
| doctor no miraba a donde apunta el hook | `impl-25` | verifica que el settings de cada backend nombre el runtime |
| `leccion list` con ancho fijo 28 | hito #27 | ancho = el nombre mas largo, con piso en 28 |

## Evidencia por AC

`sh harness_cli verify --feature 36`: los 15 comandos.

| AC | Evidencia |
| --- | --- |
| AC-1 | `close_gates_should_share_one_exit_code` (spec y leccion, los dos en 2) |
| AC-2 | `close_should_keep_usage_errors_separate_from_gates` |
| AC-3 | `verify_solo_should_accept_several_acs` (corre AC-1 y AC-3, **no** AC-2) |
| AC-4 | `verify_solo_should_name_the_missing_ac` (nombra AC-9, no menciona AC-1) |
| AC-5 | `conventions_check.sh detecta-en-src` |
| AC-6 | `conventions_check.sh sin-violaciones` |
| AC-7 | `rutas_registro_should_drop_entries_that_are_no_longer_dirty` |
| AC-8 | `rutas_registro_should_keep_live_exemptions` |
| AC-9 | `doctor_should_detect_a_hook_pointing_to_another_path` |
| AC-10 | `doctor_should_stay_quiet_with_well_wired_hooks` |
| AC-11 | `leccion_list_should_size_the_column_to_the_longest_name` |
| AC-12 | `leccion_list_should_not_change_order_fields_or_json` |
| AC-13 | `deudas_check.sh backlog-cerrado` |
| AC-14 | el rol del implementer exige que "Para el backlog" entre al backlog |
| AC-15 | 283 + 132 tests, clippy 0 |

## Tres cosas que aparecieron al medir, no al planear

### 1. La nota del backlog estaba mal

`impl-23` decia que los tres gates salian "1 / 1 / 2". Al medirlo: el gate de
leccion **ya** salia 2 (`lecciones.rs:707`) y el unico distinto era el de spec
(`spec.rs`, via `Exit::msg`). Se movio un solo camino en vez de dos.

Es el **tercer caso en tres features** de una razon escrita sin verificar (la #30
tuvo dos). Por eso `probar-contra-datos-reales` ya lleva la seccion sobre las
razones que uno escribe.

### 2. Ampliar el alcance del chequeo destapo un bug que lo mataba en silencio

Al hacer que conventions mirara `rust/src/`, el chequeo dejo de reportar **nada**
— ni siquiera lo que antes reportaba. La causa:

```bash
conv_fn="$(head -n "$conv_num" "$conv_file" | grep -E '^(pub )?fn ...' | tail -1 | sed ...)"
```

Con `set -o pipefail`, un `grep` que **legitimamente no encuentra nada** devuelve
1, la sustitucion falla y `set -e` mata el script entero. En `rust/tests/` los
`fn` estan al tope y el grep siempre encontraba algo, asi que el bug estaba
latente desde la #24. En `rust/src/` los tests viven indentados dentro de
`mod tests`, el `^fn` no matchea, y el chequeo moria antes de imprimir.

Arreglado con `|| true` y con un patron que acepta indentacion (ahora ademas
nombra el test unitario). El comentario en el codigo dice por que el `|| true` no
es decorativo, para que nadie lo "limpie".

**Un chequeo que muere en silencio es indistinguible de uno que no encuentra
nada.** Es la misma familia que el `cargo test` que sale 0 sin correr tests
(#23) y el `[ok]` del hub (#25).

### 3. El gate de spec rechazo cerrar las seis entradas como `done`

Al intentar cerrar #27 y #31-#35 citando esta feature, el gate respondio:

```
[GATE] Spec sin aprobar: docs/spec-feature-27-....md (estado: ausente)
```

Y tiene razon: esas entradas **nunca tuvieron spec propio**, porque la decision
de Alan fue agruparlas. Se cerraron como `blocked` con la nota de que el trabajo
esta hecho en la #36. `done` habria sido mentira; `pending` las habria dejado
volviendo en `next`.

El gate haciendo su trabajo contra mi conveniencia es exactamente para lo que
existe.

## La causa, no solo los sintomas

Las seis deudas no se perdieron por casualidad: se escribieron en una seccion de
prosa dentro de un documento de cierre. El AC-14 ataca eso — el rol del
implementer ahora dice:

> Si escribis una seccion **"Para el backlog"**, cada item entra al backlog en el
> MISMO cierre con `harness_cli add`. Una nota que se queda solo en el impl no es
> una deuda registrada: `next` nunca la ofrece y `journey` nunca la cuenta como
> hueco.

## Para el backlog

Ninguna. Lo que quedo afuera son **decisiones declaradas**, no olvidos: que
`doctor` no valide el handshake de PostgreSQL, que el `PreToolUse` solo exista
para Claude Code, que la deteccion de binario viejo sea por mtime. Estan en sus
impl como limites y no como pendientes.

Las que si son features propias siguen en el backlog: #28 (consolidacion con LLM)
y #29 (PRD y SDD siempre al dia).
