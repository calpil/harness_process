# Impl - Feature #73: verify corre UN comando por AC y no lo dice

Spec: docs/spec-feature-73-verify-corre-un-comando-por-ac-y-no-lo-dice-un-a.md
Plan: docs/plan-feature-73-verify-corre-un-comando-por-ac-y-no-lo-dice-un-a.md

## Lo que se midio antes de tocar nada

Sobre el corpus REAL —63 specs— hay **un solo** AC con mas de un `Comando:`: el
AC-8 de la #72, con cuatro. O sea que esto no venia rompiendo cosas en silencio
desde hace meses: es una trampa que se disparo la primera vez que alguien la
piso, y que se iba a volver a disparar porque escribir cuatro comandos debajo de
un criterio es lo natural.

La causa era el modelo, no un olvido: `Verificacion { ac, comando: Option<String> }`
no tiene donde poner el segundo, asi que `parsear` tenia un
`&& ultimo.comando.is_none()` y los descartaba.

## El hallazgo que no esperaba: el test estaba escrito para imitar el bug

`parse_should_only_report_commands_the_spec_actually_declares` corre sobre los
310+ AC reales del repo y su comentario dice **"INVARIANTE: el parser no inventa
ni pierde comandos"**. Pasaba en verde mientras el AC-8 de la #72 perdia tres.

El motivo estaba en su propio oraculo:

```rust
} else if t.starts_with("Comando:") && ac_abierto {
    n += 1;
    ac_abierto = false; // solo el primero cuenta, como en `parsear`
}
```

El oraculo copiaba a la implementacion. Un test asi no verifica: acompaña. Y
habia un segundo test, `parse_should_keep_only_the_first_command_of_an_ac`, que
afirmaba el descarte como si fuera la intencion — no estaba mal escrito,
describia con precision lo que el codigo hacia; lo que faltaba era preguntarse
si eso era lo que TENIA que hacer.

Con el oraculo arreglado, el test detecta el bug sobre el spec de verdad:

    spec-feature-72-...md: el parser reporto 1 comando(s) y el spec declara 4

## Evidencia por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/verificacion.rs:45 | `comandos: Vec<String>` reemplaza al `Option<String>`, y `rust/src/verificacion.rs:209` acumula en vez de quedarse con el primero. El test de rust/tests/cli_basics.rs:2941 comprueba por el EFECTO de cada comando —los archivos que dejan— y no por la salida, que es lo que dejaba pasar el bug. |
| AC-2 | rust/src/commands/verify.rs:97 | Un `Resultado` por comando: el reporte tiene una fila por cada uno, con su estado, su exit y su duracion. Verificado en rust/tests/cli_basics.rs:2964, que cuenta las filas. |
| AC-3 | rust/src/verificacion.rs:598 | `sin_repetir` es la UNICA funcion que deduplica nombres de AC, y la usan los dos lugares que hablan de "AC en rojo": el mensaje de `verify` y el `rojos_del_reporte` que lee el gate. Sin ella el mensaje decia `AC en rojo: AC-1, AC-1`. Test en rust/tests/cli_basics.rs:2984. |
| AC-4 | rust/src/verificacion.rs:1270 | El test del corpus real compara COMANDOS contra COMANDOS sobre los 63 specs: los 62 que declaran uno por AC no cambiaron. Ademas `verify_should_write_one_report_row_per_command` afirma que el AC de un solo comando conserva su unica fila. |
| AC-5 | rust/src/verificacion.rs:45 | `es_manual()` = lista vacia, que es lo que antes significaba `None`. Los tests de AC manual siguen verdes sin cambios de semantica. |
| AC-6 | rust/src/verificacion.rs:209 | La guarda `!ac_ilegible` de la feature #68 sigue intacta y su test tambien: un `Comando:` de una linea AC ilegible se descarta y no se le cuelga al AC de arriba. |
| AC-7 | rust/src/verificacion.rs:598 | `rojos_del_reporte` sigue parseando fila por fila, asi que un reporte viejo —una fila por AC— se lee igual. La deduplicacion no cambia nada cuando no hay repetidos. |
| AC-8 | rust/tests/cli_basics.rs:2941 | Suite completa, clippy, smoke del instalador y gate de paridad. |
| AC-9 | rust/src/verificacion.rs:1270 | MANUAL: el AC-8 de la #72 declara cuatro lineas `Comando:` (contadas sobre el archivo) y es parte del corpus que ese test recorre, que compara comandos parseados contra comandos declarados spec por spec. Con la mutacion puesta el test lo delata por nombre: `spec-feature-72-...md: el parser reporto 1 comando(s) y el spec declara 4`. NO se re-corrio `verify --feature 72`: sus cuatro comandos son la suite, clippy, el smoke y la paridad —unos seis minutos— y no aportaria informacion que el test del corpus no de ya. |

## Las tres mutaciones

| Mutacion | Que cae |
| --- | --- |
| `&& ultimo.comandos.is_empty()` (el bug original) | `parse_should_keep_every_command_of_an_ac_in_order`, `verify_should_run_every_command_an_ac_declares` (exit 0 en vez de 1: el comando que falla no se corre) y `parse_should_only_report_commands_the_spec_actually_declares` sobre el spec-72 real |
| sacar `sin_repetir` de `rojos_del_reporte` y del mensaje | `verify_should_fail_an_ac_when_any_of_its_commands_fails` |

## Lo que NO hace

- **No acota cuanto tarda un AC.** Un AC con cuatro comandos tarda la suma de
  los cuatro; cada uno conserva su timeout propio.
- **No cambia la sintaxis del spec.** Sigue siendo una linea `Comando:` por
  verificacion, debajo del criterio.
- **No reescribe** los reportes `docs/verify-*.md` ya emitidos ni los specs.
