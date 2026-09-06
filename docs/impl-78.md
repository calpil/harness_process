# Impl - Feature #78: el instalador respalda sus scripts pero no el backlog, y --reset lo borra

Spec: docs/spec-feature-78-el-instalador-respalda-sus-scripts-pero-no-el-ba.md
Plan: docs/plan-feature-78-el-instalador-respalda-sus-scripts-pero-no-el-ba.md

## Lo que estaba pasando (realestate, 2026-09-06 19:12)

Una corrida del instalador dejo `feature_list.json` en la plantilla vacia. En
`bkp/` habia respaldo de trece scripts (`init.sh`, `harness_cli`, `roles/*`,
`CLAUDE.md`...) y ninguno del backlog. Medido en fixture: reinstalar no respalda
el backlog; `--reset` lo borra (con backup); `--reset --force` lo borra sin
backup; y si el archivo falta, se siembra la plantilla sin decir nada.

## El arreglo: los datos no son superficie

El instalador regenera scripts y templates; puede permitirse no respaldarlos con
`--force`. El backlog y `progress/` no los regenera nadie: si se pierden, se
pierden. Entonces (a) salen de la lista de borrado del `--reset`, (b) se
respaldan en TODA corrida, en un paso propio que `--force` no saltea, y (c) si
faltan, la siembra avisa en `[WARN]` y nombra los respaldos que hay.

## Evidencia por AC

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | setup_harness.sh:396 · setup_harness.ps1:1892 | `feature_list.json` y `progress` ya no estan en `reset_targets` ni en `$targets` de `Invoke-HarnessReset`; el comentario dice por que. Test: `tests/backlog_backup_check.sh` modos `reset` y `reset-force` (tests/backlog_backup_check.sh:62-63): backlog byte-identico via `sigue_igual` (:48). |
| AC-2 | setup_harness.sh:700 · :766 · :2720 | `backup_datos` respalda `feature_list.json`, `progress/current.md`, `progress/history.md` y los `progress/current-<id>.md` con `backup_path` (timestamp en `bkp/`), sin mirar `FORCE`. Se llama en `--reset` con `BKP_DIR` ya resuelto y antes de borrar nada (:766), y en el camino normal antes de tocar el primer asset (:2720). En ps1: `Backup-HarnessData` (setup_harness.ps1:1811), llamada en :1853 y :2050. Test: `respaldo_en_bkp` (tests/backlog_backup_check.sh:52) exige `bkp/feature_list.json.bak.*` byte-identico al backlog plantado, en los tres modos. |
| AC-3 | setup_harness.sh:724 · :2920 | `sembrar_dato_avisando`: `[WARN] FALTA feature_list.json: se siembra la plantilla VACIA...`, luego una linea por respaldo (`bkp/feature_list.json.bak.*` y `docs/bkp-backlog/feature_list.json`) con `N feature(s), M regla(s)` contados con python3, y `para volver: cp <respaldo> ...`. Siembra igual. ps1: `Install-HarnessDataIfMissing` (setup_harness.ps1:1830). Test: `modo_faltante` (tests/backlog_backup_check.sh:64) afirma `[WARN]`, el nombre del `.bak` y `2 feature`. |
| AC-4 | setup_harness.sh:2927-2928 · setup_harness.ps1:2058 | La misma funcion siembra `progress/current.md` y `progress/history.md`; el aviso es el mismo. |
| AC-5 | tests/backlog_backup_check.sh · tests/setup_smoke.sh:1702 | Planta 2 features + 5 reglas + una linea en `history.md` (`plantar`, :37), corre reinstalar / `--reset` / `--reset --force` / archivo faltante. Enganchado al smoke. Prueba del rojo: contra `git show HEAD:setup_harness.sh` caen los cuatro modos (ver "El rojo"). |
| AC-6 | setup_harness.ps1:1811-1853 · tests/parity_check.sh | Mismas tres piezas en PowerShell. `bash tests/parity_check.sh`: los diez modos verdes. Asimetria declarada: aca no hay `pwsh`, el ps1 se verifica por paridad y lectura, no por ejecucion. |
| AC-7 | setup_harness.sh:383 · UPDATING.md:48 · templates/UPDATING.md | `--help` dice que `--force` no evita el respaldo de datos y que `--reset` no toca backlog ni `progress/`. Seccion nueva en UPDATING.md (las dos copias identicas). |
| AC-8 | docs/review-78.md ("Decision del usuario (AC-8)") | MANUAL: la politica de espejo del backlog gitignorado de este repo se le pregunto al usuario al cerrar; decidio dejarlo como esta (solo `bkp/`). Registrado en el review. |

## El rojo

Contra el instalador de HEAD (`git show HEAD:setup_harness.sh` en el worktree,
`cmp` para confirmar el swap), los cuatro modos caen con su propio motivo:

```
[!] reinstall:   no hay bkp/feature_list.json.bak.* (el instalador no respaldo el backlog)
[!] reset:       el backlog CAMBIO (o desaparecio)
[!] reset-force: el backlog CAMBIO (o desaparecio)
[!] faltante:    el aviso no nombra el respaldo que hay en bkp/
```

Con el fix restaurado: `[Ok]` en los cuatro. Antes de llegar ahi hubo un rojo
falso (el check caia en "falta el binario prebuilt", una precondicion, en las
dos corridas) y un verde falso (mi script imprimia `=== VERDE ===` como titulo y
salia 0 con cuatro `[!]` debajo). Los dos estan en la leccion.

## Los dos bugs que el test atrapo en mi propio fix

1. `for dato in $DATOS_DEL_PROYECTO` con `IFS=$'\n\t'` (setup_harness.sh:26)
   itera UNA palabra: la lista entera. La funcion corria y respaldaba cero,
   sin error. Ahora la lista son palabras literales del `for` (:700).
2. En `--reset`, llame a `backup_datos` antes de resolver `BKP_DIR`. Movida
   despues (:766), y siempre antes del primer `rm -rf`.

## Lo que NO hace

- No decide la politica de espejo del backlog de este repo (AC-8).
- No restaura nada solo: nombra los respaldos y el comando.
- No toca el binario Rust; el unico cambio en `rust/` es la entrada de este spec
  en el corpus del test `los_siete_que_faltaban_y_ninguno_mas`
  (rust/src/verificacion.rs:1266), porque el AC-8 va con `(MANUAL)`.

## Suite

- `bash tests/backlog_backup_check.sh todos`: 4/4 verdes (con el binario prebuilt de main).
- `bash tests/parity_check.sh`: los diez modos verdes.
- `tests/commit_guard_check.sh`, `commit_guard_stdin_check.sh`, `stop_hook_check.sh`: verdes.
- `cargo test --locked`: 473 + 263 tests, 0 fallos. `cargo clippy --locked --all-targets -- -D warnings`: exit 0.
- `bash tests/setup_smoke.sh` (con el bloque nuevo): verde hasta la ultima linea, `[Ok] Backlog #78`.
