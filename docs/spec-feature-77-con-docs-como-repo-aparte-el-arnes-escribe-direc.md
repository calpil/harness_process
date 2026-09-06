# Spec - Feature #77: con docs/ como repo aparte, el arnes escribe directo en docs/ y no crea docs-wt

Estado: approved
Aprobado: 2026-09-06T21:57:19Z por USUARIO (confirmacion explicita) - Aprobado por Alan en chat: con docs aparte, directo en docs/, sin worktree; revierte la OBS-5 de la #72
Plan: docs/plan-feature-77-con-docs-como-repo-aparte-el-arnes-escribe-direc.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md

## La historia (antes -> despues)

ANTES: Alan abre el arbol de realestate y ve `docs-wt/131-…`, `docs-wt/132-…`,
`docs-wt/142-…`. Adentro estan el spec, el plan, el impl de cada feature. En
`docs/`, al lado del PRD y el SDD, no estan. Cada feature dejo su documentacion
en una rama aparte del repo docs que nadie mergeo. Pregunta: "¿por que esta
haciendo esto y no lo mantiene en docs con su PRD y SDD?"

DESPUES: los documentos de cada feature aparecen en `docs/` en cuanto se
escriben. No hay `docs-wt/`, no hay ramas del repo docs, no hay merge pendiente.
El worktree sigue existiendo para lo que era: implementar features en paralelo.

## Lo que se midio (realestate, 2026-09-06)

```
docs/          dab5879 [main]
docs-wt/131-…  02d4314 [feature/131-…]   SIN MERGEAR
docs-wt/132-…  dab5879 [feature/132-…]   SIN MERGEAR
docs-wt/142-…  dab5879 [bugfix/142-…]    SIN MERGEAR
```

Los artefactos de las tres estan SIN COMMITEAR en su worktree (`??`): ni
siquiera llegaron a la rama. En `docs/` solo esta el spec de la 142.

## De donde salio

De la OBS-5 de la #72 —"el arnes crea el worktree de docs"— mas la #71, que
decidio que el cierre no integra el repo docs porque es del usuario. La #72
resolvia un problema real (el `docs/` VACIO dentro del worktree del repo
principal, que fue la excusa de la #98 para correr `--sin-worktree`), pero lo
resolvio con una pieza de mas: un worktree del repo docs. El precio es que la
documentacion queda en una rama que nadie va a mergear.

**Decision del usuario (2026-09-06): se revierte la OBS-5.** "El worktree era
para poder implementar las feat en paralelo, no para crear otro docs-wt."

## El modelo correcto

Con `docs/` como repo aparte, los documentos de la feature van **directo a
`<raiz>/docs/`**. Sin worktree, sin rama. Es lo mismo que ya hacen el PRD, el
SDD, `architecture.md` y el sello de cierre (#71): los documentos compartidos
viven en la raiz.

¿Chocan dos features en paralelo? No: sus artefactos tienen nombre por feature
(`spec-feature-131-…`, `impl-132.md`). Lo unico compartido es lo que ya iba a
la raiz. El worktree del repo principal sigue existiendo para el codigo, que si
choca.

Con `docs/` como parte del repo principal **nada cambia**: ahi los documentos
viajan en el worktree y la rama de la feature, y el merge los trae.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given `docs/` como repo git aparte, When se arranca una feature, Then su
  spec y su plan se escriben en `<raiz>/docs/`, y NO existe `docs-wt/`.
  Comando: `cd rust && cargo test --locked --test cli_basics docs_repo`
- AC-2: Given ese mismo layout, When se arranca, Then el backlog NO lleva
  `docs_worktree`, y el repo docs no gana ninguna rama.
- AC-3: Given una feature arrancada ANTES de esta correccion, con
  `docs_worktree` en el backlog, When cualquier comando resuelve sus rutas,
  Then ese campo se ignora y la feature lee y escribe en `<raiz>/docs/`.
- AC-4: Given un cierre, When integra, Then no commitea ni menciona ningun
  worktree de docs.
- AC-5: Given `docs/` como parte del repo principal, When se arranca y se
  cierra una feature, Then los documentos siguen viajando en el worktree y la
  rama, exactamente como hoy.
- AC-6: Given dos features en paralelo con `docs/` aparte, When las dos
  escriben, Then las dos terminan en `docs/` sin pisarse.
- AC-7: Given el test de la #72 que afirmaba el worktree de docs, When se
  aplica esta correccion, Then se reescribe contra la regla nueva; no se borra.
- AC-8: Given el cambio completo, When se corre la suite, Then quedan verdes
  los tests, clippy, el smoke y el gate de paridad.
  Comando: `cd rust && cargo test --locked`
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`
  Comando: `bash tests/parity_check.sh`
- AC-9 (MANUAL): Given los tres `docs-wt/` de realestate, When se pasan sus
  artefactos a `docs/` con los comandos, Then quedan junto al PRD y el SDD y
  los worktrees se retiran. Esto NO lo hace el binario.

## Los datos que se tocan

- `docs_worktree` en el backlog: deja de escribirse y se ignora al leer.
- `HarnessPaths::para_feature`: con docs aparte, `plans = <raiz>/docs`.

## Pseudo-codigo (el acuerdo)

```
RESOLVER docs/ DE UNA FEATURE:
  docs/ es repo aparte      -> <raiz>/docs        (compartido, como el PRD)
  docs/ es del repo principal, hay worktree -> <worktree>/docs (viaja en la rama)
  sin worktree              -> <raiz>/docs
```

Promesa: los documentos de una feature estan en `docs/` desde que se escriben,
y `docs-wt/` no vuelve a existir.

## No funcionales y verificacion

- Tests de comportamiento con fixture de repo docs aparte, incluido el de dos
  features en paralelo.
- Prueba del rojo: cada test nuevo falla contra el codigo actual.
- Compatibilidad: el fixture de la #72 (`sandbox_git_con_docs_aparte`) se
  reusa; el test que afirmaba el worktree se reescribe.

## Alcance de instalacion y fuera de alcance

Se corrige `harness_process` y se actualiza realestate con su comando. Mover lo
que ya esta en `docs-wt/` se hace con comandos de shell, no desde el binario.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por el usuario 2026-09-06): directo en `docs/`, sin worktree.
- OBS-2: la rama de integracion se pregunta antes de `close --status done --to`.
