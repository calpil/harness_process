# Evidencia de implementacion - Feature #20: buscar_en_el_historial

Spec: `docs/spec-feature-20-buscar-en-el-historial.md` (`Estado: approved`, 19 AC,
sello 2026-08-17T03:44:47Z)
Plan: `docs/plan-feature-20-buscar-en-el-historial.md` (D1-D9)
PRD: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 4)

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/buscar.rs` | D1, D2, D3, D5 | NUEVO. `Fuente`, `score`, `corpus`, `buscar`; 21 tests |
| `rust/src/commands/buscar.rs` | D4, D6 | NUEVO. Salida humana y `--json` |
| `rust/src/cli.rs`, `main.rs`, `commands/mod.rs` | D4 | Cableado |
| `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md` | D8 | El comando, el orden del ranking y las tres garantias |
| `setup_harness.sh` / `.ps1` | D8 | Superficies: "preguntale al repo antes de leerlo entero" |
| `templates/roles/*.md` (+ `roles/` y espejos) | D8 | Lider (paso 4.0), implementer (1.5) y reviewer (citas verificables) |
| `rust/tests/cli_basics.rs` | D9 | 9 tests de integracion |

## Evidencia por AC

### AC-1 — Que se recorre

`corpus()` recorre `docs/**/*.md` recursivo (lecciones, PRDs, ADRs, specs,
planes, impl, review, `estado-feature-*` por OBS-5) mas `progress/history.md`, y
excluye `bkp/`, `.git`, `node_modules`, `target` y todo directorio oculto.
Tests: `corpus_should_skip_backups_and_hidden_dirs` (unit) y
`buscar_should_skip_backup_directories` (integracion, con un `bkp/viejo.md` real
que menciona el termino y NO aparece).

### AC-2 — AND, con caida a "alguno" avisada

`buscar_should_require_all_terms`: dos lineas, una con ambos terminos y otra con
uno; devuelve **una**. `buscar_should_fall_back_to_any_term_and_flag_it`: sin
coincidencia completa, cae a parcial y marca `parcial = true`. En integracion,
el comando lo dice:

```
[i] Ninguna linea tiene TODOS los terminos: se muestran las que tienen alguno.
```

### AC-3 — Consulta vacia

```
$ sh harness_cli buscar "   "
Falta la consulta.
    Uso: sh harness_cli buscar "<terminos>" [--json] [--todos]
    Ejemplo: sh harness_cli buscar "ureq adr"
exit 2
```

### AC-4 / AC-5 / AC-6 — El ranking

`Fuente` es un enum cuyo **orden de variantes es el orden de relevancia**, con
pesos de saltos grandes:

```
Leccion 100 · Perfil 95 · Spec 80 · Adr 78 · Plan 75 · Prd 70
Impl 55 · Review 50 · Estado 45 · Doc 40 · Historia 20
```

Tests unitarios, uno por regla: `score_should_reward_headings`,
`score_should_reward_frontmatter_fields`,
`score_should_reward_a_contiguous_phrase`, `score_should_prefer_recent_features`
y —el que protege la propiedad importante—
`score_freshness_should_never_beat_the_source_weight`: una leccion vieja le sigue
ganando a una bitacora nueva. En integracion,
`buscar_should_rank_curated_knowledge_first` verifica el orden
leccion > adr > impl sobre un corpus sembrado.

### AC-7 — `--json` auditable

```json
{ "archivo": "...", "linea": 1, "feature": "", "fecha": "2026-08-16",
  "fuente": "adr", "texto": "# ADR-0001: cliente HTTP `ureq` ...", "score": 108 }
```

El `score` sale en la salida justamente para que el orden no sea una caja negra.

### AC-8 — La salida humana

`docs/adr/ADR-0001-cliente-http-ureq.md:1  [adr 2026-08-16]` + el texto recortado.
Ruta relativa a la raiz y con `/` (clickeable igual en Windows). La fecha sale del
timestamp de la linea en `history.md` y del mtime en el resto (OBS-4).

### AC-9 — Nunca trunca en silencio

`buscar_should_never_truncate_silently`: 30 resultados => imprime 20 y

```
  ... 10 resultado(s) mas. Vealos con --todos.
```

Con `--todos` ese aviso desaparece.

### AC-10 / AC-11 — Sin resultados

`Sin coincidencias para 'X' en N archivo(s).` + sugerencia de usar menos
terminos, **exit 0**. Con `--json`, `resultados: []` y `total: 0`: JSON valido
igual, para que un script no maneje dos formatos.

### AC-12 — Milisegundos y sin indice

Medicion sobre el corpus real de este repo (114 archivos, ~28.400 lineas, 1,1 MB),
cinco corridas:

```
real 0.01 · real 0.01 · real 0.01 · real 0.00 · real 0.00
```

**~10 ms**, la mayor parte arranque de proceso. No se crea ningun archivo de
indice (verificado en test: `docs/.buscar-index` no existe tras buscar).

### AC-13 / AC-14 — Sin dependencias, sin hub

`git diff` sobre `rust/Cargo.toml` y `Cargo.lock`: vacio.
`buscar_should_write_nothing_and_ignore_the_hub` corre la misma consulta con el
hub sano y con `DB_HOST=127.0.0.1 DB_PORT=1`, y compara **el stdout completo**:
identico. Ningun `use` de `graph` en los dos modulos nuevos.

### AC-15 / AC-16 — Degradacion

Un archivo ilegible se saltea (`let Ok(contenido) = read_to_string else
{ continue }`). Sin `docs/`: `No hay corpus que buscar todavia` y exit 0
(`buscar_should_say_so_without_a_corpus`).

### AC-17 / AC-18 — Docs y roles

README (seccion con el diagrama del orden y las tres garantias), UPDATING
(+ espejo), `architecture.md` (el modulo, con el porque de los saltos de peso),
las superficies de ambos instaladores, y los tres roles: lider paso 4.0
("preguntale al repo antes de leerlo entero"), implementer paso 1.5 ("antes de
reconstruir algo, buscalo") y reviewer ("una cita que no aparece en ningun
artefacto es una cita inventada").

### AC-19 — Verificacion oficial

```
$ (cd rust && cargo test --locked)
test result: ok. 194 passed; 0 failed   (unitarios, +21)
test result: ok.  73 passed; 0 failed   (integracion, +9)

$ (cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)
Finished
```

`tests/setup_smoke.sh` y `harness_check.sh`: ver la seccion de cierre.

## La prueba que motivo la feature

El criterio de cierre del plan exigia que "¿donde decidimos usar ureq?" devolviera
el ADR primero. **La primera corrida fallo**, y eso descubrio dos bugs reales de
clasificacion:

```
puesto 1: docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md:52   | `arreglo-ureq` | ...
puesto 6: docs/spec-feature-15-...md:347  OBS-9 ... el ejecutor REST usa `ureq`
puesto 10: docs/adr/ADR-0001-cliente-http-ureq.md          score=70 fuente=doc
```

1. **La guia de lecciones se clasificaba como `Leccion`** y cobraba el peso del
   conocimiento curado (100). Pero es una **plantilla del arnes**, y su
   `arreglo-ureq` es un EJEMPLO de nombre malo, no una decision. Ahora se excluye
   igual que ya hace `lecciones::scan`.
2. **El ADR se clasificaba como `Doc`** (peso 40). Un ADR es una decision tecnica
   con nombre propio y sin fecha de vencimiento: merece peso de decision (78).

Con las dos correcciones:

```
$ sh harness_cli buscar ureq
24 resultado(s) en 114 archivo(s):

  docs/adr/ADR-0001-cliente-http-ureq.md:1  [adr 2026-08-16]
    # ADR-0001: cliente HTTP `ureq` para el ejecutor REST de Atlassian
```

Ambas quedaron con test: `fuente_should_be_derived_from_the_path` incluye la guia
como `Doc`, y `adr_should_rank_as_a_decision_not_as_a_generic_doc` documenta el
hallazgo en el propio test.

## Que no escribe nada

```
$ sh harness_cli buscar "gate leccion"
$ find docs progress -newermt '-5 seconds' -type f | wc -l
0
```

## Skills aplicadas

- **`rust-patterns`**: `Fuente` como enum con orden semantico y `match`
  exhaustivo en `peso()`/`etiqueta()`; cadenas de iteradores en `corpus` y en el
  filtrado de terminos.
- **`rust-best-practices`**: la regla "duplicacion vs abstraccion equivocada" es
  lo que sostiene la decision de **NO** compartir codigo con
  `perfil::recolectar` (filtran cosas distintas y van a divergir); `&str` en los
  parametros; sin `unwrap` fuera de tests.
- **`rust-testing`**: helper `sandbox(&[(ruta, contenido)])` documentado, y
  table-driven (`fuente_should_be_derived_from_the_path` cubre 12 rutas en un
  `for`) en vez de `rstest`, que seria dependencia nueva.
- **`rust-async-patterns`**: no aplica; `buscar` es I/O sincrono de archivos.

## Riesgos pendientes para el reviewer

1. **El ranking es heuristico.** Se probo con una consulta real conocida y con
   tests por regla, pero pesos distintos darian ordenes distintos. Mitigacion: el
   `score` es auditable en `--json` y los pesos estan en un solo lugar.
2. **Sin plegado de acentos.** Este repo escribe sin acentos por convencion; si
   entrara texto acentuado, `buscar "decision"` no encontraria "decisión". Es un
   limite conocido y declarado, no un bug silencioso.
3. **`setup_smoke.ps1` sin ejecutar** (igual que #17, #18 y #19). Esta feature no
   agrego aserciones ahi: `buscar` no toca el instalador mas alla de la
   superficie.
