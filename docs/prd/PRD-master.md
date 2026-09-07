# PRD Master - <nombre del proyecto>

Estado: draft
Duenno: <quien responde por este documento>
Ultima actualizacion: <YYYY-MM-DD>
Alcance: <en una linea: que abarca este producto y que NO toca>
Como se escribe: docs/prd/COMO-ESCRIBIR-UN-PRD.md
Diseno tecnico: docs/prd/SDD-master.md
Constitution: docs/constitution.md

> Documento del USUARIO: el instalador lo siembra una sola vez y nunca lo pisa.
> Es la fuente de la que salen las features del backlog: cada hito de la seccion
> "Hitos" se convierte en una entrada de `feature_list.json`, y de ahi en un
> `docs/spec-feature-<id>-<slug>.md` con sus AC-n.
>
> Para un proyecto que arranca de cero, completa este archivo ANTES de cargar la
> primera feature. Borra los ejemplos entre <> a medida que los reemplazas.
> Si no sabes cuanto escribir ni por donde empezar, lee primero
> `docs/prd/COMO-ESCRIBIR-UN-PRD.md`.

---

**LA REGLA DURA: SIN CODIGO. SOLO PSEUDO-CODIGO.** Este documento fija la
**estructura** — la historia, que entidades se tocan y como cambian — en
pseudo-codigo y explicaciones. Nunca lleva codigo final, la implementacion
exacta, pantallas terminadas ni configuracion. Eso se escribe despues, en otra
parte. Si la estructura esta bien en papel, el codigo es la parte facil; si esta
mal, ningun codigo la arregla.

---

## 1. Resumen (hoy -> despues)

<El dibujo mas barato que existe: dos lineas. Si no podes escribirlas, todavia
no entendes el cambio.>

- **Hoy:** <que pasa hoy, y que no pasa>
- **Despues:** <que pasa cuando esto exista>

## 2. La historia

<El corazon del documento. Tiene que poder contarse en palabras, sin
tecnicismos, con una persona con nombre y un momento concreto. Si la historia no
convence, el resto no importa.>

**ANTES**

<Marta cerro su compra un viernes a las 6 de la tarde. Nadie la llamo. El lunes
le llego la misma plantilla de siempre, y esa confianza recien ganada se enfrio
justo cuando mas cerca estaba de recomendarnos.>

**DESPUES**

<Cinco segundos despues de cerrar, suena su telefono: la saludan por su nombre y
le agradecen la confianza. Marta cuelga sonriendo — y esa misma semana trae a
una amiga.>

> ASI NO: "escuchar el cambio de estado", "agendar una tarea de llamada",
> "disparar el agente de voz". Eso es implementacion, no historia.
> ASI SI: quien es el usuario, como lo usa, cual es el dolor y cual es la
> experiencia que quiere vivir. Todo lo demas en este documento existe para
> hacer esa historia realidad.

## 3. Objetivos / No-objetivos

<Con nombre y apellido: las secciones siguientes los citan ("cumple O2"). Los
no-objetivos frenan el "ya que estamos...".>

| ID | Objetivo | Como se ve cumplido |
| --- | --- | --- |
| O1 | <lo que tiene que lograr> | <senal observable> |
| O2 | <...> | <...> |

| ID | No-objetivo | Por que no |
| --- | --- | --- |
| NO1 | <lo que explicitamente NO se hace> | <razon> |

## 4. Usuarios y jobs-to-be-done

| Usuario | Que intenta lograr | Como lo resuelve hoy | Por que no alcanza |
| --- | --- | --- | --- |
| <rol> | <job> | <workaround actual> | <limitacion> |

## 5. Metricas de exito

<Como sabras que funciono, en numeros. Cada metrica con su valor de partida y su
objetivo, y el objetivo O-n que mide. Sin metrica no hay forma de cerrar el
proyecto.>

| Metrica | Hoy | Objetivo | Mide | Como se mide |
| --- | --- | --- | --- | --- |
| <ej. tiempo de alta de un cliente> | <45 min> | <5 min> | <O1> | <log/dashboard> |

## 6. Como funciona hoy -> como va a funcionar

<El flujo, dibujado dos veces. Dibujar el HOY obliga a reusar lo que ya existe
en vez de inventar arquitectura nueva.>

```
HOY                          DESPUES
<evento> -> (nada)           <evento> -> <lo que se agenda>
                                  |__ <componente> llama a <componente>
                                            |__ <donde se guarda el resultado>
```

## 7. Los datos

<El plano de los datos a nivel PRODUCTO: que dispara el flujo, que interruptor
lo apaga por cliente y que candado evita que pase dos veces. Entidades y campos
en palabras; el esquema fisico vive en `docs/prd/SDD-master.md`.>

| Que | Entidad / campo | Para que |
| --- | --- | --- |
| disparador | <el lead pasa a estado «venta cerrada»> | <que arranca el flujo> |
| interruptor | <cliente.<flag>: 'apagado' \| 'prueba' \| 'activo'> | <apagar por cliente en 1 clic> |
| candado | <lead.<campo>_en: fecha> | <evitar repetir la accion> |

## 8. Pseudo-codigo (el acuerdo)

<La receta, en palabras: que lo dispara, que lo frena y que promete — sin una
sola linea de codigo. Este es el acuerdo a nivel producto; cada feature refina
el suyo, y el detalle vinculante de cada cambio vive en su
`docs/spec-feature-<id>-<slug>.md`.>

```
CUANDO <ocurre el disparador>

  ¿<el cliente activo la funcionalidad>?  -> si no, no hacemos nada
  ¿<ya lo hicimos para este caso>?        -> si si, no hacemos nada
  ¿<tenemos lo minimo para actuar>?       -> si no, no hacemos nada

  ENTONCES <que hacemos, en una frase>,
           con <la restriccion que lo hace aceptable>.
```

**Promesas:** <una sola vez por caso> · <nunca fuera de horario> · <si no
contesta, no insiste>.

## 9. Restricciones y supuestos

- Tecnicas: <stack obligado, sistemas con los que hay que integrar>
- Negocio / legales: <plazos, normativa, contratos>
- Supuestos: <lo que damos por cierto y habria que validar; si un supuesto cae,
  el alcance cambia>

## 10. Hitos -> features

<Cada fila se carga al backlog con:
 sh harness_cli add --name <slug> --service <servicio> --acceptance "<criterio>" [--prd <ruta>]
y al arrancarla (`start`) genera su spec con AC-n, citando su PRD de origen. Al
cerrarla (`close --status done`) el arnes marca aca su Estado y deja bitacora.
Si un hito no entra en una historia sola, no lo escribas aca: dale su propio PRD
anidado con `sh harness_cli prd add --name <parte>`.>

| # | Hito | Slug de feature | Objetivo que cumple | Criterio de aceptacion (resumen) | Estado |
| --- | --- | --- | --- | --- | --- |
| 1 | AC-n ejecutables: el spec declara como se verifica cada criterio | ac_ejecutables_verify | <O1> | `harness_cli verify` corre el comando de cada AC-n, escribe `docs/verify-<id>.md` y `require_verify_green` bloquea el cierre si alguno no pasa; los AC sin comando siguen siendo validos | done (2026-08-17) |
| 2 | Escalera de huella y politica de tests en las convenciones | conventions_escalera_y_tests | <O1> | `docs/conventions.md` (+ espejo) lleva la escalera de menor huella y las tres reglas de test (contratos y no snapshots, prohibido leer el fuente en un test, prohibido el detector-de-cambios); el reviewer las verifica | done (2026-08-17) |
| 3 | Diagnostico de la instalacion con remedio por linea | harness_doctor | <O1> | `harness_cli doctor [--json]` revisa binario, hooks, espejos, marker, hub, PATH y graphify, e imprime el comando exacto de remedio por cada falla; exit 0/2 sin solaparse con `harness_check.sh` | done (2026-08-17) |
| 4 | Rutas protegidas: el PRD y la constitution dejan de depender de la buena fe | rutas_protegidas_deny | <O1> | Lista de rutas protegidas (default `docs/prd/**`, `docs/constitution.md`, `.env`) con tres capas: prevenir donde el backend lo soporte, detectar al instante con el comando de reversion, y `harness_check.sh` como red de seguridad que bloquea | done (2026-08-18) |
| 5 | El catalogo de lecciones se lee bien con nombres largos | leccion_list_alineacion_dinamica | <O1> | `leccion list` calcula el ancho de la columna en vez de usar el 28 fijo; solo formato de salida, sin tocar orden, campos, `--json` ni exit codes | done (2026-08-18) |
| 6 | El PRD, el SDD y architecture.md dejan de poder quedar mintiendo | prd_y_sdd_siempre_al_dia | <O1> | Al cerrar, el arnes calcula el alcance (PRD de origen + padres + SDD + architecture.md), siembra una pregunta por documento en `docs/prd-diff-<id>.md`, y solo con el SI del usuario `prd apply --yes` lo escribe; `require_docs_al_dia` lo exige al cerrar | done (2026-08-18) |
| 7 | Un AC que no ejecuto ningun caso deja de contar como verificado | verify_detecta_filtro_vacio | <O1> | `verify` mira la SALIDA ademas del exit code: si reconoce el formato de libtest y la suma de `passed` es cero, el AC queda en `vacio`, se cuenta aparte en el resumen y bloquea el cierre igual que un rojo; sobre salidas que no son de tests el estado no cambia | done (2026-08-19) |
| 8 | Features en paralelo sin pisarse | features_en_paralelo_con_worktrees | <O1> | `start` deja de rechazar la segunda feature activa y le da a cada una su rama GitFlow (`feature/<id>-<slug>`, `bugfix/` si es `kind: bug`) y su worktree hermano; el estado del arnes sigue siendo unico (repo principal) y el vivo se parte en `current-<id>.md` con `current.md` como indice; dentro del worktree los comandos infieren la feature; `close --status done` exige `--to <rama>`, mergea, borra el worktree y conserva la rama (desde la #72 la publicacion es explicita con --publicar), y un conflicto aborta sin dejar nada a medias | done (2026-08-22) |
| 9 | Revisar en serio sin que cueste una fortuna | revision_adversarial_y_modelos_por_rol | <O1> | Un modelo por rol de Claude (implementer `claude-opus-5`, lider y reviewer `claude-fable-5`, los tres `xhigh`) definido en la tabla de roles de los dos instaladores y tuneable por variable; el reviewer intenta REFUTAR cada AC y verifica por su cuenta lo que la evidencia declara verde; y `revision --feature <id>` arma el paquete minimo (AC + estado de verify + evidencia + archivos + diff + rutas protegidas) acotado por presupuesto, que declara lo que recorta y reporta su propio tamaño | done (2026-08-22) |
| 10 | El MCP de Atlassian ya conectado en cada backend | mcp_atlassian_en_los_cuatro_backends | <O1> | Instalar el arnes en un repo con binding de Atlassian deja tambien el MCP por PROYECTO en los backends que lo admiten (`.mcp.json` de Claude, `.kimi-code/mcp.json` de Kimi y `.grok/config.toml` de Grok via `mcp-remote`, porque su cliente HTTP no completa el OAuth), y para Codex —que no admite alcance de proyecto— imprime los dos comandos (servidor + plugin `atlassian-rovo`, imprescindible) en vez de tocar su configuracion global; respeta lo que ya haya, no escribe credenciales y dice por CLI como autorizar | done (2026-08-22) |
| 11 | Empezar con el material en la mano, no explorando | paquete_de_contexto_para_implementar | <O1> | `contexto --feature <id>` (o `--tema`) entrega el mapa —siguiendo el puntero si `architecture.md` apunta a otro archivo—, si ese mapa CUBRE el tema, el impacto del hub con limite, la edad del grafo (vencido a los 7 dias), la historia acotada, las lecciones que aplican y las features del mismo servicio; declara su tamaño y sus huecos, y el resumen sale solo en cada `start`. Disparador: un mapeo de 4 agentes y 693.6k tokens sobre un tema que el mapa no mencionaba | done (2026-08-22) |
| 12 | El arnes no se bloquea a si mismo | el_guard_no_bloquea_por_lo_que_escribe_el_arnes | <O1> | El commit guard deja de contar como sucios los documentos que escribio el propio arnes (specs, planes, impl, review, verify, estados, prd-diff, `docs/prd/**`, `docs/lecciones/**`, architecture y perfil), exigiendo nombre Y ubicacion bajo `docs/`; sigue bloqueando por codigo y por cualquier documento ajeno, y dice en una linea `[i]` cada vez que se saltea un repo. Disparador: en un proyecto donde `docs/` es su propio repo, cada start/advance/prd apply terminaba el turno pidiendo un commit por microservicio de archivos que el `close` iba a commitear | done (2026-08-22) |
| 13 | Verificar lo que de verdad prueba, aunque hable mucho | verify_no_se_cuelga_con_salida_grande | <O1> | `verify` lee los pipes con un hilo por descriptor MIENTRAS el comando corre, en vez de leerlos despues de esperarlo: un comando que imprime mas que el buffer del pipe (~64 KB) ya no cuelga el gate. Retiene la cola con tope de 4 MB declarando el recorte, sigue midiendo el estado sobre la salida completa (leccion #44), sigue cortando por timeout y no se deja pisar por un nieto que hereda el pipe. Disparador: el smoke del instalador dejo a verify once minutos colgado y quedo sin poder declararse como AC | done (2026-08-22) |
| 14 | El paralelo aisla los cambios y acota los workflows | el_paralelo_aisla_los_cambios | <O1> | `start` resuelve el aislamiento ANTES de marcar `in_progress`: un fallo de git, o dos features que escribirian en el checkout compartido, RECHAZAN el arranque y dejan el backlog intacto; una feature con su worktree arranca siempre (corregido en la #76: la regla original vetaba a todas), y sin repo git corre una feature a la vez; un `docs/` que es otro repo recibia su propio worktree (revertido en la #77: ahora sus documentos van directo a `docs/`); el cierre muestra origen, destino y TODO el rango de commits, se niega si arrastra trabajo de otra feature, serializa por destino y ya no publica sin `--publicar`; el Stop revisa el worktree de la sesion en vez de reclamar los repos compartidos; y una tarea delegada fallida se registra y bloquea `approved` hasta cubrirse. Disparador: tres features activas sin rama ni worktree escribiendo en el mismo checkout, y un commit que se habia acordado dejar local publicado por ser el padre de otro | done (2026-09-05) |
| 15 | El sello de cierre deja de perderse | el_close_no_pierde_el_sello_de_cierre | <O1> | El `close` escribe `docs/estado-feature-<id>-<slug>.md` —que lleva adentro el cuerpo de `progress/current-<id>.md` y es su UNICA copia, porque `progress/` esta gitignorado— en el `docs/` del repo PRINCIPAL y despues de integrar, no en el `docs/` de la feature y antes. Escribirlo en la rama significaba escribirlo en un worktree que el propio cierre borra: con un `docs/` que es repo aparte eso lo perdia. El mensaje nombra la ruta real y avisa que queda sin commitear; el caso especial que elegia entre dos rutas (#63) se elimina porque ya no hay dos. Disparador: el cierre de la #124 de realestate, cuyo sello hubo que reconstruir a mano y cuyo cuerpo literal es irrecuperable | done (2026-09-05) |
| 16 | Un AC verifica con TODOS los comandos que declara | verify_corre_todos_los_comandos_del_ac | <O1> | `verify` ejecuta cada linea `Comando:` que el AC declara, en orden, y deja UNA FILA POR COMANDO en `docs/verify-<id>.md` con su estado, su exit y su duracion; el AC queda rojo si cualquiera falla y el gate lo nombra una sola vez. Antes el modelo era `comando: Option<String>` y el segundo `Comando:` se descartaba sin marca. Disparador: el AC-8 de la #72 declaro cuatro verificaciones, se corrio una y el reporte dijo "1 verde, 0 en rojo" — con DOS tests en verde al lado, uno que afirmaba el descarte como intencion y otro cuyo oraculo contaba "solo el primero, como en `parsear`" | done (2026-09-05) |
| 17 | El backlog sabe que feature espera a cual | dependencias_y_circuit_breaker | <O1> | Una feature declara `depends_on` (al crearla o despues, con `harness depende`); `next` no la ofrece hasta que esas cierren y DICE quien espera a que; `start` avisa sin bloquear; los ids inexistentes, la auto-referencia y los ciclos se rechazan sin escribir nada, nombrando el camino del ciclo. `superseded` y `resuelto-aguas-arriba` satisfacen una dependencia; `blocked` y `pending` no. Ademas, a partir del N-esimo cierre `blocked` (`rules.bloqueos_antes_de_decidir`, default 2) el cierre exige decir si la causa es la misma. Medido antes de implementar: 84 cierres reales, CERO features bloqueadas dos veces — el circuit breaker se implemento por decision del usuario con esa medicion a la vista, y sus dos tests son toda su evidencia | done (2026-09-05) |
| 18 | Una feature sin worktree ya no veta a las que traen el suyo | el_checkout_compartido_tiene_capacidad_uno | <O1> | Regresion de la #72, mismo dia: una feature abierta sin worktree rechazaba a TODAS las demas, y el usuario termino esperando a que "la #99 libere" para arrancar una feature con su propio arbol. El modelo pasa a ser "el checkout compartido tiene capacidad UNO": se rechaza solo que dos features escriban ahi; una con worktree arranca siempre y se le informa con quien convive. Sin git sigue siendo una a la vez. Disparador: el reporte del usuario con captura; la regla original generalizaba de mas sobre el incidente real (cuatro sin worktree en el MISMO arbol) | done (2026-09-06) |
| 19 | Con docs/ como repo aparte, los documentos van directo a docs/ | docs_aparte_escribe_en_docs | <O1> | Se revierte la OBS-5 de la #72: el repo `docs/` aparte ya no recibe un worktree por feature. Spec, plan, impl y review van a `<raiz>/docs/`, junto al PRD y el SDD, desde que se escriben; no existe `docs-wt/`, no hay ramas del repo docs y `docs_worktree` se ignora si quedo en el backlog. Con docs dentro del repo principal nada cambia. Disparador: tres `docs-wt/` en realestate con sus specs sin commitear en ramas que nadie mergeo, y la pregunta del usuario de por que sus documentos no estaban en docs/ con el PRD | done (2026-09-06) |
| 20 | El instalador respalda el backlog y `--reset` deja de borrarlo | el_instalador_respalda_el_backlog | <O1> | `feature_list.json` y `progress/` son los unicos datos del proyecto que el instalador no puede regenerar, y eran los unicos que no respaldaba: `bkp/` guardaba trece scripts y ningun backlog. Ahora salen de los targets del `--reset` (sh y ps1), se respaldan en TODA corrida en un paso propio (`backup_datos` / `Backup-HarnessData`) que `--force` no saltea —`--force` significa "no respaldes lo que vas a regenerar", no "no respaldes los datos"— y si faltan, la siembra de la plantilla vacia lo dice en `[WARN]`, nombra los respaldos que hay (`bkp/` y `docs/bkp-backlog/`, con cuantas features tiene cada uno) y el comando para volver. El check `tests/backlog_backup_check.sh` (dentro del smoke) planta un backlog y afirma el efecto en los cuatro modos; contra el instalador anterior caen los cuatro. La politica de espejo del backlog gitignorado de ESTE repo queda como decision del usuario, fuera de la feature. Disparador: una corrida del instalador sobre realestate el 2026-09-06 dejo el backlog en la plantilla, sin aviso y sin copia | done (2026-09-06) |
| 21 | Lo aprendido tiene ciclo de vida: tope, racha y dos avisos | el_autoaprendizaje_tiene_ciclo_de_vida | <O1> | Medido el 2026-09-06 sobre este repo: 10 de los ultimos 15 cierres declararon la misma leccion (442 lineas, nueve secciones "(feature #N)"), el paso 3 de la guia (`<clase>/referencias/`) con cero usos, 340 decisiones sin incorporar al perfil y la consolidacion sin correr en 19 dias y sin registro; `require_leccion` media que se declare una leccion, no que se aprenda. Cuatro umbrales en `rules` (`0` apaga): `leccion_max_lineas` (250) —sobre el tope, `close --leccion` y `leccion usar` se niegan con el contrato de particion a `referencias/`, sin escape por flag—; `leccion_repeticiones` (3) —la misma clase K cierres seguidos exige `--leccion-motivo`, `ninguna` no cuenta—; `perfil_pendientes_max` (25) y `consolidar_cada_dias` (30) —el cierre avisa por stderr, y `consolidar`/`curar` registran su corrida en `history.md`—. `lecciones status` y `harness_check.sh` lo muestran. La biblioteca de este repo se puso dentro del tope (442 -> 186, 316 -> 167, doce referencias), sin fusionar las tres lecciones que el consolidador proponia, y el perfil recibio cuatro entradas aprobadas una por una. Disparador: la pregunta del usuario "¿quedo bien implementado el autoaprendizaje de Hermes?" y la medicion que la respondio | done (2026-09-06) |
| 22 | El marcador stale de graphify deja de bloquear el Stop | graphify_stale_no_bloquea | <O1> | `graphify-out/.graphify_stale` lo deja el propio arnes (el hook post-commit tras cada commit que toca un `.md`, o `autocheck` cuando `graphify update` falla) y solo lo limpia el propio arnes cuando logra el rebuild semantico, que esta debounced a 30 minutos y salta sin backend LLM. `harness_check.sh` lo contaba como fallo y el Stop hook bloqueaba al agente hasta un `/graphify --update` a mano. Ahora se avisa con `[i]` —quien lo limpia solo, como forzarlo, y que no bloquea— y el check no falla por el; `templates/harness_check.sh` identico y un test en fixture instalado enganchado al smoke. Tercera vez que un enriquecimiento best-effort se cuela como gate (#18 nudge, #80 avisos): lo que no impide trabajar se avisa. Disparador: captura del usuario del Stop hook de realestate, 2026-09-06 22:40 | done (2026-09-07) |
| 23 | `add` no carga dos veces la misma feature | add_no_duplica | <O1> | Reabierta por decision del usuario tras cerrarse `blocked` (medido: 83 features, cero nombres repetidos y cero parecidos; la defensa es para el escenario que no deja huella: dos sesiones con el mismo hallazgo, un `add` re-corrido, un script que se relanza). `add` compara el nombre normalizado (minusculas, sin acentos ni puntuacion, sin palabras vacias) con el backlog ANTES de escribir: igual a una feature abierta (`pending`, `in_progress`, `blocked`) se niega con exit 2 nombrandola, sin flag de escape; igual a una cerrada avisa `[i]` y la crea (una regresion es legitima). `--clave <k>` es el idempotency-key de Hermes: con la misma clave devuelve la feature existente sin escribir nada. Modulo puro `duplicados.rs`, campo opcional `clave`, nada existente se toca; sin aviso por parecidos (medido cero pares). Disparador: idea 7 del catalogo de Hermes, y el usuario pidiendo implementarla con los skills de Rust cargados | done (2026-09-07) |
| 24 | `close` refresca el espejo del backlog y de la bitacora | close_refresca_el_espejo | <O1> | `feature_list.json` y `progress/history.md` estan gitignorados y son lo unico que el instalador no regenera (#78). El 2026-09-06 el checkout se borro por error: el backlog volvio del espejo `docs/bkp-backlog/` refrescado a mano en el ultimo cierre; la bitacora no tenia espejo y se perdio. Ahora cada `close` (cualquier `--status`), despues de guardar el estado y de la linea de bitacora de ese cierre, deja `docs/bkp-backlog/feature_list.json` y `docs/bkp-backlog/history.md` byte-identicos en la raiz del repo principal (copia atomica) y lo dice; quedan sin commitear, como el sello. Politica en `rules.espejo_backlog`: ausente = solo si el directorio existe (el directorio es el opt-in), `true` = crea, `false` = no toca. Best-effort no mudo: si la copia falla, el cierre sigue y avisa `[!]`. Modulo puro `espejo.rs` (politica como enum, decision pura); sin refresco en `add`/`start`. Disparador: la perdida de la bitacora del 2026-09-06 y la decision del usuario de espejarla tambien | done (2026-09-07) |
| 25 | El aviso de perfil mide crecimiento, no acumulado | aviso_de_perfil_mide_crecimiento | <O1> | El aviso de la #80 (`rules.perfil_pendientes_max`) contaba TODAS las decisiones sin incorporar (268 en este repo, casi todas de agosto y ya destiladas sin cita) y salia en cada cierre para siempre; el remedio habia sido subir el umbral a 300. Ahora cada registro lleva su momento (la bitacora, su timestamp; un plan o spec, el `started_at` de su feature) y el cierre compara con el umbral solo las posteriores a la ultima entrada del perfil: la ultima linea `perfil add|replace` de `history.md` o el `started_at` de la feature mas alta que cita el perfil, la mas reciente; `perfil remove` no cuenta. Sin corte se cuenta todo, como antes. `lecciones status` (texto y `--json`: `perfil_pendientes` sigue siendo lo comparado, mas `perfil_pendientes_total`, `perfil_corte` y `perfil_corte_origen`) y `perfil sugerir` muestran las dos cuentas y el corte. Medido al cerrar sobre este repo: corte desde el backlog (inicio de la #80; la bitacora se perdio el 2026-09-06), 26 nuevas de 270; el umbral vuelve a 25. Disparador: 289 decisiones historicas hacian saltar el aviso en cada cierre (#80) | done (2026-09-07) |

> El programa de **aprendizaje del arnes** (lecciones, nudge, perfil, buscar,
> curador y mapa) no esta aca: tiene su propio PRD anidado en
> `docs/prd/aprendizaje/PRD-aprendizaje.md`, porque no entraba en una historia
> sola.

## 11. Riesgos

| Riesgo | Impacto | Mitigacion |
| --- | --- | --- |
| <riesgo> | <alto/medio/bajo> | <que se hace al respecto> |

## 12. Decisiones abiertas

<Mismo protocolo que los planes: una decision sin resolver se pregunta al
USUARIO antes de implementar lo que dependa de ella. Registra aqui la respuesta
con su fecha.>

- <pregunta> — DECIDIDO (<usuario>, <fecha>): <respuesta>
- <pregunta> — ABIERTA

## PRDs anidados

<Las partes en las que se divide este producto. Cada fila la agrega
 `sh harness_cli prd add --name <parte>`, que crea el PRD hijo con las mismas 12
 secciones y lo deja colgado aca. Cada hijo cuenta su propia historia; este
 documento no carga con todo el peso. Para ver el arbol entero con sus hitos:
 `sh harness_cli prd tree`.>

| PRD | Archivo | Que cuenta |
| --- | --- | --- |
| aprendizaje | aprendizaje/PRD-aprendizaje.md | El arnes que aprende: lecciones, nudge, perfil de usuario, buscar, curador y mapa |

## Bitacora

<Lo que el arnes cerro contra este PRD. Si lo implementado difiere de lo que
 promete este documento, actualiza el documento: esa parte es tuya.>

-
- #14 hub_batch_upserts_atomic_install -> done 2026-08-14 · spec: docs/spec-feature-14-hub-batch-upserts-atomic-install.md · impl: docs/impl-14.md
- #15 atlassian_binding_and_outbox -> done 2026-08-16 · spec: docs/spec-feature-15-atlassian-binding-and-outbox.md · impl: docs/impl-15.md
- #16 atlassian_auto_push -> done 2026-08-16 · spec: docs/spec-feature-16-atlassian-auto-push.md · impl: docs/impl-16.md
- #23 ac_ejecutables_verify -> done 2026-08-17 · spec: docs/spec-feature-23-ac-ejecutables-verify.md · impl: docs/impl-23.md
- #24 conventions_escalera_y_tests -> done 2026-08-17 · spec: docs/spec-feature-24-conventions-escalera-y-tests.md · impl: docs/impl-24.md
- #25 harness_doctor -> done 2026-08-17 · spec: docs/spec-feature-25-harness-doctor.md · impl: docs/impl-25.md
- #26 rutas_protegidas_deny -> done 2026-08-18 · spec: docs/spec-feature-26-rutas-protegidas-deny.md · impl: docs/impl-26.md
- #30 paridad_ps1_verificable -> done 2026-08-18 · spec: docs/spec-feature-30-paridad-ps1-verificable.md · impl: docs/impl-30.md
- #36 deudas_anotadas_del_arnes -> done 2026-08-18 · spec: docs/spec-feature-36-deudas-anotadas-del-arnes.md · impl: docs/impl-36.md
- #29 prd_y_sdd_siempre_al_dia -> done 2026-08-18 · spec: docs/spec-feature-29-prd-y-sdd-siempre-al-dia.md · impl: docs/impl-29.md
- #37 estado_superseded -> done 2026-08-18 · spec: docs/spec-feature-37-estado-superseded.md · impl: docs/impl-37.md
- #44 verify_detecta_filtro_vacio -> done 2026-08-19 · spec: docs/spec-feature-44-verify-detecta-filtro-vacio.md · impl: docs/impl-44.md
- #47 features_en_paralelo_con_worktrees -> done 2026-08-22 · spec: docs/spec-feature-47-features-en-paralelo-con-worktrees.md · impl: docs/impl-47.md
- #49 architecture_en_el_worktree_de_la_feature -> done 2026-08-22 · spec: docs/spec-feature-49-architecture-en-el-worktree-de-la-feature.md · impl: docs/impl-49.md
- #50 mensaje_de_cierre_dice_la_verdad -> done 2026-08-22 · spec: docs/spec-feature-50-mensaje-de-cierre-dice-la-verdad.md · impl: docs/impl-50.md
- #51 revision_adversarial_y_modelos_por_rol -> done 2026-08-22 · spec: docs/spec-feature-51-revision-adversarial-y-modelos-por-rol.md · impl: docs/impl-51.md
- #52 mcp_atlassian_en_los_cuatro_backends -> done 2026-08-22 · spec: docs/spec-feature-52-mcp-atlassian-en-los-cuatro-backends.md · impl: docs/impl-52.md
- #56 paquete_de_contexto_para_implementar -> done 2026-08-22 · spec: docs/spec-feature-56-paquete-de-contexto-para-implementar.md · impl: docs/impl-56.md
- #58 el_guard_no_bloquea_por_lo_que_escribe_el_arnes -> done 2026-08-22 · spec: docs/spec-feature-58-el-guard-no-bloquea-por-lo-que-escribe-el-arnes.md · impl: docs/impl-58.md
- #46 verify_no_se_cuelga_con_salida_grande -> done 2026-08-22 · spec: docs/spec-feature-46-verify-no-se-cuelga-con-salida-grande.md · impl: docs/impl-46.md
- #57 verify_corre_en_el_worktree_de_la_feature -> done 2026-08-26 · spec: docs/spec-feature-57-verify-corre-en-el-worktree-de-la-feature.md · impl: docs/impl-57.md
- #38 prd_propose_texto_candidato -> done 2026-08-26 · spec: docs/spec-feature-38-prd-propose-texto-candidato.md · impl: docs/impl-38.md
- #39 prd_senales_mas_alla_del_nombre -> done 2026-08-26 · spec: docs/spec-feature-39-prd-senales-mas-alla-del-nombre.md · impl: docs/impl-39.md
- #40 prd_sello_se_invalida_al_editar -> done 2026-08-26 · spec: docs/spec-feature-40-prd-sello-se-invalida-al-editar.md · impl: docs/impl-40.md
- #41 consolidar_usa_relacionadas -> done 2026-08-26 · spec: docs/spec-feature-41-consolidar-usa-relacionadas.md · impl: docs/impl-41.md
- #42 consolidar_esqueleto_del_paraguas -> done 2026-08-26 · spec: docs/spec-feature-42-consolidar-esqueleto-del-paraguas.md · impl: docs/impl-42.md
- #43 consolidar_check_sin_cuota -> done 2026-08-26 · spec: docs/spec-feature-43-consolidar-check-sin-cuota.md · impl: docs/impl-43.md
- #53 check_no_se_cuelga_por_stdin -> done 2026-08-26 · spec: docs/spec-feature-53-check-no-se-cuelga-por-stdin.md · impl: docs/impl-53.md
- #54 prd_apply_escribe_en_el_docs_de_la_feature -> done 2026-08-26 · spec: docs/spec-feature-54-prd-apply-escribe-en-el-docs-de-la-feature.md · impl: docs/impl-54.md
- #55 check_resuelve_el_spec_de_la_feature -> done 2026-08-26 · spec: docs/spec-feature-55-check-resuelve-el-spec-de-la-feature.md · impl: docs/impl-55.md
- #1 powershell_windows_installer -> done 2026-06-11 · impl: docs/impl-1.md
- #2 remove_python_full_rust_migration -> done 2026-06-11 · impl: docs/impl-2.md
- #3 spec_driven_development -> done 2026-07-24 · spec: docs/spec-feature-3-spec-driven-development.md · impl: docs/impl-3.md
- #4 harness_docs_to_root_docs -> done 2026-07-24 · spec: docs/spec-feature-4-harness-docs-to-root-docs.md · impl: docs/impl-4.md
- #5 prd_master_templates -> done 2026-07-24 · spec: docs/spec-feature-5-prd-master-templates.md · impl: docs/impl-5.md
- #6 interactive_spec_approval -> done 2026-07-24 · spec: docs/spec-feature-6-interactive-spec-approval.md · impl: docs/impl-6.md
- #7 harness_check_robustness -> done 2026-07-28 · spec: docs/spec-feature-7-harness-check-robustness.md · impl: docs/impl-7.md
- #8 kimi_cli_backend -> done 2026-07-29 · spec: docs/spec-feature-8-kimi-cli-backend.md · impl: docs/impl-8.md
- #9 codex_roles_can_write_artifacts -> done 2026-07-29 · spec: docs/spec-feature-9-codex-roles-can-write-artifacts.md · impl: docs/impl-9.md
- #10 layout_inferred_from_footprint -> done 2026-07-30 · spec: docs/spec-feature-10-layout-inferred-from-footprint.md · impl: docs/impl-10.md
- #11 link_kimi_guide_in_surfaces -> done 2026-08-07 · spec: docs/spec-feature-11-link-kimi-guide-in-surfaces.md · impl: docs/impl-11.md
- #12 prd_story_method -> done 2026-08-12 · spec: docs/spec-feature-12-prd-story-method.md · impl: docs/impl-12.md
- #13 nested_prds -> done 2026-08-12 · spec: docs/spec-feature-13-nested-prds.md · impl: docs/impl-13.md
- #60 la_vuelta_al_prd_no_se_pierde_ni_miente -> done 2026-08-27 · spec: docs/spec-feature-60-la-vuelta-al-prd-no-se-pierde-ni-miente.md · impl: docs/impl-60.md
- #61 el_merge_del_cierre_no_toca_tu_checkout -> done 2026-08-27 · spec: docs/spec-feature-61-el-merge-del-cierre-no-toca-tu-checkout.md · impl: docs/impl-61.md
- #62 el_cierre_no_declara_hecho_lo_que_no_hizo -> done 2026-08-27 · spec: docs/spec-feature-62-el-cierre-no-declara-hecho-lo-que-no-hizo.md · impl: docs/impl-62.md
- #59 cmd_smoke_real_en_windows -> done 2026-08-27 · spec: docs/spec-feature-59-cmd-smoke-real-en-windows.md · impl: docs/impl-59.md
- #63 el_arnes_no_afirma_lo_que_no_puede_comprobar -> done 2026-08-27 · spec: docs/spec-feature-63-el-arnes-no-afirma-lo-que-no-puede-comprobar.md · impl: docs/impl-63.md
- #64 el_arnes_no_promete_enforcement_que_no_hace -> done 2026-08-30 · spec: docs/spec-feature-64-el-arnes-no-promete-enforcement-que-no-hace.md · impl: docs/impl-64.md
- #66 el_stop_hook_no_entra_en_bucle -> done 2026-08-30 · spec: docs/spec-feature-66-el-stop-hook-no-entra-en-bucle.md · impl: docs/impl-66.md
- #67 los_dos_parsers_del_review_no_se_contradicen -> done 2026-09-01 · spec: docs/spec-feature-67-los-dos-parsers-del-review-no-se-contradicen.md · impl: docs/impl-67.md
- #65 el_arnes_cierra_lo_resuelto_aguas_arriba -> done 2026-09-01 · spec: docs/spec-feature-65-el-arnes-cierra-lo-resuelto-aguas-arriba.md · impl: docs/impl-65.md
- #68 el arnes no pierde los AC que pide revisar a mano -> done 2026-09-01 · spec: docs/spec-feature-68-el-arnes-no-pierde-los-ac-que-pide-revisar-a-man.md · impl: docs/impl-68.md
- #69 una linea AC ilegible no desaparece en silencio -> done 2026-09-01 · spec: docs/spec-feature-69-una-linea-ac-ilegible-no-desaparece-en-silencio.md · impl: docs/impl-69.md
- #70 El gate de citas del review no puede ver un repo hermano: una feature de backend no puede citar su codigo -> done 2026-09-04 · spec: docs/spec-feature-70-el-gate-de-citas-del-review-no-puede-ver-un-repo.md · impl: docs/impl-70.md
- #72 El paralelo aisla los cambios y acota los workflows -> done 2026-09-05 · spec: docs/spec-feature-72-el-paralelo-aisla-los-cambios-y-acota-los-workfl.md · impl: docs/impl-72.md
- #71 El close archiva el sello de cierre en el worktree que acaba de borrar, y lo pierde -> done 2026-09-05 · spec: docs/spec-feature-71-el-close-archiva-el-sello-de-cierre-en-el-worktr.md · impl: docs/impl-71.md
- #73 verify corre UN comando por AC y no lo dice: un AC con varias verificaciones se cree verde con una -> done 2026-09-05 · spec: docs/spec-feature-73-verify-corre-un-comando-por-ac-y-no-lo-dice-un-a.md · impl: docs/impl-73.md
- #75 el backlog no sabe de dependencias ni de features que se traban una y otra vez -> done 2026-09-05 · spec: docs/spec-feature-75-el-backlog-no-sabe-de-dependencias-ni-de-feature.md · impl: docs/impl-75.md
- #76 una feature sin worktree veta a todas las demas y mata el paralelismo -> done 2026-09-06 · spec: docs/spec-feature-76-una-feature-sin-worktree-veta-a-todas-las-demas-.md · impl: docs/impl-76.md
- #77 con docs/ como repo aparte, el arnes escribe directo en docs/ y no crea docs-wt -> done 2026-09-06 · spec: docs/spec-feature-77-con-docs-como-repo-aparte-el-arnes-escribe-direc.md · impl: docs/impl-77.md
- #78 el instalador respalda sus scripts pero no el backlog, que es lo unico irrecuperable -> done 2026-09-06 · spec: docs/spec-feature-78-el-instalador-respalda-sus-scripts-pero-no-el-ba.md · impl: docs/impl-78.md
- #80 el autoaprendizaje no tiene ciclo de vida: lecciones sin tope, la misma leccion en diez cierres, perfil sin alimentar y consolidacion sin correr -> done 2026-09-07 · spec: docs/spec-feature-80-el-autoaprendizaje-no-tiene-ciclo-de-vida-leccio.md · impl: docs/impl-80.md
- #83 el Stop hook bloquea por graphify-out/.graphify_stale, un marcador de enriquecimiento best-effort que el hook post-commit deja y solo borra si logra el rebuild semantico -> done 2026-09-07 · spec: docs/spec-feature-83-el-stop-hook-bloquea-por-graphify-out-graphify-s.md · impl: docs/impl-83.md
- #74 add no protege contra la feature duplicada, y no esta medido que haga falta -> done 2026-09-07 · spec: docs/spec-feature-74-add-no-protege-contra-la-feature-duplicada-y-no-.md · impl: docs/impl-74.md
- #79 close refresca el espejo docs/bkp-backlog/feature_list.json al cerrar una feature -> done 2026-09-07 · spec: docs/spec-feature-79-close-refresca-el-espejo-docs-bkp-backlog-featur.md · impl: docs/impl-79.md
- #82 el aviso de perfil del cierre cuenta solo las decisiones posteriores a la ultima entrada del perfil -> done 2026-09-07 · spec: docs/spec-feature-82-el-aviso-de-perfil-del-cierre-cuenta-solo-las-de.md · impl: docs/impl-82.md
