# Veredicto del reviewer - Feature #47: features_en_paralelo_con_worktrees

Veredicto: **approved**
Fecha: 2026-08-21
Spec: `docs/spec-feature-47-features-en-paralelo-con-worktrees.md` (approved, 25 AC)
Evidencia: `docs/impl-47.md`

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | 345 unit + 170 integracion = **515 en verde** |
| `cargo clippy --all-targets -- -D warnings` | limpio |
| `bash tests/setup_smoke.sh` | exit 0 |
| `./harness_check.sh` | limpio |

## Cobertura de los AC

24 de 25 completos; AC-23 (prefijos configurables por repo) queda **parcial**:
los prefijos GitFlow son constantes del modulo y la rama base se elige por
existencia (`develop` y si no `main`), que es lo que el spec pide como default,
pero no hay forma de cambiarlos por configuracion. Es la unica deuda declarada.

Trece AC se verificaron **en el repo real** con las features #47 y #48 abiertas
a la vez, no solo en sandbox: arranque en paralelo, ramas y worktrees, estado
por feature, indice, foco por carpeta, exigencia de `--feature` desde el
principal, y el cierre `pending` conservando todo.

Lo mas importante: **AC-11 quedo demostrado sobre el repo de verdad** — cerrar
la #48 dejo el `current-47.md` byte a byte identico, con su stamp y su worktree
intactos. El bug que motivo la feature #45 ya no puede ocurrir: no hay un
archivo compartido que pisar.

## Constitution

- **Articulo 1**: tests nuevos junto al codigo tocado (10 unit del modulo `git`,
  6 de integracion con repos git reales) y los cuatro comandos oficiales en
  verde.
- **Articulo 2**: spec `approved` antes de implementar, con los tres puntos
  delicados (el cierre publica, `one_feature_at_a_time` deja de bloquear, en
  este repo todo va a `main`) confirmados explicitamente por el USUARIO.
- **Articulo 3**: D1..D9 citan sus AC-n; `impl-47.md` se organiza por AC.
- **Articulo 4**: sin `--force`, sin rebase, sin squash y sin borrar ramas. El
  merge corre en un worktree TEMPORAL, asi que no cambia la rama del checkout
  principal ni exige tener el arbol limpio. Un conflicto aborta y no deja nada a
  medias. Exit codes estables (2 = falta `--to`, 1 = fallo de integracion).
- **Articulo 5**: quince decisiones del USUARIO registradas (OBS-1..OBS-10 del
  spec), incluida la eleccion de GitFlow, el merge automatico con push, y que en
  este repo todo va a `main`.
- **Articulo 6**: sin dependencias nuevas (todo con el `git` del sistema);
  `templates/UPDATING.md` propagado; los commits del arnes no llevan trailers de
  IA, verificado por test.

## Reparos / observaciones del reviewer

1. **AC-23 parcial**: los prefijos (`feature/`, `bugfix/`) no son configurables
   por repo todavia. Un equipo con otra convencion de nombres tendria que
   tocarlos en el codigo. Deuda declarada, no bloqueante.
2. **Un bug de diseno propio, encontrado en la verificacion real**: la primera
   version resolvia `docs/` por el directorio actual, asi que el spec
   "desaparecia" segun desde donde miraras. Corregido con `para_feature()` y
   reordenando `start` (el worktree se crea antes que los docs). Vale como
   recordatorio de que el sandbox no alcanza: esto aparecio recien al usarlo
   sobre el repo de verdad.
3. **Ruido del gate de frescura**: mover un archivo cambia su `mtime` aunque el
   `hash` sea identico, y el gate lo reporta como "actualizado por otro LLM".
   Se resuelve con un `advance`, pero es ruido evitable: merece feature propia.
4. **AC-17 (push) sin verificar en sandbox**: los repos de test no tienen
   remoto, asi que el push solo se ejercita en el cierre real de esta feature.
5. **Worktrees huerfanos**: si alguien borra la carpeta a mano, `start` la
   recrea y el arnes hace `worktree prune` al cerrar; no hay limpieza periodica.
6. **La feature #48 quedo en `pending`** con su rama y su worktree: fue el
   segundo frente que se uso para probar el paralelismo. Se puede borrar cuando
   moleste (`git worktree remove` + `git branch -D`).
