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
 sh harness_cli add --name <slug> --service <servicio> --acceptance "<criterio>"
y al arrancarla (`start`) genera su spec con AC-n. Cada hito es un PRD anidado:
si uno no entra en una historia sola, partilo en dos.>

| # | Hito | Slug de feature | Objetivo que cumple | Criterio de aceptacion (resumen) | Estado |
| --- | --- | --- | --- | --- | --- |
| 1 | <hito> | <slug_snake_case> | <O1> | <que tiene que ser cierto> | pendiente |

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
