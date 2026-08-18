# Spec - Feature #19: perfil_de_usuario

Estado: approved
Aprobado: 2026-08-16T23:32:38Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #19 en el chat (AskUserQuestion: 'Si, lo apruebo'), con el spec mostrado en el chat y abierto en su editor. 20 AC. Decisiones OBS-1..OBS-5: inyeccion en las CUATRO superficies reales (GROK.md de la raiz no existe, correccion al backlog), limite de 1500 chars, sugerir marca lo ya incorporado, el escaneo de secretos y unicode invisible BLOQUEA (el perfil se versiona y se inyecta: un secreto ahi es irreversible), y sugerir lee history.md + planes + los DECIDIDO de los specs.
Plan: docs/plan-feature-19-perfil-de-usuario.md
PRD: docs/prd/aprendizaje/PRD-aprendizaje.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: en la #14, ante un fork de concurrencia, Alan eligio la opcion segura
aunque costara mas. En la #15 y la #16 exigio sincronia total con el sistema
externo, incluido el backfill de lo ya cerrado. En la #18 rechazo una cadencia de
10 porque "se vuelve ruido de fondo". Las tres decisiones estan escritas,
fechadas y firmadas en `progress/history.md`.

**Y ningun agente las lee nunca.** Cada sesion nueva —Claude, Gemini, Codex,
Kimi— arranca sin saber nada de eso, propone la opcion rapida en vez de la
segura, y Alan lo corrige otra vez. La correccion se escribe en `history.md`,
otra vez, y muere ahi otra vez. El arnes acumulo 18 features de evidencia sobre
como quiere trabajar su usuario y no usa ni una linea.

DESPUES: esas decisiones repetidas se destilan en `docs/perfil-usuario.md` —
pocas entradas, cortas, con limite duro— y el instalador las **inyecta en las
superficies** que cada backend lee al arrancar. Codex abre la sesion sabiendo que
ante un fork de consistencia se elige la opcion segura, sin que nadie se lo diga.

Y como es el documento del usuario, nada entra sin su si explicito: el arnes
junta la evidencia (`perfil sugerir`), el agente propone, Alan decide.

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES

decisiones del usuario                  decisiones del usuario
  |__ approve-spec --nota "..."           |__ approve-spec --nota "..."
  |__ advance --nota "Decision: ..."      |__ advance --nota "Decision: ..."
  `__ mueren en history.md                `__ history.md
                                                |  perfil sugerir (junta y agrupa,
                                                |  no escribe nada)
                                                v
                                          el agente PROPONE una entrada durable
                                                |  Alan dice que si
                                                v
                                          perfil add --yes -> docs/perfil-usuario.md
                                                |  (limite duro: no auto-compacta)
                                                v
                                          instalador: bloque delimitado en
                                          CLAUDE.md / AGENTS.md / GEMINI.md / LLM.md
                                                |
                                                v
                                          la proxima sesion arranca sabiendo
```

## Recorridos de usuario (priorizados)

- P1: Como Alan, quiero no repetir por cuarta vez la misma preferencia, porque el
  arnes ya la tiene escrita tres veces.
- P1: Como agente de cualquier backend, quiero arrancar sabiendo como quiere
  trabajar este usuario, sin tener que leer 18 features de historial.
- P1: Como Alan, quiero que **nada** entre a mi perfil sin mi si explicito, y
  poder sacar una entrada que quedo mal.
- P2: Como lider, quiero que el arnes me junte la evidencia de lo que ya se
  decidio, para proponer entradas con fundamento y no de memoria.
- P2: Como usuario de un proyecto que no quiere perfil, quiero no ver nada: sin
  `docs/perfil-usuario.md` las superficies quedan como hoy.

## Criterios de aceptacion (Given/When/Then)

### El documento

- AC-1: Given un proyecto sin `docs/perfil-usuario.md`, When corre el instalador
  (`sh` o `ps1`), Then se siembra el archivo con su encabezado y **sin ninguna
  entrada**; y como es documento del USUARIO, un reinstall **no lo pisa** y
  `--reset` **no lo borra** (mismo trato que `PRD-master.md`, no que las
  plantillas del arnes).
- AC-2: Given `docs/perfil-usuario.md`, Then su formato es un encabezado fijo mas
  entradas como items de lista (`- <texto>`), una por linea logica; el limite
  cuenta **solo** los caracteres de las entradas, no el encabezado.

### El limite duro

- AC-3: Given un perfil cuyas entradas suman menos de 1500 caracteres, When se
  agrega una entrada que haria superar el limite, Then el comando **falla con
  exit 2**, no escribe nada, y el mensaje trae: cuanto ocupa hoy, cuanto ocuparia,
  **la lista de las entradas actuales**, y la instruccion de consolidar
  (`replace`/`remove`) y reintentar **en el mismo turno**. Nunca se auto-compacta
  ni se descarta nada en silencio.
- AC-4: Given un perfil cualquiera, When corre `perfil show`, Then se imprimen las
  entradas numeradas y el uso en la forma `[N% - X/1500 chars]`, para que se sepa
  cuanta capacidad queda antes de agregar.
- AC-5: Given una entrada existente, When se la reemplaza por una mas larga que
  no entra, Then `replace` **tambien** falla por limite (el limite aplica a toda
  escritura, no solo a `add`).

### Solo el usuario escribe

- AC-6: Given cualquier intento de `perfil add`, `replace` o `remove` **sin**
  `--yes`, Then el comando se niega con exit 2 y un mensaje que explica el ritual
  (mostrar la entrada, preguntar, y solo con el si registrar con `--yes`),
  exactamente como `approve-spec`. Ningun agente escribe el perfil por su cuenta.
- AC-7: Given una entrada identica a una existente, When se agrega, Then no se
  duplica: exit 0 con un mensaje que dice que ya estaba.
- AC-8: Given `perfil replace --old <substring> --texto "<nuevo>" --yes` o
  `perfil remove --old <substring> --yes`, Then el `<substring>` matchea por
  **subcadena unica**: si no matchea ninguna entrada, exit 2 diciendo que no se
  encontro; si matchea mas de una, exit 2 pidiendo un fragmento mas especifico y
  listando las que matchearon. Nunca se toca la entrada equivocada.
- AC-9: Given cualquier escritura al perfil, Then queda una linea en
  `progress/history.md` (`perfil add/replace/remove`) con el texto involucrado:
  el perfil se audita como cualquier otra transicion del flujo.

### Seguridad de un documento que se inyecta

- AC-10: Given una entrada que contiene lo que parece una credencial (patron de
  token, `password=`, `api_key`, clave privada) o caracteres Unicode invisibles
  (zero-width, bidi), When se intenta escribir, Then el comando **rechaza** con
  exit 2 explicando cual patron disparo. El perfil se versiona Y se inyecta en el
  prompt de cada agente: es superficie de ataque y de filtracion a la vez.

### Inyeccion en las superficies

- AC-11: Given un perfil con entradas, When corre el instalador (`sh` o `ps1`),
  Then cada superficie generada (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`)
  recibe las entradas dentro de un bloque delimitado por marcadores propios del
  arnes, y el reemplazo es **idempotente**: reinstalar no duplica el bloque ni
  toca nada fuera de los marcadores.
- AC-12: Given un proyecto **sin** `docs/perfil-usuario.md`, o con el archivo sin
  entradas, When corre el instalador, Then **no** se inyecta ningun bloque y las
  superficies quedan byte a byte como hoy.
- AC-13: Given una sesion en curso, When se corre `perfil add --yes`, Then el
  archivo cambia pero las superficies **no**: el bloque es un **snapshot
  congelado** que se refresca al reinstalar, y el comando lo dice en su salida
  para que nadie espere el efecto inmediato.

### `perfil sugerir`

- AC-14: Given `progress/history.md` con notas de `approve-spec`/`advance`/`close`
  y planes con bloques de decision, When corre `perfil sugerir`, Then se imprimen
  los registros de decision encontrados, **agrupados y con su evidencia** (feature
  y fecha), y **no se escribe absolutamente nada**.
- AC-15: Given esa salida, Then termina con el **contrato**: como destilar esos
  registros en una entrada durable (corta, en presente, sobre COMO trabajar, no
  sobre que paso), que NO poner (hechos de una feature puntual, datos personales,
  secretos) y el recordatorio de que solo el usuario aprueba.
- AC-16: Given un repo sin material (sin `history.md` o sin decisiones), When
  corre `perfil sugerir`, Then lo dice claramente y sale con 0.
- AC-17: Given cualquier subcomando de `perfil`, When el hub PostgreSQL esta
  caido, Then el comportamiento y los exit codes son identicos: el perfil es un
  archivo y no depende del hub.

### Integridad, docs y verificacion

- AC-18: Given un `docs/perfil-usuario.md` editado a mano que supera el limite o
  quedo mal formado, When corre `harness_check.sh`, Then lo reporta nombrando el
  archivo y el remedio; superar el limite **bloquea** (es lo que despues se
  inyecta en cada prompt). Sin el archivo, el bloque del check se omite entero.
- AC-19: Given `README.md`, `UPDATING.md` (+ espejo), `docs/architecture.md`
  (+ plantilla) y los tres roles, Then documentan el perfil, su limite, el ritual
  del `--yes`, la inyeccion como snapshot congelado y `perfil sugerir`; el lider
  lo usa para proponer y el reviewer verifica que ninguna entrada haya entrado sin
  el si del usuario.
- AC-20: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test` y `cargo clippy --all-targets -- -D warnings` estan verdes con
  tests del limite (incluido `replace`), del matcheo por subcadena (cero, uno,
  varios), del rechazo sin `--yes`, del escaneo de seguridad, del duplicado y de
  `sugerir` sin material; y `tests/setup_smoke.sh` verifica siembra, no-pisa,
  supervivencia al `--reset`, inyeccion idempotente y ausencia total de bloque
  cuando no hay perfil.

## Los datos que se tocan

- **disparador**: `perfil add|replace|remove --yes` (escritura, solo con el si del
  usuario) y el instalador (inyeccion en superficies).
- **interruptor**: la **existencia** de `docs/perfil-usuario.md` con entradas. Sin
  archivo o sin entradas, nada de esto ocurre.
- **candado**: el limite duro de 1500 caracteres, que hace fallar la escritura en
  vez de recortar; y los marcadores del bloque en las superficies, que hacen el
  reemplazo idempotente.
- **entidad**: `docs/perfil-usuario.md` — documento del USUARIO, versionado, que
  ningun reinstall pisa y ningun `--reset` borra.
- **lo que NO se toca**: el Memory Hub (el perfil es un archivo), las lecciones
  (`docs/lecciones/` guarda procedimiento; el perfil guarda preferencias) y el
  cuerpo de cualquier artefacto de feature.

## Pseudo-codigo (el acuerdo)

```
CUANDO alguien quiere agregar algo al perfil

  ¿viene con --yes?               -> si no, NOS NEGAMOS y explicamos el ritual
  ¿la entrada trae secretos o
   unicode invisible?             -> si si, RECHAZAMOS diciendo cual patron
  ¿ya existe identica?            -> si si, no hacemos nada
  ¿entra en el limite?            -> si no, FALLAMOS mostrando las entradas
                                     actuales y pidiendo consolidar AHORA

  ENTONCES la escribimos y lo registramos en la bitacora,
           con la restriccion de que las superficies NO cambian hasta el
           proximo reinstall (snapshot congelado).


CUANDO el instalador genera una superficie

  ¿existe el perfil y tiene entradas? -> si no, la superficie queda como hoy

  ENTONCES insertamos las entradas entre nuestros marcadores,
           reemplazando lo que hubiera entre ellos y sin tocar nada mas.
```

**Promesas:** nada entra sin tu si · nada se recorta en silencio · nada se
duplica · el perfil no se borra ni se pisa · sin perfil, cero cambios.

## No funcionales

- **SLOs**: `perfil show` y las escrituras son operaciones sobre un archivo de
  ~1500 caracteres: instantaneas y sin red. `perfil sugerir` recorre `history.md`
  y los planes, que son texto y son pocos.
- **Seguridad**: es el AC-10 y es lo mas delicado de la feature. El perfil se
  versiona (queda en git para siempre) **y** se inyecta en el prompt de cada
  agente (superficie de inyeccion). Por eso el escaneo es previo a escribir, y por
  eso ninguna entrada puede entrar sin que el usuario la haya visto.
- **Observabilidad**: exit codes estables (0 ok / 2 negativa, limite o rechazo);
  toda escritura deja linea en `progress/history.md`.

## Fuera de alcance

- `buscar` (#20), el curador (#21) y el mapa `journey` (#22).
- Que el arnes **destile** las entradas por su cuenta: `perfil sugerir` junta y
  agrupa la evidencia y emite el contrato; **el agente propone y el usuario
  decide**. Ninguna llamada a un modelo entra en esta feature (NO1 del PRD).
- `GROK.md` en la raiz: no se genera (Grok Build lee `AGENTS.md`/`CLAUDE.md`, y el
  instalador incluso archiva un `GROK.md` viejo si lo encuentra). Las superficies
  que reciben el bloque son las cuatro que el instalador genera.
- Perfiles multiples o por equipo: hay un solo `docs/perfil-usuario.md`.

## Observaciones (decisiones pendientes)

Todas decididas por Alan el 2026-08-16, en el mismo acto de aprobacion del spec.
No queda ninguna observacion abierta: el implementer puede avanzar sin preguntar.

- OBS-1: ¿En cuantas superficies se inyecta? — **DECIDIDO: en las cuatro reales**
  (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `LLM.md`). El backlog hablaba de cinco,
  pero `GROK.md` de la raiz **no existe**: el instalador genera cuatro y archiva
  cualquier `GROK.md` viejo, porque Grok Build lee `AGENTS.md`. Vinculante para
  AC-11.
- OBS-2: ¿Cuanto mide el limite? — **DECIDIDO: 1500 caracteres**, en linea con el
  `USER.md` de Hermes (1375). El limite chico no es tacaneria: cada caracter se
  paga en **todas** las sesiones de **todos** los backends, para siempre, y es lo
  que fuerza a consolidar hasta que solo sobrevivan las preferencias que de
  verdad se repiten. Vinculante para AC-3 y AC-4.
- OBS-3: ¿Que hace `sugerir` con lo ya incorporado? — **DECIDIDO: marcarlo**, para
  no volver a proponer lo mismo cada vez. Vinculante para AC-14.
- OBS-4: ¿El escaneo de seguridad bloquea o avisa? — **DECIDIDO: bloquea.** El
  perfil se versiona (queda en el historial de git para siempre) **y** se inyecta
  en el prompt de cada agente. Un secreto ahi es irreversible: rotarlo es la unica
  salida. Un falso positivo cuesta reescribir una frase; un falso negativo cuesta
  una credencial. Vinculante para AC-10.
- OBS-5: ¿De donde saca material `sugerir`? — **DECIDIDO: de los tres**
  (`progress/history.md`, los planes y los `## Observaciones` de los specs),
  porque el spec es donde la decision quedo mejor redactada. Vinculante para
  AC-14.
