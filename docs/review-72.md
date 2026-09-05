# Review - Feature #72: El paralelo aisla los cambios y acota los workflows
Revisado: approved · 2026-09-05T03:41:24Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento (no hubo delegacion: ver "Sobre la
delegacion" mas abajo). Metodo: atacar el propio trabajo con mutaciones sobre
produccion y fixtures git reales, no releer el diff.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/aislamiento.rs:148 | CUBIERTO. `decidir` es pura y rechaza los cuatro casos; `rust/src/commands/start.rs:145` la corre antes de escribir estado, y `rust/src/commands/start.rs:28` exige que el worktree declarado EXISTA. Probado en rojo: con el fallo de git de vuelta a `println!`, `start_should_refuse_when_git_fails_instead_of_warning_and_continuing` FALLA. |
| AC-2 | rust/src/git.rs:108 | CUBIERTO. `repo_de_docs` distingue el docs que es repo propio; `rust/src/paths.rs:54` lo prefiere sobre el docs vacio del worktree. Probado en rojo: ignorando `docs_worktree`, `start_should_give_a_separate_docs_repo_its_own_worktree` FALLA. |
| AC-3 | rust/src/git.rs:387 | CUBIERTO. El rango completo con los ajenos marcados; `rust/src/commands/close.rs:666` bloquea nombrandolos. El test `close_should_refuse_to_drag_another_features_commit` reproduce el incidente: un commit propio cuyo padre es de otra feature. |
| AC-3 | rust/src/git.rs:443 | CUBIERTO. Candado por destino (`el_candado_de_integracion_serializa_por_destino`), y `rust/src/commands/close.rs:776` deja el push detras de `--publicar`. |
| AC-4 | commit_guard.sh:157 | CUBIERTO. El guard resuelve el worktree de la sesion y exige que sea de ESTE proyecto; `commit_guard.sh:198` informa lo ajeno sin bloquear. Probado en rojo contra el `commit_guard.sh` de HEAD: falla con el texto exacto del bug diagnosticado. |
| AC-5 | roles/leader.md:95 | CUBIERTO COMO CONTRATO, no como enforcement. El arnes no lanza los agentes, asi que no puede imponer el tope de cuatro ni el orden de las etapas: lo que SI impone es el registro de resultados (AC-6). La distincion esta escrita en `roles/leader.md:118`, que declara los reintentos internos del runtime y la preferencia `small` como NO imponibles. |
| AC-6 | rust/src/revision.rs:543 | CUBIERTO. Las tareas no-`ok` bloquean `approved` en los dos lados (el gate del cierre y `estampar`), y la cuenta esperada impide que borrar los nulos complete la cobertura. Probado en rojo: sin la cuenta, `borrar_las_fallidas_no_completa_la_cobertura` FALLA. |
| AC-7 | roles/leader.md:124 | CUBIERTO, y aplicado a esta misma feature: el hallazgo de `verify` quedo en `progress/hallazgo-verify-un-solo-comando.md` sin abrir otra feature. |
| AC-8 | tests/parity_check.sh:1 | CUBIERTO CON SALVEDAD. Los diez modos de paridad verdes, espejo `.claude/agents/leader.md` regenerado (lo detecto `setup_smoke.sh`, no yo). SALVEDAD: `harness verify` corrio 1 de los 4 comandos del AC; los otros tres se corrieron a mano. Ver abajo. |
| AC-9 | progress/current-72.md:14 | CUBIERTO POR TRABAJO PREVIO, no por esta feature. La configuracion ya estaba aplicada con respaldo (`progress/current-72.md:15`). NO VERIFICADO: la confirmacion visual del recuadro en la interfaz, que ningun archivo puede probar. Queda pendiente y se dice. |
| AC-10 | rust/src/commands/status.rs:148 | CUBIERTO. El preflight inventaria y declara lo que no hace. Nada de realestate se toco: sus features siguen como estaban. |

## Lo que NO esta verificado, dicho entero

1. **AC-8, tres de cuatro comandos a mano.** `harness verify --feature 72`
   ejecuta un solo `Comando:` por AC y no avisa. Corridos a mano, todos verdes:
   `cargo clippy --all-targets --all-features --locked -- -D warnings` limpio,
   `bash tests/setup_smoke.sh` verde, `bash tests/parity_check.sh` diez modos.
   Que yo los haya corrido no es lo mismo que un reporte: por eso el hallazgo
   quedo anotado en `progress/`.
2. **AC-9, el recuadro.** Que el JSON diga `quiet` no prueba que el aviso
   desaparecio de la pantalla. Lo confirma Alan mirando, o no esta confirmado.
3. **AC-5, el tope de cuatro tareas.** Es contrato, no candado. El arnes no
   corre los agentes.
4. **El repo `docs/` no se integra solo.** Con docs como repo aparte, sus
   artefactos quedan commiteados en su rama y el cierre lo dice; mergearlos es
   decision del usuario.

## Sobre la delegacion

No hubo. Esta feature se implemento sin subagentes, a proposito: la evidencia de
esta misma sesion es que las sondas puntuales encontraron mas defectos que el
fan-out, y el costo lo paga el usuario. Por eso `docs/review-72.md` no lleva
lineas `Tarea:`: registrar tareas que no existieron para lucir cobertura seria
exactamente el fraude que el AC-6 vino a cerrar.

## Riesgos que el cambio introduce

- **`start` es mas estricto.** En un proyecto sin git, o con una feature abierta
  sin worktree, la segunda no arranca. Es la decision del usuario del
  2026-09-05 y REVOCA el AC-1 de la feature #47 para ese caso. Tres tests de la
  #47 se reescribieron contra la regla nueva, ninguno se borro.
- **`close` ya no publica solo.** Quien dependiera del push automatico tiene que
  agregar `--publicar`. Es el objetivo del AC-3.

## Veredicto

Los diez AC tienen cobertura, con las cuatro salvedades de arriba dichas en
lugar de disimuladas. Las salvedades 1 y 2 son de verificacion, no de
implementacion: ningun AC quedo sin implementar.
