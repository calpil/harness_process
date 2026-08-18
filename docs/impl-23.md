# Evidencia de implementacion - Feature #23: ac_ejecutables_verify

Spec: `docs/spec-feature-23-ac-ejecutables-verify.md` (`Estado: approved`, 20 AC,
todos con `Comando:` — el primer spec del repo que declara como se prueba)
Plan: `docs/plan-feature-23-ac-ejecutables-verify.md` (D1-D8)
PRD: `docs/prd/PRD-master.md` (hito 1)

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/verificacion.rs` | D1, D2, D3, D4 | NUEVO. `parsear()` puro, `ejecutar()` con timeout, enum `Estado`, `render_reporte()`, `rojos_del_reporte()` y `gate()`; 19 tests |
| `rust/src/commands/verify.rs` | D2, D3 | NUEVO. La barrera del spec aprobado, la impresion previa, `--solo`, `--json` |
| `rust/src/commands/close.rs` | D4 | El tercer gate, junto a los dos que ya existian |
| `rust/src/spec.rs` | D6 | La plantilla documenta `Comando:` como opcional |
| `rust/src/cli.rs`, `main.rs`, `commands/mod.rs` | D2 | Cableado |
| `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md`, `docs/verification.md` (+ espejo), `templates/docs/architecture.md` | D7 | El comando, la regla, las tres barreras y las trampas del verde facil |
| `setup_harness.sh` / `.ps1` | D7 | Superficies |
| `templates/roles/*.md` -> `roles/*.md` -> `.claude/agents/*.md` | D7 | Lider declara, implementer corre, reviewer exige |
| `rust/tests/cli_basics.rs` | D5, D8 | 14 tests de integracion |

## Evidencia por AC

Esta feature se verifica a si misma: `sh harness_cli verify --feature 23` corre
los 20 comandos que el spec declara. **20 verde(s), 0 en rojo, 0 manual(es)**
(`docs/verify-23.md`).

| AC | Evidencia |
| --- | --- |
| AC-1 | `verificacion::tests::parse_*` (7 tests: con backticks, sin backticks, dos comandos, no-AC, sin AC, bloques ```, specs reales) |
| AC-2 | `verify_should_do_nothing_without_declared_commands` + corrida real sobre `spec-feature-22` (18 AC, 0 comandos) y `spec-feature-20` (19 AC): informa y sale 0 |
| AC-3 | `manual_should_never_block`; `Estado::Manual` no esta en `bloquea()` |
| AC-4 | `verify_should_print_each_command_before_running_it` (stdout `AC-1  $ true`) |
| AC-5 | `verify_should_refuse_to_run_commands_from_a_draft_spec` (exit 2 + `rastro.txt` inexistente) y la corrida real de mas abajo |
| AC-6 | `verify_should_time_out_a_hung_command` (`verify_timeout_segundos: 1` + `sleep 30` -> `timeout`), `verify_should_keep_going_after_a_failure`, `timeout_should_come_from_rules_with_a_default` |
| AC-7 | `verify_should_not_be_wired_into_any_hook` |
| AC-8 | `verify_should_write_a_report_per_ac` (AC, comando, exit, ms, estado) |
| AC-9 | `verify_should_include_output_of_failures`, `recortar_salida_should_keep_the_tail` |
| AC-10 | `verify_json_should_expose_the_result_per_ac` |
| AC-11 | `verify_should_run_a_single_ac_on_demand` (corre AC-2 y NO AC-1; `--solo AC-9` inexistente -> exit 2) |
| AC-12 | `close_should_stay_identical_without_the_verify_rule` (comandos declarados, sin reporte, sin regla -> cierra) |
| AC-13 | `close_should_demand_a_verify_report` (exit 2), `close_should_not_gate_a_spec_without_commands_even_with_the_rule_on` |
| AC-14 | `close_should_block_on_a_red_report` (nombra AC-1 y **no** nombra AC-2, que estaba verde) |
| AC-15 | `close_should_block_on_a_stale_report` (mtime del spec +60s -> exit 2) |
| AC-16 | `close_should_never_execute_verify_commands` (reporte verde, cierre exitoso, el rastro NO reaparece) + `rojos_del_reporte_*` (round trip render -> lectura) |
| AC-17 | `start_should_document_the_command_line_in_the_spec_template` |
| AC-18 | `grep -q "require_verify_green" README.md UPDATING.md docs/architecture.md` |
| AC-19 | `grep -q "verify" roles/{leader,implementer,reviewer}.md` |
| AC-20 | `cargo test` 250 + 109 verde, clippy 0, `tests/setup_smoke.sh` verde, `harness_check.sh` limpio |

## Los criterios de cierre: lo que solo aparecio corriendo esto de verdad

El plan exigia correr `verify` sobre el spec **real** de esta feature. Los tests
con fixtures pasaban desde el principio. La primera corrida real encontro dos
cosas, y la segunda es la mas importante de toda la feature.

### 1. El spec ejecuto su propio ejemplo

El spec explica el formato con un ejemplo dentro de un bloque:

````
```
- AC-1: Given un proyecto sin docs/lecciones/, When corre el instalador,
  Then se siembra la guia y ninguna leccion.
  Comando: `bash tests/setup_smoke.sh`
```
````

`parsear()` no distinguia prosa de criterio, asi que la corrida arranco
**ejecutando `bash tests/setup_smoke.sh`** —el instalador entero— por un ejemplo
que solo estaba ahi para ensenar la sintaxis. Ademas el spec quedaba con 21 AC en
vez de 20, con dos `AC-1` distintos.

Arreglado saltando los bloques ` ``` `, con el hallazgo escrito dentro del test
(`parse_should_ignore_examples_inside_fenced_blocks`) para que nadie lo
"simplifique" despues. Un spec que documenta la sintaxis no puede terminar
verificando su documentacion.

### 2. Un comando puede dar verde sin ejecutar nada

`cargo test <nombre>` **sale 0 cuando el filtro no matchea ningun test**. En la
primera corrida, 8 de los 20 AC declaraban nombres de test que yo no habia
escrito con ese nombre exacto:

```
AC-2  $ cd rust && cargo test verify_should_do_nothing_without_declared_commands
       [ok] verde (84 ms)     <- 0 tests corridos
```

Verde. Sin ejecutar nada. Es exactamente el fallo que la feature existe para
evitar, disfrazado de exito.

Se corrigio en la direccion correcta: **el spec es el contrato**, asi que renombre
los tests a los nombres que el spec declara (y escribi los que faltaban:
`verify_should_write_a_report_per_ac`, `verify_should_include_output_of_failures`,
`close_should_stay_identical_without_the_verify_rule`,
`close_should_never_execute_verify_commands`,
`start_should_document_the_command_line_in_the_spec_template`). Despues verifique
uno por uno que cada comando matchea al menos un test:

```
AC-1 ok (7 tests)   AC-8  ok (1)   AC-15 ok (1)
AC-2 ok (1 test)    AC-9  ok (1)   AC-16 ok (1)
...                                AC-17 ok (1)
```

Lo que **no** se puede arreglar con codigo quedo documentado en los tres roles,
en el README y en `docs/verification.md`: un comando que no puede fallar no
verifica, decora.

### 3. El AC-7 declara un comando que no puede fallar

```
Comando: `grep -rn "verify" bin/harness-hook setup_harness.sh | ... | grep -c "..." || true`
```

`grep -c` devuelve 1 cuando cuenta 0, y el `|| true` lo traga: **ese comando sale
0 siempre**. Lo digo aca en vez de taparlo, porque es el segundo caso real de la
trampa de arriba y porque el spec ya estaba aprobado (cambiarlo obligaria a
re-aprobarlo y dejaria el reporte stale).

La evidencia real del AC-7 no es ese comando sino
`verify_should_not_be_wired_into_any_hook`, que ademas tuvo que aprender una
distincion que yo no habia visto: **documentar** `verify` en las superficies es
obligatorio (AC-18/AC-19) mientras que **invocarlo** desde un hook esta
prohibido. La primera version del test grepeaba el texto plano y empezo a fallar
cuando documente el comando en `setup_harness.sh`. Ahora ignora lo que esta entre
backticks (prosa) y mira solo lo ejecutable, y ademas exige que el runtime de
hooks que escribe el instalador no lo nombre **ni en prosa**.

### 4. Exit code: el spec pedia 2, el codigo daba 1

Los AC-13, AC-14 y AC-15 dicen "exit 2" con todas las letras. Mi gate usaba
`Exit::msg`, que es exit **1**, igual que los otros dos gates de `close`. Gana el
spec: es el contrato aprobado. Queda la inconsistencia de que los tres gates del
mismo comando salen con codigos distintos (spec y leccion con 1, verify con 2)
— anotada abajo para el backlog, no resuelta por mi cuenta.

## La barrera del draft, sobre datos reales

No alcanza con el test: la corri contra el spec real de esta feature, que declara
20 comandos.

```
$ sed -i '' 's/^Estado: approved$/Estado: draft/' docs/spec-feature-23-*.md
$ sh harness_cli verify --feature 23
[BARRERA] Spec sin aprobar: docs/spec-feature-23-ac-ejecutables-verify.md (estado: draft).
    verify NO ejecuta comandos de un spec que el usuario no aprobo:
    aprobar el spec es el acto en el que alguien leyo esos comandos.
[OK] el reporte NO se toco (mtime 1786945126)
[OK] spec restaurado byte a byte
```

Exit 2, ningun comando ejecutado, `docs/verify-23.md` intacto.

## Compatibilidad, medida y no prometida

- `parse_should_stay_compatible_with_the_310_existing_acs` parsea **los specs
  reales de las features #1-#22** y falla si alguno declarara un comando.
- Corrida real: `verify --feature 22` -> "18 AC, ninguno declara `Comando:`. Nada
  que ejecutar", exit **0**. Idem `--feature 20` en `--json`.
- Sin `require_verify_green`, `close` es byte a byte el de antes (AC-12).

## Limites declarados

- **`tests/setup_smoke.ps1` no se corrio**: esta maquina no tiene pwsh ni Windows
  PowerShell. Mismo limite aceptado en las features #15-#22. La superficie `.ps1`
  se actualizo en paridad con la `.sh` por inspeccion.
- **`verify` ejecuta con `sh -c` / `cmd /C`**: los comandos dependen del shell del
  sistema. Los del spec de esta feature usan `cd rust && ...`, que corre desde la
  raiz del proyecto; queda documentado en el README.
- **La barrera protege del descuido, no de aprobar a ciegas.** Si el usuario
  aprueba un spec sin leer los comandos, no hay barrera que valga. Esta dicho en
  el plan y en el README.

## Para el backlog

- **Unificar el exit code de los tres gates de `close`** (1 / 1 / 2).
- **`--solo` acepta varios AC** (`--solo AC-3,AC-7`): iterar sobre dos hoy obliga
  a dos corridas.
- **Detectar comandos que no pueden fallar** al escribir el spec: un aviso de
  `check-spec` cuando un `Comando:` termina en `|| true` o filtra tests por nombre
  sin `--exact` cubriria por herramienta lo que hoy es disciplina.
