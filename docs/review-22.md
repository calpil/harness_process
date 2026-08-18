# Veredicto del reviewer - Feature #22: mapa_de_aprendizaje

Spec: `docs/spec-feature-22-mapa-de-aprendizaje.md` (`Estado: approved`, sello
`2026-08-17T04:38:04Z por USUARIO (confirmacion explicita)`, 18 AC)
Plan: `docs/plan-feature-22-mapa-de-aprendizaje.md` (D1-D8)
Evidencia: `docs/impl-22.md`
PRD de origen: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 6, **ultimo**)

## Veredicto global: `approved`

Los 18 AC cubiertos, y los criterios de cierre hicieron su trabajo: **encontraron
tres bugs que los tests con fixtures no veian**.

## Trazabilidad de la aprobacion (Articulo 2)

Sello con las cinco decisiones OBS-1..OBS-5, linea `approve-spec feature #22` en
`progress/history.md`, `check-spec` y `check-plan` limpios.

Dos de las cinco observaciones **redujeron el alcance** respecto del backlog
(sin `delete`, sin `edit`), y eso quedo escrito en el spec y en el plan para que
nadie lea el backlog dentro de seis meses y crea que falto algo.

## Estado por AC

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | Linea de tiempo cronologica con los tres tipos de nodo |
| AC-2 | cubierto | La #17 muestra sus DOS lecciones (declarada + origen) y la declarada **una sola vez** |
| AC-3 | cubierto | Usos y ultimo uso en cada nodo de leccion |
| AC-4 | cubierto | La entrada del perfil cuelga de la feature mas reciente que cita, una sola vez |
| AC-5 | cubierto | `Tipo::LeccionArchivada` con etiqueta propia |
| AC-6 | cubierto | Enlace roto desde una leccion, con el id inexistente |
| AC-7 | cubierto | Enlace roto desde el perfil |
| AC-8 | cubierto | Cierre sin leccion, **acotado a la era de las lecciones** (ver abajo) |
| AC-9 | cubierto | Leccion huerfana |
| AC-10 | cubierto | Sin huecos se dice explicitamente, en vez de callar |
| AC-11 | cubierto | Los dos modulos no importan nada que escriba; comprobacion negativa en el repo real: 0 archivos tocados |
| AC-12 | cubierto | `Motivo::remedio()` exhaustivo: un tipo de hueco nuevo no puede quedar sin comando |
| AC-13 | cubierto | `--json` con nodos, enlaces (las tres clases) y huecos con su remedio |
| AC-14 | cubierto | Mismo stdout con el hub sano y con el hub muerto |
| AC-15 | cubierto | Repo fresco: mensaje + exit 0 |
| AC-16 | cubierto | Archivo ilegible: se saltea y se cuenta como hueco |
| AC-17 | cubierto | README, UPDATING (+ espejo), architecture y ambas superficies |
| AC-18 | cubierto | 231 unit + 91 integracion, clippy limpio, smoke exit 0, check limpio |

## Los criterios de cierre encontraron lo que los tests no

Es el punto que el reviewer quiere dejar registrado. Los tests con corpus sembrado
pasaron **desde la primera corrida**. Los tres bugs salieron de exigir el mapa
sobre **este** repo y verificar a mano:

| # | Bug | Como se veia |
| --- | --- | --- |
| 1 | La leccion declarada salia duplicada | `docs-generados` dos veces bajo la #17 |
| 2 | El perfil colgaba de todas las features citadas | la misma entrada repetida en #14 y #16 |
| 3 | 16 huecos, ninguno corregible | features #1-#16, cerradas antes de que existieran las lecciones |

El tercero tuvo dos capas: acotar a "despues de que el proyecto empezo a declarar
lecciones" bajo de 16 a 2, y los 2 restantes eran features cerradas el **mismo
dia** que la #17 pero horas antes — comparaba fechas truncadas en vez de
timestamps. Recien ahi llego a 0, y la verificacion manual confirmo que 0 es el
numero correcto.

**Ningun fixture razonable habria producido esos tres casos.** Salen de tener
historia real: una feature que declara una leccion y pare otra, una preferencia
que cita dos features, y un repo con dieciseis features anteriores a la
maquinaria. Es la segunda vez (tras el ADR de la #20) que el criterio "corre esto
sobre el repo real" atrapa algo que la suite verde no veia.

## Constitution

| Articulo | Verificacion |
| --- | --- |
| 1 - Calidad y tests | 231 unit + 91 integracion (26 nuevos), clippy `-D warnings`, smoke exit 0 |
| 2 - Spec aprobado | Sello + history + gates verdes |
| 3 - Trazabilidad AC-n | Cada D cita sus AC; evidencia y veredicto por AC |
| 4 - Seguridad y observabilidad | Solo lectura; los comandos sugeridos se imprimen como TEXTO, nunca se ejecutan; sin interpolacion de entrada del usuario |
| 5 - Decisiones del usuario | Las 5 OBS decididas antes de implementar; 2 reducen alcance y quedo escrito |
| 6 - Reglas puente | Sin dependencias nuevas; `templates/` espejado; sin modelo y sin hub |

## Lo que el reviewer destaca

**No agregar `journey delete` fue la decision correcta, y por una razon que el
propio repo escribio la feature anterior.** La #21 dejo `promesas-estructurales-vs-disciplina`:
un invariante que depende de acordarse no es un invariante. Un `journey delete`
seria una segunda puerta a dos garantias que hoy son estructurales — el "nunca
borra" del curador y el gate del `--yes` del perfil — y mantener dos puertas en
sincronia es precisamente lo que esa leccion advierte que falla.

Que una leccion escrita ayer haya cambiado el alcance de la feature de hoy es,
ademas, la mejor evidencia de que el programa de aprendizaje funciona.

## Riesgos que quedan abiertos

1. **El mapa no tiene tope.** Con 21 features entra en pantalla; con 200 no.
   Sin evidencia de que haga falta todavia; `--desde <fecha>` es la salida.
2. **La regla de la "prehistoria" tiene un supuesto**: que el proyecto empezo a
   usar lecciones en la primera que declaro una. Declarar una leccion
   retroactivamente en una feature vieja correria la ventana hacia atras y
   reaparecerian huecos viejos. Caso raro y visible, no silencioso.
3. **`setup_smoke.ps1` sin ejecutar**, igual que en #17-#21. Cinco features con la
   misma brecha declarada: vale decidir si se instala PowerShell o si se acepta
   como limite permanente del entorno.

## Nota sobre la declaracion de cierre

La feature deja `docs/lecciones/probar-contra-datos-reales.md`: por que una suite
verde sobre fixtures no dice nada sobre la calibracion, y que casos solo existen
en un repo con historia. Sale de tres bugs concretos de esta sesion y aplica a
toda feature que produzca un ranking, un umbral o un reporte.

Se corrio `leccion usar promesas-estructurales-vs-disciplina`: esa leccion —de la
#21— es la que decidio que `journey` no tuviera `delete`. **Tercera vez que una
leccion cambia una feature posterior.**
