# Implementacion — feature #70: el gate de citas dice contra que resolvio

## La premisa del ticket era falsa, y eso cambio la feature

El ticket decia que una feature de backend **no tiene forma** de citar el codigo
de un repo hermano. Medido con el gate real antes de tocar nada:

| cita | hoy |
| --- | --- |
| `ms-media-service/handlers/portal_documents.go:3` (layout `subdir`) | **resuelve** |
| `propio.md:2` | resuelve |
| `../ms-media-service/...` | no (guarda de `..`) |
| ruta absoluta | no (guarda de `is_absolute`) |
| `ms-media-service/...` en layout `root` | no (los hermanos quedan fuera de toda raiz) |

En layout `subdir` —el que usa este proyecto— `repo_root` **ES** el directorio que
contiene los repos vecinos, y `raices_desde` ya lo ofrece como raiz. O sea que la
forma que anda existia; lo que no existia era **manera de enterarse**.

El defecto real, entonces, no es de resolucion sino de **mensaje**: las dos formas
que una persona escribe por instinto (`../` y la absoluta) se rechazan con el
mismo texto que cuando falta el archivo, y ese texto no nombra ni una sola de las
raices contra las que resolvio. El reviewer de la #117 termino citando
`docs/impl-117.md` en la columna que el gate comprueba, y el gate se dio por
satisfecho con una cita que no era la evidencia.

El alcance se decidio con el usuario antes de implementar (OBS-1 del spec).

## Lo que se hizo

`diagnostico_de_citas` se agrega al mensaje de rechazo y separa dos cosas que
antes se decian igual:

- **"por la FORMA de la ruta"** cuando la cita tiene `..` o es absoluta, diciendo
  que el gate no las acepta y por que (para que un review no cite fuera del
  arbol), mas la instruccion de escribirla relativa.
- **"no se encontro"** cuando la ruta es aceptable pero el archivo o la linea no
  aparecen.

Y en los dos casos **lista las raices, en orden, con su ruta**. Sin eso no hay
como deducir cual es la forma que anda.

`forma_repo_hermano` ofrece la forma `<repo-hermano>/<archivo>:<linea>` **solo**
cuando `repo_root != root`. En layout `root` los dos coinciden, los hermanos
quedan fuera de toda raiz y no se ofrece nada: un remedio que no funciona es peor
que ninguno, porque manda a la persona a probar algo que va a fallar
(`docs/lecciones/remedios-que-la-herramienta-sugiere.md`). El AC-4 lo fija en las
dos direcciones.

Los dos sitios que rechazan —el gate del cierre y `estampar`— llaman a
`porque_no_resolvieron`, no arman el texto cada uno por su cuenta: la #69 ya costo
un test rojo por tener dos copias de la misma pregunta.

## Lo que NO se toco, y por que

- **La resolucion.** Las mismas citas resuelven antes y despues. Esta feature no
  afloja el gate, lo hace explicable. El AC-3 fija que el hermano siga
  resolviendo, y que una linea inexistente siga sin hacerlo.
- **Las guardas de `..` y de rutas absolutas.** Estan para que un review no cite
  `/etc/passwd` ni se escape del arbol. El arreglo es explicar, no permitir.
- **El layout `root` viendo repos hermanos.** Requiere decidir de donde salen esas
  raices, y el campo `microservicios` del backlog es **prosa libre** —dice
  `harness` o `harness_process (rust/src/revision.rs)`—, no rutas. Por eso el AC
  que proponia el ticket ("`raices_desde` suma las raices de los microservicios
  que el plan declara") no se puede implementar tal cual.

## Disciplina de test rojo

| mutacion | test que se pone rojo |
| --- | --- |
| no lista las raices | `el_mensaje_lista_las_raices` |
| no distingue la forma de la ausencia | `el_mensaje_distingue_forma_de_ausencia` |
| el remedio se ofrece siempre | `el_remedio_no_miente_en_layout_root` |
| explica aunque la cita resuelva | `el_gate_verde_no_explica_nada` |
| el hermano deja de resolver | `el_repo_hermano_resuelve_en_subdir` |

## Lo que no se pudo comprobar

Cual de los dos casos le paso a la #117: vive en otro repo y aca no se toca. Que
su review terminara citando `impl-117.md` es compatible con las dos hipotesis
—layout `root`, o `../` y absoluta rechazadas—. Se dice como hipotesis, no como
dato (OBS-2 del spec).
