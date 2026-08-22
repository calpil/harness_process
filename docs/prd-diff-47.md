Aplicado: 2026-08-22T00:01:13Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #47: features_en_paralelo_con_worktrees

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 47`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'features_en_paralelo_con_worktrees')
Veredicto: cambio
Antes:
| 7 | Un AC que no ejecuto ningun caso deja de contar como verificado | verify_detecta_filtro_vacio | <O1> | `verify` mira la SALIDA ademas del exit code: si reconoce el formato de libtest y la suma de `passed` es cero, el AC queda en `vacio`, se cuenta aparte en el resumen y bloquea el cierre igual que un rojo; sobre salidas que no son de tests el estado no cambia | done (2026-08-19) |
Despues:
| 7 | Un AC que no ejecuto ningun caso deja de contar como verificado | verify_detecta_filtro_vacio | <O1> | `verify` mira la SALIDA ademas del exit code: si reconoce el formato de libtest y la suma de `passed` es cero, el AC queda en `vacio`, se cuenta aparte en el resumen y bloquea el cierre igual que un rojo; sobre salidas que no son de tests el estado no cambia | done (2026-08-19) |
| 8 | Features en paralelo sin pisarse | features_en_paralelo_con_worktrees | <O1> | `start` deja de rechazar la segunda feature activa y le da a cada una su rama GitFlow (`feature/<id>-<slug>`, `bugfix/` si es `kind: bug`) y su worktree hermano; el estado del arnes sigue siendo unico (repo principal) y el vivo se parte en `current-<id>.md` con `current.md` como indice; dentro del worktree los comandos infieren la feature; `close --status done` exige `--to <rama>`, mergea, publica, borra el worktree y conserva la rama, y un conflicto aborta sin dejar nada a medias | pendiente |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'features_en_paralelo_con_worktrees')
Veredicto: cambio
Antes:
## 4. Decisiones tecnicas
Despues:
## 4. Decisiones tecnicas

**Aislamiento entre features** (feature #47). Dos implementaciones simultaneas
no comparten archivos: cada feature vive en su rama GitFlow y en su worktree
hermano del repo. Tres decisiones que valen para cualquier feature futura que
toque el flujo:

- **El estado del arnes es unico.** `feature_list.json` y `progress/` se
  resuelven contra el repo PRINCIPAL (`git rev-parse --git-common-dir`) aunque
  el binario se invoque desde un worktree: el backlog no se bifurca nunca.
- **Los docs se resuelven DESDE la feature, no desde el directorio actual.**
  `HarnessPaths::para_feature()` apunta `docs/` al worktree de esa feature, para
  que su spec, su plan y su evidencia viajen con el merge de su rama.
- **El arnes nunca reescribe historia ni elige la rama destino.** Sin `--force`,
  sin rebase, sin squash y sin borrar ramas; el merge corre en un worktree
  temporal (no toca tu checkout) y `--to` lo decide el USUARIO.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'features_en_paralelo_con_worktrees')
Veredicto: cambio
Antes:
## Flujo Spec-Driven Development (SDD)
Despues:
## Features en paralelo (feature #47)

`start` le da a cada feature su rama GitFlow (`feature/<id>-<slug>`, o
`bugfix/<id>-<slug>` si se cargo con `add --kind bug`) y su worktree hermano
(`../<repo>-wt/<id>-<slug>`), creado ANTES de escribir el plan y el spec para
que nazcan en esa rama. El checkout principal no cambia de rama nunca.

- `rust/src/git.rs`: ramas, worktrees, merge (en un worktree temporal), push y
  commit sin trailers de IA. Sin repo git, todo degrada a no hacer nada.
- Estado: `feature_list.json` y `progress/` son unicos y del repo principal;
  el estado vivo es `progress/current-<id>.md` por feature y `current.md` pasa a
  ser el indice de lo abierto, con `.last_autocheck-<id>` por feature.
- Foco: dentro de un worktree los comandos infieren la feature por la carpeta
  (`feature_por_worktree`); fuera y con varias activas, exigen `--feature`.
- Cierre: `close --status done` exige `--to <rama>` (el arnes no la elige),
  mergea, publica, borra el worktree y conserva la rama. Un conflicto aborta.

## Flujo Spec-Driven Development (SDD)

