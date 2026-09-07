# Evidencia de implementación — Feature #81

Spec: `docs/spec-feature-81-consolidar-la-declaracion-mutua-en-relacionadas-.md` (approved)

## Cambios

- `rust/src/consolidacion.rs:200-218,267-272` agrega la procedencia estructural
  (`Relacionadas`, `Modelo`, `Ambas`), baja la señal local a `0.50` y añade la
  explicación de única evidencia sin interpretar el texto del modelo.
- `rust/src/consolidacion.rs:297-333,459-469` deduplica por miembros y motivos
  completos, conserva el máximo y marca la combinación de fuentes.
- `rust/src/commands/leccion.rs:726-750` mantiene validación y bitácora y usa
  el motivo derivado de procedencia al informar.
- `rust/src/consolidacion.rs:806-1001` añade regresiones para local, backend
  ausente, modelo exclusivo, combinación, orden, duplicados y motivos que
  imitan etiquetas locales.
- `rust/tests/cli_basics.rs:4515-4554` verifica el recorrido CLI y la
  inmutabilidad de las lecciones.
- `tests/consolidar_check.sh:130-269` agrega fixtures de backend controlado para
  ausencia, respuesta vacía, error, respuesta malformada y combinación.

## Evidencia por AC

| AC | Evidencia |
| --- | --- |
| AC-1 | `rust/src/consolidacion.rs:806-829` fija confianza 0.50 y única evidencia; `rust/tests/cli_basics.rs:4515-4554` lo verifica por CLI. |
| AC-2 | `rust/src/consolidacion.rs:944-962` evita confirmar pares no propuestos; `tests/consolidar_check.sh:131-154` cubre apagado, vacío, malformado y falla. |
| AC-3 | `rust/src/consolidacion.rs:858-1001` cubre máximo, orden inverso, duplicados, modelo solo y motivo impostor; `tests/consolidar_check.sh:157-169` cubre la salida combinada. |
| AC-4 | `rust/src/commands/leccion.rs:726-735` conserva validación, bitácora y no escritura; `rust/tests/cli_basics.rs:4539-4553` comprueba bytes y ausencia de backup. |
| AC-5 | `docs/verify-81.md` registra cargo test, clippy, `tests/consolidar_check.sh` y `tests/setup_smoke.sh` verdes; `progress/repro-81-antes.txt` y `progress/repro-81-despues.txt` muestran los cuatro pares reales 1.00 -> 0.50. La regresión roja previa queda en `progress/test-81-rojo.log` y `progress/test-81-cli-rojo.log`. |

## Decisiones y límites

Se implementó OBS-1 aprobada: 0.50 es un peso heurístico local y la unión
conserva el máximo. No se añadió dependencia, comando, flag ni llamada de red.
El hub no respondió durante el contexto; el cambio no depende de él.

## Comandos ejecutados

- `cargo test --manifest-path rust/Cargo.toml --locked` — verde.
- `cargo clippy --manifest-path rust/Cargo.toml --all-targets --all-features --locked -- -D warnings` — verde.
- `bash tests/consolidar_check.sh` — verde.
- `bash tests/setup_smoke.sh` — verde.
- `bash harness_check.sh` desde el checkout principal — verde.
- `git diff --check` — limpio.
