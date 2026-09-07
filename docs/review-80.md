# Review - Feature #80: el autoaprendizaje no tiene ciclo de vida
Revisado: approved · 2026-09-07T00:23:36Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento, y la que escribio las diez secciones
"(feature #N)" que esta feature saca de la leccion mas usada. Metodo: correr los
cinco tests contra el binario de HEAD y leer POR QUE cayeron; dos mutantes con
`cmp`; `lecciones status` contra la biblioteca real; y el smoke completo, que
atrapo dos cosas que la lectura no.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/lecciones.rs:981 · rust/tests/cli_basics.rs:8479 | CUBIERTO. Rojo contra HEAD por `code(2)`; mutante `if false && sobre_el_tope` lo tumba. El mensaje nombra las dos secciones "(feature #N)" y la ruta `referencias/`. |
| AC-2 | rust/src/commands/leccion.rs:185 · rust/tests/cli_basics.rs:8529 | CUBIERTO. `usos` intacto tras el rechazo; `status` muestra `/20 lineas` y `SOBRE EL TOPE`. La primera version del test fallaba por su propio fixture (la "corta" tenia 13 lineas con tope 12): corregido el fixture, no el aserto. |
| AC-3 | rust/src/lecciones.rs:594 · rust/tests/cli_basics.rs:8562 | CUBIERTO. Racha [x, ninguna, x] con K=2 rechaza nombrando #5 y #7; mutante "ninguna cuenta como clase" lo tumba (la racha se corta y el cierre pasa). Con motivo, `feature_list.json` lleva `leccion_motivo` y history `leccion=x (motivo)`. |
| AC-4 | rust/src/lecciones.rs:634 · rust/tests/cli_basics.rs:8645 | CUBIERTO. stdout identico con y sin aviso; umbral 0 apaga. |
| AC-5 | rust/src/lecciones.rs:613 · rust/src/commands/leccion.rs:432 · rust/tests/cli_basics.rs:8704 | CUBIERTO. "nunca registrada" hasta que `curar` registra su informe; despues no sale. Cambia una promesa de la #28: el modo informe de `consolidar` ahora escribe UNA linea en `history.md` (no toca lecciones ni crea backup); el test de la #28 se actualizo diciendolo (rust/tests/cli_basics.rs:4515). |
| AC-6 | harness_check.sh:396 · :434 · tests/leccion_tope_check.sh | CUBIERTO con un cambio de comando aprobado por el usuario: `bash harness_check.sh` entero no corre limpio ni en un worktree ni en este checkout (`roles/` diverge de `templates/roles/` desde antes), asi que el `[i]` se prueba en un fixture instalado: avisa por la larga con el default 250, no por la corta, no por `referencias/`, y calla con la regla en 0 o en 400. Paridad 10/10. |
| AC-7 | docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md:174 · docs/perfil-usuario.md:15 | MANUAL, CUMPLIDO: 442 -> 186 y 316 -> 167 lineas, doce archivos en `referencias/` con puntero; OBS-7 sin fusion y sin el trigger compartido; cuatro entradas de perfil aprobadas una por una; curar y consolidar se corren con el binario nuevo despues del cierre (con el viejo no dejaban rastro). |
| AC-8 | rust/src/verificacion.rs:1267 · tests/setup_smoke.sh:1708 | CUBIERTO. 476 + 268 tests, clippy limpio, `verify` 8 verdes / 1 manual, stop_hook 10/10, smoke verde con `leccion_tope_check` adentro. |

## Lo que el review tiene que decir

1. **El check que mataba al check.** Mi primer `[i]` en `harness_check.sh` hacia
   `lec_tope="$(grep ... | grep ...)"`. En cualquier proyecto sin la regla el
   `grep` devuelve 1 y, bajo `set -Eeuo pipefail`, la asignacion aborta el
   script entero: el Stop hook dejaba de bloquear en TODOS los proyectos. No lo
   vi leyendo; lo vio `tests/stop_hook_check.sh`. Es la leccion
   `promesas-estructurales-vs-disciplina` al reves: una promesa estructural
   (el check bloquea) rota por una linea que no tenia nada que ver con ella.
2. **El tope me alcanzo a mi.** Con el binario nuevo, `close --leccion
   criterios-de-cierre-que-se-pueden-fallar` se habria negado (442 lineas) y
   `--leccion` de la misma clase por undecima vez habria pedido motivo. Esta
   feature se cierra con OTRA clase, `reglas-que-se-aplican-a-si-mismas`, que
   es exactamente de lo que trata: la regla de aprendizaje que el arnes no se
   aplico a si mismo.
3. **Los espejos.** `templates/roles/leader.md` tiene dos copias rastreadas
   (`roles/leader.md` y `.claude/agents/leader.md`, esta con frontmatter). Las
   edite de a una y el smoke fallo dos veces hasta que las tres coincidieron.
   La regla del Articulo 6 esta; lo que no hay es un comando que las propague.

## Riesgo declarado

- La racha y la ultima consolidacion se leen de `history.md`, local a cada
  instalacion. En una instalacion nueva arrancan en cero; en una con la
  bitacora borrada, tambien. Es la misma fuente que el contador del nudge.
- `texto_avisos_de_ciclo` corre `perfil::recolectar` en cada cierre `done`:
  lee la bitacora y todos los `spec-`/`plan-` de `docs/`. En este repo (80
  features) no se nota; se midio al cerrar, no en un repo de mil.
- El mensaje del tope lista las secciones "(feature #N)" por el titulo. Una
  leccion que narra casos sin ese sufijo recibe el contrato generico.

## Veredicto

Ocho AC con cobertura (siete ejecutables, uno manual cumplido y registrado).
Cinco tests nuevos rojos contra HEAD por su aserto, dos mutantes muertos, un
test viejo actualizado con su motivo. La biblioteca de este repo queda dentro
del tope y el perfil con las entradas que el usuario aprobo.
