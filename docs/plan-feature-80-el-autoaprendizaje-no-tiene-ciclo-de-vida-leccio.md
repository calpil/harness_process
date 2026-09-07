# Plan - Feature #80: el autoaprendizaje no tiene ciclo de vida

Spec: docs/spec-feature-80-el-autoaprendizaje-no-tiene-ciclo-de-vida-leccio.md
(approved 2026-09-06; OBS-1..5 y OBS-7 decididas: 250/3/25/30, tope duro,
partir a `referencias/`, no fusionar).

## Alcance

Cuatro piezas chicas sobre lo que ya existe, sin comando nuevo:
`lecciones::gate` gana el tope y la racha; `close` gana dos avisos por stderr;
`leccion usar`/`lecciones status`/`harness_check.sh` muestran el tope;
`lecciones curar|consolidar` registran su corrida. Mas la biblioteca de este
repo puesta dentro del tope (AC-7, manual).

## Peldano de huella

Extender comandos existentes con `rules` nuevas (peldano 1 de la escalera de
`docs/conventions.md`). Ninguna dependencia, ningun subcomando nuevo: `status`
ya existe, `usar` ya existe, el gate ya existe. La particion de lecciones es
contenido, no codigo: el arnes dice QUE partir y a donde.

## Delegacion (implementer)

- D-1 (tests PRIMERO, AC-1..5): cinco tests de integracion en
  `rust/tests/cli_basics.rs` cuyo nombre contiene `tope_de_lineas`,
  `usar_rechaza_sobre_el_tope`, `repeticiones`, `aviso_de_perfil`,
  `aviso_de_consolidacion` (son los `Comando:` del spec). Se corren antes del
  fix: tienen que caer por el ASERTO, no por una precondicion (leccion #78).
- D-2 (AC-1, AC-3): `lecciones::Politica::from_rules` (cuatro claves, defaults
  250/3/25/30, `<= 0` apaga), `Leccion::lineas()`, `secciones_por_feature()`,
  `contrato_de_particion()`, `racha()` sobre `history.md`; `gate()` aplica tope
  (duro) y racha (exige `--leccion-motivo`). `close` imprime la clase con su
  motivo (hoy asume que un motivo es de `ninguna`).
- D-3 (AC-2): `leccion usar` aplica el tope sin tocar `usos`; `lecciones
  status` muestra `lineas/tope` (texto y JSON) y marca las que lo superan.
- D-4 (AC-4, AC-5): `lecciones::avisos_de_ciclo(paths, data)` -> texto para
  stderr al final del cierre `done` (mismo lugar y canal que el contrato #18);
  `curar` y `consolidar` (informe y aplicar) registran `lecciones curar|consolidar
  <modo>` en `history.md`; `ultima_consolidacion()` lo lee; `status` lo muestra.
- D-5 (AC-6): `harness_check.sh` avisa `[i]` por leccion activa sobre el tope
  (lee `leccion_max_lineas` de `feature_list.json` con el mismo grep que usa
  para `prd`); docs: `UPDATING.md` (dos copias), la guia (dos copias),
  `AGENTS.md`, `README.md`, `templates/roles/leader.md`, `--help` del
  instalador si nombra `require_leccion`.
- D-6 (AC-7, manual, con el usuario): partir `criterios-de-cierre-que-se-
  pueden-fallar` y `promesas-estructurales-vs-disciplina` a `referencias/` con
  la particion de la OBS-5 (mover, no reescribir; puntero de una linea por
  archivo movido); quitar el trigger compartido `falso verde` de dos de las
  tres (OBS-7); correr `lecciones curar` y `lecciones consolidar` (quedan
  registrados); proponer entradas de perfil desde `perfil sugerir` y escribir
  SOLO las aprobadas, una por una.
- D-7 (AC-8): suite, clippy, smoke, stop_hook, paridad.

## Criterios de cierre (reviewer)

- Cada test nuevo cae contra el binario de HEAD por su aserto (mensaje leido,
  no exit code); luego pasa. Mutacion: desactivar el tope en `gate` tumba
  `tope_de_lineas`; ignorar `ninguna` en `racha` tumba `repeticiones`.
- Ningun aviso cambia stdout ni exit code: los tests de AC-4/5 lo afirman.
- `lecciones status` en este repo, al cerrar: ninguna activa sobre 250.

## Riesgos

- R-1: la racha se lee de `history.md`, que es local a la instalacion. Es la
  misma fuente que el contador del nudge; en una instalacion nueva la racha
  arranca en cero, que es lo correcto.
- R-2: el tope duro bloquea el cierre de ESTA feature hasta partir las dos
  lecciones. Es a proposito (P1 del spec) y esta en D-6.
- R-3: `perfil::recolectar` lee specs y planes de `docs/`; en un repo grande
  suma unos milisegundos al cierre. Se mide en el impl.

## Observaciones (decisiones pendientes)

- OBS-6: la rama de integracion se pregunta antes de `close --status done --to`.
- Las entradas del perfil (D-6) se aprueban una por una en el chat.

---
Cerrado: 2026-09-07T00:24:35Z - status=done - 
