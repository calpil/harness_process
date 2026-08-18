# Veredicto del reviewer - Feature #18: nudge_de_aprendizaje

Spec: `docs/spec-feature-18-nudge-de-aprendizaje.md` (`Estado: approved`, sello
`2026-08-16T23:06:20Z por USUARIO (confirmacion explicita)`, 21 AC)
Plan: `docs/plan-feature-18-nudge-de-aprendizaje.md` (D1-D8)
Evidencia: `docs/impl-18.md`
PRD de origen: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 2)

## Veredicto global: `approved`

Los 21 AC cubiertos con evidencia ejecutada. A diferencia de la #17, **no queda
ningun AC parcial**: esta feature no toco `setup_smoke.ps1`, asi que la brecha de
PowerShell no crece (sigue siendo la misma de la #17, y Alan decidio el
2026-08-16 dejarla declarada en vez de instalar `pwsh`).

## Trazabilidad de la aprobacion (Articulo 2)

- Sello de `approve-spec` con quien/cuando y las siete decisiones OBS-1..OBS-7.
- Linea `approve-spec feature #18` en `progress/history.md`.
- `check-spec` => `[OK] Spec aprobado y fresco`; `check-plan` => `[OK] Plan
  fresco`.
- El spec se re-firmo una vez para corregir el diagrama, que habia quedado con el
  default viejo (10) despues de que Alan decidiera 25. La nota del sello lo dice
  explicitamente: **no fue una decision nueva**, fue alinear el diagrama con el
  AC-1 y el AC-6 ya aprobados.

## Estado por AC

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | Corrida real con intervalo 3: silencio, silencio, recordatorio de 4 lineas. `texto_recordatorio_should_stay_short` cuenta las lineas |
| AC-2 | cubierto | Mismo test de integracion: las invocaciones 1..N-1 no dicen nada de lecciones |
| AC-3 | cubierto | Sin `docs/lecciones/` el contador **ni se crea** (la guarda esta antes de tocar el filesystem) |
| AC-4 | cubierto | `7:2` -> feature 8 -> `8:1`; el id va adentro del archivo |
| AC-5 | cubierto | Default 25 con `rules` ausente/vacio/propio; `0` y `-1` apagan |
| AC-6 | cubierto | Test que copia la guia REAL y verifica las cinco reglas literales; ademas que no se cuela `## Sin secretos` |
| AC-7 | cubierto | Ni con `--leccion <clase>` ni con `--leccion ninguna` |
| AC-8 | cubierto | `--status blocked` no dispara el contrato |
| AC-9 | cubierto | Sandbox sin `docs/lecciones/`: stderr limpio |
| AC-10 | cubierto | `status.success()`, stdout con el mensaje de cierre y **sin** el contrato |
| AC-11 | cubierto | Primer aviso emite y deja nivel 1 |
| AC-12 | cubierto | Escala 2 -> 3 y se estaciona en 3 (probado con el reloj corrido, no con esperas) |
| AC-13 | cubierto | Reset a piso, y **solo escribe si hacia falta** (mtime intacto cuando ya estaba en 0) |
| AC-14 | cubierto | Vacio -> 0, basura -> 0, `"3\n"` -> 3 |
| AC-15 | cubierto | `feature_list.json` = `{ roto` -> `nudge` sale con 0 |
| AC-16 | cubierto | Los unicos `fs::write` del camino nuevo son los dos dotfiles de `progress/`; `nudge.rs` no importa `Leccion` |
| AC-17 | cubierto | Ningun camino nuevo importa `graph`; el hub de esta maquina esta caido y todo corrio |
| AC-18 | cubierto | README, UPDATING (+ espejo), architecture (modulo + los dos dotfiles) y ambas superficies |
| AC-19 | cubierto | `implementer` ("no lo ignores" + que hacer) y `reviewer` (verificar el contrato) |
| AC-20 | cubierto | `cargo test` 156+56, clippy limpio, `setup_smoke.sh` exit 0, `harness_check.sh` limpio |
| AC-21 | cubierto | Tres formas de degradar en unit test + end-to-end: el cierre **sale con exito** y emite el puntero |

## Constitution (`docs/constitution.md`)

| Articulo | Verificacion |
| --- | --- |
| 1 - Calidad y tests | 156 unit + 56 integracion (19 nuevos), clippy `-D warnings` limpio, smoke exit 0. Ninguno saltado |
| 2 - Spec aprobado | Sello + history + `check-spec` verde; la re-firma esta justificada en su nota |
| 3 - Trazabilidad AC-n | Cada D del plan cita sus AC; `impl-18.md` por AC; este veredicto lista AC-1..AC-21 |
| 4 - Seguridad y observabilidad | El texto emitido es fijo y no interpola contenido del usuario (no hay superficie de inyeccion); no se escribe fuera de `progress/`; todo va a stderr y el exit code es invariante |
| 5 - Decisiones del usuario | Las 7 OBS decididas antes de implementar; ninguna abierta |
| 6 - Reglas puente | Sin dependencias nuevas (`filetime` ya era dev-dependency); `templates/` y raiz espejados; **backend-agnostico**: el arnes emite el contrato y lo ejecuta el agente que este corriendo, sea cual sea |

## Checkpoints

- [x] Feature activa refleja el estado real.
- [x] `check-plan` y `check-spec` limpios.
- [x] Sin observaciones pendientes (7 decididas).
- [x] Plan al dia con lo implementado.
- [~] **Impacto**: `graph impacto` intentado; el hub sigue sin responder. Impacto
      derivado por inspeccion (un microservicio). Coherente con el AC-17, que
      exige justamente independencia del hub.
- [x] `graphify query` consultado; decidio el diseno del contador (reusar la
      invocacion del hook en vez de infraestructura nueva).
- [x] Tests ejecutados y verdes.
- [ ] `validate_ui.sh`: no aplica.
- [x] Evidencia y veredicto por AC.
- [x] `harness_check.sh` limpio.
- [x] **Aprendizaje declarado**: `estado-local-en-progress` (ver abajo).

## Lo que el reviewer encontro y se corrigio antes de cerrar

**El recordatorio decia "escrituras" y contaba otra cosa.** El matcher del hook
depende del backend: Claude usa `Edit|Write|MultiEdit`, pero Codex usa
`Bash|Edit|Write|apply_patch`. Con Codex, un `ls` sumaba al contador y el mensaje
igual anunciaba "Van N escrituras". Es un detalle chico y es exactamente el tipo
de detalle que este subsistema no puede permitirse: **un sistema que existe para
que el proyecto no le mienta a sus sesiones futuras no puede empezar mintiendo en
su propio mensaje.** Cambiado a "Van N acciones", con el porque documentado en el
codigo para que nadie lo "corrija" de vuelta.

## Riesgos que quedan abiertos

1. **El numero 25 es una hipotesis.** Esta razonado (a 10 el aviso se vuelve
   ruido de fondo) pero no medido. Vale revisarlo despues de unas semanas de uso
   real; la palanca (`leccion_nudge_interval`) ya existe para ajustarlo sin
   tocar codigo.
2. **`setup_smoke.ps1` sigue sin correrse.** Sin cambios respecto de la #17: esta
   feature no agrego aserciones ahi.
3. **Dogfooding a partir de aca.** `require_leccion` quedo activa en este repo
   (decision de Alan), asi que este es el primer cierre del arnes sometido a su
   propio gate. Si el gate tuviera un problema, se descubre en este cierre — que
   es exactamente para lo que se prendio.

## Nota sobre la declaracion de cierre

La feature deja `docs/lecciones/estado-local-en-progress.md`: como se guarda
estado del arnes entre invocaciones (dotfile en `progress/`, `mtime` como reloj,
toda lectura degrada al default) y los cuatro pitfalls reales, empezando por el
que casi rompe esta feature: `.last_nudge` **ya existia vacio** en toda
instalacion previa, y cambiarle el formato sin leer el vacio como default habria
roto el aviso en silencio.

Es de clase, es reusable (gobierna `autocheck`, `nudge` y lo que venga) y sale de
algo verificado en esta sesion, no de una narrativa de la tarea. **No** se corrio
`leccion usar` sobre ninguna leccion existente porque ninguna se consulto de
verdad en esta feature: inflar esa telemetria arruinaria justamente la senal que
el curador (#21) va a necesitar.
