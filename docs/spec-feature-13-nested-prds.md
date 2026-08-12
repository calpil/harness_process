# Spec - Feature #13: nested_prds

Estado: approved
Aprobado: 2026-08-12T11:17:50Z por USUARIO (confirmacion explicita) - Alan aprueba las tres propuestas: carpeta corta + archivo con cadena completa, --prd por segmento unico, y gate sin ciclos/duplicados
Plan: docs/plan-feature-13-nested-prds.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan arranca un producto nuevo y abre `docs/prd/COMO-ESCRIBIR-UN-PRD.md`.
La guia le promete, en la seccion 3, que "un PRD puede contener otros PRDs, y
esos a su vez mas", y le dibuja el arbol. Alan se convence, escribe el maestro,
y cuando llega el momento de partirlo descubre que la promesa es solo prosa: no
hay ningun comando que cree un PRD hijo, `docs/prd/` es una carpeta plana con
tres archivos que el instalador siembra, `feature_list.json` no sabe de que PRD
viene cada feature y `harness_check.sh` no mira nada de esto. Alan termina
creando `docs/prd/PRD-cobranza.md` a mano, decidiendo el nombre por su cuenta,
enlazandolo (o no) desde el maestro, y a la tercera feature ya nadie sabe que
hito de que PRD dio origen a que spec. El arbol existe en la cabeza de Alan, no
en el repositorio.

DESPUES: Alan escribe `sh harness_cli prd add --name cobranza`, y el arnes crea
`docs/prd/cobranza/PRD-cobranza.md` con las 12 secciones del metodo ya puestas y
la fila correspondiente enlazada en el maestro. Parte otra vez:
`prd add --name mora --parent cobranza` cuelga
`docs/prd/cobranza/mora/PRD-cobranza-mora.md` un nivel mas abajo. Cuando carga un
hito al backlog escribe `add --name avisar_mora ... --prd cobranza/mora`, y el
spec que nace de esa feature ya trae en su encabezado
`PRD: docs/prd/cobranza/mora/PRD-cobranza-mora.md`. En cualquier momento
`sh harness_cli prd tree` le dibuja el arbol completo con cuantos hitos lleva
cada PRD y cuantos estan cerrados, y `harness_check.sh` le avisa si movio un
archivo a mano y dejo el arbol incoherente. La cadena producto -> parte -> hito
-> feature -> spec es navegable en los dos sentidos, sin que Alan tenga que
recordar nada.

## Hoy -> Como va a funcionar

```
HOY                                        DESPUES

docs/prd/ (plano, sembrado)                docs/prd/ (arbol, cultivado)
  PRD-master.md                              PRD-master.md
  SDD-master.md                              SDD-master.md
  COMO-ESCRIBIR-UN-PRD.md                    COMO-ESCRIBIR-UN-PRD.md
  (los hijos: a mano o nunca)                cobranza/
                                               PRD-cobranza.md
                                               mora/
                                                 PRD-cobranza-mora.md

harness_cli add --name f ...               harness_cli prd add --name mora --parent cobranza
        |                                          |__ crea el hijo desde plantilla
        v                                          |__ lo enlaza en el padre
feature_list.json {id,name,...}
        |                                  harness_cli add --name f ... --prd cobranza/mora
        v                                          |__ feature_list.json {..., "prd": "cobranza/mora"}
docs/spec-feature-<id>-<slug>.md                   v
  Plan: / Constitution: / Metodo:           docs/spec-feature-<id>-<slug>.md
  (no sabe de que PRD viene)                  Plan: / PRD: docs/prd/cobranza/mora/PRD-cobranza-mora.md

(nadie ve el arbol)                        harness_cli prd tree  -> dibuja el arbol + hitos
(nadie valida el arbol)                    harness_check.sh      -> gate de integridad
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como duenno de un producto grande, quiero partir mi PRD en PRDs hijos con
  un comando, para que cada parte tenga su propia historia sin que yo invente
  convenciones de nombre ni de ubicacion.
- P1: Como quien carga el backlog, quiero decir de que PRD viene una feature,
  para que su spec cite el PRD de origen y nadie tenga que reconstruir esa
  relacion despues.
- P1: Como lider que retoma el proyecto, quiero ver el arbol completo con hitos y
  estado en un comando, para saber que parte del producto esta viva y cual sigue
  siendo solo un documento.
- P2: Como duenno del repo, quiero que `harness_check.sh` me avise si el arbol
  quedo incoherente (archivo movido, encabezado que miente, feature que apunta a
  un PRD inexistente), para enterarme antes de cerrar una feature y no seis
  meses despues.
- P2: Como agente de cualquier backend (Claude, Codex, Gemini, Kimi), quiero que
  la guia y las superficies raiz describan los comandos reales, para usarlos sin
  que nadie me cuente que existen.
- P2: Como usuario de Windows, quiero paridad: el mismo binario expone los mismos
  comandos y `setup_harness.ps1` describe lo mismo que su par sh.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->

- AC-1: Given un proyecto con `docs/prd/PRD-master.md`, When corro
  `sh harness_cli prd add --name cobranza`, Then el arnes crea
  `docs/prd/cobranza/PRD-cobranza.md` (carpeta con el segmento + archivo cuyo
  nombre es `PRD-` + la cadena de segmentos unida por `-`), y con
  `--parent cobranza --name mora` crea
  `docs/prd/cobranza/mora/PRD-cobranza-mora.md`; sin `--parent` el padre es el
  maestro. El slug se normaliza con la misma `slugify` que ya usan planes y
  specs (`cobranza_mora` -> `cobranza-mora`).
- AC-2: Given `prd add`, When el padre no existe, el destino ya existe o el slug
  queda vacio tras normalizar, Then el comando falla con exit 1 y un mensaje que
  nombra el problema y (si el padre no existe) lista los PRDs disponibles; no
  crea ni modifica ningun archivo.
- AC-3: Given un PRD hijo recien creado, When lo abro, Then trae el encabezado
  con `Padre: <ruta del padre>` (`master` para un hijo del maestro), `Estado:
  Borrador`, `Alcance:` con su "NO toca", el puntero a
  `docs/prd/COMO-ESCRIBIR-UN-PRD.md`, y las mismas 12 secciones del metodo que
  `PRD-master.md` en el mismo orden, incluyendo `## 10. Hitos -> features` con
  la linea `sh harness_cli add --name <slug> --service <servicio> --acceptance
  "<criterio>" --prd <ruta>`.
- AC-4: Given el PRD padre, When `prd add` termina, Then el padre queda con una
  seccion `## PRDs anidados` que lista al hijo con su ruta relativa y su
  descripcion; si la seccion no existia, se agrega al final del archivo sin
  reordenar ni reescribir nada mas del documento (es un documento del USUARIO), y
  si ya existia se agrega una fila mas sin duplicar una existente.
- AC-5: Given `sh harness_cli add`, When paso `--prd <ref>`, Then la feature
  guarda `"prd": "<ruta canonica>"` en `feature_list.json`; `<ref>` se resuelve
  por ruta completa (`cobranza/mora`) o por segmento final si es unico en el
  arbol (`mora`), y una referencia ambigua o inexistente falla con exit 1
  listando los candidatos. Sin `--prd` la feature se guarda exactamente como
  hoy, sin campo nuevo.
- AC-6: Given una feature con campo `prd`, When corro `start` y se genera su
  spec, Then el encabezado incluye `PRD: docs/prd/<ruta>/PRD-<cadena>.md` en la
  linea siguiente a `Plan:`; sin campo `prd` la linea apunta a
  `docs/prd/PRD-master.md`. `Estado: draft` sigue en la linea 3 y el orden de las
  secciones del spec no cambia.
- AC-7: Given `sh harness_cli prd tree`, When lo corro, Then dibuja el arbol
  desde el maestro con un nodo por PRD, y por nodo el conteo de hitos de su tabla
  `## 10. Hitos -> features` y cuantas de las features que lo declaran (`prd`)
  estan `done`; marca `[!] sin hitos` a los PRDs sin filas y `[!] Padre: X (no
  coincide con su ubicacion)` a los encabezados incoherentes. Con `--prd <ref>`
  dibuja solo ese subarbol.
- AC-8: Given `bash harness_check.sh` en un proyecto con `docs/prd/`, When el
  arbol esta sano, Then no agrega fallos; y cuando un `PRD-*.md` esta en una
  carpeta que no corresponde a su cadena de segmentos, una carpeta bajo
  `docs/prd/` no contiene su PRD, un encabezado `Padre:` no coincide con la
  ubicacion real, o una feature declara un `prd` inexistente, Then reporta `[!]`
  por cada caso, suma a `failures` y sale 2 en modo estricto (0 en modo `warn`).
  Un PRD sin hitos avisa con `[i]` y NO bloquea.
- AC-9: Given que `docs/prd/` puede no existir (instalacion minima o proyecto sin
  PRDs), When corro `harness_check.sh` o `prd tree`, Then no fallan: el check
  omite el bloque y `prd tree` informa que no hay arbol todavia.
- AC-10: Given `templates/docs/prd/COMO-ESCRIBIR-UN-PRD.md` y su copia en
  `docs/prd/`, When leo la seccion del tamano y la del mapeo al arnes, Then la
  promesa de PRDs anidados esta acompanada de los comandos reales (`prd add`,
  `prd tree`, `add --prd`), del layout en carpetas con un ejemplo de arbol, y de
  la tabla de niveles actualizada (Producto = `PRD-master.md`, Parte = PRD
  anidado, Cambio = spec de la feature).
- AC-11: Given `templates/docs/prd/PRD-master.md` y su copia, When leo el
  maestro, Then declara la seccion `## PRDs anidados` (donde `prd add` engancha a
  los hijos) y su tabla de hitos menciona `--prd <ruta>`; las planillas siguen
  siendo documentos del USUARIO (`PRD_DOCS` / `$script:PrdDocs`): ningun
  reinstall ni `--force` las pisa.
- AC-12: Given las superficies raiz que generan ambos instaladores y las docs del
  repo (`README.md`, `AGENTS.md`, `UPDATING.md` raiz y `templates/`,
  `docs/architecture.md`), When se instalan o se leen, Then describen el arbol de
  PRDs y sus tres comandos, con paridad sh/ps1 y sin tocar
  `write_basic_agent_surface` ni `.grok/GROK.md`.
- AC-13: Given `harness_check.sh` y `templates/harness_check.sh`, When comparo
  ambos, Then son identicos (regla de espejo del Articulo 6) y el gate nuevo vive
  en los dos.
- AC-14: Given `cargo test --locked`, When corre, Then cubre con tests unitarios:
  derivacion de ruta/nombre de archivo por cadena de segmentos, plantilla del PRD
  hijo (secciones y orden), enlace en el padre (crea seccion / agrega fila / no
  duplica), resolucion de `--prd` (exacta, por segmento unico, ambigua,
  inexistente), encabezado `PRD:` del spec con y sin campo, y render de
  `prd tree`; ademas siguen verdes los tests existentes de spec, plan y features.
- AC-15: Given `tests/setup_smoke.sh` y `tests/setup_smoke.ps1`, When corren,
  Then verifican sobre un fixture instalado: `prd add` anidado en dos niveles, el
  enlace en el padre, `add --prd` + `start` dejando el encabezado `PRD:` en el
  spec, `prd tree` dibujando los dos niveles, y `harness_check.sh` detectando un
  arbol roto a proposito. `bash tests/setup_smoke.sh` sale 0; sin `pwsh` la
  version ps1 se verifica estaticamente, como en las features #1 y #4 a #12.
- AC-16: Given el repo, When corro los comandos oficiales de
  `docs/verification.md`, Then pasan `bash harness_check.sh`,
  `cargo test --locked`,
  `cargo clippy --all-targets --all-features --locked -- -D warnings` y
  `bash tests/setup_smoke.sh`.
- AC-17: Given una feature con campo `prd` (o sin el, que cuenta para el
  maestro), When corro `close --feature <id> --status done`, Then el arnes
  escribe de vuelta en el PRD de origen: (a) si la tabla
  `## 10. Hitos -> features` tiene una fila cuyo slug de feature coincide con el
  nombre de la feature, su celda Estado pasa a `done (YYYY-MM-DD)`; (b) se
  agrega una linea a la seccion `## Bitacora` del PRD (creada al final si falta)
  con el id, el nombre, la fecha y los punteros a su spec y a
  `docs/impl-<id>.md`. Es idempotente (no duplica la entrada de una feature ya
  registrada), NUNCA reescribe el cuerpo del PRD (historia, datos,
  pseudo-codigo), y si el PRD no existe o la tabla no tiene la fila, el cierre
  sigue adelante sin fallar (avisando con `[i]`).
  `close --status blocked|pending` no toca ningun PRD.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->

- Entidad nueva **PRD anidado**: no tiene registro propio; su identidad ES su
  ubicacion. Cadena de segmentos (`cobranza/mora`) -> carpeta
  `docs/prd/cobranza/mora/` -> archivo `PRD-cobranza-mora.md`. El filesystem es
  la fuente de verdad; el encabezado `Padre:` es una declaracion que el gate
  contrasta contra la ubicacion real.
- Campo nuevo en `feature_list.json`: `prd` (string, ruta canonica). Es
  **opcional**: las 13 features existentes no lo tienen y siguen validas; el
  escritor de `feature_list.json` ya preserva claves desconocidas y su orden.
- disparador: `prd add` (nace un PRD hijo) y `add --prd` (una feature se cuelga
  de un PRD hoja).
- interruptor: la ausencia de `docs/prd/` apaga todo el bloque nuevo (check y
  arbol) sin fallar; una feature sin `--prd` sigue el camino de siempre.
- candado: `prd add` se niega si el destino ya existe, y el enlace en el padre no
  duplica una fila ya presente (re-correr el comando no ensucia el documento).

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->

```
CUANDO alguien corre  prd add --name <slug> [--parent <ruta>]

  ¿el padre existe como PRD?        -> si no, error + lista de PRDs disponibles
  ¿el destino ya existe?            -> si si, error (no pisamos documentos del usuario)
  ¿el slug queda vacio al normalizar? -> si si, error de uso

  ENTONCES creamos la carpeta del segmento y dentro el PRD del hijo,
           con las 12 secciones del metodo y su Padre declarado,
           y enganchamos una fila en la seccion "PRDs anidados" del padre,
           sin tocar una sola linea mas de ese documento.

CUANDO alguien corre  add --name <feature> --prd <ref>

  ¿<ref> resuelve a un PRD unico?   -> si no, error listando candidatos
  ENTONCES guardamos la ruta canonica en la feature,
           y el spec que nazca de ella citara ese PRD en su encabezado.

CUANDO se cierra una feature como done

  ¿tiene PRD de origen (o cuenta para el maestro)? -> si no existe el archivo, [i] y seguimos
  ¿su tabla de hitos tiene la fila de esta feature? -> si si, la marcamos done + fecha
  ¿ya estaba registrada en la bitacora?             -> si si, no escribimos de nuevo

  ENTONCES dejamos una linea de bitacora con spec e impl,
           y NUNCA tocamos el cuerpo del PRD: eso lo escribe el usuario.

CUANDO corre el check

  ¿cada PRD esta donde dice su cadena de segmentos?
  ¿cada carpeta tiene su PRD?
  ¿cada encabezado Padre coincide con la ubicacion real?
  ¿cada feature con prd apunta a un PRD que existe?
     -> cualquier "no" es un [!] que suma a failures
  ¿un PRD sin hitos?  -> [i] informativo, no bloquea
```

Promesas: nunca reescribe un PRD existente · el unico campo nuevo en
`feature_list.json` es opcional · sin `docs/prd/` el arnes se comporta como hoy ·
el arbol se lee sin correr nada (son carpetas y archivos).

## No funcionales
- SLOs: el arbol se recorre con un walk acotado a `docs/prd/` (decenas de
  archivos); `prd tree` y el gate leen solo encabezado y tabla de hitos. Sin
  dependencias nuevas.
- Seguridad: `prd add` solo escribe dentro de `docs/prd/`; el slug se normaliza
  antes de tocar el filesystem (sin `..`, sin separadores, sin rutas absolutas).
- Observabilidad: cada comando imprime la ruta creada y el archivo enlazado; el
  gate usa el mismo formato `[!]` / `[i]` que el resto de `harness_check.sh`.
- Multi-LLM: los tres comandos viven en el binario, iguales para cualquier
  backend; la guia y las superficies los describen sin nombrar un LLM.

## Fuera de alcance
- Mover o renombrar PRDs con un comando (`prd move` / `prd rm`): el gate avisa si
  el arbol quedo incoherente, pero reacomodar es trabajo manual en esta feature.
- Reescribir el CUERPO del PRD con lo que quedo implementado (historia, datos,
  pseudo-codigo): el arnes marca el hito y deja bitacora (AC-17), pero el texto
  del documento lo actualiza el USUARIO. Un gate que exija esa actualizacion
  (avisar si un hito cerro y el PRD no se toco desde entonces) queda para otra
  feature.
- Anidar el SDD: `SDD-master.md` sigue siendo unico y plano.
- Migrar los PRDs planos que alguien ya haya creado a mano: el gate los reporta,
  el usuario decide.
- Editar `roles/*.md` (obligaria a regenerar los espejos con el instalador dentro
  del checkout fuente y dejaria el gate de espejo stale, mismo criterio que #12).

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->

- Layout, creacion, cadena y validacion: DECIDIDO por el usuario (2026-08-12) en
  las cuatro preguntas previas al spec — (1) carpetas anidadas reales bajo
  `docs/prd/`, (2) comando `prd add` que crea desde plantilla y enlaza en el
  padre, (3) `add --prd` con campo en `feature_list.json` y encabezado `PRD:` en
  el spec, (4) `prd tree` + gate de integridad en `harness_check.sh`.
- PROPUESTA a confirmar en la aprobacion (deriva de la opcion elegida en (1)): la
  identidad de un PRD es su cadena de segmentos; la carpeta lleva el segmento
  propio (`cobranza/mora/`) y el archivo la cadena completa
  (`PRD-cobranza-mora.md`), tal como el ejemplo elegido. Asi el nombre de archivo
  es unico en todo el repo (greppable, sin ambiguedad entre ramas) y la carpeta
  queda corta.
- PROPUESTA a confirmar: `--prd` acepta el segmento final (`mora`) cuando es
  unico en el arbol, ademas de la ruta completa. Ergonomia sin ambiguedad: si dos
  ramas tienen `mora`, el comando falla y pide la ruta.
- ENMIENDA post-aprobacion: vuelta del cierre al PRD — DECIDIDO por el usuario
  (2026-08-12), a su pregunta "¿los PRD se van actualizando con lo que en
  realidad quedo implementado?". Se agrega AC-17: `close --status done` marca el
  hito en la tabla del PRD de origen y deja una linea de bitacora con spec e
  impl; el cuerpo del PRD nunca se reescribe solo. Descartadas las otras dos
  opciones: solo lectura (el PRD se pudre en "pendiente") y el gate de
  divergencia por mtime (queda para otra feature).
- Los ciclos y los slugs duplicados que mencionaba la pregunta (4) son
  imposibles por construccion con carpetas anidadas (un arbol de directorios no
  cicla y dos hermanos no comparten nombre), asi que el gate cubre lo que si
  puede romperse: archivo movido, carpeta sin PRD, encabezado que miente y
  feature apuntando a un PRD inexistente.
