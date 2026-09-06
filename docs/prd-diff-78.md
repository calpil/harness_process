Aplicado: 2026-09-06T22:53:26Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #78: el instalador respalda sus scripts pero no el backlog, que es lo unico irrecuperable

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 78`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:11 (spec `instalador`) y 168 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/verificacion.rs`, `setup_harness.ps1` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 19 | Con docs/ como repo aparte, los documentos van directo a docs/ | docs_aparte_escribe_en_docs | <O1> | Se revierte la OBS-5 de la #72: el repo `docs/` aparte ya no recibe un worktree por feature. Spec, plan, impl y review van a `<raiz>/docs/`, junto al PRD y el SDD, desde que se escriben; no existe `docs-wt/`, no hay ramas del repo docs y `docs_worktree` se ignora si quedo en el backlog. Con docs dentro del repo principal nada cambia. Disparador: tres `docs-wt/` en realestate con sus specs sin commitear en ramas que nadie mergeo, y la pregunta del usuario de por que sus documentos no estaban en docs/ con el PRD | done (2026-09-06) |
Despues:
| 19 | Con docs/ como repo aparte, los documentos van directo a docs/ | docs_aparte_escribe_en_docs | <O1> | Se revierte la OBS-5 de la #72: el repo `docs/` aparte ya no recibe un worktree por feature. Spec, plan, impl y review van a `<raiz>/docs/`, junto al PRD y el SDD, desde que se escriben; no existe `docs-wt/`, no hay ramas del repo docs y `docs_worktree` se ignora si quedo en el backlog. Con docs dentro del repo principal nada cambia. Disparador: tres `docs-wt/` en realestate con sus specs sin commitear en ramas que nadie mergeo, y la pregunta del usuario de por que sus documentos no estaban en docs/ con el PRD | done (2026-09-06) |
| 20 | El instalador respalda el backlog y `--reset` deja de borrarlo | el_instalador_respalda_el_backlog | <O1> | `feature_list.json` y `progress/` son los unicos datos del proyecto que el instalador no puede regenerar, y eran los unicos que no respaldaba: `bkp/` guardaba trece scripts y ningun backlog. Ahora salen de los targets del `--reset` (sh y ps1), se respaldan en TODA corrida en un paso propio (`backup_datos` / `Backup-HarnessData`) que `--force` no saltea —`--force` significa "no respaldes lo que vas a regenerar", no "no respaldes los datos"— y si faltan, la siembra de la plantilla vacia lo dice en `[WARN]`, nombra los respaldos que hay (`bkp/` y `docs/bkp-backlog/`, con cuantas features tiene cada uno) y el comando para volver. El check `tests/backlog_backup_check.sh` (dentro del smoke) planta un backlog y afirma el efecto en los cuatro modos; contra el instalador anterior caen los cuatro. La politica de espejo del backlog gitignorado de ESTE repo queda como decision del usuario, fuera de la feature. Disparador: una corrida del instalador sobre realestate el 2026-09-06 dejo el backlog en la plantilla, sin aviso y sin copia | done (2026-09-06) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:10 (spec `ninguna`) y 262 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/verificacion.rs`, `setup_harness.ps1` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D7 | Los documentos compartidos van a la raiz, y con `docs/` como repo aparte eso incluye TODOS los documentos de la feature (`paths::para_feature` resuelve `<raiz>/docs`) | (a) un worktree del repo docs por feature (la OBS-5 de la #72); (b) mergear ese worktree al cerrar | Un worktree por feature tiene ciclo de vida —rama, merge, borrado— del que nadie se hacia cargo, y cada feature dejaba su spec en una rama que nadie mergeaba. Mergearlo al cerrar es tocar el repo de documentacion del usuario desde codigo. La respuesta chica ya estaba tomada dos veces en este repo (#60 bitacora del PRD, #71 sello de cierre): lo compartido va a la raiz | 2026-09-06 |
Despues:
| D7 | Los documentos compartidos van a la raiz, y con `docs/` como repo aparte eso incluye TODOS los documentos de la feature (`paths::para_feature` resuelve `<raiz>/docs`) | (a) un worktree del repo docs por feature (la OBS-5 de la #72); (b) mergear ese worktree al cerrar | Un worktree por feature tiene ciclo de vida —rama, merge, borrado— del que nadie se hacia cargo, y cada feature dejaba su spec en una rama que nadie mergeaba. Mergearlo al cerrar es tocar el repo de documentacion del usuario desde codigo. La respuesta chica ya estaba tomada dos veces en este repo (#60 bitacora del PRD, #71 sello de cierre): lo compartido va a la raiz | 2026-09-06 |
| D8 | Los DATOS del proyecto (`feature_list.json`, `progress/`) no son superficie del instalador: no entran en `--reset`, se respaldan en toda corrida sin mirar `--force`, y su siembra nunca es silenciosa | (a) dejar `--force` como "sin backup de nada" y documentarlo; (b) restaurar automaticamente desde `bkp/` cuando el backlog falta | `--force` nacio para no respaldar lo que el instalador regenera; el backlog no lo regenera nadie, asi que saltearlo no ahorraba nada y costaba lo unico irrecuperable. Restaurar solo es decidir por el usuario cual de varios respaldos es el bueno —el instalador no lo sabe—; nombrarlos con su cantidad de features y dar el comando deja la decision donde corresponde. El test afirma el efecto (`bkp/feature_list.json.bak.*` existe y es identico al plantado), no la llamada: eso atrapo un `for` que con `IFS=$'\n\t'` no partia la lista y respaldaba nada sin error | 2026-09-06 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:100 (spec `aplica`), docs/architecture.md:101 (módulo `cierre`) y 457 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `UPDATING.md`, `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`, `rust/src/verificacion.rs`, `setup_harness.ps1` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
- `--reset` borra los docs generados del arnes en ambas ubicaciones (nueva y
  vieja) y conserva la constitution, los dotfiles Kimi y los artefactos de
  feature.
Despues:
- `--reset` borra los docs generados del arnes en ambas ubicaciones (nueva y
  vieja) y conserva la constitution, los dotfiles Kimi, los artefactos de
  feature, y (feature #78) el backlog y `progress/`: `feature_list.json`,
  `progress/current*.md` y `progress/history.md` no estan en los reset targets
  y se respaldan en `bkp/` en TODA corrida (`backup_datos` sh /
  `Backup-HarnessData` ps1) en un paso que `--force` no saltea; si faltan, la
  siembra (`sembrar_dato_avisando` / `Install-HarnessDataIfMissing`) avisa en
  `[WARN]` y nombra los respaldos de `bkp/` y `docs/bkp-backlog/`.

