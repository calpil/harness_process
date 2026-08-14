# Review - Feature #14: hub_batch_upserts_atomic_install

Spec: `docs/spec-feature-14-hub-batch-upserts-atomic-install.md`
(approved 2026-08-14T03:43:37Z, con las decisiones OBS-1..OBS-3 del usuario)
Plan: `docs/plan-feature-14-hub-batch-upserts-atomic-install.md` (D1..D9)
Implementacion: `docs/impl-14.md`

**Veredicto: APROBADO para cierre.** Los 14 AC tienen evidencia verificable, la
verificacion oficial esta en verde, la constitution se cumple y la mejora esta
medida contra el hub real, no estimada.

## Verificacion re-ejecutada en esta revision

| Comando | Resultado |
| --- | --- |
| `cargo test --locked` | 70 passed (unit) + 27 passed (cli_basics), 0 failed |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | sin hallazgos |
| `bash tests/setup_smoke.sh` | exit 0, incluido `[Ok] re-instalacion atomica del binario: inode nuevo, sin temporales, y el binario responde.` |
| `bash harness_check.sh` | `[Ok] Harness Check limpio.` (plan fresco, spec `approved` fresco, espejos sin novedad) |
| `sh -n setup_harness.sh` / `bash -n tests/setup_smoke.sh` | sintaxis OK |
| `git diff --stat -- rust/Cargo.toml rust/Cargo.lock` | vacio: sin dependencias nuevas (Articulo 6) |
| `diff -q harness_check.sh templates/harness_check.sh` | identicos |
| `git diff --stat -- roles/ templates/roles/ .claude/` | sin salida: el gate de espejo no puede quedar stale |
| `sh harness_cli graph impacto --microservicio harness_process/harness` | ningun microservicio registrado depende de el |
| `graphify query "...save del store"` | confirma los llamadores de `save()` (`store.rs`, `PgGraphStore`, `.client()`) |

## Cobertura por AC

| AC | Estado | Como se verifico |
| --- | --- | --- |
| AC-1 nodos por lote (UNNEST, chunk 1000) | OK | `store.rs::save()`, `nodes.chunks(UPSERT_CHUNK)`; upsert identico al previo |
| AC-2 aristas por lote | OK | mismo `save()`, 4 arreglos paralelos + `ON CONFLICT (source,target,type)` |
| AC-3 fusion de aristas con misma clave | OK | `edge_rows()` + test `edge_rows_fusiona_misma_clave`; evita "cannot affect row a second time" |
| AC-4 solo el delta | OK | `dirty_nodes`/`dirty_edges`, `load()` los vacia, `save()` corta temprano si no hay nada; `unmark` llama `mark_node_dirty`; 5 tests |
| AC-5 medicion real | OK | 2688 sentencias = **456,31 s** vs 4 sentencias por lote = **2,74 s** sobre el mismo enlace; RTT 164 ms medido por pendiente; end-to-end `graph registrar` 11,33 s |
| AC-6 una transaccion, sin migracion | OK | `transaction()`/`commit()` unicos; `init_db()` intacto; Cargo.toml sin cambios |
| AC-7 tests sin base de datos | OK | `node_rows`/`edge_rows` son funciones libres puras; 6 tests nuevos que no construyen el store |
| AC-8 candado por proyecto | OK | `GraphEnv::resolve` + `lock_slug`; verificado en vivo: aparece `.lock-harness_process` y el comando no espera al `.lock` global tomado por otros proyectos |
| AC-9 timeout de sentencia | OK | `DB_STATEMENT_TIMEOUT` validado + `-c statement_timeout` + keepalives; verificado en vivo: corta a los 30 s con error legible y exit 1 en vez de colgar |
| AC-10 sh: mv atomico | OK | `install_binary_atomic`; inode 21581471 -> 55513486 en la instalacion real, sin temporales, binario responde |
| AC-11 fallo limpio | OK | valida el origen, borra el temporal ante cualquier fallo y devuelve error; el instalador conserva su `log_error` + `exit 1` |
| AC-12 ps1 | PARCIAL (documentado) | `Install-BinaryAtomic` con temporal + `Move-Item -Force` + aparte del `.exe` bloqueado; **no ejecutado**: no hay PowerShell en esta maquina. Cubierto por assert de contenido en el smoke ps1, igual que en las features #1 y #13 |
| AC-13 smoke de doble instalacion | OK | `tests/setup_smoke.sh` (inode + temporales) y paridad en `tests/setup_smoke.ps1` |
| AC-14 verificacion oficial limpia | OK | tabla de arriba |

## Constitution

- **Articulo 1**: tests nuevos junto al codigo tocado (6 unitarios) y los cuatro
  comandos oficiales en verde.
- **Articulo 2**: spec `approved` ANTES de implementar, con el si explicito del
  usuario registrado por `approve-spec --yes` (sello del 2026-08-14T03:43:37Z).
- **Articulo 3**: cada item D1..D9 del plan cita su AC-n; `impl-14.md` y este
  veredicto mapean AC-1..AC-14.
- **Articulo 4**: sin secretos nuevos; props siguen viajando como parametros
  ligados (`$1..$4`), nunca interpolados; `DB_STATEMENT_TIMEOUT` validado antes
  de armar las opciones; exit codes estables (el instalador conserva su `exit 1`,
  el comando del hub falla con 1 y mensaje legible).
- **Articulo 5**: las tres decisiones del usuario (UNNEST, paridad sh/ps1,
  sumar timeout + candado por proyecto con la variante segura del delta) estan
  registradas en el spec (OBS-1..OBS-3), en el plan y en el sello de aprobacion.
  No se implemento nada con decisiones abiertas: OBS-4 queda REGISTRADA sin
  accion y declarada fuera de alcance.
- **Articulo 6**: sin dependencias nuevas; `UPDATING.md` propagado a
  `templates/UPDATING.md`; commit Conventional sin trailers de IA.

## Reparos / observaciones del reviewer

1. **AC-12 no se ejecuto** (sin PowerShell en la maquina). Es el mismo limite
   aceptado en las features #1 y #13. La logica esta cubierta por lectura y por
   el assert del smoke ps1, pero la primera corrida real en Windows deberia
   confirmar la rama del `.exe` bloqueado.
2. **Ventana de transicion**: hasta que se re-corra el instalador en TODOS los
   proyectos, los binarios viejos siguen tomando el `.lock` global y reescriben
   el grafo entero en transacciones largas; las escrituras nuevas pueden cortar
   por `DB_STATEMENT_TIMEOUT` mientras tanto (fallan legible, no cuelgan). Esto
   se observo en vivo durante la implementacion y quedo documentado en
   `UPDATING.md` / `templates/UPDATING.md` y en `docs/impl-14.md`.
3. **OBS-4 sigue abierta** como bug preexistente: `desmarcar` no puede borrar la
   clave `tipo` porque el upsert fusiona props con `||` y nunca elimina. Esta
   feature no lo empeora (el nodo se sigue escribiendo igual). Merece feature
   propia.
4. **Bloqueo documentado (ajeno a esta feature)**: el refresh de graphify que
   dispara `close` dejo `graphify-out/.graphify_stale`, asi que
   `harness_check.sh` reporta ese 1 problema despues del cierre. La causa NO es
   este cambio: `graphify update` se niega a sobrescribir porque el grafo nuevo
   tiene 1426 nodos contra los 1440 del `graph.json` existente
   (*"Refusing to overwrite -- you may be missing chunk files from a previous
   session. Pass --force to override"*), y ademas avisa que el grafo usa el
   esquema de IDs pre-#1504. Resolverlo implica pisar el grafo del USUARIO
   (`graphify update --force` / `graphify extract --force`), asi que se deja a
   su decision en vez de forzarlo. `graphify-out/` esta en `.gitignore`: es
   estado local, no entra al commit. Antes del cierre, `harness_check.sh` pasaba
   limpio.
