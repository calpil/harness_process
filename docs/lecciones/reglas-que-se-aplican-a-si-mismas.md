---
nombre: reglas-que-se-aplican-a-si-mismas
descripcion: La primera aplicacion de una regla decide si va a existir o no.
triggers: [convencion, regla, politica, escalera, lint, estandar, excepcion, precedente, guia]
relacionadas: [criterios-de-cierre-que-se-pueden-fallar, promesas-estructurales-vs-disciplina, probar-contra-datos-reales]
origen: [24]
usos: 1
ultimo_uso: 2026-08-17
ultima_actualizacion: 2026-08-17
estado: activa
---

## Cuando aplica

Cuando la feature entrega una **regla** en vez de una capacidad: una convencion,
una politica de tests, un estandar de nombres, una escalera de decision, un lint.
Todo lo que se escribe para que alguien **rechace** algo despues.

Sintoma de que va a fracasar: la feature se puede cerrar entera sin que la regla
haya rechazado nada todavia. Ahi la regla no esta viva, esta escrita — y son dos
estados muy distintos.

## Procedimiento

1. **Aplicale la regla a la feature que la introduce**, antes de escribirla del
   todo. Si la regla es una escalera de decision, pasa el diseno de esta misma
   feature por la escalera y publica el resultado en el plan. Si la feature que
   introduce la regla no la cumple, la regla nace refutada y te enteras ahora en
   vez de en la revision.
2. **Busca la primera violacion existente y pagala en la misma feature.** No la
   dejes para "un backlog de limpieza": una regla que se estrena con una deuda
   sin pagar es una recomendacion.

   **Pero hay deudas que no se pueden pagar honestamente, y ahi lo que se paga
   es el CORTE** (feature #64). `require_review` se estreno con 15 cierres sin
   review (#38-43, #53-55, #57, #59-63). Reconstruirlos habria sido escribir 15
   veredictos sobre codigo ya integrado y funcionando: un review es "intentar
   ROMPER" (`roles/reviewer.md:6`), y lo que se escribe despues de que todo
   anduvo no rompe nada — llena el casillero, que es peor que dejarlo vacio
   porque despues se cita como si fuera revision. Lo que se hizo en su lugar:
   documentar el corte con los ids, las dos fechas (ultimo con review #46
   2026-08-22, primero sin #57 2026-08-26) y el argumento, en `UPDATING.md`.
   El corte pretende separar: **¿pagar la deuda produce la senal que la regla
   busca, o solo el artefacto?** Si solo el artefacto, la deuda se declara, no
   se paga. Y se declara con la lista completa: un corte sin los ids es una
   amnistia.
3. **No declares excepcion en la primera aplicacion.** Es la decision mas cara de
   toda la feature. La primera excepcion no exime un caso: crea el precedente que
   van a citar todas las siguientes. Si el caso realmente merece excepcion, lo
   que hay que arreglar es el enunciado de la regla, no agregarle un permiso.
4. **Si la regla admite excepciones, escribi el CORTE, no la lista.** Una lista
   de casos permitidos se estira; una pregunta que separa se aplica sola.
   Ejemplo real: "prohibido leer el fuente en un test, salvo que el archivo sea
   dato de entrada del codigo bajo prueba", con el corte *¿el test seguiria
   valiendo si la implementacion se reescribiera entera?*
5. **Pone chequeo automatico solo donde se pueda, y deci cual NO tiene.** Media
   regla automatizada y la otra media dicha como disciplina es honesto. Fingir
   que el script cubre todo es lo que hace que nadie revise el resto.
6. **Corre la prueba del rojo sobre el chequeo** antes de cerrar: sembra una
   violacion, confirma que la reporta, borrala. Ver
   [[criterios-de-cierre-que-se-pueden-fallar]].

## Pitfalls

- **La regla que solo mira hacia adelante.** Si aplica de aca en mas y nada del
  pasado se revisa, no se entera nadie de que existe. Auditar lo que ya hay es la
  mitad del trabajo, y ademas es donde aparecen los casos que el enunciado no
  contemplaba.
- **Auditar y reportar solo las violaciones.** Un informe que lista tres
  problemas no deja saber si se miraron 3 casos o 300. Escribi tambien los que
  revisaste y quedaron **correctos**, con el motivo: es lo unico que hace
  auditable la auditoria.
- **Creer que la regla nueva no toca lo que celebraste ayer.** En la #24 la regla
  del detector-de-cambios condeno un test que la #23 habia presentado como su
  mejor idea ("la compatibilidad es un test y no una promesa"). Era verdad Y era
  un detector-de-cambios: se rompio en la feature siguiente sin que nada
  estuviera mal. Si al aplicar la regla no incomoda nada tuyo, probablemente la
  escribiste para que no incomodara.
- **Confundir "el reviewer lo verifica" con estar cubierto.** Si el rol no dice
  el verbo (**rechaza**, no "revisa"), en la practica se anota como observacion y
  se aprueba igual.
- **Ejemplos inventados.** Una escalera con ejemplos hipoteticos es una lista que
  nadie sabe aplicar. Cada peldano con un caso real del repo, citando su feature,
  se usa; sin eso, se lee una vez.

## Cuando la regla te rechaza a vos, ya funciono

En la #64 la prediccion de esta leccion se cumplio dos veces en la misma tarde, y
las dos veces el rechazo fue la evidencia de que la regla servia:

1. Al encender `require_review` en el molde, el E2E de PRD de
   `tests/setup_smoke.sh` **dejo de pasar**: cerraba `done` sin review. Era
   correcto que lo rechazara. Se pago haciendo que el test pase por el flujo
   completo, y de paso quedo ejercitado el review de punta a punta sobre una
   instalacion real en vez de sobre un fixture.
2. El **reviewer adversarial de la propia feature la rechazo**
   (`changes_requested`) con ocho bloqueantes, incluido el mas caro: el spec
   afirmaba que el sello era "imposible de fabricar escribiendo el archivo a
   mano" y **no era cierto** — la linea se puede tipear. La feature se llamaba
   "el arnes no promete enforcement que no hace" y su propio spec prometia de
   mas. El arreglo no fue bajar la promesa: fue subir el enforcement (el gate
   re-verifica la cobertura por AC, que es lo que no se fabrica en cinco
   segundos) Y corregir la afirmacion.

Corolario para la proxima: **la regla se prueba contra la feature que la
introduce, con un revisor que no sea el que la escribio.** Si el autor firma su
propia regla, lo unico que se probo es que sabe escribirla.

## Verificacion

```bash
# 1. ¿La feature que introduce la regla la cumple? Tiene que estar en el plan.
grep -n "Peldano elegido:\|<la frase que la regla exige>" docs/plan-feature-<id>-*.md

# 2. ¿Se pago alguna deuda real, o la regla se estrena limpia por casualidad?
grep -n "VIOLACION\|Auditoria" docs/impl-<id>.md

# 3. La prueba del rojo sobre el chequeo, si lo hay
bash tests/<chequeo>.sh detecta
```

Si al terminar la feature la regla todavia no rechazo nada —ni un test, ni un
diseno, ni una linea— no la agregaste: la anunciaste.
