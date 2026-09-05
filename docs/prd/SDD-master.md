# SDD Master - Harness Process

Estado: en uso
Ultima actualizacion: 2026-08-18
Producto: docs/prd/PRD-master.md
Constitution: docs/constitution.md

> Documento del USUARIO: el instalador lo siembra una sola vez y nunca lo pisa.
> Es el "como" maestro del proyecto: la arquitectura objetivo y las decisiones
> tecnicas que ninguna feature individual deberia re-litigar.
>
> Relacion con los otros documentos:
> - `docs/prd/PRD-master.md`: que se construye y por que.
> - Este archivo: como se construye, a nivel proyecto.
> - `docs/architecture.md`: el mapa de lo que YA existe (se actualiza solo).
> - `docs/spec-feature-*.md`: el detalle por feature, con sus AC-n.

## 1. Arquitectura objetivo

<Diagrama en texto o descripcion de los componentes y como se hablan. Marca que
existe hoy y que hay que construir.>

```
<componente A> --(<protocolo>)--> <componente B>
```

| Componente | Responsabilidad | Estado | Owner |
| --- | --- | --- | --- |
| <servicio> | <que hace, en una linea> | <existe / a construir> | <equipo> |

## 2. Stack y dependencias

- Lenguajes / runtimes: <...>
- Almacenamiento: <base de datos, colas, cache>
- Servicios externos: <APIs de terceros, con su modo de falla>

> Regla sugerida (ajusta en la constitution): ninguna dependencia nueva de
> runtime entra sin una decision registrada abajo.

## 3. Contratos entre componentes

<Interfaces que cruzan un limite de equipo o de servicio: endpoints, eventos,
esquemas. Un cambio aqui impacta a otros; se registra impacto antes de mergear.>

| Contrato | Productor | Consumidores | Versionado |
| --- | --- | --- | --- |
| <endpoint/evento> | <servicio> | <servicios> | <como se versiona> |

## 4. Decisiones tecnicas

**Un solo parser por formato** (feature #67). Habia CUATRO parsers de bloques de
codigo markdown con TRES semanticas sobre los mismos documentos, y el costo no
era teorico: `verify` ejecutaba los `Comando:` escritos dentro de un bloque
`~~~` —ejecucion de shell salida de una seccion que el autor marco como
documentacion, el bug que la #23 cerro para backticks y seguia abierto para
tildes— y `revision --veredicto` borraba prosa del reviewer o dejaba dos sellos
contradictorios segun la paridad de fences ajenos citados. Tres decisiones:

- **Una sola implementacion, en `markdown.rs`.** Fences EMPAREJADOS: se recuerda
  cual abrio el bloque y solo lo cierra el mismo. Es la unica de las tres
  semanticas que coincide con como se renderiza el markdown de verdad.
- **Se devuelve la clasificacion completa, no una lista filtrada.** Cada
  consumidor necesita algo distinto de la MISMA respuesta —el gate quiere lo de
  afuera, el limpiador necesita todas las lineas para reescribir conservando los
  fences, el parseo de AC quiere todo lo que no sea contenido—. Un `Vec<&str>`
  compartido no alcanzaba, y eso fue exactamente lo que hizo que cada uno se
  escribiera el suyo.
- **La regla se hace cumplir sola.** `tests/conventions_check.sh` gana el modo
  `un-solo-parser`, que se pone rojo ante un quinto. La unica exencion es una
  implementacion vieja conservada para poder medir contra ella, declarada por
  linea con `PARSER-VIEJO-A-PROPOSITO` y NOMBRADA en la salida del check: eximir
  los tests enteros dejaria que un parser de verdad se esconda en uno, que es
  justo lo que paso con el cross-check de `verificacion.rs`.

**Un gate solo verifica lo que puede ejecutar** (feature #46). El comando que
mejor prueba una feature suele ser el mas verboso, y era justo el que no se
podia declarar: `verify` leia los pipes DESPUES de esperar al proceso, asi que
cualquier comando que pasara los ~64 KB del buffer trababa a los dos. Tres
decisiones:

- **Leer mientras corre, no despues.** Un hilo por descriptor, lanzado antes de
  esperar. Es la unica forma de que el productor no se bloquee.
- **Ningun camino puede esperar sin limite.** El timeout corta al proceso y una
  gracia corta corta a los lectores: si un nieto heredo el pipe, se reporta lo
  leido y se sigue. Cambiar un cuelgue por otro es no haber arreglado nada.
- **Lo que se recorta se declara.** Tope de 4 MB reteniendo la cola —donde estan
  los resumenes que deciden el estado— y una linea en el reporte diciendo
  cuanto quedo afuera y sobre que se midio.

**El arnes no se bloquea a si mismo, tambien en el guard** (feature #58).
`docs/rutas-protegidas.md` ya declaraba la regla —"la proteccion es contra las
herramientas del agente, no contra el binario"— pero el commit guard no la
aplicaba, y en un proyecto donde `docs/` es su propio repo eso bloqueaba el
turno en cada documento que el arnes escribia. Dos decisiones:

- **La exencion es por ARTEFACTO y por UBICACION, nunca por carpeta.** Un
  `docs/runbook.md` sigue bloqueando, y un `impl-notas.md` dentro de un
  microservicio tampoco se exime: el nombre solo no alcanza. Un gate que se
  relaja de mas es peor que uno estricto, porque nadie revisa lo que cree
  cubierto.
- **Cuando un gate se saltea algo, lo dice.** Una linea `[i]` con el repo y la
  razon. Un guard que se calla en silencio es indistinguible de uno apagado.
- **Y cuando bloquea, ofrece una salida que el que lee PUEDA tomar** (feature
  #66). El guard nombraba el repo y nada mas ("Cambios sin commitear en: docs"),
  con dos remedios: commitear —trabajo que puede ser de otra sesion, a ciegas— o
  apagar el guard para todo el repo. Ahora nombra los archivos no exentos y
  agrega la salida que faltaba: si no es tuyo, decilo y no lo commitees.

**Un gate del fin de turno no puede quedarse sin salida** (feature #66). La
diferencia entre un gate y una trampa es si existe una accion que lo satisfaga.
Cuando lo que falla no depende del agente —un repo hermano sucio de otra sesion,
un espejo de rol cuyo remedio es re-correr el instalador, un spec en draft que
EXIGE el si del usuario— cada intento de cerrar el turno lo volvia a disparar.
`harness_check.sh` bloquea la PRIMERA vuelta —la unica chance del agente de
arreglar lo que SI es suyo— y degrada la segunda: imprime TODO (mas, no menos),
dice que no lo puede resolver solo, y deja cerrar.

La señal de "segunda vuelta" llega por dos caminos, y el segundo existe porque el
primero es prestado: `HARNESS_STOP_HOOK_ACTIVE`, que sale del JSON del evento
pero lo manda el CLI (de Claude y Kimi hay evidencia; de Codex, Gemini y Grok no
hay ninguna), y `progress/.stop_streak`, el centinela propio que se da cuenta
solo cuando el MISMO conjunto de fallos se repite. La firma es del conjunto y no
de la cantidad, asi que un problema nuevo reinicia la racha y vuelve a bloquear.

**Una defensa que depende de que el otro se acuerde de avisar no es una
defensa.** Y el bug de origen fue de la misma familia: habia DOS escritores de
hooks y uno no se entero del contrato —cinco superficies pasaban por
`bin/harness-hook` y `.claude/settings.json` en POSIX no—, asi que
`tests/parity_check.sh` gana un modo que lo impide.

**El material se entrega y el vacio se dice** (feature #56). La #51 dejo de
hacer que el REVISOR explorara; esta hace lo mismo con el que IMPLEMENTA, y
agrega la parte que faltaba: avisar cuando no hay nada que entregar. Tres
decisiones que valen para cualquier feature futura que le de contexto a un
agente:

- **Los punteros se siguen y se verifican.** Un `architecture.md` que apunta a
  otro archivo se resuelve contra el directorio del documento, y si el destino
  no existe eso es un HUECO con la ruta que falta — un diagnostico distinto de
  "no hay mapa". Un puntero roto se lee como "aca no hay nada escrito" y manda a
  explorar el repo entero.
- **El vacio se declara, no se disimula.** Si el mapa no menciona el tema, el
  paquete lo dice con esas palabras y con los terminos que busco, para que un
  falso positivo se pueda diagnosticar de un vistazo. Y si la consulta no tiene
  terminos utiles, el aviso apunta a la consulta, no al mapa.
- **El aviso no depende de que alguien lo pida.** `start` imprime el resumen
  siempre, porque el caso donde mas importa —el paquete vacio— es justo el que
  nadie pediria (`promesas-estructurales-vs-disciplina`).

**El MCP se instala por proyecto; la autorizacion es del usuario** (feature #52).
`atlassian drain` imprime un plan de llamadas MCP desde la feature #15, pero el
arnes nunca instalaba el MCP que ese plan asume. Tres decisiones que valen para
cualquier integracion futura por MCP:

- **Alcance de proyecto, nunca global.** El instalador escribe la configuracion
  MCP DEL REPO (`.mcp.json`, `.kimi-code/mcp.json`, `.grok/config.toml`) y para
  el backend que no admite alcance de proyecto (Codex) imprime el comando en vez
  de tocar la configuracion global del usuario. Instalar un arnes en un repo no
  cambia como se comportan sus herramientas en los demas.
- **El arnes no hace el OAuth y no lo finge.** Dice, por CLI, que comando correr
  y deja claro que esa parte es del usuario.
- **Las rarezas de cada backend se reproducen y se escriben.** Grok necesita el
  bridge `mcp-remote`; Codex necesita el plugin `atlassian-rovo` ADEMAS del
  servidor. Las dos se verificaron contra los CLIs instalados y quedaron en el
  spec como hallazgos, no como deducciones.

**Que revisar no cueste una fortuna** (feature #51). Verificar lo implementado
llego a costar 10 millones de tokens, casi todos gastados explorando el repo.
Dos decisiones que valen para cualquier feature futura que involucre a un
agente revisando:

- **El material se entrega, no se busca.** `revision --feature <id>` arma el
  paquete (AC + estado de verify + evidencia + archivos + diff + rutas
  protegidas) acotado por presupuesto, declara lo que recorta y reporta su
  propio tamaño antes de que alguien lo lea.
- **Un modelo por rol, en la tabla de roles de cada instalador.** El que escribe
  codigo piensa con Opus; el que planifica y el que revisa, con Fable; los tres
  en `xhigh`. `.claude/agents/*.md` es artefacto generado: editarlo a mano no
  sobrevive a la instalacion.

**Aislamiento entre features** (feature #47). Dos implementaciones simultaneas
no comparten archivos: cada feature vive en su rama GitFlow y en su worktree
hermano del repo. Tres decisiones que valen para cualquier feature futura que
toque el flujo:

- **El estado del arnes es unico.** `feature_list.json` y `progress/` se
  resuelven contra el repo PRINCIPAL (`git rev-parse --git-common-dir`) aunque
  el binario se invoque desde un worktree: el backlog no se bifurca nunca.
- **Los docs se resuelven DESDE la feature, no desde el directorio actual.**
  `HarnessPaths::para_feature()` apunta `docs/` al worktree de esa feature, para
  que su spec, su plan y su evidencia viajen con el merge de su rama.
- **Salvo lo que es de TODAS las features** (feature #60). El PRD es un
  documento raiz y compartido: la vuelta al cierre (marcar el hito, dejar
  bitacora) se escribe en el `docs/prd/` del checkout PRINCIPAL y DESPUES de
  integrar. Guardar un log compartido dentro de una rama por feature hacia que
  dos cierres en paralelo apendearan al final de la misma seccion: el merge
  conflictuaba y la linea se perdia en la resolucion (7 de 18 cierres). La
  pregunta que decide donde va un documento es de quien es el dato, no desde
  donde se escribe.
- **El arnes nunca reescribe historia ni elige la rama destino.** Sin `--force`,
  sin rebase, sin squash y sin borrar ramas; el merge corre en un worktree
  temporal (no toca tu checkout) y `--to` lo decide el USUARIO.
- **Y esa promesa no tiene excepciones** (feature #61). El worktree temporal se
  crea con `--detach`, asi que vale tambien cuando el destino es la rama que el
  usuario tiene abierta — el caso mas comun, y el que antes se colaba por una
  excepcion justificada en un limite de git que nadie volvio a comprobar. La
  rama destino se avanza despues con `reset --keep` (conserva lo que el usuario
  tenga sin commitear) o con `update-ref` con guarda de valor viejo. El unico
  caso irreductible —el merge cambia un archivo que el usuario tiene sucio— se
  DETECTA antes de commitear o mergear y detiene el cierre nombrando los
  archivos: el arnes no elige entre su merge y el trabajo ajeno.
- **Y nada del ESTADO se escribe hasta que la integracion ocurrio** (feature
  #62). El cierre corre en cuatro fases: lo que puede negarse (gates, `--to`,
  colisiones), los artefactos que tienen que viajar en la rama (la anotacion del
  plan y el estado archivado, idempotentes porque el merge borra el worktree
  donde viven), la integracion, y recien despues el estado (backlog, Atlassian,
  `progress/`, `history.md`, memorias y el mensaje de exito). No hay rollback a
  proposito: quedaria parcial —un intent emitido a Jira y una memoria escrita en
  el hub no se deshacen— y habria que acordarse de mantenerlo cada vez que el
  cierre gane un efecto nuevo. La regla que vale para cualquier comando futuro
  del arnes: **los efectos que no se pueden deshacer van ultimos**.

**Como habla el arnes con un LLM** (feature #28, la unica parte que usa modelo).
La cadena es `HARNESS_CONSOLIDAR_CMD` -> primer CLI de una tabla corta
(`claude -p`, `kimi -p`) -> **skip limpio**. Apagada por default y de forma
estructural: sin `rules.consolidar_backend` no se resuelve backend ni se mira el
entorno. Tres decisiones que valen para cualquier feature futura con modelo:

- **Al modelo se le manda lo minimo.** Ve nombre, descripcion y triggers; nunca
  el cuerpo de una leccion. Lo peor que puede hacer es equivocarse, no filtrar.
- **El prompt viaja como item de argv, jamas por `sh -c`.** Por eso NO se reusa
  `verificacion::ejecutar`, que si corre con shell.
- **El modelo propone; lo que muta sale de argv.** La mitad que escribe se
  verifica sin backend y de forma determinista.

El tramo HTTP con API key **no esta implementado** y el mensaje de skip lo dice.

<Una fila por decision estructural. Si una decision es grande o polemica, mueve
el detalle a su propio ADR y enlazalo aqui.>

| # | Decision | Alternativas descartadas | Por que | Fecha |
| --- | --- | --- | --- | --- |
| D1 | El aislamiento de una feature se DECIDE en una funcion pura (`aislamiento::decidir`) y se ejecuta despues; un arranque que no lo consigue no arranca | (a) avisar con `[i]` y seguir, como estaba; (b) una regla `require_aislamiento` apagada por defecto | Avisar y seguir dejo tres features `in_progress` sin rama ni worktree escribiendo en el mismo checkout. Una regla opcional habria repetido el problema en toda instalacion que no la active. Separar decidir de ejecutar es lo que impide volver al fallback: la parte que decide no tiene con que continuar | 2026-09-05 |
| D2 | La publicacion del cierre pasa a ser explicita (`close --publicar`); sin el flag el merge queda local y se imprime el comando | (a) seguir publicando siempre; (b) preguntar interactivamente | Un `push` automatico despues del merge publico un commit que se habia acordado dejar local, porque era el padre del que si iba. Preguntar no sirve: el cierre corre en hooks y en sesiones sin nadie mirando | 2026-09-05 |
| D3 | El sello de cierre se escribe en el `docs/` de la RAIZ y en la fase del estado, no en el `docs/` de la feature ni en la fase de los artefactos que viajan en la rama | (a) escribirlo en los dos lados; (b) dejarlo donde estaba y arreglar solo el mensaje | Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra, y ahi vive la unica copia del estado vivo. Dos copias divergen —la familia de bug mas repetida de este repo— y arreglar solo el mensaje lo dejaba en una rama que nadie mergea. Movido el lugar, la fase pudo bajar: estaba en la fase 1 por una razon fisica que dejo de existir | 2026-09-05 |

## 5. Datos

**Los estados de una feature** (`feature_list.json`). Son seis y significan
cosas distintas: `pending` (sin empezar, `next` la ofrece), `in_progress`
(pueden convivir varias, cada una en su worktree, desde la #47), `done` (hecha,
con spec aprobado y su evidencia), `blocked` (trabada por algo externo),
`superseded` (el trabajo se hizo en OTRA feature, que se nombra en
`superseded_by` y se valida al cerrar) y `resuelto-aguas-arriba` (el trabajo se
hizo en OTRO PROYECTO, que se nombra en `resuelto_en` con la forma
`<proyecto>/feature-<id>`, feature #65). Solo `done` pasa por los cinco gates de
cierre; `superseded` y `resuelto-aguas-arriba` no cuentan ni en el numerador ni
en el denominador de `prd tree`, porque no son trabajo hecho ni pendiente.

De la referencia externa se comprueba **la forma y nada mas**, y `status` lo dice
literal ("sin verificar"): la feature de aguas arriba vive en un repo que el
arnes no puede abrir, y validar su existencia seria prometer enforcement que no
se hace (feature #64). Por la misma razon el cierre no transiciona el ticket de
Atlassian: mandarlo a `done` afirmaria que este proyecto lo entrego, y dejarlo
caer en el brazo por defecto lo REABRIRIA.

**Un estado nuevo tiene que decidir en cada consumidor** (feature #65). El campo
es un `&str` comparado por igualdad en varios lugares, asi que un estado que no
declara su rama en alguno cae en el brazo por defecto de ese consumidor — que en
Atlassian significa reabrir el ticket. Las decisiones dejan de vivir en `match`
inline y son produccion consultable (`close::ESTADOS_DE_CIERRE`,
`emit::efecto_de`, `prd::cuenta_en_el_avance`, `status::ESTADOS_CON_BUCKET`), y
un test recorre la tabla completa —estados x consumidores— contra ellas. Un
estado que no se agregue a la tabla no compila; uno que se agregue sin decidir
que hace cada consumidor, falla.

**El quinto gate: el veredicto del reviewer** (feature #64). `require_review`
exige que `docs/review-<id>.md` lleve la linea que estampa
`revision --veredicto`, y que esa linea diga `approved`. La decision estructural
es CUAL es la prueba: el gate no parsea la prosa del review, porque la escribe el
mismo agente que quiere cerrar —y ya habia un `Veredicto: approved (implementacion)
- cierre BLOQUEADO` en disco, que un `contains("approved")` habria aprobado. Dos
Y conviene ser exacto sobre cuanto aguanta cada barrera, porque la primera
version de este texto prometia de mas y el reviewer lo desmintio con un `printf`
de cuatro lineas:

- **El sello** lo escribe solo el binario, pero es texto: un agente decidido lo
  tipea. **Filtra el descuido, no la mala fe.**
- **La cobertura por AC** es la que aguanta: una fila por cada AC-n del spec,
  cada una citando `archivo:linea` **que resuelve** (el archivo existe y tiene
  esa linea), verificada al estampar Y de nuevo en el cierre. Eso sube el costo
  de fabricar un review falso de cinco segundos a leer el codigo. No lo vuelve
  imposible: lo que el arnes NO comprueba es que la cita sea PERTINENTE al AC.
- **Y cuando una cita no resuelve, el gate dice contra QUE resolvio** (feature
  #70). Antes contestaba lo mismo para dos casos distintos —"un archivo que exista
  y una linea que exista en el"— sin nombrar ninguna de sus raices, asi que un
  reviewer que citaba un repo hermano probaba `../repo/archivo:353` y la ruta
  absoluta, las dos se rechazaban por la FORMA, y el mensaje lo mandaba a buscar
  un archivo que estaba donde el creia. Lo que hacia entonces era citar el
  documento que el mismo habia escrito en la columna que el gate comprueba: el
  gate satisfecho con una cita que no era la evidencia. Ahora separa "la forma de
  la ruta no se acepta" de "no se encontro", lista las raices en orden, y ofrece
  la forma de citar un repo hermano **solo en los layouts donde resuelve** — un
  remedio que no funciona es peor que ninguno.

El corolario general, que vale mas que el mecanismo: **una barrera se documenta
por lo que filtra, no por lo que uno quisiera que filtrara.** Un gate descrito de
mas es un gate en el que se confia de mas.

**Y una regla que no gatea no se queda en `rules`** (feature #64).
`require_tests_to_close`, `require_impact_check` y `one_feature_at_a_time`
estuvieron en `true` sin que ningun codigo las leyera —ni en Rust ni en el
Python previo al port: nacieron decorativas— y la tercera ademas contradecia a
`start.rs` desde la #47. Se borraron del molde. El corolario para el futuro: una
clave en `rules` es una promesa de enforcement, y el arnes no promete lo que no
hace.

- Entidades principales y su dueno: <...>
- Migraciones: <como se aplican y como se revierten>
- Retencion y datos sensibles: <que se guarda, cuanto tiempo, con que proteccion>

## 6. No funcionales

- SLOs: <latencia, disponibilidad, throughput objetivo>
- Seguridad: <autenticacion, autorizacion, manejo de secretos>
- Observabilidad: <logs, metricas, trazas; que se alerta y a quien>
- Costos: <limites o presupuesto que condicionan el diseno>

## 7. Estrategia de verificacion

<Como se prueba el sistema, mas alla de los tests de cada feature. Los comandos
concretos viven en `docs/verification.md`.>

- **Tests automaticos**: unitarios en `rust/src/**` (modulos `mod tests`, sobre
  todo para las funciones PURAS: parsear, planificar, diagnosticar, decidir),
  integracion en `rust/tests/cli_basics.rs` (el binario de verdad contra un
  sandbox `tempfile`), y chequeos de shell en `tests/*.sh` para lo que vive
  fuera de Rust (los dos instaladores, los espejos, el corpus real del repo).
- **Los AC se ejecutan**: cada AC-n de un spec puede declarar `Comando:`, y
  `harness_cli verify --feature <id>` los corre y escribe `docs/verify-<id>.md`.
  Con `require_verify_green`, `close --status done` LEE ese reporte —nunca
  ejecuta— y no deja cerrar con alguno bloqueando.
- **Un AC que el parser no ve no existe para nadie**, y eso incluia justo a los
  que pedian una persona (feature #68). `ac_de` exigia los dos puntos PEGADOS a
  los digitos, asi que `- AC-11 (MANUAL):` —la anotacion que marca "esto lo tiene
  que auditar alguien"— hacia desaparecer el AC de `verify` Y del gate del
  review, que saca su lista del mismo parser. Medido: siete AC invisibles en 55
  specs, en dos familias (`(MANUAL)` y el sufijo de letra `AC-4b`). La marca de
  "esto no lo puede comprobar la maquina" era exactamente lo que hacia que nadie
  tuviera que comprobarlo. La gramatica del nombre es ahora
  `AC-<digitos><letras?>` mas una anotacion opcional entre parentesis que NO
  entra en el nombre, y se afloja **lo justo**: un parser que inventa un AC es
  peor que uno que lo pierde, porque hace fallar cierres que estaban bien. Y cuando
  una linea DICE ser un AC y no se puede leer, el arnes la **nombra** en vez de
  descartarla: `verify` la imprime con su texto antes de correr nada y el gate del
  review se niega (feature #69). Un criterio que desaparece por un typo es la
  misma clase de perdida que uno que desaparece por una anotacion, y el silencio
  es lo que la vuelve cara: el autor se entera —si se entera— en el review.
- **Un `Comando:` nunca se cuelga del AC equivocado.** `parsear` lo asocia al
  ultimo AC abierto, asi que un encabezado ilegible le regalaba su comando al AC
  de arriba: reproducido, `AC-1` se quedaba con el `touch` que era del `AC-2`, y
  `verify` habria impreso "AC-1 verde" tras correr la prueba de otro criterio.
  Una linea que arranca como AC y no se puede leer cierra el anterior: vale mas
  perder un comando que adjudicarselo al criterio equivocado.
- **Un AC que no midio nada no cuenta como verificado**: `cargo test <nombre>`
  con un filtro que no matchea sale 0, y eso ya produjo un falso verde real. Por
  eso `verify` mira la salida ademas del exit code y marca `vacio` al AC que
  reconocidamente no ejecuto ningun caso. Sobre salidas que no son de libtest no
  opina: el estado no cambia.
- **Y el andamiaje que no puede medir se pone ROJO, no verde** (feature #63).
  Un test que corta por tiempo con `timeout(1)` no mide nada en macOS —no viene
  con el sistema— y salia verde igual: el codigo 127 de "no existe" no era el
  124 de "se corto". La regla que queda: cuando una prueba depende de una
  herramienta externa, se elige entre varias (`timeout`, `gtimeout`,
  `perl alarm`), se PRUEBA el mecanismo elegido contra un caso que falla y uno
  que no, y si no hay ninguno se falla nombrando cual instalar. Un skip verde es
  la forma mas cara de no enterarse.
- **Entornos**: local y un runner Windows en CI (feature #59). El arnes no se
  despliega; se instala en el repo del proyecto con `setup_harness.sh` / `.ps1`,
  y `tests/parity_check.sh` verifica que los dos DECLAREN lo mismo. Lo que la
  paridad no puede probar desde macOS o Linux —que `setup_harness.cmd` de verdad
  arranque el `.ps1` y le traduzca los flags— lo ejecuta
  `.github/workflows/windows-cmd-installer.yml` en `windows-latest`, y
  `tests/cmd_installer_check.ps1` se NIEGA a correr fuera de Windows en vez de
  informar un skip verde.
- **Criterio de "listo"**: los AC del spec en verde en su reporte, la suite
  completa y `cargo clippy -D warnings` limpios, los chequeos de `tests/` en
  verde, y `harness_check.sh` sin problemas.

## 8. Riesgos tecnicos

| Riesgo | Probabilidad | Impacto | Mitigacion |
| --- | --- | --- | --- |
| <riesgo> | <alta/media/baja> | <alto/medio/bajo> | <plan> |

## 9. Decisiones abiertas

<Igual que en el PRD: sin decision registrada, se pregunta al USUARIO antes de
implementar lo que dependa de ella.>

- <pregunta> — DECIDIDO (<usuario>, <fecha>): <respuesta>
- <pregunta> — ABIERTA
