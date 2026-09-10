# Impl - Feature #85: Copilot CLI como backend de primera clase: superficie .github/copilot-instructions.md, hooks en .github/copilot.json, agentes en .github/agents, fila en doctor y backend LLM en la tabla de CLIs

Spec: docs/spec-feature-85-copilot-cli-como-backend-de-primera-clase-superf.md
Plan: docs/plan-feature-85-copilot-cli-como-backend-de-primera-clase-superf.md

## Lo que habia

Copilot no aparecia ni una vez en el repo: sin hooks (sin commit guard), sin
fila en `doctor`, fuera de la tabla de CLIs de `consolidar`. Lo unico que le
llegaba era `AGENTS.md`, porque el CLI lo lee solo.

## El arreglo

Un modulo `copilot.rs` con lo que hay que escribir en `.github/` (mezcla
idempotente de los tres hooks sobre el `copilot.json` del usuario, en el
lugar y sin mover claves, y un bloque entre marcadores en
`copilot-instructions.md`), expuesto como `harness copilot instalar|quitar`
para que los dos instaladores llamen al binario en vez de repetir el formato;
el modo `copilot-json` en los dos runtimes de hooks (`agentStop` enganchado
como Stop, con `stopHookActive` como corte, y todo lo legible por stderr); la
fila en `doctor` (que mira `hooks.agentStop`); y `copilot -s -p` al final de
la tabla de `consolidar`, con la autenticacion en el skip y en el error. Todo
medido contra `copilot --help` de la 1.0.83 instalada y la documentacion; lo
que no se pudo medir sin sesion (la semantica exacta de `agentStop`) quedo
declarado en el spec, el README, UPDATING, el SDD y el AC-8 manual.

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | rust/src/copilot.rs:62 · :45 · :511 · :530 · :546 · rust/tests/cli_basics.rs:9659 | `mezclar_config` conserva claves y hooks ajenos de otros eventos, deja (y avisa) un hook ajeno en uno de los tres eventos, reemplaza los propios (reconocidos por el token `copilot-json`), agrega `timeout` (ms) y NO mueve `hooks` de lugar. Integracion: `model` y `toolCall` intactos, `sessionStart` ajeno con `[i]`, `agentStop`/`sessionEnd` del arnes, segunda corrida con los mismos bytes y "Sin cambios", `--shell powershell`, prefijo por `HARNESS_COPILOT_HOOK` y exit 2 sin ninguno de los dos. |
| AC-2 | rust/src/copilot.rs:144 · :225 · :172 · :612 · :635 · rust/tests/cli_basics.rs:9737 | Bloque entre `harness:copilot:inicio/fin` que apunta a `AGENTS.md`, explica `agentStop` y usa las rutas de `--arnes`; el texto ajeno queda al principio byte a byte, dos corridas no duplican, sin archivo se crea solo con el bloque; dos bloques convergen a uno, un bloque en el medio conserva el blanco del usuario, y un marcador sin pareja no se toca (se avisa). |
| AC-3 | rust/src/copilot.rs:108 · :196 · :408 · :597 · :662 · rust/tests/cli_basics.rs:9776 | `quitar` deja el JSON ajeno igual en contenido y orden de claves (se re-serializa con sangria de dos y conserva el BOM: los bytes del formato no se prometen, y se documenta) y el `.md` ajeno byte a byte; borra el archivo que solo tenia lo del arnes POR ESO (un `hooks` vacio o no-objeto del usuario no se toca); sin archivos dice "Nada que quitar". |
| AC-4 | setup_harness.sh:1599 · :1516 · setup_harness.ps1:1464 · tests/copilot_hook_check.sh:68 | `copilot-json`: `agentStop` corre `run_stop` (lee `stopHookActive` con la misma grep que `stop_hook_active`) y emite `{"block":true,"reason":...}` o `{"block":false}`, siempre exit 0; `sessionStart`/`sessionEnd` emiten `{}`. El check monta un fixture con un repo hermano sucio: bloquea; con `stopHookActive: true` no; limpio no; `sessionEnd` sucio -> `{}`. El runtime PowerShell replica los cuatro casos y manda lo legible a stderr (`*>&1` a `[Console]::Error`), no ejecutable aca. |
| AC-5 | setup_harness.sh:488 · :1921 · :777 · :1225 · setup_harness.ps1:22 · :1784 · :1823 · :2034 · tests/parity_check.sh:171 · tests/setup_smoke.sh:1717 · tests/setup_smoke.ps1:131 · tests/copilot_hook_check.sh:84 · :96 | `--copilot`/`--no-copilot` (`-Copilot`/`-NoCopilot`), deteccion en PATH, respaldo previo, llamada al binario con `--arnes` (el ps1 pasa el prefijo por `HARNESS_COPILOT_HOOK` y baja `$ErrorActionPreference` a Continue alrededor de la llamada), `--reset` -> respalda los dos archivos y corre `quitar` antes de borrar nada (prueba `harness` y `harness.exe`; sin binario, avisa), texto AGENTS. Check: sin `copilot` en PATH (sacado del PATH real de la maquina) no se toca `.github/`; `--copilot` genera; `--no-copilot` omite y lo dice; reset devuelve lo ajeno y deja los respaldos en `bkp/`. Paridad 11/11 con el tema `copilot`. |
| AC-6 | rust/src/doctor.rs:288 · :373 · :724 | Fila `("copilot", ".github/copilot.json", ...)`: Ok nombrando `copilot` cuando `hooks.agentStop` apunta al runtime; Falla "no apuntan" con un `agentStop` ajeno aunque otro evento sea del arnes, y el remedio dice `--copilot`. |
| AC-7 | rust/src/consolidacion.rs:41 · :46 · :572 · :729 · :754 · :744 | `copilot -s -p` ultimo de la tabla (kimi gana); los dos skips nombran `copilot`, `/login` y `GH_TOKEN`; y si `copilot` fue elegido y sale sin sesion, el error del backend agrega la misma pista. |
| AC-8 | docs/review-85.md:1 | MANUAL, pendiente de una sesion autenticada: `copilot -s -p '...'` y una sesion `-p` con los hooks para contar los `agentStop`. Sin login el CLI sale 1 ("No authentication information found"); el `--help` de la 1.0.83 confirmo `-p` (prompt por argv), `-s`, `--output-format json` y que lee `AGENTS.md`. |
| AC-9 | README.md:1145 · UPDATING.md:857 · docs/architecture.md:161 · roles/README.md:73 · AGENTS.md:125 · rust/src/verificacion.rs:1270 | Seccion nueva en README y en las dos copias de UPDATING (identicas), bullet del modulo y del comando en architecture, entrada en roles/README y su template (agentes fuera de alcance, semantica de `agentStop` por medir), bullet en el AGENTS.md de la raiz y el texto AGENTS de los dos instaladores; AC-8 en el corpus. |

## El rojo

Los tres tests de integracion y el check de hooks se corrieron contra HEAD
antes de escribir nada: los tres cayeron por `unrecognized subcommand
'copilot'` y el check por `KeyError: 'sessionStart'` (el instalador no
escribia el hook). Los unitarios nacieron con el modulo; uno se corrigio en
el camino (el fixture usaba un prefijo sin `harness-hook`, y eso destapo que
reconocer lo propio por esa huella era fragil: ahora se reconoce por el token
`copilot-json`, que solo el arnes escribe).

Mutaciones, con `cmp` antes y despues, restauracion y `touch`:
- todo hook es del arnes (pisa lo ajeno): caen dos unitarios y los dos tests
  de integracion de AC-1/AC-3.
- `agentStop` nunca bloquea: cae `tests/copilot_hook_check.sh` ("con el repo
  sucio no bloqueo").
- instalar aunque no haya `copilot`: cae el check (el reset ya no vuelve a lo
  ajeno).

## La revision adversarial (ultracode: tres lentes y un refutador, solo lectura)

Tres agentes leyeron el worktree —spec, casos borde en archivos del usuario,
docs— y un cuarto intento refutar cada hallazgo (4 agentes, 562k tokens, 23
minutos). Veintinueve hallazgos: uno refutado y veintiocho confirmados que son
catorce distintos (las tres lentes repitieron varios). Trece se corrigieron en
el diff, con test cada uno donde se puede probar aca:

1. **Un `.md` que no es UTF-8 se REEMPLAZABA por el bloque sin aviso**
   (bloqueante): `read_to_string(..).unwrap_or_default()` trataba el error
   como archivo vacio. Ahora `leer_texto` distingue "no existe" de "no se
   puede leer" y un archivo ilegible no se toca (rust/src/copilot.rs:283,
   test :690).
2. **`quitar` borraba un `hooks` vacio o no-objeto del usuario**, y el archivo
   entero si era `{"hooks":{}}`: ahora solo se saca lo que era del arnes y
   `hooks` se va solo si quedo vacio por eso (:108, test :597).
3. **Un marcador de inicio sin fin borraba todo hasta el final del archivo**:
   un inicio sin fin no es un bloque (:172, test :635), y se avisa.
4. **Dos bloques nunca convergian y un bloque en el medio se comia una linea
   en blanco del usuario**: `sin_bloque` quita todos y solo toca el blanco
   previo cuando el bloque estaba al final (:196).
5. **`hooks` se movia al final del objeto** (remove + insert): ahora se mezcla
   en el lugar (test :530).
6. **JSON con BOM o con comentarios**: el BOM se conserva; un JSON que no
   parsea no se toca y se avisa, y `quitar` sigue con el `.md` en vez de
   abortar (test :690).
7. **Un enlace simbolico se reemplazaba por un archivo regular**: no se toca,
   con aviso (:277, test :740).
8. **El runtime PowerShell mandaba `status` y el gate a stdout, delante del
   JSON**: ahora `Invoke-HarnessEvent *>&1` va a `[Console]::Error`
   (setup_harness.ps1:1464).
9. **Windows PowerShell 5.1 no pasa comillas ni argumentos vacios por argv**
   (el prefijo del hook las lleva): el prefijo viaja por `HARNESS_COPILOT_HOOK`
   (rust/src/commands/copilot.rs:14) y `--arnes` solo se pasa con valor; y
   el `2>&1` bajo `$ErrorActionPreference = Stop` convertia el aviso de hook
   ajeno en error terminal: se baja a Continue alrededor de la llamada
   (setup_harness.ps1:1823).
10. **`--reset` no respaldaba los dos archivos** antes de `quitar`, usaba
    `harness` literal (Git Bash instala `harness.exe`) y callaba sin binario:
    respalda, prueba los dos nombres y avisa (setup_harness.sh:777,
    check :84).
11. **`doctor` daba OK con `agentStop` ajeno** si otro evento era del arnes
    (el texto contenia `harness-hook`): mira `hooks.agentStop` y el remedio
    dice `--copilot` (rust/src/doctor.rs:373).
12. **`consolidar` con `copilot` elegido y sin sesion** repetia el error del
    CLI sin decir como autenticarse: `con_pista_de_auth`
    (rust/src/consolidacion.rs:572).
13. **Docs**: README y roles/README presentaban "agentStop es el Stop" como
    hecho y nadie decia la unidad del `timeout` ni que el JSON se re-serializa:
    corregidos, con la salvedad y la unidad.

Queda sin cambiar, por decision: el `.md` sin salto final gana uno (los bytes
`A\n\n<bloque>` no distinguen `A` de `A\n`), y un hook ajeno ENCADENADO con el
del arnes cuenta como del arnes (el pseudo-codigo del spec lo define asi; fue
el unico hallazgo refutado). El AC-3 se lee, entonces, como contenido igual
para el JSON y bytes iguales para el `.md`.

## Lo que no se pudo medir

Copilot CLI 1.0.83 esta instalado en la maquina pero sin sesion de GitHub, y
`gh` tampoco esta logueado: los probes con `-p` salieron 1 antes de tocar
ningun hook. Lo que el spec afirma sale de `copilot --help` (autoritativo para
flags) y de la documentacion; la semantica de `agentStop` ("after each tool
call" en el texto, pero con `toolCall` aparte y `stopHookActive` copiado del
Stop de Claude) queda para el AC-8 con login (OBS-4). El runtime PowerShell no
se ejecuto (no hay Windows aca).

## Estilo (skills cargados)

`rust-patterns` / `rust-best-practices`: `mezclar_config`, `sin_bloque` y
`con_bloque` son puros y se prueban con round trip; `instalar`/`quitar` son la
unica capa con I/O, con escritura atomica, "sin cambios" comparando bytes y
"no se toca" como respuesta a todo lo que no se puede leer; `Cambio`/`Accion`
en vez de strings sueltos; sin `unwrap` fuera de tests. `rust-testing`: tests
por escenario con datos ajenos reales en el fixture, mutantes por AC.
`find-skills`: sin skill especifica. `rust-async-patterns`: no aplica.
