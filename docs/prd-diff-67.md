Aplicado: 2026-08-31T02:36:52Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #67: los_dos_parsers_del_review_no_se_contradicen

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 67`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:110 (spec `master`), docs/prd/PRD-master.md:114 (spec `disparador`) y 173 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/commands/revision.rs`, `rust/src/main.rs`, `rust/src/markdown.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica La tabla de hitos (seccion 10) llega hasta el #13 y desde el #14 los cierres van a la Bitacora, que la escribe el propio `close` — las #64 y #66 estan ahi, no en la tabla. La #67 no agrega ni cambia una capacidad del producto: arregla como se leen documentos que el arnes ya leia. Lo que si cambia a nivel proyecto va al SDD (decision tecnica) y a architecture.md (modulo nuevo), que son los dos bloques de abajo.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `deberia`), docs/prd/SDD-master.md:10 (spec `ningun`) y 252 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/commands/revision.rs`, `rust/src/main.rs`, `rust/src/markdown.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio

Antes:
## 4. Decisiones tecnicas

**Un gate solo verifica lo que puede ejecutar** (feature #46).

Despues:
## 4. Decisiones tecnicas

**Un solo parser por formato** (feature #67). Habia CUATRO parsers de bloques de
codigo markdown con TRES semanticas sobre los mismos documentos, y el costo no
era teorico: `verify` ejecutaba los `Comando:` escritos dentro de un bloque
`~~~` —ejecucion de shell salida de una seccion que el autor marco como
documentacion, el bug que la #23 cerro para backticks y seguia abierto para
tildes— y `revision --veredicto` borraba prosa del reviewer o dejaba dos sellos
contradictorios segun la paridad de fences ajenos citados. Tres decisiones:

- **Una sola implementacion, en `markdown.rs`.** Fences EMPAREJADOS: se recuerda
  cual abrio el bloque y solo lo cierra el mismo. Es la unica de las tres
  semanticas que coincide con como se renderiza el markdown de verdad.
- **Se devuelve la clasificacion completa, no una lista filtrada.** Cada
  consumidor necesita algo distinto de la MISMA respuesta —el gate quiere lo de
  afuera, el limpiador necesita todas las lineas para reescribir conservando los
  fences, el parseo de AC quiere todo lo que no sea contenido—. Un `Vec<&str>`
  compartido no alcanzaba, y eso fue exactamente lo que hizo que cada uno se
  escribiera el suyo.
- **La regla se hace cumplir sola.** `tests/conventions_check.sh` gana el modo
  `un-solo-parser`, que se pone rojo ante un quinto. La unica exencion es una
  implementacion vieja conservada para poder medir contra ella, declarada por
  linea con `PARSER-VIEJO-A-PROPOSITO` y NOMBRADA en la salida del check: eximir
  los tests enteros dejaria que un parser de verdad se esconda en uno, que es
  justo lo que paso con el cross-check de `verificacion.rs`.

**Un gate solo verifica lo que puede ejecutar** (feature #46).

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (spec `corpus`), docs/architecture.md:101 (spec `lectura`), docs/architecture.md:102 (módulo `lecciones`) y 486 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/commands/revision.rs`, `rust/src/main.rs`, `rust/src/markdown.rs` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio

Antes:
- `verificacion.rs`: AC ejecutables (feature #23). `parsear()` es **pura** —lee

Despues:
- `markdown.rs`: el UNICO parser de bloques de codigo del arnes (feature #67).
  `lineas_clasificadas()` marca cada linea como `Fuera` / `Fence` / `Dentro` con
  semantica de fences EMPAREJADOS (un `~~~` dentro de un bloque ``` es
  contenido, no un cierre), y `lineas_fuera_de_bloque()` es el filtro que quieren
  el gate del review y el parseo de AC. Sus consumidores son `verificacion.rs`
  (los AC del spec), `revision.rs` (el gate y el sello) y `commands/revision.rs`
  (el limpiador de `estampar`); `atlassian/markdown.rs` queda aparte porque
  CONSUME el bloque para convertirlo a ADF. Antes cada uno tenia el suyo, con
  tres semanticas distintas. `tests/conventions_check.sh un-solo-parser` impide
  que aparezca un quinto.
- `verificacion.rs`: AC ejecutables (feature #23). `parsear()` es **pura** —lee

