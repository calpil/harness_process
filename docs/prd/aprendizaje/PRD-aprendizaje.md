# PRD - Aprendizaje del arnes

Estado: Borrador
Padre: master
Duenno: Alan
Ultima actualizacion: 2026-08-16
Alcance: que el arnes acumule y reutilice lo que aprende de cada feature cerrada. NO toca el ciclo de vida (start/advance/close), ni el Memory Hub, ni la integracion Atlassian.
Como se escribe: ../../prd/COMO-ESCRIBIR-UN-PRD.md
PRD padre: docs/prd/PRD-master.md
Diseno tecnico: ../../prd/SDD-master.md
Constitution: ../../constitution.md

> PRD anidado (`aprendizaje`): una parte del producto, con su propia historia. Es un
> documento del USUARIO: el arnes lo creo una vez y solo vuelve a tocarlo para
> marcar un hito cerrado y dejar bitacora. Todo lo demas lo escribis vos.
>
> Si esta parte sigue siendo demasiado grande para una sola historia, partila:
> `sh harness_cli prd add --name <parte> --parent aprendizaje`

---

**LA REGLA DURA: SIN CODIGO. SOLO PSEUDO-CODIGO.** Este documento fija la
**estructura** — la historia, que entidades se tocan y como cambian — en
pseudo-codigo y explicaciones. Nunca lleva codigo final, la implementacion
exacta, pantallas terminadas ni configuracion.

---

## 1. Resumen (hoy -> despues)

- **Hoy:** el arnes produce spec, plan, impl, review y bitacora en cada feature, y
  despues los abandona. El conocimiento queda archivado **por id de feature**, que
  es el orden en que nadie lo busca. Cada agente que arranca — Claude, Gemini,
  Codex, Kimi, Grok — empieza sin saber nada de las 16 features anteriores ni de
  las decisiones que el usuario ya tomo.
- **Despues:** el arnes acumula lo aprendido en **lecciones por clase de trabajo**,
  mantiene un **perfil del usuario** que viaja solo hasta las superficies de cada
  backend, se **auto-empuja** a persistir lo que aprendio, y puede **buscar en su
  propio pasado** en milisegundos.

## 2. La historia

**ANTES**

Alan arranca la feature #23 con Codex. Codex no sabe que en la #7 se decidio que
`roles/` es la fuente unica y que los espejos por backend se regeneran desde el
instalador; no sabe que en la #14, ante un fork de concurrencia, Alan eligio la
opcion segura aunque costara mas; no sabe que en la #15 y la #16 pidio sincronia
total con el sistema externo, incluido el backfill de lo ya cerrado.

Asi que Codex propone editar directamente un `.claude/agents/*.md`, o propone la
opcion rapida en vez de la segura, o propone reflejar "solo lo nuevo". Alan lo
corrige. La correccion queda escrita en `progress/history.md` — otra vez — y
muere ahi.

Lo que duele no es que el agente no sepa: es que **el arnes ya lo sabia**. Esta
escrito, fechado y firmado en el repo desde hace meses, y no llega.

**DESPUES**

Alan arranca la feature #23 con Codex. La superficie que Codex lee ya trae el
bloque de perfil: *ante un fork de concurrencia elige la opcion segura*, *prefiere
features amplias y completas antes que incrementales*, *exige sincronia total con
sistemas externos*. Cuando toca los espejos de roles, encuentra
`docs/lecciones/espejo-de-roles.md` porque su trigger matcheo, y ahi estan el
procedimiento y los pitfalls que costaron dos features aprender.

Al cerrar, el nudge le pide la leccion de esta feature: Codex patchea la que
estuvo en juego en vez de crear una nueva. Y cuando Alan pregunta "¿donde
decidimos usar ureq?", `sh harness_cli buscar ureq` le contesta con archivo,
linea y fecha, sin abrir un solo documento.

## 3. Objetivos / No-objetivos

| ID | Objetivo | Como se ve cumplido |
| --- | --- | --- |
| O1 | El conocimiento de una feature cerrada queda reutilizable **por clase de trabajo**, no por id | Cada cierre declara una leccion patcheada o creada, o `leccion: ninguna` con motivo |
| O2 | Las decisiones repetidas del usuario llegan **solas** al agente que arranca | El bloque de perfil aparece en las cinco superficies generadas, en todos los backends |
| O3 | Responder "¿donde decidimos X?" sin leer el repo entero | `buscar` devuelve archivo:linea, feature y fecha en milisegundos |
| O4 | Lo aprendido **no se pudre**: lo que no se usa envejece y lo que esta mal se puede podar | Ciclo de vida activa -> stale -> archivada, con `journey delete/edit` para corregir |

| ID | No-objetivo | Por que no |
| --- | --- | --- |
| NO1 | No se agrega un LLM adentro del binario | El arnes emite el **contrato** de la revision; el agente que este corriendo la ejecuta con sus tools. Asi sigue siendo backend-agnostico (Articulo 6) |
| NO2 | No reemplaza al Memory Hub ni a graphify | Son tres cosas distintas: el hub guarda **eventos**, las lecciones guardan **procedimiento**, el perfil guarda **preferencias** |
| NO3 | Nada entra al perfil sin el si explicito del usuario | Es el documento del usuario: mismo ritual que la aprobacion del spec (Articulo 2 y 5) |
| NO4 | Nada se borra automaticamente | El peor resultado posible de una pasada automatica es archivar, y el archivo es recuperable |

## 4. Usuarios y jobs-to-be-done

| Usuario | Que intenta lograr | Como lo resuelve hoy | Por que no alcanza |
| --- | --- | --- | --- |
| Alan (usuario del arnes) | Que el agente arranque sabiendo lo que ya se decidio | Se lo explica de nuevo en cada sesion, o corrige a mitad de camino | La correccion se pierde apenas termina la sesion; el costo se paga en cada feature |
| Agente lider (cualquier backend) | Entender el terreno antes de escribir el plan | Lee el plan anterior si adivina cual es | Los artefactos estan ordenados por id de feature, no por tema |
| Agente implementer | Repetir un procedimiento que ya salio bien | Reconstruye desde cero o repite un error ya cometido | Lo que funciono quedo enterrado en un `impl-<id>.md` |
| Agente reviewer | Verificar contra criterios estables | Relee la constitution y el spec | Las lecciones aprendidas (los pitfalls reales) no estan en ningun lado |

## 5. Metricas de exito

| Metrica | Hoy | Objetivo | Mide | Como se mide |
| --- | --- | --- | --- | --- |
| Features cerradas que dejan leccion declarada | 0 % | > 80 % | O1 | Campo de leccion en `feature_list.json` al cerrar |
| Lecciones a nivel de clase vs por feature | n/a | 0 lecciones con id de feature en el nombre | O1 | `leccion nueva` rechaza el nombre; el curador reporta |
| Entradas de perfil vigentes | 0 | 5-10, dentro del limite duro | O2 | `perfil` reporta uso en % |
| Tiempo de responder "¿donde decidimos X?" | minutos de grep manual | milisegundos | O3 | `buscar` |
| Lecciones rancias sin revisar | n/a | 0 lecciones archivadas por sorpresa | O4 | Reporte por corrida del curador |

## 6. Como funciona hoy -> como va a funcionar

```
HOY                                    DESPUES

close --status done                    close --status done
  |__ archiva estado                     |__ archiva estado
  |__ hub: registra evento               |__ hub: registra evento
  |__ refresca graphify                  |__ refresca graphify
  (fin: el conocimiento queda            |__ nudge: emite el CONTRATO de revision
   ordenado por id de feature)           |     |__ el agente patchea la leccion
                                         |        que estuvo en juego, o crea una
                                         |        a nivel de clase, o dice
                                         |        "ninguna" con motivo
                                         |__ gate: exige esa declaracion

(nada)                                 setup_harness.sh / .ps1
                                         |__ inyecta el bloque de perfil en
                                            CLAUDE.md / AGENTS.md / GEMINI.md /
                                            GROK.md / LLM.md (snapshot congelado)

grep manual por docs/ e history.md     harness_cli buscar "<consulta>"
                                         |__ specs, planes, impl, review,
                                            lecciones, history + hub
```

## 7. Los datos

| Que | Entidad / campo | Para que |
| --- | --- | --- |
| disparador (por trabajo) | conteo de escrituras del hook `PostToolUse` | pedir la revision cada N escrituras sin depender del backend |
| disparador (por cierre) | transicion `close --status done` | el momento de maxima senal: la feature termino y se sabe que funciono |
| leccion | `docs/lecciones/<clase>.md` con frontmatter (`nombre`, `descripcion`, `triggers`, `relacionadas`, `origen`, `usos`, `ultima_actualizacion`, `estado`) | memoria procedural buscable por trigger, no por id |
| apoyo de leccion | `docs/lecciones/<clase>/referencias/<tema>.md` | el detalle de una sesion sin inflar el cuerpo de la leccion |
| perfil | `docs/perfil-usuario.md`, entradas separadas, con limite duro | preferencias durables del usuario, inyectadas a las superficies |
| evidencia de una entrada de perfil | features y fechas que la respaldan | que `perfil sugerir` proponga con prueba, no con impresion |
| candado | `usos` + `ultima_vez` por leccion | evitar que se archive lo que si se usa, y detectar lo que no |
| interruptor | reglas `require_leccion` en `feature_list.json`, y `enabled` del curador | apagar el gate o la maquinaria entera sin desinstalar nada |

## 8. Pseudo-codigo (el acuerdo)

```
CUANDO el arnes acumulo N escrituras, O cuando se cierra una feature

  ¿esta activado el aprendizaje para este repo?  -> si no, no hacemos nada
  ¿ya emitimos el contrato hace poco (backoff)?  -> si si, no hacemos nada

  ENTONCES emitimos por stderr el CONTRATO de revision:
           "revisa lo que paso; si hay senal, patchea la leccion que estuvo
            en juego, o el paraguas, o agrega una referencia, o —recien
            entonces— crea una leccion a nivel de clase; y NO captures
            fallas de entorno, afirmaciones negativas sobre herramientas,
            errores transitorios ni fracasos no resueltos",
           con la restriccion de que el arnes NUNCA escribe la leccion:
           la escribe el agente, y el gate del cierre la verifica.


CUANDO el usuario aprueba una entrada de perfil

  ¿la entrada entra en el limite duro?  -> si no, FALLA con la lista de
                                            entradas actuales y pide consolidar
                                            en este mismo turno
  ¿ya existe identica?                  -> si si, no hacemos nada

  ENTONCES la escribimos en docs/perfil-usuario.md,
           con la restriccion de que solo ocurre tras el SI explicito del
           usuario, igual que approve-spec.
```

**Promesas:** una sola leccion por clase (no una por feature) · el arnes emite,
nunca escribe · nada entra al perfil sin tu si · nada se borra, solo se archiva ·
sin LLM en el binario y sin dependencias nuevas de runtime.

## 9. Restricciones y supuestos

- **Tecnicas:** todo en `rust/src/` con sus tests (Articulo 1). Sin dependencias
  nuevas de runtime sin ADR (Articulo 6). Todo lo que involucre un modelo es
  backend-agnostico: override explicito -> auto-deteccion por API key -> CLI del
  backend -> skip limpio.
- **De proceso:** `one_feature_at_a_time` sigue vigente; estos hitos se toman de a
  uno. `templates/` y la raiz se mantienen espejados.
- **Supuestos:** los artefactos del arnes son texto y son pocos, asi que un
  escaneo local alcanza para `buscar` sin indice propio; el hook `PostToolUse` ya
  existe en los backends que lo soportan y es suficiente para contar escrituras;
  el corpus de `progress/history.md` ya contiene las decisiones del usuario en
  forma explotable.

## 10. Hitos -> features

<Cada fila se carga al backlog con:
 sh harness_cli add --name <slug> --service <servicio> --acceptance "<criterio>" --prd aprendizaje
y al arrancarla (`start`) su spec nace citando este PRD. Al cerrarla
(`close --status done`) el arnes marca aca su Estado y deja bitacora.>

| # | Hito | Slug de feature | Objetivo que cumple | Criterio de aceptacion (resumen) | Estado |
| --- | --- | --- | --- | --- | --- |
| 1 | Lecciones: memoria procedural por clase | lecciones_memoria_procedural | O1 | Existe `docs/lecciones/<clase>.md` con frontmatter y ciclo `list/show/nueva/usar`; los nombres por feature se rechazan; las reglas de que NO capturar estan en la plantilla y en los tres roles | done (2026-08-16) |
| 2 | El arnes se auto-empuja | nudge_de_aprendizaje | O1 | El nudge emite el contrato cada N escrituras y en cada cierre, con backoff adaptativo, sin escribir nunca un artefacto y sin cambiar exit codes | done (2026-08-16) |
| 3 | Perfil del usuario | perfil_de_usuario | O2 | `docs/perfil-usuario.md` con limite duro que no auto-compacta, inyectado en las cinco superficies por ambos instaladores, alimentado por `perfil sugerir` y escrito solo con `--yes` | done (2026-08-17) |
| 4 | Buscar en el propio historial | buscar_en_el_historial | O3 | `buscar` responde con archivo:linea, feature y fecha sobre specs, planes, impl, review, lecciones, history y hub; sin LLM, sin dependencias nuevas, degradando limpio sin hub | done (2026-08-17) |
| 5 | Curador de lecciones | curador_de_lecciones | O4 | Transiciones deterministas 30d/90d que nunca borran, con pin, backup previo, rollback reversible y reporte por corrida; consolidacion con LLM opt-in y backend-agnostica | done (2026-08-17) |
| 6 | Mapa de aprendizaje | mapa_de_aprendizaje | O4 | `journey` dibuja la linea de tiempo sobre datos ya existentes y permite podar con `list/delete/edit` | done (2026-08-17) |
| 7 | Consolidacion de lecciones asistida por LLM | consolidacion_de_lecciones_con_llm | O4 | `lecciones consolidar` detecta solapamientos viendo solo nombre, descripcion y triggers (NUNCA el cuerpo) e informa; con `--aplicar` fusiona bajo un paraguas tomando la fusion de argv y archiva las miembros con backup y rollback. Apagada por default; cadena override -> CLI -> skip limpio | done (2026-08-18) |

## 11. Riesgos

| Riesgo | Impacto | Mitigacion |
| --- | --- | --- |
| El arnes aprende algo falso y lo repite durante meses | alto | La lista de que NO capturar portada literal a la plantilla y a los roles; el perfil solo se escribe con el si del usuario; el curador nunca borra y su rollback es reversible |
| Se confunde con el Memory Hub y graphify, y termina habiendo tres memorias que dicen cosas distintas | alto | Decidido el 2026-08-16 (seccion 12): tres almacenes con limite explicito — hub = eventos, lecciones = procedimiento, perfil = preferencias. Las lecciones y el perfil son archivos en `docs/` y **no** agregan nada al hub; el limite se escribe en `docs/architecture.md` como parte del hito 1 |
| El nudge se vuelve ruido y los agentes lo ignoran | medio | Backoff adaptativo, salida por stderr, best-effort con exit 0, y la posibilidad de apagarlo por regla |
| La biblioteca de lecciones se llena de casi-duplicados | medio | Nombres obligatoriamente a nivel de clase + orden de preferencia que favorece patchear + el curador (hito 5) |
| Se cuela un LLM en el camino critico del binario | medio | NO1: el arnes emite contratos y no llama a ningun modelo; lo unico que podria usarlo (consolidacion del curador) es opt-in y degrada a skip |

## 12. Decisiones abiertas

- ¿Que piezas del loop se adoptan? — DECIDIDO (Alan, 2026-08-16): las seis, como
  PRD anidado, con las features #17 a #22 cargadas al backlog.
- ¿Donde viven las lecciones y el perfil? — DECIDIDO (Alan, 2026-08-16): **archivos
  en `docs/`**, versionados como todo el resto del proceso. Quedan **tres
  almacenes con un limite explicito**: el hub PostgreSQL guarda **eventos**,
  `docs/lecciones/<clase>.md` guarda **procedimiento** y `docs/perfil-usuario.md`
  guarda **preferencias**. Consecuencias vinculantes para el hito 1: el
  aprendizaje **funciona sin hub arriba**, no se agregan tablas ni filas nuevas al
  hub, y el limite se escribe en `docs/architecture.md` como parte del hito 1.
- ¿El gate `require_leccion` arranca prendido o apagado? — DECIDIDO (Alan,
  2026-08-16): **apagado (opt-in)**, igual que arranco `require_spec_approved`. La
  regla se escribe en `rules` de `feature_list.json` en `false`; sin ella (o en
  `false`) el gate queda mudo y ninguna instalacion previa se rompe.
- ¿El perfil se versiona o queda local? — DECIDIDO (Alan, 2026-08-16):
  **versionado en `docs/perfil-usuario.md`**, un solo archivo compartido por el
  equipo y revisable en un PR. Implicancia a respetar en el hito 3: es un
  documento publico del repo, asi que el escaneo previo a escribir debe rechazar
  secretos y datos personales, y `--reset` no lo borra (es del usuario, como el
  PRD y la constitution).
- ¿Por cual hito se arranca? — DECIDIDO (Alan, 2026-08-16): por el **hito 1
  (`lecciones_memoria_procedural`, feature #17)**, el orden natural del PRD:
  sin lecciones el nudge no tiene que pedir y el curador no tiene que curar.
  Arranca al cerrar la feature #16 (`one_feature_at_a_time`).

## Bitacora

<Lo que el arnes cerro contra este PRD. Si lo implementado difiere de lo que
 promete este documento, actualiza el documento: esa parte es tuya.>

- #17 lecciones_memoria_procedural -> done 2026-08-16 · spec: docs/spec-feature-17-lecciones-memoria-procedural.md · impl: docs/impl-17.md
- #18 nudge_de_aprendizaje -> done 2026-08-16 · spec: docs/spec-feature-18-nudge-de-aprendizaje.md · impl: docs/impl-18.md
- #19 perfil_de_usuario -> done 2026-08-17 · spec: docs/spec-feature-19-perfil-de-usuario.md · impl: docs/impl-19.md
- #20 buscar_en_el_historial -> done 2026-08-17 · spec: docs/spec-feature-20-buscar-en-el-historial.md · impl: docs/impl-20.md
- #21 curador_de_lecciones -> done 2026-08-17 · spec: docs/spec-feature-21-curador-de-lecciones.md · impl: docs/impl-21.md
- #22 mapa_de_aprendizaje -> done 2026-08-17 · spec: docs/spec-feature-22-mapa-de-aprendizaje.md · impl: docs/impl-22.md
- #28 consolidacion_de_lecciones_con_llm -> done 2026-08-18 · spec: docs/spec-feature-28-consolidacion-de-lecciones-con-llm.md · impl: docs/impl-28.md
