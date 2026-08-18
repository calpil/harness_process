# Plan - Feature #28: consolidacion_de_lecciones_con_llm

Estado: in_progress
Microservicios:
- harness

## Alcance

Ultimo hito del PRD de aprendizaje, y la unica parte del arnes que necesita un
modelo. Sale de la OBS-1 de la #21, que la aparto por ser "la unica parte que
necesita modelo, apagada por default, no verificable aqui".

Ahora **si** es verificable: `claude -p` y `kimi -p` funcionan no interactivos en
esta maquina, y el corpus real tiene exactamente un solapamiento genuino.

Spec aprobado (27 AC, cada uno con su `Comando:`):
`docs/spec-feature-28-consolidacion-de-lecciones-con-llm.md`.

## Peldano elegido: 3 (comando nuevo, dentro del grupo `lecciones`)

| Peldano | ¿Alcanzaba? |
| --- | --- |
| 1. extender lo que existe | **NO**. La deteccion necesita un backend externo, un parser tolerante y una cadena de resolucion; meterla dentro de `lecciones curar` mezclaria una pasada determinista con una que depende de un modelo, y `curar` dejaria de poder correrse sin red |
| 2. flag en un comando existente | **NO**, por lo mismo: `curar --con-llm` haria que el mismo comando tenga dos naturalezas y dos modos de fallar |
| **3. comando nuevo** | **SI, elegido**, y se monta DENTRO del grupo `lecciones` que ya existe (`status`/`curar`/`pin`/`archivar`/`rollback`), asi que no suma verbo de nivel superior |
| 4. superficie nueva | no |
| 5. dependencia nueva | no: el LLM se invoca como proceso con `wait-timeout`, que ya es dependencia (Articulo 6 sin ADR) |

**Peldano elegido: 3 (comando nuevo dentro del grupo `lecciones`) porque la
deteccion depende de un backend externo y meterla en `curar` haria que una pasada
hoy determinista y sin red pase a tener dos naturalezas y dos modos de fallar.**

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> el hub
responde con `statement timeout`, como en las trece features anteriores.

- `rust/src/consolidacion.rs` (NUEVO): cadena de backend, prompt, parser
  tolerante, validacion de candidatos y requisitos del paraguas. **Puro** salvo
  el spawn.
- `rust/src/commands/leccion.rs` + `cli.rs`: el subcomando.
- `tests/consolidar_check.sh` (NUEVO): los AC que exigen backend real.
- Docs, guia de lecciones y espejos.

**El riesgo es de otra clase que en todas las features anteriores**: es la
primera vez que contenido de `docs/` sale del repo hacia un proceso externo, y la
primera vez que una salida de modelo influye sobre archivos del usuario. Por eso
las defensas son estructurales y no de prosa.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

El diseno salio de un workflow de 17 agentes (5 mapearon codigo y corpus, 3
disenaron, 9 refutaron). Lo que decidio el plan, todo medido:

- **El corpus real es casi ortogonal.** Jaccard sobre triggers: un solo par en
  0.400 (`docs-generados-por-el-instalador` + `documentos-del-usuario-vs-plantillas`),
  el siguiente en 0.050, y 36 de 45 pares con interseccion vacia. **Mi propia
  hipotesis previa era falsa**: yo habia apostado a tres pares.
- Ese par comparte el pitfall del `.ps1` casi literal, la misma regla y el mismo
  bloque de Verificacion; y la mas nueva **declara el solapamiento en su prosa**.
- **Consecuencia dura**: el umbral de confianza **no se puede calibrar** (nada en
  la zona gris). Por eso la confianza se reporta sin filtrar (OBS-3).
- `buscar.rs:57` puntua `Leccion` 100 y `buscar.rs:73` `LeccionArchivada` 30, y
  `lecciones.rs:314-335` (`scan`) no es recursivo: archivar **degrada** la
  recuperabilidad. De ahi el AC-17 (el paraguas hereda todos los triggers).
- La guia manda "patchea el paraguas existente", asi que el paraguas **puede ser
  una de las miembros** (AC-15). El diseno original lo prohibia y habria forzado
  justo el antipatron que la guia pone ultimo.

## Delegacion (implementer)

- **D1 (AC-1..AC-5)** — `consolidacion.rs`: `resolver_backend()` con la cadena
  **override -> CLI -> skip limpio**. Sin `rules.consolidar_backend` no se mira
  ni el entorno. El override elige cual, nunca enciende.
- **D2 (AC-6, AC-11)** — El parser: `extraer_json()` toma el primer objeto de
  llaves balanceadas. Fixtures con la salida **real** de los dos backends
  (`claude` devuelve JSON pelado; `kimi` lo envuelve). Basura -> informa y sale 0.
- **D3 (AC-7, AC-8)** — El recorte y el argv: se manda solo nombre, descripcion y
  triggers; el prompt viaja como **un item de argv**, nunca por `sh -c`.
- **D4 (AC-9, AC-10, AC-21)** — Validacion: miembros inexistentes descartados y
  dichos, grupos con `pinneada` descartados, confianza reportada sin filtrar.
- **D5 (AC-12, AC-13, AC-14)** — La simetria de `curar`: sin flag informa; con
  `--aplicar` la fusion se toma de **argv** (`--en/--de/--motivo`), no del modelo.
- **D6 (AC-15..AC-18)** — Los requisitos del paraguas: puede ser una miembro, no
  puede tener placeholders, hereda **todos** los triggers y cita `[[cada]]`.
- **D7 (AC-19, AC-20)** — Backup previo, archivado byte a byte y rollback,
  reusando `curador::respaldar`/`rollback`.
- **D8 (AC-22..AC-27)** — `tests/consolidar_check.sh` con backend real, la
  corrida contra el corpus real documentada, docs y verificacion oficial.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-27 en `docs/impl-28.md`; veredicto en `docs/review-28.md`.
- `sh harness_cli verify --feature 28` **verde**, con sus 27 comandos.
- **Corrida contra el corpus REAL con backend real**, documentada: que propuso el
  modelo, que se descarto y por que. Si el modelo no encuentra el unico par que
  existe, eso se escribe — no se ajusta el prompt hasta que salga lo que quiero.
- **El paraguas de la fusion real, mostrado a Alan y aprobado por el** antes de
  aplicar (OBS-2). Si le parece peor que las dos separadas, no se aplica y se
  dice.
- **Nunca borra, verificable byte a byte**: cuerpo de cada archivada identico al
  de antes, y backup presente.
- **El modelo no ve el cuerpo**: verificable inspeccionando el prompt que se
  arma.
- **Se puede deshacer**: `lecciones rollback` restaura.
- `cargo test`, clippy, `setup_smoke.sh`, `parity_check.sh`, `harness_check.sh`.

## Riesgos

- **Fusionar destruye matices.** El valor de una leccion esta en sus pitfalls y
  en los casos con numero de feature. Mitigado porque el modelo **no redacta**:
  solo senala el par, y la prosa la escribe una persona con el AC-17/AC-18 como
  piso verificable.
- **Contenido del repo saliendo a un proceso externo.** Mitigado por el recorte
  (nunca el cuerpo) y porque es un CLI local, no una API.
- **Inyeccion via el texto de una leccion.** Mitigado estructuralmente: argv, no
  shell.
- **No determinismo.** Dos corridas pueden dar dos propuestas. Aceptado: la
  deteccion solo informa, y lo que muta se toma de argv.
- **El umbral no se puede calibrar** con 9 lecciones. Declarado, no disimulado.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las tres del spec fueron decididas por Alan el 2026-08-18:

- OBS-1 el camino HTTP con API key **fuera de alcance**, nombrado en el skip.
- OBS-2 **se fusionan de verdad** las dos lecciones reales, con el paraguas
  mostrado y aprobado por separado.
- OBS-3 la confianza **se reporta sin filtrar**.

## Skills aplicadas

- **`rust-patterns`**: `detectar()` no recibe `&HarnessPaths` en su firma, asi
  que **no puede escribir aunque quiera**; `fusionar()` no recibe la propuesta
  del modelo. Las dos promesas son estructurales, no comentarios.
- **`rust-best-practices`**: se reusan `curador::respaldar`/`rollback`,
  `lecciones::scan` y `wait-timeout`; cero dependencias nuevas.
- **`rust-testing`**: la mitad que muta se prueba **sin backend** y de forma
  determinista; la del modelo, **sin mutar**. Ninguna queda bloqueada por la
  otra, que es justo la deuda que la OBS-1 de la #21 dejo abierta.

### Avance 2026-08-18
Plan de la #28 escrito: D1-D8 citando cada AC. El corpus real refuto mi hipotesis previa (yo esperaba tres pares solapados; hay uno solo, y el segundo Jaccard esta un orden de magnitud abajo). Dos bloqueos de la refutacion cambiaron el diseno: el paraguas puede ser una de las miembros (es lo que la guia manda) y tiene que heredar todos los triggers, porque archivar degrada la recuperabilidad de 100 a 30 en `buscar`.

### Avance 2026-08-18T21:28:46Z
Feature #28 implementada: lecciones consolidar detecta solapamientos con un LLM (solo nombre, descripcion y triggers; NUNCA el cuerpo) e informa; --aplicar toma la fusion de argv y archiva con backup. Corrida real contra el corpus: el modelo encontro el par que el analisis lexico habia identificado (0.85) y propuso un segundo a 0.60 que el Jaccard daba en 0.048. Fusion real aplicada con el si de Alan: la biblioteca paso de 9 a 8 lecciones, el cuerpo archivado quedo byte a byte identico y install_asset sigue encontrando el paraguas.

---
Cerrado: 2026-08-18T22:18:57Z - status=done - Consolidacion de lecciones con LLM: el modelo ve solo nombre, descripcion y triggers (nunca el cuerpo), no puede escribir (detectar no recibe HarnessPaths) y el prompt va por argv, no por shell. La fusion la pide una persona con argv. Verificada de punta a punta con dos backends reales, y aplicada al corpus real: la biblioteca paso de 9 a 8 lecciones sin perder un solo pitfall.
