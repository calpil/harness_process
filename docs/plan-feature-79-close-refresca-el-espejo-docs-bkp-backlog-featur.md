# Plan - Feature #79: close refresca el espejo del backlog

Spec: docs/spec-feature-79-close-refresca-el-espejo-docs-bkp-backlog-featur.md

## Alcance

Un modulo nuevo (`rust/src/espejo.rs`: politica como enum, decision pura,
copia atomica), una llamada en la FASE 3 de `close` justo despues de
`save_features`, una frase en el mensaje de cierre, docs. Sin instalador, sin
ps1, sin bitacora.

## Peldano de huella

Peldano 1: extender un comando existente con una regla opcional en `rules`.
Ningun comando ni flag nuevo.

## Delegacion (implementer)

- D-1 (tests PRIMERO, AC-1..AC-7): unitarios de `Politica::from_rules` y
  `decidir` en `#[cfg(test)]` del modulo (seis combinaciones); integracion en
  `rust/tests/cli_basics.rs`: espejo existente + done; espejo + blocked; Auto
  sin espejo; Siempre sin espejo; Nunca con espejo viejo; destino ilegible.
  Correrlos contra HEAD: los de integracion caen por el aserto (espejo viejo
  donde deberia estar fresco; stdout sin la frase); los unitarios caen por no
  compilar hasta que exista el modulo (precondicion esperada: el modulo ES la
  feature).
- D-2 (AC-6): `espejo::Politica` {Auto, Siempre, Nunca}, `from_rules`
  (ausente/no bool -> Auto), `decidir(politica, existe) -> bool`.
- D-3 (AC-1..AC-5, AC-7, AC-10): `espejo::refrescar(raiz, backlog, bitacora,
  politica)`: `decidir` contra el directorio `raiz/docs/bkp-backlog/`,
  `create_dir_all` solo en Siempre, `write_text_atomic` de cada fuente que
  exista; devuelve las rutas escritas. En `close`, DESPUES de `save_features`
  y de la linea de bitacora: vacio calla, `Err` -> `[!]` por stderr y sigue;
  las rutas relativas se suman al mensaje final.
- D-4 (AC-8): UPDATING (dos copias), README (parrafo junto al del instalador
  #78), architecture (bullet del modulo).
- D-5 (AC-9): suite, clippy, paridad.
- D-6 (ultracode): revision adversarial con un workflow chico (3 lentes:
  correctitud contra el spec, casos borde y mutacion, docs y paridad) ANTES de
  `verify`; lo que encuentre se corrige y queda en el review.

## Criterios de cierre (reviewer)

- Cada test de integracion cae contra HEAD por su aserto.
- Mutantes: `decidir` que devuelve `true` en Auto sin archivo tumba AC-3;
  `refrescar` que no vuelve a leer el backlog despues de `save_features` (copia
  el `data` en memoria de antes del cierre) tumba AC-1 (el espejo no tendria
  `done`).
- El cierre con destino ilegible termina en 0 y la feature queda cerrada.

## Riesgos

- R-1: con `docs/` como repo aparte, el espejo queda modificado en ese repo y
  nadie lo commitea si el usuario no lo hace; es lo mismo que la bitacora del
  PRD, y el mensaje lo dice.
- R-2: un backlog grande (miles de features) se copia entero en cada cierre;
  hoy son 140 KB.

## Observaciones (decisiones pendientes)

- OBS-1: rama de integracion.
- OBS-2: sin refresco en `add`/`start` (propuesta: no).
- OBS-3 (DECIDIDA: incluido): `history.md` se espeja en el mismo paso, despues
  de la linea de bitacora del cierre (AC-10).

---
Cerrado: 2026-09-07T03:11:05Z - status=done - 
