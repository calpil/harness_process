# Evidencia de implementacion - Feature #19: perfil_de_usuario

Spec: `docs/spec-feature-19-perfil-de-usuario.md` (`Estado: approved`, 20 AC,
sello 2026-08-16T23:32:38Z)
Plan: `docs/plan-feature-19-perfil-de-usuario.md` (D1-D10)
PRD: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 3)

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/perfil.rs` | D1, D2, D3, D5 | NUEVO. Modelo, limite, `Coincidencia`, escaneo de seguridad, `bloque` y `recolectar`; 20 tests |
| `rust/src/commands/perfil.rs` | D4, D5, D8 | NUEVO. `show/add/replace/remove/sugerir/check/bloque` |
| `rust/src/cli.rs`, `main.rs`, `commands/mod.rs` | D4 | Cableado del comando |
| `templates/docs/perfil-usuario.md` | D6 | NUEVO. Plantilla sembrada (con test anti-drift) |
| `setup_harness.sh` / `.ps1` | D6, D7, D9 | `USER_DOCS`/`$script:UserDocs`, siembra, inyeccion idempotente y superficie |
| `harness_check.sh` (+ espejo) | D8 | Gate que delega en `perfil check` |
| `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md` | D9 | Comando, limite, ritual, snapshot y los tres almacenes |
| `templates/roles/leader.md`, `reviewer.md` (+ `roles/` y espejos) | D9 | Proponer con `sugerir`; verificar que nada entro sin el si |
| `rust/tests/cli_basics.rs` | D10 | 8 tests de integracion |
| `tests/setup_smoke.sh` / `.ps1` | D10 | Siembra vacia, no-pisa, reset, inyeccion idempotente, ausencia de bloque |
| `docs/perfil-usuario.md` | dogfood | Sembrado en este repo (vacio) |

## Evidencia por AC

### AC-1 — Siembra, no-pisa y supervivencia al reset

El perfil entra en `USER_DOCS` / `$script:UserDocs`, la lista de **documentos del
USUARIO** (no en `HARNESS_DOCS`), asi que se siembra solo si falta, ningun
reinstall lo pisa y **no** entra en los reset targets. Es la leccion
`docs-generados-por-el-instalador` aplicada tal cual — y la primera vez que esa
leccion se usa desde que se escribio.

`tests/setup_smoke.sh` (exit 0):

```
test -f "$SUBDIR_ROOT/docs/perfil-usuario.md"        # sembrado
grep -q '^# Perfil de usuario' ...                   # con su encabezado
if grep -q '^- ' ...; then FALLO; fi                 # y VACIO
grep -q "$PERFIL_SENTINEL" ...                       # el reinstall no lo pisa
grep -q "$PERFIL_RESET_SENTINEL" ...                 # --reset no lo borra
```

### AC-2 — Formato y que cuenta el limite

`usados_should_count_only_entries`: el encabezado no cuenta; una entrada de 3
caracteres da 3. `entradas_should_survive_a_round_trip` y
`render_should_preserve_the_user_header` garantizan que la prosa del usuario
alrededor de las entradas no se pierde.

### AC-3 / AC-5 — El limite duro falla, no recorta

`perfil_should_refuse_to_exceed_the_hard_limit` (integracion): dos entradas de
900 caracteres; la segunda sale con exit 2 y **no** se escribe.
`error_de_limite_should_list_current_entries_and_ask_to_consolidate` verifica el
mensaje: "no se recorta nada", "en este mismo turno" y la lista numerada de las
entradas actuales. `usados_con_should_account_for_a_replacement` cubre el AC-5:
un `replace` cuenta el texto nuevo en lugar del viejo, y tambien puede fallar.

### AC-4 — `perfil show`

```
Perfil de usuario [0% - 0/1500 chars]
```
Verificado en integracion (`/1500 chars` y las entradas numeradas). Con el perfil
vacio, `show` explica como empezar en vez de imprimir una tabla vacia.

### AC-6 — Solo el usuario escribe

`perfil_writes_should_refuse_without_the_user_yes`: los **tres** comandos salen
con exit 2 y el texto "exige la confirmacion explicita del USUARIO", y el
archivo **no se crea**. El mensaje repite el ritual de `approve-spec`: mostrar,
preguntar, y recien entonces registrar.

### AC-7 — Duplicado

`perfil add` con un texto identico responde `El perfil ya tenia esa entrada; no
se duplico.` y sale con 0 (no-op, no error).

### AC-8 — Subcadena unica, con los tres casos

`Coincidencia` es un **enum** (`Ninguna` / `Unica` / `Ambigua`) y no un `Option`:
"matchea varias" es un estado real del dominio con su propio remedio, y como
`None` se perderia. Patron tomado de la skill `rust-patterns` ("model states as
enums" + "exhaustive matching, no catch-all"). Los tres casos tienen test unitario
y los dos de error, test de integracion:

```
$ perfil remove --old "inexistente" --yes   -> exit 2, "Ninguna entrada..."
$ perfil remove --old "e" --yes             -> exit 2, "usa un fragmento mas especifico"
```

### AC-9 — Auditoria

`perfil_should_add_show_replace_and_remove` verifica que `progress/history.md`
recibe las tres lineas (`perfil add`, `perfil replace`, `perfil remove`).

### AC-10 — Escaneo de seguridad que BLOQUEA

`motivo_inseguro_should_reject_credentials` cubre cinco familias (`password=`,
`api_key:`, bloque `BEGIN ... PRIVATE KEY`, token `ghp_...`, clave `AKIA...`) y
`motivo_inseguro_should_reject_invisible_unicode` cubre zero-width y bidi,
nombrando el codepoint (`U+200B`). El contrapeso —igual de importante— es
`motivo_inseguro_should_accept_ordinary_preferences`: las tres entradas reales
que este repo produciria **no** disparan falsos positivos.

End to end, el rechazo ocurre **antes** de escribir:

```
$ perfil add --texto "el api_key: abc123 del hub" --yes
La entrada trae algo que parece una credencial ('api_key' seguido de : o =): no entra al perfil.
    ...nombra la VARIABLE de entorno, nunca su valor.
exit 2   # y docs/perfil-usuario.md no existe
```

### AC-11 / AC-12 — Inyeccion idempotente, y nada sin perfil

El bloque lo **renderiza el binario** (`perfil bloque`), no los instaladores: el
formato y el parseo del perfil viven en un solo lugar y los dos instaladores no
pueden divergir. Prueba real de idempotencia:

```
== tras 1 inyeccion: 1 bloque(s) ==
== tras 3 inyecciones: 1 bloque(s) ==
```

...con el contenido previo de la superficie intacto. El smoke lo verifica en las
**cuatro** superficies tras un reinstall completo, y verifica lo contrario con el
perfil vacio: `if grep -q 'harness:perfil:inicio' ...; then FALLO; fi`.

### AC-13 — Snapshot congelado

Toda escritura imprime:

```
  Las superficies (CLAUDE.md, AGENTS.md, GEMINI.md, LLM.md) se refrescan
  al reinstalar: este cambio recien llega a los agentes en la proxima sesion.
```

Verificado en integracion (`"se refrescan"`). El comando no toca ninguna
superficie: solo `docs/perfil-usuario.md`.

### AC-14 / AC-15 / AC-16 — `perfil sugerir`

Corrida real sobre este repo (19 features de historial):

```
Evidencia encontrada: 160 registro(s) de decision, 160 sin incorporar al perfil.

== feature #6 ==
  [history] 2026-07-24 ... advance feature #6 ... flag de barrera decidido: --yes
  [plan] Nombre del flag de barrera: DECIDIDO por el usuario (2026-07-24): `--yes`.
  [spec] Nombre del flag de barrera: DECIDIDO por el usuario (2026-07-24): `--yes`
...
COMO DESTILAR UNA ENTRADA (el arnes no lo hace por vos):
- Una entrada dice COMO trabajar, en presente y en general.
    Bien: "Ante un fork de consistencia, elige la opcion segura aunque cueste mas."
    Mal:  "En la #14 eligio escribir solo el delta." (es un hecho, no una preferencia)
...
```

Lee las **tres** fuentes (OBS-5), agrupa por feature, marca lo ya citado (OBS-3,
por `#<id>` en una entrada) y **no escribe nada** (verificado: tras `sugerir`,
`docs/perfil-usuario.md` no existe en el sandbox). Sin material dice
`Sin material todavia` y sale 0.

### AC-17 — Sin hub

Ningun camino de `perfil.rs` ni de `commands/perfil.rs` importa `graph`. En este
entorno el hub esta caido (`connection timed out`) y todos los comandos de la
feature corrieron normal, incluida la corrida real de `sugerir` de arriba.

### AC-18 — Gate de integridad

La validacion vive en el binario (`perfil check`) y `harness_check.sh` la invoca:
contar caracteres UTF-8 en awk es poco confiable y el limite tiene que ser
**exactamente** el mismo que aplica al escribir. Prueba real:

```
$ (perfil con 1600 caracteres) && bash harness_check.sh
[GATE] docs/perfil-usuario.md supera el limite: 1600/1500 caracteres.
[Harness] Check fallo con 1 problema(s).
```

Sin el archivo, el gate ni siquiera corre.

### AC-19 — Docs y roles

README (seccion propia), UPDATING (+ espejo), `architecture.md` (modulo + los
tres almacenes, donde el perfil deja de figurar como "pendiente"), superficies de
ambos instaladores, y los dos roles: el lider propone con `sugerir` (paso 4.2) y
el reviewer da `blocked` si una entrada aparecio sin su rastro de aprobacion.

### AC-20 — Verificacion oficial

```
$ (cd rust && cargo test --locked)
test result: ok. 176 passed; 0 failed   (unitarios, +20)
test result: ok.  64 passed; 0 failed   (integracion, +8)

$ (cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)
Finished

$ bash tests/setup_smoke.sh
[exited with code 0]

$ bash harness_check.sh
[Ok] Harness Check limpio.
```

`tests/setup_smoke.ps1` recibio las aserciones en paridad (siembra vacia,
encabezado, ausencia de bloque) pero **no se ejecuto**: sin PowerShell en esta
maquina, decision de Alan del 2026-08-16 de dejarlo declarado.

## Skills aplicadas

- **`rust-patterns`**: `Coincidencia` como enum con matcheo exhaustivo (AC-8) —
  el cambio de diseno mas concreto que aporto una skill en esta feature.
- **`rust-best-practices`**: `#[expect(clippy::unwrap_used, reason = "...")]` en
  vez de `#[allow(...)]` en los tres regex constantes de `motivo_inseguro`;
  nombres de test `x_should_y_when_z`.
- **`rust-testing`**: helper `perfil_con(&[...])` documentado dentro de
  `mod tests`.
- **No adoptado**: `rstest` y `proptest`. Serian dependencias nuevas y el
  Articulo 6 las condiciona a un ADR; los bucles table-driven ya cubren lo mismo
  (ver `motivo_inseguro_should_reject_credentials`, cinco casos en un `for`).

## Hallazgo del dogfooding

Correr `perfil sugerir` sobre este repo devolvia **165** registros, y cinco eran
ruido: lineas como `(ninguna observacion pendiente sin decision)` o `Sin
decisiones pendientes abiertas` MENCIONAN decisiones para decir que **no** hay
ninguna. Se agrego `ANTI_SENALES` con su test
(`recolectar_should_skip_lines_that_say_there_are_no_decisions`) y quedaron 160.
Solo se encontraba corriendolo contra datos reales.

## Riesgos pendientes para el reviewer

1. **`setup_smoke.ps1` sin ejecutar** (igual que #17 y #18).
2. **160 registros son muchos para leer de una vez.** `sugerir` no trunca nada a
   proposito, y el numero baja solo a medida que las entradas citan sus features.
   Aun asi, en un repo con anios de historia la salida seria inmanejable: vale
   evaluar un `--desde <feature>` en una feature futura, no en esta.
3. **El escaneo puede dar falsos positivos.** Una entrada legitima que diga
   "token" seguido de `:` seria rechazada. El mensaje dice cual patron disparo,
   asi que el remedio es reescribir la frase; el costo asimetrico ya se evaluo
   (OBS-4).
4. **El perfil de este repo quedo sembrado y VACIO.** La feature entrega la
   maquinaria; las entradas las decide Alan, y ese paso es suyo, no del agente.
