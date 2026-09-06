# Review - Feature #78: el instalador respalda sus scripts pero no el backlog, y --reset lo borra
Revisado: approved · 2026-09-06T22:53:40Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento. Metodo: correr el check nuevo contra
el instalador de HEAD (rojo) y contra el fix (verde), con `cmp` en cada swap;
paridad; y leer el ps1 linea a linea contra el sh porque aca no hay `pwsh`.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | setup_harness.sh:396 · setup_harness.ps1:1892 | CUBIERTO. Modos `reset` y `reset-force` caen contra HEAD ("el backlog CAMBIO") y pasan con el fix. |
| AC-2 | setup_harness.sh:700 · setup_harness.ps1:1811 | CUBIERTO. `respaldo_en_bkp` compara el `.bak` mas nuevo byte a byte con el backlog plantado; pasa en reinstall, reset y reset-force. `--dry-run` solo loguea `[DRY-RUN] Backup de datos`. |
| AC-3 | setup_harness.sh:724 · setup_harness.ps1:1830 | CUBIERTO. `modo_faltante` cae contra HEAD ("el aviso no nombra el respaldo") y pasa con el fix: `[WARN]`, `.bak` nombrado, `2 feature(s)`. |
| AC-4 | setup_harness.sh:2927 · setup_harness.ps1:2058 | CUBIERTO por la misma funcion; el check afirma que `progress/history.md` tiene respaldo (`respaldo_en_bkp`). |
| AC-5 | tests/backlog_backup_check.sh · tests/setup_smoke.sh:1702 | CUBIERTO. Los cuatro asertos caen contra HEAD, cada uno por su motivo, y el backlog plantado tiene features y reglas distintas de la plantilla. |
| AC-6 | setup_harness.ps1:1811-1853, :2050-2058 · tests/parity_check.sh | CUBIERTO con asimetria declarada: el ps1 no se ejecuta en esta maquina (sin `pwsh`); se verifico paridad (10/10) y lectura funcion por funcion (`Get-RelativeBackupName`, `$script:BackupDir`, `$script:Counters.backed_up`, `$script:SurfaceDir` existen y se usan como en `Backup-HarnessPath`). |
| AC-7 | setup_harness.sh:383 · UPDATING.md:48 | CUBIERTO. Las dos copias de UPDATING.md son identicas (`cmp`). |
| AC-8 | docs/spec-feature-78-el-instalador-respalda-sus-scripts-pero-no-el-ba.md:93 | MANUAL, CUMPLIDO: la politica quedo registrada como decision del usuario ("Decision del usuario (AC-8)"). |

## Lo que el review tiene que decir

El primer "verde" que reporte no era verde. El script de rojo/verde imprimia el
titulo `VERDE` y salia 0 con cuatro `[!]` abajo; yo lei el titulo. Cuando lei
los `[!]`, el fix tenia dos bugs reales: el `for` sobre una variable con espacios
bajo `IFS=$'\n\t'` (respaldaba nada, sin error) y el respaldo del `--reset`
llamado antes de resolver `BKP_DIR`. El test los atrapo porque afirma el efecto
(`bkp/feature_list.json.bak.*` existe y es identico) y no la llamada. Los dos
episodios quedaron en `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`.

## Riesgo declarado

- `--reset` de instalaciones que CONTABAN con que borrara el backlog: cambia de
  comportamiento. No hay evidencia de ese uso; esta en UPDATING.md.
- El ps1 no corrio en esta maquina. Si en Windows `Get-ChildItem -Filter` con
  `feature_list.json.bak.*` no lista los respaldos, el aviso de AC-3 dira "no se
  encontro ningun respaldo" en vez de nombrarlos; la siembra y el respaldo
  (AC-1, AC-2) no dependen de eso.

## Decision del usuario (AC-8)

El backlog de `harness_process` esta gitignorado y no tiene espejo. Se le
pregunto al usuario al cerrar (2026-09-06) si queria versionar uno en
`docs/bkp-backlog/feature_list.json`, como realestate. Decidio: **dejar como
esta, solo `bkp/`**. No se abre feature. Queda registrado aca y en el hito 20
del PRD.

## Veredicto

Ocho AC con cobertura (siete ejecutables, uno manual registrado). Los cuatro
asertos nuevos caen contra HEAD y pasan con el fix. Suite Rust (473 + 263, 0 fallos),
clippy limpio y smoke verde con el bloque nuevo adentro.
