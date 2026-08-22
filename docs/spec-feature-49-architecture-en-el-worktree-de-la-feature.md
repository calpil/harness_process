# Spec - Feature #49: architecture_en_el_worktree_de_la_feature

Estado: approved
Aprobado: 2026-08-22T12:03:23Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #49 en el chat (6 AC): architecture.md se resuelve contra el docs/ de la feature igual que el PRD y el SDD, con test que impide la regresion
Plan: docs/plan-feature-49-architecture-en-el-worktree-de-la-feature.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan cierra la feature #47 desde su worktree. El arnes le pide poner los
documentos al dia, el contesta los tres bloques y `prd apply` escribe. El PRD y
el SDD quedan dentro del worktree — en la rama de la feature — y viajan con el
merge, como corresponde. Pero `architecture.md` no: se escribe en el checkout
principal, fuera de la rama. Resultado: el merge trae dos de los tres
documentos, y el tercero queda suelto, sin commitear, en `main`. Si Alan no
mira `git status` en ese momento, el cambio se pierde entre otros pendientes o
lo commitea alguien despues sin saber de donde salio.

DESPUES: los tres documentos se escriben en el mismo lugar — el `docs/` de la
feature — y los tres viajan con el merge de su rama. El cierre deja el arbol
principal como lo encontro.

## Hoy -> Como va a funcionar

```
HOY                                          DESPUES
prd apply (desde el worktree de la #47)      prd apply (desde el worktree)
  |__ PRD  -> <worktree>/docs/prd/...  OK      |__ PRD  -> <worktree>/docs/prd/...
  |__ SDD  -> <worktree>/docs/prd/...  OK      |__ SDD  -> <worktree>/docs/prd/...
  |__ architecture.md -> <principal>/docs/     |__ architecture.md -> <worktree>/docs/
      (queda fuera de la rama)                     (viaja con el merge)
```

Una sola linea: `documentos::alcance()` arma la ruta de `architecture.md` con
`paths.repo_root` mientras que el PRD y el SDD usan `paths.plans` — que es el
que respeta el worktree de la feature (feature #47).

## Recorridos de usuario (priorizados)

- P1: Como Alan cerrando una feature desde su worktree, quiero que los tres
  documentos queden en la rama, para que el merge se lleve todo y el checkout
  principal no acumule cambios sueltos.
- P2: Como el proximo que toque `documentos.rs`, quiero que un test falle si
  alguien vuelve a armar esa ruta contra la raiz, para que la deuda no vuelva.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una feature con worktree, When corro `prd propose`, Then el bloque
  de `docs/architecture.md` apunta al archivo del worktree de esa feature (el
  mismo `docs/` donde estan su PRD y su SDD).
- AC-2: Given esa propuesta contestada, When corro `prd apply --yes`, Then el
  cambio se escribe en el `architecture.md` del worktree y el del checkout
  principal NO se toca.
- AC-3: Given una feature SIN worktree (modo clasico, `--sin-worktree`, o repo
  sin git), When corro `prd propose` o `prd apply`, Then `architecture.md` se
  resuelve como siempre, contra el `docs/` de la raiz: cero regresion.
- AC-4: Given el codigo de `documentos::alcance()`, When alguien vuelva a armar
  la ruta de `architecture.md` contra `repo_root` en vez de `plans`, Then un
  test lo detecta y falla.
- AC-5: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh`, Then los cuatro terminan limpios.
- AC-6: Given el cierre de ESTA feature desde su worktree, When se integra a
  `main`, Then los tres documentos del alcance llegan por el merge y el arbol
  principal no queda con cambios sueltos: la prueba es el propio cierre.

## Los datos que se tocan

- disparador: `prd propose` y `prd apply` (via `documentos::alcance`).
- interruptor: ninguno nuevo; el comportamiento sigue al worktree de la feature,
  que ya decide donde viven el PRD y el SDD desde la feature #47.
- candado: ninguno nuevo.
- Ningun formato cambia: `docs/prd-diff-<id>.md` sigue nombrando el documento
  como `docs/architecture.md` (la ruta relativa es la etiqueta, no el destino).

## Pseudo-codigo (el acuerdo)

```
CUANDO se arma el alcance de documentos de una feature

  el PRD y el SDD ya salen del docs/ de esa feature

  ENTONCES architecture.md sale del MISMO docs/, no de la raiz,
           con la restriccion de que sin worktree ese docs/ ES el de la raiz,
           asi que el modo clasico no cambia en nada.
```

Promesas: los tres documentos viajan juntos · sin worktree, cero cambios · una
linea de codigo y un test que la sostiene.

## No funcionales

- SLOs: sin impacto (es la construccion de una ruta).
- Seguridad: sin cambios; no toca permisos ni escribe fuera del arbol.
- Observabilidad: el bloque del diff ya imprime la ruta relativa; no cambia.

## Fuera de alcance

- Revisar otras rutas del arnes que usen `repo_root`: la auditoria mostro que
  `architecture.md` es el UNICO caso en codigo de produccion (los otros dos son
  un test y el fallback correcto de `paths.rs`).
- Mover el estado (`feature_list.json`, `progress/`) al worktree: es unico a
  proposito y asi tiene que seguir.

## Observaciones (decisiones pendientes)

- OBS-1 [REGISTRADA]: la deuda la dejo la feature #47 y la encontro su propia
  verificacion de cierre, no un test. El AC-4 existe para que la proxima vez la
  encuentre un test.
