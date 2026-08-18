# Veredicto del reviewer - Feature #20: buscar_en_el_historial

Spec: `docs/spec-feature-20-buscar-en-el-historial.md` (`Estado: approved`, sello
`2026-08-17T03:44:47Z por USUARIO (confirmacion explicita)`, 19 AC)
Plan: `docs/plan-feature-20-buscar-en-el-historial.md` (D1-D9)
Evidencia: `docs/impl-20.md`
PRD de origen: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 4)

## Veredicto global: `approved`

Los 19 AC cubiertos con evidencia ejecutada, y los **tres criterios de cierre que
el plan agrego** —la consulta real, el SLO medido y la garantia de solo lectura—
verificados con numeros, no con afirmaciones.

## Trazabilidad de la aprobacion (Articulo 2)

Sello de `approve-spec` con las cinco decisiones OBS-1..OBS-5, linea
`approve-spec feature #20` en `progress/history.md`, y `check-spec` / `check-plan`
limpios.

El spec corrigio de oficio un segundo punto del backlog (tras el `GROK.md` de la
#19): decia que `buscar` usara `to_tsvector` sobre el hub. OBS-1 lo saco con tres
razones escritas, la principal siendo que **ese camino se habria entregado sin
poder ejecutarse ni una vez** con el hub caido. Es la misma clase de deuda que ya
arrastramos con `setup_smoke.ps1`, y esta vez se evito en vez de aceptarse.

## Estado por AC

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | `corpus()` recorre `docs/**` + `history.md` y excluye `bkp/`; probado con un `bkp/viejo.md` real que menciona el termino y no aparece |
| AC-2 | cubierto | AND por defecto; caida a parcial marcada en el tipo (`Resultado.parcial`) y avisada en la salida |
| AC-3 | cubierto | Consulta vacia => exit 2 con la forma de uso |
| AC-4 | cubierto | Orden leccion > adr > impl verificado en integracion sobre corpus sembrado |
| AC-5 | cubierto | Encabezados y campos de frontmatter pesan mas (dos tests separados) |
| AC-6 | cubierto | Frase contigua > terminos dispersos |
| AC-7 | cubierto | `--json` expone `score`: el ranking es auditable |
| AC-8 | cubierto | `archivo:linea [fuente #feature fecha]` + texto recortado por caracteres |
| AC-9 | cubierto | 30 resultados => 20 + "10 resultado(s) mas"; con `--todos` desaparece |
| AC-10 | cubierto | Sin coincidencias: mensaje, sugerencia y **exit 0** |
| AC-11 | cubierto | `--json` valido sin resultados (`total: 0`) |
| AC-12 | cubierto | **~10 ms** medidos en 5 corridas sobre 114 archivos / 1,1 MB; sin archivo de indice |
| AC-13 | cubierto | `git diff` de `Cargo.toml`/`Cargo.lock` vacio |
| AC-14 | cubierto | Mismo **stdout completo** con el hub sano y con el hub apuntando a un puerto muerto |
| AC-15 | cubierto | Archivo ilegible se saltea sin abortar |
| AC-16 | cubierto | Sin `docs/`: lo dice y sale 0 |
| AC-17 | cubierto | README, UPDATING (+ espejo), architecture y ambas superficies |
| AC-18 | cubierto | Lider (4.0), implementer (1.5) y reviewer (citas verificables) |
| AC-19 | cubierto | 194 unit + 73 integracion, clippy limpio, smoke exit 0, check limpio |

## Los tres criterios extra del plan

| Criterio | Resultado |
| --- | --- |
| La consulta real ("¿donde decidimos usar ureq?") devuelve el ADR primero | **Cumplido, despues de fallar.** Ver abajo |
| SLO medido y publicado | **~10 ms** sobre el corpus real, 5 corridas |
| No escribe nada | `find docs progress -newermt '-5 seconds' -type f` => **0** |

## Constitution

| Articulo | Verificacion |
| --- | --- |
| 1 - Calidad y tests | 194 unit + 73 integracion (30 nuevos), clippy `-D warnings`, smoke exit 0 |
| 2 - Spec aprobado | Sello + history + gates verdes |
| 3 - Trazabilidad AC-n | Cada D cita sus AC; evidencia y veredicto por AC |
| 4 - Seguridad y observabilidad | La consulta del usuario **nunca** se compila a regex ni se interpola en un comando: se compara como texto, asi que no hay ReDoS ni inyeccion. Exit 0/2 estables |
| 5 - Decisiones del usuario | Las 5 OBS decididas antes de implementar |
| 6 - Reglas puente | **Sin dependencias nuevas**; `templates/` espejado; sin LLM (el ranking es aritmetica pura, auditable en `--json`) |

## Lo que el reviewer destaca

**El criterio de cierre hizo su trabajo: fallo.** El plan exigia que la consulta
que motivo la feature devolviera el ADR primero, y la primera corrida lo puso en
el **puesto 10**, debajo de un ejemplo de nombre malo (`arreglo-ureq`) sacado de
la guia de lecciones. Eso destapo dos errores de clasificacion reales:

1. La **guia** de lecciones cobraba el peso del conocimiento curado (100) siendo
   una plantilla del arnes. Sus ejemplos le ganaban a las decisiones reales.
2. Un **ADR** —una decision tecnica con nombre propio y sin vencimiento— pesaba
   como un doc generico (40).

Los dos quedaron corregidos y con test. Vale subrayar el metodo: **un criterio de
cierre que se puede fallar es lo unico que distingue una verificacion de una
afirmacion.** Si el plan hubiera dicho "el ranking debe ser razonable", esta
feature se cerraba con el ADR en el puesto 10.

## Riesgos que quedan abiertos

1. **El ranking es heuristico y sin datos de uso.** Los pesos estan razonados y
   probados contra una consulta conocida, pero son una hipotesis. Mitigacion: el
   `score` es auditable y los pesos viven en un solo `match`. Vale revisarlo
   cuando haya varias consultas reales encima.
2. **Sin plegado de acentos.** Limite conocido y declarado (este repo escribe sin
   acentos por convencion); `buscar "decision"` no encontraria "decisión".
3. **`setup_smoke.ps1` sin ejecutar**, igual que en #17, #18 y #19. Esta feature
   casi no toca el instalador (solo la superficie), asi que la brecha no crece.
4. **Solapamiento aparente con `perfil::recolectar`.** El plan documenta por que
   NO se comparte codigo (filtran cosas distintas y van a divergir). Queda
   escrito para que una futura "simplificacion" no las fusione sin pensar.

## Nota sobre la declaracion de cierre

La feature deja `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`: la
diferencia entre un criterio verificable y uno decorativo, con el caso concreto
de esta feature (el ADR en el puesto 10) como pitfall. Es de clase, aplica a toda
feature futura del repo, y sale de algo que **paso de verdad en esta sesion**, no
de una narrativa.
