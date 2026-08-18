# Plan - Feature #29: prd_y_sdd_siempre_al_dia

Estado: in_progress
Microservicios:
- harness

## Alcance

Que el cuerpo del PRD, el SDD y `docs/architecture.md` dejen de poder quedar
mintiendo. El agente PROPONE, el usuario APRUEBA, el binario ESCRIBE (D-1).

Spec aprobado (23 AC, cada uno con su `Comando:`):
`docs/spec-feature-29-prd-y-sdd-siempre-al-dia.md`.

## Peldano elegido: 1 para el gate, 3 para el comando

| Peldano | ¿Alcanzaba? |
| --- | --- |
| **1. extender lo que existe** | **SI para el gate**: entra en `close.rs` junto a los otros tres, con la misma forma, y el alcance sale de funciones de `prd.rs` que ya existen (`feature_prd_slug`, `segments`) |
| 2. flag en un comando existente | **NO**. Un `close --aplicar-docs` haria que el MISMO comando que gatea sea el que escribe, y el usuario aprobaria a ciegas algo que todavia no vio. La D-1 exige que ver y aprobar ocurran en un turno separado del cierre |
| **3. comando nuevo** | **SI para `propose`/`apply`**, y se monta DENTRO del grupo `prd` que ya existe (`PrdCommand` tiene hoy `add` y `tree`), asi que no suma verbo de nivel superior |
| 4. superficie nueva | `docs/prd-diff-<id>.md` es un artefacto por feature, hermano de `docs/verify-<id>.md`, no una superficie que el instalador siembre |
| 5. dependencia nueva | no. El anclaje es reemplazo de texto literal, no un motor de patch (Articulo 6 sin ADR) |

**Peldano elegido: 3 (comando nuevo, dentro del grupo `prd`) porque el flag no
alcanza: aplicar la propuesta es una escritura sobre documentos del USUARIO que
exige su SI en un turno SEPARADO del cierre, y un `close --aplicar-docs` haria
que el mismo comando que gatea sea el que escribe.**

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder, como en las doce features anteriores.

- `rust/src/documentos.rs` (NUEVO): alcance, parser de la propuesta, validacion
  de veredictos y citas, plan de escritura y `gate`. Todo **puro**.
- `rust/src/commands/prd.rs`: `propose` y `apply`; `rust/src/cli.rs`: dos
  variantes mas de `PrdCommand`.
- `rust/src/commands/close.rs`: el cuarto gate.
- `CHECKPOINTS.md`, roles, README, UPDATING, todos con espejo en `templates/`.

**Riesgo central, y es nuevo**: esta feature le pide algo al USUARIO en **cada**
cierre. Si eso se vuelve molesto, la regla se apaga y la feature muere. Por eso
el formato de la propuesta —cuatro renglones legibles en 30 segundos— importa
mas que el gate.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

El diseno salio de un workflow de 18 agentes (6 mapearon el codigo con 73
hallazgos citando `archivo:linea`, 3 disenaron en paralelo, 9 refutaron). Las
cuatro afirmaciones sobre las que se apoya se verificaron **a mano** antes de
escribir el spec:

- `prd::feature_prd_slug` (`prd.rs:673`) y `prd::segments` (`prd.rs:77`) dan la
  cadena PRD de una feature: el alcance no hay que inventarlo.
- `prd::echo_close` corta secciones con `starts_with("## ")` (`prd.rs:629`) y
  `docs/architecture.md` tiene **3** encabezados `###`. Anclar por seccion se los
  tragaria -> se ancla por texto literal.
- `verificacion::ejecutar` lanza los `Comando:` con `sh -c`
  (`verificacion.rs:163`) -> un AC no puede invocar `prd apply --yes` sin
  saltearse el ritual. Es el AC-19.
- `approve_spec.rs:44-55` es el molde del `[GATE]` de tres pasos + `Exit::code(2)`;
  no hay funcion compartida, se duplica la forma con texto propio.

Tres bloqueos que la refutacion mato antes de que llegaran al codigo:

1. **Deadlock de frescura** (AC-18): exigir `mtime(propuesta) >= mtime(reporte de
   verify)` dejaria la propuesta vieja para siempre, porque `verify` reescribe su
   reporte en cada corrida y `prd apply` es idempotente.
2. **Auto-aplicacion via `verify`** (AC-19).
3. **El slicing por `## `** (AC-8).

## Delegacion (implementer)

- **D1 (AC-1, AC-2, AC-3)** — `documentos.rs`: `alcance(paths, feature)` reusa
  `prd::feature_prd_slug` + `prd::segments` para la cadena, agrega SDD y
  `architecture.md`, omite lo que no existe. **Funcion pura.**
- **D2 (AC-4, AC-5, AC-6)** — `prd propose`: siembra un bloque por documento con
  `Veredicto: PENDIENTE`, conserva lo ya contestado, y **el binario** precomputa
  `Presente en:` / `Ausente en:`.
- **D3 (AC-7..AC-11)** — El parser y los tres veredictos: lista cerrada de
  bloques (no se puede agregar ni quitar), `cambio` con `Antes:`/`Despues:`,
  `ya-esta <archivo>:<L1>-<L2>` **con la cita verificada contra el disco**, y
  `no-aplica <razon>` con razon no vacia.
- **D4 (AC-12, AC-13, AC-14)** — El ritual: sin `--yes` muestra y sale 2 con el
  `[GATE]` calcado de `approve-spec`; con `--yes` escribe, sella y deja bitacora;
  idempotente **por contenido**.
- **D5 (AC-15, AC-16)** — Composicion con la #26: la propuesta vive fuera de
  `docs/prd/**`, y `prd apply --yes` registra sus escrituras con
  `commands::rutas::registrar_escritura_del_arnes`, igual que `close` y
  `prd add`.
- **D6 (AC-17, AC-18)** — `documentos::gate` en `close.rs`, junto a los otros
  tres, leyendo `rules.require_docs_al_dia` con `unwrap_or(false)`. **Sin
  frescura contra el reporte de verify**, y un test que lo fija.
- **D7 (AC-19)** — El test que prohibe que cualquier `Comando:` de cualquier spec
  invoque `prd apply --yes`. Se corre sobre los specs REALES del repo.
- **D8 (AC-20..AC-23)** — CHECKPOINTS, los tres roles, README y UPDATING con sus
  espejos, y la verificacion oficial.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-23 en `docs/impl-29.md`; veredicto en `docs/review-29.md`.
- `sh harness_cli verify --feature 29` **verde**, con sus 23 comandos.
- **La feature se aplica a si misma**: se corre `prd propose --feature 29` sobre
  este repo y se contesta de verdad. Si el resultado no mejora `architecture.md`
  —que hoy no menciona `doctor.rs` ni `rutas.rs`— la feature no sirve.
- **La prueba del rojo sobre la cita**: escribir un `ya-esta` que apunte a un
  rango que NO dice eso, y confirmar que `prd apply` lo rechaza nombrandolo.
- **El ritual no se puede saltear**: con la propuesta completa, `prd apply` sin
  `--yes` no escribe un byte (mtimes antes y despues, identicos).
- **El gate no se deadlockea**: aplicar, correr `verify` despues, y confirmar que
  `close` sigue pasando.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash tests/parity_check.sh`,
  `bash harness_check.sh`: todo verde.

## Riesgos

- **Que se vuelva ceremonia y se apague.** Riesgo numero uno. Mitigado por el
  formato (cuatro renglones), por `no-aplica` como salida honesta y por que la
  regla es opcional. No se puede mitigar del todo: si Alan se cansa, se apaga.
- **Que el agente mienta con `ya-esta`.** Mitigado estructuralmente: la cita se
  verifica contra el disco (AC-9). Es la unica de las tres respuestas que se
  puede refutar por maquina.
- **Romper el ritual sin darse cuenta.** El AC-19 existe porque la interaccion
  entre esta feature y la #23 abre un agujero real.
- **Escribir mal en un documento del usuario.** El anclaje literal falla ruidoso
  (si el `Antes:` no esta, no escribe) en vez de escribir en el lugar equivocado.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las tres del spec fueron decididas por Alan el 2026-08-18, y
las cuatro de fondo (D-1..D-4) estaban decididas desde el 2026-08-17 en
`docs/analisis-drift-documentos-vs-codigo.md`:

- OBS-1 el gate exige la propuesta **aplicada** -> D6.
- OBS-2 `require_docs_al_dia` **encendida** en este repo.
- OBS-3 **los cuatro documentos** desde el dia uno -> D1.

## Skills aplicadas

- **`rust-patterns`**: `documentos.rs` separa decidir (puro: alcance, parseo,
  validacion, plan de escritura) de actuar (el comando escribe). La promesa "el
  gate solo lee" la sostiene que el modulo del gate no tiene con que escribir.
- **`rust-best-practices`**: se reusan `prd::feature_prd_slug`, `prd::segments`,
  `progress::log` y `commands::rutas::registrar_escritura_del_arnes` en vez de
  reimplementarlos; cero dependencias nuevas.
- **`rust-testing`**: la cita verificable es la pieza que hace testeable lo que
  normalmente no lo es (¿el documento refleja el codigo?). Y el AC-19 se prueba
  contra los specs REALES del repo, no contra fixtures.

### Avance 2026-08-18
Plan de la #29 escrito: D1-D8 citando cada AC. El diseno salio de un workflow de 18 agentes y tres bloqueos verificados contra el codigo murieron antes de llegar a la implementacion: el deadlock de frescura, la auto-aplicacion via verify (un AC no puede declarar `Comando: prd apply --yes`) y el slicing por `## ` que se traga los `###` de architecture.md.

### Avance 2026-08-18T13:14:59Z
Feature #29 implementada: prd propose siembra una pregunta por documento (alcance calculado por el binario desde el arbol real), el agente contesta con cambio/ya-esta/no-aplica, el binario VERIFICA las citas contra el disco, y solo con el SI del usuario prd apply --yes escribe. Aplicada sobre este repo: architecture.md ya documenta doctor.rs, rutas.rs y documentos.rs, y el SDD dejo de publicar <nombre del proyecto> a Confluence. Bug encontrado dogfooding y arreglado: la idempotencia por contenido fallaba cuando Despues contiene a Antes (el patron 'insertar antes de esta linea') y DUPLICABA el texto.

---
Cerrado: 2026-08-18T13:17:46Z - status=done - El PRD, el SDD y architecture.md dejan de poder quedar mintiendo: el binario calcula el alcance desde el arbol real, siembra una pregunta por documento, verifica las citas contra el disco, y solo con el SI del usuario prd apply --yes escribe. Aplicada sobre este repo: corrigio el drift real de architecture.md y el SDD que se publicaba a Confluence con placeholders.
