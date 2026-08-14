# Plan - Feature #14: hub_batch_upserts_atomic_install

Estado: in_progress
Microservicios:
- harness

## Alcance

Sacar del camino los dos costos que hoy hacen del arnes algo lento y fragil:

1. El hub reescribe el grafo entero fila por fila en cada comando. Pasa a
   escribir SOLO lo que el comando toco, en lotes con `UNNEST`, con candado por
   proyecto y con corte de sentencia configurable.
2. Los instaladores copian el binario encima del binario vivo. Pasan a escribir
   un temporal hermano y moverlo con rename atomico.

Spec aprobado: `docs/spec-feature-14-hub-batch-upserts-atomic-install.md`
(AC-1 a AC-14), aprobado por el usuario el 2026-08-14.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

Microservicio unico (`harness`): el arnes es el producto. El `graph impacto` no
se pudo correr contra el hub durante la fase de lider porque el candado global
estaba tomado por un `sync_git` de otro proyecto (1 h 09 min) -- que es
exactamente lo que esta feature arregla --, asi que el impacto se calculo por
lectura directa del repo:

- `rust/src/graph/store.rs`: `save()`, `add_node`, `add_edge`, `load`,
  construccion del cliente (timeout).
- `rust/src/graph/mod.rs`: candado por proyecto (`GraphEnv::lock_file`) y
  lectura de `DB_STATEMENT_TIMEOUT`.
- `rust/src/graph/commands.rs`: `unmark` muta `store.nodes` directo; debe marcar
  el nodo como sucio para no perder la escritura (AC-4).
- `setup_harness.sh` y `setup_harness.ps1`: escritura del binario (paridad).
- `tests/setup_smoke.sh`: caso de doble instalacion (AC-13).
- `README.md` / `UPDATING.md`: `DB_STATEMENT_TIMEOUT` y el candado por proyecto.
- Sin impacto: `templates/` (ninguno de los archivos tocados tiene espejo ahi:
  `setup_harness.*` y `rust/` no se copian a `templates/`), esquema de la base,
  `Cargo.toml` (sin dependencias nuevas).

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`graphify-out/` de este repo esta al dia de la feature #13. La superficie tocada
(`rust/src/graph/*`, los dos instaladores, el smoke) se leyo directo en la fase
de lider, incluyendo la medicion en vivo contra el hub real: 1047 nodos, 1641
aristas, 164 ms de round-trip (30 vs 130 consultas: 6,21 s vs 22,62 s).

## Delegacion (implementer)

- D1 [AC-1, AC-2, AC-3, AC-6]: `store.rs` -- `save()` por lotes con
  `INSERT ... SELECT * FROM UNNEST(...)` (constante `UPSERT_CHUNK = 1000`),
  dentro de la misma transaccion. Antes de armar el lote de aristas, fusionarlas
  por `(source, target, type)` con "la ultima gana clave a clave" para no
  gatillar el error de ON CONFLICT sobre fila repetida.
- D2 [AC-4, AC-7]: `store.rs` -- registro de lo sucio: `add_node`/`add_edge`
  anotan la clave tocada, `load()` lo vacia, `save()` escribe solo eso y no abre
  transaccion si no hay nada. Funciones puras separadas (`node_rows`,
  `edge_rows`) para poder testearlas sin base de datos.
- D3 [AC-4]: `commands.rs` -- `unmark` marca su nodo con el metodo publico nuevo
  despues de mutar `store.nodes` a mano, de modo que lo persistido sea identico
  a hoy.
- D4 [AC-8]: `mod.rs` -- `lock_file` pasa a `<hub>/.lock-<proyecto>` con el
  nombre del proyecto saneado para el filesystem.
- D5 [AC-9]: `mod.rs` + `store.rs` -- `DB_STATEMENT_TIMEOUT` (ms, default 30000,
  `0` desactiva) validado como numero y pasado como `-c statement_timeout=<ms>`
  en las opciones de conexion, mas keepalives TCP.
- D6 [AC-10, AC-11]: `setup_harness.sh` -- helper `install_binary_atomic`
  (temporal hermano + `chmod +x` + `mv -f`, limpieza y error accionable ante
  cualquier fallo) usado por la rama de compilacion del binario.
- D7 [AC-12]: `setup_harness.ps1` -- equivalente con `Copy-Item` a temporal +
  `Move-Item -Force`, apartando el destino bloqueado a `.harness.exe.old.<pid>`
  cuando esta en uso, con limpieza y sin tocar nada en `-DryRun`.
- D8 [AC-13]: `tests/setup_smoke.sh` -- doble instalacion sobre el mismo
  directorio: el inode del binario cambia y no quedan temporales.
- D9 [AC-9, AC-8]: `README.md` y `UPDATING.md` -- documentar
  `DB_STATEMENT_TIMEOUT` y el candado por proyecto.

## Criterios de cierre (reviewer)

- `cargo test` y `cargo clippy -- -D warnings` limpios (Articulo 1).
- `bash tests/setup_smoke.sh` limpio, incluido el caso nuevo de doble
  instalacion.
- `bash harness_check.sh` limpio.
- Evidencia por AC-n en `docs/impl-feature-14.md` (Articulo 3), incluida la
  medicion real contra el hub antes/despues.
- Sin dependencias nuevas en `rust/Cargo.toml` (Articulo 6).
- Commit Conventional sin trailers de IA (Articulo 6).

## Riesgos

- R1: `UNNEST` con `ON CONFLICT` falla si el lote trae la misma clave dos veces
  ("cannot affect row a second time"). Mitigado por D1 (fusion previa de
  aristas) y por la unicidad natural de `IndexMap` para nodos. Cubierto por
  test (AC-7).
- R2: El delta puede dejar de escribir algo que hoy si se escribe, si alguien
  muta `store.nodes`/`store.edges` sin pasar por `add_node`/`add_edge`. Hoy el
  unico caso es `unmark` (D3); queda documentado en el modulo.
- R3: Candado por proyecto = dos proyectos escribiendo a la vez. Es seguro
  porque cada uno escribe solo sus filas (D2); las unicas filas compartidas son
  los nodos `Agente` (`AgentCLI`, `Agente_Implementador`), cuyo upsert es
  idempotente y de contenido identico.
- R4: `statement_timeout` demasiado corto podria cortar una operacion legitima.
  Mitigado: con los lotes, la sentencia mas cara es un upsert de 1000 filas
  (sub-segundo); el default de 30 s deja margen amplio y `0` lo desactiva.
- R5: El binario del repo fuente (`./harness`) se recompila con estos cambios,
  pero los binarios YA instalados en otros proyectos siguen con el codigo viejo
  hasta que se re-corra el instalador ahi. Es esperado y se avisa en el cierre.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- OBS-1 [decidida 2026-08-14]: lote con `UNNEST`, no `COPY` a tabla temporal.
- OBS-2 [decidida 2026-08-14]: `mv` atomico en los DOS instaladores (paridad).
- OBS-3 [decidida 2026-08-14]: timeout de sentencia y candado por proyecto
  entran en esta feature; ante el fork del candado, el usuario eligio la opcion
  segura (escribir solo el delta).
- OBS-4 [registrada, sin accion]: `desmarcar` no puede borrar la clave `tipo`
  del hub (el merge `||` nunca elimina). Bug preexistente, no se empeora, queda
  para una feature propia.

### Avance 2026-08-14T03:49:12Z
D1-D5 implementados: save() por lotes con UNNEST y solo el delta sucio, candado por proyecto y DB_STATEMENT_TIMEOUT. Plan completo escrito; se re-firman plan y spec.

### Avance 2026-08-14T04:09:22Z
D6-D9 completos: instaladores sh/ps1 con mv atomico, smoke de doble instalacion (inode nuevo) en sh y ps1, README + UPDATING (y su espejo en templates) documentando DB_STATEMENT_TIMEOUT, candado por proyecto y la ventana de transicion. Evidencia por AC en docs/impl-14.md y veredicto en docs/review-14.md. Medicion: 2688 sentencias = 456,31 s vs 4 por lote = 2,74 s.

---
Cerrado: 2026-08-14T04:10:09Z - status=done - Hub por lotes con UNNEST y escritura solo del delta (2688 sentencias -> 4; 456,31 s -> 2,74 s medidos), candado por proyecto, DB_STATEMENT_TIMEOUT, e instalacion atomica del binario en sh y ps1 (adios SIGKILL en cada actualizacion)
