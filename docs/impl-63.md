# Impl - Feature #63: el_arnes_no_afirma_lo_que_no_puede_comprobar

Spec: docs/spec-feature-63-el-arnes-no-afirma-lo-que-no-puede-comprobar.md

## (1) El test que salia verde sin medir

`tests/commit_guard_check.sh` decidia si un script se colgaba cortandolo con
`timeout 10`. `timeout(1)` es de coreutils y **no viene en macOS**. Medido en
esta maquina: no hay `timeout` ni `gtimeout`; el subshell devuelve **127**, y
como el test solo consideraba colgado el **124**:

| Modo | Antes en macOS | Por que |
| --- | --- | --- |
| `no-cuelga` | VERDE siempre | 127 != 124 -> "termino" |
| `prueba-del-rojo` | ROJO siempre | esperaba 124 y recibia 127 |

O sea: el modo que verifica salia verde sin medir, y el que existe para avisar
de eso fallaba — y se leia como ruido de un test viejo. Es exactamente la
leccion `criterios-de-cierre-que-se-pueden-fallar`.

Y habia un segundo defecto encima: la prueba del rojo reconstruia **una sola**
de las dos defensas contra el cuelgue. La #52 le cerro la entrada al guard en
la invocacion (`</dev/null`) y la #53 le puso al guard su propia guarda
(`[ -t 0 ]`). Revertir solo la primera dejaba la segunda en pie, asi que el rojo
no aparecia ni con `timeout` disponible.

Arreglo:

- `mecanismo_de_limite()` elige `timeout`, `gtimeout` o `perl` — alguno hay en
  macOS y en Linux. Sin ninguno **falla** nombrando cual instalar.
- `con_limite()` traduce el codigo del mecanismo (124 de `timeout`, 142 de
  SIGALRM) a "se colgo" / "termino". La variante perl vigila a un hijo y sale
  ella normalmente: si se dejara matar por la senal, el shell imprimiria un
  `Alarm clock` que ensucia la salida.
- Modo nuevo `limite`: prueba el mecanismo contra un caso que se cuelga y uno
  que no, y ademas comprueba que con el PATH vacio la deteccion FALLA.
- `modo_prueba_del_rojo` ahora revierte **las dos** defensas.

## (2) La ruta que nombraba un worktree ya borrado

`close` imprimia `Estado archivado en ../<repo>-wt/<id>-<slug>/docs/estado-feature-<id>.md`.
Ese worktree lo borra el propio cierre con `--force` unas lineas antes de
imprimir el mensaje; el archivo, despues del merge, vive en
`docs/estado-feature-<id>-<slug>.md` de la raiz. Mismo defecto que la #92
arreglo para los punteros del PRD, sobreviviendo en un mensaje de consola.

Arreglo: `ruta_del_estado_archivado(rel_real, canonica, borra_el_worktree)`,
funcion PURA, y `PlanDeIntegracion::borra_el_worktree()` para saber cual
corresponde con lo que el cierre YA sabe, sin consultar el disco.

## Evidencia por AC

- **AC-1**: `bash tests/commit_guard_check.sh` -> los SEIS modos verdes en esta
  Mac, sin `timeout(1)`.
- **AC-2**: modo `limite`: con el mecanismo elegido, `sleep 0` se reporta como
  terminado y `sleep 30` con limite 1 como cortado.
- **AC-3**: el mismo modo comprueba que con `PATH=/nonexistent` la deteccion
  falla, asi que quien la llama se detiene. Nunca hay skip verde.
- **AC-4**: `bash tests/commit_guard_check.sh prueba-del-rojo` -> la version
  previa (las DOS defensas revertidas) se cuelga, y el corte lo demuestra.
- **AC-5**: `cargo test estado_archivado_apunta_a_donde_quedo_el_archivo`. El
  stdout dice `Estado archivado en docs/estado-feature-1-cobranza.md`, no
  contiene `Estado archivado en ../`, y `git show main:docs/estado-feature-1-cobranza.md`
  confirma que el archivo esta donde la ruta promete.
- **AC-6**: `cargo test estado_archivado_sin_integrar_mantiene_la_ruta_real`.
  Con `pending` el worktree sigue vivo y el archivo esta ahi.
- **AC-7**: `cargo test ruta_del_estado_archivado_es_pura`. Los dos casos, sin
  tocar el disco, y la ruta post-integracion nunca contiene `..`.

## Lo que NO se toco

Ningun otro test de `tests/` usa `timeout` (se reviso: la unica otra mencion es
un comentario en `setup_smoke.sh`). El comportamiento del cierre no cambia:
cambia el TEXTO que informa donde quedo el estado archivado.
