# Impl - Feature #83: el Stop hook bloquea por graphify-out/.graphify_stale

Spec: docs/spec-feature-83-el-stop-hook-bloquea-por-graphify-out-graphify-s.md
Plan: docs/plan-feature-83-el-stop-hook-bloquea-por-graphify-out-graphify-s.md

## Lo que estaba pasando (realestate, 2026-09-06 22:40)

El Stop hook mostraba `[!] graphify-out/.graphify_stale existe; corre /graphify
--update cuando aplique.` y lo contaba como fallo. El marcador lo deja el propio
arnes: el hook `post-commit` (init.sh:162) tras cada commit que toca un `.md`
—cada cierre—, y lo borra solo si logra el rebuild semantico (init.sh:194),
que esta debounced a 30 minutos y salta sin backend LLM; `autocheck`
(rust/src/graphify.rs:56) lo deja cuando `graphify update` falla. El agente
quedaba bloqueado hasta un `/graphify --update` a mano.

## El arreglo: avisar, no bloquear

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | harness_check.sh:142 · tests/graphify_stale_check.sh:30 | El bloque ya no llama a `sumar_fallo`; con el marcador y nada mas, el check da rc 0. El test lo mide en un fixture instalado por el instalador: primero sin marcador (rc 0 y sin mencion), despues con el marcador (`[i]` y rc 0). |
| AC-2 | harness_check.sh:143 · tests/graphify_stale_check.sh:35 | El aviso nombra al hook post-commit y al rebuild semantico, a `harness autocheck`, `/graphify --update` para forzarlo, y termina en "No bloquea". El test exige las tres cosas. |
| AC-3 | templates/harness_check.sh:142 | `cmp` limpio: el instalador copia desde `templates/`, asi que el fix vive en los dos lados. |
| AC-4 | tests/setup_smoke.sh:1713 | Enganchado al smoke despues del bloque de la #80. Rojo: contra el check de HEAD el test cae con `con el marcador no sale el [i] ... primer [!]: [!] graphify-out/.graphify_stale existe`. Verde con el fix. |
| AC-5 | UPDATING.md:48 | Seccion nueva en las dos copias (`cmp` limpio): que es el marcador, quien lo deja, por que bloqueaba y que cambia. |

## El rojo

`HARNESS_PREBUILT_BIN=... bash tests/graphify_stale_check.sh` con el
`harness_check.sh` de HEAD: `[!] graphify_stale: con el marcador no sale el [i]
(stderr: 3 lineas; primer [!]: [!] graphify-out/.graphify_stale existe; corre
/graphify --update cuando aplique.)`, rc 1. Cayo por el aserto del `[i]`, no por
una precondicion: el fixture pasaba limpio sin el marcador antes de plantarlo.

## El rojo que no era rojo (otra vez)

La primera corrida de `verify` marco AC-1 y AC-2 en rojo con `falta el binario
(HARNESS_PREBUILT_BIN o rust/target/debug/harness)`: en el worktree de la #83 no
hay binario compilado porque la feature no toca Rust. Una precondicion, no el
aserto (leccion #78, referencia "el rojo que fallo antes de llegar al aserto").
Arreglo en los dos tests de fixture (`tests/graphify_stale_check.sh:10-20` y
`tests/leccion_tope_check.sh`): sin binario propio, toman el del checkout
principal via `git rev-parse --git-common-dir`, y si tampoco esta, compilan.
Corridos desde el worktree sin `HARNESS_PREBUILT_BIN`: los dos verdes.

## Lo que NO hace

- No toca `init.sh` ni `graphify.rs`: el marcador se sigue creando y limpiando
  igual; solo deja de contar como fallo del proceso.
- No toca el guard: los "cambios sin commitear en docs" de la misma captura son
  archivos ajenos al arnes en el repo `docs` de realestate (scripts de
  publicacion a Confluence de otra sesion) y el guard hace bien en nombrarlos.

## Suite

- `tests/graphify_stale_check.sh`: verde. `tests/stop_hook_check.sh`: 10/10.
  `tests/parity_check.sh`: 10/10.
- `harness verify --feature 83`: corre los cinco AC, incluido el smoke completo
  (ver `docs/verify-83.md`).
- Sin cambios en Rust: la suite de la #80 (476 + 268, clippy limpio) es la
  vigente.
