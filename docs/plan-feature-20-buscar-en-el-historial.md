# Plan - Feature #20: buscar_en_el_historial

Estado: in_progress
Microservicios:
- harness

## Alcance

Hito 4 del PRD `docs/prd/aprendizaje/PRD-aprendizaje.md`: hacer **consultable** la
memoria que las features #17-#19 acumularon. Un comando de **solo lectura** que
recorre `docs/**/*.md` y `progress/history.md`, rankea por relevancia y devuelve
`archivo:linea` + feature + fecha, con `--json` para scripts.

Sin indice, sin LLM, sin dependencias nuevas y sin tocar el hub (OBS-1).

Spec aprobado (19 AC): `docs/spec-feature-20-buscar-en-el-historial.md`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> el hub
sigue sin responder. Esta vez la irrelevancia es **por diseno**: OBS-1 saco al hub
del alcance, y el AC-14 exige que `buscar` se comporte igual con el caido.

Impacto por inspeccion (un microservicio, `harness`):

- `rust/src/buscar.rs` (NUEVO) — corpus, matcheo, ranking. Todo el dominio.
- `rust/src/commands/buscar.rs` (NUEVO) + `cli.rs` — el comando y su salida.
- Docs, roles y superficies.

**Riesgo para lo existente: ninguno.** `buscar` no escribe un byte, no tiene
estado, no tiene regla que lo apague y no toca ningun camino previo. Es la feature
mas aislada del programa: lo peor que puede pasar es que devuelva resultados
malos, no que rompa algo.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`graphify query "buscar artefactos docs ranking"` + la medicion del corpus real,
que es lo que decidio el diseno:

```
113 archivos .md en docs/ · ~28.391 lineas · 1,1 MB · history.md 28 KB
```

**Consecuencia**: a ese tamano, escanear todo en cada corrida es del orden de
milisegundos y **cualquier indice seria peor** — hay que mantenerlo, se
desactualiza y un indice viejo miente. Por eso el AC-12 prohibe el indice en vez
de dejarlo como detalle de implementacion.

Segundo hallazgo, este de la #19: `perfil::recolectar` ya recorre `history.md`,
planes y specs. **No se refactoriza para compartir codigo**: `recolectar` filtra
por senal de decision y devuelve `Registro`s con semantica propia; `buscar`
filtra por terminos del usuario. Fusionarlos acoplaria dos cosas que van a
divergir. Es la regla "duplicacion vs abstraccion equivocada" de la skill
`rust-best-practices` (capitulo 1), aplicada a conciencia.

## Delegacion (implementer)

- **D1 (AC-1, AC-15, AC-16)** — `rust/src/buscar.rs`: enumeracion del corpus.
  `docs/**/*.md` recursivo (incluye `lecciones/`, `prd/`, `adr/`, los
  `estado-feature-*` por OBS-5) mas `progress/history.md`. **Excluye** `bkp/` y
  cualquier directorio de respaldo. Un archivo ilegible o con bytes invalidos se
  saltea; sin `docs/` se informa y sale 0.
- **D2 (AC-2, AC-3)** — Matcheo: terminos en minusculas, AND sobre la linea
  (orden indistinto); si **ninguna** linea tiene todos, se cae a OR y se marca el
  resultado como tal para que el comando pueda avisarlo. Consulta vacia => exit 2
  con la forma de uso. La consulta **no** se compila a regex: se compara como
  texto (sin ReDoS ni inyeccion).
- **D3 (AC-4, AC-5, AC-6)** — Ranking. `Fuente` es un **enum** con orden
  explicito (`Leccion`/`Perfil` > `Spec`/`Plan`/`Prd` > `Impl`/`Review`/`Estado` >
  `Historia`), derivado del nombre del archivo. Score = peso de fuente + bonus por
  encabezado o campo de frontmatter + bonus por frase contigua + frescura por id
  de feature. Todo el calculo en una funcion pura y testeable.
- **D4 (AC-7, AC-8, AC-11)** — `rust/src/commands/buscar.rs`: salida humana
  (`archivo:linea` relativo a la raiz, feature, fecha, texto recortado) y `--json`
  con `archivo/linea/feature/fecha/fuente/texto/score`. JSON valido tambien sin
  resultados (lista vacia).
- **D5 (AC-8, OBS-4)** — Fecha: el timestamp de la propia linea en `history.md`;
  para el resto, el mtime del archivo.
- **D6 (AC-9, AC-10)** — Tope de 20 con `--todos`, y la linea final que dice
  **cuantos quedaron fuera**. Sin resultados: mensaje claro, sugerencia de usar
  menos terminos, exit **0**.
- **D7 (AC-12, AC-13, AC-14)** — Garantias: ningun `use` de `graph`, ninguna
  dependencia nueva en `Cargo.toml`, ningun archivo de indice. Se verifica con
  test, no solo por lectura.
- **D8 (AC-17, AC-18)** — Docs (README, UPDATING + espejo, architecture +
  plantilla, superficies de ambos instaladores) y roles: lider e implementer
  buscan antes de proponer o reconstruir; el reviewer puede verificar una cita.
- **D9 (AC-19)** — Tests: unitarios del ranking (por fuente, por encabezado, por
  frase, por frescura), del matcheo (AND, caida a OR, vacia), de la enumeracion
  (excluye `bkp/`, saltea ilegibles) y del recorte; integracion de la salida
  humana, `--json` con y sin resultados, el tope con su aviso, `--todos` y la
  independencia del hub.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-19 en `docs/impl-20.md`; veredicto por AC en
  `docs/review-20.md`.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh` y `bash harness_check.sh`: todo verde.
- **Demostrar que no escribe nada**: tras una busqueda, el arbol de trabajo queda
  identico (ni un archivo nuevo, ni un mtime cambiado).
- **Demostrar el ranking con una consulta real de este repo**, no solo con
  fixtures: la pregunta que motivo la feature ("¿donde decidimos usar ureq?")
  tiene que devolver el ADR primero.
- **Demostrar el SLO**: medir la corrida sobre el corpus real y publicar el numero.
- `templates/` y raiz espejados; espejos de roles regenerados.
- El hito 4 del PRD `aprendizaje` queda marcado por el cierre, con declaracion de
  leccion (`require_leccion` activa).

## Riesgos

- **Que el ranking sea peor que `grep`.** Es el riesgo real de la feature: un
  orden malo es peor que ninguno, porque da falsa confianza. Mitigacion: el
  `score` va en `--json` (auditable), el peso de cada fuente es explicito, y el
  criterio de cierre exige probarlo contra una consulta real conocida.
- **Falsos negativos por acentos.** Este repo escribe sin acentos por convencion,
  asi que no se hace plegado de acentos. Si algun dia entra texto acentuado, una
  busqueda podria fallar: queda documentado como limite conocido, no como bug.
- **Volumen de salida.** Mitigado por el tope de 20 + aviso explicito.
- **Tentacion de compartir codigo con `perfil::recolectar`.** Ver la seccion del
  grafo: se decide NO hacerlo, y queda escrito para que nadie lo "arregle" despues.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cinco del spec fueron decididas por Alan el 2026-08-17 y
estan en su seccion "Observaciones" y en el sello:

- OBS-1 el hub queda FUERA; solo archivos -> D1, D7.
- OBS-2 tope de 20 con aviso y `--todos` -> D6.
- OBS-3 caida a "algun termino", avisando -> D2.
- OBS-4 fecha: timestamp en `history.md`, mtime en el resto -> D5.
- OBS-5 se recorren los `estado-feature-*` -> D1.

## Skills aplicadas

- **`rust-patterns`**: `Fuente` como enum con orden explicito y matcheo
  exhaustivo (D3); cadenas de iteradores en vez de bucles manuales.
- **`rust-best-practices`**: la regla "duplicacion vs abstraccion equivocada" es
  lo que decide NO compartir codigo con `perfil::recolectar`; `&str` en los
  parametros; `#[expect(...)]` con motivo si hace falta.
- **`rust-testing`**: helpers de fixture documentados en `mod tests`;
  table-driven en vez de `rstest` (seria dependencia nueva, Articulo 6).
- **`rust-async-patterns`**: no aplica — `buscar` es I/O de archivos sincrono y el
  binario no tiene runtime async.

### Avance 2026-08-17T03:45:54Z
Plan de la #20 escrito: D1-D9 citando cada AC, impacto (el hub es irrelevante por diseno tras OBS-1), medicion real del corpus (113 archivos, 28.391 lineas, 1,1 MB) que justifica no tener indice, y la decision explicita de NO compartir codigo con perfil::recolectar. Las 5 observaciones quedaron decididas por Alan.

### Avance 2026-08-17T03:54:17Z
D1-D9 implementados: modulo buscar.rs (Fuente como enum ordenado por relevancia, score puro y testeable, corpus que excluye bkp), comando buscar con --json y --todos, docs/superficies/roles y 30 tests nuevos. El criterio de cierre de la consulta real FALLO en la primera corrida y destapo dos bugs de clasificacion (la guia de lecciones cobraba peso de conocimiento curado; el ADR pesaba como doc generico); ambos corregidos y con test.

---
Cerrado: 2026-08-17T03:54:24Z - status=done - buscar: hace preguntable la memoria del arnes. Recorre docs/**/*.md + history.md y ordena de lo mas curado (lecciones, perfil) a lo mas crudo (bitacora), con encabezados, frases contiguas y frescura como desempate. Sin indice (~10 ms medidos sobre 1,1 MB), sin LLM, sin hub, sin dependencias nuevas y de solo lectura. 19 AC cubiertos; el criterio de la consulta real fallo primero y corrigio dos bugs de ranking.
