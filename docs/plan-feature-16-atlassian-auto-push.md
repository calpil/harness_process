# Plan - Feature #16: atlassian_auto_push

Estado: in_progress
Microservicios:
- harness

## Alcance

Cerrar la segunda mitad de la integracion: que el flujo empuje solo. Entra el
worker detached (`atlassian-worker`, subcomando oculto) que corre `apply` +
`publish` reusando el codigo de la feature #15, su lock y su log; el disparo
desde las seis transiciones (incluida `prd add`, que hoy no emite nada); el
interruptor de tres niveles; `add --kind bug|feature|task` con su mapeo de
tipos; y la documentacion. No entra: reintentos programados, sincronizacion
desde Jira, ni reutilizar epics existentes.

## Impacto entre microservicios

Un solo microservicio: `harness`. El cambio es aditivo y con interruptor: sin
binding no corre nada (AC-15) y sin token tampoco (AC-12), asi que ningun repo
instalado cambia de comportamiento hasta que tenga las dos cosas. El campo
`kind` de `feature_list.json` es opcional: las 15 features ya cargadas siguen
leyendose igual.

## Consulta al grafo (graphify)

El alcance toca rutas ya conocidas por la feature #15 (`rust/src/atlassian/`,
`rust/src/commands/`) mas `rust/src/graphify.rs` como modelo del worker
detached. No hace falta consulta nueva.

## Delegacion (implementer)

- D1 [AC-1, AC-2, AC-4, AC-4b, AC-5]: `atlassian::push` — lanzador detached con lock
  (`progress/atlassian/.push.lock`), subcomando oculto `atlassian-worker` que
  corre `apply` + `publish` y escribe `last-push.log`, liberando el lock
  SIEMPRE, con una segunda pasada si aparecieron intents nuevos durante la
  primera (OBS-9). Modelado sobre `graphify::refresh_bg` / `graphify::worker`.
- D2 [AC-12, AC-13, AC-14]: interruptor de tres niveles (env
  `HARNESS_ATLASSIAN_AUTO=0` > `"auto": false` en el binding > ausencia de
  token) y el aviso, una sola vez, de como drenar con el agente.
- D3 [AC-1, AC-3]: disparo desde `add`, `start`, `advance`, `approve-spec`,
  `close` y `prd add`; este ultimo ademas emite el intent del epic del PRD
  recien creado.
- D4 [AC-8, AC-9, AC-10, AC-11]: `add --kind feature|bug|task` (validado, con
  exit 2 y lista de validos), campo `kind` opcional en `feature_list.json`,
  `issue_types.bug` / `issue_types.task` en el binding y su uso al construir el
  intent.
- D5 [AC-6, AC-7]: `publish` dentro del worker, apoyado en el hash de la #15
  para no generar versiones inutiles.
- D6 [AC-5, AC-12, AC-13]: `atlassian status` suma el estado del envio
  automatico y la fecha del ultimo intento.
- D7 [AC-16]: tests — lock ocupado, interruptor en sus tres niveles, `--kind`
  valido e invalido, ausencia de token, y que ninguna transicion cambie su exit
  code.
- D8 [AC-17]: verificacion real en `calpil.atlassian.net` con un repo fixture,
  incluyendo un `--kind bug`, SIN correr `apply` ni `publish` a mano.
- D9 [AC-1..AC-17]: documentacion — `docs/atlassian-integracion.md` (+
  `templates/`), README, UPDATING.md (+ espejo) y las superficies de agentes.

## Criterios de cierre (reviewer)

- Evidencia por AC-n en `docs/impl-16.md`.
- `cargo test`, `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh` limpios.
- Ciclo real sin comandos manuales de Atlassian (AC-17).
- Constitution: Articulo 2 (spec approved antes de implementar), Articulo 4 (el
  token nunca en el log del worker), Articulo 6 (sin dependencias nuevas;
  espejos `templates/` propagados).

## Riesgos

- R1: un worker detached mal manejado deja procesos zombis o locks trabados.
  Mitigacion: copiar el patron ya probado de graphify (lock como directorio,
  liberacion en todos los caminos) y test del lock ocupado.
- R2: publicar en CADA transicion podria multiplicar las llamadas a Confluence.
  Mitigacion: el hash corta antes de cualquier escritura, y el worker corre uno
  a la vez por el lock.
- R3: el usuario podria no enterarse de un fallo, porque el worker es
  silencioso. Mitigacion: `last-push.log`, `atlassian status` con la fecha del
  ultimo intento, y los intents que quedan visibles como pendientes.
- R4: `--kind bug` contra un proyecto sin tipo `Bug` falla. Mitigacion: AC-11
  exige error legible y que el resto de los intents se apliquen igual.

## Observaciones (decisiones pendientes)

- OBS-1 a OBS-4 [DECIDIDAS 2026-08-16]: worker detached, publicacion en cada
  transicion, `--kind` explicito sin heuristica, y envio automatico encendido
  cuando hay binding y token.
- OBS-6 a OBS-9 [DECIDIDAS 2026-08-16]: sin backfill retroactivo, publish sobre
  todo el arbol, un solo interruptor, y segunda pasada del worker.
- OBS-5 [REGISTRADA, sin accion]: el push de `close` ocurre despues de que el
  comando devolvio; para confirmarlo en el momento existe `atlassian apply`,
  que sigue siendo sincrono.

### Avance 2026-08-16T05:16:22Z
Alan aprobo en el chat el spec AMPLIADO de la #16 (29 AC): se sumaron validacion del binding (bind + status), creacion de proyecto/space solo con --create-project/--create-space, backfill completo de Jira en el primer push (OBS-12 reemplaza OBS-6), sincronia total incluyendo subtasks de features cerradas (OBS-14) y reutilizacion de epics existentes por titulo (OBS-15)

---
Cerrado: 2026-08-16T05:36:40Z - status=done - Envio automatico completo: worker detached en las seis transiciones (apply + publish), backfill del repo existente con adopcion de epics, add --kind bug/feature/task, verificacion del binding con creacion opcional e interruptor de tres niveles. Verificado en real: un add creo epic, historia y paginas sin comandos manuales
