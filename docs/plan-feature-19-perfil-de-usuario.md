# Plan - Feature #19: perfil_de_usuario

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 3 del PRD `docs/prd/aprendizaje/PRD-aprendizaje.md`: el tercer almacen de
memoria. El hub guarda **eventos**, `docs/lecciones/` guarda **procedimiento**, y
`docs/perfil-usuario.md` guarda **preferencias** — y es el unico de los tres que
viaja solo hasta la superficie que cada backend lee al arrancar.

Es el documento del USUARIO: el arnes junta la evidencia y verifica; el agente
propone; **Alan decide**. Ninguna llamada a un modelo (NO1 del PRD).

Spec aprobado (20 AC): `docs/spec-feature-19-perfil-de-usuario.md`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> el hub
sigue sin responder (`connection timed out`), igual que en la #17 y la #18. No
bloquea, y el AC-17 exige justamente que el perfil no dependa de el.

Impacto por inspeccion (un microservicio, `harness`):

- `rust/src/perfil.rs` (NUEVO) — modelo, limite, matcheo por subcadena, escaneo.
- `rust/src/commands/perfil.rs` (NUEVO) + `cli.rs` — `show|add|replace|remove|sugerir`.
- `rust/src/paths.rs` — sin cambios: el perfil se resuelve desde `paths.plans`.
- `setup_harness.sh` / `.ps1` — siembra (lista de documentos del USUARIO) e
  **inyeccion** en las cuatro superficies. Es la parte de mas riesgo: toca la
  generacion de superficies, que es el corazon del instalador.
- `harness_check.sh` (+ espejo) — bloque de integridad del perfil.
- Docs, roles y espejos.

**Lo que NO cambia**: sin `docs/perfil-usuario.md`, ni el binario ni el
instalador hacen nada distinto (AC-12). Esa es la garantia de no-regresion.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`graphify query "superficies generadas inyeccion bloque marcadores"` + lectura de
`setup_harness.sh`. Lo que decidio el diseno:

- `write_agent_surface()` (linea ~790) escribe las cuatro superficies desde un
  heredoc, y se invoca cuatro veces (~2488). **Consecuencia**: la inyeccion es un
  paso POSTERIOR sobre el archivo ya escrito, no un cambio dentro del heredoc.
  Asi el heredoc queda igual y el bloque se arma desde el perfil.
- `write_kimi_hooks()` (~1393) ya resuelve exactamente este problema para el
  `config.toml` global: bloque entre marcadores propios, reemplazo idempotente
  SOLO entre marcadores, backup previo. **Se imita ese patron**, que ademas ya
  tiene tests de idempotencia en el smoke.
- `PRD_DOCS` / `$script:PrdDocs` es la lista de documentos del USUARIO: se
  siembran si faltan, no se pisan y **no** entran a los reset targets. El perfil
  va ahi (AC-1), no en `HARNESS_DOCS`. Es la leccion
  `docs-generados-por-el-instalador` aplicada tal cual.

## Delegacion (implementer)

- **D1 (AC-2, AC-3, AC-4, AC-5)** — `rust/src/perfil.rs`: parseo del documento
  (encabezado fijo + entradas `- <texto>`), `entradas()`, `usados()` contando
  **solo** las entradas, `LIMITE = 1500`, y el error de limite con su formato
  completo (ocupa hoy / ocuparia / lista de entradas actuales / instruccion de
  consolidar y reintentar en el mismo turno).
- **D2 (AC-7, AC-8)** — Matcheo: `add` rechaza duplicado exacto (exit 0, no-op);
  `buscar_unica(substring)` devuelve `NoMatch` / `Unica(idx)` / `Ambigua(lista)`,
  y cada caso tiene su mensaje. Nunca se toca la entrada equivocada.
- **D3 (AC-10)** — Escaneo previo a escribir, que **bloquea** (OBS-4): patrones de
  credencial (`password=`, `api[_-]?key`, `token`, `secret`, `BEGIN * PRIVATE
  KEY`, cadenas largas tipo clave) y Unicode invisible (zero-width, bidi
  overrides). El mensaje nombra **cual** patron disparo, para que el usuario pueda
  reescribir la frase.
- **D4 (AC-6, AC-9)** — `rust/src/commands/perfil.rs` + `cli.rs`:
  `perfil show`, `add --texto --yes`, `replace --old --texto --yes`,
  `remove --old --yes`. Sin `--yes` los tres de escritura se niegan con exit 2 y
  el ritual explicado (mismo texto y mismo espiritu que `approve-spec`). Toda
  escritura deja linea en `progress/history.md`.
- **D5 (AC-14, AC-15, AC-16)** — `perfil sugerir`: recorre `progress/history.md`
  (notas de `approve-spec`/`advance`/`close`), `docs/plan-feature-*.md` y los
  `## Observaciones` de `docs/spec-feature-*.md` (OBS-5), extrae los registros de
  decision con su feature y fecha, **marca los que ya estan citados en una entrada
  del perfil** (OBS-3), y emite al final el **contrato** de como destilar una
  entrada durable. No escribe nada. Sin material, lo dice y sale 0.
- **D6 (AC-1)** — Siembra: agregar el perfil a `PRD_DOCS` /`$script:PrdDocs` (o a
  su equivalente de "documentos del USUARIO") en ambos instaladores, con su
  plantilla en `templates/docs/`. Verificar que NO entre a los reset targets.
- **D7 (AC-11, AC-12, AC-13)** — Inyeccion: funcion nueva en ambos instaladores
  que, **despues** de `write_agent_surface`, inserta el bloque entre marcadores en
  las cuatro superficies, de forma idempotente. Sin perfil o sin entradas, no se
  inyecta nada. El comando `perfil add` avisa en su salida que el bloque se
  refresca al reinstalar (snapshot congelado).
- **D8 (AC-18)** — Bloque en `harness_check.sh` (+ espejo): perfil que supera el
  limite **bloquea** (es lo que se inyecta en cada prompt); formato roto avisa.
  Sin el archivo, el bloque se omite entero.
- **D9 (AC-19)** — Docs (README, UPDATING + espejo, architecture + plantilla,
  superficies de ambos instaladores) y roles: el lider usa `sugerir` para
  proponer; el reviewer verifica que ninguna entrada haya entrado sin el si.
- **D10 (AC-17, AC-20)** — Tests: unitarios de limite (add y replace), matcheo
  (cero/una/varias), duplicado, escaneo (cada familia de patron + unicode), y
  `sugerir` sin material; integracion del rechazo sin `--yes` y de la
  independencia del hub; smoke con siembra, no-pisa, supervivencia al `--reset`,
  inyeccion **idempotente** (dos instalaciones seguidas = un solo bloque) y
  ausencia total de bloque sin perfil.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-20 en `docs/impl-19.md`; veredicto por AC en
  `docs/review-19.md`.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh` y `bash harness_check.sh`: todo verde.
- **Demostrar la no-regresion**: un proyecto sin `docs/perfil-usuario.md` produce
  superficies byte a byte identicas a las de antes de esta feature.
- **Demostrar la idempotencia**: dos instalaciones seguidas dejan UN bloque.
- **Demostrar el gate del `--yes`**: los tres comandos de escritura se niegan sin
  el, y el rechazo del escaneo de seguridad ocurre ANTES de escribir.
- `templates/` y raiz espejados; espejos de roles regenerados.
- El hito 3 del PRD `aprendizaje` queda marcado por el cierre, y el cierre exige
  declaracion de leccion (`require_leccion` esta activa en este repo).

## Riesgos

- **Tocar la generacion de superficies.** Es el corazon del instalador y lo usan
  todos los backends. Mitigacion: la inyeccion es un paso posterior y aislado, el
  heredoc no se toca, y el AC-12 exige que sin perfil no cambie **nada**.
- **Falsos positivos del escaneo.** Una entrada legitima que mencione la palabra
  "token" seria rechazada. Mitigacion: el mensaje dice cual patron disparo, asi
  el remedio es reescribir la frase; y el costo asimetrico ya se evaluo (OBS-4).
- **Que el perfil se llene de ruido.** El limite de 1500 es la defensa, y por eso
  falla en vez de recortar.
- **Que alguien edite el perfil a mano y lo rompa.** Por eso el AC-18: el check
  bloquea si supera el limite.
- **Espejos `templates/` <-> raiz.** Esta feature toca cuatro pares de archivos;
  el gate de espejo del propio `harness_check.sh` lo detecta.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cinco observaciones del spec fueron decididas por Alan el
2026-08-16 y estan en la seccion "Observaciones" del spec y en el sello:

- OBS-1 inyeccion en las CUATRO superficies reales (`GROK.md` de la raiz no
  existe; es una correccion al texto del backlog) -> D7.
- OBS-2 limite de 1500 caracteres -> D1.
- OBS-3 `sugerir` marca lo ya incorporado -> D5.
- OBS-4 el escaneo de seguridad **bloquea** -> D3.
- OBS-5 `sugerir` lee `history.md` + planes + los `DECIDIDO` de los specs -> D5.

## Skills aplicadas

Las de Go/PostgreSQL/Angular del primer mensaje **no aplican** (el arnes es Rust +
instaladores + Markdown, y el perfil es un archivo por decision del PRD); Alan lo
confirmo. A pedido suyo se instalaron tres de Rust, que si aplican:

| Skill | Que aporta a esta feature |
| --- | --- |
| `apollographql/skills@rust-best-practices` | `#[expect(clippy::lint)]` con justificacion en vez de `#[allow]`; nombres de test `x_should_y_when_z`; una asercion por test cuando se pueda |
| `affaan-m/ecc@rust-patterns` | **Modelar estados como enum** y **matcheo exhaustivo sin catch-all**: el resultado de buscar por subcadena es un enum (`NoMatch`/`Unica`/`Ambigua`), no un `Option` que pierde el caso ambiguo. Combinadores de `Option` en vez de `match` anidado |
| `affaan-m/ecc@rust-testing` | Helpers de test documentados dentro de `mod tests`; organizacion unit + integracion |

`rust-async-patterns` quedo fuera: el binario del arnes es 100% sincrono
(`postgres` y `ureq` sync, sin runtime async), asi que no tiene donde aplicarse.

**No adoptado a proposito**: `rstest` (tests parametrizados) y `proptest`
(property-based) que las skills recomiendan. Los dos serian **dependencias nuevas**
y el Articulo 6 de la constitution las condiciona a un ADR; la cobertura
equivalente ya se logra con los bucles table-driven que este repo usa (por ejemplo
`for malo in [...]` en `validar_nombre_de_clase`). Si en algun momento la
parametrizacion se vuelve dolorosa, ahi si corresponde el ADR.

### Avance 2026-08-16T23:48:31Z
Plan de la #19 escrito: D1-D10 citando cada AC, impacto (hub caido), consulta al grafo (write_agent_surface + el patron de bloque entre marcadores de write_kimi_hooks + PRD_DOCS como lista de documentos del usuario) y riesgos. Las 5 observaciones quedaron decididas por Alan. Se instalaron y aplicaron 3 skills de Rust (best-practices, patterns, testing); rstest/proptest quedan fuera por ser dependencias nuevas sin ADR.

### Avance 2026-08-17T03:25:52Z
D1-D10 implementados: modulo perfil.rs (limite duro 1500 que falla en vez de recortar, Coincidencia como enum, escaneo de secretos y unicode invisible que bloquea, bloque para superficies, recolectar de history+planes+specs), comando perfil show|add|replace|remove|sugerir|check|bloque con --yes obligatorio, siembra via USER_DOCS e inyeccion idempotente en las 4 superficies en ambos instaladores, gate en harness_check, docs/roles y 28 tests nuevos. El pase de reviewer agrego tolerancia a binario viejo tras git pull; el dogfooding agrego el filtro de anti-senales.

---
Cerrado: 2026-08-17T03:26:09Z - status=done - Perfil de usuario: docs/perfil-usuario.md con limite duro de 1500 chars que falla en vez de recortar, escritura solo con --yes, escaneo que bloquea secretos y unicode invisible antes de escribir, inyeccion idempotente en las 4 superficies reales como snapshot congelado, y perfil sugerir que junta la evidencia de history+planes+specs y emite el contrato sin escribir nada. 20 AC cubiertos, sin dependencias nuevas y sin tocar el hub.
