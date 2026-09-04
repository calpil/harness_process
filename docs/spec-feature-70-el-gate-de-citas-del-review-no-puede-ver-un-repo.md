# Spec - Feature #70: El gate de citas del review no puede ver un repo hermano: una feature de backend no puede citar su codigo

Estado: approved
Aprobado: 2026-09-04T19:53:53Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-70-el-gate-de-citas-del-review-no-puede-ver-un-repo.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

El reviewer de la #117 escribe su veredicto. Una fila tiene que citar
`handlers/portal_documents.go:353`, que vive en `ms-media-service`, un repo
hermano. Prueba `../ms-media-service/handlers/portal_documents.go:353`: rechazado.
Prueba la ruta absoluta: rechazado. El gate le contesta lo mismo las dos veces:

> Cada AC-n necesita una fila que lo nombre y cite `archivo:linea`, con un archivo
> que exista y una linea que exista en el.

El archivo existe. La linea existe. `sed` la muestra. El mensaje no menciona
**contra que** esta resolviendo, asi que no hay como deducir cual es la forma que
anda. El reviewer se rinde y cita `docs/impl-117.md:<linea>` en la columna que el
gate comprueba, dejando el codigo real en la de al lado. El gate se da por
satisfecho con una cita que no es la evidencia.

Y lo peor: **la forma que funcionaba estaba ahi**. Medido con el gate real, en el
layout `subdir`, `ms-media-service/handlers/portal_documents.go:353` —relativa al
padre, sin `../`— **resuelve**. `repo_root` en ese layout ES el directorio que
contiene los repos hermanos, y `raices_desde` ya lo ofrece.

Despues: cuando una cita no resuelve, el gate dice contra que raices probo y con
que forma se cita un archivo de un repo hermano. El reviewer no tiene que
adivinar, y no termina citando el documento que el mismo escribio.

## Hoy -> Como va a funcionar

El defecto NO es de resolucion, es de **mensaje**. Medido antes de escribir esto:

| cita | hoy |
| --- | --- |
| `ms-media-service/handlers/portal_documents.go:3` (layout subdir) | **resuelve** |
| `propio.md:2` | resuelve |
| `../ms-media-service/...` | no (guarda de `..`) |
| ruta absoluta | no (guarda de `is_absolute`) |
| `ms-media-service/...` en layout `root` | no (los hermanos quedan fuera de toda raiz) |

Las dos formas que una persona escribe por instinto se rechazan, la que anda no se
nombra en ningun lado, y el mensaje no distingue "el archivo no existe" de "esa
forma de ruta no la acepto".

Despues, el mensaje del gate:

- lista **las raices contra las que resolvio**, con su ruta;
- si la cita fallo por la guarda de `..` o por ser absoluta, **lo dice** y ofrece
  la forma relativa equivalente;
- nombra la forma de citar un repo hermano cuando el layout la admite.

Lo que NO cambia: que raices hay. La resolucion queda igual — esta feature no
afloja el gate, lo hace explicable.

## Recorridos de usuario (priorizados)

- P1: Como reviewer, quiero que el gate me diga contra que resolvio, para escribir
  una cita que apunte al codigo de verdad en vez de rendirme y citar el documento
  que yo mismo escribi.
- P2: Como reviewer que uso `../` o una ruta absoluta, quiero que me diga que el
  problema es la FORMA y no el archivo, para no salir a buscar un archivo que
  esta donde yo creia.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un review con una cita que no resuelve, When corre el gate, Then el
  mensaje lista las raices contra las que probo, cada una con su ruta.
  Comando: `cd rust && out=$(cargo test el_mensaje_lista_las_raices 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-2: Given una cita con `..` o absoluta, When corre el gate, Then el mensaje
  dice que el rechazo es por la FORMA de la ruta —no porque falte el archivo— y
  nombra la forma relativa que si se acepta.
  Comando: `cd rust && out=$(cargo test el_mensaje_distingue_forma_de_ausencia 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-3: Given el layout `subdir`, When se cita `<hermano>/<archivo>:<linea>` de un
  repo vecino, Then RESUELVE. Es lo que ya pasa hoy; se fija para que no se pierda
  y para que el mensaje del AC-1 no mienta al ofrecerlo.
  Comando: `cd rust && out=$(cargo test el_repo_hermano_resuelve_en_subdir 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-4: Given el layout `root`, When se cita un repo hermano, Then NO resuelve, y
  el mensaje NO ofrece esa forma. Un remedio que no funciona en ese layout es peor
  que ninguno: manda a la persona a probar algo que va a fallar.
  Comando: `cd rust && out=$(cargo test el_remedio_no_miente_en_layout_root 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-5: Given una cita que resuelve, When corre el gate, Then no imprime nada de
  esto. El diagnostico aparece cuando hace falta; un gate que explica siempre se
  deja de leer.
  Comando: `cd rust && out=$(cargo test el_gate_verde_no_explica_nada 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

## Los datos que se tocan

- disparador: `revision --veredicto` y el gate de `close`.
- lee: el review, el spec y las raices de citas.
- escribe: nada nuevo; cambia el TEXTO de un mensaje de error.
- borra: nada.

## Pseudo-codigo (el acuerdo)

```
al rechazar por cobertura:
    por cada AC sin fila:
        por cada cita de sus filas que NO resolvio:
            si tiene ".." o es absoluta:
                decir "forma no aceptada" + la relativa equivalente
            si no:
                decir "no se encontro" + las raices probadas
    si repo_root != root (layout subdir):
        ofrecer la forma <hermano>/<archivo>:<linea>
```

## No funcionales

- La resolucion no cambia: las mismas citas resuelven antes y despues.
- El mensaje sale por el mismo camino que hoy (`anyhow::bail!` / `Exit`).

## Fuera de alcance

- **Aflojar la guarda de `..` o de rutas absolutas.** Estan para que un review no
  cite `/etc/passwd` ni se escape del arbol. El arreglo es explicar, no permitir.
- **Hacer que el layout `root` vea repos hermanos.** Requiere decidir de donde
  salen esas raices, y el campo `microservicios` del backlog es prosa libre —dice
  `harness` o `harness_process (rust/src/revision.rs)`—, no rutas. Queda fuera y
  el AC-4 se asegura de que el mensaje no prometa lo que ese layout no puede.
- **Cambiar la columna que el gate comprueba** en la tabla del review.

## Observaciones (decisiones pendientes)

- OBS-1: **la premisa del ticket es falsa en el layout `subdir`.** El ticket dice
  que "no tiene forma de citar" el codigo de un repo hermano; medido, la forma
  relativa al padre resuelve. El defecto real es que esa forma no esta nombrada en
  ningun lado y las dos que la gente escribe por instinto se rechazan sin decir
  por que. El alcance quedo en el mensaje; decidido con el usuario antes de
  implementar.
- OBS-2: no se puede comprobar cual de los dos casos le paso a la #117 —vive en
  otro repo y aca no se toca—. Que su review terminara citando `impl-117.md` es
  compatible con las dos hipotesis (layout `root`, o `../` y absoluta rechazadas).
  Se dice como hipotesis, no como dato.
