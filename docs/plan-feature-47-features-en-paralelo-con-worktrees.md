# Plan - Feature #47: features_en_paralelo_con_worktrees

Estado: in_progress
Microservicios:
- harness

## Alcance

Poder implementar varias features a la vez sin que se pisen. Entra: quitar el
bloqueo de una-a-la-vez, rama + worktree por feature (GitFlow), estado vivo por
feature con `current.md` como indice, resolucion del estado contra el repo
principal desde cualquier worktree, foco automatico por carpeta, y el cierre
que exige `--to`, mergea, pushea sin trailers de IA, borra el worktree y
conserva la rama. No entra: detectar solapamientos, crear `develop`, PRs,
rebase/squash.

## Impacto entre microservicios

Un solo microservicio: `harness`. El cambio toca el nucleo del flujo (`start`,
`close`, estado vivo), asi que el riesgo esta en la compatibilidad hacia atras:
un repo sin git, sin worktrees o con backlog viejo tiene que seguir andando
igual (AC-5, AC-6). Los campos `branch` y `worktree` son opcionales.

## Consulta al grafo (graphify)

Alcance acotado a rutas conocidas: `rust/src/commands/{start,close,advance,
approve_spec}.rs`, `rust/src/{paths,features,progress}.rs` y un modulo nuevo
para git/worktrees.

## Delegacion (implementer)

- D1 [AC-7]: resolucion del repo principal desde un worktree
  (`git rev-parse --git-common-dir`) en `paths.rs`, para que el estado no se
  bifurque. Es la base de todo lo demas: va primero.
- D2 [AC-8, AC-9, AC-10, AC-11]: estado vivo por feature
  (`progress/current-<id>.md`), `current.md` como indice de activas y stamp de
  autocheck por feature; `close` deja de tocar el estado ajeno.
- D3 [AC-1, AC-6]: quitar el rechazo de la segunda feature in_progress y sumar
  `--sin-worktree`; `one_feature_at_a_time` deja de bloquear (OBS-10).
- D4 [AC-2, AC-3, AC-4, AC-5, AC-22, AC-23]: modulo `git` del arnes — crear o
  reusar la rama desde la base, crear o reusar el worktree hermano, degradar con
  aviso si no hay repo, y la configuracion de prefijos y rama base.
- D5 [AC-12, AC-13]: foco por worktree (mapa worktree -> feature y resolucion
  por la carpeta actual), conservando el `--feature` explicito.
- D6 [AC-14, AC-15, AC-16, AC-17, AC-18, AC-19, AC-20, AC-21]: cierre GitFlow —
  `--to` obligatorio para `done`, validacion de la rama destino, merge sin
  trailers de IA, push, abort limpio ante conflicto, borrado del worktree y
  conservacion de la rama.
- D7 [AC-24]: tests — arranque en paralelo, estado por feature, cierre sin
  `--to`, merge exitoso, merge con conflicto, sin git, y `--sin-worktree`.
- D8 [AC-25]: verificacion real en este repo con dos features abiertas a la vez.
- D9 [AC-1..AC-25]: documentacion — README, UPDATING.md (+ espejo en
  `templates/`), superficies de agentes y guia del flujo en paralelo.

## Criterios de cierre (reviewer)

- Evidencia por AC-n en `docs/impl-47.md`.
- `cargo test`, `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh` limpios.
- Dos features abiertas a la vez y una cerrada sin tocar a la otra (AC-25).
- Constitution: Articulo 2 (spec approved antes de implementar), Articulo 4
  (nunca `push --force`, nunca borrar ramas, merge abortado ante conflicto),
  Articulo 6 (sin dependencias nuevas; espejos `templates/` propagados; commits
  sin trailers de IA).

## Riesgos

- R1: tocar `start`/`close` es tocar el corazon del flujo; una regresion afecta
  a todos los repos instalados. Mitigacion: todo lo nuevo degrada al
  comportamiento actual (sin git, `--sin-worktree`, campos opcionales) y hay
  tests para cada camino viejo.
- R2: el merge automatico puede ensuciar la rama destino. Mitigacion: `--to`
  obligatorio, validacion previa de la rama, abort ante el primer conflicto y
  cero reescritura de historia.
- R3: el push automatico publica. Mitigacion: solo la rama destino que el
  USUARIO indico, nunca `--force`, y el cierre solo llega ahi con los gates en
  verde.
- R4: worktrees huerfanos si alguien borra la carpeta a mano. Mitigacion:
  `start` reusa lo que existe y el arnes limpia la referencia al cerrar
  (`git worktree prune`).
- R5: el estado podria bifurcarse si algun camino lee el `progress/` del
  worktree. Mitigacion: D1 va primero y hay test explicito (AC-7).

## Observaciones (decisiones pendientes)

- OBS-1 a OBS-9 [DECIDIDAS 2026-08-21]: ver el spec (GitFlow, main en este repo,
  estado por feature, foco por worktree, merge + push sin trailers, borrar
  worktree y conservar rama, worktrees hermanos, sin deteccion de solapamientos,
  y la #45 cerrada como superseded por esta).
- OBS-10 [REGISTRADA]: `one_feature_at_a_time` se conserva como clave del
  backlog pero deja de bloquear.

### Avance 2026-08-21T22:35:43Z
D1-D9 implementados: modulo git (ramas GitFlow, worktrees, merge en worktree temporal, push, commit sin trailers), rutas por feature (para_feature/current_de/autocheck_stamp_de), start en paralelo con aislamiento previo a los docs, foco por worktree, cierre con --to que integra y limpia, docs en README/UPDATING/superficies. 515 tests + clippy limpio; verificado en real con las features #47 y #48 abiertas a la vez

---
Cerrado: 2026-08-22T00:02:01Z - status=done - Features en paralelo: rama y worktree por feature, estado unico con vivo por feature, foco por carpeta y cierre GitFlow que integra, publica y limpia
