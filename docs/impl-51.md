# Evidencia de implementacion - Feature #51: revision_adversarial_y_modelos_por_rol

Spec: `docs/spec-feature-51-revision-adversarial-y-modelos-por-rol.md` (approved, 18 AC)
Plan: `docs/plan-feature-51-revision-adversarial-y-modelos-por-rol.md`

## Que se construyo

1. **Tabla de roles** en los dos instaladores: modelo y esfuerzo por rol en UN
   solo lugar, con variables de entorno para cambiarlos sin tocar codigo.
2. **`roles/reviewer.md` reescrito**: la instruccion primaria pasa a ser
   refutar, con verificacion independiente, hallazgos con caso concreto y seis
   reglas de disciplina de tokens.
3. **`rust/src/revision.rs` + `harness revision`**: el paquete minimo de
   revision, acotado por presupuesto y con su propio tamaño reportado.

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 implementer con Opus | OK | `tests/setup_smoke.sh` (fixture `roles-modelo`): `model: claude-opus-5` en `.claude/agents/implementer.md` tras correr el instalador |
| AC-2 leader y reviewer con Fable | OK | Mismo bloque del smoke: `model: claude-fable-5` en los dos, y `effort: xhigh` en los tres |
| AC-3 paridad ps1 | PARCIAL (documentado) | Asserts de contenido en el smoke (`$claudeModels`, `claude-opus-5`, `"xhigh"`). **No ejecutado**: no hay PowerShell en esta maquina (mismo limite de las features #1, #13, #14, #15 y #16) |
| AC-4 reinstalar no pisa | OK | Smoke: se guarda el espejo, se reinstala y se compara con `cmp -s` — sin diff |
| AC-5 un solo lugar | OK | Smoke: `HARNESS_MODEL_IMPLEMENTER=claude-sonnet-5 HARNESS_CLAUDE_EFFORT=high` cambia el resultado sin tocar codigo; `roles/README.md` documenta donde |
| AC-6 refutar, no confirmar | OK | `roles/reviewer.md`: seccion "Tu trabajo es intentar ROMPER, no confirmar", con las cinco familias de caso (limite, error, estado previo, topologia, adyacente) |
| AC-7 verificacion independiente | OK | Misma seccion: "no le creas por su cuenta — comprobalo vos, sin partir de su conclusion... Si solo pudiste confirmar leyendo lo que escribio el implementer, eso NO es verificacion" |
| AC-8 hallazgo con caso concreto | OK | Misma seccion, con el contraejemplo explicito ("El manejo de errores es debil" no es un hallazgo) |
| AC-9 que significa `approved` | OK | Misma seccion: "no se pudo romper con los casos probados" + obligacion de nombrar lo que no se probo |
| AC-10 disciplina de tokens | OK | `roles/reviewer.md`: seccion "Cuanto te cuesta revisar", seis reglas verificables (arrancar por el paquete, del diff hacia afuera, citar en vez de pegar, no repetir, priorizar los AC que duelen, pedir solo lo recortado) |
| AC-11 el paquete completo | OK | Unit `render_should_show_state_missing_pieces_and_the_cut` + integracion `revision_should_gather_the_package_and_report_its_size` y `revision_should_cross_the_verify_state_and_expose_json` |
| AC-12 presupuesto declarado | OK | Unit `recortar_should_declare_what_was_left_out` + integracion `revision_should_respect_the_budget_and_say_what_it_cut` (`se muestran 20 de ...`) |
| AC-12b reporta su tamaño | OK | Unit `tamano_should_report_the_cost_before_spending_it` + la linea `[paquete] N lineas, ~M tokens estimados` verificada en integracion |
| AC-12c entra en un turno | OK | **Medido en real** sobre esta feature: ver abajo |
| AC-13 tolera ausencias | OK | Integracion: sin `verify-1.md` ni `impl-1.md`, el paquete se arma igual y los nombra en `## Falta` |
| AC-14 JSON | OK | Integracion `revision_should_cross_the_verify_state_and_expose_json`: estados por AC, evidencia, recorte y tamaño |
| AC-15 comandos oficiales | OK | `cargo test`: 355 unit + 173 integracion = **528**; `clippy --all-targets -- -D warnings` limpio; `setup_smoke.sh` exit 0; `harness_check.sh` limpio |
| AC-16 la regla sobre si misma | OK | El veredicto (`docs/review-51.md`) se escribio con `revision --feature 51` como material y declara que intento refutar cada AC |

## AC-12c: cuanto cuesta el paquete de esta feature

Medido sobre esta misma feature, con el presupuesto por default:

```
$ harness revision --feature 51
[paquete] 478 lineas, ~6654 tokens estimados.
```

Contra los **10 millones de tokens** que motivaron la feature, el paquete entra
holgado en un turno: tres ordenes de magnitud menos. El numero varia con el
tamaño del diff; lo que no varia es que el paquete lo dice antes de que lo leas,
y que el presupuesto lo acota.

## Dos bugs que encontro la verificacion en vivo (los dos, de features anteriores)

1. **El paquete no veia el trabajo sin commitear** (bug de diseño de ESTA
   feature, encontrado al correrla sobre si misma): comparaba `base...rama`, y
   como el reviewer revisa ANTES del cierre, el paquete decia "archivos tocados:
   ninguno". Ahora compara el worktree contra la base, asi que incluye lo
   commiteado, lo modificado y —listados aparte— los archivos nuevos que
   todavia no pasaron por `git add`.
2. **El foco por worktree contaminaba arboles ajenos** (bug de la feature #47):
   `worktree_actual()` miraba el CWD sin comprobar que ese worktree fuera del
   MISMO repo que el arnes. Correr la suite parado en un worktree hizo que los
   sandboxes escribieran sus specs en el `docs/` del worktree real. Ahora se
   exige que el repo principal del CWD y el del arnes sean el mismo. Sin este
   arreglo, el binario de un proyecto invocado desde el worktree de otro repo le
   desviaba los documentos.
