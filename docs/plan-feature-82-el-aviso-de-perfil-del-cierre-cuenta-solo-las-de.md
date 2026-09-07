# Plan - Feature #82: el aviso de perfil del cierre cuenta solo las decisiones posteriores a la ultima entrada del perfil

Estado: in_progress
Microservicios:
- (sin servicios)

## Alcance
El aviso de perfil de la #80 (`rules.perfil_pendientes_max`) mide el acumulado
historico de decisiones sin incorporar, asi que sale en cada cierre para
siempre. Pasa a medir crecimiento: cuenta solo las decisiones sin incorporar
posteriores a la ultima entrada del perfil, y `lecciones status` y `perfil
sugerir` muestran las dos cuentas con el corte. Solo el binario Rust y sus
docs; los instaladores no cambian.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->
- Sin servicios: solo el arnes.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->
- `harness contexto --feature 82`: 14 secciones del mapa; lecciones que aplican:
  criterios-de-cierre-que-se-pueden-fallar, promesas-estructurales-vs-disciplina,
  remedios-que-la-herramienta-sugiere.

## Delegacion (implementer)
- `rust/src/perfil.rs`: `Registro.momento` (timestamp completo; plan/spec desde
  el `started_at` del backlog), `Corte` (enum: `Ninguno` / `Bitacora` /
  `Backlog`), `ultima_entrada(paths)`, `contar(registros, corte)` puro y
  `pendientes(paths)` (AC-1, AC-2, AC-3, AC-4).
- `rust/src/lecciones.rs`: `texto_avisos_de_ciclo` usa `perfil::pendientes` y
  dice nuevas/total/corte (AC-1, AC-4).
- `rust/src/commands/leccion.rs` (`status`, texto y `--json`) y
  `rust/src/commands/perfil.rs` (`sugerir`): las dos cuentas y el corte (AC-5).
- Tests primero, rojos contra HEAD por su aserto: `rust/tests/cli_basics.rs`
  (cierre, status, sugerir) y unitarios en `perfil.rs` (`contar`, `ultima_entrada`).
- Docs: README (tabla de reglas), `UPDATING.md` + `templates/UPDATING.md`,
  `docs/architecture.md` (AC-7). Corpus de `verificacion.rs`: el AC-6 (MANUAL).
- Al cerrar: quitar `rules.perfil_pendientes_max: 300` del backlog (AC-6).

## Criterios de cierre (reviewer)
- Cada AC con test o comando; los tests nuevos fallan contra HEAD por su
  aserto, no por precondicion (leccion criterios-de-cierre-que-se-pueden-fallar).
- `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- `lecciones status` en este repo muestra corte desde el backlog y menos de 25
  nuevas (AC-6).

## Riesgos
- Timestamps: se comparan como texto RFC3339 UTC (`%Y-%m-%dT%H:%M:%SZ`, el
  formato unico de `now_stamp` y `started_at`); una linea de bitacora con otro
  formato queda sin momento y se cuenta (avisa de mas).
- Un consumidor del `--json` que lea `perfil_pendientes` ve el numero nuevo
  (el que se compara con el umbral); el total queda en `perfil_pendientes_total`.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- OBS-1..3: ver el spec; se preguntan con la aprobacion.

---
Cerrado: 2026-09-07T23:33:06Z - status=done - 
