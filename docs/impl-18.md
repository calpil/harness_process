# Evidencia de implementacion - Feature #18: nudge_de_aprendizaje

Spec: `docs/spec-feature-18-nudge-de-aprendizaje.md` (`Estado: approved`, 21 AC,
sello 2026-08-16T23:06:20Z)
Plan: `docs/plan-feature-18-nudge-de-aprendizaje.md` (D1-D8)
PRD: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 2)

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/commands/nudge.rs` | D1, D2, D5 | Reescrito: backoff con nivel, contador por feature, recordatorio; 9 tests unitarios |
| `rust/src/lecciones.rs` | D3 | Lector del contrato (`seccion`, `contrato`, `texto_contrato_de_cierre`, `texto_recordatorio`); 4 tests |
| `rust/src/commands/close.rs` | D4 | Emision del contrato al final, a stderr, solo si done + sin declaracion + con `docs/lecciones/` |
| `rust/src/paths.rs` | D2 | Campo `nudge_lecciones` |
| `rust/tests/cli_basics.rs` | D8 | 6 tests de integracion, uno de ellos anti-drift contra la guia REAL |
| `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md` | D6 | Los dos disparadores, la regla, el backoff y el estado local |
| `setup_harness.sh`, `setup_harness.ps1` | D6 | Superficies generadas |
| `templates/roles/implementer.md`, `reviewer.md` (+ `roles/` y espejos) | D7 | Que hacer al ver el recordatorio; verificar el contrato |
| `docs/lecciones/estado-local-en-progress.md` | dogfood | La leccion que deja esta feature |

## Evidencia por AC

### AC-1 / AC-2 — El recordatorio sale al intervalo, no antes

Corrida real en este repo con `leccion_nudge_interval: 3`:

```
-- nudge 1 --   (silencio)
-- nudge 2 --   (silencio)
-- nudge 3 --
[harness] Van 3 escrituras en esta feature. ¿Aparecio una tecnica,
un pitfall o una correccion que una sesion futura necesite?
Mira el catalogo ('sh harness_cli leccion list') y PATCHEA la que estuvo
en juego antes de crear otra. Nada que guardar es valido; no es el default.
```

Cuatro lineas, dentro del limite de cinco que fija el AC-1 (verificado ademas por
`texto_recordatorio_should_stay_short`, que cuenta lineas). Integracion:
`nudge_should_stay_silent_until_the_interval_and_never_fail` verifica el silencio
en 1..N-1, la emision en N y el exit 0 en las tres.

### AC-3 — Sin `docs/lecciones/` no pasa nada

`recordatorio_should_do_nothing_without_the_lecciones_dir`: tras invocar el
recordatorio, **el archivo contador ni siquiera existe**. La guarda esta antes de
cualquier toque al filesystem.

### AC-4 — Cambiar de feature reinicia

`recordatorio_should_restart_when_the_active_feature_changes`: `7:2` -> se activa
la feature 8 -> `8:1`. El id va dentro del archivo, asi que no hace falta limpiar
nada desde `start` ni desde `close`.

### AC-5 — Configurable y apagable

`intervalo_recordatorio_should_default_to_25` (sin `rules`, con `rules` vacio, y
con valor propio) y `recordatorio_should_be_switchable_off` (`0` y `-1` no crean
ni el contador).

### AC-6 — El contrato se LEE de la guia

`close_should_emit_the_contract_read_from_the_real_guide` **copia la guia real
del repo** (`templates/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`) al sandbox y
verifica que el contrato emitido trae el orden de preferencia y **las cinco
reglas** de que NO capturar, con su texto literal de la guia. Es un gate
anti-drift: si alguien renombra una seccion de la guia, el contrato deja de
encontrarla y el build se pone rojo. No hay copia del texto en el binario que
pueda divergir (OBS-6).

El mismo test verifica que **no** se cuela la seccion `## Sin secretos`: la
extraccion corta en el proximo `## `.

### AC-7 / AC-8 / AC-9 — Cuando NO sale

- `close_should_not_emit_the_contract_when_the_lesson_was_declared`: ni con
  `--leccion <clase>` ni con `--leccion ninguna --leccion-motivo`.
- `close_should_not_emit_the_contract_on_blocked_or_without_lecciones_dir`:
  cubre `--status blocked` y un repo sin `docs/lecciones/`.

### AC-10 — El contrato no cambia el resultado del cierre

Mismo test de AC-6: `out.status.success()` sigue siendo cierto, el stdout tiene
`Feature #1 cerrada como done` y **no** contiene el contrato. La emision esta al
final de `close::run`, despues de que stdout y exit code quedaron fijados.

### AC-11 / AC-12 — Backoff que escala y se estaciona

`intervalo_should_double_until_the_ceiling`: 600, 1200, 2400, 3600, y 3600 para
siempre (probado con nivel 99).
`aviso_should_escalate_and_park_at_the_ceiling`: emite, sube a nivel 1, respeta
el debounce inmediato, y con el reloj corrido (`filetime::set_file_mtime`) escala
2 -> 3 -> 3 -> 3. Sin esperas reales: el test no es lento ni flaky.

### AC-13 — Vuelve al piso con feature activa

`resetear_backoff_should_return_to_the_floor_only_when_needed`: nivel 3 -> 0. Y
la parte que importa para el costo: **si ya estaba en 0 no reescribe** (el mtime
queda intacto), porque este camino corre en cada tool-use y reescribir correria
el reloj sin querer.

### AC-14 — Compatibilidad con el formato previo

`nivel_backoff_should_read_zero_from_a_legacy_empty_stamp`: archivo vacio -> 0,
basura -> 0, `"3\n"` -> 3. Es el punto que mas facil se rompia: `.last_nudge`
existia vacio en toda instalacion previa a esta feature.

### AC-15 — Best-effort absoluto

`nudge_should_never_fail_even_with_a_corrupt_backlog`: con
`feature_list.json` = `{ roto`, `nudge` sale con **0**. La estructura
`run() { let _ = inner(); Ok(()) }` se conservo, y todas las escrituras nuevas
usan `let _ = ...`.

### AC-16 — No escribe artefactos

Por construccion: los unicos `fs::write` del camino nuevo son
`paths.nudge_stamp` y `paths.nudge_lecciones`. Ninguna funcion de `nudge.rs`
importa `lecciones::Leccion` ni toca `docs/`.

### AC-17 — No depende del hub

Ningun camino nuevo importa `graph`. `nudge` nunca lo hizo, y el contrato solo
lee un archivo. En este entorno el hub esta caido y todo funciono.

### AC-18 / AC-19 — Docs y roles

- `README.md`: subseccion "El arnes te empuja solo (feature #18)".
- `UPDATING.md` (+ espejo): que cambia en una instalacion existente, con el
  aviso de que un proyecto sin `docs/lecciones/` no ve nada.
- `docs/architecture.md`: el modulo, y los dos dotfiles de `progress/` con su
  semantica (nivel + contador, mtime como reloj, vacio = 0).
- Superficies de ambos instaladores.
- `implementer`: "cuando el arnes te lo recuerde, no lo ignores" + que hacer.
- `reviewer`: verificar que un cierre sin declaracion recibio el contrato, y que
  ignorarlo en una feature que costo es `changes_requested`.

### AC-20 — Verificacion oficial

```
$ (cd rust && cargo test --locked)
test result: ok. 156 passed; 0 failed   (unitarios, +13)
test result: ok.  56 passed; 0 failed   (integracion, +6)

$ (cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)
Finished

$ bash harness_check.sh
[Ok] Harness Check limpio.
```

`tests/setup_smoke.sh`: ver la seccion de cierre de este documento.

### AC-21 — Degradacion cuando la guia no da

`contrato_should_degrade_when_the_guide_is_unusable` cubre las tres formas: sin
guia, guia sin las secciones, y encabezado sin cuerpo.
`texto_contrato_should_fall_back_to_a_pointer` verifica que el puntero entra en
dos lineas y nombra la guia.
`close_should_degrade_to_a_pointer_when_the_guide_is_missing` lo prueba end to
end: el cierre **sale con exito**, emite el puntero y no inventa el contenido del
contrato.

## Decisiones aplicadas (todas de Alan, ninguna del agente)

| OBS | Decision | Donde vive |
| --- | --- | --- |
| OBS-1 | Sin `docs/lecciones/` no se emite nada | guarda en `recordatorio_de_lecciones` y en `close.rs` |
| OBS-2 | El intervalo en `rules`, no en env | `intervalo_recordatorio()` |
| OBS-3 | Contrato solo sin declaracion | `declaracion.is_none()` en `close.rs` |
| OBS-4 | Backoff 600 s -> 3600 s | `BACKOFF_PISO` / `BACKOFF_TECHO` |
| OBS-5 | Recordatorio y plan stale independientes | los dos avisos en el mismo `inner()` |
| OBS-6 | El contrato se lee de la guia | `lecciones::contrato()` + test anti-drift |
| OBS-7 | Default 25, no 10 | `RECORDATORIO_DEFAULT` |

## Riesgos pendientes para el reviewer

1. **`tests/setup_smoke.ps1` sigue sin ejecutarse** (sin PowerShell en la
   maquina; Alan decidio el 2026-08-16 dejarlo declarado en vez de instalarlo).
   Esta feature no agrego aserciones nuevas al smoke ps1, asi que la brecha es la
   misma de la #17, ni mayor ni menor.
2. **El recordatorio podria seguir siendo ruido.** El default subio a 25 y la
   palanca (`leccion_nudge_interval: 0`) existe, pero solo el uso real va a
   decir si 25 es el numero correcto. Vale revisarlo despues de unas semanas.
3. **Primer cierre con `require_leccion` activa en este repo.** Es deliberado
   (dogfooding, decision de Alan), pero significa que este cierre va a exigir la
   declaracion: si algo del gate esta mal, se descubre aca.
