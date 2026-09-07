Aplicado: 2026-09-07T00:24:14Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #80: el autoaprendizaje no tiene ciclo de vida: lecciones sin tope, la misma leccion en diez cierres, perfil sin alimentar y consolidacion sin correr

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 80`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (módulo `nombre`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:101 (spec `evento`) y 331 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `AGENTS.md`, `README.md`, `UPDATING.md` y 31 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| 20 | El instalador respalda el backlog y `--reset` deja de borrarlo | el_instalador_respalda_el_backlog | <O1> | `feature_list.json` y `progress/` son los unicos datos del proyecto que el instalador no puede regenerar, y eran los unicos que no respaldaba: `bkp/` guardaba trece scripts y ningun backlog. Ahora salen de los targets del `--reset` (sh y ps1), se respaldan en TODA corrida en un paso propio (`backup_datos` / `Backup-HarnessData`) que `--force` no saltea —`--force` significa "no respaldes lo que vas a regenerar", no "no respaldes los datos"— y si faltan, la siembra de la plantilla vacia lo dice en `[WARN]`, nombra los respaldos que hay (`bkp/` y `docs/bkp-backlog/`, con cuantas features tiene cada uno) y el comando para volver. El check `tests/backlog_backup_check.sh` (dentro del smoke) planta un backlog y afirma el efecto en los cuatro modos; contra el instalador anterior caen los cuatro. La politica de espejo del backlog gitignorado de ESTE repo queda como decision del usuario, fuera de la feature. Disparador: una corrida del instalador sobre realestate el 2026-09-06 dejo el backlog en la plantilla, sin aviso y sin copia | done (2026-09-06) |
Despues:
| 20 | El instalador respalda el backlog y `--reset` deja de borrarlo | el_instalador_respalda_el_backlog | <O1> | `feature_list.json` y `progress/` son los unicos datos del proyecto que el instalador no puede regenerar, y eran los unicos que no respaldaba: `bkp/` guardaba trece scripts y ningun backlog. Ahora salen de los targets del `--reset` (sh y ps1), se respaldan en TODA corrida en un paso propio (`backup_datos` / `Backup-HarnessData`) que `--force` no saltea —`--force` significa "no respaldes lo que vas a regenerar", no "no respaldes los datos"— y si faltan, la siembra de la plantilla vacia lo dice en `[WARN]`, nombra los respaldos que hay (`bkp/` y `docs/bkp-backlog/`, con cuantas features tiene cada uno) y el comando para volver. El check `tests/backlog_backup_check.sh` (dentro del smoke) planta un backlog y afirma el efecto en los cuatro modos; contra el instalador anterior caen los cuatro. La politica de espejo del backlog gitignorado de ESTE repo queda como decision del usuario, fuera de la feature. Disparador: una corrida del instalador sobre realestate el 2026-09-06 dejo el backlog en la plantilla, sin aviso y sin copia | done (2026-09-06) |
| 21 | Lo aprendido tiene ciclo de vida: tope, racha y dos avisos | el_autoaprendizaje_tiene_ciclo_de_vida | <O1> | Medido el 2026-09-06 sobre este repo: 10 de los ultimos 15 cierres declararon la misma leccion (442 lineas, nueve secciones "(feature #N)"), el paso 3 de la guia (`<clase>/referencias/`) con cero usos, 340 decisiones sin incorporar al perfil y la consolidacion sin correr en 19 dias y sin registro; `require_leccion` media que se declare una leccion, no que se aprenda. Cuatro umbrales en `rules` (`0` apaga): `leccion_max_lineas` (250) —sobre el tope, `close --leccion` y `leccion usar` se niegan con el contrato de particion a `referencias/`, sin escape por flag—; `leccion_repeticiones` (3) —la misma clase K cierres seguidos exige `--leccion-motivo`, `ninguna` no cuenta—; `perfil_pendientes_max` (25) y `consolidar_cada_dias` (30) —el cierre avisa por stderr, y `consolidar`/`curar` registran su corrida en `history.md`—. `lecciones status` y `harness_check.sh` lo muestran. La biblioteca de este repo se puso dentro del tope (442 -> 186, 316 -> 167, doce referencias), sin fusionar las tres lecciones que el consolidador proponia, y el perfil recibio cuatro entradas aprobadas una por una. Disparador: la pregunta del usuario "¿quedo bien implementado el autoaprendizaje de Hermes?" y la medicion que la respondio | done (2026-09-06) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:10 (spec `ningun`), docs/prd/SDD-master.md:10 (spec `ninguna`), docs/prd/SDD-master.md:101 (spec `cuando`) y 486 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `AGENTS.md`, `README.md`, `UPDATING.md` y 31 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
| D8 | Los DATOS del proyecto (`feature_list.json`, `progress/`) no son superficie del instalador: no entran en `--reset`, se respaldan en toda corrida sin mirar `--force`, y su siembra nunca es silenciosa | (a) dejar `--force` como "sin backup de nada" y documentarlo; (b) restaurar automaticamente desde `bkp/` cuando el backlog falta | `--force` nacio para no respaldar lo que el instalador regenera; el backlog no lo regenera nadie, asi que saltearlo no ahorraba nada y costaba lo unico irrecuperable. Restaurar solo es decidir por el usuario cual de varios respaldos es el bueno —el instalador no lo sabe—; nombrarlos con su cantidad de features y dar el comando deja la decision donde corresponde. El test afirma el efecto (`bkp/feature_list.json.bak.*` existe y es identico al plantado), no la llamada: eso atrapo un `for` que con `IFS=$'\n\t'` no partia la lista y respaldaba nada sin error | 2026-09-06 |
Despues:
| D8 | Los DATOS del proyecto (`feature_list.json`, `progress/`) no son superficie del instalador: no entran en `--reset`, se respaldan en toda corrida sin mirar `--force`, y su siembra nunca es silenciosa | (a) dejar `--force` como "sin backup de nada" y documentarlo; (b) restaurar automaticamente desde `bkp/` cuando el backlog falta | `--force` nacio para no respaldar lo que el instalador regenera; el backlog no lo regenera nadie, asi que saltearlo no ahorraba nada y costaba lo unico irrecuperable. Restaurar solo es decidir por el usuario cual de varios respaldos es el bueno —el instalador no lo sabe—; nombrarlos con su cantidad de features y dar el comando deja la decision donde corresponde. El test afirma el efecto (`bkp/feature_list.json.bak.*` existe y es identico al plantado), no la llamada: eso atrapo un `for` que con `IFS=$'\n\t'` no partia la lista y respaldaba nada sin error | 2026-09-06 |
| D9 | Las lecciones de clase tienen memoria ACOTADA como el perfil: tope de lineas duro, el detalle por feature en `<clase>/referencias/`, y la misma clase K veces seguidas exige motivo | (a) `--leccion-motivo` como escape del tope; (b) fusionar las lecciones solapadas bajo un paraguas, como proponia `consolidar`; (c) partir en clases nuevas | Un escape por flag se vuelve el default (paso con `ninguna`, que por eso exige motivo), y el limite del perfil es duro y funciona. Fusionar iba contra el tope: el paraguas naceria con ~1000 lineas, y los tres candidatos con confianza 1.00 se explicaban por declararse `relacionadas` entre si, no por ensenar lo mismo. Clases nuevas es lo que la guia desaconseja salvo que ninguna cubra el tema. La guia ya tenia la respuesta (paso 3, `referencias/`) con cero usos: el gate que media la declaracion no empujaba a usarla; un limite fisico si | 2026-09-06 |

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (spec `aplica`), docs/architecture.md:100 (spec `aplicar`), docs/architecture.md:101 (módulo `cierre`) y 814 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `AGENTS.md`, `README.md`, `UPDATING.md` y 31 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
- `lecciones.rs`: la memoria procedural (`docs/lecciones/<clase>.md`, feature
  #17). Expone `validar_nombre_de_clase` (rechaza nombres de sesion: con
Despues:
- `lecciones.rs`: la memoria procedural (`docs/lecciones/<clase>.md`, feature
  #17; ciclo de vida de lo aprendido, feature #80: `Politica::from_rules` lee los cuatro umbrales de `rules`, `Leccion::lineas`/`sobre_el_tope`/`secciones_por_feature` y `contrato_de_particion` sostienen el tope, `cierre_declarado`/`racha` leen `history.md` para la racha, y `ultima_consolidacion`/`perfil_pendientes`/`texto_avisos_de_ciclo` arman los avisos que `close` emite por stderr). Expone `validar_nombre_de_clase` (rechaza nombres de sesion: con

