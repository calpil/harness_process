# Plan - Feature #50: mensaje_de_cierre_dice_la_verdad

Estado: in_progress
Microservicios:
- harness

## Alcance

Que la linea del cierre sobre la rama y el worktree informe lo que hay en el
repo en vez de afirmar por costumbre. Entra: la comprobacion y las cuatro
variantes del mensaje, con un test cada una. No entra: ofrecer limpiar, ni
auditar otros mensajes.

## Impacto entre microservicios

Un solo microservicio: `harness`. Solo cambia texto de salida en el camino de
cierre que NO integra; el resto del cierre (estado, archivado, gates) no se
toca (AC-6).

## Consulta al grafo (graphify)

No hace falta: el cambio esta acotado a `integrar()` en
`rust/src/commands/close.rs`, con dos helpers que ya existen (`git::rama_existe`
y una comprobacion de directorio).

## Delegacion (implementer)

- D1 [AC-1, AC-2, AC-3, AC-4, AC-5]: en `close::integrar()`, reemplazar la
  afirmacion fija por las cuatro variantes segun lo que exista, y el silencio
  cuando no queda nada.
- D2 [AC-1..AC-4, AC-7]: una funcion pura que devuelva el mensaje (o `None`) a
  partir de dos booleanos y el nombre de la rama, con un test por combinacion.
- D3 [AC-6, AC-7]: los cuatro comandos oficiales y el cierre de esta misma
  feature como prueba real.

## Criterios de cierre (reviewer)

- Evidencia por AC-n en `docs/impl-50.md`.
- `cargo test`, `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh` limpios.
- El cierre de esta feature imprime la variante correcta para su caso real.

## Riesgos

- R1: que el mensaje quede mas largo y ruidoso. Mitigacion: una sola linea por
  caso, y silencio cuando no hay nada que decir.

## Observaciones (decisiones pendientes)

- OBS-1 [REGISTRADA]: lo encontro el uso real, no un test; por eso D2 exige la
  funcion pura y su tabla de casos.

---
Cerrado: 2026-08-22T12:41:32Z - status=done - El cierre que no integra informa lo que realmente existe (rama y/o worktree) y calla cuando no queda nada
