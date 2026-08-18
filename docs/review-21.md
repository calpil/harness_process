# Veredicto del reviewer - Feature #21: curador_de_lecciones

Spec: `docs/spec-feature-21-curador-de-lecciones.md` (`Estado: approved`, sello
`2026-08-17T04:08:36Z por USUARIO (confirmacion explicita)`, 20 AC)
Plan: `docs/plan-feature-21-curador-de-lecciones.md` (D1-D10)
Evidencia: `docs/impl-21.md`
PRD de origen: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 5)

## Veredicto global: `approved`

Los 20 AC cubiertos y **los cuatro criterios de cierre corridos de punta a punta**
con fechas falsas, no solo con tests unitarios.

## Trazabilidad de la aprobacion (Articulo 2)

Sello con las cinco decisiones OBS-1..OBS-5, linea `approve-spec feature #21` en
`progress/history.md`, `check-spec` y `check-plan` limpios.

**Tres de las cinco observaciones fueron correcciones al backlog**, y vale
registrar el patron: el backlog de este programa se escribio en la #16-#17 por
analogia con Hermes, y ya lleva **cinco correcciones acumuladas** al chocar con
la realidad de este arnes (`GROK.md` inexistente en la #19, el hub en la #20, y
la consolidacion / `adoptar` / `.archivo/` en esta). No es una falla del backlog:
es lo que pasa cuando un plan se escribe antes de tocar el codigo. Lo que importa
es que cada correccion quedo **decidida por el usuario y escrita**, no aplicada en
silencio.

## Estado por AC

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | `status` con dias hasta la proxima transicion y el resumen de candidatas de hoy |
| AC-2 | cubierto | Sin biblioteca: los tres subcomandos informan y salen 0 |
| AC-3 | cubierto | `--json` con los ocho campos |
| AC-4 | cubierto | Umbral **exacto**: 29 no, 30 si |
| AC-5 | cubierto | Umbral **exacto**: 89 no, 90 si; y archivar MUEVE (cuerpo intacto en destino) |
| AC-6 | cubierto | Nunca usada cuenta desde `ultima_actualizacion` |
| AC-7 | cubierto | Pin con 200+ dias sigue `activa`, unit e integracion |
| AC-8 | cubierto | `stale` usada ayer vuelve a `activa`; archivada NO vuelve sola |
| AC-9 | cubierto | Contenido **y mtime** identicos tras el informe; `archivo/` ni se crea |
| AC-10 | cubierto | Backup contiene el original byte a byte |
| AC-11 | cubierto | Restaura exacto **y** deja backup `pre-rollback` |
| AC-12 | cubierto | `rollback --list` con id y motivo |
| AC-13 | cubierto | `pin`/`unpin` no tocan cuerpo ni telemetria |
| AC-14 | cubierto | Round-trip con contenido identico; los dos errores con exit 2 |
| AC-15 | cubierto | Sugerencias por similitud, reusando `parecidas` |
| AC-16 | cubierto | `REPORT.md` con transiciones, dias, pins, backup y "Nada se borro" |
| AC-17 | cubierto | Sin cambios: ni backup ni reporte |
| AC-18 | cubierto | Archivada visible en `buscar` (score 60) por debajo de la activa (130/100) |
| AC-19 | cubierto | Fuera del catalogo por default, visible con `--archivadas`; check con `maxdepth 2` |
| AC-20 | cubierto | 213 unit + 83 integracion, clippy limpio, smoke exit 0, check limpio |

## Constitution

| Articulo | Verificacion |
| --- | --- |
| 1 - Calidad y tests | 213 unit + 83 integracion (29 nuevos), clippy `-D warnings`, smoke exit 0 |
| 2 - Spec aprobado | Sello + history + gates verdes |
| 3 - Trazabilidad AC-n | Cada D cita sus AC; evidencia y veredicto por AC |
| 4 - Seguridad y observabilidad | **Es el eje de la feature**: nunca borra, backup antes de cada mutacion, rollback reversible, y `--aplicar` explicito. El peor error posible es recuperable |
| 5 - Decisiones del usuario | Las 5 OBS decididas antes de implementar; 3 corrigen el backlog |
| 6 - Reglas puente | Sin dependencias nuevas; `templates/` espejado; sin modelo (el ciclo es aritmetica de fechas) |

## Lo que el reviewer destaca

**La separacion `planificar()` / `aplicar()` es lo que hace cierta la promesa.**
El AC-9 ("el informe no toca nada") no se sostiene por disciplina del que
implementa: se sostiene porque la funcion que calcula el plan **solo lee**, y la
que muta esta detras de un flag. Un test que compara mtimes lo verifica, pero la
estructura ya lo garantizaba. Esa es la diferencia entre una promesa y una
propiedad.

**Los umbrales se probaron en sus bordes exactos** (29/30, 89/90) en vez de
"alrededor de 30". Un off-by-one en un umbral de 30 dias no se nota en produccion
hasta un mes despues, y para entonces ya archivo algo.

## Riesgos que quedan abiertos

1. **Los umbrales nunca corrieron contra el tiempo real.** Todo se probo con
   fechas falsas: la logica esta cubierta, el uso no. La primera pasada de verdad
   llega en ~30 dias y vale mirarla — es el momento en que se sabra si 30/90 son
   los numeros correctos para lecciones de proceso (que envejecen mas lento que
   una skill de herramienta).
2. **Sin politica de retencion de backups.** Cada pasada mutante copia el arbol
   entero. Con decenas de lecciones chicas es irrelevante; con cientos, no. Es
   deuda consciente y declarada, no un olvido.
3. **`setup_smoke.ps1` sin ejecutar**, igual que en #17-#20.
4. **`lecciones curar --aplicar` es facil de correr sin pensar.** El comando
   avisa, respalda y es reversible, y el rol reviewer ahora dice explicitamente
   que no se corre sin avisarle al usuario. Pero la barrera final es social, no
   tecnica.

## Nota sobre la declaracion de cierre

La feature deja `docs/lecciones/promesas-estructurales-vs-disciplina.md`: cuando
una garantia del spec ("no toca nada", "nunca borra") se puede volver una
**propiedad del codigo** en vez de una regla que alguien tiene que recordar. Sale
de la decision concreta de esta feature (separar la funcion que lee de la que
muta) y aplica a cualquier feature futura que prometa un invariante.

Se corrio `leccion usar criterios-de-cierre-que-se-pueden-fallar`: esa leccion
—escrita en la #20— fue la que hizo que los criterios de cierre de este plan se
escribieran como corridas concretas en vez de adjetivos, y los cuatro se pudieron
ejecutar. Segunda vez que una leccion cambia una feature posterior.
