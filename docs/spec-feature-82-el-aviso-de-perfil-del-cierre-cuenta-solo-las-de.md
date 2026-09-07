# Spec - Feature #82: el aviso de perfil del cierre cuenta solo las decisiones posteriores a la ultima entrada del perfil

Estado: approved
Aprobado: 2026-09-07T13:51:57Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado' a los siete AC; OBS-1 bitacora + backlog (gana la mas reciente); OBS-2 started_at; OBS-3 perfil remove no cuenta
Plan: docs/plan-feature-82-el-aviso-de-perfil-del-cierre-cuenta-solo-las-de.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)
ANTES: Alan cierra la #79 y el arnes le avisa "268 decision(es) sin incorporar
al perfil". Son las mismas 268 que le aviso al cerrar la #78, la #80 y la #83:
decisiones de agosto, casi todas ya destiladas en las ocho entradas del perfil
sin citar su numero. El aviso mide el acumulado historico, asi que sale en cada
cierre para siempre, y la salida fue subir el umbral a 300: una regla que ya no
mide nada.
DESPUES: Alan agrega una entrada al perfil. El aviso vuelve a contar desde ahi:
solo las decisiones sin incorporar registradas DESPUES de esa entrada. Si en dos
semanas se juntan mas de 25 nuevas, el cierre avisa, y `lecciones status` y
`perfil sugerir` le muestran las nuevas separadas del acumulado y desde cuando
se cuentan. El umbral vuelve a 25 y significa crecimiento, no historia.

## Hoy -> Como va a funcionar
```
HOY                                              DESPUES
close done -> perfil_pendientes()                close done -> perfil::pendientes()
   = TODOS los registros sin incorporar             |__ corte = ultima entrada del perfil:
   > umbral -> [i] aviso (en cada cierre)           |     la mas reciente entre la ultima linea
                                                    |     `perfil add|replace` de history.md y el
                                                    |     `started_at` de la feature mas alta que
                                                    |     cita el perfil (backlog)
                                                    |__ nuevas = sin incorporar con momento > corte
                                                    |__ nuevas > umbral -> [i] aviso
                                                          (dice nuevas, total y corte)
lecciones status -> "N sin incorporar"           lecciones status -> "N nuevas desde <corte>, M en total"
perfil sugerir   -> "M sin incorporar"           perfil sugerir   -> "M sin incorporar, N posteriores a <corte>"
```
El momento de cada registro: una linea de la bitacora trae su timestamp; una
decision de plan o spec no tiene fecha propia y se fecha por el `started_at` de
su feature en el backlog.

## Recorridos de usuario (priorizados)
- P1: Como usuario que ya alimento el perfil, quiero que el aviso del cierre
  cuente solo lo acumulado desde mi ultima entrada, para que avise cuando hay
  material nuevo y no en cada cierre por la historia.
- P1: Como usuario mirando `lecciones status` o `perfil sugerir`, quiero ver las
  dos cuentas (nuevas y total) y desde cuando se cuenta, para saber si el aviso
  es por crecimiento o por acumulado.
- P2: Como usuario que perdio la bitacora (o edito el perfil a mano), quiero que
  el corte se reconstruya desde el backlog, para no volver a los 268 despues de
  un incidente hasta el proximo `perfil add`.
- P2: Como usuario con el perfil vacio, quiero que se cuente todo como hoy: sin
  entradas no hay "desde cuando".

## Criterios de aceptacion (Given/When/Then)
- AC-1: Given un perfil con entradas, una bitacora con una linea `perfil add`
  (o `perfil replace`) fechada y decisiones sin incorporar anteriores y
  posteriores a esa linea, When un cierre `done` evalua
  `rules.perfil_pendientes_max`, Then compara contra el umbral SOLO las
  posteriores: con las posteriores por debajo del umbral y el total por encima
  no hay aviso, y cuando lo hay dice cuantas nuevas, cuantas en total y desde
  cuando. stdout y exit code no cambian; `0` sigue apagando el aviso.
  Comando: `cd rust && cargo test --locked aviso_de_perfil_cuenta`
- AC-2: Given una decision de plan o de spec (sin fecha propia), When se decide
  si es posterior al corte, Then se fecha por el `started_at` de su feature en
  el backlog; sin feature o sin `started_at` se cuenta como nueva (ante la duda,
  avisa de mas, nunca de menos).
  Comando: `cd rust && cargo test --locked momento_de_plan`
- AC-3: Given un perfil con entradas que citan `#<id>` pero SIN linea
  `perfil add|replace` en la bitacora (bitacora perdida o perfil editado a
  mano), When se calcula el corte, Then es el `started_at` de la feature citada
  mas alta que este en el backlog; con las dos senales gana la mas reciente.
  `perfil remove` no cuenta como entrada.
  Comando: `cd rust && cargo test --locked corte_desde_el_backlog`
- AC-4: Given un perfil sin entradas, o con entradas sin cita y sin linea en la
  bitacora, When se evalua el umbral, Then se cuenta todo como hoy y el texto
  dice que no hay corte.
  Comando: `cd rust && cargo test --locked perfil_sin_corte`
- AC-5: Given las dos cuentas, When `lecciones status` (texto y `--json`) y
  `perfil sugerir` informan, Then muestran nuevas, total y el corte con su
  origen (`bitacora` o `backlog`). En el JSON `perfil_pendientes` sigue siendo
  el numero que se compara con el umbral (para los consumidores de la #80) y se
  agregan `perfil_pendientes_total`, `perfil_corte` y `perfil_corte_origen`.
  Comando: `cd rust && cargo test --locked dos_cuentas`
- AC-6 (MANUAL): Given este repo, When se cierra esta feature, Then el backlog
  pierde la regla provisoria `perfil_pendientes_max: 300` (vuelve al default
  25) y `lecciones status` muestra el corte desde el backlog (la bitacora se
  perdio el 2026-09-06) con menos de 25 nuevas: el aviso deja de salir en cada
  cierre sin subir el umbral.
- AC-7: Given README, `UPDATING.md` (las dos copias) y `architecture.md`, When
  se cierra, Then describen las dos cuentas y el corte, y las dos copias de
  `UPDATING.md` son identicas.
  Comando: `cmp UPDATING.md templates/UPDATING.md && grep -q "perfil_pendientes_total" README.md UPDATING.md docs/architecture.md`

## Los datos que se tocan
- disparador: el cierre `done` (aviso por stderr), `lecciones status`,
  `perfil sugerir`.
- interruptor: `rules.perfil_pendientes_max: 0` apaga el aviso, como hoy.
- candado: no aplica; nada se escribe, el calculo es idempotente.
- entradas: `progress/history.md` (lineas `- <ts> perfil add|replace ...`, y
  el timestamp de cada decision), `feature_list.json` (`started_at` por
  feature), `docs/perfil-usuario.md` (citas `#<id>` de las entradas).
- salida: texto. El backlog de ESTE repo pierde `rules.perfil_pendientes_max:
  300` al cerrar (AC-6); es dato, no codigo, y no se commitea.

## Pseudo-codigo (el acuerdo)
```
CUANDO close done | lecciones status | perfil sugerir

  registros = recolectar()      (bitacora: su timestamp; plan/spec: started_at de su feature)
  corte     = la mas reciente entre
                la ultima linea `perfil add|replace` de history.md
                y el started_at de la feature mas alta que cita el perfil
  ¿no hay corte?  -> nuevas = total (como hoy), y el texto lo dice

  total  = registros sin incorporar
  nuevas = los de total con momento > corte, o sin momento
  aviso  = umbral > 0 y nuevas > umbral
```
Promesas: no escribe nada · el aviso sigue por stderr y no cambia stdout ni el
exit code · `0` apaga · sin corte nunca cuenta menos que hoy · sin fecha se
cuenta (avisa de mas, no de menos).

## No funcionales
- SLOs: lee tres archivos que ya se leian; sin red, sin hub.
- Seguridad: nada nuevo; no escribe.
- Observabilidad: el aviso y `lecciones status --json` dicen el corte y de donde
  salio, asi el usuario puede discutir el numero.

## Fuera de alcance
- Fechar las entradas dentro de `docs/perfil-usuario.md` (cambiar el formato del
  documento del usuario).
- Marcar como incorporadas las decisiones sin cita `#<id>`: sigue siendo por
  cita, como en la #19.
- Tocar `perfil sugerir` mas alla de las cuentas (sigue listando todo lo sin
  incorporar, agrupado por feature).

## Observaciones (decisiones pendientes)
- OBS-1 (de donde sale "la ultima entrada"): propuesta: la mas reciente entre la
  bitacora (`perfil add|replace`) y el backlog (`started_at` de la feature mas
  alta citada). Alternativas: (a) solo la bitacora: exacta, pero se pierde con
  ella (hoy mismo seguiriamos en 268 hasta el proximo `perfil add`); (b) una
  fecha escrita dentro del perfil por `perfil add`: sobrevive a todo, pero
  cambia el formato del documento del usuario y una edicion a mano no la
  actualiza. DECIDIDO (Alan, 2026-09-07): bitacora + backlog, gana la mas
  reciente.
- OBS-2 (como se fechan plan y spec): propuesta: `started_at` de su feature,
  cota inferior; sin fecha se cuenta. DECIDIDO (Alan, 2026-09-07): si.
- OBS-3 (`perfil remove`): propuesta: no cuenta como entrada, quitar no es
  incorporar. DECIDIDO (Alan, 2026-09-07): no cuenta.
