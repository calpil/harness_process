# Spec - Feature #80: el autoaprendizaje no tiene ciclo de vida: lecciones sin tope, la misma leccion en diez cierres, perfil sin alimentar y consolidacion sin correr

Estado: approved
Aprobado: 2026-09-06T23:54:35Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado' a los ocho AC; umbrales 250/3/25/30; tope duro sin escape; partir a referencias/ y no fusionar

Feature: `feature_list.json` id 80 · kind `bug`.
Origen: diagnostico del 2026-09-06 sobre el programa "el arnes que aprende"
(`docs/analisis-hermes-agent.md`, seccion 3; features #17, #18, #19, #21, #28).

## La historia (antes -> despues)

Las seis piezas del loop de aprendizaje existen y corren. Lo que no existe es
el ciclo de vida de lo aprendido. Medido hoy en este repo:

| Senal | Medido 2026-09-06 |
| --- | --- |
| Leccion declarada en los ultimos 15 cierres `done` | 10 veces `criterios-de-cierre-que-se-pueden-fallar` |
| Tamano de esa leccion | 442 lineas, 17 secciones, 8 features de origen |
| Segunda mas grande | `promesas-estructurales-vs-disciplina`, 316 lineas |
| Archivos de apoyo (`<clase>/referencias/`, paso 3 de la guia) | 0 en toda la biblioteca |
| Lecciones con 0 usos, a 9-11 dias de `stale` | 4 de 8 activas |
| Decisiones registradas sin incorporar al perfil | 340 de 408 (`perfil sugerir`) |
| Ultima entrada del perfil | 2026-08-16 |
| Ultima consolidacion (`lecciones consolidar`) | 2026-08-18 (`bkp/lecciones/consolidar`) |
| Eventos de curador/consolidacion en `history.md` | 0 |

Hermes evita esto con un limite duro de memoria que OBLIGA a consolidar
(seccion 1.3 del analisis). El arnes lo copio para el perfil (`perfil::LIMITE`,
1500 caracteres, falla sin recortar) y no para las lecciones. El resultado es
una leccion-paraguas que absorbe todo lo que se aprende —cada cierre le agrega
una seccion `(feature #N)`, que es exactamente la forma "una-leccion-por-feature"
que la guia prohibe, solo que adentro de un archivo— y cuatro lecciones que
nadie lee y se van a enfriar solas. El gate `require_leccion` no lo ve: mide
que se DECLARE una leccion, no que se haya aprendido algo.

Despues: la leccion de clase tiene tope de lineas; el detalle por feature va a
`docs/lecciones/<clase>/referencias/<tema>.md`, como dice la guia desde la #17;
declarar la misma leccion K cierres seguidos exige decir por que no es otra
clase; y el cierre avisa cuando el perfil o la consolidacion llevan demasiado
sin correr. Todo con umbrales en `rules`, con defaults medidos.

## Hoy -> Como va a funcionar

- Hoy `close --leccion X` acepta X de cualquier tamano. Despues: si X supera
  `rules.leccion_max_lineas` (OBS-1), el gate RECHAZA (exit 2) y emite el
  contrato de particion: que secciones parecen de una sola feature (las que
  llevan `(feature #N)` en el titulo), a donde van (`<clase>/referencias/`) y
  el puntero de una linea que queda en la leccion. `leccion usar X` rechaza
  igual. Sin escape por flag (OBS-4).
- Hoy `close --leccion X` no sabe cuantas veces seguidas se declaro X. Despues:
  si los ultimos `rules.leccion_repeticiones` (OBS-2) cierres `done` de
  `history.md` declararon X, declararla otra vez exige `--leccion-motivo` que
  diga por que no es otra clase de trabajo; el mensaje nombra esos cierres.
  `ninguna` no cuenta ni corta la racha.
- Hoy nadie recuerda `perfil sugerir` ni `lecciones consolidar`. Despues: el
  cierre `done` emite por stderr —el mismo canal del contrato de lecciones, sin
  tocar stdout ni exit code— un aviso cuando hay mas de
  `rules.perfil_pendientes_max` decisiones sin incorporar (OBS-3), y otro cuando
  pasaron mas de `rules.consolidar_cada_dias` desde la ultima corrida de
  `lecciones consolidar` o `lecciones curar` (OBS-3), o nunca corrieron. Para
  saberlo, esos dos comandos registran su corrida en `history.md`
  (`lecciones consolidar informe|aplicar`, `lecciones curar informe|aplicar`).
- `lecciones status` muestra ademas: lineas/tope por leccion, decisiones sin
  incorporar al perfil y dias desde la ultima consolidacion.
- `harness_check.sh` avisa `[i]` por cada leccion sobre el tope (no bloquea: el
  bloqueo es del cierre, que es donde se decide).
- Los cuatro umbrales viven en `rules`; `<= 0` apaga cada uno, como
  `leccion_nudge_interval`.
- La biblioteca de ESTE repo queda dentro del tope (AC-7): las dos lecciones
  grandes se parten con la particion aprobada por el usuario (OBS-5), el
  curador y la consolidacion corren, y el perfil recibe las entradas que el
  usuario apruebe.

## Recorridos de usuario (priorizados)

- P1 (agente que cierra): `close --feature N --status done --leccion
  criterios-de-cierre-que-se-pueden-fallar` con la leccion en 442 lineas ->
  `[GATE]` exit 2 con el contrato de particion -> el agente mueve las secciones
  por feature a `referencias/`, deja punteros, reintenta -> cierra.
- P2 (agente que cierra por cuarta vez con la misma clase): `close ...
  --leccion X` -> `[GATE]` "X fue la leccion de los ultimos 3 cierres (#76, #77,
  #78); si esta vez tambien, deci por que no es otra clase: --leccion-motivo"
  -> el agente o bien nombra otra clase, o bien da el motivo -> cierra.
- P3 (usuario que mira el estado): `lecciones status` -> ve lineas/tope, 340
  decisiones sin incorporar, "ultima consolidacion: nunca registrada" -> corre
  `perfil sugerir` y `lecciones consolidar`.
- P4 (cierre normal): todo dentro de umbrales -> ningun aviso nuevo; stdout y
  exit iguales a hoy.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given `rules.leccion_max_lineas = L` y una leccion de clase con mas de L
  lineas (sin contar `<clase>/referencias/`), When `close --status done
  --leccion <esa>`, Then exit 2, el backlog no cambia, y el mensaje dice las
  lineas, el tope, las secciones cuyo titulo lleva `(feature #N)` y el comando
  para partirla. Con la leccion en L lineas o menos, cierra como hoy.
  Comando: `cd rust && cargo test --locked tope_de_lineas`
- AC-2: Given la misma leccion sobre el tope, When `leccion usar <esa>`, Then
  exit 2 con el mismo contrato y sin tocar `usos`; y `lecciones status` muestra
  `lineas/tope` por leccion, marcando las que lo superan.
  Comando: `cd rust && cargo test --locked usar_rechaza_sobre_el_tope`
- AC-3: Given `rules.leccion_repeticiones = K` y los ultimos K cierres `done` de
  `history.md` con `leccion=X` (los `ninguna` no cuentan ni cortan la racha),
  When `close --status done --leccion X` sin `--leccion-motivo`, Then exit 2 y
  el mensaje nombra esos K cierres; con `--leccion-motivo "<por que no es otra
  clase>"` cierra y el motivo queda en `history.md` y en la feature. Con K-1
  repeticiones, o con otra clase, cierra como hoy.
  Comando: `cd rust && cargo test --locked repeticiones`
- AC-4: Given mas de `rules.perfil_pendientes_max` decisiones sin incorporar
  (segun `perfil::recolectar`), When un cierre `done` termina, Then stderr lleva
  un aviso con la cuenta y `harness perfil sugerir`; stdout y exit code no
  cambian. Con `<= 0` no hay aviso.
  Comando: `cd rust && cargo test --locked aviso_de_perfil`
- AC-5: Given que `lecciones consolidar` y `lecciones curar` registran su
  corrida en `history.md`, When pasaron mas de `rules.consolidar_cada_dias`
  desde la ultima, o nunca hubo una, Then el cierre `done` avisa por stderr con
  los dias (o "nunca registrada") y el comando; `lecciones status` lo muestra.
  Comando: `cd rust && cargo test --locked aviso_de_consolidacion`
- AC-6: Given `UPDATING.md`, la guia `COMO-ESCRIBIR-UNA-LECCION.md`, los roles y
  `AGENTS.md`, When se leen, Then dicen el tope, la racha, los dos avisos, las
  cuatro claves de `rules` con sus defaults y el procedimiento de particion a
  `referencias/`. `harness_check.sh` avisa `[i]` por leccion sobre el tope.
  Comando: `bash tests/parity_check.sh`
  Comando: `bash tests/leccion_tope_check.sh`
- AC-7 (MANUAL): Given la biblioteca de este repo, When se cierra esta feature,
  Then ninguna leccion activa supera el tope (las dos grandes partidas con la
  particion que el usuario aprobo, OBS-5, con el detalle en `referencias/` y
  punteros en la leccion), `lecciones curar` y `lecciones consolidar` corrieron
  y quedaron registrados, y el perfil recibio SOLO las entradas que el usuario
  aprobo una por una (ritual de `perfil add --yes`).
- AC-8: Given la suite, When corre, Then `cargo test --locked`, clippy con
  `-D warnings`, `tests/setup_smoke.sh`, `tests/stop_hook_check.sh` y
  `tests/parity_check.sh` verdes; el smoke cubre el `[i]` del check.
  Comando: `bash tests/stop_hook_check.sh`

## Los datos que se tocan

- `feature_list.json` -> `rules`: `leccion_max_lineas`, `leccion_repeticiones`,
  `perfil_pendientes_max`, `consolidar_cada_dias` (todas opcionales, con default
  en el binario; `<= 0` apaga).
- `progress/history.md`: dos eventos nuevos (`lecciones consolidar <modo>`,
  `lecciones curar <modo>`); el `leccion_motivo` de una racha se registra como
  hoy el de `ninguna`.
- `docs/lecciones/<clase>/referencias/<tema>.md`: archivos nuevos al partir
  (contenido movido, no reescrito). `docs/lecciones/<clase>.md` pierde
  secciones y gana punteros.
- `docs/perfil-usuario.md`: solo con el si del usuario, entrada por entrada.

## Pseudo-codigo (el acuerdo)

```
// lecciones.rs
pub struct Politica { max_lineas: i64, repeticiones: i64, perfil_pendientes: i64, consolidar_dias: i64 }
impl Politica { fn from_rules(data) -> Politica }   // mismo patron que Umbrales::from_rules

fn lineas_de_clase(leccion) -> usize                // el archivo de la clase, no referencias/
fn secciones_por_feature(leccion) -> Vec<(titulo, feature)>   // "## ... (feature #N)"
fn contrato_de_particion(leccion, politica) -> String

fn racha(history, clase, k) -> Option<Vec<cierre>>  // ultimos k `close ... status=done leccion=`; ninguna se salta

// gate(): despues de "la clase existe":
//   si lineas > max_lineas (> 0)         -> Err(exit 2, contrato_de_particion)
//   si racha(k) es Some y motivo es None -> Err(exit 2, nombra los k cierres)
//   motivo presente                      -> Declaracion { clase, motivo: Some }

// close.rs, FASE 3, junto al contrato de cierre (stderr, sin tocar stdout/exit):
//   avisos::perfil(paths, politica)      // perfil::recolectar().filter(!ya_incorporado).count()
//   avisos::consolidacion(paths, politica) // ultimo evento `lecciones (consolidar|curar)` en history

// commands/leccion.rs: usar() aplica el tope; status() imprime lineas/tope, pendientes y dias
// commands/leccion.rs: curar() y consolidar() -> progress::log("lecciones curar informe|aplicar")
```

## No funcionales

- El cierre no cambia stdout ni exit code por los avisos: son stderr y
  best-effort, como el contrato de lecciones (#18, AC-8 de aquella).
- Ninguna medicion nueva lee el cuerpo de las lecciones mas de una vez por
  cierre; `perfil::recolectar` ya corre en `sugerir` y su costo es el mismo.
- Los umbrales se leen del backlog en cada invocacion (sin cache): cambiarlos
  en `rules` tiene efecto inmediato, como `leccion_nudge_interval`.

## Fuera de alcance

- Refrescar el espejo del backlog al cerrar: es la #79.
- Partir lecciones automaticamente: el arnes dice QUE partir y a donde; mover
  el texto es del agente, con el usuario mirando.
- Escribir en el perfil sin el si del usuario: sigue prohibido (#19).

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA 2026-09-06: 250): `leccion_max_lineas` default. Propuesta: 250 (hoy deja afuera a
  `criterios-de-cierre` 442 y `promesas-estructurales` 316; las otras seis
  quedan adentro con margen; la guia tiene 171).
- OBS-2 (DECIDIDA 2026-09-06: 3): `leccion_repeticiones` default. Propuesta: 3 (hoy la racha real es
  10; con 3, el cuarto cierre seguido pide motivo).
- OBS-3 (DECIDIDA 2026-09-06: 25 y 30): `perfil_pendientes_max` default 25 (hoy 340) y `consolidar_cada_dias`
  default 30 (el mismo umbral que `stale`; hoy 19 dias y sin registro).
- OBS-4 (DECIDIDA 2026-09-06: duro, sin escape): el tope es DURO, sin `--leccion-motivo` como escape. Motivo: el del
  perfil es duro y funciona; un escape por flag se vuelve el default (es lo que
  paso con `ninguna`, que por eso exige motivo). Alternativa: aceptar
  `--leccion-motivo` tambien para el tope.
- OBS-5 (DECIDIDA 2026-09-06: partir a referencias/, sin clases nuevas): particion de las dos lecciones grandes. Propuesta:
  `criterios-de-cierre-que-se-pueden-fallar` conserva "Cuando aplica",
  "Procedimiento", "Cuando el criterio ya es un comando", "Pitfalls" y
  "Verificacion" mas un indice de reglas (una linea por hallazgo); las nueve
  secciones con `(feature #N)` o con un caso unico (el PIPE, la herramienta
  externa, el oraculo copiado, el recorrido P1, el nombre del test, la regla
  ancha, la pieza de mas, el arnes que miente, el rojo que fallo antes) van a
  `criterios-de-cierre-que-se-pueden-fallar/referencias/<tema>.md`.
  `promesas-estructurales-vs-disciplina` conserva "Cuando aplica",
  "Procedimiento", "Pitfalls", "Verificacion" y la tesis de "La otra cara"; "El
  ORDEN tambien es estructura", "aplicado a ARREGLAR un bug (feature #44)" y "La
  variante del DESTINO IMPLICITO" van a `referencias/`. Alternativa: partir en
  clases nuevas (p. ej. `prueba-del-rojo`), que la guia desaconseja salvo que
  ninguna clase cubra el tema.
- OBS-6: la rama de integracion se pregunta antes de `close --status done --to`.
- OBS-7 (DECIDIDA 2026-09-06: NO fusionar; quitar el trigger compartido): `lecciones consolidar` (corrido hoy en modo informe, backend `claude`)
  propone 4 fusiones con confianza 1.00, todas entre lecciones que se declaran
  `relacionadas` entre si: `criterios-de-cierre` + `probar-contra-datos-reales`,
  `criterios-de-cierre` + `promesas-estructurales`, `probar-contra-datos-reales`
  + `promesas-estructurales`, `probar-contra-datos-reales` +
  `reglas-que-se-aplican-a-si-mismas`. Fusionarlas va en contra del tope: el
  paraguas naceria con ~1000 lineas. Propuesta: NO fusionar —son tres clases
  distintas (como se disena un criterio / contra que datos se prueba / que
  garantiza la estructura) que comparten el trigger "falso verde"— y en su
  lugar quitar ese trigger compartido de dos de ellas para que dejen de
  solaparse, dejando la consolidacion registrada como corrida. Alternativa:
  fusionar bajo `verificacion-que-no-verifica` con el detalle en `referencias/`.
  Nota para el curador: que 4 de 4 candidatos tengan confianza 1.00 por
  declararse mutuamente sugiere que `relacionadas` pesa de mas (#41); no se
  toca en esta feature, queda anotado.
