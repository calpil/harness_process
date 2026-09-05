# Impl - Feature #72: El paralelo aisla los cambios y acota los workflows

Spec: docs/spec-feature-72-el-paralelo-aisla-los-cambios-y-acota-los-workfl.md
Plan: docs/plan-feature-72-el-paralelo-aisla-los-cambios-y-acota-los-workfl.md

## Lo que estaba roto, en una linea cada uno

El diagnostico del 2026-09-04 no encontro un bug: encontro cuatro formas
distintas de que el arnes **declarara** algo que no era cierto.

1. `start` marcaba `in_progress` ANTES de conseguir el aislamiento, y un fallo
   de git se imprimia con `[i]` y seguia. Asi quedaron activas #98, #122 y #126
   sin rama ni worktree, escribiendo las tres en el mismo checkout.
2. En el worktree de una feature, un `docs/` que es otro repo git queda vacio.
   El coordinador de la #98 lo leyo como "aca no se puede trabajar" y arranco
   `--sin-worktree`. El directorio vacio fue la excusa.
3. El cierre hablaba de "la rama" y nunca del rango. Se publico `9750cc2` (#117)
   y con el se fue `2fd6c5f` (#106), que se habia acordado dejar local: era su
   padre. Ademas el `push` era automatico, o sea que publicar no necesitaba que
   nadie lo pidiera.
4. `commit_guard.sh` recorria todos los repos hermanos sin atribuir nada a una
   sesion — y NUNCA miraba el worktree donde la sesion trabaja de verdad.
   Resultado: reclamaba lo ajeno y no miraba lo propio.

Y una quinta, del lado de la delegacion: el workflow de revision de la #117
registro 74 arranques para 14 tareas, con 12 fallidas, y su script hacia
`filter(Boolean)`. Los nulos desaparecieron y quedo "sin hallazgos".

## Evidencia por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/aislamiento.rs:148 | La decision de aislar es PURA y tiene una variante `Rechazar` con cuatro motivos; `rust/src/commands/start.rs:145` la corre ANTES de escribir el estado, asi que un rechazo sale sin haber tocado el backlog. |
| AC-2 | rust/src/git.rs:108 | `repo_de_docs` reconoce el `docs/` que es repo aparte; `start` le da su propio worktree y `rust/src/paths.rs:54` lo prefiere sobre el `docs/` vacio del worktree principal. |
| AC-3 | rust/src/git.rs:387 | `rango_de_integracion` devuelve TODOS los commits del rango y marca los que tambien viven en la rama de otra feature; `rust/src/commands/close.rs:666` bloquea con el rango a la vista. |
| AC-3 | rust/src/git.rs:443 | El candado por destino serializa dos integraciones sobre la misma rama, y `rust/src/commands/close.rs:776` deja la publicacion detras de `--publicar`. |
| AC-4 | commit_guard.sh:157 | El guard resuelve el worktree de la sesion —y exige que sea de ESTE proyecto— y desde ahi revisa lo propio; `commit_guard.sh:198` informa lo no atribuible UNA vez sin bloquear, con `HARNESS_COMMIT_GUARD_SCOPE=global` para el barrido completo. |
| AC-5 | roles/leader.md:95 | El contrato de delegacion acotada: cada tarea cita su AC, hasta cuatro por etapa, la revision empieza cuando terminaron los escritores, y no se relanza por inercia. |
| AC-5 | roles/leader.md:118 | Lo que el arnes NO puede imponer queda declarado como tal: los reintentos internos del runtime y la preferencia `small`, que es un consejo del proveedor y no un limite configurable. |
| AC-6 | rust/src/revision.rs:543 | `motivo_delegacion_incompleta` conserva las tareas fallidas/canceladas/sin resultado y bloquea `approved`; la cuenta esperada declarada antes es lo que impide que borrar los nulos complete la cobertura. |
| AC-7 | roles/leader.md:124 | El hallazgo adyacente se anota aparte en `progress/` y NO abre otra feature, workflow o reparacion sin autorizacion nueva. |
| AC-8 | tests/parity_check.sh:1 | Los diez modos verdes, con `roles/leader.md` y `templates/roles/leader.md` coherentes salvo el placeholder `__HREL__`. Suite: 457 unitarios + 248 de integracion, clippy limpio. |
| AC-9 | progress/current-72.md:14 | La configuracion local (`feedbackDrafts=quiet`, `workflowSizeGuideline=small`) ya estaba aplicada con respaldo en `progress/current-72.md:15`; esta feature NO la volvio a tocar. Sigue pendiente la confirmacion VISUAL del recuadro, que un archivo no puede probar. |
| AC-10 | rust/src/commands/status.rs:148 | El preflight inventaria las features abiertas sin worktree y dice explicitamente que no mueve commits, no cambia ramas, no borra worktrees y no para procesos. |

## Lo que se probo en rojo

Cada arreglo se verifico mutando la produccion y confirmando que el test cae:

- `decidir` sin el rechazo del bypass -> `sin_worktree_con_otra_abierta_se_rechaza` FALLA.
- El fallo de git de vuelta a `println!` + seguir ->
  `start_should_refuse_when_git_fails_instead_of_warning_and_continuing` FALLA.
- `para_feature` ignorando `docs_worktree` ->
  `start_should_give_a_separate_docs_repo_its_own_worktree` FALLA.
- `motivo_delegacion_incompleta` sin la cuenta esperada ->
  `borrar_las_fallidas_no_completa_la_cobertura` FALLA.
- El `commit_guard.sh` de HEAD -> el modo `sesion-acotada` FALLA con el texto
  exacto del bug diagnosticado ("Cambios sin commitear en: otroservicio").

## Tres huecos que encontre atacando mi propio trabajo

1. **Un worktree declarado que ya no existe seguia contando como aislamiento.**
   `ocupaciones` le creia al backlog. El AC-1 pide verificar la IDENTIDAD, no
   leer un campo: ahora la ruta tiene que existir
   (`rust/src/commands/start.rs:28`), y el test borra el worktree por fuera del
   arnes para probarlo.
2. **Una tarea que fallo bloqueaba para siempre.** El AC-6 pide conservar los
   estados, pero tambien dice "hasta cubrir la verificacion requerida": una
   tarea que despues se cubre tiene que desbloquear. Ahora manda el ULTIMO
   estado de cada id y las lineas anteriores se conservan igual
   (`rust/src/revision.rs:543`).
3. **Los artefactos del repo `docs/` quedaban varados.** Con docs como repo
   aparte, el spec, el plan, el impl y el review viven en una rama de ESE repo, y
   `close --to` solo integra el principal. Ahora el cierre los commitea, NO borra
   ese worktree y dice explicitamente que su integracion es decision del usuario
   (`rust/src/commands/close.rs:798`). Integrarlo solo seria mergear en un repo
   del usuario sin que nadie lo pidiera.

## Tres defectos que aparecieron implementando, y como se cerraron

1. **El rango que se mostraba no era el rango.** El AC-3 calculaba el rango
   ANTES de commitear el worktree, asi que el cierre anunciaba
   "(ninguno todavia)" y a la linea siguiente commiteaba y mergeaba. Lo
   encontro `close_should_not_publish_without_being_asked`. Se invirtio el
   orden: commit -> rango definitivo -> re-chequeo de ajenos -> merge.
2. **El guard tomaba el worktree de cualquier repo.** Mi primera version leia el
   worktree del CWD sin comprobar que fuera de ESTE proyecto, asi que
   `tests/stop_hook_check.sh` —que corre desde el worktree del arnes contra un
   proyecto de fixture— dejaba de ver el repo sucio de la fixture. Es el mismo
   cuidado que `rust/src/paths.rs` ya documentaba para el binario.
3. **El preflight cruzaba una frontera ajena.** Estaba en `doctor`, y
   `doctor_should_not_duplicate_the_process_checks` lo rechazo: `doctor`
   diagnostica la INSTALACION, el estado del proceso es de otro lado. Se movio a
   `status`, donde el backlog ya vive.

## Lo que esta feature NO promete

- **No es un sandbox.** Los hooks del arnes no interceptan un `git push` hecho a
  mano desde otra terminal. El AC-3 acota lo que el arnes hace al cerrar.
- **No controla los reintentos del runtime.** Los 74 arranques para 14 tareas
  salieron del runtime de Claude, no del arnes. La preferencia `small` esta
  documentada como consejo y no como limite configurable; el arnes no la
  presenta como garantia estructural.
- **No migro nada vivo.** Las features de realestate (#98, #122, #126) siguen
  como estaban: el preflight las inventaria y punto (AC-10).

## Un hallazgo adyacente, anotado y NO arreglado

`harness verify --feature 72` corrio **un solo** comando de los cuatro que el
AC-8 declara, y no lo dijo: `verificacion.rs` modela `comando: Option<String>`,
uno por AC. Es la misma familia que la #69 y la #64 —un reporte que dice verde
sobre algo que no comprobo— pero **no es de esta feature**.

Aplicando el AC-7 a mi propio trabajo: quedo anotado aparte en
`progress/hallazgo-verify-un-solo-comando.md` y no se abrio otra feature ni se
amplio esta. Los otros tres comandos se corrieron A MANO y quedaron verdes; esta
en `docs/review-72.md`. Correrlos a mano no reemplaza el arreglo.

## Deuda declarada

- La suite entera de este worktree tenia **10 errores de clippy previos** con el
  toolchain actual (4 `unwrap()` en `rust/src/revision.rs`, 6 borrows en
  `rust/tests/cli_basics.rs`). O sea que el comando `cargo clippy ... -D
  warnings` que el AC-8 declara estaba ROJO antes de esta feature. Se arreglaron
  como parte del AC-8; no eran regresiones de este trabajo.
