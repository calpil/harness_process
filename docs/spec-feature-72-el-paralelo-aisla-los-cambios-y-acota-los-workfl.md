# Spec - Feature #72: El paralelo aisla los cambios y acota los workflows

Estado: approved
Aprobado: 2026-09-05T02:36:17Z por USUARIO (confirmacion explicita) - Aprobado por Alan en chat: 'approved spec #72'
Plan: docs/plan-feature-72-el-paralelo-aisla-los-cambios-y-acota-los-workfl.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md

## La historia (antes -> despues)

ANTES: Alan termina una tarea en Claude, vuelve a ver un recuadro de reporte
antiguo y no sabe si el paralelo fallo. Los agentes si se solapan y algunos
workflows esperan correctamente a sus trabajadores; pero varias features
escriben en checkouts compartidos, un Stop reclama cambios de otra sesion y
una publicacion arrastra un commit que se habia acordado dejar local. Un
workflow de revision registra 74 arranques para 14 tareas, con 12 fallidas.

DESPUES: Alan conserva el paralelo util. Cada feature tiene rutas de escritura
aisladas, los fallos quedan visibles y los cierres no arrastran tareas ajenas.
El aviso de feedback queda silencioso sin borrar los borradores. El arnes
distingue sus controles verificables de las preferencias del runtime de Claude.

Evidencia de partida: `progress/diagnostico-aviso-bug-report.md` del repo
principal. No se interpreta `subagent_count: 0` como ausencia de workflows,
ni la duracion de pared de un workflow como horas facturadas.

## Objetivos y no objetivos

- O-1: Evitar mezcla de cambios entre features y repositorios afectados.
- O-2: Acotar cada cierre a su feature y conservar la cobertura incompleta.
- O-3: Evitar relanzamientos y ampliaciones automaticas tras un fallo o cierre.
- O-4: Aplicar los arreglos con evidencia, espejos coherentes y reversibilidad.
- NO-1: No construir otro runtime de agentes ni modificar el binario de Claude.
- NO-2: No reescribir commits publicados, mover trabajo vivo ni resolver otras features.

## Hoy -> Como va a funcionar

```
HOY:     varias features -> checkout compartido -> Stop global -> nuevos pendientes
         workflow fallido -> reintentos/reanudacion -> resultados faltantes ocultos
DESPUES: feature -> identidad y rutas aisladas -> trabajo -> verificacion completa
                  fallo de aislamiento -> arranque rechazado sin falso in_progress
         Stop -> contexto de la sesion -> pendientes propios -> cierre del objetivo
         fallo de tarea -> resultado incompleto explicito -> decision, no otra ronda
```

## Recorridos de usuario (priorizados)

- P1: Alan abre dos features y cada una modifica solo sus worktrees autorizados.
- P1: Alan revisa una feature; si falta un agente, ve que falta y no recibe un aprobado.
- P1: Alan termina una tarea sin que Stop le asigne los cambios de otra sesion.
- P1: Alan integra o publica y conoce el conjunto completo de commits implicados.
- P2: Alan mantiene sus sesiones existentes sin migraciones destructivas ni avisos repetidos.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un proyecto Git con features concurrentes, When se inicia o reanuda una feature, Then el arnes verifica la identidad de su rama y worktree; rechaza compartir checkout entre features, el bypass inseguro con --sin-worktree y el fallback silencioso ante un fallo de Git; un arranque rechazado no queda marcado in_progress ni sobrescribe el estado previo. El uso serial sin worktree y los proyectos sin Git se identifican como no aislados y no habilitan paralelo de escritura.
- AC-2: Given una feature que afecta repos separados o anidados, incluido un docs que es otro repo Git, When se resuelven sus rutas de trabajo, Then cada repo escribible tiene un worktree propio de la feature y el plan/spec/evidencia apuntan al docs correcto; una ruta ambigua o compartida impide autorizar esas escrituras. Los hooks de edicion usan esa resolucion y no confunden un directorio docs vacio con permiso para volver al checkout compartido.
- AC-3: Given una integracion o publicacion desde una feature, When el arnes prepara la operacion, Then presenta origen, destino y todo el rango de commits; si hay cambios ajenos o procedencia ambigua bloquea la automatizacion y pide decision. Los trabajadores no hacen operaciones Git compartidas, las integraciones del arnes sobre el mismo destino se serializan y no se publica ni se cambia el destino sin autorizacion. Una prueba reproduce el caso de un commit propio cuyo padre es otro commit pendiente.
- AC-4: Given dos sesiones/features con cambios pendientes distintos, When se ejecuta Stop para una, Then verifica solo su contexto y no la bloquea por la otra; si el contexto es ambiguo, informa una sola vez y no inicia una reparacion global. Se conserva la defensa contra reentrada de Stop y el check global sigue disponible explicitamente, sin quedar desactivado.
- AC-5: Given una delegacion paralela del arnes, When se preparan sus etapas, Then cada tarea cita su AC, declara rutas y dependencias y la revision comienza despues de terminar los escritores; el modo habitual usa hasta cuatro tareas por etapa, sin crear una tarea nueva por cada hallazgo. Ante fallo no se relanza automaticamente el workflow: se muestran las tareas y los intentos observados y se requiere una decision para otra ejecucion. Los controles de lanzamiento verifican lo que pueden imponer y no presentan la preferencia small como limite duro del proveedor.
- AC-6: Given tareas requeridas fallidas, canceladas, sin resultado o sin evidencia, When se agregan resultados y se solicita revision/cierre, Then se conservan sus identificadores y estados; el resultado queda incompleto y el gate no permite approved/done hasta cubrir la verificacion requerida. Eliminar valores nulos no puede convertir cobertura parcial en completa. Una sustitucion manual debe aportar evidencia por AC, no fingir que el agente termino.
- AC-7: Given que el objetivo solicitado ya esta resuelto, When el lider entrega el resultado y encuentra algo adyacente, Then registra el hallazgo separado en progress y no inicia otra feature, workflow o reparacion sin nueva autorizacion; el cierre explica lo verificado, lo pendiente y el motivo real del aviso, sin encadenar tareas.
- AC-8: Given los cambios aprobados del arnes, When se prueban e instalan, Then quedan coherentes raiz, templates, roles y sus espejos Claude/Gemini/Codex/Kimi; los casos de concurrencia, Stop y cobertura incompleta se prueban por comportamiento y el gate de espejos pasa. Las diferencias especificas de Claude quedan en su adaptador y se informa si PowerShell no pudo ejecutarse.
  Comando: `cd rust && cargo test --locked`
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`
  Comando: `bash tests/setup_smoke.sh`
  Comando: `bash tests/parity_check.sh`
- AC-9: Given la configuracion local de Claude de Alan, When se silencia el feedback y se prefiere un workflow pequeno, Then existe respaldo, el JSON es valido y no cambian otras preferencias ni se borran/envian drafts; se distingue la comprobacion del archivo de la confirmacion visual del recuadro en una sesion. Este ajuste reversible ya fue aplicado por autorizacion de arreglar los problemas, fuera del codigo del arnes.
- AC-10: Given features y sesiones de realestate ya activas en checkouts compartidos, When se despliega el arreglo, Then se inventarian y preservan sus cambios; no se matan procesos, se mueven commits, se cambian ramas ni se eliminan worktrees automaticamente. La actualizacion usa respaldo y preflight; cualquier migracion que requiera pausar escritores queda identificada para coordinarla con Alan, sin afirmar que el trabajo vivo ya quedo aislado.

## Los datos que se tocan

- Feature: identidad, estado, rama, worktree y asociacion de repos afectados.
- Sesion: vinculacion verificable con feature y rutas permitidas; sin inferir
  propiedad de un archivo solo porque aparezca en un git status global.
- Delegacion/evidencia: AC, tarea esperada, rutas, dependencia, estado terminal,
  intentos observados y resultado o error; se conserva la lista completa.
- Integracion: base, destino y rango de commits presentado al usuario.
- Configuracion local: preferencias de feedback/workflows y respaldo anterior.
- Disparadores: start, edicion, lanzamiento/reanudacion, Stop, revision y close.
- Candados: identidad real de Git y exclusividad por feature/repositorio;
  operacion de integracion serializada por destino, sin perder actualizaciones.
- Interruptores: el modo serial explicito no concede aislamiento; restaurar
  preferencias locales no desactiva los gates de calidad o de seguridad.

## Pseudo-codigo (el acuerdo)

```
AL INICIAR: resolver repos y propietario -> validar aislamiento y colisiones
            si falla: informar y conservar el estado anterior
            si pasa: registrar estado, rutas y documentos de esa feature
AL DELEGAR: fijar AC y tareas acotadas -> escritores independientes -> esperar
            guardar TODOS los resultados -> revisar solo con evidencia completa
            si falla: informar el alcance del fallo; no relanzar por inercia
AL PARAR:   resolver la sesion -> verificar lo propio sin autoasignar lo ajeno
AL CERRAR:  verificar AC y commits -> integrar al destino autorizado -> terminar
            hallazgo nuevo -> anotarlo separado, no convertirlo en otro encargo
```

Promesas: no falso aislamiento; no aprobado con cobertura faltante; no mutacion
automatica del trabajo vivo; no limite ficticio de tiempo, coste o reintentos.

## No funcionales y verificacion

- Seguridad: no secretos en evidencia; backups antes de configuracion/instalacion;
  no force push, resets, stash ajeno, borrado de drafts ni cambios al PRD del usuario.
- Observabilidad: errores accionables con feature/ruta/tarea y exit code estable;
  los logs separan reintentos internos de relanzamientos pedidos por el arnes.
- Verificacion: fixtures temporales con dos features, repo docs independiente,
  fallo de start, dos sesiones Stop y tarea fallida; pruebas negativas deben fallar
  antes del arreglo. No pruebas basadas en grep del fuente ni cero tests en verde.
- Cierre: harness_check limpio, evidencia por AC y revision registrada; no se
  ocultan fallos de entorno o checks globales que requieran decision explicita.
- Limite conocido: Claude documenta small como consejo y no expone en la referencia
  consultada un limite configurable de los reintentos internos vistos en el journal.
  No se promete detenerlos con un cambio de prompt. Si el control no es imponible
  en el adaptador, debe indicarse y no etiquetarse como garantia estructural.
- Limite de seguridad: los hooks de edicion y gates del arnes no son un sandbox
  contra cualquier shell externo. No se afirma interceptar todo git push manual.

## Alcance de instalacion y fuera de alcance

Se corrige harness_process y se prepara/aplica su actualizacion segura en la
instalacion de realestate afectada. No se distribuyen cambios a otros proyectos.
No se migran automaticamente las features vivas #98, #122 o #126 ni se toca
la feature pendiente #71 del arnes. No se repara historial remoto ya publicado.
No se modifica el runtime propietario de Claude ni se apagan todos sus workflows.

## Observaciones (decisiones pendientes)

- OBS-1: Alan debe aprobar este spec mostrado en chat y abierto en su editor.
  La peticion de arreglar los problemas autoriza preparar el trabajo, pero no
  sustituye la aprobacion del spec exigida por el articulo 2 de la constitucion.
- OBS-2: La rama de integracion se preguntara antes de close --status done --to.
- OBS-3: La migracion de sesiones vivas, si requiere detener escritores o mover
  cambios, se coordinara por separado; no forma parte de una accion automatica.

## Fuentes y memoria de diseno

- Diagnostico local: progress/diagnostico-aviso-bug-report.md (repo principal).
- Claude workflows: https://code.claude.com/docs/en/workflows (consulta 2026-09-04).
- Lecciones: promesas-estructurales-vs-disciplina y criterios-de-cierre-que-se-pueden-fallar.
- Contexto: mapa cubre el tema, grafo reciente; hub sin datos concretos de impacto.
