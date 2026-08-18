# Plan - Feature #22: mapa_de_aprendizaje

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 6 y ultimo del PRD `docs/prd/aprendizaje/PRD-aprendizaje.md`: la vista que
cruza los tres almacenes (lecciones, perfil, features cerradas), muestra sus
enlaces y **senala los huecos** — enlaces rotos, features que cerraron sin
declarar nada, lecciones huerfanas.

**Solo lectura.** Sin `delete` ni `edit` (OBS-2, OBS-3): una segunda puerta para
podar podria saltear el "nunca borra" de la #21 y el gate del `--yes` de la #19.
Para corregir, el mapa **imprime el comando del almacen que corresponde**.

Spec aprobado (18 AC): `docs/spec-feature-22-mapa-de-aprendizaje.md`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder. Irrelevante por diseno: OBS-1 dejo el mapa en archivos, y el AC-14
exige que se comporte igual con el hub caido.

Impacto por inspeccion (un microservicio, `harness`):

- `rust/src/journey.rs` (NUEVO) — nodos, enlaces y huecos. Todo el dominio.
- `rust/src/commands/journey.rs` (NUEVO) + `cli.rs` — el render y `--json`.
- Docs y superficies.

**Riesgo para lo existente: ninguno.** No toca ningun modulo previo: solo LEE
`feature_list.json`, `lecciones::scan`, `lecciones::scan_archivadas` y
`Perfil::load`. Es, junto con `buscar`, la feature mas aislada del programa.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "journey mapa aprendizaje"` (segunda feature que se disena
con la #20) mas la inspeccion de los datos reales del repo:

```
lecciones:  6, con origen=[17|18|19|20|21] y usos 0..1
perfil:     4 entradas, todas con citas (#14, #16) / (#17, #19) / (#15, #16)
features:   5 cerradas con `leccion` declarada, de #17 a #21
```

**Consecuencia**: los enlaces ya existen en los datos, no hay que inventarlos ni
guardarlos. Tres tipos, todos derivables:

- `feature -> leccion` por el campo `leccion` del cierre (declarada).
- `feature -> leccion` por el `origen` del frontmatter (nacida ahi).
- `feature -> entrada de perfil` por las citas `(#n)` del texto.

Y una observacion que decidio el AC-2: las dos primeras **no son la misma cosa**.
La #17 declaro `docs-generados-por-el-instalador` pero tambien parió
`hitos-del-prd`; mostrar solo la declarada perderia la mitad de lo aprendido.

## Delegacion (implementer)

- **D1 (AC-1, AC-2, AC-4, AC-5)** — `rust/src/journey.rs`: `Nodo` con su `Tipo`
  (**enum**: `Feature` / `Leccion` / `LeccionArchivada` / `Perfil`), `Enlace` con
  su `Clase` (**enum**: `Declarada` / `Origen` / `Cita` / `Relacionada`), y
  `construir(paths) -> Mapa` que lee las tres fuentes y teje los enlaces. Las
  citas `(#n)` del perfil se extraen con un parseo de texto simple, sin regex
  compilada sobre entrada del usuario.
- **D2 (AC-3)** — Cada nodo de leccion lleva sus `usos` y `ultimo_uso`: es lo que
  distingue lo vivo de lo que solo esta escrito.
- **D3 (AC-6..AC-10, AC-16)** — `Hueco` con su `Motivo` (**enum**:
  `EnlaceRoto` / `CierreSinLeccion` / `LeccionHuerfana` / `ArchivoIlegible`).
  Se calculan en la misma pasada. Sin huecos, se dice explicitamente.
- **D4 (AC-1, AC-12)** — `rust/src/commands/journey.rs`: render cronologico
  agrupado por fecha, y por cada hueco **el comando exacto** para corregirlo
  (`lecciones archivar`, `perfil remove --yes`, `leccion show`). El comando se
  imprime como TEXTO: nunca se ejecuta nada.
- **D5 (AC-13)** — `--json` con `nodos`, `enlaces` y `huecos`.
- **D6 (AC-11, AC-14, AC-15)** — Garantias: ningun `use` de `graph`, ningun
  `write`/`create`/`remove` en los dos modulos nuevos, y repo vacio => mensaje +
  exit 0. Se verifica con test, no solo por lectura.
- **D7 (AC-17)** — Docs (README, UPDATING + espejo, architecture + plantilla,
  superficies de ambos instaladores): `journey` como vista de solo lectura, y que
  podar se hace con los comandos de cada almacen.
- **D8 (AC-18)** — Tests: unitarios del tejido de enlaces (los cuatro tipos), de
  cada tipo de hueco, del caso sin huecos, del orden cronologico y de la
  extraccion de citas; integracion del render, `--json`, repo vacio, hub caido y
  la comprobacion negativa de que no escribe nada.

## Criterios de cierre (reviewer)

Escritos para poder fallar (leccion `criterios-de-cierre-que-se-pueden-fallar`,
que ya se uso en la #21):

- Evidencia por AC-1..AC-18 en `docs/impl-22.md`; veredicto por AC en
  `docs/review-22.md`.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash harness_check.sh`: todo verde.
- **El mapa sobre el repo REAL tiene que ser correcto**, no solo sobre fixtures:
  la #17 tiene que mostrar sus DOS lecciones (la declarada
  `docs-generados-por-el-instalador` y la de origen `hitos-del-prd`), y la entrada
  3 del perfil tiene que colgar de la #19 (la mas reciente que cita).
- **Los huecos tienen que ser reales**: correr el mapa sobre este repo y verificar
  a mano que cada hueco reportado existe de verdad. Un falso positivo aca es peor
  que no reportar nada.
- **No escribe nada**: `find docs progress -newermt` despues de correr `journey`
  devuelve 0.
- **Sin huecos lo dice**: fabricar un sandbox coherente y verificar el mensaje.
- `templates/` y raiz espejados.
- Hito 6 del PRD marcado por el cierre, con declaracion de leccion.

## Riesgos

- **Falsos positivos en los huecos.** Es el riesgo real: un mapa que grita por
  cosas que estan bien se ignora en dos dias. Mitigacion: cada tipo de hueco tiene
  su test, y el criterio de cierre exige verificar a mano los que salgan sobre el
  repo real.
- **Ruido cuando el proyecto crezca.** Con 21 features entra en pantalla; con 200
  no. No se pone tope en esta feature (no hay evidencia de que haga falta) pero
  queda anotado: si molesta, `--desde <fecha>` es la salida natural.
- **Tentacion de agregar `delete` mas adelante.** Queda escrito en el spec y aca:
  la razon no es pereza, es que seria una segunda puerta a garantias que hoy son
  estructurales.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cinco del spec fueron decididas por Alan el 2026-08-17:

- OBS-1 solo archivos, sin hub ni graphify -> D1, D6.
- OBS-2 sin `journey delete` -> D4 (imprime el comando del almacen).
- OBS-3 sin `journey edit` -> idem.
- OBS-4 las features sin leccion aparecen y cuentan como hueco -> D3.
- OBS-5 la entrada de perfil se ubica en la feature mas reciente que cita -> D1.

Con OBS-2 y OBS-3 la feature entrega **menos** que el backlog: es una vista, no un
editor. Queda escrito para que nadie crea dentro de seis meses que falto algo.

## Skills aplicadas

- **`rust-patterns`**: cuarto uso del mismo patron ya consolidado en este repo —
  `Tipo`, `Clase` y `Motivo` como enums con matcheo exhaustivo, en vez de strings
  sueltos. Aca ademas rinde doble: agregar un tipo de hueco obliga al compilador a
  exigir su mensaje y su comando de correccion.
- **`rust-best-practices`**: `construir()` es una funcion que **solo lee** y
  devuelve el mapa; el render vive aparte. Esa separacion es lo que hace
  estructural la promesa de solo lectura (leccion
  `promesas-estructurales-vs-disciplina`, de la feature anterior).
- **`rust-testing`**: helpers de fixture que arman los tres almacenes; un test por
  tipo de enlace y uno por tipo de hueco, en vez de un test grande que los mezcle.

### Avance 2026-08-17T04:39:10Z
Plan de la #22 escrito: D1-D8 citando cada AC, disenado con buscar (#20) y con inspeccion de los datos reales, que mostraron que 'leccion declarada' y 'leccion de origen' NO son lo mismo (la #17 parió dos). Criterios de cierre escritos para poder fallar, incluido verificar a mano que cada hueco reportado sobre el repo real existe de verdad. Las 5 observaciones decididas; 2 reducen el alcance respecto del backlog.

### Avance 2026-08-17T04:54:16Z
D1-D8 implementados: journey.rs (Tipo/Clase/Motivo como enums, construir() solo lee, Mapa::hijos con dedup y anclaje), comando con render cronologico y --json, docs/superficies y 26 tests. Los criterios de cierre encontraron TRES bugs que los fixtures no veian: leccion declarada duplicada, perfil colgando de todas las features citadas, y 16 huecos no corregibles (features anteriores a la maquinaria) que bajaron a 0 tras acotar por era y comparar timestamps completos.

---
Cerrado: 2026-08-17T04:54:23Z - status=done - Mapa de aprendizaje: los tres almacenes juntos con sus enlaces (declarada / origen / cita / relacionada) y sus HUECOS, cada uno con el comando que lo corrige. Solo lectura por decision: sin delete ni edit, porque serian una segunda puerta al 'nunca borra' del curador y al gate del --yes del perfil. Sin hub, sin modelo, sin escrituras. Ultimo hito del PRD de aprendizaje.
