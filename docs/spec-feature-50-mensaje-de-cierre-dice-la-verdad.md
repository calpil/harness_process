# Spec - Feature #50: mensaje_de_cierre_dice_la_verdad

Estado: approved
Aprobado: 2026-08-22T12:35:27Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #50 en el chat (7 AC): el mensaje del cierre que no integra informa lo que realmente existe (rama y/o worktree) y calla cuando no queda nada, con funcion pura y un test por combinacion
Plan: docs/plan-feature-50-mensaje-de-cierre-dice-la-verdad.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan borra a mano la rama y el worktree de la feature #48 — ya cumplieron
su rol — y despues la cierra como `superseded`. El arnes le contesta:

    Rama feature/48-verificacion-paralelo-47 conservada
    (el cierre `superseded` no integra); su worktree tambien.

Las dos cosas son falsas: no queda ni la rama ni la carpeta. El mensaje no
miente por mala fe, miente por costumbre — se escribio asumiendo que si la
feature tiene `branch` en el backlog, la rama existe. Es exactamente el tipo de
`[ok]` que la leccion `probar-contra-datos-reales` describe: informa sobre algo
adyacente (lo que dice el backlog) como si hubiera mirado lo que importa (lo que
hay en el repo).

DESPUES: el arnes mira antes de hablar. Si estan las dos cosas, lo dice; si
falta una, lo dice; si no queda ninguna, no promete nada. El usuario puede
confiar en esa linea para saber si tiene algo que limpiar despues.

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES
close --status pending|blocked|         mira el repo y informa lo que HAY:
      superseded                          rama + worktree -> "conservados"
  -> "Rama X conservada; su worktree      solo la rama    -> "conservada; su worktree ya no esta"
     tambien" (sin mirar nada)            solo el worktree-> "la rama ya no esta; queda el worktree"
                                          ninguno         -> no dice nada
```

## Recorridos de usuario (priorizados)

- P1: Como Alan cerrando una feature que no integra, quiero que la linea sobre
  la rama y el worktree diga lo que realmente hay, para saber si me queda algo
  por limpiar sin tener que verificarlo yo.
- P2: Como el proximo que lea ese codigo, quiero un test por cada combinacion,
  para que nadie vuelva a afirmar sin mirar.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una feature con rama y worktree existentes, When la cierro como
  `blocked`, `pending` o `superseded`, Then el mensaje dice que se conservan los
  dos y nombra la rama.
- AC-2: Given que el worktree ya no existe pero la rama si, When cierro sin
  integrar, Then el mensaje dice que la rama se conserva y que el worktree ya no
  esta.
- AC-3: Given que la rama ya no existe pero el worktree si, When cierro sin
  integrar, Then el mensaje dice que la rama ya no esta y que queda el worktree.
- AC-4: Given que no queda ni la rama ni el worktree, When cierro sin integrar,
  Then el arnes NO imprime ninguna linea sobre conservacion: no hay nada que
  informar.
- AC-5: Given una feature sin `branch` en el backlog (modo clasico o repo sin
  git), When cierro sin integrar, Then el comportamiento es el de siempre: no se
  imprime nada y el cierre no cambia su exit code.
- AC-6: Given cualquiera de los casos anteriores, When cierro, Then el estado
  del backlog, el archivado y el resto de la salida no cambian: esto solo toca
  esa linea.
- AC-7: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh`, Then los cuatro terminan limpios, con un test por cada
  combinacion de AC-1 a AC-4.

## Los datos que se tocan

- disparador: `close` con un estado que no integra (`blocked`, `pending`,
  `superseded`).
- interruptor: ninguno.
- candado: ninguno.
- No se toca ningun archivo ni ningun campo: es solo lo que se imprime. La rama
  se consulta con `git::rama_existe` y el worktree con una comprobacion de
  directorio.

## Pseudo-codigo (el acuerdo)

```
CUANDO se cierra una feature con un estado que no integra

  ¿la feature tiene rama propia?   -> si no, no decimos nada (modo clasico)

  miramos si la rama sigue en el repo
  miramos si la carpeta del worktree sigue en disco

  ENTONCES informamos SOLO lo que encontramos,
           con la restriccion de que si no queda nada, no se imprime
           ninguna linea: el silencio es mas honesto que una promesa vacia.
```

Promesas: nunca afirma sobre algo que no miro · no toca archivos ni estado ·
sin rama en el backlog, cero cambios.

## No funcionales

- SLOs: dos consultas locales (un `git rev-parse` y un `is_dir`), sin red.
- Seguridad: no escribe nada; solo lee.
- Observabilidad: es, precisamente, una mejora de observabilidad.

## Fuera de alcance

- Ofrecer limpiar la rama o el worktree al cerrar: informar no es actuar, y
  borrar ramas sigue siendo decision del usuario.
- Revisar otros mensajes del arnes que puedan afirmar sin mirar: si aparecen,
  cada uno con su feature.

## Observaciones (decisiones pendientes)

- OBS-1 [REGISTRADA]: el caso lo encontro el uso real (borrar la rama de la #48
  y cerrarla despues), no un test. Es el mismo patron de la leccion
  `probar-contra-datos-reales`, esta vez del lado del "OK que dice de mas".
