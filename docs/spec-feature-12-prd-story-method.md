# Spec - Feature #12: prd_story_method

Estado: approved
Aprobado: 2026-08-12T02:19:54Z por USUARIO (confirmacion explicita) - Alan aprueba con ajuste: PRD-master tambien lleva Los datos y Pseudo-codigo a nivel producto
Plan: docs/plan-feature-12-prd-story-method.md
Constitution: docs/constitution.md

## Problema

El arnes ya encadena PRD -> backlog -> spec -> implementacion, pero el documento
donde nace todo (`docs/prd/PRD-master.md`, feature #5) esta escrito en el molde
clasico de PRD corporativo: "Problema / Usuarios / Metricas / Alcance". Ese
molde produce documentos abstractos ("escuchar el cambio de estado", "agendar
una tarea") que una IA implementa mal porque nunca supo que experiencia tenia
que existir al final.

El metodo de `how-i-spec.pdf` ("Escribe tu maldito PRD") ataca justo eso y hoy
NO esta en ninguna superficie del arnes:

1. **La historia primero** (ANTES/DESPUES, con nombre y momento). Es el corazon
   del documento: si la historia no convence, el resto no importa. El PRD actual
   arranca en "Problema" abstracto y nunca pide narrar el despues.
2. **Resumen Hoy -> Despues en dos lineas** (el dibujo mas barato que existe).
3. **Objetivos y no-objetivos numerados** (O1, O2 / NO1) para que las secciones
   siguientes los citen. Hoy el alcance es una lista de bullets sin ID citable,
   a diferencia de los AC-n del spec, que si tienen trazabilidad (Articulo 3).
4. **El flujo dibujado dos veces** (HOY vs DESPUES), que fuerza a reusar lo que
   ya existe en vez de inventar arquitectura.
5. **Los datos que se tocan**: disparador, interruptor por cliente, candado.
6. **Pseudo-codigo como acuerdo** (CUANDO / guards / ENTONCES / Promesas) y la
   regla dura: el PRD fija la estructura en pseudo-codigo y explicaciones,
   **nunca en codigo final**.
7. **El tamano lo decide el cambio** (1 pagina un ajuste, 3-8 una funcionalidad,
   10+ una grande, PRDs anidados para un producto nuevo). El arnes no dice en
   ningun lado cuanto PRD escribir, y esa es la duda practica que frena a quien
   arranca.

El mismo hueco existe un nivel mas abajo: `spec_template()`
(`rust/src/spec.rs:29-69`) genera cada `docs/spec-feature-<id>-<slug>.md` con
Recorridos + AC-n + No funcionales + Fuera de alcance + Observaciones. En el
vocabulario del PDF, ese spec ES el PRD del cambio (el PRD anidado), y le
faltan exactamente las cuatro secciones que hacen implementable un PRD: la
historia, el hoy->despues, los datos y el pseudo-codigo del acuerdo.

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como duenno de un proyecto que arranca de cero, quiero que
  `docs/prd/PRD-master.md` me pida la historia (antes/despues) y el hoy->despues
  antes que cualquier tabla, para escribir un PRD que una IA pueda implementar
  en vez de un documento que suene bien y no diga nada.
- P1: Como quien escribe el PRD, quiero una guia del metodo
  (`docs/prd/COMO-ESCRIBIR-UN-PRD.md`) que responda "que va y que nunca va
  adentro", "cuanto escribo" y "como se anidan", para no tener que adivinar el
  tamano ni copiar codigo final dentro del documento.
- P1: Como lider que abre una feature, quiero que el spec generado ya traiga
  Historia, Hoy -> Despues, Los datos y Pseudo-codigo, para que el implementer
  reciba el acuerdo completo y no solo criterios sueltos.
- P2: Como agente de cualquier backend (Claude, Codex, Gemini, Kimi), quiero que
  la superficie raiz enlace la guia, para descubrir el metodo sin que nadie me
  diga que existe.
- P2: Como usuario de Windows, quiero paridad en `setup_harness.ps1`: la guia se
  siembra y se verifica igual que en la version sh.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->
- AC-1: Given `templates/docs/prd/PRD-master.md`, When lo reescribo con la
  anatomia del metodo, Then su orden de secciones es: encabezado (Estado /
  Duenno / Actualizado / Alcance con su "NO toca"), `## 1. Resumen (hoy ->
  despues)`, `## 2. La historia`, `## 3. Objetivos / No-objetivos`,
  `## 4. Usuarios y jobs-to-be-done`, `## 5. Metricas de exito`,
  `## 6. Como funciona hoy -> como va a funcionar`, `## 7. Los datos`,
  `## 8. Pseudo-codigo (el acuerdo)`, `## 9. Restricciones y supuestos`,
  `## 10. Hitos -> features`, `## 11. Riesgos`, `## 12. Decisiones abiertas`; la
  seccion 2 trae los dos bloques ANTES y DESPUES con ejemplo narrado, y la 3 usa
  IDs citables (`O1`, `O2`, `NO1`).
- AC-2: Given el PRD reescrito, When leo sus secciones 7 y 8, Then el maestro
  lleva los datos a nivel PRODUCTO (entidades principales, disparadores,
  interruptores por cliente y candados) y el pseudo-codigo del acuerdo general
  con el esqueleto CUANDO / guards / ENTONCES / Promesas, declarando que cada
  feature refina el suyo en su spec; y declara explicitamente la regla dura: el
  PRD fija estructura en pseudo-codigo y explicaciones, **nunca** codigo final,
  pantallas terminadas ni configuracion.
- AC-3: Given el PRD reescrito, When reviso la cadena del arnes, Then conserva
  intacta la seccion de hitos con la tabla que alimenta el backlog y la linea
  `sh harness_cli add --name <slug> --service <servicio> --acceptance "<criterio>"`,
  y sigue enlazando `docs/prd/SDD-master.md` y `docs/constitution.md`.
- AC-4: Given `templates/docs/prd/COMO-ESCRIBIR-UN-PRD.md` (archivo nuevo), When
  lo leo, Then contiene: (a) que contiene / que nunca contiene un PRD; (b) "todo
  empieza con una historia" con el contraste asi-no / asi-si; (c) la tabla de
  tamano (ajuste 1 pagina / funcionalidad 3-8 / grande 10+ / producto nuevo =
  PRDs anidados); (d) la anatomia seccion por seccion con el ejemplo en
  miniatura, incluyendo Los datos y el Pseudo-codigo (CUANDO / guards /
  ENTONCES / Promesas); (e) el mapeo al arnes: PRD-master = producto,
  `docs/spec-feature-<id>-<slug>.md` = PRD del cambio, y el pseudo-codigo vive
  ahi.
- AC-5: Given ambos instaladores, When corren en layout subdir y root, Then
  siembran `docs/prd/COMO-ESCRIBIR-UN-PRD.md` en el `docs/` de la RAIZ tratando
  la guia como plantilla del arnes (`HARNESS_DOCS` / `$script:HarnessDocs`, con
  la ruta `prd/COMO-ESCRIBIR-UN-PRD.md`): solo si falta, refrescable con
  `--force`, incluida en los reset targets y en `required_assets`; las planillas
  `PRD-master.md` y `SDD-master.md` siguen siendo documentos del USUARIO
  (`PRD_DOCS` / `$script:PrdDocs`), que ningun reinstall ni `--force` pisa.
- AC-6: Given `rust/src/spec.rs`, When `spec_template()` genera un spec nuevo,
  Then el archivo trae, en este orden: encabezado (`Estado: draft` en la linea 3,
  `Plan:`, `Constitution:` y un puntero a `docs/prd/COMO-ESCRIBIR-UN-PRD.md`),
  `## La historia (antes -> despues)`, `## Hoy -> Como va a funcionar`,
  `## Recorridos de usuario (priorizados)`,
  `## Criterios de aceptacion (Given/When/Then)`, `## Los datos que se tocan`,
  `## Pseudo-codigo (el acuerdo)`, `## No funcionales`, `## Fuera de alcance`,
  `## Observaciones (decisiones pendientes)`; cada seccion nueva incluye su
  comentario guia y el bloque de pseudo-codigo trae el esqueleto
  CUANDO / guards / ENTONCES / Promesas.
- AC-7: Given el cambio de plantilla, When corro `cargo test`, Then
  `spec_template_should_declare_draft_and_sections` verifica las cuatro
  secciones nuevas, `spec_state` sigue leyendo `Estado:` dentro de las primeras
  diez lineas y los tests de `approve_spec` / firmas siguen verdes; los specs ya
  existentes en `docs/` NO se reescriben (`write_spec` solo crea si falta).
- AC-8: Given las superficies que genera `setup_harness.sh`
  (`write_agent_surface`) y su par ps1, When se instalan, Then la lista
  "Archivos principales" enlaza `docs/prd/COMO-ESCRIBIR-UN-PRD.md` describiendola
  como el metodo para escribir el PRD (historia, tamano, sin codigo final), sin
  tocar `write_basic_agent_surface` ni `.grok/GROK.md`.
- AC-9: Given `tests/setup_smoke.sh` y `tests/setup_smoke.ps1`, When corren,
  Then verifican con fixtures: (a) la guia sembrada en `docs/prd/` en layout
  subdir y root; (b) `PRD-master.md` con las secciones nuevas (`## 2. La
  historia`, `## 8. Pseudo-codigo (el acuerdo)` y `## 10. Hitos -> features`,
  con la numeracion nueva reflejada en los asserts que hoy buscan
  `## 7. Hitos -> features`) y con la linea `harness_cli add`;
  (c) el PRD del usuario sigue sobreviviendo a reinstall y a `--reset`
  (sentinels ya existentes); (d) la superficie instalada enlaza la guia.
  `bash tests/setup_smoke.sh` sale 0; sin `pwsh` la version ps1 se verifica
  estaticamente, como en las features #1 y #4 a #11.
- AC-10: Given las docs del repo, When leo `README.md`, `AGENTS.md`,
  `UPDATING.md` (raiz y `templates/`) y `docs/architecture.md`, Then describen la
  guia nueva, su siembra como plantilla del arnes y las secciones nuevas del
  spec generado; y las copias de este repo (`docs/prd/PRD-master.md`,
  `docs/prd/COMO-ESCRIBIR-UN-PRD.md`) quedan identicas a las de `templates/`.
- AC-11: Given el repo, When corro los comandos oficiales de
  `docs/verification.md`, Then pasan `bash harness_check.sh`,
  `cargo test --locked`, `cargo clippy --all-targets --all-features --locked --
  -D warnings` y `bash tests/setup_smoke.sh`.

## No funcionales
- SLOs: solo texto (plantillas + heredocs) y una funcion pura de Rust; sin
  dependencias nuevas, sin I/O extra ni cambios de rendimiento.
- Seguridad: sin secretos; el ejemplo del PRD usa datos ficticios y ninguna ruta
  fuera del proyecto.
- Observabilidad: la siembra de la guia se reporta con el aviso habitual del
  instalador (`write_file_notice` / `Install-HarnessAsset`), como el resto de
  `HARNESS_DOCS`; exit codes sin cambios.
- Multi-LLM: la guia se enlaza en las cuatro superficies sh y en la ps1; el
  metodo y el spec generado son identicos para cualquier backend (Claude, Codex,
  Gemini, Kimi).

## Fuera de alcance
- Tocar `docs/prd/SDD-master.md` (el "como" tecnico del proyecto no cambia de
  metodo en esta feature).
- Reescribir los specs ya cerrados (#1 a #11): la plantilla nueva rige para los
  specs que se generen de aqui en adelante.
- Editar `roles/*.md` (leader/implementer/reviewer): cambiarlos obliga a
  regenerar los espejos `.claude` / `.codex` / `.gemini` / `.kimi-code` con el
  instalador dentro del checkout fuente, y el gate de espejo de roles quedaria
  stale. La plantilla del spec ya impone el metodo donde importa.
- Validar en `harness_check.sh` que un spec traiga las secciones nuevas (gate
  nuevo = otra feature; aqui solo cambia la plantilla que se siembra).
- Traducir la guia o las planillas a ingles.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Alcance y vehiculo: DECIDIDO por el usuario (2026-08-11) — se aplica el metodo
  en las tres superficies (planilla PRD, guia nueva y `spec_template()` de Rust)
  y se ejecuta como feature #12 por el flujo del arnes.
- PROPUESTA a confirmar en la aprobacion: `COMO-ESCRIBIR-UN-PRD.md` se trata
  como plantilla del arnes (`HARNESS_DOCS`: se refresca con `--force`, entra en
  reset targets) y NO como documento del usuario, por el mismo criterio de la
  feature #11 con `kimi-cli-uso-eficiente.md`: es documentacion del metodo, no
  contenido del proyecto. `PRD-master.md` y `SDD-master.md` no cambian de
  regimen: siguen siendo del USUARIO.
- Datos y pseudo-codigo en los DOS niveles: DECIDIDO por el usuario
  (2026-08-11), corrigiendo la propuesta original de dejarlos solo en el spec.
  El PRD maestro los lleva a nivel PRODUCTO (entidades, disparadores,
  interruptores, candados y el acuerdo general) y el spec de cada feature refina
  el suyo a nivel CAMBIO. Para que el maestro no se desactualice feature tras
  feature, su seccion 8 declara que el detalle vinculante de cada cambio vive en
  su spec.
