# Evidencia de implementacion - Feature #37: estado_superseded

Spec: `docs/spec-feature-37-estado-superseded.md` (`Estado: approved`, 15 AC)
Plan: `docs/plan-feature-37-estado-superseded.md` (D1-D8, `Peldano elegido: 1`)
PRD: `docs/prd/PRD-master.md`

## El efecto, medido antes y despues

```
ANTES:   PRD-master  6 hitos | features: 23/36 done
DESPUES: PRD-master  6 hitos | features: 23/30 done
```

Las seis entradas que la #36 absorbio dejaron de inflar el denominador. Y se
leen distinto:

```
#27 [superseded por #36] leccion_list_alineacion_dinamica
#31 [superseded por #36] close_exit_codes_unificados
```

Antes decian `[blocked]`, que sugeria un problema donde no lo habia.

## El mapeo previo cambio el tamano de la feature

Se mapearon los **14 lugares** que comparan contra `status` ANTES de disenar, y
el resultado fue que **tres de los cuatro consumidores ya trataban bien un valor
nuevo**, porque comparan por igualdad contra `done`/`pending` en vez de hacer un
`match` exhaustivo:

| Consumidor | Que hace | ¿Habia que tocarlo? |
| --- | --- | --- |
| `commands/next.rs:10` | solo ofrece `pending` | **no** |
| `commands/close.rs:89,174` | los gates solo aplican a `done` | **no** |
| `journey.rs:260` | solo mira `done` | **no** |
| `prd.rs:686` | cuenta `done` sobre el total | **si**, una linea |
| `commands/status.rs:47` | imprime el status crudo | si, para nombrar al absorbente |

Asi que el cambio de comportamiento real fue **una linea** en
`prd::feature_counts`. Lo demas es el estado nuevo, su validacion, y **cuatro
tests de regresion** que fijan lo que ya era cierto: sin ellos, "no rompe nada"
seria una afirmacion; con ellos es un contrato.

## Evidencia por AC

`sh harness_cli verify --feature 37`: **15 verde, 0 rojo, 0 manual**.

| AC | Evidencia |
| --- | --- |
| AC-1 | `close_should_accept_the_superseded_status` |
| AC-2 | `superseded_should_demand_the_absorbing_feature` |
| AC-3 | `superseded_should_refuse_an_unknown_absorber` (inexistente **y** a si misma) |
| AC-4 | `superseded_should_record_the_absorbing_feature` (campo, no prosa) |
| AC-5 | `superseded_should_not_trigger_the_done_gates` — **con las reglas encendidas** |
| AC-6 | `next_should_not_offer_a_superseded_feature` (regresion) |
| AC-7 | `status_should_show_who_absorbed_a_superseded_feature` |
| AC-8 | `prd_tree_should_ignore_superseded_features` + `prd_tree_should_still_count_blocked_features` |
| AC-9 | `journey_should_not_flag_a_superseded_feature` (regresion) |
| AC-10 | `bash tests/superseded_check.sh migradas` sobre el backlog REAL |
| AC-11 | `blocked_features_should_stay_blocked` |
| AC-12..AC-15 | README + UPDATING + espejo; rol del reviewer; peldano; 321 + 161 tests y clippy 0 |

## Dos decisiones que valen mas que el codigo

1. **La referencia se valida.** `--absorbida-por 99` sale 2 nombrandola, y una
   feature no puede absorberse a si misma. Sin eso, el estado seria una etiqueta
   que cualquiera pone: la diferencia entre trazabilidad y prosa es que la
   trazabilidad se puede seguir.
2. **La migracion es explicita.** El arnes no puede saber cuales de tus `blocked`
   estaban absorbidas y cuales trabadas de verdad, asi que no adivina. El AC-11
   lo fija con una feature sembrada.

## Un test de regresion que vale la pena mirar

`superseded_should_not_trigger_the_done_gates` enciende
`require_spec_approved` **y** `require_leccion` antes de cerrar. Pasa. Eso es
exactamente lo que hacia falta el 2026-08-18, cuando el gate de spec rechazo
cerrar esas seis como `done` —con razon, porque nunca tuvieron spec— y hubo que
usar `blocked` como sucedaneo.

## Dos defectos que encontro la revision adversarial, DESPUES de implementar

Los 15 AC estaban verdes y la migracion hecha cuando un pase de refutacion
encontro dos cosas que ningun test mio cubria. Las dos son reales y las dos
estan arregladas.

### 1. `superseded` movia la historia de Jira de vuelta a To Do

`emit::on_close` hace `match status`, y sus brazos eran `"blocked"`, `"done"` y
`_`. Un cierre con `superseded` caia en el `_`, que **emite una transicion a
`statuses.pending`** (`emit.rs:267`). O sea: una feature absorbida quedaba
anunciada al tablero como trabajo por hacer — **exactamente el sintoma opuesto**
al que el estado vino a arreglar.

En este repo el dano fue cero porque no hay binding de Atlassian configurado y
`on_close` sale temprano, pero en cualquier instalacion con Jira las seis
migraciones habrian movido seis tickets.

Arreglado con un brazo propio que **no transiciona**:

- mandarla a `done` afirmaria que se entrego como tal, y nunca tuvo spec ni
  evidencia propia;
- dejarla caer en `_` la devuelve a la cola.

Se deja el ticket como esta y se comenta quien absorbio, para que una persona lo
cierre como corresponda en su tablero. Fijado en
`superseded_should_not_move_the_jira_ticket`.

### 2. La migracion rompio un AC ya cerrado

`tests/deudas_check.sh:32` aceptaba `done|blocked|ausente` para #27 y #31-#35, y
su comentario explicaba que **`blocked` era el estado correcto**. Ese script es
el `Comando:` del AC-13 de la feature #36, registrado verde en
`docs/verify-36.md`.

Al migrar las seis a `superseded`, ese chequeo paso a **rojo**:

```
[!] backlog-cerrado: quedan entradas abiertas que esta feature ya pago:
    #27(superseded) #31(superseded) ...
```

O sea: arreglar el vocabulario rompio una verificacion cerrada dos features
antes. Arreglado aceptando `superseded` y reescribiendo el comentario, que ahora
dice que `blocked` se sigue aceptando para instalaciones que no migraron.

**Es la leccion de la feature: un estado nuevo no se agrega solo donde se lee.**
Yo mapee los 14 lugares que comparan contra `status` en Rust y me quedaron
afuera el `match` de Atlassian —que era el unico `match` exhaustivo del repo, o
sea el unico que el compilador PODRIA haber protegido si el campo fuera un
enum— y un test de shell.

### 3. Un test que no discriminaba

`next_should_not_offer_a_superseded_feature` pasaba igual con
`--status kfjhds`: solo comprobaba una ausencia. Se reforzo para que ademas
exija que la `pending` SI se ofrezca, y asi distinga "next filtra bien" de "next
no ofrece nada".

## Limites declarados

- **El `status` sigue siendo un `&str`, no un enum.** Convertirlo tocaria los 14
  consumidores por un beneficio que esta feature no necesita. Queda anotado, no
  hecho: un valor invalido en `feature_list.json` editado a mano sigue pasando
  desapercibido para todos salvo clap.
- **No hay estado para "abandonada" o "descartada".** Son otra cosa y no hay caso
  real todavia.
- **Las superseded desaparecen del conteo del PRD.** Es lo decidido (OBS-1), y
  el costo es que `prd tree` ya no deja rastro de que existieron; `status` si.

## Para el backlog

- **`status` como enum** con matcheo exhaustivo. Ya no es una mejora estetica:
  el defecto #1 de arriba existio **porque** el campo es un `&str`. El unico
  `match` exhaustivo del repo (`emit.rs:244`) fue justo el que se rompio, y un
  enum lo habria hecho fallar en `cargo build` en vez de en produccion.
