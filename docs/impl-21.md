# Evidencia de implementacion - Feature #21: curador_de_lecciones

Spec: `docs/spec-feature-21-curador-de-lecciones.md` (`Estado: approved`, 20 AC,
sello 2026-08-17T04:08:36Z)
Plan: `docs/plan-feature-21-curador-de-lecciones.md` (D1-D10)
PRD: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 5)

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/lecciones.rs` | D1, D2 | `Umbrales`, `Transicion`, `dias_inactiva`, `pinneada`, `archivo_dir`, `scan_archivadas`; 11 tests nuevos |
| `rust/src/curador.rs` | D4, D5, D7 | NUEVO. `planificar` (lee), `aplicar` (muta), backup, rollback y reporte; 8 tests |
| `rust/src/commands/leccion.rs` | D3, D6, D8 | `lecciones status/curar/pin/unpin/archivar/restaurar/rollback` + `leccion list --archivadas` |
| `rust/src/buscar.rs` | D2 | `Fuente::LeccionArchivada` con peso 30 |
| `rust/src/cli.rs`, `main.rs` | D3 | Cableado |
| `harness_check.sh` (+ espejo) | D2 | Valida tambien `docs/lecciones/archivo/` (maxdepth 2) |
| `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md` | D10 | El ciclo, las cuatro garantias y el porque de la carpeta visible |
| `setup_harness.sh` / `.ps1`, `templates/roles/reviewer.md` (+ espejos) | D10 | Superficies y el rol que mira `status` antes de cerrar |
| `rust/tests/cli_basics.rs` | D9 | 10 tests de integracion |

## Los cuatro criterios de cierre del plan

Escritos para poder fallar. Los cuatro se corrieron de punta a punta en un
sandbox con fechas falsas:

| Criterio | Resultado |
| --- | --- |
| El modo informe no toca nada | `find -exec stat` antes y despues: **diff vacio** |
| Ciclo completo + rollback con contenido intacto | archivada -> `rollback` -> **diff vacio** contra el original |
| Archivar no la borra de la busqueda | `buscar`: activa score **130/100**, archivada **60** |
| Un `pin` sobrevive a una pasada | 200+ dias de inactividad, sigue `activa` |

Transcripcion de la corrida real:

```
=== curar (informe) ===
Evaluadas: 2 leccion(es).
  Salteadas por pin: protegida
  tecnica-vieja                             108d inactiva  -> ARCHIVAR (mover a archivo/)
Esto es solo un informe: no se toco ningun archivo.
=== mtimes despues del informe ===
IDENTICOS: el informe no toco nada

=== aplicar ===
1 transicion(es) aplicada(s).
  Backup previo: .../bkp/lecciones/20260817-041556
  Reporte: .../progress/lecciones/20260817-041556/REPORT.md
=== rollback ===
Lecciones restauradas desde el backup 20260817-041556.
=== volvio con el contenido EXACTO? ===
DIFF VACIO: contenido intacto
```

## Evidencia por AC

### AC-1 / AC-3 — `lecciones status`

Corrida sobre las lecciones reales del repo:

```
Lecciones: 5 activa(s), 0 archivada(s). Umbrales: stale >= 30d, archivo >= 90d.
  docs-generados-por-el-instalador           1 usos |  0d inactiva | activa | -> stale en 30d
  documentos-del-usuario-vs-plantillas       0 usos |  1d inactiva | activa | -> stale en 29d
  ...
Candidatas HOY: 0 a stale, 0 a archivar.
```

Los **dias que faltan** para la proxima transicion son lo que convierte el estado
en accionable. `--json` con los ocho campos del AC-3, verificado en
`lecciones_status_should_report_days_to_the_next_transition`.

### AC-2 — Sin biblioteca

Los tres subcomandos informan y salen 0
(`lecciones_should_say_so_without_a_library`).

### AC-4 / AC-5 — Umbrales EXACTOS

Probados en sus bordes, no "alrededor de":

- `transicion_should_respect_the_exact_stale_threshold`: 29 dias => `Ninguna`,
  30 dias => `AStale`.
- `transicion_should_respect_the_exact_archive_threshold`: 89 => `Ninguna`,
  90 => `AArchivada`.

Archivar **mueve**: `aplicar_should_move_the_lesson_instead_of_deleting_it`
verifica que el archivo desaparece del activo, aparece en `archivo/` y **conserva
su cuerpo**.

### AC-6 — El piso de gracia

`dias_inactiva_should_prefer_last_use_over_last_update`: una leccion nunca usada
cuenta desde `ultima_actualizacion`, asi que una recien escrita no envejece antes
que una usada. Cero usos es ausencia de evidencia, no prueba de que sobra.

### AC-7 — El pin congela

`transicion_should_never_touch_a_pinned_lesson`: la misma leccion, con 200+ dias,
pasa de `AArchivada` a `Ninguna` al pinnearla. Y en integracion,
`lecciones_pin_should_survive_a_pass` lo verifica end to end.

### AC-8 — El uso resucita

`transicion_should_revive_a_stale_lesson_that_was_used`: una `stale` que se uso
ayer vuelve a `activa`. Complemento:
`transicion_should_not_bring_an_archived_lesson_back_by_itself` — restaurar es
manual, una archivada no vuelve sola.

### AC-9 — Nada se mueve sin `--aplicar`

`lecciones_curar_should_not_touch_anything_without_aplicar` compara **contenido y
mtime** antes y despues, y verifica que ni siquiera se creo `archivo/`. Es el AC
que define la feature: la separacion `planificar()` (lee) / `aplicar()` (muta) es
lo que lo hace estructuralmente cierto, no una promesa.

### AC-10 / AC-11 / AC-12 — Backup y rollback reversible

`lecciones_curar_aplicar_should_move_backup_and_report` verifica que el backup
contiene el original **byte a byte**.
`lecciones_rollback_should_restore_exactly_and_stay_reversible` verifica el
contenido restaurado exacto **y** que aparecio un backup `pre-rollback`:
deshacer se deshace. `rollback --list` los muestra con su motivo.

### AC-13 / AC-14 / AC-15 — Comandos manuales

`pin`/`unpin` no tocan cuerpo ni telemetria
(`lecciones_pin_and_unpin_should_toggle_without_touching_the_body`).
`archivar`/`restaurar` hacen round-trip con contenido identico, y sus dos errores
(archivar lo ya archivado, restaurar lo que no lo esta) salen con exit 2.
Una clase inexistente sugiere las parecidas, reusando `lecciones::parecidas`.

### AC-16 / AC-17 — Reporte

`el_reporte_should_explain_each_transition_and_the_pins` verifica que
`REPORT.md` trae cada transicion con sus dias de inactividad, la lista de
salteadas por pin, donde quedo el backup y la linea "Nada se borro".
`aplicar_should_do_nothing_without_actions`: sin cambios **no** se crea backup ni
reporte — correr un chequeo no ensucia el repo.

### AC-18 / AC-19 — Integracion

`an_archived_lesson_should_stay_searchable_below_an_active_one` es el test que
justifica la decision de OBS-4: archiva una leccion, busca un termino que esta en
la archivada y en una activa, y verifica que **aparecen las dos** con la activa
primero. Ademas: `leccion list` ya no la muestra, `leccion list --archivadas` si.
`harness_check.sh` pasa a `maxdepth 2` para seguir validando el formato de las
archivadas.

### AC-20 — Verificacion oficial

```
$ (cd rust && cargo test --locked)
test result: ok. 213 passed; 0 failed   (unitarios, +19)
test result: ok.  83 passed; 0 failed   (integracion, +10)

$ (cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)
Finished
```

## Decisiones aplicadas

| OBS | Decision | Donde vive |
| --- | --- | --- |
| OBS-1 | La consolidacion con LLM sale a la **#28** | fuera de esta feature |
| OBS-2 | `adoptar` NO se implementa | no existe el subcomando |
| OBS-3 | La pasada solo informa | `planificar()` vs `aplicar()` |
| OBS-4 | `archivo/` visible | `ARCHIVO_DIR` + `Fuente::LeccionArchivada` |
| OBS-5 | 30/90 configurables | `Umbrales::from_rules` |

## Skills aplicadas

- **`rust-patterns`**: `Transicion` como enum con `estado_destino()` exhaustivo —
  el tercer uso del mismo patron (tras `Coincidencia` en la #19 y `Fuente` en la
  #20), ya consolidado como la forma de este repo para modelar decisiones.
- **`rust-best-practices`**: extender `lecciones.rs` en vez de crear un modulo
  paralelo; separar la funcion pura (`transicion`, `planificar`) de la I/O
  (`aplicar`), que es lo que hace testeable un ciclo de 90 dias en 10 ms.
- **`rust-testing`**: umbrales probados en sus **bordes exactos** (29/30, 89/90),
  no "alrededor de"; helper `seed_leccion` con fechas controladas.
- **`rust-async-patterns`**: no aplica.

## Riesgos pendientes para el reviewer

1. **Los umbrales no se ejercitaron en la realidad.** Ninguna leccion de este
   repo tiene 30 dias; todo se probo con fechas falsas. La logica esta cubierta,
   el uso real no: la primera pasada de verdad llega en un mes y vale mirarla.
2. **`setup_smoke.ps1` sin ejecutar** (igual que #17-#20).
3. **El backup copia el arbol entero en cada pasada.** Con decenas de lecciones
   chicas es irrelevante; con cientos habria que podar backups viejos. No hay
   politica de retencion: es deuda consciente, no un olvido.
