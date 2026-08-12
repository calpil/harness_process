# Impl - Feature #13: nested_prds

Spec: `docs/spec-feature-13-nested-prds.md` (Estado: approved, AC-1..AC-17)
Plan: `docs/plan-feature-13-nested-prds.md` (D1..D12)

## Que se construyo

La promesa de "un PRD puede contener otros PRDs" que la guia hacia desde la
feature #12 dejo de ser prosa. El arbol es real, se crea con un comando, se
encadena hasta el spec, vuelve marcado al cerrar y lo valida el check.

Identidad de un PRD = su **cadena de segmentos**. De ella salen las dos rutas,
sin registro intermedio: la carpeta lleva el segmento propio y el archivo la
cadena completa (nombre unico en todo el repo, greppable sin ambiguedad).

```
docs/prd/PRD-master.md                        ""              (la raiz)
docs/prd/cobranza/PRD-cobranza.md             "cobranza"      (Padre: master)
docs/prd/cobranza/mora/PRD-cobranza-mora.md   "cobranza/mora" (Padre: cobranza)
```

El FILESYSTEM es la fuente de verdad; el `Padre:` del encabezado es una
declaracion que el gate contrasta contra la ubicacion real.

## Evidencia por AC

- **AC-1** (rutas por cadena): `prd::file_name_for` / `dir_for` / `file_for` /
  `rel_path` en `rust/src/prd.rs`; `commands/prd.rs::add` compone la cadena
  `padre + segmento`. Test `paths_should_derive_from_the_segment_chain`. E2E
  real: `prd add --name cobranza` -> `docs/prd/cobranza/PRD-cobranza.md`;
  `--name mora --parent cobranza` -> `docs/prd/cobranza/mora/PRD-cobranza-mora.md`
  (smoke sh, bloque `PRD_E2E`). Slug normalizado con la misma `plan::slugify`
  (`cobranza_mora` -> `cobranza-mora`).
- **AC-2** (errores de uso, sin escribir nada): `prd::resolve` falla con la lista
  de PRDs disponibles cuando el padre no existe; `commands/prd.rs::add` se niega
  si el destino existe (`Exit` code 1) ANTES de crear carpeta o archivo;
  `prd::normalize_segment` rechaza el nombre vacio o sin alfanumericos. Tests
  `resolve_should_accept_path_tail_and_reject_ambiguity` y
  `normalize_segment_should_slugify_and_reject_empty_names` (incluye `../../etc`
  -> `etc`: el slug hostil se disuelve antes de tocar el filesystem). Smoke:
  el segundo `prd add --name cobranza` debe fallar y no duplicar la fila.
- **AC-3** (plantilla del hijo): `prd::child_template` — `Estado: Borrador`,
  `Padre: <ruta>`, `Alcance:`, punteros relativos a la guia/SDD/constitution
  segun profundidad, las 12 secciones en el orden del maestro y la linea
  `harness_cli add ... --prd <ruta>` en la seccion 10. Test
  `child_template_should_declare_parent_and_keep_the_twelve_sections` (verifica
  el ORDEN, no solo la presencia). Smoke: `grep '^Padre: cobranza'` +
  `'^## 10. Hitos -> features'` + `'^## 2. La historia'`.
- **AC-4** (enlace en el padre): `prd::link_child` — crea `## PRDs anidados` al
  final si falta, agrega una fila si existe, nunca duplica (compara la primera
  celda), ruta del hijo RELATIVA al padre para que el link funcione al abrir el
  documento. Tests
  `link_child_should_create_the_section_then_append_without_duplicating`
  (verifica ademas que el cuerpo original queda intacto y en su lugar) y
  `link_child_should_use_a_path_relative_to_the_parent`.
- **AC-5** (`add --prd`): `cli.rs` expone el flag; `commands/add.rs` resuelve el
  PRD **antes** de tocar el backlog (una referencia mala no deja una feature a
  medio cargar) y guarda `"prd": "<ruta canonica>"`. Sin `--prd` no se agrega
  campo alguno. `prd::resolve` acepta ruta completa, `master`, o el ultimo
  segmento si es unico; ambigua lista las ramas. Tests `resolve_*`. Smoke:
  `--prd mora` guarda `cobranza/mora`; `--prd noexiste` sale != 0.
- **AC-6** (encabezado del spec): `spec.rs::spec_template` inserta
  `PRD: <ruta>` entre `Plan:` y `Constitution:`, derivado de
  `prd::feature_prd_rel` (el maestro por defecto). `Estado: draft` sigue en la
  linea 3 y el orden de secciones no cambio: el test existente
  `spec_template_should_declare_draft_and_sections` se actualizo con la linea
  nueva y `spec_template_sections_should_keep_the_prd_order` sigue verde. Smoke:
  `grep '^PRD: docs/prd/cobranza/mora/PRD-cobranza-mora.md'` en el spec generado.
- **AC-7** (`prd tree`): `prd::render_tree` + `note_for`. Por nodo: hitos de su
  tabla (`milestone_rows` ignora encabezado, separador y el ejemplo `<...>` de
  la plantilla), `features: <done>/<total>` de las que lo declaran, `[!] sin
  hitos` y `[!] declara Padre: X (su lugar dice Y)`. `--prd <ref>` dibuja el
  subarbol. Tests `render_tree_should_draw_children_with_milestones_and_feature_state`
  y `render_tree_should_flag_a_header_that_lies_about_its_parent`. Salida real:

  ```
  PRD-master                  [!] sin hitos
   `-- PRD-cobranza           [!] sin hitos
       `-- PRD-cobranza-mora  1 hito | features: 1/1 done
  ```

- **AC-8** (gate): bloque nuevo al final de `harness_check.sh`. Las cuatro
  incoherencias suman a `failures` (exit 2 en modo `block`): PRD fuera de lugar,
  carpeta sin su PRD, `Padre:` que no coincide con la ubicacion, feature con
  `prd` inexistente. Un PRD sin hitos avisa con `[i]` y NO bloquea. Verificado
  en fixture rompiendo el arbol a proposito: 5 `[!]` y exit 2; con
  `HARNESS_CHECK_MODE=warn`, exit 0. Smoke: bloque `AC-8 (roto)`.
- **AC-9** (sin `docs/prd/`): el bloque del check esta envuelto en
  `if [ -d "$prd_root" ]`; `commands/prd.rs::tree` informa "No hay PRDs todavia"
  y sale 0. Smoke: se borra `docs/prd/` y el check vuelve a pasar.
- **AC-10** (guia): `docs/prd/COMO-ESCRIBIR-UN-PRD.md` seccion 3 ahora trae los
  comandos reales, el layout en carpetas con ejemplo y la salida de `prd tree`;
  la seccion 5 suma el nivel **Parte** a la tabla, extiende la cadena hasta el
  cierre y explica que se actualiza solo y que no. Copia espejo en
  `templates/docs/prd/`.
- **AC-11** (planilla maestra): `docs/prd/PRD-master.md` (+ template) suma
  `## PRDs anidados` y `## Bitacora`, y su tabla de hitos menciona `--prd <ruta>`
  y el efecto del cierre. Siguen siendo documentos del USUARIO (`PRD_DOCS`).
- **AC-12** (superficies y docs): `write_agent_surface` de `setup_harness.sh` y
  el bloque equivalente de `setup_harness.ps1`; `AGENTS.md`, `README.md`
  (arbol del repo, cadena completa y seccion "PRDs anidados: el arbol de
  producto"), `UPDATING.md` raiz y `templates/` (seccion nueva) y
  `docs/architecture.md` (paso 0b del flujo SDD, modulo `prd.rs`, lista de
  comandos y regimen de archivos). Sin tocar `write_basic_agent_surface` ni
  `.grok/GROK.md`.
- **AC-13** (espejo): `harness_check.sh` y `templates/harness_check.sh`
  identicos (`diff -q` limpio, verificado tras el cambio).
- **AC-14** (tests unitarios): 13 tests nuevos en `prd::tests` cubriendo rutas,
  normalizacion, escaneo (ignora lo mal ubicado), resolucion (exacta / cola
  unica / ambigua / inexistente), plantilla y su orden, enlace en el padre
  (crea / agrega / no duplica / ruta relativa), filas de hitos, vuelta del
  cierre (con y sin fila de hito, idempotente) y render del arbol (dos casos).
  `cargo test --locked`: 64 unit + 27 integracion, 0 fallos.
- **AC-15** (smoke): bloque `E2E Feature #13` en `tests/setup_smoke.sh` sobre
  fixture dedicado (`PRD_E2E`, root layout) que ejercita AC-1..AC-9 y AC-17 de
  punta a punta, mas asserts de AC-10/AC-11/AC-12 en el fixture subdir.
  `bash tests/setup_smoke.sh` sale 0. `tests/setup_smoke.ps1` recibe los asserts
  equivalentes (arbol, enlace, `prd tree`, cadena `--prd` -> spec, planilla y
  guia); sin `pwsh` en el entorno se verifico estaticamente, como en #1 y #4-#12.
- **AC-16** (verificacion oficial): `bash harness_check.sh` limpio;
  `cargo test --locked` verde; `cargo clippy --all-targets --all-features
  --locked -- -D warnings` sin hallazgos; `bash tests/setup_smoke.sh` exit 0.
- **AC-17** (vuelta del cierre): `prd::echo_close` + `commands/close.rs::echo_to_prd`.
  Marca la fila del hito cuyo slug coincide con el nombre de la feature
  (`Estado` -> `done (YYYY-MM-DD)`), agrega la linea de `## Bitacora` con spec e
  impl (creando la seccion al final si falta) y NUNCA reescribe el cuerpo. Es
  best-effort: PRD ausente o ilegible avisa con `[i]` y no impide cerrar;
  `blocked`/`pending` no tocan ningun PRD. Idempotente en los dos sentidos: no
  duplica la bitacora y **conserva la fecha del primer cierre** (un re-cierre
  con otra fecha no reescribe la historia del documento). Tests
  `echo_close_should_mark_the_milestone_and_log_once` y
  `echo_close_should_log_even_without_a_milestone_row`.

## Decisiones tomadas por el USUARIO en esta feature

1. Layout: carpetas anidadas reales (no plano con metadato, no indice JSON).
2. Creacion: comando `prd add` que crea desde plantilla **y** enlaza en el padre.
3. Cadena: `add --prd` con campo en `feature_list.json` y encabezado `PRD:` en el
   spec.
4. Validacion: `prd tree` + gate de integridad en `harness_check.sh`.
5. ENMIENDA post-aprobacion (a la pregunta "¿los PRD se van actualizando con lo
   que en realidad quedo implementado?"): AC-17, la vuelta del cierre al PRD.
   Descartadas: solo-lectura (el PRD se pudre en "pendiente") y el gate de
   divergencia por mtime (queda para otra feature).

Detalles derivados, confirmados en la aprobacion: carpeta corta + archivo con la
cadena completa; `--prd` acepta el ultimo segmento si es unico; ciclos y slugs
duplicados NO van al gate porque son imposibles por construccion con carpetas.

## Archivos tocados

| Archivo | Que |
| --- | --- |
| `rust/src/prd.rs` | modulo nuevo: arbol, plantilla, enlace, hitos, bitacora, render |
| `rust/src/commands/prd.rs` | comando nuevo: `prd add` / `prd tree` |
| `rust/src/cli.rs` | subcomando `prd`, flag `--prd` en `add` |
| `rust/src/main.rs`, `rust/src/commands/mod.rs` | registro de modulos |
| `rust/src/commands/add.rs` | `--prd`: resolucion previa + campo opcional |
| `rust/src/commands/close.rs` | vuelta al PRD al cerrar como done |
| `rust/src/spec.rs` | encabezado `PRD:` + test actualizado |
| `harness_check.sh`, `templates/harness_check.sh` | gate del arbol (espejo) |
| `docs/prd/COMO-ESCRIBIR-UN-PRD.md`, `docs/prd/PRD-master.md` (+ `templates/`) | metodo y planilla |
| `setup_harness.sh`, `setup_harness.ps1` | superficies (paridad) |
| `AGENTS.md`, `README.md`, `UPDATING.md` (+ `templates/`), `docs/architecture.md` | docs |
| `tests/setup_smoke.sh`, `tests/setup_smoke.ps1` | smoke |

## Notas

- El hub PostgreSQL estuvo inalcanzable toda la sesion (timeout de conexion,
  igual que en #10-#12): es best-effort y no altera exit codes; el impacto se
  calculo por lectura directa del repo.
- El binario `harness` del repo se recompilo en release con los comandos nuevos.
