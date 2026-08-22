# Spec - Feature #51: revision_adversarial_y_modelos_por_rol

Estado: approved
Aprobado: 2026-08-22T12:52:00Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #51 en el chat (18 AC): modelos por rol (implementer opus-5, leader y reviewer fable-5, los tres xhigh) en tabla unica de cada instalador, reviewer adversarial con verificacion independiente, y el comando revision que arma el paquete acotado con presupuesto declarado. Disparador: una verificacion costo 10M de tokens
Plan: docs/plan-feature-51-revision-adversarial-y-modelos-por-rol.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: Alan quiere que el implementer piense con Opus y que los tres roles
trabajen con el mismo esfuerzo. Edita el frontmatter de
`.claude/agents/*.md`... y a la siguiente instalacion el arnes se lo pisa: el
modelo y el `effort` estan escritos a mano dentro de `setup_harness.sh` y
`setup_harness.ps1`, iguales para los tres roles. Cada `setup` le borra la
decision.

Y verificar lo implementado se volvio carisimo: una sola verificacion llego a
gastar **10 millones de tokens**. El agente sale a explorar el repo entero, abre
archivos que no hacian falta, vuelve a leer lo que ya estaba en el spec y pega
bloques de codigo en el veredicto que nadie va a releer. A ese precio, revisar
en serio deja de ser viable: la tentacion es revisar menos, que es exactamente
lo contrario de lo que hace falta.

Y lo que se gasta, encima, se gasta confirmando. El reviewer hace lo que hace
cualquiera al que le muestran una tabla de evidencia: la valida. Lee
`impl-<id>.md`, ve que cada AC tiene su fila, cruza que el test exista, y firma.
Nunca intenta que el AC falle. Con la tabla delante, encontrar el caso que rompe
la promesa es antinatural: el sesgo juega a favor de aprobar. Diez millones de
tokens para terminar diciendo que si.

DESPUES: revisar cuesta lo que tiene que costar. El arnes le entrega al reviewer
el paquete ya armado — `revision --feature <id>`: los AC con su estado en
verify, la tabla de evidencia, los archivos tocados y el diff, todo acotado a un
presupuesto que se declara — asi que el turno se gasta pensando y no explorando.
El objetivo es que una revision entre en un paquete que se lee de una sentada,
no en diez millones de tokens de paseo por el repo.

Y lo que se gasta, se gasta bien: el reviewer cambia de postura. Su trabajo deja
de ser confirmar la evidencia y pasa a ser **intentar romperla**. Por cada AC
busca el caso que lo haria fallar y, cuando la evidencia dice "verde", lo
comprueba por su cuenta antes de creerle. Aprobar pasa a significar "no pude
tumbarlo", y eso se consigue con un paquete acotado, no con exploracion libre.

Los modelos, ademas, quedan donde tienen que estar y sobreviven a las
instalaciones: implementer con Opus, lider y reviewer con Fable, los tres en
`xhigh`.

## Hoy -> Como va a funcionar

```
HOY                                    DESPUES
setup_harness.{sh,ps1}                 los tres roles, un solo lugar:
  los 3 roles: fable-5 / max             implementer -> claude-opus-5  / xhigh
  (y pisa lo que el usuario edito)       leader      -> claude-fable-5 / xhigh
                                         reviewer    -> claude-fable-5 / xhigh

reviewer: lee impl-<id>.md y confirma  reviewer: intenta REFUTAR cada AC
  explora el repo a mano                 y verifica los verdes por su cuenta
  pega bloques de codigo                 arranca del paquete, cita archivo:linea

(no existe)                            harness revision --feature <id>
                                         |__ AC del spec + su estado en verify
                                         |__ tabla de evidencia de impl-<id>
                                         |__ diff de la rama (acotado)
                                         |__ rutas protegidas tocadas
                                         |__ presupuesto de lineas explicito
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero que el implementer corra con Opus y los tres roles en
  `xhigh`, para que el que escribe codigo tenga la cabeza mas grande y nadie
  quede corto de esfuerzo.
- P1: Como Alan, quiero que esa eleccion sobreviva a `setup_harness.sh`, para no
  volver a editar el frontmatter despues de cada instalacion.
- P1: Como Alan, quiero que el reviewer intente romper cada AC en vez de
  confirmarlo, para que un veredicto `approved` signifique "no pude tumbarlo" y
  no "la tabla estaba completa".
- P1: Como Alan pagando los tokens, quiero que una verificacion no vuelva a
  costar 10 millones de tokens, para poder revisar en serio en cada cierre sin
  pensar en el precio.
- P2: Como reviewer, quiero recibir el material ya juntado, para gastar el
  presupuesto en pensar y no en explorar.

## Criterios de aceptacion (Given/When/Then)

### Modelos y esfuerzo por rol

- AC-1: Given una instalacion nueva, When corro `setup_harness.sh`, Then
  `.claude/agents/implementer.md` queda con `model: claude-opus-5` y
  `effort: xhigh`.
- AC-2: Given esa misma instalacion, When corro el instalador, Then
  `.claude/agents/leader.md` y `.claude/agents/reviewer.md` quedan con
  `model: claude-fable-5` y `effort: xhigh`.
- AC-3: Given el instalador de Windows, When corro `setup_harness.ps1`, Then
  produce exactamente el mismo frontmatter que la version Bash para los tres
  roles (paridad verificada por assert de contenido).
- AC-4: Given los espejos ya commiteados en este repo, When corro el instalador,
  Then el resultado COINCIDE con lo commiteado: reinstalar deja de producir un
  diff en `.claude/agents/*.md`.
- AC-5: Given que el modelo o el esfuerzo de un rol tengan que cambiar mañana,
  When alguien lo edite, Then hay UN solo lugar por instalador donde tocarlo (la
  tabla de roles), y `roles/README.md` dice cual es.

### Revision adversarial

- AC-6: Given el rol reviewer, When lo leo, Then su instruccion primaria es
  intentar REFUTAR cada AC-n (buscar la entrada limite, el camino de error, el
  caso concurrente que lo rompe) y solo darlo por bueno cuando el intento fallo.
- AC-7: Given un AC que la evidencia declara verde, When el reviewer lo revisa,
  Then verifica por su cuenta antes de creerle: corre el comando del AC o lee el
  codigo SIN partir de la conclusion del implementer, y deja registrado que
  verifico de forma independiente.
- AC-8: Given un hallazgo del reviewer, When lo escribe en el veredicto, Then
  incluye el caso concreto que rompe la promesa (entradas y resultado esperado),
  no una impresion general.
- AC-9: Given que el reviewer no pudo refutar ningun AC, When cierra su
  veredicto, Then el `approved` dice explicitamente que significa: "no se pudo
  tumbar con los casos probados", nombrando lo que NO se probo.

### Disciplina de tokens

- AC-10: Given el rol reviewer, When lo leo, Then trae reglas concretas y
  verificables: partir del diff y no del repo, leer por rangos en vez de
  archivos enteros, citar `archivo:linea` en lugar de pegar codigo, y no repetir
  en el veredicto lo que ya esta en el spec o en la evidencia.
- AC-11: Given `harness revision --feature <id>`, When lo corro, Then imprime en
  UN solo documento: los AC-n del spec con su estado en `verify-<id>.md`, la
  tabla de evidencia de `impl-<id>.md`, la lista de archivos tocados por la rama
  de la feature, el diff acotado y las rutas protegidas tocadas (si las hay).
- AC-12: Given un diff grande, When corro `revision`, Then el paquete respeta un
  presupuesto de lineas (`--max-lineas`, default razonable) y dice explicitamente
  que recorto y cuanto quedo afuera: nunca trunca en silencio.
- AC-12b: Given cualquier paquete, When termina de imprimirse, Then reporta su
  propio tamaño (lineas y estimacion de tokens) para que el costo de revisar sea
  visible ANTES de gastarlo, y no una sorpresa al final.
- AC-12c: Given el paquete por default de una feature normal de este repo, When
  lo mido, Then entra holgado en un turno de revision (orden de magnitud: miles
  de tokens, no millones), y si una feature lo excede el paquete lo dice en vez
  de crecer sin limite.
- AC-13: Given una feature sin rama propia (modo clasico) o sin `verify`/`impl`
  todavia, When corro `revision`, Then arma el paquete con lo que exista y
  nombra lo que falta, sin fallar.
- AC-14: Given `revision --feature <id> --json`, When lo corro, Then la misma
  informacion sale en JSON, para que un agente la consuma sin parsear texto.

### Verificacion

- AC-15: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh`, Then los cuatro terminan limpios, con tests del paquete
  (presupuesto, ausencias, JSON) y asserts de los instaladores.
- AC-16: Given el cierre de ESTA feature, When el reviewer la revisa, Then usa
  `revision --feature 51` como material y su veredicto declara que intento
  refutar cada AC: la primera aplicacion de la regla es sobre si misma.

## Los datos que se tocan

- disparador: `setup_harness.{sh,ps1}` al generar los espejos de roles, y el
  comando nuevo `revision`.
- interruptor: ninguno nuevo. El paquete es un comando que se corre a mano.
- candado: ninguno; `revision` es de solo lectura y no escribe estado.
- La tabla de roles de cada instalador gana el modelo y el esfuerzo por rol (hoy
  son literales repetidos en la llamada).
- `roles/reviewer.md` (+ su espejo en `templates/roles/`) suma la postura
  adversarial y las reglas de tokens.
- `roles/README.md` (+ espejo) documenta donde se cambia el modelo/effort.
- `revision` NO escribe archivos: imprime. Si mañana hace falta persistirlo, es
  otra feature.

## Pseudo-codigo (el acuerdo)

```
CUANDO el instalador genera los espejos de roles

  cada rol trae SU modelo y SU esfuerzo de una tabla unica
  ENTONCES el frontmatter sale de ahi,
           con la restriccion de que los dos instaladores escriben lo mismo
           y de que reinstalar no cambia lo que ya estaba.

CUANDO el reviewer revisa una feature

  para cada AC-n:
    intenta ROMPERLO (entrada limite, error, concurrencia)
    si la evidencia dice verde -> lo comprueba por su cuenta
  ENTONCES aprueba solo lo que no pudo tumbar,
           y dice que casos probo y cuales no.

CUANDO se pide el paquete de revision

  juntamos AC + estado de verify + evidencia + archivos tocados + diff
  ¿entra en el presupuesto? -> si no, recortamos y DECIMOS cuanto quedo afuera
  ENTONCES lo entregamos en un solo documento (texto o JSON),
           con la restriccion de no leer nada que el reviewer no vaya a usar.
```

Promesas: el modelo y el esfuerzo sobreviven a la instalacion · aprobar
significa "no pude romperlo" · el recorte se dice, nunca se silencia · el
paquete no escribe nada.

## No funcionales

- SLOs: `revision` responde en menos de 2 s en este repo (es git + leer dos
  archivos); el presupuesto por default acota la salida a algo que un agente
  pueda leer entero sin gastar el turno.
- Seguridad: `revision` es de solo lectura; nunca imprime el contenido de
  archivos de rutas protegidas — los nombra y avisa que fueron tocados.
- Observabilidad: el paquete dice siempre que incluyo, que recorto y que falta.

## Fuera de alcance

- Correr la revision automaticamente al cerrar: `revision` se invoca cuando el
  reviewer la necesita.
- Un segundo revisor con otra mirada (dos pasadas): se evaluo y quedo afuera por
  costo; esta feature hace adversarial UNA pasada.
- Persistir el paquete en `docs/`: hoy se imprime.
- Cambiar modelos de Codex, Gemini, Kimi o Antigravity: el pedido es sobre los
  subagentes de Claude.

## Observaciones (decisiones pendientes)

- OBS-1 [DECIDIDA por el USUARIO, 2026-08-22]: implementer con `claude-opus-5`;
  leader y reviewer con `claude-fable-5`; los tres con `effort: xhigh`.
- OBS-2 [DECIDIDA por el USUARIO, 2026-08-22]: la revision adversarial incluye
  verificadores independientes (el reviewer comprueba por su cuenta, sin partir
  de la conclusion del implementer), no solo cambiar la postura.
- OBS-3 [DECIDIDA por el USUARIO, 2026-08-22]: la optimizacion de tokens se hace
  por los dos lados: disciplina escrita en el rol Y el paquete que arma el
  arnes.
- OBS-4 [DECIDIDA por el USUARIO, 2026-08-22]: todo en una sola feature.
- OBS-5 [DATO DEL USUARIO, 2026-08-22]: el disparador de esta feature es
  concreto — verificar lo implementado llego a gastar **10 millones de tokens**
  en una sola pasada. El objetivo primario no es que la revision sea mas
  estricta, sino que sea estricta Y acotada: el paquete existe para que el
  reviewer no tenga que explorar.
- OBS-6 [REGISTRADA]: el `effort: max` que estaba commiteado en los espejos era
  una edicion manual del usuario que el instalador venia pisando; esta feature
  la vuelve innecesaria al fijar `xhigh` en la fuente.
