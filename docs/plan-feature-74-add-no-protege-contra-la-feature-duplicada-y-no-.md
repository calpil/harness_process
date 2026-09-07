# Plan - Feature #74: add no protege contra la feature duplicada

Spec: docs/spec-feature-74-add-no-protege-contra-la-feature-duplicada-y-no-.md

## Alcance

Un modulo puro nuevo (`rust/src/duplicados.rs`: normalizar, buscar por nombre,
buscar por clave), tres lineas de decision en `add.rs` ANTES de escribir, un
flag `--clave` en `cli.rs`, docs. Sin instalador, sin ps1, sin migracion.

## Peldano de huella

Peldano 1 (extender un comando existente con una validacion y un flag
opcional). Ningun comando nuevo, ninguna dependencia.

## Delegacion (implementer)

- D-1 (tests PRIMERO, AC-1..AC-5): unitarios de `normalizar`/`buscar` en el
  modulo (`#[cfg(test)]`), e integracion en `rust/tests/cli_basics.rs` con
  nombres que describen el escenario: `add_duplicado_abierto_...`,
  `add_mismo_nombre_cerrada_...`, `add_clave_idempotente_...`,
  `add_sin_clave_no_cambia_el_backlog_...`. Correrlos antes del fix: caen por
  el aserto (exit 2 esperado, `[i]` esperado, segunda feature que no deberia
  existir), no por precondicion.
- D-2 (AC-4): `duplicados::normalizar` — minusculas, sin acentos (NFKD o tabla
  fija de vocales), solo `[a-z0-9]`, palabras vacias fuera, espacios
  colapsados. `Coincidencia` como enum (rust-patterns: estados como enum).
- D-3 (AC-1, AC-2): `duplicados::buscar(features, nombre)`: abierta gana sobre
  cerrada; `add.rs` decide antes de `load`->`push`, en el mismo bloque que
  `--kind`/`--prd`/`--depends-on`. El rechazo es `Exit{2}` con el mensaje del
  spec; el aviso es `eprintln!("[i] ...")`.
- D-4 (AC-3): `--clave` en `cli.rs` (`Command::Add`), `duplicados::por_clave`,
  retorno temprano `Feature #N ya existe (clave K).` sin escribir, sin `log`,
  sin `emit::on_add`; si no existe, `feature.insert("clave", ...)`.
- D-5 (AC-6, AC-7): README (seccion de `add`), UPDATING (dos copias, `cmp`),
  `add --help`; suite, clippy, paridad.

## Criterios de cierre (reviewer)

- Cada test nuevo cae contra HEAD por su aserto (leer el mensaje).
- Mutantes: `normalizar` que no baja a minusculas tumba AC-1 y AC-4; `buscar`
  que ignora `blocked` como abierta tumba AC-1.
- `add` sin `--clave` y sin duplicado produce el MISMO JSON que antes (AC-5):
  ningun campo nuevo se cuela.

## Riesgos

- R-1: la lista de palabras vacias es un criterio; corta o larga, cambia que
  cuenta como "igual". Se elige corta (articulos, preposiciones, `no`, `y`) y
  se documenta en el modulo: agregarle palabras es una decision, no un ajuste.
- R-2: `blocked` cuenta como ABIERTA a proposito: una feature bloqueada sigue
  siendo la feature; cargar otra igual es esconder el bloqueo.

## Observaciones (decisiones pendientes)

- OBS-1: `--clave` vs `--idempotency-key`.
- OBS-2: sin aviso por nombres parecidos (medido 0 pares).
- OBS-3: rama de integracion.

---
Cerrado: 2026-09-07T02:47:32Z - status=done - 
