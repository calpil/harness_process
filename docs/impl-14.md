# Impl - Feature #14: hub_batch_upserts_atomic_install

Spec: `docs/spec-feature-14-hub-batch-upserts-atomic-install.md` (Estado:
approved, AC-1..AC-14)
Plan: `docs/plan-feature-14-hub-batch-upserts-atomic-install.md` (D1..D9)

## Que se construyo

Dos costos que se pagaban todos los dias dejaron de existir.

**El hub.** `PgGraphStore::save()` emitia UNA sentencia por fila sobre el grafo
ENTERO en cada comando. En el hub de referencia (Aiven, 164 ms de round-trip)
eso son 1047 nodos + 1641 aristas = 2688 ida-y-vuelta, medidos en **456,31 s**.
Ahora escribe solo lo que el comando toco y en lotes de `UNNEST`:

```
comando que toca k nodos y j aristas

  antes:  N + M sentencias     (todo el grafo, una fila por sentencia)
  ahora:  ceil(k/1000) + ceil(j/1000) sentencias   (cero si no toco nada)
```

El registro de lo sucio (`dirty_nodes` / `dirty_edges`) lo llevan `add_node` y
`add_edge`, `load()` lo vacia y `save()` lo consume. Escribir solo el delta es
ademas lo que vuelve seguro el candado **por proyecto** (`<hub>/.lock-<proyecto>`
en vez de un `<hub>/.lock` unico para toda la maquina): ningun proyecto
reescribe filas de otro.

**El instalador.** Los dos instaladores copiaban el binario nuevo encima del
binario vivo, reescribiendo el mismo inode. En macOS eso invalida la firma
cacheada del Mach-O y la corrida siguiente muere con SIGKILL. Ahora escriben un
temporal HERMANO del destino y lo mueven con rename atomico.

## Evidencia por AC

- **AC-1** (nodos por lote): `rust/src/graph/store.rs::save()` emite
  `INSERT INTO graph_nodes (id, label, props) SELECT * FROM UNNEST($1::text[],
  $2::text[], $3::jsonb[]) ON CONFLICT (id) DO UPDATE SET label =
  EXCLUDED.label, props = graph_nodes.props || EXCLUDED.props` por cada
  `chunk(UPSERT_CHUNK = 1000)` de `node_rows(...)`. El upsert es literalmente el
  mismo de antes; lo que cambia es cuantas filas viajan por sentencia.

- **AC-2** (aristas por lote): mismo `save()`, con
  `UNNEST($1::text[], $2::text[], $3::text[], $4::jsonb[])` y
  `ON CONFLICT (source, target, type) DO UPDATE SET props =
  COALESCE(graph_edges.props, '{}'::jsonb) || EXCLUDED.props`.

- **AC-3** (aristas repetidas se fusionan): `edge_rows()` agrupa por
  `(source, target, type)` en un `IndexMap` y fusiona props clave a clave con la
  ultima ganando, que es exactamente lo que encadenaban los INSERT secuenciales
  (`existing || P1`, despues `|| P2`). Sin esto, Postgres rechazaria el lote con
  *"ON CONFLICT DO UPDATE command cannot affect row a second time"*. Test:
  `edge_rows_fusiona_misma_clave` (dos aristas `a->b DEPENDE_DE` con `origen`
  distinto colapsan en una fila con `{"origen":"graphify","peso":"1"}`).

- **AC-4** (solo el delta): `dirty_nodes`/`dirty_edges` en `store.rs`;
  `add_node` marca el id, `add_edge` marca la clave (solo cuando realmente
  agrega, respetando el dedup por dict completo que ya existia), `load()`
  vacia ambos conjuntos (lo que viene de la base ya esta en la base) y `save()`
  retorna sin abrir conexion ni transaccion si no hay nada sucio. La unica
  mutacion directa sobre `store.nodes` del codebase (`commands.rs::unmark`)
  llama a `mark_node_dirty` para que lo persistido sea identico a lo de antes.
  Tests: `node_rows_solo_lleva_lo_sucio`, `edge_rows_solo_lleva_lo_sucio`,
  `node_rows_vacio_sin_sucios`, `node_rows_ignora_sucio_inexistente`,
  `edge_rows_conserva_orden`.

- **AC-5** (medicion real): mismo enlace, misma carga (1047 nodos + 1641
  aristas), tablas temporales para no tocar datos del hub:

  | Forma de escribir | Sentencias | Tiempo |
  | --- | --- | --- |
  | Una por fila (lo de antes) | 2688 | **456,31 s** |
  | Lotes de `UNNEST` (lo de ahora) | 4 | **2,74 s** |

  166x. Round-trip del enlace medido por pendiente: 30 consultas = 6,21 s vs
  130 consultas = 22,62 s -> **164 ms** por ida-y-vuelta, 1,29 s de conexion.
  End-to-end del binario ya instalado, con `graph registrar` (load + save del
  delta): **11,33 s**, de los cuales ~3,9 s son el `load()` completo
  (`graph mapa`, que no escribe, mide 3,86 s). Con el delta, un `sync_git`
  tipico escribe 2 sentencias.

- **AC-6** (una transaccion, sin migracion): `save()` sigue abriendo
  `client.transaction()` y haciendo `commit()` una sola vez, con todos los lotes
  adentro. `init_db()` no cambio: mismas dos tablas, mismas claves. `git diff`
  de `rust/Cargo.toml` vacio (sin dependencias nuevas).

- **AC-7** (tests sin base de datos): `node_rows` y `edge_rows` son funciones
  libres y puras (entrada: el grafo en memoria + el conjunto de sucios; salida:
  las filas del lote), asi que el modulo de tests las ejercita sin construir un
  `PgGraphStore` (que conectaria). 6 tests nuevos en `graph::store::tests`.

- **AC-8** (candado por proyecto): `GraphEnv::resolve` arma
  `hub_dir.join(format!(".lock-{}", lock_slug(&project)))`; `lock_slug` reduce
  el nombre a `[A-Za-z0-9._-]` (el proyecto puede venir de `HARNESS_PROJECT` con
  cualquier cosa) y cae en `proyecto` si queda vacio. Verificado en vivo:
  tras correr un comando del hub aparecio
  `~/.harness-hub/.lock-harness_process` junto al `.lock` viejo, y el comando
  NO espero a los procesos de otros proyectos que tenian tomado el `.lock`
  global.

- **AC-9** (timeout de sentencia): `GraphMemoryManager::new` lee
  `DB_STATEMENT_TIMEOUT` con el mismo `lookup` que el resto de las `DB_*`
  (entorno o `<hub>/.env`), lo valida como numero (`Exit` con mensaje
  accionable si no lo es), y `PgGraphStore::new` lo pasa como
  `-c statement_timeout=<ms>` mas `keepalives(true)` /
  `keepalives_idle(30s)`. Default `DEFAULT_STATEMENT_TIMEOUT_MS = 30000`, `0`
  desactiva. **Verificado en vivo**: con un binario VIEJO de otro proyecto
  manteniendo abierta su transaccion (la que reescribe la tabla entera) y por lo
  tanto los locks de fila sobre los nodos `Agente` compartidos, el comando nuevo
  corta a los 30 s con `db error: ERROR: canceling statement due to statement
  timeout` y exit 1, en vez de esperar indefinidamente como antes. En
  `hub_register` (best-effort) el comando local igual completa.

- **AC-10** (sh: mv atomico): `setup_harness.sh::install_binary_atomic` copia a
  `<dir>/.<binario>.new.$$`, hace `chmod +x` sobre el temporal y `mv -f` al
  destino; la rama de compilacion lo usa en lugar del `cp` + `chmod` previos.
  Verificado a mano al instalar el binario de esta feature: inode `21581471` ->
  `55513486`, sin temporales residuales y `sh harness_cli status` respondiendo
  (sin SIGKILL). Verificado en automatico por el smoke (AC-13).

- **AC-11** (fallo limpio): `install_binary_atomic` valida que exista el binario
  compilado (error accionable + `return 1` si no), borra el temporal ante
  cualquier fallo de `cp`/`chmod`/`mv` y devuelve error, de modo que el
  instalador entra en la rama de error que ya existia (`log_error` + `exit 1`) y
  el binario previo queda intacto.

- **AC-12** (ps1): `setup_harness.ps1::Install-BinaryAtomic` hace
  `Copy-Item` a `.<leaf>.new.<PID>` y `Move-Item -Force` al destino; si el
  destino esta bloqueado por un `harness.exe` en ejecucion, aparta el destino a
  `.<leaf>.old.<PID>` (mover el archivo en uso SI esta permitido en Windows),
  completa la instalacion y borra el apartado. Ante cualquier excepcion borra el
  temporal, loguea el error y devuelve `$false`, y `Build-HarnessBinary` no
  cuenta la instalacion. El bloque `-DryRun` retorna antes de tocar nada
  (sin cambios). No ejecutable en esta maquina (sin PowerShell), como en las
  features #1 y #13: cubierto por assert de contenido en el smoke ps1.

- **AC-13** (smoke): `tests/setup_smoke.sh` -- bloque del binario Rust: se
  extrajo `run_rust_setup()` y se instala DOS veces sobre el mismo directorio,
  comparando el inode antes/despues (`[!]` explicito si no cambio: "vuelve el
  SIGKILL de macOS"), verificando que no queden `.harness*.new.*` y que el
  binario reinstalado responda. Salida: `[Ok] re-instalacion atomica del
  binario: inode nuevo, sin temporales, y el binario responde.`
  `tests/setup_smoke.ps1` -- paridad: segunda corrida con un cargo falso que
  emite `fake harness v2`, assert de que `harness.exe` quedo con el contenido
  nuevo y de que no hay `.harness.exe.*` colgados.

- **AC-14** (verificacion oficial): ver la tabla de abajo.

## Verificacion

| Comando | Resultado |
| --- | --- |
| `cargo test` | 70 passed (unit, +6 nuevos) + 27 passed (cli_basics), 0 failed |
| `cargo clippy --all-targets -- -D warnings` | sin hallazgos |
| `bash tests/setup_smoke.sh` | exit 0, con `[Ok] re-instalacion atomica del binario: inode nuevo, sin temporales, y el binario responde.` |
| `bash harness_check.sh` | `[Ok] Harness Check limpio.` |
| `sh -n setup_harness.sh` | sintaxis OK |
| `git diff --stat -- rust/Cargo.toml` | vacio (Articulo 6: sin dependencias nuevas) |

## Notas de transicion (importante)

Mientras un proyecto conserve el binario VIEJO, ese binario sigue tomando el
`.lock` global (que el nuevo ya no mira) y sigue reescribiendo el grafo entero
dentro de una transaccion larga. Durante esa ventana, las escrituras nuevas
pueden chocar con sus locks de fila sobre los nodos `Agente` compartidos y
cortar por `DB_STATEMENT_TIMEOUT` (fallan legible, no cuelgan). El remedio es
re-correr el instalador en TODOS los proyectos, y esta documentado en
`UPDATING.md` y en su espejo `templates/UPDATING.md`.
