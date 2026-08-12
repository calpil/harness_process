# Como escribir un PRD

Guia del metodo que usa este arnes para especificar. Obtene claridad, control y
velocidad siguiendo estos pasos ANTES de promptear una IA.

> Plantilla del arnes: se siembra en `docs/prd/` y se refresca reinstalando (o
> con `--force`). Tus documentos son `PRD-master.md` y `SDD-master.md`; este
> archivo es el manual.

## 1. Que es un PRD

Un documento que cuenta y «raya la cancha» de como la IA deberia construir lo
que queres.

| CONTIENE | NUNCA CONTIENE |
| --- | --- |
| la historia (antes y despues) | codigo final |
| hoy -> mañana | la implementacion exacta |
| tablas y entidades a tocar | pantallas terminadas |
| pseudo-codigo | configuracion |
| explicacion de los cambios | |

**La unica regla dura:** el PRD fija la **estructura** — la historia, que
entidades se tocan y como cambian — en pseudo-codigo y explicaciones, nunca en
codigo final. Eso se escribe despues, en otra parte.

## 2. Todo empieza con una historia

Tenes que poder contarla en palabras, sin tecnicismos.

**ASI NO**

```
escuchar el cambio de estado
agendar una tarea de llamada
disparar el agente de voz
```

**ASI SI**

> «Marta cerro su compra un viernes a las 6 de la tarde. **Nadie la llamo.** El
> lunes le llego la misma plantilla de siempre — y esa confianza recien ganada
> se enfrio justo cuando mas cerca estaba de recomendarnos.»

La historia dice **quien es el usuario**, **como lo usa**, **cual es el dolor** y
**cual es la experiencia que quiere vivir**. Todo lo demas en el documento existe
para hacer esa historia realidad.

Si la historia no convence, el resto no importa. Escribila primero.

## 3. El tamano lo decide el cambio

| Tipo de cambio | Tamano | Que es |
| --- | --- | --- |
| Un ajuste | 1 pagina | un arreglo o una mejora pequena |
| Una funcionalidad | 3-8 paginas | algo nuevo que el producto hace |
| Una funcionalidad grande | 10+ paginas (a veces 20 o 30) | una pieza nueva completa del producto |
| Un producto nuevo | varios PRDs anidados | tan grande que un solo documento no alcanza |

Un PRD puede contener otros PRDs, y esos a su vez mas. Cada uno cuenta su propia
historia; ninguno carga con todo el peso.

```
PRD  el producto nuevo completo
 |-- PRD  una parte del todo — con su propia historia, sus datos y su plan
 `-- PRD  otra parte, que a su vez se divide en piezas mas chicas:
      |-- PRD
      `-- PRD  ...
```

## 4. Anatomia: las partes de un PRD

Las secciones, en el orden en que aparecen. El ejemplo — una llamada de
agradecimiento al cerrar la venta — se repite en todas para que se vea el hilo.

### 0 · Encabezado

```
PRD — Llamada de agradecimiento post-venta («el gracias»)
Estado: Borrador · Duenno: A. Rivera · Creado: 2026-08-10
Alcance: llamada de voz al cerrar la venta — NO toca correos ni recordatorios
```

Los datos basicos primero: estado, duenno, y **que queda fuera**.

### 1 · Resumen

```
Hoy:     se cierra una venta y no pasa nada. Alguien tiene que acordarse.
Despues: al marcar la venta cerrada, se agenda una llamada que agradece.
```

Antes / despues en dos lineas. El dibujo mas barato que existe.

### 2 · La historia

El corazon del documento: el antes y el despues, con nombre y momento (ver §2).

### 3 · Objetivos / No-objetivos

```
O1   toda venta cerrada recibe su llamada en menos de 5 segundos
O2   reversible — interruptor por cliente, se apaga en 1 clic
NO1  no vende ni reagenda nada — solo agradece
```

Con nombre y apellido: las secciones siguientes los citan («cumple O2»). Los
no-objetivos frenan el «ya que estamos...».

### 4 · Como funciona hoy -> como va a funcionar

```
HOY                        DESPUES
venta cerrada -> (nada)    venta cerrada -> agenda la llamada
                                |__ Retell llama -> el agente agradece
                                          |__ el pipeline guarda el resultado
```

El flujo, dibujado dos veces. Reusa lo que ya existe.

### 5 · Los datos

```
disparador    el lead pasa a un estado marcado como «venta cerrada»
por cliente   llamadas_gracias: 'apagado' | 'prueba' | 'activo'   <- interruptor
por lead      agradecido_en: fecha    <- candado: una sola llamada por venta
```

El plano de los datos: que dispara, el interruptor por cliente, y el candado que
evita hacerlo dos veces.

### 6 · Pseudo-codigo — el acuerdo

```
CUANDO se cierra una venta

  ¿el cliente activo «las gracias»?  -> si no, no hacemos nada
  ¿ya lo llamamos por esta venta?    -> si si, no hacemos nada
  ¿tenemos su numero?                -> si no, no hacemos nada

  ENTONCES lo llamamos en 5 segundos, con un guion
           que agradece la confianza — solo en horario habil.
```

**Promesas:** una sola llamada por venta · nunca fuera de horario · si no
contesta, no insiste.

La receta, en palabras: que la dispara, que la frena y que promete — sin una sola
linea de codigo.

## 5. Como se aplica en este arnes

El metodo esta anidado en las plantillas que ya usas:

| Nivel | Archivo | Que cuenta |
| --- | --- | --- |
| Producto | `docs/prd/PRD-master.md` | la historia del producto, objetivos O-n, los datos y el acuerdo a nivel producto, y los hitos |
| Como tecnico | `docs/prd/SDD-master.md` | arquitectura objetivo y decisiones que ninguna feature re-litiga |
| Cambio | `docs/spec-feature-<id>-<slug>.md` | **el PRD anidado de cada cambio**: su historia, su hoy->despues, sus datos, su pseudo-codigo y sus AC-n |

La cadena completa:

```
docs/prd/PRD-master.md          (hitos priorizados)
        |  sh harness_cli add --name <slug> --service <svc> --acceptance "<criterio>"
        v
feature_list.json               (backlog ejecutable)
        |  sh harness_cli start --feature <id>
        v
docs/spec-feature-<id>-<slug>.md   <- el PRD del cambio, ya con las secciones del metodo
        |  lo aprueba el USUARIO (Estado: approved)
        v
implementacion -> docs/impl-<id>.md -> docs/review-<id>.md
```

El pseudo-codigo vinculante de cada cambio vive en su spec, no en el maestro:
asi el PRD del producto no se desactualiza feature tras feature.

## 6. Ahora te toca

Apunta tu IA a estos archivos y empeza a armar tus PRDs. Despues, en vez de
dictarle codigo, solo le decis:

```
> implementa al 100% @docs/spec-feature-<id>-<slug>.md
```

...y esperas un resultado buenisimo.
