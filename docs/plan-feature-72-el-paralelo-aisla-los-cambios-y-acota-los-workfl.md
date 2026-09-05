# Plan - Feature #72: El paralelo aisla los cambios y acota los workflows

Estado: in_progress
Microservicios:
- harness

## Alcance

Los diez AC del spec caen en cuatro grupos, y solo tres de ellos son codigo:

1. **Aislamiento verificable** (AC-1, AC-2): `start` deja de mentir. Hoy marca
   `in_progress` ANTES de preparar el aislamiento y traga cualquier fallo de git
   con un `println!` (`commands/start.rs:52`), asi que una feature puede quedar
   activa sin rama ni worktree — que es exactamente el estado en que el
   diagnostico encontro a #98, #122 y #126.
2. **Integracion honesta** (AC-3): el cierre presenta origen, destino y TODO el
   rango de commits, y se niega si arrastra trabajo ajeno. El incidente
   verificado (`9750cc2` publicado con `2fd6c5f` de otra feature colgando de su
   padre) es la prueba de que el rango importa mas que el commit.
3. **Stop acotado** (AC-4): `commit_guard.sh` recorre TODOS los repos hermanos y
   pregunta por el estado git completo, sin atribuir nada a una sesion. Se acota
   a lo atribuible y lo no atribuible se informa UNA vez, sin bloquear.
4. **Cobertura que no se puede fingir** (AC-5, AC-6, AC-7): el gate del review
   ya exige una fila por AC; falta que una tarea delegada fallida, cancelada o
   sin resultado quede REGISTRADA POR EL BINARIO y bloquee `approved`/`done`.
   El `filter(Boolean)` del workflow de #117 es el antipatron a cerrar: descartar
   nulos convirtio 12 fallos en "no hay hallazgos".

AC-8 es el gate de espejos y la suite. AC-9 ya esta aplicado (configuracion
local de Claude, fuera del codigo del arnes). AC-10 es procedimiento de
despliegue: preflight + respaldo, sin tocar sesiones vivas.

## Impacto entre microservicios

Solo `harness`. Nada se distribuye a otros proyectos en esta feature: a
realestate se le prepara la actualizacion y se le entrega el comando (AC-10).

## Peldano de huella

`Peldano elegido: 2 (flag/campo en lo que existe) porque el registro de tareas
delegadas entra como flags de `harness revision` y campos del backlog, no como
comando nuevo.` El peldano 3 (comando `delegacion`) se descarto: el gate que
tiene que leer esos estados ya vive en `revision.rs`, y partirlo en dos
superficies repetiria la divergencia que las features #67 y #69 pagaron.

## Delegacion (implementer)

- D-1 (AC-1): `rust/src/aislamiento.rs` NUEVO — decision PURA
  (`decidir(contexto) -> Aislamiento`), sin tocar disco ni git. Casos:
  aislado / serial-no-aislado / sin-git-no-aislado / RECHAZADO (comparte
  checkout, `--sin-worktree` con otra feature activa, fallo de git).
- D-2 (AC-1): `commands/start.rs` — invertir el orden: resolver aislamiento
  ANTES de escribir `status`/`started_at`. Un rechazo sale por `Exit` y deja el
  backlog exactamente como estaba.
- D-3 (AC-2): `paths.rs` + `aislamiento.rs` — resolver el `docs/` real de la
  feature. Si `docs/` es OTRO repo git, tiene su propio worktree; si la ruta es
  ambigua o compartida, no se autoriza la escritura (no se cae al checkout
  compartido por ver un directorio vacio).
- D-4 (AC-3): `git.rs` — `rango_de_integracion()` (origen, destino, lista
  completa de commits) y `commits_ajenos()` (los del rango que no son de la
  rama de la feature). `close.rs` los presenta y bloquea.
- D-5 (AC-3): `git.rs` — candado por destino para serializar integraciones.
- D-6 (AC-4): `commit_guard.sh` + `harness_check.sh` — atribuir por worktree de
  feature; lo no atribuible informa una vez y no bloquea.
- D-7 (AC-5/AC-6): `revision.rs` — registro de tareas delegadas escrito por el
  BINARIO (`--tarea/--estado`), y `gate()` que niega `approved` con cobertura
  incompleta. Estados terminales explicitos, incluido `sin-resultado`.
- D-8 (AC-7/AC-5): `roles/leader.md` + `AGENTS.md` + espejos — el contrato de
  delegacion acotada y del hallazgo adyacente que se anota y NO se convierte en
  otro encargo. Se declara lo que el arnes NO puede imponer (reintentos
  internos del runtime de Claude).
- D-9 (AC-8): `tests/parity_check.sh` verde y espejos coherentes.
- D-10 (AC-10): preflight de actualizacion con respaldo, sin migrar nada vivo.

## Criterios de cierre (reviewer)

- Cada AC con su fila `| AC-n | archivo:linea | veredicto |` y cita que resuelve.
- Pruebas por COMPORTAMIENTO con fixtures git de verdad (dos features, repo
  docs aparte, fallo de start, dos sesiones Stop, tarea fallida). Nada de grep
  del fuente, nada de AC en verde con cero casos corridos.
- Lo que no es imponible queda DECLARADO como no imponible, no como garantia.

## Riesgos

- R-1: AC-1 endurece `start`. Una instalacion que hoy arranca con
  `--sin-worktree` en paralelo va a ser rechazada. Es el objetivo del spec, pero
  el mensaje tiene que decir exactamente que hacer.
- R-2: AC-2 crea ramas en el repo `docs/` del usuario. Solo cuando docs es un
  repo aparte, y nunca borra nada.
- R-3: realestate tiene sesiones VIVAS ahora mismo. AC-10 prohibe migrarlas.

## Observaciones (decisiones pendientes)

- OBS-1 (del spec): resuelta — el usuario aprobo el spec el 2026-09-05.
- OBS-2 (del spec): la rama de integracion se pregunta antes de
  `close --status done --to`.
- OBS-3 (del spec): la migracion de sesiones vivas se coordina aparte.
- OBS-4 (AC-1, DECIDIDA por el usuario 2026-09-05): en un proyecto SIN git el
  rechazo es DURO — solo una feature abierta a la vez. Esto REVOCA el AC-1 de la
  feature #47 ("varias features pueden estar in_progress a la vez") para el caso
  sin git, y los tres tests que lo codificaban se reescriben contra la regla
  nueva. Alternativa descartada: avisar sin rechazar, que deja abierta la puerta
  por la que entraron #98, #122 y #126.
- OBS-5 (AC-2, DECIDIDA por el usuario 2026-09-05): cuando `docs/` es otro repo
  git, el arnes CREA su rama y su worktree (`../docs-wt/<id>-<slug>`), igual que
  con el repo principal. Nunca borra nada y la rama se conserva. Alternativa
  descartada: detectar y negarse, que agrega un paso manual a cada feature que
  toque docs.

---
Cerrado: 2026-09-05T03:48:04Z - status=done - Aislamiento verificable en start, rango completo en el cierre, Stop acotado a la sesion y cobertura de delegacion que no se puede fingir
