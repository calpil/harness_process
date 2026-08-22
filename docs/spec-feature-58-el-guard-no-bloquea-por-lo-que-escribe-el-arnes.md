# Spec - Feature #58: el_guard_no_bloquea_por_lo_que_escribe_el_arnes

Estado: approved
Aprobado: 2026-08-22T16:58:38Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #58 en el chat (10 AC): el commit guard deja de bloquear el turno por los documentos que escribe el propio arnes, y sigue bloqueando por codigo. Decidio las dos observaciones: exencion POR ARTEFACTO (no por carpeta) y linea [i] cuando aplica.
Plan: docs/plan-feature-58-el-guard-no-bloquea-por-lo-que-escribe-el-arnes.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan trabaja en el SaaS inmobiliario, donde `docs/` es **su propio repo
git** con 137 artefactos del arnes adentro (specs, planes, impl, review,
estados). Cada vez que el arnes escribe uno de esos documentos —o sea, en cada
`start`, cada `advance`, cada `prd apply`— el turno termina asi:

```
Stop hook error: Cambios sin commitear en: docs
Haz commit por microservicio con Conventional Commits o usa HARNESS_COMMIT_GUARD_MODE=warn/off.
[Harness] Check fallo con 1 problema(s).
```

El arnes le pide a Alan que commitee "por microservicio" unos archivos que el
propio arnes acaba de escribir y que su propio `close` va a commitear. `docs/`
no es un microservicio: es donde vive el proceso. El guard lo cuenta como uno
porque su regla es "todo subdirectorio que sea un repo git", y ahi entra.

Esto no es una molestia de una vez: es en cada turno, en un proyecto instalado y
sano, y empuja a la salida facil (`HARNESS_COMMIT_GUARD_MODE=off`), que apaga el
guard tambien para el codigo, que es donde si sirve.

El arnes ya tiene escrita la regla que falta aplicar aca. `docs/rutas-protegidas.md`:

> **El arnes no se bloquea a si mismo.** La proteccion es contra las
> herramientas del agente, no contra el binario.

DESPUES: el guard sigue frenando el turno cuando queda codigo sin commitear —que
es para lo que existe— y deja de frenarlo por los documentos que escribio el
propio arnes. En el mismo proyecto, el mismo turno termina limpio, sin apagar
nada.

## Hoy -> Como va a funcionar

```
HOY                                        DESPUES
el arnes escribe docs/spec-feature-N.md    el arnes escribe docs/spec-feature-N.md
  |__ stop hook: commit_guard                |__ stop hook: commit_guard
  |__ "docs" esta sucio -> BLOQUEA           |__ ¿lo sucio es SOLO del arnes? -> sigue
  |__ Alan apaga el guard                    |__ codigo sin commitear -> BLOQUEA igual
```

## Recorridos de usuario (priorizados)

- P1: Como Alan trabajando en un proyecto ya instalado, quiero que el arnes no
  me bloquee el turno por los documentos que el mismo acaba de escribir, para no
  tener que apagar el guard.
- P1: Como Alan, quiero que el guard **siga** bloqueando cuando hay codigo sin
  commitear, porque esa es la unica razon por la que existe.
- P1: Como Alan, quiero que el arreglo viaje por el instalador a los proyectos
  que ya tienen el arnes puesto, sin tener que editar nada a mano.
- P2: Como Alan, quiero que cuando el guard se calle por esta razon quede dicho
  en algun lado, para no descubrir por accidente que dejo de mirar algo.

## Criterios de aceptacion (Given/When/Then)

### Lo que deja de bloquear

- AC-1: Given un repo cuyos unicos cambios sin commitear son artefactos del
  arnes (`spec-feature-*.md`, `plan-feature-*.md`, `impl-*.md`, `review-*.md`,
  `verify-*.md`, `estado-feature-*.md`, `prd-diff-*.md`, `docs/prd/**`,
  `docs/lecciones/**`, `docs/architecture.md`, `docs/perfil-usuario.md`), When
  corre el guard, Then no bloquea y ese repo no aparece como sucio.
- AC-2: Given ese mismo caso, When corre el guard, Then lo dice en una linea
  informativa (que repo se salteo y por que), para que el silencio no sea mudo.

### Lo que SIGUE bloqueando

- AC-3: Given un repo con codigo sin commitear (`.go`, `.ts`, `.rs`, lo que
  sea), When corre el guard, Then bloquea igual que hoy (exit 2 y el mensaje de
  Conventional Commits).
- AC-4: Given un repo con artefactos del arnes **y** codigo sin commitear, When
  corre el guard, Then bloquea: alcanza UN archivo que no sea del arnes.
- AC-5: Given un documento del proyecto que NO es artefacto del arnes
  (`docs/README.md`, `docs/runbook.md`), When queda sin commitear, Then bloquea:
  la exencion es por artefacto, no por carpeta.

### Que llegue a los proyectos

- AC-6: Given un proyecto ya instalado, When se re-corre el instalador, Then
  `commit_guard.sh` queda con el comportamiento nuevo (viaja por
  `templates/commit_guard.sh`, como todos los scripts).
- AC-7: Given el instalador de Windows, Then la paridad se mantiene: el guard es
  el mismo archivo generado, no hay logica duplicada en el `.ps1`.

### Verificacion

- AC-8: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, `bash tests/setup_smoke.sh` y
  `bash harness_check.sh`, Then los cuatro terminan limpios.
  Comando: `cd rust && cargo clippy --all-targets -- -D warnings`
- AC-9: Given el smoke, When corre, Then hay casos que prueban los cinco
  comportamientos de arriba (solo-arnes no bloquea, codigo bloquea, mixto
  bloquea, doc ajeno bloquea, y la linea informativa aparece). **Sin `Comando:`
  a proposito**: el smoke imprime mucho mas que el buffer de un pipe y
  `harness verify` se cuelga con el — es la feature **#46**, pendiente
  (`ejecutar()` espera al proceso ANTES de leer los pipes). Se corre a mano y el
  reviewer verifica el resultado; poner un comando que cuelga el gate seria
  peor que no ponerlo.
- AC-10: Given el proyecto real (`GolandProjects/realestate`), When se re-instala
  y se corre el guard con `docs/` sucio de artefactos del arnes, Then el turno
  termina limpio. Es el caso que disparo la feature.

## Los datos que se tocan

- `templates/commit_guard.sh` (y su copia generada `commit_guard.sh`): la lista
  de patrones de artefactos del arnes y la decision por repo.
- Nada de estado nuevo: la decision se toma leyendo `git status --porcelain` del
  repo, que el guard ya lee.
- Interruptor existente: `HARNESS_COMMIT_GUARD_MODE=warn|off` sigue igual.

## Pseudo-codigo (el acuerdo)

```
PARA CADA subdirectorio que sea un repo git:
  sucios = git status --porcelain
  si no hay sucios -> siguiente

  ¿TODOS los sucios matchean un patron de artefacto del arnes?
     SI  -> no cuenta como sucio; se anota en una linea informativa
     NO  -> cuenta como sucio (bloquea, como hoy)
```

Promesas: el guard nunca deja pasar codigo sin commitear · la exencion es por
archivo, no por carpeta · lo que se saltea se dice.

## No funcionales

- SLOs: una comparacion de strings por archivo sucio; sin costo perceptible.
- Seguridad: el guard no deja de mirar nada que hoy mire, salvo lo que el propio
  arnes escribio.
- Observabilidad: la linea informativa nombra el repo y la razon (AC-2).

## Fuera de alcance

- Cambiar como el arnes decide que es un microservicio (hoy: subdirectorio que
  es repo git). Eso es mas grande y toca el hub.
- Que el arnes commitee sus propios documentos fuera del `close`.
- El aviso `[i] PRD-master no declara hitos todavia`: es informativo y correcto
  (esa tabla esta vacia en ese proyecto), no es un fallo.

## Observaciones y decisiones

- OBS-1 [DECIDIDA por el USUARIO, 2026-08-22]: la exencion es **por artefacto**,
  no por carpeta. Un `docs/runbook.md` tocado a mano sigue bloqueando: el guard
  no deja de mirar nada que hoy mire, salvo lo que escribio el propio arnes.
- OBS-2 [DECIDIDA por el USUARIO, 2026-08-22]: cuando la exencion aplica, el
  guard lo DICE en una linea `[i]` con el repo y la razon. Un guard que se calla
  en silencio es como no tenerlo.
