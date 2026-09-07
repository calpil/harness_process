# Impl - Feature #80: el autoaprendizaje no tiene ciclo de vida

Spec: docs/spec-feature-80-el-autoaprendizaje-no-tiene-ciclo-de-vida-leccio.md
Plan: docs/plan-feature-80-el-autoaprendizaje-no-tiene-ciclo-de-vida-leccio.md

## Lo que estaba pasando (este repo, 2026-09-06)

Las seis piezas del loop de aprendizaje (#17, #18, #19, #20, #21/#28, #22)
existian y corrian. Lo que no existia era el ciclo de vida de lo aprendido:
10 de los ultimos 15 cierres declararon la misma leccion, que tenia 442 lineas
y nueve secciones "(feature #N)"; el paso 3 de la guia (`<clase>/referencias/`)
tenia cero usos; 340 decisiones sin incorporar al perfil; la consolidacion sin
correr en 19 dias y sin registro. `require_leccion` media que se DECLARE una
leccion, no que se aprenda.

## El arreglo: un limite fisico, una pregunta y dos avisos

Cuatro umbrales en `rules` (`Politica`, rust/src/lecciones.rs:468), con los
defaults que el usuario decidio (250 / 3 / 25 / 30) y `0` para apagar cada uno.

## Evidencia por AC

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | rust/src/lecciones.rs:981 · :533 | El gate carga la leccion y, sobre el tope (`sobre_el_tope`, :513), devuelve exit 2 con `contrato_de_particion`: lineas, tope, las secciones `## ... (feature #N)` (`secciones_por_feature`, :520) y la ruta `docs/lecciones/<clase>/referencias/<tema>.md`. Sin `--leccion-motivo` como escape (OBS-4). Test rust/tests/cli_basics.rs:8479: rechazo con las dos secciones nombradas, backlog intacto, y con el tope en 0 cierra. |
| AC-2 | rust/src/commands/leccion.rs:185 · :360 | `leccion usar` aplica el mismo contrato antes de `registrar_uso`; `lecciones status` imprime `lineas/tope lineas` por fila y ` SOBRE EL TOPE` cuando corresponde (JSON: `lineas`, `tope_lineas`, `sobre_el_tope`). Test rust/tests/cli_basics.rs:8529: exit 2, `usos: 1` intacto, `/20 lineas` y `SOBRE EL TOPE` en status, y la corta se usa normal. |
| AC-3 | rust/src/lecciones.rs:992 · :574 · :594 | `racha` lee `history.md` con `cierre_declarado` (solo cierres `done` con `leccion=` distinta de `ninguna`); si los ultimos K son la misma clase y no hay motivo, exit 2 nombrando los `#id`. Con motivo, `Declaracion.motivo` se registra en la feature y en history (`leccion=x (motivo)`), y `close` lo imprime (rust/src/commands/close.rs:388). Test rust/tests/cli_basics.rs:8562: racha [x, ninguna, x] con K=2 -> rechaza nombrando #5 y #7; con motivo cierra; con K=9 no pide nada. |
| AC-4 | rust/src/lecciones.rs:634 · :624 · rust/src/commands/close.rs:406 | `texto_avisos_de_ciclo` cuenta `perfil::recolectar` sin `ya_incorporado`; sobre el umbral, aviso por stderr con `perfil sugerir`. Se emite al final del cierre `done`, con las rutas de la RAIZ, sin tocar stdout ni exit. Test rust/tests/cli_basics.rs:8645: 4 decisiones con umbral 2 -> aviso; umbral 0 -> sin aviso; stdout igual. |
| AC-5 | rust/src/lecciones.rs:613 · rust/src/commands/leccion.rs:432 · :734 | `curar` (informe, o aplicar sin nada que aplicar) y `consolidar` (informe) registran `lecciones curar informe: N ...` / `lecciones consolidar informe: N candidato(s)` en `history.md`; `ultima_consolidacion` toma la ultima; el cierre avisa "nunca registrada" o los dias sobre el umbral; `status` lo muestra (:397). Test rust/tests/cli_basics.rs:8704: aviso, `status` lo dice, `curar` lo registra y el aviso deja de salir. |
| AC-6 | harness_check.sh:392-397 · :434 · tests/leccion_tope_check.sh | El check lee `leccion_max_lineas` del backlog (default 250) y avisa `[i]` por leccion activa sobre el tope; `archivo/` y `referencias/` no cuentan. `templates/harness_check.sh` es copia identica (el instalador copia desde ahi). Docs: UPDATING.md:48 (las dos copias), guia docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md:91 (las dos copias), README.md:720, AGENTS.md y su fuente en setup_harness.sh / setup_harness.ps1, templates/roles/leader.md:163 y sus espejos. |
| AC-7 | docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md · docs/lecciones/promesas-estructurales-vs-disciplina.md | MANUAL. Ver "La biblioteca de este repo". |
| AC-8 | rust/src/verificacion.rs:1267 · tests/setup_smoke.sh:1708 | Suite, clippy, smoke (con `leccion_tope_check` enganchado), stop_hook, paridad. Ver "Suite". |

## El rojo

Los cinco tests se corrieron contra el binario ANTES del fix. Cayeron los cinco
por su aserto —`code(2)` esperado y el cierre dio 0; `stderr` sin "sin
incorporar al perfil"; `stderr` sin "nunca registrada"—, no por una
precondicion (leccion #78). Con el fix: 5/5.

Mutaciones (reviewer): `if false && sobre_el_tope` en el gate tumba
`tope_de_lineas`; dejar que `ninguna` cuente como clase en `cierre_declarado`
tumba `repeticiones`. Las dos con `cmp` antes y despues, archivo restaurado y
`touch`.

## Lo que el fix rompio y el test atrapo

- La primera version del `[i]` en `harness_check.sh` hacia `lec_tope="$(grep
  ... | grep ...)"`: sin la regla en `rules`, el `grep` devuelve 1 y bajo `set
  -Eeuo pipefail` mataba el check ENTERO, en cualquier proyecto sin la regla.
  Lo atrapo `tests/stop_hook_check.sh` ("la primera vuelta no bloqueo"). Arreglo:
  `|| true` (harness_check.sh:396).
- El fixture del test de AC-2 tenia la leccion "corta" con 13 lineas y el tope
  en 12: el test fallaba por su propio fixture. Tope a 20.
- El test de la #28 `consolidar_should_report_mutual_relacionadas_without_a_backend`
  afirmaba que el modo informe no creaba `history.md`. Esa promesa cambio a
  proposito (AC-5): la deteccion sigue sin tocar una leccion ni crear backup,
  pero registra su corrida. El test lo dice (rust/tests/cli_basics.rs:4515).
- `templates/roles/leader.md` tiene espejos (`roles/leader.md`,
  `.claude/agents/leader.md`); la regla de espejo del Articulo 6 los exige en el
  mismo commit y el smoke lo detecto dos veces.

## La biblioteca de este repo (AC-7)

- `criterios-de-cierre-que-se-pueden-fallar`: 442 -> 186 lineas. Nueve
  secciones movidas, sin reescribir, a
  `criterios-de-cierre-que-se-pueden-fallar/referencias/` (el PIPE, la
  herramienta externa, el oraculo copiado, el recorrido P1, el nombre del
  test, la regla ancha, la pieza de mas, el arnes que miente, el rojo que
  fallo antes); una seccion "Referencias" con un puntero por archivo.
- `promesas-estructurales-vs-disciplina`: 316 -> 167 lineas. Tres secciones a
  `referencias/` (el ORDEN, aplicado a ARREGLAR un bug, el DESTINO IMPLICITO).
- OBS-7: no se fusiono nada; el trigger compartido `falso verde` se quito de
  `promesas-estructurales` (era la unica de las otras dos que lo tenia).
- `lecciones status` al cerrar: ninguna activa sobre 250 (la mayor,
  `probar-contra-datos-reales`, 240).
- Perfil: cuatro entradas aprobadas una por una por el usuario y escritas con
  `perfil add --yes` (950/1500 caracteres).
- `lecciones curar` y `lecciones consolidar` corren con el binario nuevo
  despues del cierre (quedan registrados en `history.md`); el informe de
  consolidacion de hoy con el binario viejo no dejo rastro, que es justo el
  problema que AC-5 arregla.
- La leccion declarada al cerrar es `reglas-que-se-aplican-a-si-mismas`, con
  una seccion nueva sobre esta feature: la regla de aprendizaje que el arnes
  no se aplico a si mismo.

## Suite

- Rust: 476 + 268 tests, 0 fallos. Clippy `-D warnings`: exit 0 (dos lints
  corregidos en `ultima_consolidacion`: `last` -> `next_back` -> `rfind`).
- `harness verify --feature 80`: 8 verdes, 0 rojos, 1 manual.
- `tests/parity_check.sh`: 10/10. `tests/stop_hook_check.sh`: 10/10.
  `tests/leccion_tope_check.sh`: verde.
- `tests/setup_smoke.sh`: verde con el bloque nuevo, tras espejar
  `roles/leader.md` y `.claude/agents/leader.md`.
