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
| 8 | Features en paralelo sin pisarse | features_en_paralelo_con_worktrees | <O1> | `start` deja de rechazar la segunda feature activa y le da a cada una su rama GitFlow (`feature/<id>-<slug>`, `bugfix/` si es `kind: bug`) y su worktree hermano; el estado del arnes sigue siendo unico (repo principal) y el vivo se parte en `current-<id>.md` con `current.md` como indice; dentro del worktree los comandos infieren la feature; `close --status done` exige `--to <rama>`, mergea, publica, borra el worktree y conserva la rama, y un conflicto aborta sin dejar nada a medias | done (2026-08-22) |
| 9 | Revisar en serio sin que cueste una fortuna | revision_adversarial_y_modelos_por_rol | <O1> | Un modelo por rol de Claude (implementer `claude-opus-5`, lider y reviewer `claude-fable-5`, los tres `xhigh`) definido en la tabla de roles de los dos instaladores y tuneable por variable; el reviewer intenta REFUTAR cada AC y verifica por su cuenta lo que la evidencia declara verde; y `revision --feature <id>` arma el paquete minimo (AC + estado de verify + evidencia + archivos + diff + rutas protegidas) acotado por presupuesto, que declara lo que recorta y reporta su propio tamaño | done (2026-08-22) |
| 10 | El MCP de Atlassian ya conectado en cada backend | mcp_atlassian_en_los_cuatro_backends | <O1> | Instalar el arnes en un repo con binding de Atlassian deja tambien el MCP por PROYECTO en los backends que lo admiten (`.mcp.json` de Claude, `.kimi-code/mcp.json` de Kimi y `.grok/config.toml` de Grok via `mcp-remote`, porque su cliente HTTP no completa el OAuth), y para Codex —que no admite alcance de proyecto— imprime los dos comandos (servidor + plugin `atlassian-rovo`, imprescindible) en vez de tocar su configuracion global; respeta lo que ya haya, no escribe credenciales y dice por CLI como autorizar | done (2026-08-22) |
| 11 | Empezar con el material en la mano, no explorando | paquete_de_contexto_para_implementar | <O1> | `contexto --feature <id>` (o `--tema`) entrega el mapa —siguiendo el puntero si `architecture.md` apunta a otro archivo—, si ese mapa CUBRE el tema, el impacto del hub con limite, la edad del grafo (vencido a los 7 dias), la historia acotada, las lecciones que aplican y las features del mismo servicio; declara su tamaño y sus huecos, y el resumen sale solo en cada `start`. Disparador: un mapeo de 4 agentes y 693.6k tokens sobre un tema que el mapa no mencionaba | done (2026-08-22) |
| 12 | El arnes no se bloquea a si mismo | el_guard_no_bloquea_por_lo_que_escribe_el_arnes | <O1> | El commit guard deja de contar como sucios los documentos que escribio el propio arnes (specs, planes, impl, review, verify, estados, prd-diff, `docs/prd/**`, `docs/lecciones/**`, architecture y perfil), exigiendo nombre Y ubicacion bajo `docs/`; sigue bloqueando por codigo y por cualquier documento ajeno, y dice en una linea `[i]` cada vez que se saltea un repo. Disparador: en un proyecto donde `docs/` es su propio repo, cada start/advance/prd apply terminaba el turno pidiendo un commit por microservicio de archivos que el `close` iba a commitear | done (2026-08-22) |
| 13 | Verificar lo que de verdad prueba, aunque hable mucho | verify_no_se_cuelga_con_salida_grande | <O1> | `verify` lee los pipes con un hilo por descriptor MIENTRAS el comando corre, en vez de leerlos despues de esperarlo: un comando que imprime mas que el buffer del pipe (~64 KB) ya no cuelga el gate. Retiene la cola con tope de 4 MB declarando el recorte, sigue midiendo el estado sobre la salida completa (leccion #44), sigue cortando por timeout y no se deja pisar por un nieto que hereda el pipe. Disparador: el smoke del instalador dejo a verify once minutos colgado y quedo sin poder declararse como AC | done (2026-08-22) |

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
