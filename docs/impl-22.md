# Evidencia de implementacion - Feature #22: mapa_de_aprendizaje

Spec: `docs/spec-feature-22-mapa-de-aprendizaje.md` (`Estado: approved`, 18 AC,
sello 2026-08-17T04:38:04Z)
Plan: `docs/plan-feature-22-mapa-de-aprendizaje.md` (D1-D8)
PRD: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 6, ultimo)

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/journey.rs` | D1, D2, D3 | NUEVO. `Tipo`/`Clase`/`Motivo` como enums, `construir()`, `Mapa::hijos()`; 18 tests |
| `rust/src/commands/journey.rs` | D4, D5 | NUEVO. Render cronologico y `--json` |
| `rust/src/cli.rs`, `main.rs`, `commands/mod.rs` | D4 | Cableado |
| `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md` | D7 | El mapa, los huecos y por que es solo lectura |
| `setup_harness.sh` / `.ps1` | D7 | Superficies |
| `rust/tests/cli_basics.rs` | D8 | 8 tests de integracion |

## Los criterios de cierre: tres bugs que solo aparecieron con datos reales

El plan exigia correr el mapa sobre **este** repo y verificar a mano. Los tests
con fixtures pasaban desde el principio; los tres hallazgos salieron de la
corrida real.

### 1. La leccion declarada salia duplicada

La #17 declaro `docs-generados-por-el-instalador` al cerrar, y esa misma leccion
la cita como `origen`. El mapa la mostraba **dos veces** bajo la misma feature:

```
  #17 lecciones_memoria_procedural
      `-- [leccion declarada] docs-generados-por-el-instalador
      `-- [leccion (origen)] docs-generados-por-el-instalador   <-- duplicada
```

Arreglo: `Clase::prioridad()` (declarada > origen > cita > relacionada) y dedup
por nodo destino en `Mapa::hijos()`. Test:
`hijos_should_not_show_the_same_lesson_twice`.

### 2. Una entrada del perfil colgaba de TODAS las features que cita

`perfil:1` cita `(#14, #16)` y aparecia bajo las dos, repitiendo el mismo texto.
El OBS-5 dice que se ubica en la **mas reciente**. Arreglo: `hijos()` saltea un
nodo de perfil cuya fecha no coincide con la del padre. Test:
`hijos_should_anchor_a_profile_entry_to_its_most_recent_feature`.

### 3. Los huecos: 16 -> 2 -> 0

La primera corrida reporto **16 huecos**, todos `cierre-sin-leccion`, todos de las
features #1 a #16 — que cerraron **antes de que existiera la maquinaria de
lecciones** (la creo la #17). Ninguno era corregible. Es exactamente el riesgo que
el plan anoto: *"un mapa que grita por cosas que estan bien se ignora en dos
dias"*.

Arreglo 1: solo es hueco una feature cerrada **despues** de que el proyecto
empezo a declarar lecciones (la fecha de cierre mas temprana entre las que
declararon). Bajo a **2**.

Los 2 restantes eran la #15 y la #16, que cerraron el **mismo dia** que la #17
pero horas antes (04:16 y 05:36 vs 20:00): comparaba fechas truncadas.

Arreglo 2: comparar **timestamps completos**. Bajo a **0**.

Verificacion a mano, que es lo que el criterio de cierre pedia:

```
$ python3 -c "features done con id>=17 y sin leccion"
  (sin salida = ninguno, coincide con el mapa)

$ sh harness_cli journey | tail -1
[Ok] Sin huecos: los tres almacenes son coherentes entre si.
```

Los dos arreglos con test: `construir_should_not_report_closes_from_before_the_machinery_existed`
y `construir_should_use_full_timestamps_not_just_dates`.

## Evidencia por AC

### AC-1 / AC-2 — Linea de tiempo y los dos tipos de enlace a leccion

Corrida sobre el repo real:

```
2026-08-16
  #17 lecciones_memoria_procedural
      `-- [leccion declarada] docs-generados-por-el-instalador
          Sumar un doc al arnes es una linea en HARNESS_DOCS... — 1 uso(s), ultimo 2026-08-17
      `-- [leccion (origen)] hitos-del-prd
          La celda del slug se compara literal: sin backticks no se marca. — nunca usada
```

Las dos lecciones de la #17, distinguidas. `journey_should_show_both_lessons_of_a_feature_without_duplicating`
verifica ambas cosas: que estan las dos y que la declarada sale una sola vez.

### AC-3 — Usos visibles

Cada nodo de leccion dice `N uso(s), ultimo <fecha>` o `nunca usada`: es lo que
distingue lo vivo de lo que solo esta escrito.

### AC-4 — Las entradas del perfil, ancladas

```
  #19 perfil_de_usuario
      `-- [perfil] perfil:3
          Ante un gate, prefiere bloquear a avisar... (#17, #19) — cita: #17, #19
```

Cuelga de la #19 (la mas reciente que cita), una sola vez. Verificado en
integracion comparando posiciones en el texto.

### AC-5 — Archivadas aparte

`Tipo::LeccionArchivada` con su etiqueta propia; no se mezclan con las activas.

### AC-6..AC-10 — Los huecos

Un test por tipo: `construir_should_report_a_broken_link_from_a_lesson`,
`..._from_the_profile`, `..._a_close_without_a_lesson`, `..._an_orphan_lesson`,
`..._an_unreadable_lesson`, y `construir_should_find_no_gaps_in_a_coherent_repo`
para el caso sin huecos, que se dice explicitamente en vez de callar.

### AC-11 / AC-12 — Solo lectura, y una sola puerta

`journey.rs` y `commands/journey.rs` **no importan nada que escriba**: la promesa
es estructural, no una regla que recordar. Comprobacion negativa en integracion
(`journey_should_write_nothing_and_ignore_the_hub`) y en la corrida real:

```
$ sh harness_cli journey >/dev/null && find docs progress -newermt '-5 seconds' -type f | wc -l
0
```

Por cada hueco se imprime el comando del almacen que corresponde, y el mapa lo
dice explicitamente:

```
  journey no escribe nada: cada correccion pasa por el comando de su almacen.
```

`Motivo::remedio()` es un `match` exhaustivo: agregar un tipo de hueco **obliga**
a darle su comando (`motivo_should_offer_a_command_for_every_gap`).

### AC-13 — `--json`

`nodos` / `enlaces` / `huecos`, cada hueco con su `remedio`. Verificado que las
tres clases de enlace (`declarada`, `origen`, `cita`) aparecen.

### AC-14 / AC-15 / AC-16 — Degradacion

Mismo stdout con el hub sano y con el hub apuntando a un puerto muerto. Repo
fresco: mensaje + exit 0. Archivo ilegible: se saltea y se cuenta como hueco.

### AC-17 / AC-18 — Docs y verificacion

README (con el ejemplo real y el por que de solo lectura), UPDATING (+ espejo),
`architecture.md` (incluida la regla de los timestamps) y ambas superficies.

```
$ (cd rust && cargo test --locked)
test result: ok. 231 passed; 0 failed   (unitarios, +18)
test result: ok.  91 passed; 0 failed   (integracion, +8)

$ (cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings)
Finished
```

## Decisiones aplicadas

| OBS | Decision | Donde vive |
| --- | --- | --- |
| OBS-1 | Solo archivos, sin hub ni graphify | `construir()` no importa `graph` |
| OBS-2 | Sin `journey delete` | no existe; `Motivo::remedio()` apunta al comando del almacen |
| OBS-3 | Sin `journey edit` | idem |
| OBS-4 | Las features sin leccion aparecen y son hueco | con el matiz de la prehistoria (ver arriba) |
| OBS-5 | El perfil se ancla a la feature mas reciente | `Mapa::hijos()` |

## Skills aplicadas

- **`rust-patterns`**: cuarto uso del patron ya consolidado — `Tipo`, `Clase` y
  `Motivo` como enums. Aca rindio doble: `Motivo::remedio()` es exhaustivo, asi
  que un tipo de hueco nuevo no puede quedar sin su comando de correccion.
- **`rust-best-practices`**: `construir()` solo lee y el render vive aparte, que
  es lo que hace estructural la promesa de solo lectura (leccion
  `promesas-estructurales-vs-disciplina`, aplicada a la feature siguiente de
  donde nacio).
- **`rust-testing`**: un test por tipo de enlace y uno por tipo de hueco, en vez
  de un test grande que los mezcle. Y los tres bugs reales quedaron cada uno con
  su test, con el hallazgo documentado adentro.

## Riesgos pendientes para el reviewer

1. **El mapa no tiene tope.** Con 21 features entra en pantalla; con 200 no. No
   se puso limite porque no hay evidencia de que haga falta; si molesta,
   `--desde <fecha>` es la salida natural.
2. **La regla de la "prehistoria" tiene un supuesto.** Asume que el proyecto
   empezo a usar lecciones en la primera que declaro una. Si alguien declara una
   leccion retroactivamente en una feature vieja, la ventana se corre hacia atras
   y volverian a aparecer huecos viejos. Es un caso raro y visible, no silencioso.
3. **`setup_smoke.ps1` sin ejecutar** (igual que #17-#21).
